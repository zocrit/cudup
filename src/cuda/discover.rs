use crate::cuda::metadata;
use reqwest::blocking::Client;
use serde::Deserialize;
use serde_json::Value;

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

pub fn fetch_available_cuda_versions() {
    let client = Client::new();
    let response = client
        .get("https://developer.download.nvidia.com/compute/cuda/redist/redistrib_13.1.0.json")
        .send();

    match response {
        Ok(resp) => {
            let body = resp.text();
            println!("body = {body:?}");
        }
        Err(e) => {
            println!("Error fetching CUDA versions: {e}");
        }
    }
}
