use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{Context, Result, bail};

use super::{aliases, default_link, default_on_path, is_alias_name, resolve_installed, shell_link};
use crate::resolve::Spec;
use crate::version::Version;
use crate::{dirs, link, version_file};

pub fn use_version(input: Option<String>) -> Result<()> {
    let input = match input {
        Some(input) => input,
        None => {
            let found = version_file::find(&env::current_dir()?).context(
                "no version given, and no .nvmrc, .node-version or package.json `engines.node` was found",
            )?;
            found.spec
        }
    };
    let version = resolve_installed(&input)?;
    match shell_link() {
        Some(link) => {
            link::create(&dirs::bin_dir(version), &link)?;
            println!("Using Node {version}");
        }
        // No setup line ran in this terminal. Switch the default instead, which the installers
        // put on PATH, so `sne use` works with no setup at all.
        None => {
            link::create(&dirs::version_dir(version), &default_link())?;
            println!("Using Node {version} in every terminal");
            if !default_on_path() {
                println!(
                    "Node is not on PATH in this terminal yet. Open a new terminal. If it is still \
                     missing there, run the sne installer again or see `sne env --help`."
                );
            }
        }
    }
    Ok(())
}

/// Called by the shell hook after every directory change. Must stay fast and quiet.
pub fn use_auto() -> Result<()> {
    let Some(link) = shell_link() else { return Ok(()) };
    let Some(found) = version_file::find(&env::current_dir()?) else {
        return Ok(());
    };
    let current = link::version(&link);
    if current.is_some_and(|v| satisfies(&found.spec, v)) {
        return Ok(());
    }
    match resolve_installed(&found.spec) {
        Ok(version) if Some(version) == current => {}
        Ok(version) => {
            link::create(&dirs::bin_dir(version), &link)?;
            println!("Using Node {version} (from {})", found.path.display());
        }
        Err(_) => eprintln!(
            "sne: Node `{}` (from {}) is not installed. Run `sne install`",
            found.spec,
            found.path.display()
        ),
    }
    Ok(())
}

/// Cheap check that avoids switching when the active version already fits the request.
fn satisfies(input: &str, version: Version) -> bool {
    match Spec::parse(input) {
        Ok(spec) if !spec.needs_lts_info() => spec.matches(version, None),
        _ => false,
    }
}

pub fn set_default(input: &str) -> Result<()> {
    let version = resolve_installed(input)?;
    link::create(&dirs::version_dir(version), &default_link())?;
    println!("Node {version} is now the default for new shells");
    Ok(())
}

/// Version active in this shell, or the default when the shell is not set up.
pub fn active() -> Option<Version> {
    let link = shell_link().unwrap_or_else(default_link);
    link::version(&link).filter(|v| dirs::version_dir(*v).is_dir())
}

pub fn current() -> Result<()> {
    match active() {
        Some(version) => println!("{version}"),
        None => println!("none"),
    }
    Ok(())
}

pub fn which(input: Option<String>) -> Result<()> {
    let version = match input {
        Some(input) => resolve_installed(&input)?,
        None => active().context("no Node version is active")?,
    };
    println!("{}", dirs::bin_dir(version).join(dirs::node_exe()).display());
    Ok(())
}

pub fn alias(name: Option<String>, input: Option<String>) -> Result<()> {
    let (Some(name), Some(input)) = (name, input) else {
        for (name, version) in aliases() {
            println!("{name} -> {version}");
        }
        return Ok(());
    };
    if !is_alias_name(&name) {
        bail!("`{name}` cannot be used as an alias name (use letters, digits, `-` and `_`; not a version)");
    }
    let version = resolve_installed(&input)?;
    link::create(&dirs::version_dir(version), &dirs::aliases().join(&name))?;
    println!("{name} -> {version}");
    Ok(())
}

pub fn unalias(name: &str) -> Result<()> {
    let path = dirs::aliases().join(name);
    if !is_alias_name(name) || link::target(&path).is_none() {
        bail!("there is no alias named `{name}`");
    }
    link::remove(&path)?;
    println!("Removed alias {name}");
    Ok(())
}

pub fn uninstall(input: &str) -> Result<()> {
    let version = resolve_installed(input)?;
    for (name, target) in aliases() {
        if target == version {
            link::remove(&dirs::aliases().join(&name))?;
            println!("Removed alias {name}, which pointed to {version}");
        }
    }
    fs::remove_dir_all(dirs::version_dir(version))
        .with_context(|| format!("could not remove Node {version}; is it still running?"))?;
    println!("Uninstalled Node {version}");
    Ok(())
}

/// Runs a command with the given version first on `PATH`. Returns the command's exit code.
pub fn exec(input: &str, command: &[String]) -> Result<i32> {
    let version = resolve_installed(input)?;
    let bin = dirs::bin_dir(version);
    let mut paths = vec![bin.clone()];
    paths.extend(env::split_paths(&env::var_os("PATH").unwrap_or_default()));
    let (program, args) = command.split_first().context("no command given")?;
    let status = Command::new(find_program(&bin, program))
        .args(args)
        .env("PATH", env::join_paths(paths)?)
        .status()
        .with_context(|| format!("could not run `{program}`"))?;
    Ok(status.code().unwrap_or(1))
}

/// On Windows `npm` and `npx` are `.cmd` files, which are not found without their extension.
fn find_program(bin: &Path, program: &str) -> PathBuf {
    let bare = !program.contains(['/', '\\']) && Path::new(program).extension().is_none();
    if cfg!(windows) && bare {
        for ext in ["exe", "cmd", "bat"] {
            let candidate = bin.join(format!("{program}.{ext}"));
            if candidate.is_file() {
                return candidate;
            }
        }
    }
    PathBuf::from(program)
}
