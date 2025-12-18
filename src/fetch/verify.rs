use anyhow::{Result, bail};
use sha2::{Digest, Sha256};
use std::path::Path;
use tokio::fs;
use tokio::io::AsyncReadExt;

pub async fn verify_checksum(path: &Path, expected_sha256: &str) -> Result<()> {
    let expected = expected_sha256.trim().to_lowercase();

    let mut file = fs::File::open(path).await?;

    let mut hasher = Sha256::new();
    let mut buffer = vec![0u8; 64 * 1024];

    loop {
        let bytes_read = file.read(&mut buffer).await?;
        if bytes_read == 0 {
            break;
        }
        hasher.update(&buffer[..bytes_read]);
    }

    let actual = format!("{:x}", hasher.finalize());

    if actual != expected {
        bail!(
            "Checksum mismatch for {}: expected {}, got {}",
            path.display(),
            expected,
            actual
        );
    }

    Ok(())
}
