use std::fs;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use anyhow::Result;

use super::{default_link, same_path, shell_link};
use crate::shell::Shell;
use crate::{dirs, link};

/// Links left behind by shells that closed long ago. `sne use` recreates a link if it is missing.
const STALE_AFTER: Duration = Duration::from_secs(30 * 24 * 60 * 60);

pub fn run(shell: Option<Shell>, use_on_cd: bool) -> Result<()> {
    let shell = shell.unwrap_or_else(Shell::detect);
    let shells = dirs::shells();
    fs::create_dir_all(&shells)?;
    remove_stale_links(&shells);

    let millis = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |d| d.as_millis());
    let link = shells.join(format!("{}_{millis}", std::process::id()));
    if let Some(version) = link::version(&default_link())
        && dirs::bin_dir(version).is_dir()
    {
        link::create(&dirs::bin_dir(version), &link)?;
    }
    // The hook calls this program directly, which skips a launcher such as npm's on every `cd`.
    let sne = std::env::current_exe().ok().map(|exe| {
        let text = exe.to_string_lossy().into_owned();
        PathBuf::from(text.strip_prefix(r"\\?\").unwrap_or(&text))
    });
    print!("{}", shell.setup(&link, use_on_cd, sne.as_deref()));
    Ok(())
}

fn remove_stale_links(shells: &Path) {
    let Ok(entries) = fs::read_dir(shells) else { return };
    let now = SystemTime::now();
    for entry in entries.flatten() {
        let path = entry.path();
        let stale = fs::symlink_metadata(&path)
            .and_then(|m| m.modified())
            .ok()
            .and_then(|m| now.duration_since(m).ok())
            .is_some_and(|age| age > STALE_AFTER);
        if stale {
            let _ = link::remove(&path);
        }
    }
}

pub fn doctor() -> Result<()> {
    println!("sne {}", env!("CARGO_PKG_VERSION"));
    println!("data directory: {}", dirs::base().display());
    println!("installed versions: {}", dirs::installed().len());
    let mut problems = 0;
    let mut problem = |message: &str| {
        problems += 1;
        println!("problem: {message}");
    };

    match link::version(&default_link()) {
        Some(version) if dirs::version_dir(version).is_dir() => println!("default: {version}"),
        Some(version) => problem(&format!(
            "the default is {version}, which is not installed. Run `sne default <version>` to choose another"
        )),
        None => println!("default: not set (run `sne default <version>`)"),
    }

    let path_entries: Vec<PathBuf> =
        std::env::split_paths(&std::env::var_os("PATH").unwrap_or_default()).collect();
    match shell_link() {
        None if super::default_on_path() => {
            println!(
                "this terminal: uses the default version; `sne use` changes it for every terminal \
                 (add the setup line from `sne env --help` to switch per terminal)"
            );
            if let Some(dir) = super::shadowing_node() {
                problem(&format!(
                    "another Node comes first on PATH: {}. Remove that folder from PATH",
                    dir.display()
                ));
            }
        }
        None => problem(
            "Node from sne is not on PATH in this terminal. Run `sne use <version>` to add it, then \
             open a new terminal; or add the setup line from `sne env --help`",
        ),
        Some(link) => {
            match link::version(&link) {
                Some(version) => println!("active in this shell: {version}"),
                None => println!("active in this shell: none (run `sne use <version>`)"),
            }
            if !path_entries.iter().any(|entry| same_path(entry, &link)) {
                problem("the sne link is not on PATH; something reset PATH after `sne env` ran");
            }
            let first_node = path_entries
                .iter()
                .find(|dir| dir.join(dirs::node_exe()).is_file());
            match first_node {
                Some(dir) if same_path(dir, &link) => println!("`node` resolves to the sne link"),
                Some(dir) => problem(&format!(
                    "another Node comes first on PATH: {}. Move the sne setup line to the end of your profile",
                    dir.display()
                )),
                None => {}
            }
        }
    }
    println!("{}", super::update::status_line());
    if problems == 0 {
        println!("no problems found");
    }
    super::update::hint();
    Ok(())
}
