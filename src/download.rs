use std::io::{self, Read};
use std::sync::OnceLock;
use std::time::Duration;

use anyhow::{Context, Result, bail};
use indicatif::ProgressBar;
use sha2::{Digest, Sha256};

fn agent() -> &'static ureq::Agent {
    static AGENT: OnceLock<ureq::Agent> = OnceLock::new();
    AGENT.get_or_init(|| {
        ureq::Agent::config_builder()
            .timeout_connect(Some(Duration::from_secs(20)))
            .timeout_recv_response(Some(Duration::from_secs(30)))
            .user_agent(concat!("sne/", env!("CARGO_PKG_VERSION")))
            .build()
            .into()
    })
}

pub fn fetch_text(url: &str) -> Result<String> {
    let mut response = agent()
        .get(url)
        .call()
        .with_context(|| format!("request to {url} failed"))?;
    Ok(response.body_mut().read_to_string()?)
}

/// Conditional GET. Returns `None` when the server answers 304 Not Modified.
pub fn fetch_if_changed(url: &str, etag: Option<&str>) -> Result<Option<(String, Option<String>)>> {
    let mut request = agent().get(url);
    if let Some(etag) = etag {
        request = request.header("If-None-Match", etag);
    }
    let mut response = request.call()?;
    if response.status().as_u16() == 304 {
        return Ok(None);
    }
    let new_etag = response
        .headers()
        .get("etag")
        .and_then(|v| v.to_str().ok())
        .map(str::to_string);
    Ok(Some((response.body_mut().read_to_string()?, new_etag)))
}

/// Opens a streaming download. Returns the body and its length, when the server reports one.
pub fn open(url: &str) -> Result<(impl Read + use<>, Option<u64>)> {
    let response = agent()
        .get(url)
        .call()
        .with_context(|| format!("download of {url} failed"))?;
    let length = response.body().content_length();
    Ok((response.into_body().into_reader(), length))
}

/// SHA-256 that the mirror publishes for `file` in `SHASUMS256.txt`.
pub fn expected_sha256(release_url: &str, file: &str) -> Result<String> {
    let sums = fetch_text(&format!("{release_url}/SHASUMS256.txt"))?;
    for line in sums.lines() {
        let mut parts = line.split_whitespace();
        if let (Some(hash), Some(name)) = (parts.next(), parts.next())
            && name.trim_start_matches('*') == file
        {
            return Ok(hash.to_ascii_lowercase());
        }
    }
    bail!("{file} is not listed in SHASUMS256.txt")
}

/// Hashes and reports progress for everything read through it.
pub struct HashReader<R> {
    inner: R,
    hasher: Sha256,
    progress: ProgressBar,
}

impl<R: Read> HashReader<R> {
    pub fn new(inner: R, progress: ProgressBar) -> Self {
        Self {
            inner,
            hasher: Sha256::new(),
            progress,
        }
    }

    /// Reads whatever is left, then returns the hex digest of the whole stream.
    pub fn finish(mut self) -> io::Result<String> {
        io::copy(&mut self, &mut io::sink())?;
        let digest = self.hasher.finalize();
        Ok(digest.iter().map(|b| format!("{b:02x}")).collect())
    }
}

impl<R: Read> Read for HashReader<R> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let n = self.inner.read(buf)?;
        self.hasher.update(&buf[..n]);
        self.progress.inc(n as u64);
        Ok(n)
    }
}
