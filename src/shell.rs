//! Shell setup code printed by `sne env`.

use std::path::Path;

use clap::ValueEnum;

#[derive(Clone, Copy, Debug, PartialEq, ValueEnum)]
pub enum Shell {
    Bash,
    Zsh,
    Fish,
    #[value(name = "powershell", alias = "pwsh")]
    #[allow(clippy::enum_variant_names)]
    PowerShell,
    Cmd,
}

impl Shell {
    /// Best guess when `--shell` is not given.
    pub fn detect() -> Self {
        let from_env = std::env::var("SHELL").unwrap_or_default().to_ascii_lowercase();
        let name = from_env
            .rsplit(['/', '\\'])
            .next()
            .unwrap_or("")
            .trim_end_matches(".exe");
        match name {
            "zsh" => Shell::Zsh,
            "fish" => Shell::Fish,
            "bash" | "sh" => Shell::Bash,
            _ if cfg!(windows) => Shell::PowerShell,
            _ => Shell::Bash,
        }
    }

    /// The code that puts `link` on `PATH` and remembers it in `SNE_SHELL_LINK`. With `use_on_cd`,
    /// also a hook that runs `sne use --auto` when the folder changes.
    ///
    /// The hook runs the program at `sne` directly. Found through `PATH`, `sne` may be a launcher
    /// (npm's starts Node.js first, Scoop's is a shim) that makes every `cd` several times slower.
    /// If that file is gone, as after an upgrade by Homebrew, the hook falls back to `sne`.
    pub fn setup(self, link: &Path, use_on_cd: bool, sne: Option<&Path>) -> String {
        let native = link.to_string_lossy().into_owned();
        // POSIX shells on Windows (Git Bash, MSYS2) need `/c/...` style entries in PATH.
        let posix = if cfg!(windows) {
            to_posix(&native)
        } else {
            native.clone()
        };
        let mut out = match self {
            Shell::Bash | Shell::Zsh => format!(
                "export SNE_SHELL_LINK={}\nexport PATH={}:\"$PATH\"\n",
                single_quote(&native),
                single_quote(&posix)
            ),
            Shell::Fish => format!(
                "set -gx SNE_SHELL_LINK {};\nset -gx PATH {} $PATH;\n",
                single_quote(&native),
                single_quote(&posix)
            ),
            Shell::PowerShell => {
                let quoted = native.replace('\'', "''");
                let sep = if cfg!(windows) { ';' } else { ':' };
                format!("$env:SNE_SHELL_LINK = '{quoted}'\n$env:PATH = '{quoted}{sep}' + $env:PATH\n")
            }
            // The quotes keep `&` and other special characters in a folder name from being run.
            Shell::Cmd => format!("SET \"SNE_SHELL_LINK={native}\"\nSET \"PATH={native};%PATH%\"\n"),
        };
        if use_on_cd {
            out.push_str(&self.auto_command(sne));
            out.push_str(self.cd_hook());
        }
        out
    }

    /// Defines `__sne_auto`, which the hook calls: the program at `sne` when it still exists,
    /// otherwise whatever `sne` is on `PATH`.
    fn auto_command(self, sne: Option<&Path>) -> String {
        let Some(sne) = sne.map(|p| p.to_string_lossy().into_owned()) else {
            return match self {
                Shell::Bash | Shell::Zsh => "__sne_auto() { sne use --auto; }\n".to_string(),
                Shell::Fish => "function __sne_auto\n  sne use --auto\nend\n".to_string(),
                Shell::PowerShell => "function global:__sne_auto { sne use --auto }\n".to_string(),
                Shell::Cmd => String::new(),
            };
        };
        match self {
            Shell::Bash | Shell::Zsh | Shell::Fish => {
                let quoted = single_quote(&if cfg!(windows) { to_posix(&sne) } else { sne });
                if self == Shell::Fish {
                    format!(
                        "function __sne_auto\n  if test -x {quoted}\n    {quoted} use --auto\n  else\n    \
                         sne use --auto\n  end\nend\n"
                    )
                } else {
                    format!(
                        "__sne_auto() {{ if [ -x {quoted} ]; then {quoted} use --auto; else sne use --auto; fi; }}\n"
                    )
                }
            }
            Shell::PowerShell => {
                let quoted = format!("'{}'", sne.replace('\'', "''"));
                format!(
                    "function global:__sne_auto {{ if (Test-Path -LiteralPath {quoted}) {{ & {quoted} use --auto }} \
                     else {{ sne use --auto }} }}\n"
                )
            }
            Shell::Cmd => String::new(),
        }
    }

    fn cd_hook(self) -> &'static str {
        match self {
            Shell::Bash => concat!(
                "__sne_hook() {\n",
                "  if [ \"$PWD\" != \"${__SNE_LAST_PWD-}\" ]; then\n",
                "    __SNE_LAST_PWD=\"$PWD\"\n",
                "    __sne_auto\n",
                "  fi\n",
                "}\n",
                "case \";${PROMPT_COMMAND-};\" in\n",
                "  *\";__sne_hook;\"*) ;;\n",
                "  *) PROMPT_COMMAND=\"__sne_hook${PROMPT_COMMAND:+;$PROMPT_COMMAND}\" ;;\n",
                "esac\n",
            ),
            Shell::Zsh => concat!(
                "__sne_hook() { __sne_auto; }\n",
                "autoload -U add-zsh-hook\n",
                "add-zsh-hook chpwd __sne_hook\n",
                "__sne_hook\n",
            ),
            Shell::Fish => concat!(
                "function __sne_hook --on-variable PWD\n",
                "  __sne_auto\n",
                "end\n",
                "__sne_hook\n",
            ),
            Shell::PowerShell => concat!(
                "function global:Set-LocationWithSne {\n",
                "  param($path)\n",
                "  if ($null -eq $path) { Set-Location } else { Set-Location $path }\n",
                "  __sne_auto\n",
                "}\n",
                "Set-Alias -Name cd -Value Set-LocationWithSne -Option AllScope -Scope Global -Force\n",
                "__sne_auto\n",
            ),
            // cmd.exe has no hook that runs when the directory changes.
            Shell::Cmd => "",
        }
    }
}

fn single_quote(s: &str) -> String {
    format!("'{}'", s.replace('\'', "'\\''"))
}

/// `C:\Users\me` becomes `/c/Users/me`.
fn to_posix(path: &str) -> String {
    let bytes = path.as_bytes();
    if bytes.len() >= 2 && bytes[1] == b':' && bytes[0].is_ascii_alphabetic() {
        let drive = (bytes[0] as char).to_ascii_lowercase();
        return format!("/{drive}{}", path[2..].replace('\\', "/"));
    }
    path.replace('\\', "/")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn converts_windows_paths_for_posix_shells() {
        assert_eq!(to_posix(r"C:\Users\me\sne"), "/c/Users/me/sne");
        assert_eq!(to_posix("/home/me/sne"), "/home/me/sne");
    }

    #[test]
    fn quotes_single_quotes() {
        assert_eq!(single_quote("it's"), r"'it'\''s'");
    }

    #[test]
    fn cmd_lines_survive_special_characters() {
        let link = r"C:\Users\R&D team\sne\shells\1";
        assert_eq!(
            Shell::Cmd.setup(Path::new(link), false, None),
            format!("SET \"SNE_SHELL_LINK={link}\"\nSET \"PATH={link};%PATH%\"\n")
        );
    }

    #[test]
    fn hook_calls_the_program_directly_with_a_fallback() {
        let link = Path::new("/tmp/sne/shells/1");
        let sne = Path::new("/opt/it's here/sne");

        let bash = Shell::Bash.setup(link, true, Some(sne));
        assert!(
            bash.contains(
                r"__sne_auto() { if [ -x '/opt/it'\''s here/sne' ]; then '/opt/it'\''s here/sne' use --auto; else sne use --auto; fi; }"
            ),
            "{bash}"
        );
        assert!(bash.contains("    __sne_auto\n"), "{bash}");

        let fish = Shell::Fish.setup(link, true, Some(sne));
        assert!(fish.contains("  if test -x '/opt/it'\\''s here/sne'\n"), "{fish}");

        let pwsh = Shell::PowerShell.setup(link, true, Some(Path::new(r"C:\Users\O'Neil\sne.exe")));
        assert!(
            pwsh.contains(
                r"function global:__sne_auto { if (Test-Path -LiteralPath 'C:\Users\O''Neil\sne.exe') { & 'C:\Users\O''Neil\sne.exe' use --auto } else { sne use --auto } }"
            ),
            "{pwsh}"
        );

        // Without a known program, and in cmd, which has no hook.
        assert!(
            Shell::Zsh
                .setup(link, true, None)
                .contains("__sne_auto() { sne use --auto; }")
        );
        assert!(!Shell::Cmd.setup(link, true, Some(sne)).contains("__sne_auto"));
        assert!(!Shell::Bash.setup(link, false, Some(sne)).contains("__sne_auto"));
    }

    #[test]
    fn hook_is_only_added_on_request() {
        let link = Path::new("/tmp/sne/shells/1");
        assert!(!Shell::Zsh.setup(link, false, None).contains("add-zsh-hook"));
        assert!(
            Shell::Zsh
                .setup(link, true, None)
                .contains("add-zsh-hook chpwd __sne_hook")
        );
    }
}
