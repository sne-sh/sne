pub mod env;
pub mod install;
pub mod list;
pub mod switch;

use std::path::PathBuf;

use anyhow::{Context, Result, anyhow};

use crate::resolve::Spec;
use crate::version::Version;
use crate::{dirs, index, link};

pub fn default_link() -> PathBuf {
    dirs::aliases().join("default")
}

/// The per-shell link that `sne env` put on `PATH`.
pub fn shell_link() -> Option<PathBuf> {
    std::env::var_os("SNE_SHELL_LINK")
        .filter(|v| !v.is_empty())
        .map(PathBuf::from)
}

/// Alias names are plain words, so they can never be confused with a path or a version.
pub fn is_alias_name(name: &str) -> bool {
    !name.is_empty()
        && !name.starts_with('.')
        && name
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '.'))
        && Spec::parse(name).is_err()
}

/// All aliases that point at an installed version, sorted by name.
pub fn aliases() -> Vec<(String, Version)> {
    let mut found: Vec<_> = std::fs::read_dir(dirs::aliases())
        .into_iter()
        .flatten()
        .flatten()
        .filter_map(|e| {
            Some((
                e.file_name().to_string_lossy().into_owned(),
                link::version(&e.path())?,
            ))
        })
        .collect();
    found.sort();
    found
}

/// Resolves an alias, version or range against the versions installed on this machine.
pub fn resolve_installed(input: &str) -> Result<Version> {
    let input = input.trim();
    if is_alias_name(input)
        && let Some(v) = link::version(&dirs::aliases().join(input))
        && dirs::version_dir(v).is_dir()
    {
        return Ok(v);
    }
    let spec = Spec::parse(input)?;
    let index =
        if spec.needs_lts_info() {
            Some(index::load_cached().context(
                "LTS names need the release list, which is not cached yet. Run `sne ls-remote` once",
            )?)
        } else {
            None
        };
    let lts_of = |v: Version| index.as_ref()?.iter().find(|r| r.version == v)?.lts.as_deref();
    spec.pick(dirs::installed().into_iter().map(|v| (v, lts_of(v))))
        .ok_or_else(|| anyhow!("Node `{input}` is not installed. Run `sne install {input}`"))
}
