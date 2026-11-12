//! Directory links: symlinks on Unix, junctions on Windows (no admin rights needed).

use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use crate::version::Version;

/// Points `link` at `target`, replacing whatever link was there.
pub fn create(target: &Path, link: &Path) -> io::Result<()> {
    if let Some(parent) = link.parent() {
        fs::create_dir_all(parent)?;
    }
    replace(target, link)
}

#[cfg(unix)]
fn replace(target: &Path, link: &Path) -> io::Result<()> {
    // Create under a temporary name and rename over the old link, so it is never missing.
    let mut tmp = link.as_os_str().to_owned();
    tmp.push(format!(".tmp{}", std::process::id()));
    let tmp = PathBuf::from(tmp);
    let _ = fs::remove_file(&tmp);
    std::os::unix::fs::symlink(target, &tmp)?;
    fs::rename(&tmp, link).inspect_err(|_| {
        let _ = fs::remove_file(&tmp);
    })
}

#[cfg(windows)]
fn replace(target: &Path, link: &Path) -> io::Result<()> {
    remove(link)?;
    junction::create(target, link)
}

/// Removes the link itself, never the directory it points to.
pub fn remove(link: &Path) -> io::Result<()> {
    if fs::symlink_metadata(link).is_err() {
        return Ok(());
    }
    if cfg!(windows) {
        fs::remove_dir(link)
    } else {
        fs::remove_file(link)
    }
}

pub fn target(link: &Path) -> Option<PathBuf> {
    fs::read_link(link).ok()
}

/// Version a link points to. Works for links to a version directory and to its `bin` directory.
pub fn version(link: &Path) -> Option<Version> {
    let target = target(link)?;
    target
        .ancestors()
        .take(2)
        .filter_map(|p| p.file_name())
        .find_map(|name| Version::parse(&name.to_string_lossy()))
}
