mod commands;
mod dirs;
mod download;
mod extract;
mod index;
mod link;
mod platform;
mod resolve;
mod shell;
mod version;
mod version_file;

use anyhow::Result;
use clap::{CommandFactory, Parser, Subcommand};

use commands::{env, install, list, switch, update};

/// Simple Node Environment: a fast, cross-platform Node.js version manager.
#[derive(Parser)]
#[command(name = "sne", version, disable_version_flag = true)]
struct Cli {
    /// Print version
    #[arg(short = 'v', visible_short_alias = 'V', long = "version", action = clap::ArgAction::Version)]
    print_version: Option<bool>,
    #[command(subcommand)]
    command: Cmd,
}

#[derive(Subcommand)]
enum Cmd {
    /// Install one or more versions (`22`, `lts`, `latest`, `lts/iron`, `>=20 <23`).
    ///
    /// With no version, installs what the project asks for in .nvmrc, .node-version
    /// or package.json.
    #[command(visible_alias = "i")]
    Install { versions: Vec<String> },
    /// Switch this shell to an installed version.
    ///
    /// With no version, uses what the project asks for.
    #[command(visible_alias = "u")]
    Use {
        version: Option<String>,
        /// Used by the shell hook: switch only if the project asks for a different version.
        #[arg(long, hide = true, conflicts_with = "version")]
        auto: bool,
    },
    /// Set the version that new shells start with.
    #[command(visible_alias = "d")]
    Default { version: String },
    /// List installed versions.
    #[command(visible_alias = "l", alias = "list")]
    Ls,
    /// List versions available to install.
    #[command(name = "ls-remote", visible_alias = "lr", alias = "list-remote")]
    LsRemote {
        /// Only show versions matching this (`22`, `>=20`, `lts/iron`).
        filter: Option<String>,
        /// Only show LTS releases.
        #[arg(long)]
        lts: bool,
    },
    /// Remove an installed version.
    #[command(visible_alias = "rm", alias = "un")]
    Uninstall { version: String },
    /// Show the version active in this shell.
    #[command(visible_alias = "c")]
    Current,
    /// Show the path of the `node` executable for a version.
    #[command(visible_alias = "w")]
    Which { version: Option<String> },
    /// Run a command with a specific version: `sne exec 20 -- npm test`.
    #[command(visible_alias = "x")]
    Exec {
        version: String,
        #[arg(last = true, required = true)]
        command: Vec<String>,
    },
    /// Name a version (`sne alias work 20`), or list aliases when given no arguments.
    Alias {
        name: Option<String>,
        #[arg(requires = "name")]
        version: Option<String>,
    },
    /// Remove an alias.
    Unalias { name: String },
    /// Print the setup code for your shell.
    ///
    /// Add one line to your shell profile:
    ///
    ///   bash/zsh:    eval "$(sne env --use-on-cd)"
    ///   fish:        sne env --use-on-cd | source
    ///   PowerShell:  sne env --use-on-cd | Out-String | Invoke-Expression
    ///   cmd:         FOR /f "tokens=*" %i IN ('sne env --shell cmd') DO CALL %i
    #[command(verbatim_doc_comment)]
    Env {
        /// Shell to generate code for. Detected when omitted.
        #[arg(long, value_enum)]
        shell: Option<shell::Shell>,
        /// Switch versions automatically when you `cd` into a project.
        #[arg(long)]
        use_on_cd: bool,
    },
    /// Print a shell completion script.
    Completions { shell: clap_complete::Shell },
    /// Check that sne is set up correctly in this shell.
    #[command(visible_alias = "dr")]
    Doctor,
    /// Update sne itself to the latest version.
    #[command(visible_alias = "up")]
    Update,
    /// Commands about sne itself: `sne self check` looks for a new version.
    #[command(name = "self")]
    Own {
        #[command(subcommand)]
        action: OwnAction,
    },
    /// Desktop notifications about new sne versions: `sne notify on`, `sne notify off`.
    ///
    /// When on, sne checks once a day in the background. With no argument, shows the setting.
    Notify {
        #[arg(value_enum)]
        state: Option<Toggle>,
    },
}

#[derive(Subcommand)]
enum OwnAction {
    /// Ask GitHub whether a newer version of sne exists.
    Check {
        /// Also show a desktop notification, once per new version.
        #[arg(long)]
        notify: bool,
    },
    /// Update sne itself to the latest version (same as `sne update`).
    Update,
}

#[derive(Clone, Copy, clap::ValueEnum)]
enum Toggle {
    On,
    Off,
}

fn run() -> Result<i32> {
    match Cli::parse().command {
        Cmd::Install { versions } => install::run(versions)?,
        Cmd::Use { auto: true, .. } => switch::use_auto()?,
        Cmd::Use { version, .. } => switch::use_version(version)?,
        Cmd::Default { version } => switch::set_default(&version)?,
        Cmd::Ls => list::installed()?,
        Cmd::LsRemote { filter, lts } => list::remote(filter, lts)?,
        Cmd::Uninstall { version } => switch::uninstall(&version)?,
        Cmd::Current => switch::current()?,
        Cmd::Which { version } => switch::which(version)?,
        Cmd::Exec { version, command } => return switch::exec(&version, &command),
        Cmd::Alias { name, version } => switch::alias(name, version)?,
        Cmd::Unalias { name } => switch::unalias(&name)?,
        Cmd::Env { shell, use_on_cd } => env::run(shell, use_on_cd)?,
        Cmd::Completions { shell } => {
            clap_complete::generate(shell, &mut Cli::command(), "sne", &mut std::io::stdout());
        }
        Cmd::Doctor => env::doctor()?,
        Cmd::Update
        | Cmd::Own {
            action: OwnAction::Update,
        } => update::update()?,
        Cmd::Own {
            action: OwnAction::Check { notify },
        } => update::check(notify)?,
        Cmd::Notify {
            state: Some(Toggle::On),
        } => update::notify_on()?,
        Cmd::Notify {
            state: Some(Toggle::Off),
        } => update::notify_off()?,
        Cmd::Notify { state: None } => update::notify_status(),
    }
    Ok(0)
}

fn main() {
    match run() {
        Ok(code) => std::process::exit(code),
        Err(err) => {
            eprintln!("sne: {err:#}");
            std::process::exit(1);
        }
    }
}
