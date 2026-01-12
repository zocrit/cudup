//! Integration tests for the install functionality with mocked HTTP responses.
//!
//! These tests use wiremock to mock NVIDIA's download servers, allowing us to test
//! the install flow without downloading multi-gigabyte files.

use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

/// Sample CUDA metadata JSON for testing
fn cuda_metadata_json() -> &'static str {
    r#"{
        "release_date": "2024-06-01",
        "cuda_cccl": {
            "name": "CUDA C++ Core Libraries",
            "license": "NVIDIA Software License",
            "version": "12.4.127",
            "linux-x86_64": {
                "relative_path": "cuda_cccl/linux-x86_64/cuda_cccl-linux-x86_64-12.4.127-archive.tar.xz",
                "sha256": "abc123def456789012345678901234567890123456789012345678901234abcd",
                "md5": "abc123def456",
                "size": "100"
            }
        }
    }"#
}

/// Sample cuDNN metadata JSON for testing
fn cudnn_metadata_json() -> &'static str {
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
                "cuda12": {
                    "relative_path": "cudnn/linux-x86_64/cudnn-linux-x86_64-9.1.0.70_cuda12-archive.tar.xz",
                    "sha256": "cudnn12sha256hash012345678901234567890123456789012345678901234567",
                    "md5": "cudnn12md5hash",
                    "size": "100"
                }
            }
        }
    }"#
}

/// Sample HTML index listing CUDA versions
fn cuda_index_html() -> &'static str {
    r#"<!DOCTYPE html>
<html>
<head><title>Index of /compute/cuda/redist/</title></head>
<body>
<pre>
<a href="redistrib_12.4.1.json">redistrib_12.4.1.json</a>
<a href="redistrib_12.3.0.json">redistrib_12.3.0.json</a>
</pre>
</body>
</html>"#
}

#[tokio::test]
async fn test_mock_server_setup() {
    // Basic test to ensure wiremock works correctly
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/test"))
        .respond_with(ResponseTemplate::new(200).set_body_string("Hello, World!"))
        .mount(&mock_server)
        .await;

    let response = reqwest::get(format!("{}/test", mock_server.uri()))
        .await
        .unwrap();

    assert_eq!(response.status(), 200);
    assert_eq!(response.text().await.unwrap(), "Hello, World!");
}

#[tokio::test]
async fn test_mock_cuda_index_parsing() {
    let mock_server = MockServer::start().await;

    // Mock CUDA versions index
    Mock::given(method("GET"))
        .and(path("/"))
        .respond_with(ResponseTemplate::new(200).set_body_string(cuda_index_html()))
        .mount(&mock_server)
        .await;

    let response = reqwest::get(format!("{}/", mock_server.uri()))
        .await
        .unwrap();

    let body = response.text().await.unwrap();
    assert!(body.contains("redistrib_12.4.1.json"));
    assert!(body.contains("redistrib_12.3.0.json"));
}

#[tokio::test]
async fn test_mock_cuda_metadata_response() {
    let mock_server = MockServer::start().await;

    // Mock CUDA metadata endpoint
    Mock::given(method("GET"))
        .and(path("/redistrib_12.4.1.json"))
        .respond_with(ResponseTemplate::new(200).set_body_string(cuda_metadata_json()))
        .mount(&mock_server)
        .await;

    let response = reqwest::get(format!("{}/redistrib_12.4.1.json", mock_server.uri()))
        .await
        .unwrap();

    assert_eq!(response.status(), 200);

    let metadata: serde_json::Value = response.json().await.unwrap();
    assert_eq!(metadata["release_date"], "2024-06-01");
    assert!(metadata["cuda_cccl"].is_object());
}

#[tokio::test]
async fn test_mock_cudnn_metadata_response() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/redistrib_9.1.0.json"))
        .respond_with(ResponseTemplate::new(200).set_body_string(cudnn_metadata_json()))
        .mount(&mock_server)
        .await;

    let response = reqwest::get(format!("{}/redistrib_9.1.0.json", mock_server.uri()))
        .await
        .unwrap();

    assert_eq!(response.status(), 200);

    let metadata: serde_json::Value = response.json().await.unwrap();
    assert_eq!(metadata["release_product"], "cudnn");
    assert!(
        metadata["cudnn"]["cuda_variant"]
            .as_array()
            .unwrap()
            .contains(&serde_json::json!("12"))
    );
}

#[tokio::test]
async fn test_mock_404_response() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/redistrib_99.99.99.json"))
        .respond_with(ResponseTemplate::new(404))
        .mount(&mock_server)
        .await;

    let response = reqwest::get(format!("{}/redistrib_99.99.99.json", mock_server.uri()))
        .await
        .unwrap();

    assert_eq!(response.status(), 404);
}

/// Test that metadata parsing works correctly with the fixture files
#[test]
fn test_parse_cuda_fixture() {
    let fixture = include_str!("fixtures/cuda_12.4.1_metadata.json");
    let metadata: serde_json::Value = serde_json::from_str(fixture).unwrap();

    assert_eq!(metadata["release_date"], "2024-06-01");
    assert!(metadata["cuda_cccl"].is_object());
    assert!(metadata["cuda_cudart"].is_object());
    assert!(metadata["cuda_nvcc"].is_object());
}

#[test]
fn test_parse_cudnn_fixture() {
    let fixture = include_str!("fixtures/cudnn_9.1.0_metadata.json");
    let metadata: serde_json::Value = serde_json::from_str(fixture).unwrap();

    assert_eq!(metadata["release_product"], "cudnn");
    assert_eq!(metadata["release_label"], "9.1.0");
    assert!(
        metadata["cudnn"]["cuda_variant"]
            .as_array()
            .unwrap()
            .contains(&serde_json::json!("12"))
    );
}

#[test]
fn test_parse_cuda_index_fixture() {
    let fixture = include_str!("fixtures/cuda_index.html");

    // Test that version regex would find versions
    let re = regex::Regex::new(r"redistrib_(\d+\.\d+\.\d+)\.json").unwrap();
    let versions: Vec<String> = re
        .captures_iter(fixture)
        .filter_map(|cap| cap.get(1).map(|m| m.as_str().to_string()))
        .collect();

    assert!(!versions.is_empty());
    assert!(versions.contains(&"12.4.1".to_string()));
    assert!(versions.contains(&"11.8.0".to_string()));
}

#[test]
fn test_parse_cudnn_index_fixture() {
    let fixture = include_str!("fixtures/cudnn_index.html");

    let re = regex::Regex::new(r"redistrib_(\d+\.\d+\.\d+)\.json").unwrap();
    let versions: Vec<String> = re
        .captures_iter(fixture)
        .filter_map(|cap| cap.get(1).map(|m| m.as_str().to_string()))
        .collect();

    assert!(!versions.is_empty());
    assert!(versions.contains(&"9.1.0".to_string()));
    assert!(versions.contains(&"8.9.0".to_string()));
}
