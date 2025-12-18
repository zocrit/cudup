use std::fmt;
use std::str::FromStr;

use anyhow::{Result, bail};

/// Validated CUDA version in `major.minor.patch` format.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CudaVersion {
    raw: String,
    major: u32,
    minor: u32,
    patch: u32,
}

impl CudaVersion {
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

    pub fn major(&self) -> u32 {
        self.major
    }

    pub fn as_str(&self) -> &str {
        &self.raw
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
