use std::fs;
use std::io::Read;
use std::path::Path;
use std::time::{Duration, SystemTime};

use anyhow::{Context, Result, anyhow, bail};
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};

use super::{default_link, report_default_path, shell_link};
use crate::download::{self, HashReader};
use crate::resolve::Spec;
use crate::version::Version;
use crate::{dirs, extract, index, link, platform, version_file};

/// Versions downloaded at the same time by `sne install a b c`.
const PARALLEL_INSTALLS: usize = 4;

pub fn run(specs: Vec<String>) -> Result<()> {
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

    let (os, arch, musl) = (platform::os(), platform::arch(), platform::is_musl());
    let key = platform::index_key(os, arch, musl);
    let releases = index::load()?;
    let mut versions: Vec<Version> = Vec::new();
    for input in &specs {
        let available = releases
            .iter()
            .filter(|r| r.files.is_empty() || r.files.contains(&key))
            .map(|r| (r.version, r.lts.as_deref()));
        let version = Spec::parse(input)?
            .pick(available)
            .ok_or_else(|| anyhow!("no Node release matches `{input}` for {os}-{arch}"))?;
        if !versions.contains(&version) {
            versions.push(version);
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
                .map(|&v| {
                    let progress = &progress;
                    scope.spawn(move || install_one(v, progress))
                })
                .collect();
            workers
                .into_iter()
                .map(|w| w.join().expect("install thread panicked"))
                .collect()
        });
        for (version, result) in batch.iter().zip(results) {
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

    // The first version ever installed becomes the default, so new shells have a Node.
    if link::version(&default_link()).is_none()
        && let Some(&first) = versions.iter().find(|v| dirs::version_dir(**v).is_dir())
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
fn install_one(version: Version, progress: &MultiProgress) -> Result<bool> {
    let dest = dirs::version_dir(version);
    if dest.is_dir() {
        return Ok(false);
    }
    let file = platform::archive_name(platform::os(), platform::arch(), platform::is_musl(), version);
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
fn remove_stale_temp_dirs() {
    let Ok(entries) = fs::read_dir(dirs::versions()) else {
        return;
    };
    for entry in entries.flatten() {
        if !entry.file_name().to_string_lossy().starts_with(".tmp-") {
            continue;
        }
        let old = entry
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
