use std::fmt;
use std::str::FromStr;

use anyhow::{Result, bail};

/// A validated CUDA version string (e.g., "12.4.1")
///
/// This type ensures version strings follow the expected format
/// of `major.minor.patch` where each component is a valid number.
/// Components are parsed at construction time for O(1) access.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CudaVersion {
    raw: String,
    major: u32,
    minor: u32,
    patch: u32,
}

#[allow(dead_code)]
impl CudaVersion {
    /// Creates a new CudaVersion after validating and parsing the format
    pub fn new(version: impl Into<String>) -> Result<Self> {
        let raw = version.into();
        let (major, minor, patch) = Self::parse(&raw)?;
        Ok(Self {
            raw,
            major,
            minor,
            patch,
        })
    }

    /// Parses and validates a version string, returning the components
    fn parse(version: &str) -> Result<(u32, u32, u32)> {
        let mut parts = version.split('.');

        let parse_component = |name: &str, part: Option<&str>| -> Result<u32> {
            let part = part.ok_or_else(|| {
                anyhow::anyhow!(
                    "Invalid CUDA version '{}': expected format 'major.minor.patch' (e.g., '12.4.1')",
                    version
                )
            })?;
            part.parse::<u32>().map_err(|_| {
                anyhow::anyhow!(
                    "Invalid CUDA version '{}': {} component '{}' is not a valid number",
                    version,
                    name,
                    part
                )
            })
        };

        let major = parse_component("major", parts.next())?;
        let minor = parse_component("minor", parts.next())?;
        let patch = parse_component("patch", parts.next())?;

        if parts.next().is_some() {
            bail!(
                "Invalid CUDA version '{}': expected format 'major.minor.patch' (e.g., '12.4.1')",
                version
            );
        }

        Ok((major, minor, patch))
    }

    /// Returns the major version number (e.g., 12 for "12.4.1")
    ///
    #[must_use]
    pub fn major(&self) -> u32 {
        self.major
    }

    /// Returns the minor version number (e.g., 4 for "12.4.1")
    #[must_use]
    pub fn minor(&self) -> u32 {
        self.minor
    }

    /// Returns the patch version number (e.g., 1 for "12.4.1")
    #[must_use]
    pub fn patch(&self) -> u32 {
        self.patch
    }

    /// Returns the version as a string slice
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.raw
    }

    /// Consumes the CudaVersion and returns the inner String
    #[must_use]
    pub fn into_inner(self) -> String {
        self.raw
    }
}

impl fmt::Display for CudaVersion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.raw)
    }
}

impl FromStr for CudaVersion {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self> {
        Self::new(s)
    }
}

impl AsRef<str> for CudaVersion {
    fn as_ref(&self) -> &str {
        &self.raw
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_versions() {
        assert!(CudaVersion::new("12.4.1").is_ok());
        assert!(CudaVersion::new("11.8.0").is_ok());
        assert!(CudaVersion::new("10.0.0").is_ok());
        assert!(CudaVersion::new("12.0.0").is_ok());
    }

    #[test]
    fn test_invalid_versions() {
        assert!(CudaVersion::new("12").is_err());
        assert!(CudaVersion::new("12.4").is_err());
        assert!(CudaVersion::new("12.4.1.0").is_err());
        assert!(CudaVersion::new("").is_err());
        assert!(CudaVersion::new("12.x.1").is_err());
        assert!(CudaVersion::new("abc").is_err());
        assert!(CudaVersion::new("12.4.x").is_err());
        assert!(CudaVersion::new("-1.0.0").is_err());
    }

    #[test]
    fn test_version_components() {
        let v = CudaVersion::new("12.4.1").unwrap();
        assert_eq!(v.major(), 12);
        assert_eq!(v.minor(), 4);
        assert_eq!(v.patch(), 1);

        let v = CudaVersion::new("11.8.0").unwrap();
        assert_eq!(v.major(), 11);
        assert_eq!(v.minor(), 8);
        assert_eq!(v.patch(), 0);
    }

    #[test]
    fn test_display() {
        let v = CudaVersion::new("12.4.1").unwrap();
        assert_eq!(format!("{}", v), "12.4.1");
    }

    #[test]
    fn test_from_str() {
        let v: CudaVersion = "12.4.1".parse().unwrap();
        assert_eq!(v.as_str(), "12.4.1");
    }

    #[test]
    fn test_as_ref() {
        let v = CudaVersion::new("12.4.1").unwrap();
        let s: &str = v.as_ref();
        assert_eq!(s, "12.4.1");
    }

    #[test]
    fn test_into_inner() {
        let v = CudaVersion::new("12.4.1").unwrap();
        let s = v.into_inner();
        assert_eq!(s, "12.4.1");
    }

    #[test]
    fn test_error_messages() {
        let err = CudaVersion::new("12").unwrap_err();
        assert!(err.to_string().contains("major.minor.patch"));

        let err = CudaVersion::new("12.x.1").unwrap_err();
        assert!(err.to_string().contains("minor"));
    }
}
