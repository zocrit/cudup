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

    pub fn package_names(&self) -> impl Iterator<Item = &str> {
        self.packages.keys().map(|s| s.as_str())
    }
}

impl PackageInfo {
    pub fn get_platform(&self, platform: &str) -> Option<&PlatformInfo> {
        self.platforms.get(platform)
    }

    pub fn available_platforms(&self) -> impl Iterator<Item = &str> {
        self.platforms
            .keys()
            .filter(|k| !k.starts_with("cuda_variant"))
            .map(|s| s.as_str())
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

    pub fn variants(&self) -> impl Iterator<Item = &str> {
        let variants = match self {
            PlatformInfo::Variants(v) => Some(v),
            _ => None,
        };
        variants.into_iter().flatten().map(|(k, _)| k.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_cuda_metadata_json() -> &'static str {
        r#"{
            "release_date": "2024-06-01",
            "cuda_cccl": {
                "name": "CUDA C++ Core Libraries",
                "license": "NVIDIA Software License",
                "version": "12.4.127",
                "linux-x86_64": {
                    "relative_path": "cuda_cccl/linux-x86_64/cuda_cccl-linux-x86_64-12.4.127-archive.tar.xz",
                    "sha256": "abc123",
                    "md5": "def456",
                    "size": "1234567"
                }
            },
            "cuda_cudart": {
                "name": "CUDA Runtime",
                "license": "NVIDIA Software License",
                "version": "12.4.127",
                "linux-x86_64": {
                    "relative_path": "cuda_cudart/linux-x86_64/cuda_cudart-linux-x86_64-12.4.127-archive.tar.xz",
                    "sha256": "789abc",
                    "md5": "012def",
                    "size": "3456789"
                },
                "windows-x86_64": {
                    "relative_path": "cuda_cudart/windows-x86_64/cuda_cudart-windows-x86_64-12.4.127-archive.zip",
                    "sha256": "456xyz",
                    "md5": "789uvw",
                    "size": "4567890"
                }
            }
        }"#
    }

    fn sample_cudnn_metadata_json() -> &'static str {
        r#"{
            "release_date": "2024-05-15",
            "release_label": "9.1.0",
            "release_product": "cudnn",
            "cudnn": {
                "name": "cuDNN",
                "license": "NVIDIA cuDNN Software License",
                "license_path": "cudnn/LICENSE.txt",
                "version": "9.1.0.70",
                "cuda_variant": ["11", "12"],
                "linux-x86_64": {
                    "cuda11": {
                        "relative_path": "cudnn/linux-x86_64/cudnn-linux-x86_64-9.1.0.70_cuda11-archive.tar.xz",
                        "sha256": "cudnn11hash",
                        "md5": "cudnn11md5",
                        "size": "987654321"
                    },
                    "cuda12": {
                        "relative_path": "cudnn/linux-x86_64/cudnn-linux-x86_64-9.1.0.70_cuda12-archive.tar.xz",
                        "sha256": "cudnn12hash",
                        "md5": "cudnn12md5",
                        "size": "987654322"
                    }
                }
            }
        }"#
    }

    #[test]
    fn test_parse_cuda_metadata() {
        let metadata: CudaReleaseMetadata =
            serde_json::from_str(sample_cuda_metadata_json()).unwrap();

        assert_eq!(metadata.release_date, Some("2024-06-01".to_string()));
        assert!(metadata.release_label.is_none());
        assert!(metadata.release_product.is_none());
        assert_eq!(metadata.packages.len(), 2);
    }

    #[test]
    fn test_get_package() {
        let metadata: CudaReleaseMetadata =
            serde_json::from_str(sample_cuda_metadata_json()).unwrap();

        let cccl = metadata.get_package("cuda_cccl");
        assert!(cccl.is_some());
        assert_eq!(cccl.unwrap().name, "CUDA C++ Core Libraries");
        assert_eq!(cccl.unwrap().version, "12.4.127");

        let nonexistent = metadata.get_package("nonexistent");
        assert!(nonexistent.is_none());
    }

    #[test]
    fn test_package_names() {
        let metadata: CudaReleaseMetadata =
            serde_json::from_str(sample_cuda_metadata_json()).unwrap();

        let names: Vec<&str> = metadata.package_names().collect();
        assert_eq!(names.len(), 2);
        assert!(names.contains(&"cuda_cccl"));
        assert!(names.contains(&"cuda_cudart"));
    }

    #[test]
    fn test_get_platform_simple() {
        let metadata: CudaReleaseMetadata =
            serde_json::from_str(sample_cuda_metadata_json()).unwrap();

        let cccl = metadata.get_package("cuda_cccl").unwrap();
        let linux = cccl.get_platform("linux-x86_64");
        assert!(linux.is_some());

        let download_info = linux.unwrap().as_simple();
        assert!(download_info.is_some());
        assert!(
            download_info
                .unwrap()
                .relative_path
                .contains("cuda_cccl-linux-x86_64")
        );
    }

    #[test]
    fn test_available_platforms() {
        let metadata: CudaReleaseMetadata =
            serde_json::from_str(sample_cuda_metadata_json()).unwrap();

        let cudart = metadata.get_package("cuda_cudart").unwrap();
        let platforms: Vec<&str> = cudart.available_platforms().collect();
        assert_eq!(platforms.len(), 2);
        assert!(platforms.contains(&"linux-x86_64"));
        assert!(platforms.contains(&"windows-x86_64"));
    }

    #[test]
    fn test_parse_cudnn_metadata() {
        let metadata: CudaReleaseMetadata =
            serde_json::from_str(sample_cudnn_metadata_json()).unwrap();

        assert_eq!(metadata.release_date, Some("2024-05-15".to_string()));
        assert_eq!(metadata.release_label, Some("9.1.0".to_string()));
        assert_eq!(metadata.release_product, Some("cudnn".to_string()));
    }

    #[test]
    fn test_cudnn_cuda_variant() {
        let metadata: CudaReleaseMetadata =
            serde_json::from_str(sample_cudnn_metadata_json()).unwrap();

        let cudnn = metadata.get_package("cudnn").unwrap();
        let variants = cudnn.cuda_variant.as_ref().unwrap();
        assert_eq!(variants.len(), 2);
        assert!(variants.contains(&"11".to_string()));
        assert!(variants.contains(&"12".to_string()));
    }

    #[test]
    fn test_platform_variants() {
        let metadata: CudaReleaseMetadata =
            serde_json::from_str(sample_cudnn_metadata_json()).unwrap();

        let cudnn = metadata.get_package("cudnn").unwrap();
        let linux = cudnn.get_platform("linux-x86_64").unwrap();

        let variants: Vec<&str> = linux.variants().collect();
        assert_eq!(variants.len(), 2);
        assert!(variants.contains(&"cuda11"));
        assert!(variants.contains(&"cuda12"));

        let cuda12_info = linux.get_variant("cuda12");
        assert!(cuda12_info.is_some());
        assert!(
            cuda12_info
                .unwrap()
                .relative_path
                .contains("cuda12-archive")
        );
    }

    #[test]
    fn test_download_info_fields() {
        let metadata: CudaReleaseMetadata =
            serde_json::from_str(sample_cuda_metadata_json()).unwrap();

        let cccl = metadata.get_package("cuda_cccl").unwrap();
        let linux = cccl.get_platform("linux-x86_64").unwrap();
        let info = linux.as_simple().unwrap();

        assert!(!info.relative_path.is_empty());
        assert!(!info.sha256.is_empty());
        assert!(!info.md5.is_empty());
        assert!(!info.size.is_empty());
    }
}
