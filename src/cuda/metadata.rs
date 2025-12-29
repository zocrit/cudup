use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CudaReleaseMetadata {
    pub release_date: String,
    pub release_label: String,
    pub release_product: String,
    #[serde(flatten)]
    pub packages: HashMap<String, PackageInfo>,
}

/// Information about a specific CUDA/cuDNN package
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageInfo {
    pub name: String,
    pub license: String,
    pub license_path: String,
    pub version: String,
    /// Optional CUDA variant list (e.g., for cuda_compat with different CUDA versions)
    #[serde(default)]
    pub cuda_variant: Option<Vec<String>>,
    #[serde(flatten)]
    pub platforms: HashMap<String, PlatformInfo>,
}

/// Platform-specific download information
/// Can be either simple (direct download) or complex (with variants like cuda13.1)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum PlatformInfo {
    /// Simple platform with direct download info
    Simple(DownloadInfo),
    /// Complex platform with variant-specific downloads
    Variants(HashMap<String, DownloadInfo>),
}

/// Download information for a specific package variant
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DownloadInfo {
    pub relative_path: String,
    pub sha256: String,
    pub md5: String,
    pub size: String,
}

impl CudaReleaseMetadata {
    /// Get a specific package by name
    pub fn get_package(&self, name: &str) -> Option<&PackageInfo> {
        self.packages.get(name)
    }

    /// Get all available package names
    pub fn package_names(&self) -> Vec<&str> {
        self.packages.keys().map(|s| s.as_str()).collect()
    }
}

impl PackageInfo {
    /// Get platform info by platform name (e.g., "linux-x86_64", "windows-x86_64")
    pub fn get_platform(&self, platform: &str) -> Option<&PlatformInfo> {
        self.platforms.get(platform)
    }

    /// Get all available platforms for this package
    pub fn available_platforms(&self) -> Vec<&str> {
        self.platforms
            .keys()
            .filter(|k| !k.starts_with("cuda_variant"))
            .map(|s| s.as_str())
            .collect()
    }
}

impl PlatformInfo {
    /// Get download info from simple platform
    pub fn as_simple(&self) -> Option<&DownloadInfo> {
        match self {
            PlatformInfo::Simple(info) => Some(info),
            _ => None,
        }
    }

    /// Get download info for a specific variant
    pub fn get_variant(&self, variant: &str) -> Option<&DownloadInfo> {
        match self {
            PlatformInfo::Variants(variants) => variants.get(variant),
            _ => None,
        }
    }

    /// Get all available variants
    pub fn variants(&self) -> Vec<&str> {
        match self {
            PlatformInfo::Variants(variants) => variants.keys().map(|s| s.as_str()).collect(),
            _ => vec![],
        }
    }
}
