//! The list of Node.js releases (`index.json`), cached on disk.

use std::fs;
use std::time::{Duration, SystemTime};

use anyhow::{Context, Result};
use serde::Deserialize;

use crate::version::Version;
use crate::{dirs, download, platform};

/// Within this window the cached index is used without any network request.
const FRESH_FOR: Duration = Duration::from_secs(60 * 60);

pub struct Release {
    pub version: Version,
    pub lts: Option<String>,
    pub date: String,
    pub files: Vec<String>,
}

#[derive(Deserialize)]
struct RawRelease {
    version: String,
    #[serde(default)]
    lts: serde_json::Value,
    #[serde(default)]
    date: String,
    #[serde(default)]
    files: Vec<String>,
}

fn parse(json: &str) -> Result<Vec<Release>> {
    let raw: Vec<RawRelease> = serde_json::from_str(json).context("the release index is not valid JSON")?;
    Ok(raw
        .into_iter()
        .filter_map(|r| {
            Some(Release {
                version: Version::parse(&r.version)?,
                lts: r.lts.as_str().map(str::to_string),
                date: r.date,
                files: r.files,
            })
        })
        .collect())
}

/// Cached index only; never touches the network.
pub fn load_cached() -> Option<Vec<Release>> {
    let json = fs::read_to_string(dirs::cache().join("index.json")).ok()?;
    parse(&json).ok()
}

/// Cached index if fresh, otherwise a conditional request to the mirror.
pub fn load() -> Result<Vec<Release>> {
    let cache = dirs::cache();
    let json_path = cache.join("index.json");
    let etag_path = cache.join("index.etag");
    let url = format!("{}/index.json", platform::mirror());

    // The cache is only valid for the mirror it came from.
    let stored = fs::read_to_string(&etag_path).unwrap_or_default();
    let (cached_url, etag) = stored.split_once('\n').unwrap_or(("", ""));
    let cached = if cached_url == url {
        fs::read_to_string(&json_path).ok()
    } else {
        None
    };

    if let Some(json) = &cached {
        let age = fs::metadata(&json_path)
            .and_then(|m| m.modified())
            .ok()
            .and_then(|m| SystemTime::now().duration_since(m).ok());
        if age.is_some_and(|a| a < FRESH_FOR) {
            return parse(json);
        }
    }

    let etag = if cached.is_some() && !etag.is_empty() {
        Some(etag)
    } else {
        None
    };
    match download::fetch_if_changed(&url, etag) {
        Ok(Some((json, new_etag))) => {
            let releases = parse(&json)?;
            fs::create_dir_all(&cache)?;
            fs::write(&json_path, &json)?;
            fs::write(&etag_path, format!("{url}\n{}", new_etag.unwrap_or_default()))?;
            Ok(releases)
        }
        Ok(None) => {
            // Not modified: mark the cache as fresh again.
            if let Ok(file) = fs::File::options().write(true).open(&json_path) {
                let _ = file.set_modified(SystemTime::now());
            }
            parse(&cached.unwrap_or_default())
        }
        Err(err) => match cached {
            Some(json) => {
                eprintln!("sne: could not refresh the release list ({err:#}); using the cached copy");
                parse(&json)
            }
            None => Err(err.context(format!("could not download the release list from {url}"))),
        },
    }
}
