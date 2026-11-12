use anyhow::{Result, bail};

use crate::version::Version;

/// What the user asked for: `22`, `22.3`, `v22.3.1`, `lts`, `lts/iron`, `latest`, `>=20 <23`.
#[derive(Debug)]
pub enum Spec {
    Latest,
    Lts(Option<String>),
    Partial(u32, Option<u32>, Option<u32>),
    Range(node_semver::Range),
}

impl Spec {
    pub fn parse(input: &str) -> Result<Self> {
        let s = input.trim();
        let lower = s.to_ascii_lowercase();
        match lower.as_str() {
            "" => bail!("empty version"),
            "latest" | "node" | "stable" => return Ok(Spec::Latest),
            "lts" | "lts/*" | "lts/latest" | "--lts" => return Ok(Spec::Lts(None)),
            _ => {}
        }
        if let Some(name) = lower.strip_prefix("lts/").or_else(|| lower.strip_prefix("lts-")) {
            return Ok(Spec::Lts(Some(name.to_string())));
        }
        if let Some(partial) = parse_partial(lower.strip_prefix('v').unwrap_or(&lower)) {
            return Ok(partial);
        }
        match node_semver::Range::parse(s) {
            Ok(range) => Ok(Spec::Range(range)),
            Err(_) => bail!("`{input}` is not a valid version, alias or range"),
        }
    }

    /// `lts` is the release's LTS codename, if it has one.
    pub fn matches(&self, v: Version, lts: Option<&str>) -> bool {
        match self {
            Spec::Latest => true,
            Spec::Lts(None) => lts.is_some(),
            Spec::Lts(Some(name)) => lts.is_some_and(|l| l.eq_ignore_ascii_case(name)),
            Spec::Partial(major, minor, patch) => {
                v.major == *major && minor.is_none_or(|m| m == v.minor) && patch.is_none_or(|p| p == v.patch)
            }
            Spec::Range(range) => {
                let text = format!("{}.{}.{}", v.major, v.minor, v.patch);
                node_semver::Version::parse(&text).is_ok_and(|sv| range.satisfies(&sv))
            }
        }
    }

    pub fn needs_lts_info(&self) -> bool {
        matches!(self, Spec::Lts(_))
    }

    /// Highest version among `candidates` that satisfies this spec.
    pub fn pick<'a>(
        &self,
        candidates: impl IntoIterator<Item = (Version, Option<&'a str>)>,
    ) -> Option<Version> {
        candidates
            .into_iter()
            .filter(|(v, lts)| self.matches(*v, *lts))
            .map(|(v, _)| v)
            .max()
    }
}

fn parse_partial(s: &str) -> Option<Spec> {
    let mut nums = [None; 3];
    for (i, part) in s.split('.').enumerate() {
        if i == 3 || part.is_empty() || !part.bytes().all(|b| b.is_ascii_digit()) {
            return None;
        }
        nums[i] = Some(part.parse().ok()?);
    }
    Some(Spec::Partial(nums[0]?, nums[1], nums[2]))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn v(s: &str) -> Version {
        Version::parse(s).unwrap()
    }

    fn releases() -> Vec<(Version, Option<&'static str>)> {
        vec![
            (v("24.1.0"), None),
            (v("22.11.0"), Some("Jod")),
            (v("22.3.0"), None),
            (v("20.18.0"), Some("Iron")),
            (v("20.9.0"), Some("Iron")),
            (v("18.20.4"), Some("Hydrogen")),
        ]
    }

    fn pick(spec: &str) -> Option<Version> {
        Spec::parse(spec).unwrap().pick(releases())
    }

    #[test]
    fn partial_versions() {
        assert_eq!(pick("22"), Some(v("22.11.0")));
        assert_eq!(pick("v22.3"), Some(v("22.3.0")));
        assert_eq!(pick("20.9.0"), Some(v("20.9.0")));
        assert_eq!(pick("19"), None);
    }

    #[test]
    fn keywords() {
        assert_eq!(pick("latest"), Some(v("24.1.0")));
        assert_eq!(pick("lts"), Some(v("22.11.0")));
        assert_eq!(pick("lts/*"), Some(v("22.11.0")));
        assert_eq!(pick("lts/iron"), Some(v("20.18.0")));
        assert_eq!(pick("lts/Hydrogen"), Some(v("18.20.4")));
    }

    #[test]
    fn ranges() {
        assert_eq!(pick(">=20 <23"), Some(v("22.11.0")));
        assert_eq!(pick("^20.0.0"), Some(v("20.18.0")));
        assert_eq!(pick("22.x"), Some(v("22.11.0")));
        assert_eq!(pick(">=18"), Some(v("24.1.0")));
    }

    #[test]
    fn rejects_garbage() {
        assert!(Spec::parse("not a version!").is_err());
        assert!(Spec::parse("").is_err());
    }
}
