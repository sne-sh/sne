//! Windows notifications, shown by sne itself through the Windows Runtime.
//!
//! sne used to start Windows PowerShell with an encoded script to show them. A scheduled task that
//! starts PowerShell with an encoded script is also what a lot of malware does, so antivirus
//! programs watch for it. Calling the same Windows API directly avoids that.

use std::path::Path;

use anyhow::{Context, Result, bail};
use windows::Data::Xml::Dom::XmlDocument;
use windows::UI::Notifications::{ToastNotification, ToastNotificationManager};
use windows::core::HSTRING;
use windows_sys::Win32::Foundation::ERROR_SUCCESS;
use windows_sys::Win32::System::Registry::{HKEY_CURRENT_USER, REG_SZ, RegSetKeyValueW};

/// The name Windows files sne's notifications under, in Settings > System > Notifications.
const APP_ID: &str = "sne.sne";

fn wide(text: &str) -> Vec<u16> {
    text.encode_utf16().chain([0]).collect()
}

fn set_string(key: &str, name: &str, value: &str) -> Result<()> {
    let (key, name, value) = (wide(key), wide(name), wide(value));
    // SAFETY: the strings end with a zero, and the size given is that of `value` in bytes.
    let status = unsafe {
        RegSetKeyValueW(
            HKEY_CURRENT_USER,
            key.as_ptr(),
            name.as_ptr(),
            REG_SZ,
            value.as_ptr().cast(),
            (value.len() * 2) as u32,
        )
    };
    if status != ERROR_SUCCESS {
        bail!("could not write to the registry (Windows error {status})");
    }
    Ok(())
}

/// Gives the notifications the name "sne" and its logo, for this user. Without this, Windows
/// shows no name. No administrator rights are needed.
fn register(logo: Option<&Path>) -> Result<()> {
    let key = format!(r"Software\Classes\AppUserModelId\{APP_ID}");
    set_string(&key, "DisplayName", "sne")?;
    if let Some(logo) = logo.and_then(Path::to_str) {
        set_string(&key, "IconUri", logo)?;
    }
    Ok(())
}

fn escape(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

/// The notification: a title, one line of text, and a button that opens `url`.
fn toast_xml(title: &str, body: &str, url: &str) -> String {
    let (title, body, url) = (escape(title), escape(body), escape(url));
    format!(
        "<toast activationType=\"protocol\" launch=\"{url}\">\
         <visual><binding template=\"ToastGeneric\"><text>{title}</text><text>{body}</text></binding></visual>\
         <actions><action content=\"See what&apos;s new\" activationType=\"protocol\" arguments=\"{url}\"/></actions>\
         </toast>"
    )
}

pub fn show(title: &str, body: &str, url: &str, logo: Option<&Path>) -> Result<()> {
    // Without the name the notification still shows, so this is not a reason to give up.
    let _ = register(logo);
    let show = || -> windows::core::Result<()> {
        let xml = XmlDocument::new()?;
        xml.LoadXml(&HSTRING::from(toast_xml(title, body, url)))?;
        let toast = ToastNotification::CreateToastNotification(&xml)?;
        ToastNotificationManager::CreateToastNotifierWithId(&HSTRING::from(APP_ID))?.Show(&toast)
    };
    show().context("Windows did not show the notification")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn texts_cannot_change_the_notification() {
        let xml = toast_xml("Update available", "sne <1.0> & \"more\"", "https://x/?a=1&b=2");
        assert!(xml.contains("<text>sne &lt;1.0&gt; &amp; &quot;more&quot;</text>"));
        assert!(xml.contains("launch=\"https://x/?a=1&amp;b=2\""));
        // The Windows XML parser accepts it.
        let doc = XmlDocument::new().unwrap();
        doc.LoadXml(&HSTRING::from(xml)).unwrap();
    }
}
