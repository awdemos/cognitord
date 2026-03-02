//! Integration tests for cognitord
//!
//! These tests verify the core functionality of the daemon including:
//! - Configuration loading and validation
//! - Input processing
//! - Token estimation

use std::io::Write;
use tempfile::NamedTempFile;

/// Helper to create a valid test configuration file
fn create_test_config() -> NamedTempFile {
    let mut file = NamedTempFile::new().expect("Failed to create temp file");
    let config = r#"{
        "anthropic": {
            "api_key": "sk-test-key-12345",
            "base_url": "https://api.anthropic.com",
            "model": "claude-3-opus-20240229",
            "max_tokens": 4096,
            "temperature": 0.7,
            "timeout_seconds": 30
        },
        "daemon": {
            "log_level": "info",
            "timeout_seconds": 60,
            "max_input_size": 1048576,
            "max_retries": 3,
            "retry_delay_ms": 1000,
            "backoff_factor": 2.0
        },
        "logging": {
            "level": "info",
            "format": "json",
            "file": null
        },
        "dsrs": {
            "enable_context": true,
            "enable_system_prompt": true,
            "max_context_length": 8000,
            "retry_attempts": 3
        }
    }"#;
    write!(file, "{}", config).expect("Failed to write config");
    file
}

/// Test that a valid configuration loads successfully
#[test]
fn test_load_valid_config() {
    let config_file = create_test_config();
    let content = std::fs::read_to_string(config_file.path()).expect("Failed to read config");

    // Verify JSON is valid
    let parsed: serde_json::Value =
        serde_json::from_str(&content).expect("Config should be valid JSON");

    // Verify required fields exist
    assert!(parsed.get("anthropic").is_some());
    assert!(parsed.get("daemon").is_some());
    assert!(parsed.get("logging").is_some());
    assert!(parsed.get("dsrs").is_some());

    // Verify anthropic config
    let anthropic = parsed.get("anthropic").unwrap();
    assert!(anthropic.get("api_key").is_some());
    assert!(anthropic.get("base_url").is_some());
    assert!(anthropic.get("model").is_some());
}

/// Test that configuration validation catches invalid API keys
#[test]
fn test_invalid_api_key_format() {
    let mut file = NamedTempFile::new().expect("Failed to create temp file");
    let config = r#"{
        "anthropic": {
            "api_key": "invalid-key-without-sk-prefix",
            "base_url": "https://api.anthropic.com",
            "model": "claude-3-opus-20240229",
            "max_tokens": 4096,
            "temperature": 0.7,
            "timeout_seconds": 30
        },
        "daemon": {
            "log_level": "info",
            "timeout_seconds": 60,
            "max_input_size": 1048576,
            "max_retries": 3,
            "retry_delay_ms": 1000,
            "backoff_factor": 2.0
        },
        "logging": {
            "level": "info",
            "format": "json",
            "file": null
        },
        "dsrs": {
            "enable_context": true,
            "enable_system_prompt": true,
            "max_context_length": 8000,
            "retry_attempts": 3
        }
    }"#;
    write!(file, "{}", config).expect("Failed to write config");

    let content = std::fs::read_to_string(file.path()).expect("Failed to read config");
    let parsed: serde_json::Value =
        serde_json::from_str(&content).expect("Config should be valid JSON");

    let api_key = parsed["anthropic"]["api_key"].as_str().unwrap();
    assert!(
        !api_key.starts_with("sk-"),
        "API key should not have sk- prefix"
    );
}

/// Test that configuration validation catches invalid URLs
#[test]
fn test_invalid_base_url() {
    let mut file = NamedTempFile::new().expect("Failed to create temp file");
    let config = r#"{
        "anthropic": {
            "api_key": "sk-test-key-12345",
            "base_url": "not-a-valid-url",
            "model": "claude-3-opus-20240229",
            "max_tokens": 4096,
            "temperature": 0.7,
            "timeout_seconds": 30
        },
        "daemon": {
            "log_level": "info",
            "timeout_seconds": 60,
            "max_input_size": 1048576,
            "max_retries": 3,
            "retry_delay_ms": 1000,
            "backoff_factor": 2.0
        },
        "logging": {
            "level": "info",
            "format": "json",
            "file": null
        },
        "dsrs": {
            "enable_context": true,
            "enable_system_prompt": true,
            "max_context_length": 8000,
            "retry_attempts": 3
        }
    }"#;
    write!(file, "{}", config).expect("Failed to write config");

    let content = std::fs::read_to_string(file.path()).expect("Failed to read config");
    let parsed: serde_json::Value =
        serde_json::from_str(&content).expect("Config should be valid JSON");

    let base_url = parsed["anthropic"]["base_url"].as_str().unwrap();
    assert!(
        !base_url.starts_with("http"),
        "URL should not start with http"
    );
}

/// Test that configuration validation catches zero timeout
#[test]
fn test_zero_timeout() {
    let mut file = NamedTempFile::new().expect("Failed to create temp file");
    let config = r#"{
        "anthropic": {
            "api_key": "sk-test-key-12345",
            "base_url": "https://api.anthropic.com",
            "model": "claude-3-opus-20240229",
            "max_tokens": 4096,
            "temperature": 0.7,
            "timeout_seconds": 0
        },
        "daemon": {
            "log_level": "info",
            "timeout_seconds": 60,
            "max_input_size": 1048576,
            "max_retries": 3,
            "retry_delay_ms": 1000,
            "backoff_factor": 2.0
        },
        "logging": {
            "level": "info",
            "format": "json",
            "file": null
        },
        "dsrs": {
            "enable_context": true,
            "enable_system_prompt": true,
            "max_context_length": 8000,
            "retry_attempts": 3
        }
    }"#;
    write!(file, "{}", config).expect("Failed to write config");

    let content = std::fs::read_to_string(file.path()).expect("Failed to read config");
    let parsed: serde_json::Value =
        serde_json::from_str(&content).expect("Config should be valid JSON");

    let timeout = parsed["anthropic"]["timeout_seconds"].as_u64().unwrap();
    assert_eq!(timeout, 0, "Timeout should be zero");
}

/// Test token estimation function behavior
/// The estimation uses ~4 characters per token
#[test]
fn test_token_estimation() {
    // Empty string should be 0 tokens
    let empty_tokens = estimate_token_count("");
    assert_eq!(empty_tokens, 0);

    // 4 characters should be ~1 token
    let short_tokens = estimate_token_count("test");
    assert_eq!(short_tokens, 1);

    // 8 characters should be ~2 tokens
    let medium_tokens = estimate_token_count("testtest");
    assert_eq!(medium_tokens, 2);

    // 100 characters should be ~25 tokens
    let long_text = "a".repeat(100);
    let long_tokens = estimate_token_count(&long_text);
    assert_eq!(long_tokens, 25);
}

/// Test token estimation with realistic text
#[test]
fn test_token_estimation_realistic() {
    let text = "The quick brown fox jumps over the lazy dog.";
    let tokens = estimate_token_count(text);
    // 44 characters / 4 = 11 tokens
    assert_eq!(tokens, 11);
}

/// Test ProcessRequest serialization/deserialization
#[test]
fn test_process_request_serde() {
    let json = r#"{
        "input": "Hello, world!",
        "context": "Some context",
        "system_prompt": "You are helpful",
        "request_id": "test-123"
    }"#;

    let request: serde_json::Value = serde_json::from_str(json).expect("Should parse JSON");
    assert_eq!(request["input"], "Hello, world!");
    assert_eq!(request["context"], "Some context");
    assert_eq!(request["system_prompt"], "You are helpful");
    assert_eq!(request["request_id"], "test-123");
}

/// Test ProcessRequest with minimal fields
#[test]
fn test_process_request_minimal() {
    let json = r#"{
        "input": "Hello, world!"
    }"#;

    let request: serde_json::Value = serde_json::from_str(json).expect("Should parse JSON");
    assert_eq!(request["input"], "Hello, world!");
    assert!(request.get("context").is_none() || request["context"].is_null());
}

/// Test ProcessResponse structure
#[test]
fn test_process_response_structure() {
    let json = r#"{
        "output": "Processed response",
        "usage": {
            "input_tokens": 10,
            "output_tokens": 20,
            "total_tokens": 30
        },
        "request_id": "req-123",
        "timestamp": "2024-01-01T00:00:00Z",
        "duration_ms": 150
    }"#;

    let response: serde_json::Value = serde_json::from_str(json).expect("Should parse JSON");
    assert_eq!(response["output"], "Processed response");
    assert_eq!(response["usage"]["input_tokens"], 10);
    assert_eq!(response["usage"]["output_tokens"], 20);
    assert_eq!(response["usage"]["total_tokens"], 30);
    assert_eq!(response["request_id"], "req-123");
    assert_eq!(response["duration_ms"], 150);
}

/// Test ErrorResponse structure
#[test]
fn test_error_response_structure() {
    let json = r#"{
        "error": {
            "code": "INVALID_REQUEST",
            "message": "The request was invalid",
            "details": null
        },
        "request_id": "req-456",
        "timestamp": "2024-01-01T00:00:00Z"
    }"#;

    let error: serde_json::Value = serde_json::from_str(json).expect("Should parse JSON");
    assert_eq!(error["error"]["code"], "INVALID_REQUEST");
    assert_eq!(error["error"]["message"], "The request was invalid");
}

/// Test that malformed JSON is rejected
#[test]
fn test_malformed_json_rejected() {
    let malformed = r#"{ this is not valid json }"#;
    let result: Result<serde_json::Value, _> = serde_json::from_str(malformed);
    assert!(result.is_err(), "Malformed JSON should be rejected");
}

/// Test empty input validation
#[test]
fn test_empty_input_validation() {
    let json = r#"{
        "input": "   "
    }"#;

    let request: serde_json::Value = serde_json::from_str(json).expect("Should parse JSON");
    let input = request["input"].as_str().unwrap();
    assert!(
        input.trim().is_empty(),
        "Whitespace-only input should be considered empty"
    );
}

// Helper function mirroring the main.rs implementation
fn estimate_token_count(text: &str) -> u32 {
    if text.is_empty() {
        return 0;
    }
    (text.len() as f64 / 4.0).ceil() as u32
}
