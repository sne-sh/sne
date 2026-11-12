//! Archive extraction. Both formats drop the archive's single top-level directory.

use std::path::{Component, Path, PathBuf};

/// Removes the first path component and rejects anything that could escape the destination.
fn strip_top_level(path: &Path) -> Option<PathBuf> {
    let mut components = path.components();
    components.next()?;
    let rest = components.as_path();
    if rest.as_os_str().is_empty() || !rest.components().all(|c| matches!(c, Component::Normal(_))) {
        return None;
    }
    Some(rest.to_path_buf())
}

/// Unpacks a `.tar.xz` stream as it arrives, then hands the reader back fully drained.
#[cfg(not(windows))]
pub fn untar_xz<R: std::io::Read>(reader: R, dest: &Path) -> anyhow::Result<R> {
    use std::{fs, io};

    let mut archive = tar::Archive::new(liblzma::read::XzDecoder::new(reader));
    archive.set_preserve_permissions(true);
    for entry in archive.entries()? {
        let mut entry = entry?;
        let Some(relative) = strip_top_level(&entry.path()?) else {
            continue;
        };
        let out = dest.join(relative);
        if let Some(parent) = out.parent() {
            fs::create_dir_all(parent)?;
        }
        entry.unpack(&out)?;
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
    }
}
