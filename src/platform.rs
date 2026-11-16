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

/// CPU types that Node.js publishes builds for, by their name on nodejs.org.
const KNOWN_ARCHES: [&str; 7] = ["x64", "arm64", "x86", "armv7l", "ppc64le", "s390x", "riscv64"];

/// The processor sne was built for, by its name on nodejs.org.
fn native_arch() -> &'static str {
    match std::env::consts::ARCH {
        "x86_64" => "x64",
        "aarch64" => "arm64",
        "x86" => "x86",
        "arm" => "armv7l",
        "powerpc64" => "ppc64le",
        other => other,
    }
}

/// Whether the user chose the processor type with `SNE_ARCH`.
pub fn arch_is_chosen() -> bool {
    std::env::var_os("SNE_ARCH").is_some_and(|v| !v.is_empty())
}

/// Processor type of the Node builds to install. `SNE_ARCH` chooses another one, for example
/// `x64` on an Apple silicon Mac.
pub fn arch() -> anyhow::Result<&'static str> {
    let Ok(wanted) = std::env::var("SNE_ARCH") else {
        return Ok(native_arch());
    };
    let wanted = wanted.trim();
    if wanted.is_empty() {
        return Ok(native_arch());
    }
    KNOWN_ARCHES
        .into_iter()
        .find(|known| known.eq_ignore_ascii_case(wanted))
        .ok_or_else(|| {
            anyhow::anyhow!(
                "SNE_ARCH is `{wanted}`, which is not a processor type of Node.js. Use one of: {}",
                KNOWN_ARCHES.join(", ")
            )
        })
}

/// Another build that runs on this system when Node has none for its own processor. Node 15 and
/// older have no build for Apple silicon, and versions before Node 20 mostly none for Windows on ARM. Both
/// systems run x64 programs: macOS with Rosetta 2, Windows 11 with its built-in emulation.
pub fn emulated_arch(os: &str, arch: &str) -> Option<&'static str> {
    match (os, arch) {
        ("darwin" | "win", "arm64") => Some("x64"),
        _ => None,
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
    fn emulation_only_where_the_system_provides_it() {
        assert_eq!(emulated_arch("darwin", "arm64"), Some("x64"));
        assert_eq!(emulated_arch("win", "arm64"), Some("x64"));
        assert_eq!(emulated_arch("linux", "arm64"), None);
        assert_eq!(emulated_arch("darwin", "x64"), None);
    }

    #[test]
    fn index_keys_match_nodejs_dist() {
        assert_eq!(index_key("win", "arm64", false), "win-arm64-zip");
        assert_eq!(index_key("darwin", "x64", false), "osx-x64-tar");
        assert_eq!(index_key("linux", "arm64", false), "linux-arm64");
        assert_eq!(index_key("linux", "x64", true), "linux-x64-musl");
    }
}
