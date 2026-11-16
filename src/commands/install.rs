use std::fs;
use std::io::Read;
use std::path::Path;
use std::time::{Duration, SystemTime};

use anyhow::{Context, Result, bail};
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};

use super::{default_link, report_default_path, shell_link};
use crate::download::{self, HashReader};
use crate::resolve::Spec;
use crate::version::Version;
use crate::{dirs, extract, index, link, platform, version_file};

/// Versions downloaded at the same time by `sne install a b c`.
const PARALLEL_INSTALLS: usize = 4;

/// `reinstall_from` names an installed version whose global npm packages are installed again
/// into each version installed here.
pub fn run(specs: Vec<String>, reinstall_from: Option<String>) -> Result<()> {
    // A mistyped version should stop before anything is downloaded.
    let reinstall_from = reinstall_from
        .as_deref()
        .map(super::resolve_installed)
        .transpose()
        .context("--reinstall-packages-from must name an installed version")?;
    // Runs beside the install and never delays it.
    super::update::refresh_in_background();
    let specs = if specs.is_empty() {
        let cwd = std::env::current_dir()?;
        let found = version_file::find(&cwd).context(
            "no version given, and no .nvmrc, .node-version or package.json `engines.node` was found",
        )?;
        println!("Found `{}` in {}", found.spec, found.path.display());
        vec![found.spec]
    } else {
        specs
    };

    let (os, arch, musl) = (platform::os(), platform::arch()?, platform::is_musl());
    // A build that runs under emulation is a fallback, never chosen over the user's SNE_ARCH.
    let fallback = platform::emulated_arch(os, arch).filter(|_| !platform::arch_is_chosen());
    let releases = index::load()?;
    let pick = |spec: &Spec, arch: &str| {
        let key = platform::index_key(os, arch, musl);
        let available = releases
            .iter()
            .filter(|r| r.files.is_empty() || r.files.contains(&key))
            .map(|r| (r.version, r.lts.as_deref()));
        spec.pick(available)
    };
    let mut versions: Vec<(Version, &'static str)> = Vec::new();
    for input in &specs {
        let spec = Spec::parse(input)?;
        let (version, build) = match (pick(&spec, arch), fallback) {
            (Some(version), _) => (version, arch),
            (None, Some(other)) => match pick(&spec, other) {
                Some(version) => {
                    println!(
                        "Node {version} has no {os}-{arch} build. Installing the {other} build, \
                         which this system runs under emulation."
                    );
                    (version, other)
                }
                None => bail!("no Node release matches `{input}` for {os}-{arch}"),
            },
            (None, None) => bail!("no Node release matches `{input}` for {os}-{arch}"),
        };
        if !versions.iter().any(|(v, _)| *v == version) {
            versions.push((version, build));
        }
    }

    fs::create_dir_all(dirs::versions())?;
    remove_stale_temp_dirs();

    let progress = MultiProgress::new();
    let mut failures = 0;
    for batch in versions.chunks(PARALLEL_INSTALLS) {
        let results: Vec<Result<bool>> = std::thread::scope(|scope| {
            let workers: Vec<_> = batch
                .iter()
                .map(|&(v, build)| {
                    let progress = &progress;
                    scope.spawn(move || install_one(v, build, progress))
                })
                .collect();
            workers
                .into_iter()
                .map(|w| w.join().expect("install thread panicked"))
                .collect()
        });
        for ((version, _), result) in batch.iter().zip(results) {
            match result {
                Ok(true) => println!("Installed Node {version}"),
                Ok(false) => println!("Node {version} is already installed"),
                Err(err) => {
                    failures += 1;
                    eprintln!("sne: {err:#}");
                }
            }
        }
    }

    if let Some(from) = reinstall_from {
        for &(version, _) in &versions {
            if version == from || !dirs::version_dir(version).is_dir() {
                continue;
            }
            if let Err(err) = reinstall_packages(from, version) {
                failures += 1;
                eprintln!("sne: {err:#}");
            }
        }
    }

    // The first version ever installed becomes the default, so new shells have a Node.
    if link::version(&default_link()).is_none()
        && let Some(&(first, _)) = versions.iter().find(|(v, _)| dirs::version_dir(*v).is_dir())
    {
        link::create(&dirs::version_dir(first), &default_link())?;
        println!("Node {first} is now the default");
        if shell_link().is_none() {
            report_default_path();
        }
    }

    if failures > 0 {
        bail!("{failures} of {} installs failed", versions.len());
    }
    super::update::hint();
    super::update::enable_by_default();
    Ok(())
}

/// Returns `false` when the version was already installed.
/// Installs the global npm packages of `from` into `to`, at the same versions.
fn reinstall_packages(from: Version, to: Version) -> Result<()> {
    let (packages, skipped) = global_packages(&global_modules(from));
    for name in &skipped {
        println!("Skipping {name}: it was added with `npm link`, or its name or version is not one npm uses");
    }
    if packages.is_empty() {
        println!("Node {from} has no global npm packages to install into Node {to}");
        return Ok(());
    }
    println!(
        "Installing the global packages of Node {from} into Node {to}: {}",
        packages.join(" ")
    );
    let mut command = vec!["npm".to_string(), "install".to_string(), "--global".to_string()];
    command.extend(packages);
    let code = super::switch::exec(&to.to_string(), &command)?;
    if code != 0 {
        bail!("npm could not install the global packages of Node {from} into Node {to} (exit code {code})");
    }
    Ok(())
}

/// Where npm keeps the global packages of an installed version.
fn global_modules(version: Version) -> std::path::PathBuf {
    let dir = dirs::version_dir(version);
    if cfg!(windows) {
        dir.join("node_modules")
    } else {
        dir.join("lib").join("node_modules")
    }
}

fn is_link(path: &Path) -> bool {
    fs::symlink_metadata(path).is_ok_and(|m| m.file_type().is_symlink())
}

/// The global packages in `root` as `name@version`, sorted, and the names that were skipped.
///
/// The folder is read instead of asking npm, which is faster and works even when that npm does
/// not. npm and corepack come with Node itself. A package added with `npm link` is a link to a
/// folder of the user's, which cannot be installed from the registry. Names and versions must use
/// only the characters npm allows, because they are passed to npm on its command line.
fn global_packages(root: &Path) -> (Vec<String>, Vec<String>) {
    let (mut found, mut skipped) = (Vec::new(), Vec::new());
    let name_ok = |c: char| c.is_ascii_alphanumeric() || matches!(c, '-' | '.' | '_' | '~' | '/' | '@');
    let version_ok = |c: char| c.is_ascii_alphanumeric() || matches!(c, '-' | '.' | '+');
    let mut consider = |name: String, dir: &Path| {
        if matches!(name.as_str(), "npm" | "corepack") || name.starts_with('.') {
            return;
        }
        let version = fs::read_to_string(dir.join("package.json"))
            .ok()
            .and_then(|text| serde_json::from_str::<serde_json::Value>(&text).ok())
            .and_then(|manifest| manifest["version"].as_str().map(str::to_string));
        match version {
            Some(version)
                if !is_link(dir)
                    && name.chars().all(name_ok)
                    && !version.is_empty()
                    && version.chars().all(version_ok) =>
            {
                found.push(format!("{name}@{version}"));
            }
            _ => skipped.push(name),
        }
    };
    for entry in fs::read_dir(root).into_iter().flatten().flatten() {
        let name = entry.file_name().to_string_lossy().into_owned();
        // Scoped packages, such as @angular/cli, sit one folder deeper.
        if name.starts_with('@') && !is_link(&entry.path()) {
            for inner in fs::read_dir(entry.path()).into_iter().flatten().flatten() {
                consider(
                    format!("{name}/{}", inner.file_name().to_string_lossy()),
                    &inner.path(),
                );
            }
        } else {
            consider(name, &entry.path());
        }
    }
    found.sort();
    skipped.sort();
    (found, skipped)
}

/// `arch` is the processor type of the build, which differs from this system's under emulation.
fn install_one(version: Version, arch: &str, progress: &MultiProgress) -> Result<bool> {
    let dest = dirs::version_dir(version);
    if dest.is_dir() {
        return Ok(false);
    }
    let file = platform::archive_name(platform::os(), arch, platform::is_musl(), version);
    let release_url = format!("{}/{version}", platform::mirror());

    let install = || -> Result<()> {
        let expected = download::expected_sha256(&release_url, &file)?;
        let (body, length) = download::open(&format!("{release_url}/{file}"))?;
        let bar = progress.add(new_bar(version, length));
        let temp = dirs::versions().join(format!(".tmp-{version}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&temp);
        let result = unpack(HashReader::new(body, bar.clone()), &temp, &expected)
            .and_then(|()| move_into_place(&temp, &dest));
        bar.finish_and_clear();
        if result.is_err() {
            let _ = fs::remove_dir_all(&temp);
        }
        result
    };
    install().with_context(|| format!("could not install Node {version}"))?;
    Ok(true)
}

fn new_bar(version: Version, length: Option<u64>) -> ProgressBar {
    let bar = match length {
        Some(length) => ProgressBar::new(length).with_style(
            ProgressStyle::with_template("{msg} [{bar:30}] {bytes}/{total_bytes} {bytes_per_sec}")
                .expect("valid template")
                .progress_chars("=> "),
        ),
        None => ProgressBar::new_spinner(),
    };
    bar.with_message(version.to_string())
}

fn verify(actual: &str, expected: &str) -> Result<()> {
    if actual != expected {
        bail!("checksum mismatch: expected {expected}, got {actual}. Nothing was installed");
    }
    Ok(())
}

/// Decompresses and unpacks while the download is still arriving; no archive is written to disk.
#[cfg(not(windows))]
fn unpack<R: Read>(reader: HashReader<R>, temp: &Path, expected: &str) -> Result<()> {
    fs::create_dir_all(temp)?;
    let reader = extract::untar_xz(reader, temp)?;
    verify(&reader.finish()?, expected)
}

/// Zip needs the end of the file first, so it is saved, verified, then unpacked on several threads.
#[cfg(windows)]
fn unpack<R: Read>(mut reader: HashReader<R>, temp: &Path, expected: &str) -> Result<()> {
    let archive = std::path::PathBuf::from(format!("{}.zip", temp.display()));
    let result = (|| {
        std::io::copy(&mut reader, &mut fs::File::create(&archive)?)?;
        verify(&reader.finish()?, expected)?;
        fs::create_dir_all(temp)?;
        extract::unzip(&archive, temp)
    })();
    let _ = fs::remove_file(&archive);
    result
}

/// Atomic rename. Retries because antivirus scanners on Windows briefly lock new files.
fn move_into_place(temp: &Path, dest: &Path) -> Result<()> {
    let mut attempt = 0;
    loop {
        match fs::rename(temp, dest) {
            Ok(()) => return Ok(()),
            // Another sne process installed the same version first.
            Err(_) if dest.is_dir() => {
                let _ = fs::remove_dir_all(temp);
                return Ok(());
            }
            Err(err) if attempt == 20 => return Err(err).context("could not move the version into place"),
            Err(_) => {
                attempt += 1;
                std::thread::sleep(Duration::from_millis(100));
            }
        }
    }
}

/// Leftovers from installs that were interrupted more than an hour ago.
/// Also removes versions that `sne uninstall` moved aside but could not delete yet, because a
/// program was still running from them; those are never in use by sne, so they go at once.
pub(super) fn remove_stale_temp_dirs() {
    let Ok(entries) = fs::read_dir(dirs::versions()) else {
        return;
    };
    for entry in entries.flatten() {
        let name = entry.file_name().to_string_lossy().into_owned();
        if !name.starts_with(".tmp-") {
            continue;
        }
        let old = name.starts_with(".tmp-remove-")
            || entry
                .metadata()
                .and_then(|m| m.modified())
                .ok()
                .and_then(|m| SystemTime::now().duration_since(m).ok())
                .is_some_and(|age| age > Duration::from_secs(60 * 60));
        if old {
            let path = entry.path();
            let _ = if path.is_dir() {
                fs::remove_dir_all(&path)
            } else {
                fs::remove_file(&path)
            };
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_global_packages_to_install_again() {
        let root = std::env::temp_dir().join(format!("sne-globals-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        let package = |path: &str, json: &str| {
            let dir = root.join(path);
            fs::create_dir_all(&dir).unwrap();
            fs::write(dir.join("package.json"), json).unwrap();
        };
        package("npm", r#"{"version":"10.9.0"}"#);
        package("corepack", r#"{"version":"0.29.4"}"#);
        package("typescript", r#"{"name":"typescript","version":"5.6.3"}"#);
        package("@angular/cli", r#"{"version":"18.2.11"}"#);
        package("JSONStream", r#"{"version":"1.3.5"}"#);
        package("bad&name", r#"{"version":"1.0.0"}"#);
        package("odd-version", r#"{"version":"1.0.0 && calc"}"#);
        package("no-version", r#"{"name":"no-version"}"#);
        fs::create_dir_all(root.join(".bin")).unwrap();

        let (found, skipped) = global_packages(&root);
        let _ = fs::remove_dir_all(&root);
        assert_eq!(
            found,
            ["@angular/cli@18.2.11", "JSONStream@1.3.5", "typescript@5.6.3"]
        );
        assert_eq!(skipped, ["bad&name", "no-version", "odd-version"]);
    }
}
