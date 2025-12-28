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
