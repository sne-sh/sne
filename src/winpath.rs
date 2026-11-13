//! The user's `PATH` in the Windows registry: the one every new terminal starts with.

use std::path::Path;
use std::ptr::null_mut;

use anyhow::{Context, Result, bail};
use windows_sys::Win32::Foundation::{ERROR_FILE_NOT_FOUND, ERROR_SUCCESS};
use windows_sys::Win32::System::Registry::{
    HKEY_CURRENT_USER, REG_EXPAND_SZ, RRF_NOEXPAND, RRF_RT_REG_EXPAND_SZ, RRF_RT_REG_SZ, RegGetValueW,
    RegSetKeyValueW,
};
use windows_sys::Win32::UI::WindowsAndMessaging::{
    HWND_BROADCAST, SMTO_ABORTIFHUNG, SendMessageTimeoutW, WM_SETTINGCHANGE,
};

/// Adds `dir` to the end of the user's `PATH` unless it is already there. Returns whether it
/// was added. Terminals that are already open keep the `PATH` they started with.
pub fn ensure(dir: &Path) -> Result<bool> {
    let dir = dir.to_str().context("the folder name is not valid Unicode")?;
    let Some(updated) = with_entry(&read()?, dir) else {
        return Ok(false);
    };
    write(&updated)?;
    announce();
    Ok(true)
}

/// `paths` with `dir` added at the end, or `None` when `dir` is already one of its entries.
fn with_entry(paths: &str, dir: &str) -> Option<String> {
    let trim = |s: &str| s.trim().trim_end_matches(['\\', '/']).to_ascii_lowercase();
    if paths.split(';').any(|entry| trim(entry) == trim(dir)) {
        return None;
    }
    let kept = paths.trim_end_matches(';');
    Some(if kept.is_empty() {
        dir.to_string()
    } else {
        format!("{kept};{dir}")
    })
}

fn wide(text: &str) -> Vec<u16> {
    text.encode_utf16().chain([0]).collect()
}

/// The stored value, with `%NAME%` references left as they are. Empty when there is none.
/// Any other failure is an error, so a `PATH` that could not be read is never replaced.
fn read() -> Result<String> {
    let (key, name) = (wide("Environment"), wide("Path"));
    let flags = RRF_RT_REG_SZ | RRF_RT_REG_EXPAND_SZ | RRF_NOEXPAND;
    let mut bytes = 0u32;
    // SAFETY: the strings end with a zero, and a null buffer asks only for the size.
    let status = unsafe {
        RegGetValueW(
            HKEY_CURRENT_USER,
            key.as_ptr(),
            name.as_ptr(),
            flags,
            null_mut(),
            null_mut(),
            &mut bytes,
        )
    };
    if status == ERROR_FILE_NOT_FOUND {
        return Ok(String::new());
    }
    if status != ERROR_SUCCESS {
        bail!("could not read the user PATH (Windows error {status})");
    }
    let mut buffer = vec![0u16; bytes as usize / 2 + 1];
    let mut bytes = (buffer.len() * 2) as u32;
    // SAFETY: `bytes` is the size of `buffer` in bytes.
    let status = unsafe {
        RegGetValueW(
            HKEY_CURRENT_USER,
            key.as_ptr(),
            name.as_ptr(),
            flags,
            null_mut(),
            buffer.as_mut_ptr().cast(),
            &mut bytes,
        )
    };
    if status != ERROR_SUCCESS {
        bail!("could not read the user PATH (Windows error {status})");
    }
    let len = buffer.iter().position(|&unit| unit == 0).unwrap_or(buffer.len());
    String::from_utf16(&buffer[..len]).context("the user PATH is not valid Unicode")
}

fn write(value: &str) -> Result<()> {
    let (key, name, data) = (wide("Environment"), wide("Path"), wide(value));
    // SAFETY: the strings end with a zero, and the size given is the size of `data` in bytes.
    let status = unsafe {
        RegSetKeyValueW(
            HKEY_CURRENT_USER,
            key.as_ptr(),
            name.as_ptr(),
            REG_EXPAND_SZ,
            data.as_ptr().cast(),
            (data.len() * 2) as u32,
        )
    };
    if status != ERROR_SUCCESS {
        bail!("could not change the user PATH (Windows error {status})");
    }
    Ok(())
}

/// Tells running programs, Explorer above all, that the environment changed, so terminals
/// opened from now on get the new `PATH`.
fn announce() {
    let area = wide("Environment");
    // SAFETY: `area` outlives the call, and the timeout stops a hung window from blocking it.
    unsafe {
        SendMessageTimeoutW(
            HWND_BROADCAST,
            WM_SETTINGCHANGE,
            0,
            area.as_ptr() as isize,
            SMTO_ABORTIFHUNG,
            1000,
            null_mut(),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::with_entry;

    #[test]
    fn adds_a_missing_entry_at_the_end() {
        assert_eq!(with_entry("", r"C:\sne").as_deref(), Some(r"C:\sne"));
        assert_eq!(
            with_entry(r"C:\a;C:\b", r"C:\sne").as_deref(),
            Some(r"C:\a;C:\b;C:\sne")
        );
        assert_eq!(with_entry(r"C:\a;", r"C:\sne").as_deref(), Some(r"C:\a;C:\sne"));
    }

    #[test]
    fn leaves_a_path_that_already_has_the_entry() {
        assert_eq!(with_entry(r"C:\a;C:\sne;C:\b", r"C:\sne"), None);
        assert_eq!(with_entry(r"C:\a;c:\SNE\", r"C:\sne"), None);
    }
}
