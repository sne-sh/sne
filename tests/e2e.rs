//! End-to-end test: runs the real binary against a local fake mirror.

use std::collections::HashMap;
use std::fs;
use std::io::{Read, Write};
use std::net::TcpListener;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use sha2::{Digest, Sha256};

fn arch() -> &'static str {
    match std::env::consts::ARCH {
        "x86_64" => "x64",
        "aarch64" => "arm64",
        other => other,
    }
}

/// Name and bytes of a tiny fake Node archive in the platform's real format.
#[cfg(windows)]
fn fake_archive(version: &str) -> (String, Vec<u8>) {
    use zip::write::SimpleFileOptions;

    let top = format!("node-{version}-win-{}", arch());
    let mut zip = zip::ZipWriter::new(std::io::Cursor::new(Vec::new()));
    for (path, body) in [
        ("node.exe", version),
        ("npm.cmd", "@echo npm"),
        ("node_modules/npm/package.json", "{}"),
    ] {
        zip.start_file(format!("{top}/{path}"), SimpleFileOptions::default())
            .unwrap();
        zip.write_all(body.as_bytes()).unwrap();
    }
    (format!("{top}.zip"), zip.finish().unwrap().into_inner())
}

#[cfg(not(windows))]
fn fake_archive(version: &str) -> (String, Vec<u8>) {
    let os = if cfg!(target_os = "macos") {
        "darwin"
    } else {
        "linux"
    };
    let top = format!("node-{version}-{os}-{}", arch());
    let mut tar = tar::Builder::new(liblzma::write::XzEncoder::new(Vec::new(), 1));
    for (path, body) in [("bin/node", version), ("lib/node_modules/npm/package.json", "{}")] {
        let mut header = tar::Header::new_gnu();
        header.set_size(body.len() as u64);
        header.set_mode(0o755);
        header.set_cksum();
        tar.append_data(&mut header, format!("{top}/{path}"), body.as_bytes())
            .unwrap();
    }
    (
        format!("{top}.tar.xz"),
        tar.into_inner().unwrap().finish().unwrap(),
    )
}

fn sha256(bytes: &[u8]) -> String {
    Sha256::digest(bytes).iter().map(|b| format!("{b:02x}")).collect()
}

/// Serves `files` over HTTP on a random local port and returns the base URL.
fn serve(files: HashMap<String, Vec<u8>>) -> String {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let url = format!("http://{}", listener.local_addr().unwrap());
    std::thread::spawn(move || {
        for stream in listener.incoming() {
            let Ok(mut stream) = stream else { continue };
            let mut request = Vec::new();
            let mut buf = [0u8; 1024];
            while !request.windows(4).any(|w| w == b"\r\n\r\n") {
                match stream.read(&mut buf) {
                    Ok(0) | Err(_) => break,
                    Ok(n) => request.extend_from_slice(&buf[..n]),
                }
            }
            let request = String::from_utf8_lossy(&request);
            let path = request.split_whitespace().nth(1).unwrap_or("/");
            let _ = match files.get(path) {
                Some(body) => stream
                    .write_all(
                        format!(
                            "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
                            body.len()
                        )
                        .as_bytes(),
                    )
                    .and_then(|()| stream.write_all(body)),
                None => stream
                    .write_all(b"HTTP/1.1 404 Not Found\r\nContent-Length: 0\r\nConnection: close\r\n\r\n"),
            };
        }
    });
    url
}

struct Env {
    data: PathBuf,
    mirror: String,
    link: PathBuf,
}

impl Env {
    fn sne(&self, cwd: &Path, args: &[&str]) -> Output {
        Command::new(env!("CARGO_BIN_EXE_sne"))
            .args(args)
            .current_dir(cwd)
            .env("SNE_DIR", &self.data)
            .env("SNE_NODE_MIRROR", &self.mirror)
            .env("SNE_SHELL_LINK", &self.link)
            .env("SNE_NO_UPDATE_CHECK", "1")
            .output()
            .unwrap()
    }

    fn ok(&self, cwd: &Path, args: &[&str]) -> String {
        let out = self.sne(cwd, args);
        assert!(
            out.status.success(),
            "sne {args:?} failed: {}",
            String::from_utf8_lossy(&out.stderr)
        );
        String::from_utf8_lossy(&out.stdout).into_owned()
    }
}

#[test]
fn install_switch_and_uninstall() {
    let mut files = HashMap::new();
    let mut index = Vec::new();
    for (version, lts, corrupt) in [
        ("v22.1.0", "false", false),
        ("v20.5.0", "\"Iron\"", false),
        ("v18.0.0", "\"Hydrogen\"", true),
    ] {
        let (name, bytes) = fake_archive(version);
        let sum = if corrupt { "0".repeat(64) } else { sha256(&bytes) };
        files.insert(
            format!("/{version}/SHASUMS256.txt"),
            format!("{sum}  {name}\n").into_bytes(),
        );
        files.insert(format!("/{version}/{name}"), bytes);
        index.push(format!(
            r#"{{"version":"{version}","date":"2024-01-01","files":[],"lts":{lts}}}"#
        ));
    }
    files.insert(
        "/index.json".into(),
        format!("[{}]", index.join(",")).into_bytes(),
    );

    // A space and a letter outside ASCII, as in many user folder names.
    let root = Path::new(env!("CARGO_TARGET_TMPDIR")).join(format!("e2e señor {}", std::process::id()));
    let _ = fs::remove_dir_all(&root);
    let project = root.join("project").join("nested");
    fs::create_dir_all(&project).unwrap();
    let env = Env {
        data: root.join("data"),
        mirror: serve(files),
        link: root.join("data").join("shells").join("test"),
    };
    let node = if cfg!(windows) {
        PathBuf::from("node.exe")
    } else {
        Path::new("bin").join("node")
    };
    let version_dir = |v: &str| env.data.join("versions").join(v);

    // Two versions in one command; the first one becomes the default.
    let out = env.ok(&root, &["install", "22", "lts/iron"]);
    assert!(
        out.contains("Installed Node v22.1.0") && out.contains("Installed Node v20.5.0"),
        "{out}"
    );
    assert!(out.contains("v22.1.0 is now the default"), "{out}");
    assert_eq!(
        fs::read_to_string(version_dir("v22.1.0").join(&node)).unwrap(),
        "v22.1.0"
    );
    assert!(env.ok(&root, &["install", "22"]).contains("already installed"));

    // A wrong checksum installs nothing and leaves no temporary files behind.
    let out = env.sne(&root, &["install", "18"]);
    assert!(!out.status.success());
    assert!(String::from_utf8_lossy(&out.stderr).contains("checksum mismatch"));
    let leftovers: Vec<_> = fs::read_dir(env.data.join("versions"))
        .unwrap()
        .flatten()
        .map(|e| e.file_name())
        .collect();
    assert_eq!(leftovers.len(), 2, "{leftovers:?}");

    // Manual switching, by version and by LTS name.
    assert_eq!(env.ok(&root, &["current"]).trim(), "none");
    env.ok(&root, &["use", "20"]);
    assert_eq!(env.ok(&root, &["current"]).trim(), "v20.5.0");
    assert_eq!(
        fs::read_to_string(env.link.join(node.file_name().unwrap())).unwrap(),
        "v20.5.0"
    );
    env.ok(&root, &["use", "latest"]);
    assert_eq!(env.ok(&root, &["current"]).trim(), "v22.1.0");
    env.ok(&root, &["use", "lts"]);
    assert_eq!(env.ok(&root, &["current"]).trim(), "v20.5.0");
    assert!(!env.sne(&root, &["use", "16"]).status.success());

    // The cd hook follows the project's version file, found from a nested directory.
    env.ok(&root, &["use", "22"]);
    fs::write(root.join("project").join(".nvmrc"), "20\n").unwrap();
    assert!(
        env.ok(&project, &["use", "--auto"])
            .contains("Using Node v20.5.0")
    );
    assert_eq!(
        env.ok(&project, &["use", "--auto"]),
        "",
        "second run must be silent"
    );
    assert_eq!(
        env.ok(&root, &["use", "--auto"]),
        "",
        "no version file: nothing changes"
    );
    assert_eq!(env.ok(&root, &["current"]).trim(), "v20.5.0");

    // A version file with terminal escape codes is ignored, so nothing from it is printed.
    fs::write(root.join("project").join(".nvmrc"), "22\u{1b}]0;owned\u{7}\n").unwrap();
    let out = env.sne(&project, &["use", "--auto"]);
    assert!(out.stdout.is_empty() && out.stderr.is_empty(), "{out:?}");
    let out = env.sne(&project, &["use"]);
    assert!(!out.status.success());
    assert!(!String::from_utf8_lossy(&out.stderr).contains('\u{1b}'));
    fs::write(root.join("project").join(".nvmrc"), "20\n").unwrap();

    // Aliases, default, listing.
    env.ok(&root, &["alias", "work", "22"]);
    env.ok(&root, &["default", "20"]);
    let list = env.ok(&root, &["ls"]);
    assert!(
        list.contains("  v22.1.0  (work)") && list.contains("* v20.5.0  (default)"),
        "{list}"
    );
    assert!(env.ok(&root, &["which", "work"]).contains("v22.1.0"));
    assert!(
        !env.sne(&root, &["alias", "22", "20"]).status.success(),
        "a version is not a valid alias name"
    );
    let remote = env.ok(&root, &["ls-remote", "--lts"]);
    assert!(
        remote.contains("v20.5.0") && remote.contains("LTS: Iron") && !remote.contains("v22.1.0"),
        "{remote}"
    );

    // Shell setup points new shells at the default version.
    let setup = env.ok(&root, &["env", "--shell", "bash", "--use-on-cd"]);
    assert!(
        setup.contains("export SNE_SHELL_LINK=") && setup.contains("__sne_hook"),
        "{setup}"
    );

    // Without the setup line, `sne use` switches the default, which every terminal uses.
    // SNE_DIR is set, so the real PATH of whoever runs the tests is left alone.
    let out = Command::new(env!("CARGO_BIN_EXE_sne"))
        .args(["use", "22"])
        .current_dir(&root)
        .env("SNE_DIR", &env.data)
        .env_remove("SNE_SHELL_LINK")
        .output()
        .unwrap();
    let said = String::from_utf8_lossy(&out.stdout);
    assert!(out.status.success(), "{}", String::from_utf8_lossy(&out.stderr));
    assert!(said.contains("Using Node v22.1.0 in every terminal"), "{said}");
    assert!(said.contains("to your PATH"), "{said}");
    assert_eq!(
        fs::read_to_string(env.data.join("aliases").join("default").join(&node)).unwrap(),
        "v22.1.0"
    );
    assert_eq!(
        env.ok(&root, &["current"]).trim(),
        "v20.5.0",
        "this shell keeps its version"
    );

    // Uninstalling removes the version and the aliases that pointed to it.
    let out = env.ok(&root, &["uninstall", "22"]);
    assert!(out.contains("Removed alias work"), "{out}");
    assert!(out.contains("Removed alias default"), "{out}");
    assert!(out.contains("No default version is set now"), "{out}");
    assert!(!version_dir("v22.1.0").exists());
    assert!(version_dir("v20.5.0").join(&node).exists());
    let doctor = env.ok(&root, &["doctor"]);
    assert!(doctor.contains("default: not set"), "{doctor}");

    fs::remove_dir_all(&root).unwrap();
}

#[test]
fn self_check_reports_a_newer_release() {
    let run = |tag: &str| {
        let body = format!(r#"{{"tag_name":"{tag}","name":"sne"}}"#).into_bytes();
        let url = serve(HashMap::from([("/latest".to_string(), body)]));
        let data = std::env::temp_dir().join(format!("sne-e2e-self-{}-{tag}", std::process::id()));
        let _ = fs::remove_dir_all(&data);
        let out = Command::new(env!("CARGO_BIN_EXE_sne"))
            .args(["self", "check"])
            .env("SNE_DIR", &data)
            .env("SNE_RELEASES_URL", format!("{url}/latest"))
            .output()
            .unwrap();
        assert!(out.status.success(), "{}", String::from_utf8_lossy(&out.stderr));
        let cached = fs::read_to_string(data.join("cache").join("sne-latest.txt")).unwrap();
        let _ = fs::remove_dir_all(&data);
        (String::from_utf8_lossy(&out.stdout).into_owned(), cached)
    };

    let (stdout, cached) = run("999.0.0");
    assert!(stdout.contains("sne 999.0.0 is available"), "{stdout}");
    assert_eq!(cached, "999.0.0");

    // Tags with a leading `v` are understood too.
    let (stdout, cached) = run("v999.1.0-beta.2");
    assert!(stdout.contains("sne 999.1.0-beta.2 is available"), "{stdout}");
    assert_eq!(cached, "999.1.0-beta.2");

    let (stdout, _) = run("0.0.1");
    assert!(stdout.contains("is the latest version"), "{stdout}");
}

/// Name of the release build for the machine running the tests, as `sne update` works it out.
fn release_target() -> Option<&'static str> {
    Some(match (std::env::consts::OS, std::env::consts::ARCH) {
        ("windows", "x86_64") => "x86_64-pc-windows-msvc",
        ("windows", "x86") => "i686-pc-windows-msvc",
        ("windows", "aarch64") => "aarch64-pc-windows-msvc",
        ("macos", "x86_64") => "x86_64-apple-darwin",
        ("macos", "aarch64") => "aarch64-apple-darwin",
        ("linux", "x86_64") => "x86_64-unknown-linux-musl",
        ("linux", "aarch64") => "aarch64-unknown-linux-musl",
        _ => return None,
    })
}

/// A release archive in the platform's format that holds `program` as the sne binary.
#[cfg(windows)]
fn release_archive(folder: &str, program: &[u8], _scratch: &Path) -> (String, Vec<u8>) {
    let mut zip = zip::ZipWriter::new(std::io::Cursor::new(Vec::new()));
    zip.start_file(
        format!("{folder}/sne.exe"),
        zip::write::SimpleFileOptions::default(),
    )
    .unwrap();
    zip.write_all(program).unwrap();
    (format!("{folder}.zip"), zip.finish().unwrap().into_inner())
}

#[cfg(not(windows))]
fn release_archive(folder: &str, program: &[u8], scratch: &Path) -> (String, Vec<u8>) {
    let stage = scratch.join("stage");
    fs::create_dir_all(stage.join(folder)).unwrap();
    fs::write(stage.join(folder).join("sne"), program).unwrap();
    let archive = scratch.join(format!("{folder}.tar.gz"));
    let status = Command::new("tar")
        .arg("-czf")
        .arg(&archive)
        .arg("-C")
        .arg(&stage)
        .arg(folder)
        .status()
        .unwrap();
    assert!(status.success());
    (format!("{folder}.tar.gz"), fs::read(archive).unwrap())
}

#[test]
fn update_replaces_the_program_after_checking_it() {
    let Some(target) = release_target() else { return };
    let scratch = std::env::temp_dir().join(format!("sne-e2e-update-{}", std::process::id()));
    let _ = fs::remove_dir_all(&scratch);
    fs::create_dir_all(&scratch).unwrap();

    // `sne update` replaces the program that runs it, so it gets its own copy.
    let exe = scratch.join(if cfg!(windows) { "sne.exe" } else { "sne" });
    fs::copy(env!("CARGO_BIN_EXE_sne"), &exe).unwrap();
    let original = fs::read(&exe).unwrap();

    let tag = "999.0.0";
    let new_program = b"the new sne".to_vec();
    let (archive, bytes) = release_archive(&format!("sne-{tag}-{target}"), &new_program, &scratch);
    let latest = format!(r#"{{"tag_name":"{tag}"}}"#).into_bytes();

    let update = |sums: String| {
        let url = serve(HashMap::from([
            ("/latest".to_string(), latest.clone()),
            (format!("/{tag}/{archive}"), bytes.clone()),
            (format!("/{tag}/SHA256SUMS.txt"), sums.into_bytes()),
        ]));
        Command::new(&exe)
            .arg("update")
            .env("SNE_DIR", scratch.join("data"))
            .env("SNE_RELEASES_URL", format!("{url}/latest"))
            .env("SNE_DOWNLOAD_BASE", &url)
            .output()
            .unwrap()
    };

    // A wrong checksum must leave the program alone.
    let out = update(format!("{}  {archive}\n", "0".repeat(64)));
    assert!(!out.status.success());
    assert!(String::from_utf8_lossy(&out.stderr).contains("checksum mismatch"));
    assert_eq!(fs::read(&exe).unwrap(), original);

    let out = update(format!("{}  {archive}\n", sha256(&bytes)));
    assert!(out.status.success(), "{}", String::from_utf8_lossy(&out.stderr));
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("to 999.0.0"), "{stdout}");
    assert_eq!(fs::read(&exe).unwrap(), new_program);

    let _ = fs::remove_dir_all(&scratch);
}
