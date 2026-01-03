use crate::cuda::metadata;
use anyhow::Result;
use reqwest::blocking::Client;
use serde::Deserialize;
use serde_json::Value;
use std::collections::BTreeSet;
use std::sync::LazyLock;

static VERSION_REGEX: LazyLock<regex::Regex> =
    LazyLock::new(|| regex::Regex::new(r"redistrib_(\d+\.\d+\.\d+)\.json").unwrap());

pub struct BaseDownloadUrls {
    cuda: &'static str,
    cudnn: &'static str,
}

impl Default for BaseDownloadUrls {
    fn default() -> Self {
        Self {
            cuda: "https://developer.download.nvidia.com/compute/cuda/redist",
            cudnn: "https://developer.download.nvidia.com/compute/cudnn/redist",
        }
    }
}

impl BaseDownloadUrls {
    pub fn nvidia() -> Self {
        Self::default()
    }

    pub fn cuda() -> &'static str {
        "https://developer.download.nvidia.com/compute/cuda/redist"
    }

    pub fn cudnn() -> &'static str {
        "https://developer.download.nvidia.com/compute/cudnn/redist"
    }
}

pub fn fetch_available_cuda_versions() -> Result<BTreeSet<String>> {
    let client = Client::new();
    let response = client
        .get("https://developer.download.nvidia.com/compute/cuda/redist/")
        .send()?;

    let body = response.text()?;
    let versions = parse_available_cud_versions(&body)?;
    Ok(versions)
}

pub fn parse_available_cud_versions(html: &str) -> Result<BTreeSet<String>> {
    let versions = VERSION_REGEX
        .captures_iter(html)
        .filter_map(|cap| cap.get(1).map(|m| m.as_str().to_string()))
        .collect();

    Ok(versions)
}

pub fn fetch_available_cudnn_versions() -> Result<BTreeSet<String>> {
    let client = Client::new();
    let response = client
        .get("https://developer.download.nvidia.com/compute/cudnn/redist/")
        .send()?;

    let body = response.text()?;
    let versions = parse_available_cud_versions(&body)?;
    Ok(versions)
}
