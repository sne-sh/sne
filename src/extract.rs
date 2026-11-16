//! Archive extraction. Both formats drop the archive's single top-level directory.

use std::path::{Component, Path, PathBuf};

/// Removes the first path component and rejects anything that could escape the destination.
fn strip_top_level(path: &Path) -> Option<PathBuf> {
    let mut components = path.components();
    // An absolute path, or one that starts with `..`, is not from a well-formed archive.
    if !matches!(components.next()?, Component::Normal(_)) {
        return None;
    }
    let rest = components.as_path();
    if rest.as_os_str().is_empty() || !rest.components().all(|c| matches!(c, Component::Normal(_))) {
        return None;
    }
    Some(rest.to_path_buf())
}

/// Whether a symbolic link at `link` (relative to the destination) pointing to `target` stays
/// inside the destination. The target may start with `..` steps, which are counted against the
/// link's own folder, and then only name folders and files. A `..` after a name is refused, because
/// that name could itself be a link, and then `..` would not mean what it seems to.
#[cfg(any(not(windows), test))]
fn link_stays_inside(link: &Path, target: &Path) -> bool {
    let mut depth = link.components().count() as isize - 1;
    let mut named = false;
    for part in target.components() {
        match part {
            Component::CurDir => {}
            Component::ParentDir if !named => {
                depth -= 1;
                if depth < 0 {
                    return false;
                }
            }
            Component::Normal(_) => named = true,
            Component::ParentDir | Component::RootDir | Component::Prefix(_) => return false,
        }
    }
    true
}

/// Unpacks a `.tar.xz` stream as it arrives, then hands the reader back fully drained.
///
/// The download is checked against its SHA-256 only after it has been unpacked, so this must be
/// safe for any archive, including a damaged or altered one: everything stays inside `dest`.
/// A tar file can hold links, and a link followed by a file "inside" it would write elsewhere.
/// So links must point inside `dest`, and nothing is ever written through one.
#[cfg(not(windows))]
pub fn untar_xz<R: std::io::Read>(reader: R, dest: &Path) -> anyhow::Result<R> {
    use std::collections::HashSet;
    use std::{fs, io};

    use anyhow::{Context, bail};
    use tar::EntryType;

    let mut archive = tar::Archive::new(liblzma::read::XzDecoder::new(reader));
    // Keeps the read, write and execute bits, and drops setuid, setgid and sticky bits. Those
    // matter when sne runs as root, for example in a container.
    archive.set_preserve_permissions(false);
    archive.set_unpack_xattrs(false);

    // Folders already known to be real folders, not links, so each is looked at once.
    let mut real_dirs: HashSet<PathBuf> = HashSet::new();
    let mut no_link_on_the_way = |relative: &Path| -> anyhow::Result<()> {
        let mut at = dest.to_path_buf();
        let count = relative.components().count();
        for (i, part) in relative.components().enumerate() {
            at.push(part);
            let is_last = i + 1 == count;
            if !is_last && real_dirs.contains(&at) {
                continue;
            }
            match fs::symlink_metadata(&at) {
                Ok(meta) if meta.file_type().is_symlink() => {
                    bail!("the archive writes through the link {}", at.display())
                }
                Ok(meta) if meta.is_dir() && !is_last => {
                    real_dirs.insert(at.clone());
                }
                // A folder that does not exist yet is created by sne, so it is a real one.
                _ => {}
            }
        }
        Ok(())
    };

    for entry in archive.entries()? {
        let mut entry = entry?;
        let path = entry.path()?.into_owned();
        let Some(relative) = strip_top_level(&path) else {
            continue;
        };
        let kind = entry.header().entry_type();
        // Device files and pipes have no place in a Node.js archive.
        if !matches!(
            kind,
            EntryType::Regular
                | EntryType::Continuous
                | EntryType::GNUSparse
                | EntryType::Directory
                | EntryType::Symlink
                | EntryType::Link
        ) {
            continue;
        }
        no_link_on_the_way(&relative)?;
        let out = dest.join(&relative);
        if let Some(parent) = out.parent() {
            fs::create_dir_all(parent)?;
        }
        match kind {
            EntryType::Symlink | EntryType::Link => {
                if fs::symlink_metadata(&out).is_ok() {
                    bail!("the archive has two entries for {}", relative.display());
                }
                let target = entry
                    .link_name()?
                    .with_context(|| format!("the link {} has no target", relative.display()))?
                    .into_owned();
                if kind == EntryType::Symlink {
                    if !link_stays_inside(&relative, &target) {
                        bail!(
                            "the archive has a link that points outside the install folder: {} -> {}",
                            relative.display(),
                            target.display()
                        );
                    }
                    entry.unpack(&out)?;
                } else {
                    // A hard link names another file of the same archive.
                    let Some(inner) = strip_top_level(&target) else {
                        bail!(
                            "the archive has a hard link to a file outside it: {} -> {}",
                            relative.display(),
                            target.display()
                        );
                    };
                    no_link_on_the_way(&inner)?;
                    fs::hard_link(dest.join(inner), &out)?;
                }
            }
            _ => {
                entry.unpack(&out)?;
            }
        }
    }
    let mut decoder = archive.into_inner();
    io::copy(&mut decoder, &mut io::sink())?;
    Ok(decoder.into_inner())
}

/// Unpacks a `.zip` file using several threads, each with its own handle on the archive.
#[cfg(windows)]
pub fn unzip(archive: &Path, dest: &Path) -> anyhow::Result<()> {
    use std::fs::{self, File};
    use std::io::{self, BufReader};

    let open = || -> anyhow::Result<zip::ZipArchive<BufReader<File>>> {
        Ok(zip::ZipArchive::new(BufReader::new(File::open(archive)?))?)
    };
    let entries = open()?.len();
    let threads = std::thread::available_parallelism()
        .map_or(4, |n| n.get())
        .min(8)
        .min(entries.max(1));

    std::thread::scope(|scope| {
        let workers: Vec<_> = (0..threads)
            .map(|worker| {
                scope.spawn(move || -> anyhow::Result<()> {
                    let mut zip = open()?;
                    for i in (worker..entries).step_by(threads) {
                        let mut file = zip.by_index(i)?;
                        let Some(relative) = file.enclosed_name().as_deref().and_then(strip_top_level) else {
                            continue;
                        };
                        let out = dest.join(relative);
                        if file.is_dir() {
                            fs::create_dir_all(&out)?;
                            continue;
                        }
                        if let Some(parent) = out.parent() {
                            fs::create_dir_all(parent)?;
                        }
                        io::copy(&mut file, &mut File::create(&out)?)?;
                    }
                    Ok(())
                })
            })
            .collect();
        for worker in workers {
            worker.join().expect("extraction thread panicked")?;
        }
        Ok(())
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strips_the_top_directory_and_blocks_escapes() {
        assert_eq!(
            strip_top_level(Path::new("node-v1/bin/node")),
            Some(PathBuf::from("bin/node"))
        );
        assert_eq!(strip_top_level(Path::new("node-v1")), None);
        assert_eq!(strip_top_level(Path::new("node-v1/../../etc/passwd")), None);
        assert_eq!(strip_top_level(Path::new("/etc/passwd")), None);
        assert_eq!(strip_top_level(Path::new("../node-v1/bin/node")), None);
    }

    #[test]
    fn links_must_point_inside() {
        let inside = |link: &str, target: &str| link_stays_inside(Path::new(link), Path::new(target));
        // The links in a real Node.js archive.
        assert!(inside("bin/npm", "../lib/node_modules/npm/bin/npm-cli.js"));
        assert!(inside(
            "bin/corepack",
            "../lib/node_modules/corepack/dist/corepack.js"
        ));
        assert!(inside("lib/x", "./y"));

        assert!(!inside("bin/npm", "../../outside"));
        assert!(!inside("npm", "../outside"));
        assert!(!inside("bin/x", "/etc/passwd"));
        // `here` may be a link to `.`, and then `here/..` is the folder above the install.
        assert!(!inside("bin/x", "../here/../.."));
        assert!(!inside("x", "here/.."));
    }

    #[cfg(not(windows))]
    mod tar_files {
        use super::super::untar_xz;
        use std::fs;
        use std::path::Path;

        enum Item<'a> {
            File(&'a str, &'a [u8], u32),
            Dir(&'a str),
            Symlink(&'a str, &'a str),
            HardLink(&'a str, &'a str),
        }

        fn archive(items: &[Item]) -> Vec<u8> {
            let mut tar = tar::Builder::new(liblzma::write::XzEncoder::new(Vec::new(), 1));
            for item in items {
                let mut header = tar::Header::new_gnu();
                match *item {
                    Item::File(path, body, mode) => {
                        header.set_entry_type(tar::EntryType::Regular);
                        header.set_size(body.len() as u64);
                        header.set_mode(mode);
                        header.set_cksum();
                        tar.append_data(&mut header, path, body).unwrap();
                    }
                    Item::Dir(path) => {
                        header.set_entry_type(tar::EntryType::Directory);
                        header.set_size(0);
                        header.set_mode(0o755);
                        header.set_cksum();
                        tar.append_data(&mut header, path, &[][..]).unwrap();
                    }
                    Item::Symlink(path, target) | Item::HardLink(path, target) => {
                        header.set_entry_type(if matches!(item, Item::Symlink(..)) {
                            tar::EntryType::Symlink
                        } else {
                            tar::EntryType::Link
                        });
                        header.set_size(0);
                        header.set_mode(0o777);
                        tar.append_link(&mut header, path, target).unwrap();
                    }
                }
            }
            tar.into_inner().unwrap().finish().unwrap()
        }

        fn unpack(items: &[Item]) -> (tempdir::Dir, anyhow::Result<()>) {
            let dir = tempdir::Dir::new();
            fs::create_dir_all(dir.path().join("dest")).unwrap();
            let bytes = archive(items);
            let result = untar_xz(&bytes[..], &dir.path().join("dest")).map(|_| ());
            (dir, result)
        }

        mod tempdir {
            use std::path::{Path, PathBuf};
            pub struct Dir(PathBuf);
            impl Dir {
                pub fn new() -> Self {
                    use std::sync::atomic::{AtomicUsize, Ordering};
                    static N: AtomicUsize = AtomicUsize::new(0);
                    let path = std::env::temp_dir().join(format!(
                        "sne-extract-test-{}-{}",
                        std::process::id(),
                        N.fetch_add(1, Ordering::SeqCst)
                    ));
                    let _ = std::fs::remove_dir_all(&path);
                    std::fs::create_dir_all(&path).unwrap();
                    Dir(path)
                }
                pub fn path(&self) -> &Path {
                    &self.0
                }
            }
            impl Drop for Dir {
                fn drop(&mut self) {
                    let _ = std::fs::remove_dir_all(&self.0);
                }
            }
        }

        fn mode(path: &Path) -> u32 {
            use std::os::unix::fs::PermissionsExt;
            fs::metadata(path).unwrap().permissions().mode() & 0o7777
        }

        #[test]
        fn unpacks_a_node_like_archive() {
            let (dir, result) = unpack(&[
                Item::Dir("node-v1/bin"),
                Item::File("node-v1/bin/node", b"node", 0o4755),
                Item::File("node-v1/lib/node_modules/npm/bin/npm-cli.js", b"npm", 0o755),
                Item::Symlink("node-v1/bin/npm", "../lib/node_modules/npm/bin/npm-cli.js"),
                Item::HardLink("node-v1/bin/node2", "node-v1/bin/node"),
            ]);
            result.unwrap();
            let dest = dir.path().join("dest");
            assert_eq!(fs::read(dest.join("bin/npm")).unwrap(), b"npm");
            assert_eq!(fs::read(dest.join("bin/node2")).unwrap(), b"node");
            // Executable, but never setuid.
            assert_eq!(mode(&dest.join("bin/node")), 0o755);
        }

        #[test]
        fn refuses_to_write_through_a_link() {
            let (dir, result) = unpack(&[
                Item::Symlink("node-v1/out", ".."),
                Item::File("node-v1/out/escaped", b"x", 0o644),
            ]);
            assert!(result.is_err());
            assert!(!dir.path().join("escaped").exists());

            // A link that looks fine on its own, used to reach the folder above.
            let (dir, result) = unpack(&[
                Item::Symlink("node-v1/here", "."),
                Item::Symlink("node-v1/up", "here/.."),
                Item::File("node-v1/up/escaped", b"x", 0o644),
            ]);
            assert!(result.is_err());
            assert!(!dir.path().join("escaped").exists());

            // Even a link that points inside is never written through.
            let (dir, result) = unpack(&[
                Item::Dir("node-v1/lib"),
                Item::Symlink("node-v1/alias", "lib"),
                Item::File("node-v1/alias/file", b"x", 0o644),
            ]);
            assert!(result.is_err());
            assert!(!dir.path().join("dest/lib/file").exists());
        }

        #[test]
        fn refuses_links_that_leave() {
            let (_dir, result) = unpack(&[Item::Symlink("node-v1/bin/x", "/etc/passwd")]);
            assert!(result.is_err());
            let (_dir, result) = unpack(&[Item::HardLink("node-v1/x", "/etc/passwd")]);
            assert!(result.is_err());
            let (_dir, result) = unpack(&[Item::HardLink("node-v1/x", "node-v1/../../secret")]);
            assert!(result.is_err());
        }

        #[test]
        fn refuses_to_replace_a_file_with_a_link() {
            let (dir, result) = unpack(&[
                Item::File("node-v1/bin/node", b"node", 0o755),
                Item::Symlink("node-v1/bin/node", "../../../somewhere"),
            ]);
            assert!(result.is_err());
            assert_eq!(fs::read(dir.path().join("dest/bin/node")).unwrap(), b"node");
        }
    }
}
