use std::fs;
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

fn in_dir(dir: &Path) -> Option<Found> {
    for name in [".node-version", ".nvmrc"] {
        let path = dir.join(name);
        if let Some(spec) = fs::read_to_string(&path).ok().as_deref().and_then(parse_plain) {
            return Some(Found { spec, path });
        }
    }
    let path = dir.join("package.json");
    let spec = fs::read_to_string(&path)
        .ok()
        .as_deref()
        .and_then(parse_package_json)?;
    Some(Found { spec, path })
}

/// First line that is not blank or a `#` comment.
fn parse_plain(content: &str) -> Option<String> {
    content
        .lines()
        .map(|line| line.split('#').next().unwrap_or("").trim())
        .find(|line| !line.is_empty())
        .map(str::to_string)
}

fn parse_package_json(content: &str) -> Option<String> {
    let json: serde_json::Value = serde_json::from_str(content).ok()?;
    let spec = json.get("engines")?.get("node")?.as_str()?.trim();
    (!spec.is_empty()).then(|| spec.to_string())
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
