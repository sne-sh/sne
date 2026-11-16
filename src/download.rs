use std::io::{self, Read};
use std::sync::OnceLock;
use std::time::Duration;

use anyhow::{Context, Result, bail};
use indicatif::ProgressBar;
use sha2::{Digest, Sha256};
use ureq::tls::{RootCerts, TlsConfig};

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

/// Certificates that HTTPS servers are checked against.
///
/// The system's own store, like a browser or curl. Companies that inspect HTTPS traffic add their
/// certificate there, and with only the certificates built into sne every download failed for
/// them. A Linux system without a certificate bundle, such as a slim container, gets the ones
/// built into sne instead.
fn root_certs() -> RootCerts {
    if cfg!(any(windows, target_os = "macos")) {
        return RootCerts::PlatformVerifier;
    }
    let set = |name: &str| std::env::var_os(name).is_some_and(|v| !v.is_empty());
    let bundle = [
        "/etc/ssl/certs/ca-certificates.crt", // Debian, Ubuntu, Alpine, Arch
        "/etc/pki/tls/certs/ca-bundle.crt",   // Fedora, RHEL
        "/etc/ssl/ca-bundle.pem",             // openSUSE
        "/etc/ssl/cert.pem",                  // Alpine, BSD
    ]
    .iter()
    .any(|file| std::path::Path::new(file).is_file());
    if bundle || set("SSL_CERT_FILE") || set("SSL_CERT_DIR") {
        RootCerts::PlatformVerifier
    } else {
        RootCerts::WebPki
    }
}

fn agent(url: &str) -> Result<&'static ureq::Agent> {
    static SECURE: OnceLock<ureq::Agent> = OnceLock::new();
    static LOOPBACK: OnceLock<ureq::Agent> = OnceLock::new();
    // A proxy is taken from HTTPS_PROXY, HTTP_PROXY or ALL_PROXY, minus NO_PROXY.
    let build = |https_only: bool| -> ureq::Agent {
        ureq::Agent::config_builder()
            .timeout_connect(Some(Duration::from_secs(20)))
            .timeout_recv_response(Some(Duration::from_secs(30)))
            // A last resort against a connection that stops sending: no Node.js download takes
            // this long, even on a slow line.
            .timeout_recv_body(Some(Duration::from_secs(30 * 60)))
            .user_agent(concat!("sne/", env!("CARGO_PKG_VERSION")))
            .https_only(https_only)
            .tls_config(TlsConfig::builder().root_certs(root_certs()).build())
            .build()
            .into()
    };
    Ok(match transport(url)? {
        Transport::Secure => SECURE.get_or_init(|| build(true)),
        Transport::Loopback => LOOPBACK.get_or_init(|| build(false)),
    })
}

/// Waits before each new attempt. A dropped connection or a busy server (nodejs.org answers 503
/// now and then) often works a moment later, and one hiccup should not fail an install.
const RETRY_AFTER: [Duration; 2] = [Duration::from_millis(500), Duration::from_secs(2)];

/// Whether trying again can help. A missing file (404) or a refused certificate will not change.
fn is_transient(err: &ureq::Error) -> bool {
    match err {
        ureq::Error::StatusCode(code) => *code == 408 || *code == 429 || *code >= 500,
        ureq::Error::Io(_)
        | ureq::Error::Timeout(_)
        | ureq::Error::ConnectionFailed
        | ureq::Error::HostNotFound
        | ureq::Error::BodyStalled => true,
        _ => false,
    }
}

/// Runs `attempt` again after a short wait while it fails in a way that may pass.
fn with_retries<T>(mut attempt: impl FnMut() -> Result<T, ureq::Error>) -> Result<T, ureq::Error> {
    let mut waits = RETRY_AFTER.iter();
    loop {
        match attempt() {
            Err(err) if is_transient(&err) => match waits.next() {
                Some(wait) => std::thread::sleep(*wait),
                None => return Err(err),
            },
            done => return done,
        }
    }
}

pub fn fetch_text(url: &str) -> Result<String> {
    let agent = agent(url)?;
    with_retries(|| agent.get(url).call()?.body_mut().read_to_string())
        .with_context(|| format!("request to {url} failed"))
}

/// Conditional GET. Returns `None` when the server answers 304 Not Modified.
pub fn fetch_if_changed(url: &str, etag: Option<&str>) -> Result<Option<(String, Option<String>)>> {
    let agent = agent(url)?;
    let fetched = with_retries(|| {
        let mut request = agent.get(url);
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
    })?;
    Ok(fetched)
}

/// Opens a streaming download. Returns the body and its length, when the server reports one.
/// Starting the download is retried; once data flows, a failure ends it, and the caller cleans up.
pub fn open(url: &str) -> Result<(impl Read + use<>, Option<u64>)> {
    let agent = agent(url)?;
    let response =
        with_retries(|| agent.get(url).call()).with_context(|| format!("download of {url} failed"))?;
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
    use super::{Transport, fetch_text, transport};

    /// A server on this machine that answers each connection with the next of `answers`.
    fn flaky_server(answers: Vec<&'static str>) -> String {
        use std::io::{BufRead, BufReader, Write};
        let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let port = listener.local_addr().unwrap().port();
        std::thread::spawn(move || {
            for (stream, answer) in listener.incoming().zip(answers) {
                let mut stream = stream.unwrap();
                let mut reader = BufReader::new(stream.try_clone().unwrap());
                let mut line = String::new();
                while reader.read_line(&mut line).is_ok_and(|n| n > 2) {
                    line.clear();
                }
                let _ = stream.write_all(answer.as_bytes());
            }
        });
        format!("http://127.0.0.1:{port}/index.json")
    }

    #[test]
    fn a_busy_server_is_asked_again() {
        let busy = "HTTP/1.1 503 Service Unavailable
Content-Length: 0
Connection: close

";
        let ok = "HTTP/1.1 200 OK
Content-Length: 2
Connection: close

ok";
        assert_eq!(fetch_text(&flaky_server(vec![busy, ok])).unwrap(), "ok");
        // Three times busy: give up, with the reason.
        let err = fetch_text(&flaky_server(vec![busy, busy, busy])).unwrap_err();
        assert!(format!("{err:#}").contains("503"), "{err:#}");
    }

    #[test]
    fn a_missing_file_is_not_asked_again() {
        let missing = "HTTP/1.1 404 Not Found
Content-Length: 0
Connection: close

";
        let ok = "HTTP/1.1 200 OK
Content-Length: 2
Connection: close

ok";
        let err = fetch_text(&flaky_server(vec![missing, ok])).unwrap_err();
        assert!(format!("{err:#}").contains("404"), "{err:#}");
    }

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
