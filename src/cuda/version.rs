use std::fmt;
use std::str::FromStr;

use anyhow::{bail, Result};

/// A validated CUDA version string (e.g., "12.4.1")
///
/// This newtype ensures version strings follow the expected format
/// of `major.minor.patch` where each component is a valid number.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CudaVersion(String);

impl CudaVersion {
    /// Creates a new CudaVersion after validating the format
    pub fn new(version: impl Into<String>) -> Result<Self> {
        let version = version.into();
        Self::validate(&version)?;
        Ok(Self(version))
    }

    /// Validates that a version string matches the expected format
    fn validate(version: &str) -> Result<()> {
        let parts: Vec<&str> = version.split('.').collect();

        if parts.len() != 3 {
            bail!(
                "Invalid CUDA version '{}': expected format 'major.minor.patch' (e.g., '12.4.1')",
                version
            );
        }

        for (i, part) in parts.iter().enumerate() {
            if part.parse::<u32>().is_err() {
                let component = match i {
                    0 => "major",
                    1 => "minor",
                    _ => "patch",
                };
                bail!(
                    "Invalid CUDA version '{}': {} component '{}' is not a valid number",
                    version,
                    component,
                    part
                );
            }
        }

        Ok(())
    }

    /// Returns the major version number (e.g., 12 for "12.4.1")
    #[must_use]
    pub fn major(&self) -> u32 {
        self.0
            .split('.')
            .next()
            .and_then(|s| s.parse().ok())
            .unwrap_or(0)
    }

    /// Returns the version as a string slice
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Consumes the CudaVersion and returns the inner String
    #[must_use]
    pub fn into_inner(self) -> String {
        self.0
    }
}

impl fmt::Display for CudaVersion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
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
        &self.0
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
    fn test_major_version() {
        let v = CudaVersion::new("12.4.1").unwrap();
        assert_eq!(v.major(), 12);

        let v = CudaVersion::new("11.8.0").unwrap();
        assert_eq!(v.major(), 11);
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
