use crate::version::Version;

const OFFICIAL_MIRROR: &str = "https://nodejs.org/dist";
const MUSL_MIRROR: &str = "https://unofficial-builds.nodejs.org/download/release";

pub fn os() -> &'static str {
    match std::env::consts::OS {
        "windows" => "win",
        "macos" => "darwin",
        other => other,
    }
}

pub fn arch() -> &'static str {
    match std::env::consts::ARCH {
        "x86_64" => "x64",
        "aarch64" => "arm64",
        "x86" => "x86",
        "arm" => "armv7l",
        "powerpc64" => "ppc64le",
        other => other,
    }
}

/// Detected at run time, because a static sne binary can run on both glibc and musl systems.
pub fn is_musl() -> bool {
    if !cfg!(target_os = "linux") {
        return false;
    }
    let has = |dir: &str, prefix: &str| {
        std::fs::read_dir(dir).is_ok_and(|entries| {
            entries
                .flatten()
                .any(|e| e.file_name().to_string_lossy().starts_with(prefix))
        })
    };
    has("/lib", "ld-musl-") && !has("/lib", "ld-linux") && !has("/lib64", "ld-linux")
}

/// File name of the Node.js archive on the mirror.
pub fn archive_name(os: &str, arch: &str, musl: bool, v: Version) -> String {
    match os {
        "win" => format!("node-{v}-win-{arch}.zip"),
        "linux" if musl => format!("node-{v}-linux-{arch}-musl.tar.xz"),
        _ => format!("node-{v}-{os}-{arch}.tar.xz"),
    }
}

/// Entry in the `files` list of `index.json` that proves a build exists for this platform.
pub fn index_key(os: &str, arch: &str, musl: bool) -> String {
    match os {
        "win" => format!("win-{arch}-zip"),
        "darwin" => format!("osx-{arch}-tar"),
        "linux" if musl => format!("linux-{arch}-musl"),
        _ => format!("{os}-{arch}"),
    }
}

/// Base URL for downloads, without a trailing slash.
pub fn mirror() -> String {
    match std::env::var("SNE_NODE_MIRROR") {
        Ok(url) if !url.trim().is_empty() => url.trim().trim_end_matches('/').to_string(),
        _ if is_musl() => MUSL_MIRROR.to_string(),
        _ => OFFICIAL_MIRROR.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn archive_names_match_nodejs_dist() {
        let v = Version::parse("22.11.0").unwrap();
        assert_eq!(archive_name("win", "x64", false, v), "node-v22.11.0-win-x64.zip");
        assert_eq!(
            archive_name("darwin", "arm64", false, v),
            "node-v22.11.0-darwin-arm64.tar.xz"
        );
        assert_eq!(
            archive_name("linux", "x64", false, v),
            "node-v22.11.0-linux-x64.tar.xz"
        );
        assert_eq!(
            archive_name("linux", "x64", true, v),
            "node-v22.11.0-linux-x64-musl.tar.xz"
        );
    }

    #[test]
    fn index_keys_match_nodejs_dist() {
        assert_eq!(index_key("win", "arm64", false), "win-arm64-zip");
        assert_eq!(index_key("darwin", "x64", false), "osx-x64-tar");
        assert_eq!(index_key("linux", "arm64", false), "linux-arm64");
        assert_eq!(index_key("linux", "x64", true), "linux-x64-musl");
    }
}
