use anyhow::Result;
use std::path::Path;
use tokio::fs;

pub async fn verify_checksum(path: &Path, expected_sha256: &str) -> Result<bool> {
    use sha2::{Digest, Sha256};
    use tokio::io::AsyncReadExt;

    // Normalize expected hash: trim whitespace and convert to lowercase
    let expected = expected_sha256.trim().to_lowercase();

    // Stream the file to avoid loading it entirely into memory
    let mut file = fs::File::open(path).await?;
    let mut hasher = Sha256::new();
    let mut buffer = vec![0u8; 8192]; // 8KB buffer

    loop {
        let bytes_read = file.read(&mut buffer).await?;
        if bytes_read == 0 {
            break;
        }
        hasher.update(&buffer[..bytes_read]);
    }

    let result = hasher.finalize();
    let actual = format!("{:x}", result);

    Ok(actual == expected)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_verify_checksum_correct() {
        use tokio::io::AsyncWriteExt;

        let temp_dir = tempfile::tempdir().unwrap();
        let file_path = temp_dir.path().join("test_file.txt");

        // Create a test file with known content
        let content = b"Hello, World!";
        let mut file = tokio::fs::File::create(&file_path).await.unwrap();
        file.write_all(content).await.unwrap();
        file.flush().await.unwrap();

        // SHA256 of "Hello, World!" is known
        let expected_sha256 = "dffd6021bb2bd5b0af676290809ec3a53191dd81c7f70a4b28688a362182986f";

        let result = verify_checksum(&file_path, expected_sha256).await.unwrap();
        assert!(result);
    }

    #[tokio::test]
    async fn test_verify_checksum_incorrect() {
        use tokio::io::AsyncWriteExt;

        let temp_dir = tempfile::tempdir().unwrap();
        let file_path = temp_dir.path().join("test_file.txt");

        let content = b"Hello, World!";
        let mut file = tokio::fs::File::create(&file_path).await.unwrap();
        file.write_all(content).await.unwrap();
        file.flush().await.unwrap();

        let wrong_sha256 = "0000000000000000000000000000000000000000000000000000000000000000";

        let result = verify_checksum(&file_path, wrong_sha256).await.unwrap();
        assert!(!result);
    }
}
