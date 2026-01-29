use anyhow::{Context, Result, bail};
use sha2::{Digest, Sha256};
use std::path::Path;
use tokio::fs;
use tokio::io::AsyncReadExt;

pub async fn verify_checksum(path: &Path, expected_sha256: &str) -> Result<()> {
    let expected = expected_sha256.trim().to_lowercase();

    let mut file = fs::File::open(path)
        .await
        .with_context(|| format!("Failed to open {} for verification", path.display()))?;

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

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::io::AsyncWriteExt;

    #[tokio::test]
    async fn test_verify_checksum_correct() {
        let temp_dir = tempfile::tempdir().unwrap();
        let file_path = temp_dir.path().join("test_file.txt");

        // Create a test file with known content
        let content = b"Hello, World!";
        let mut file = tokio::fs::File::create(&file_path).await.unwrap();
        file.write_all(content).await.unwrap();
        file.flush().await.unwrap();

        // SHA256 of "Hello, World!" is known
        let expected_sha256 = "dffd6021bb2bd5b0af676290809ec3a53191dd81c7f70a4b28688a362182986f";

        verify_checksum(&file_path, expected_sha256).await.unwrap();
    }

    #[tokio::test]
    async fn test_verify_checksum_incorrect() {
        let temp_dir = tempfile::tempdir().unwrap();
        let file_path = temp_dir.path().join("test_file.txt");

        let content = b"Hello, World!";
        let mut file = tokio::fs::File::create(&file_path).await.unwrap();
        file.write_all(content).await.unwrap();
        file.flush().await.unwrap();

        let wrong_sha256 = "0000000000000000000000000000000000000000000000000000000000000000";

        let result = verify_checksum(&file_path, wrong_sha256).await;
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("Checksum mismatch")
        );
    }
}
