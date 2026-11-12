use std::io::Write;

use anyhow::Result;

use super::{aliases, switch};
use crate::resolve::Spec;
use crate::{dirs, index};

pub fn installed() -> Result<()> {
    let versions = dirs::installed();
    if versions.is_empty() {
        println!("No Node versions installed yet. Try `sne install lts`");
        return Ok(());
    }
    let active = switch::active();
    let aliases = aliases();
    for version in versions {
        let marker = if Some(version) == active { '*' } else { ' ' };
        let names: Vec<&str> = aliases
            .iter()
            .filter(|(_, v)| *v == version)
            .map(|(name, _)| name.as_str())
            .collect();
        if names.is_empty() {
            println!("{marker} {version}");
        } else {
            println!("{marker} {version}  ({})", names.join(", "));
        }
    }
    Ok(())
}

pub fn remote(filter: Option<String>, lts_only: bool) -> Result<()> {
    let spec = filter.as_deref().map(Spec::parse).transpose()?;
    let releases = index::load()?;
    let installed = dirs::installed();
    // Oldest first, so the newest releases end up next to the prompt.
    let mut out = std::io::stdout().lock();
    for release in releases.iter().rev() {
        let lts = release.lts.as_deref();
        if (lts_only && lts.is_none()) || spec.as_ref().is_some_and(|s| !s.matches(release.version, lts)) {
            continue;
        }
        let marker = if installed.contains(&release.version) {
            '*'
        } else {
            ' '
        };
        let version = release.version.to_string();
        let line = match lts {
            Some(name) => format!("{marker} {version:<10} {}  LTS: {name}", release.date),
            None => format!("{marker} {version:<10} {}", release.date),
        };
        // A closed pipe (`sne ls-remote | head`) is not an error.
        if writeln!(out, "{line}").is_err() {
            break;
        }
    }
    Ok(())
}
