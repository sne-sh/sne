//! Finding out about new versions of sne itself: `sne self check` and `sne notify`.
//!
//! Nothing here runs from `sne use`, `sne env` or the cd hook. The network is used only by
//! `sne self check` and, in the background, by `sne install`.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{Duration, SystemTime};

use anyhow::{Context, Result, anyhow, bail};

use crate::version::Version;
use crate::{dirs, download};

const LATEST_RELEASE_API: &str = "https://api.github.com/repos/sne-sh/sne/releases/latest";
const RELEASES_PAGE: &str = "https://github.com/sne-sh/sne/releases/latest";
/// `sne install` asks GitHub again only when the last answer is older than this.
const FRESH_FOR: Duration = Duration::from_secs(24 * 60 * 60);

fn running() -> Version {
    Version::parse(env!("CARGO_PKG_VERSION")).expect("the crate version is a full version")
}

fn update_command() -> &'static str {
    if cfg!(windows) {
        "irm https://raw.githubusercontent.com/sne-sh/sne/main/install.ps1 | iex"
    } else {
        "curl -fsSL https://raw.githubusercontent.com/sne-sh/sne/main/install.sh | sh"
    }
}

/// Last version GitHub reported. The file's modification time is when it was asked.
fn latest_file() -> PathBuf {
    dirs::cache().join("sne-latest.txt")
}

/// Last version a desktop notification was shown for, so each release notifies once.
fn notified_file() -> PathBuf {
    dirs::cache().join("sne-notified.txt")
}

fn read_version(path: &Path) -> Option<Version> {
    Version::parse(&fs::read_to_string(path).ok()?)
}

fn write_version(path: &Path, version: Version) {
    let _ = fs::create_dir_all(dirs::cache());
    let _ = fs::write(path, version.to_string());
}

fn tag_from_json(json: &str) -> Result<Version> {
    let value: serde_json::Value = serde_json::from_str(json).context("unexpected answer from GitHub")?;
    let tag = value["tag_name"]
        .as_str()
        .ok_or_else(|| anyhow!("the latest release has no tag name"))?;
    Version::parse(tag).ok_or_else(|| anyhow!("the latest release tag `{tag}` is not a version"))
}

fn fetch_latest() -> Result<Version> {
    let url = std::env::var("SNE_RELEASES_URL")
        .ok()
        .filter(|u| !u.trim().is_empty())
        .unwrap_or_else(|| LATEST_RELEASE_API.to_string());
    let latest = tag_from_json(&download::fetch_text(&url)?)?;
    write_version(&latest_file(), latest);
    Ok(latest)
}

/// A newer version that an earlier check already found. Never uses the network.
fn newer_known() -> Option<Version> {
    read_version(&latest_file()).filter(|latest| *latest > running())
}

fn passive_checks_disabled() -> bool {
    let set = |name: &str| std::env::var_os(name).is_some_and(|v| !v.is_empty());
    set("SNE_NO_UPDATE_CHECK") || set("CI")
}

/// Asks GitHub for the latest version on another thread, at most once a day.
/// The caller never waits for it; the answer is used by [`hint`], now or on a later run.
pub fn refresh_in_background() {
    if passive_checks_disabled() {
        return;
    }
    let fresh = fs::metadata(latest_file())
        .and_then(|m| m.modified())
        .ok()
        .and_then(|asked| SystemTime::now().duration_since(asked).ok())
        .is_some_and(|age| age < FRESH_FOR);
    if !fresh {
        std::thread::spawn(|| {
            let _ = fetch_latest();
        });
    }
}

/// One line about a newer version, when an earlier check found one.
pub fn hint() {
    if passive_checks_disabled() {
        return;
    }
    if let Some(latest) = newer_known() {
        println!(
            "sne {latest} is available (you have {}). Update: {}",
            running(),
            update_command()
        );
    }
}

/// `sne self check`
pub fn check(notify: bool) -> Result<()> {
    let latest = fetch_latest().context("could not check for a new sne version")?;
    let running = running();
    if latest <= running {
        println!("sne {running} is the latest version");
        return Ok(());
    }
    println!("sne {latest} is available (you have {running})");
    println!("Update:       {}", update_command());
    println!("What changed: {RELEASES_PAGE}");
    if notify && read_version(&notified_file()) != Some(latest) {
        os::show(
            &format!("sne {latest} is available"),
            &format!("You have {running}. Run the sne installer again to update."),
        )
        .context("could not show a desktop notification")?;
        write_version(&notified_file(), latest);
    }
    Ok(())
}

/// `sne notify on`
pub fn notify_on() -> Result<()> {
    let exe = std::env::current_exe().context("could not find the sne program")?;
    os::enable(&exe)?;
    println!("Notifications are on: sne checks once a day and tells you when a new version is out.");
    println!("Turn them off with `sne notify off`.");
    Ok(())
}

/// `sne notify off`
pub fn notify_off() -> Result<()> {
    os::disable()?;
    println!("Notifications are off.");
    Ok(())
}

/// `sne notify`
pub fn notify_status() {
    println!("{}", status_line());
}

pub fn status_line() -> String {
    if os::enabled() {
        "update notifications: on (turn off with `sne notify off`)".to_string()
    } else {
        "update notifications: off (turn on with `sne notify on`)".to_string()
    }
}

/// Runs a helper program and turns a failure into an error that shows what it printed.
fn run(command: &mut Command) -> Result<()> {
    let name = command.get_program().to_string_lossy().into_owned();
    let output = command
        .output()
        .with_context(|| format!("could not run `{name}`"))?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);
        let said = if stderr.trim().is_empty() {
            stdout.trim()
        } else {
            stderr.trim()
        };
        bail!("`{name}` failed: {said}");
    }
    Ok(())
}

#[cfg(windows)]
mod os {
    use std::os::windows::process::CommandExt;

    use super::*;

    const TASK_NAME: &str = "sne update check";
    /// Keeps helper programs from opening a console window.
    const CREATE_NO_WINDOW: u32 = 0x0800_0000;

    /// Registers sne as a notification sender for this user, then shows a toast that opens
    /// the releases page when clicked. Texts arrive through environment variables.
    const TOAST_SCRIPT: &str = r#"
$ErrorActionPreference = 'Stop'
$id = 'sne.sne'
$key = "HKCU:\Software\Classes\AppUserModelId\$id"
if (-not (Test-Path $key)) { New-Item -Path $key -Force | Out-Null }
Set-ItemProperty -Path $key -Name DisplayName -Value 'sne'
if ($env:SNE_TOAST_ICON) { Set-ItemProperty -Path $key -Name IconUri -Value $env:SNE_TOAST_ICON }
[Windows.UI.Notifications.ToastNotificationManager, Windows.UI.Notifications, ContentType = WindowsRuntime] | Out-Null
[Windows.Data.Xml.Dom.XmlDocument, Windows.Data.Xml.Dom.XmlDocument, ContentType = WindowsRuntime] | Out-Null
function Esc($text) { [System.Security.SecurityElement]::Escape($text) }
$xml = New-Object Windows.Data.Xml.Dom.XmlDocument
$xml.LoadXml("<toast activationType='protocol' launch='$(Esc $env:SNE_TOAST_URL)'><visual><binding template='ToastGeneric'><text>$(Esc $env:SNE_TOAST_TITLE)</text><text>$(Esc $env:SNE_TOAST_BODY)</text></binding></visual></toast>")
$toast = New-Object Windows.UI.Notifications.ToastNotification $xml
[Windows.UI.Notifications.ToastNotificationManager]::CreateToastNotifier($id).Show($toast)
"#;

    pub fn show(title: &str, body: &str) -> Result<()> {
        // Windows PowerShell (not PowerShell 7) is on every Windows 10 and 11 and can load WinRT types.
        let mut command = Command::new("powershell.exe");
        command
            .args(["-NoProfile", "-NonInteractive", "-ExecutionPolicy", "Bypass"])
            .args(["-EncodedCommand", &base64_utf16(TOAST_SCRIPT)])
            .env("SNE_TOAST_TITLE", title)
            .env("SNE_TOAST_BODY", body)
            .env("SNE_TOAST_URL", RELEASES_PAGE)
            .creation_flags(CREATE_NO_WINDOW);
        // The setup wizard puts the icon next to the program.
        if let Some(icon) = std::env::current_exe()
            .ok()
            .map(|exe| exe.with_file_name("sne.ico"))
            .filter(|icon| icon.is_file())
        {
            command.env("SNE_TOAST_ICON", icon);
        }
        run(&mut command)
    }

    /// PowerShell's `-EncodedCommand` takes base64 of the script as UTF-16LE.
    fn base64_utf16(text: &str) -> String {
        const ALPHABET: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
        let bytes: Vec<u8> = text.encode_utf16().flat_map(u16::to_le_bytes).collect();
        let mut out = String::with_capacity(bytes.len().div_ceil(3) * 4);
        for chunk in bytes.chunks(3) {
            let n = (u32::from(chunk[0]) << 16)
                | (u32::from(*chunk.get(1).unwrap_or(&0)) << 8)
                | u32::from(*chunk.get(2).unwrap_or(&0));
            out.push(ALPHABET[(n >> 18) as usize & 63] as char);
            out.push(ALPHABET[(n >> 12) as usize & 63] as char);
            out.push(if chunk.len() > 1 {
                ALPHABET[(n >> 6) as usize & 63] as char
            } else {
                '='
            });
            out.push(if chunk.len() > 2 {
                ALPHABET[n as usize & 63] as char
            } else {
                '='
            });
        }
        out
    }

    fn xml_escape(text: &str) -> String {
        text.replace('&', "&amp;")
            .replace('<', "&lt;")
            .replace('>', "&gt;")
            .replace('"', "&quot;")
    }

    /// A daily task for the current user. It also runs after a missed day, and it starts sne
    /// through `conhost --headless` so no console window appears.
    pub fn enable(exe: &Path) -> Result<()> {
        let arguments = xml_escape(&format!("--headless \"{}\" self check --notify", exe.display()));
        let xml = format!(
            r#"<?xml version="1.0" encoding="UTF-16"?>
<Task version="1.2" xmlns="http://schemas.microsoft.com/windows/2004/02/mit/task">
  <RegistrationInfo>
    <Description>Tells you when a new version of sne is available. Remove with: sne notify off</Description>
  </RegistrationInfo>
  <Triggers>
    <CalendarTrigger>
      <StartBoundary>2026-01-01T12:00:00</StartBoundary>
      <Enabled>true</Enabled>
      <ScheduleByDay>
        <DaysInterval>1</DaysInterval>
      </ScheduleByDay>
    </CalendarTrigger>
  </Triggers>
  <Principals>
    <Principal id="Author">
      <LogonType>InteractiveToken</LogonType>
      <RunLevel>LeastPrivilege</RunLevel>
    </Principal>
  </Principals>
  <Settings>
    <MultipleInstancesPolicy>IgnoreNew</MultipleInstancesPolicy>
    <DisallowStartIfOnBatteries>false</DisallowStartIfOnBatteries>
    <StopIfGoingOnBatteries>false</StopIfGoingOnBatteries>
    <StartWhenAvailable>true</StartWhenAvailable>
    <RunOnlyIfNetworkAvailable>true</RunOnlyIfNetworkAvailable>
    <ExecutionTimeLimit>PT5M</ExecutionTimeLimit>
    <Enabled>true</Enabled>
  </Settings>
  <Actions Context="Author">
    <Exec>
      <Command>conhost.exe</Command>
      <Arguments>{arguments}</Arguments>
    </Exec>
  </Actions>
</Task>
"#
        );
        fs::create_dir_all(dirs::cache())?;
        let file = dirs::cache().join("notify-task.xml");
        // schtasks reads UTF-16 with a byte order mark.
        let mut bytes = vec![0xFF, 0xFE];
        bytes.extend(xml.encode_utf16().flat_map(u16::to_le_bytes));
        fs::write(&file, bytes)?;
        let result = run(Command::new("schtasks.exe")
            .args(["/Create", "/F", "/TN", TASK_NAME, "/XML"])
            .arg(&file)
            .creation_flags(CREATE_NO_WINDOW));
        let _ = fs::remove_file(&file);
        result.context("could not create the scheduled task")
    }

    pub fn disable() -> Result<()> {
        if !enabled() {
            return Ok(());
        }
        run(Command::new("schtasks.exe")
            .args(["/Delete", "/F", "/TN", TASK_NAME])
            .creation_flags(CREATE_NO_WINDOW))
        .context("could not remove the scheduled task")
    }

    pub fn enabled() -> bool {
        Command::new("schtasks.exe")
            .args(["/Query", "/TN", TASK_NAME])
            .creation_flags(CREATE_NO_WINDOW)
            .output()
            .is_ok_and(|out| out.status.success())
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn encodes_scripts_for_powershell() {
            // "hi" as UTF-16LE is 68 00 69 00.
            assert_eq!(base64_utf16("hi"), "aABpAA==");
            assert_eq!(base64_utf16("a"), "YQA=");
            assert_eq!(base64_utf16("abc"), "YQBiAGMA");
        }
    }
}

#[cfg(target_os = "macos")]
mod os {
    use super::*;

    const LABEL: &str = "io.github.sne-sh.sne.update-check";

    pub fn show(title: &str, body: &str) -> Result<()> {
        let quote = |text: &str| format!("\"{}\"", text.replace('\\', "\\\\").replace('"', "\\\""));
        let script = format!("display notification {} with title {}", quote(body), quote(title));
        run(Command::new("osascript").args(["-e", &script]))
    }

    fn plist() -> PathBuf {
        let home = std::env::var_os("HOME").unwrap_or_default();
        PathBuf::from(home)
            .join("Library")
            .join("LaunchAgents")
            .join(format!("{LABEL}.plist"))
    }

    /// `gui/<uid>`, the launchd domain of the logged-in user.
    fn domain() -> Result<String> {
        let out = Command::new("id")
            .arg("-u")
            .output()
            .context("could not run `id`")?;
        Ok(format!("gui/{}", String::from_utf8_lossy(&out.stdout).trim()))
    }

    fn xml_escape(text: &str) -> String {
        text.replace('&', "&amp;")
            .replace('<', "&lt;")
            .replace('>', "&gt;")
    }

    /// A launchd agent that runs the check at login and then once a day.
    pub fn enable(exe: &Path) -> Result<()> {
        let file = plist();
        if let Some(parent) = file.parent() {
            fs::create_dir_all(parent)?;
        }
        let exe = xml_escape(&exe.to_string_lossy());
        fs::write(
            &file,
            format!(
                r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
  <key>Label</key>
  <string>{LABEL}</string>
  <key>ProgramArguments</key>
  <array>
    <string>{exe}</string>
    <string>self</string>
    <string>check</string>
    <string>--notify</string>
  </array>
  <key>RunAtLoad</key>
  <true/>
  <key>StartInterval</key>
  <integer>86400</integer>
</dict>
</plist>
"#
            ),
        )?;
        let domain = domain()?;
        // Replace an agent left by an earlier `sne notify on`.
        let _ = Command::new("launchctl")
            .args(["bootout", &format!("{domain}/{LABEL}")])
            .output();
        run(Command::new("launchctl").args(["bootstrap", &domain]).arg(&file))
            .context("could not start the launchd agent")
    }

    pub fn disable() -> Result<()> {
        if let Ok(domain) = domain() {
            let _ = Command::new("launchctl")
                .args(["bootout", &format!("{domain}/{LABEL}")])
                .output();
        }
        match fs::remove_file(plist()) {
            Err(err) if err.kind() != std::io::ErrorKind::NotFound => Err(err.into()),
            _ => Ok(()),
        }
    }

    pub fn enabled() -> bool {
        plist().is_file()
    }
}

#[cfg(all(unix, not(target_os = "macos")))]
mod os {
    use super::*;

    const UNIT: &str = "sne-update-check";

    pub fn show(title: &str, body: &str) -> Result<()> {
        run(Command::new("notify-send").args(["--app-name=sne", title, body]))
            .context("`notify-send` is needed for desktop notifications (package libnotify)")
    }

    fn unit_dir() -> PathBuf {
        let config = std::env::var_os("XDG_CONFIG_HOME")
            .filter(|dir| !dir.is_empty())
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from(std::env::var_os("HOME").unwrap_or_default()).join(".config"));
        config.join("systemd").join("user")
    }

    fn systemctl(args: &[&str]) -> Result<()> {
        run(Command::new("systemctl").arg("--user").args(args))
    }

    /// A systemd user timer that runs the check once a day, and after a missed day.
    pub fn enable(exe: &Path) -> Result<()> {
        let dir = unit_dir();
        fs::create_dir_all(&dir)?;
        fs::write(
            dir.join(format!("{UNIT}.service")),
            format!(
                "[Unit]\nDescription=Check for a new version of sne\n\n[Service]\nType=oneshot\nExecStart=\"{}\" self check --notify\n",
                exe.display()
            ),
        )?;
        fs::write(
            dir.join(format!("{UNIT}.timer")),
            "[Unit]\nDescription=Daily check for a new version of sne\n\n[Timer]\nOnCalendar=daily\nPersistent=true\nRandomizedDelaySec=1h\n\n[Install]\nWantedBy=timers.target\n",
        )?;
        systemctl(&["daemon-reload"])
            .and_then(|()| systemctl(&["enable", "--now", &format!("{UNIT}.timer")]))
            .with_context(|| {
                format!(
                    "could not start the systemd user timer. Without systemd, add this line to your crontab instead:\n  0 12 * * * \"{}\" self check --notify",
                    exe.display()
                )
            })
    }

    pub fn disable() -> Result<()> {
        if !enabled() {
            return Ok(());
        }
        let _ = systemctl(&["disable", "--now", &format!("{UNIT}.timer")]);
        let dir = unit_dir();
        let _ = fs::remove_file(dir.join(format!("{UNIT}.timer")));
        let _ = fs::remove_file(dir.join(format!("{UNIT}.service")));
        let _ = systemctl(&["daemon-reload"]);
        Ok(())
    }

    pub fn enabled() -> bool {
        unit_dir().join(format!("{UNIT}.timer")).is_file()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_the_tag_of_the_latest_release() {
        let json = r#"{"url":"x","tag_name":"v1.4.2","name":"sne 1.4.2","draft":false}"#;
        assert_eq!(tag_from_json(json).unwrap(), Version::parse("1.4.2").unwrap());
        assert!(tag_from_json(r#"{"message":"Not Found"}"#).is_err());
        assert!(tag_from_json(r#"{"tag_name":"nightly"}"#).is_err());
    }
}
