use std::env;
use std::path::PathBuf;

use crate::version::Version;

/// Root of everything sne stores. Override with `SNE_DIR`.
pub fn base() -> PathBuf {
    if let Some(dir) = env::var_os("SNE_DIR").filter(|d| !d.is_empty()) {
        return PathBuf::from(dir);
    }
    if cfg!(windows) {
        if let Some(local) = env::var_os("LOCALAPPDATA").filter(|d| !d.is_empty()) {
            return PathBuf::from(local).join("sne");
        }
        let profile = env::var_os("USERPROFILE").unwrap_or_default();
        return PathBuf::from(profile).join("AppData").join("Local").join("sne");
    }
    if let Some(data) = env::var_os("XDG_DATA_HOME").filter(|d| !d.is_empty()) {
        return PathBuf::from(data).join("sne");
    }
    let home = env::var_os("HOME").unwrap_or_default();
    PathBuf::from(home).join(".local").join("share").join("sne")
}

pub fn versions() -> PathBuf {
    base().join("versions")
}

pub fn aliases() -> PathBuf {
    base().join("aliases")
}

pub fn cache() -> PathBuf {
    base().join("cache")
}

/// Per-shell links created by `sne env`.
pub fn shells() -> PathBuf {
    base().join("shells")
}

pub fn version_dir(v: Version) -> PathBuf {
    versions().join(v.to_string())
}

/// Directory that holds `node`, `npm` and `npx` for an installed version.
pub fn bin_dir(v: Version) -> PathBuf {
    let dir = version_dir(v);
    if cfg!(windows) { dir } else { dir.join("bin") }
}

pub fn node_exe() -> &'static str {
    if cfg!(windows) { "node.exe" } else { "node" }
}

/// Installed versions, newest first.
pub fn installed() -> Vec<Version> {
    let mut found: Vec<Version> = std::fs::read_dir(versions())
        .into_iter()
        .flatten()
        .flatten()
        .filter_map(|e| Version::parse(&e.file_name().to_string_lossy()))
        .collect();
    found.sort_unstable_by(|a, b| b.cmp(a));
    found
}
