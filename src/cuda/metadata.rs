use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// The structure uses flatten to handle the dynamic package names
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CudaReleaseMetadata {
    /// Release date in YYYY-MM-DD format
    pub release_date: Option<String>,
    /// Release label (e.g., "9.0.0"); only present in cuDNN
    #[serde(default)]
    pub release_label: Option<String>,
    /// Product name (e.g., "cudnn"); only present in cuDNN
    #[serde(default)]
    pub release_product: Option<String>,
    #[serde(flatten)]
    pub packages: HashMap<String, PackageInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageInfo {
    pub name: String,
    pub license: String,
    /// License path; only present in cuDNN packages
    #[serde(default)]
    pub license_path: Option<String>,
    pub version: String,
    /// Optional CUDA variant list (e.g., for cuDNN with different CUDA versions)
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DownloadInfo {
    pub relative_path: String,
    pub sha256: String,
    pub md5: String,
    pub size: String,
}

impl CudaReleaseMetadata {
    pub fn get_package(&self, name: &str) -> Option<&PackageInfo> {
        self.packages.get(name)
    }

    pub fn package_names(&self) -> Vec<&str> {
        self.packages.keys().map(|s| s.as_str()).collect()
    }
}

impl PackageInfo {
    pub fn get_platform(&self, platform: &str) -> Option<&PlatformInfo> {
        self.platforms.get(platform)
    }

    pub fn available_platforms(&self) -> Vec<&str> {
        self.platforms
            .keys()
            .filter(|k| !k.starts_with("cuda_variant"))
            .map(|s| s.as_str())
            .collect()
    }
}

impl PlatformInfo {
    pub fn as_simple(&self) -> Option<&DownloadInfo> {
        match self {
            PlatformInfo::Simple(info) => Some(info),
            _ => None,
        }
    }

    pub fn get_variant(&self, variant: &str) -> Option<&DownloadInfo> {
        match self {
            PlatformInfo::Variants(variants) => variants.get(variant),
            _ => None,
        }
    }

    pub fn variants(&self) -> Vec<&str> {
        match self {
            PlatformInfo::Variants(variants) => variants.keys().map(|s| s.as_str()).collect(),
            _ => vec![],
        }
    }
}
