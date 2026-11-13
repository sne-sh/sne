use std::io::{self, Read};
use std::sync::OnceLock;
use std::time::Duration;

use anyhow::{Context, Result, bail};
use indicatif::ProgressBar;
use sha2::{Digest, Sha256};

/// How a URL may be fetched.
#[derive(Debug, PartialEq)]
enum Transport {
    /// HTTPS, and a redirect to plain HTTP is refused.
    Secure,
    /// Plain HTTP to this machine only, which the tests and a local mirror use.
    Loopback,
}

/// Node.js and sne itself are downloaded and then run, so everything arrives over HTTPS.
/// Plain HTTP would let anyone on the network replace both the file and its checksum.
fn transport(url: &str) -> Result<Transport> {
    if url.starts_with("https://") {
        return Ok(Transport::Secure);
    }
    if let Some(rest) = url.strip_prefix("http://") {
        let authority = rest.split(['/', '?', '#']).next().unwrap_or("");
        let host = authority.rsplit('@').next().unwrap_or("");
        let host = match host.strip_prefix('[') {
            // An IPv6 address is written in brackets, with the port after them.
            Some(v6) => v6.split(']').next().unwrap_or(""),
            None => host.split(':').next().unwrap_or(""),
        };
        if matches!(host, "127.0.0.1" | "localhost" | "::1") {
            return Ok(Transport::Loopback);
        }
    }
    bail!(
        "refusing to download from {url}: only https:// addresses are used. If this address comes \
         from SNE_NODE_MIRROR, SNE_RELEASES_URL or SNE_DOWNLOAD_BASE, change it to https://"
    )
}

fn agent(url: &str) -> Result<&'static ureq::Agent> {
    static SECURE: OnceLock<ureq::Agent> = OnceLock::new();
    static LOOPBACK: OnceLock<ureq::Agent> = OnceLock::new();
    let build = |https_only: bool| -> ureq::Agent {
        ureq::Agent::config_builder()
            .timeout_connect(Some(Duration::from_secs(20)))
            .timeout_recv_response(Some(Duration::from_secs(30)))
            .user_agent(concat!("sne/", env!("CARGO_PKG_VERSION")))
            .https_only(https_only)
            .build()
            .into()
    };
    Ok(match transport(url)? {
        Transport::Secure => SECURE.get_or_init(|| build(true)),
        Transport::Loopback => LOOPBACK.get_or_init(|| build(false)),
    })
}

pub fn fetch_text(url: &str) -> Result<String> {
    let mut response = agent(url)?
        .get(url)
        .call()
        .with_context(|| format!("request to {url} failed"))?;
    Ok(response.body_mut().read_to_string()?)
}

/// Conditional GET. Returns `None` when the server answers 304 Not Modified.
pub fn fetch_if_changed(url: &str, etag: Option<&str>) -> Result<Option<(String, Option<String>)>> {
    let mut request = agent(url)?.get(url);
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
    let response = agent(url)?
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

#[cfg(test)]
mod tests {
    use super::{Transport, transport};

    #[test]
    fn only_https_and_this_machine_are_allowed() {
        assert_eq!(
            transport("https://nodejs.org/dist/index.json").unwrap(),
            Transport::Secure
        );
        assert_eq!(
            transport("http://127.0.0.1:8080/index.json").unwrap(),
            Transport::Loopback
        );
        assert_eq!(
            transport("http://localhost/v22.1.0/x").unwrap(),
            Transport::Loopback
        );
        assert_eq!(transport("http://[::1]:9000/x").unwrap(), Transport::Loopback);

        assert!(transport("http://nodejs.org/dist/index.json").is_err());
        assert!(transport("http://mirror.example/node").is_err());
        // The name of this machine in front of another host is not this machine.
        assert!(transport("http://localhost.example.com/x").is_err());
        assert!(transport("http://127.0.0.1@example.com/x").is_err());
        assert!(transport("http://example.com/127.0.0.1").is_err());
        assert!(transport("ftp://nodejs.org/dist").is_err());
        assert!(transport("file:///etc/passwd").is_err());
    }
}
