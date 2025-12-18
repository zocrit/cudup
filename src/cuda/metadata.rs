use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CudaReleaseMetadata {
    pub release_date: Option<String>,
    #[serde(default)]
    pub release_label: Option<String>,
    #[serde(default)]
    pub release_product: Option<String>,
    #[serde(flatten)]
    pub packages: HashMap<String, PackageInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageInfo {
    pub name: String,
    pub license: String,
    #[serde(default)]
    pub license_path: Option<String>,
    pub version: String,
    #[serde(default)]
    pub cuda_variant: Option<Vec<String>>,
    #[serde(flatten)]
    pub platforms: HashMap<String, PlatformInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum PlatformInfo {
    Simple(DownloadInfo),
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
}

impl PackageInfo {
    pub fn get_platform(&self, platform: &str) -> Option<&PlatformInfo> {
        self.platforms.get(platform)
    }
}
