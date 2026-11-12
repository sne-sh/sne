use std::fmt;

/// An exact Node.js release version such as `v22.11.0`.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct Version {
    pub major: u32,
    pub minor: u32,
    pub patch: u32,
}

impl Version {
    /// Parses `v22.11.0` or `22.11.0`. Partial versions return `None`.
    pub fn parse(s: &str) -> Option<Self> {
        let s = s.trim();
        let s = s.strip_prefix('v').unwrap_or(s);
        let mut parts = s.split('.');
        let major = parts.next()?.parse().ok()?;
        let minor = parts.next()?.parse().ok()?;
        let patch = parts.next()?.parse().ok()?;
        if parts.next().is_some() {
            return None;
        }
        Some(Self { major, minor, patch })
    }
}

impl fmt::Display for Version {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "v{}.{}.{}", self.major, self.minor, self.patch)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_exact_versions_only() {
        let v = Version {
            major: 22,
            minor: 11,
            patch: 0,
        };
        assert_eq!(Version::parse("v22.11.0"), Some(v));
        assert_eq!(Version::parse("22.11.0"), Some(v));
        assert_eq!(Version::parse("22.11"), None);
        assert_eq!(Version::parse("22.11.0.1"), None);
        assert_eq!(Version::parse("v22.11.0-rc.1"), None);
        assert_eq!(v.to_string(), "v22.11.0");
    }

    #[test]
    fn orders_numerically() {
        assert!(Version::parse("v9.11.2").unwrap() < Version::parse("v10.0.0").unwrap());
    }
}
