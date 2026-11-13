pub mod env;
pub mod install;
pub mod list;
pub mod switch;
pub mod update;

use std::path::{Path, PathBuf};

use anyhow::{Context, Result, anyhow};

use crate::resolve::Spec;
use crate::version::Version;
use crate::{dirs, index, link};

pub fn default_link() -> PathBuf {
    dirs::aliases().join("default")
}

/// Folder of the default version's programs. The installers put it on `PATH`, so Node works in
/// every terminal and program even where the shell setup line has not run.
pub fn default_bin() -> PathBuf {
    if cfg!(windows) {
        default_link()
    } else {
        default_link().join("bin")
    }
}

/// Whether [`default_bin`] is on `PATH` in this process.
pub fn default_on_path() -> bool {
    let wanted = default_bin();
    path_entries().any(|entry| same_path(&entry, &wanted))
}

fn path_entries() -> impl Iterator<Item = PathBuf> {
    let path = std::env::var_os("PATH").unwrap_or_default();
    std::env::split_paths(&path).collect::<Vec<_>>().into_iter()
}

pub fn same_path(a: &Path, b: &Path) -> bool {
    let normalize = |p: &Path| {
        let text = p.to_string_lossy().replace('\\', "/");
        let text = text.trim_end_matches('/').to_string();
        if cfg!(windows) {
            text.to_ascii_lowercase()
        } else {
            text
        }
    };
    normalize(a) == normalize(b)
}

/// The folder of a Node that `PATH` finds before the default version, if there is one. Only
/// meaningful in a terminal without the per-shell link, which is meant to come first.
pub fn shadowing_node() -> Option<PathBuf> {
    let first = path_entries().find(|dir| dir.join(dirs::node_exe()).is_file())?;
    (!same_path(&first, &default_bin())).then_some(first)
}

/// For a terminal without the per-shell link, after the default version changed: says what is
/// still needed before `node` runs that version. On Windows it first adds the default version
/// to the user `PATH` when it is missing, which happens after an update from a release whose
/// installer did not add it.
pub fn report_default_path() {
    if !default_on_path() && !added_to_user_path() {
        println!(
            "Node is not on PATH in this terminal. Add {} to your PATH, or add the setup line \
             from `sne env --help`.",
            default_bin().display()
        );
        return;
    }
    if let Some(dir) = shadowing_node() {
        println!(
            "Another Node comes first on PATH: {}. Remove that folder from PATH to use this one.",
            dir.display()
        );
    }
}

/// Makes sure new terminals get the default version on `PATH`, and says so. False when that
/// could not be done. A data folder chosen with `SNE_DIR` is often a temporary one, so it is
/// never added.
fn added_to_user_path() -> bool {
    #[cfg(windows)]
    if std::env::var_os("SNE_NO_MODIFY_PATH").is_none_or(|v| v != "1")
        && std::env::var_os("SNE_DIR").is_none_or(|v| v.is_empty())
    {
        match crate::winpath::ensure(&default_bin()) {
            Ok(added) => {
                if added {
                    println!("Added {} to your PATH.", default_bin().display());
                }
                println!("Open a new terminal to use it: this one started before Node was on PATH.");
                return true;
            }
            Err(err) => eprintln!("sne: {err:#}"),
        }
    }
    false
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
