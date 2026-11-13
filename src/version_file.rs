use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};

/// A version request found in a project file.
#[derive(Debug, PartialEq)]
pub struct Found {
    pub spec: String,
    pub path: PathBuf,
}

/// Walks up from `start` looking for `.node-version`, `.nvmrc` or `engines.node` in `package.json`.
pub fn find(start: &Path) -> Option<Found> {
    start.ancestors().find_map(in_dir)
}

/// Largest `.nvmrc` or `.node-version` that is read. Real ones are a few bytes.
const PLAIN_LIMIT: u64 = 64 * 1024;
/// Largest `package.json` that is read.
const PACKAGE_JSON_LIMIT: u64 = 1024 * 1024;
/// Longest version request that is accepted. `>=20.11.0 <21 || >=22.3.0 <23` is 31 characters.
const SPEC_LIMIT: usize = 100;

fn in_dir(dir: &Path) -> Option<Found> {
    for name in [".node-version", ".nvmrc"] {
        let path = dir.join(name);
        if let Some(spec) = read(&path, PLAIN_LIMIT).as_deref().and_then(parse_plain) {
            return Some(Found { spec, path });
        }
    }
    let path = dir.join("package.json");
    let spec = read(&path, PACKAGE_JSON_LIMIT)
        .as_deref()
        .and_then(parse_package_json)?;
    Some(Found { spec, path })
}

/// Reads a project file, which may come from a repository that someone else wrote. This runs on
/// every `cd`, so it must never hang or follow a link out of the project.
///
/// Only a regular file is read: a symbolic link could point at a private file, whose first line
/// would then be printed, and a pipe or device would block the shell. Large files are skipped.
fn read(path: &Path, limit: u64) -> Option<String> {
    let meta = fs::symlink_metadata(path).ok()?;
    if !meta.is_file() || meta.len() > limit {
        return None;
    }
    let mut content = String::new();
    fs::File::open(path)
        .ok()?
        .take(limit)
        .read_to_string(&mut content)
        .ok()?;
    Some(content)
}

/// A version request is printed back to the terminal, so control characters, which can move the
/// cursor or change the window title, are refused, and so is anything too long to be a version.
fn accept(spec: &str) -> Option<String> {
    let spec = spec.trim();
    let fine = !spec.is_empty() && spec.chars().count() <= SPEC_LIMIT && !spec.chars().any(char::is_control);
    fine.then(|| spec.to_string())
}

/// First line that is not blank or a `#` comment.
fn parse_plain(content: &str) -> Option<String> {
    content
        .lines()
        .map(|line| line.split('#').next().unwrap_or("").trim())
        .find(|line| !line.is_empty())
        .and_then(accept)
}

fn parse_package_json(content: &str) -> Option<String> {
    let json: serde_json::Value = serde_json::from_str(content).ok()?;
    accept(json.get("engines")?.get("node")?.as_str()?)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plain_files_skip_comments_and_blank_lines() {
        assert_eq!(
            parse_plain("# pinned\n\n  v22.11.0  # lts\n"),
            Some("v22.11.0".into())
        );
        assert_eq!(parse_plain("lts/iron\r\n"), Some("lts/iron".into()));
        assert_eq!(parse_plain("\n# nothing\n"), None);
    }

    #[test]
    fn package_json_engines() {
        assert_eq!(
            parse_package_json(r#"{"name":"x","engines":{"node":">=20 <23"}}"#),
            Some(">=20 <23".into())
        );
        assert_eq!(parse_package_json(r#"{"name":"x"}"#), None);
        assert_eq!(parse_package_json("not json"), None);
    }

    #[test]
    fn refuses_requests_that_are_not_versions() {
        // Escape codes would be printed to the terminal on `cd`.
        assert_eq!(parse_plain("22\u{1b}]0;owned\u{7}\n"), None);
        assert_eq!(
            parse_package_json("{\"engines\":{\"node\":\"\\u001b[2J20\"}}"),
            None
        );
        assert_eq!(parse_plain(&"9".repeat(SPEC_LIMIT + 1)), None);
        assert_eq!(parse_plain(&"9".repeat(SPEC_LIMIT)), Some("9".repeat(SPEC_LIMIT)));
        assert_eq!(
            parse_plain(">=20.11.0 <21 || >=22.3.0 <23"),
            Some(">=20.11.0 <21 || >=22.3.0 <23".into())
        );
    }

    #[test]
    fn reads_only_small_regular_files() {
        let root = std::env::temp_dir().join(format!("sne-vf-read-{}", std::process::id()));
        fs::create_dir_all(&root).unwrap();

        let secret = root.join("secret.txt");
        fs::write(&secret, "22\n").unwrap();
        assert_eq!(read(&secret, PLAIN_LIMIT).as_deref(), Some("22\n"));

        // A link is not followed, even to a file that would be accepted.
        let link = root.join(".nvmrc");
        #[cfg(unix)]
        let linked = std::os::unix::fs::symlink(&secret, &link).is_ok();
        // Creating a symbolic link on Windows needs a privilege most users do not have.
        #[cfg(windows)]
        let linked = std::os::windows::fs::symlink_file(&secret, &link).is_ok();
        if linked {
            assert_eq!(read(&link, PLAIN_LIMIT), None);
            assert_eq!(in_dir(&root), None);
        }

        let big = root.join("big");
        fs::write(&big, vec![b'1'; 200]).unwrap();
        assert_eq!(read(&big, 100), None);
        assert_eq!(read(&root, PLAIN_LIMIT), None, "a directory is not a file");
        assert_eq!(read(&root.join("missing"), PLAIN_LIMIT), None);

        fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    fn nearest_file_wins_when_walking_up() {
        let root = std::env::temp_dir().join(format!("sne-vf-test-{}", std::process::id()));
        let nested = root.join("a").join("b");
        fs::create_dir_all(&nested).unwrap();
        fs::write(root.join(".nvmrc"), "20\n").unwrap();
        assert_eq!(find(&nested).unwrap().spec, "20");
        fs::write(root.join("a").join(".node-version"), "22\n").unwrap();
        assert_eq!(find(&nested).unwrap().spec, "22");
        fs::remove_dir_all(&root).unwrap();
    }
}
