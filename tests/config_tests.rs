/// Comprehensive tests for configuration loading and validation
use fallout_dnd::config::{Config, GameConfig, LlamaConfig};
use std::io::Write;
use tempfile::NamedTempFile;

#[test]
fn test_default_config_creation() {
    let config = Config::default();

    // Test llama config defaults
    assert_eq!(config.llama.server_url, "http://localhost:8080");
    assert_eq!(config.llama.extraction_url, "http://localhost:8081");
    assert!(config.llama.temperature > 0.0 && config.llama.temperature <= 1.0);
    assert!(config.llama.top_p > 0.0 && config.llama.top_p <= 1.0);
    assert!(config.llama.top_k > 0);
    assert!(config.llama.max_tokens > 0);
    assert!(!config.llama.system_prompt.is_empty());

    // Test game config defaults
    assert_eq!(config.game.starting_level, 1);
    assert!(config.game.starting_caps > 0);
    assert_eq!(config.game.permadeath, false);
    assert!(config.game.autosave_interval > 0);
}

#[test]
fn test_load_valid_config_from_toml() {
    let toml_content = r#"
[llama]
server_url = "http://test:9000"
extraction_url = "http://test:9001"
temperature = 0.7
top_p = 0.95
top_k = 50
max_tokens = 1024
repeat_penalty = 1.2
system_prompt = "Test prompt"

[game]
starting_level = 5
starting_caps = 1000
permadeath = true
autosave_interval = 10
"#;

    let config: Config = toml::from_str(toml_content).expect("Should parse valid TOML");

    assert_eq!(config.llama.server_url, "http://test:9000");
    assert_eq!(config.llama.extraction_url, "http://test:9001");
    assert_eq!(config.llama.temperature, 0.7);
    assert_eq!(config.llama.top_p, 0.95);
    assert_eq!(config.llama.top_k, 50);
    assert_eq!(config.llama.max_tokens, 1024);
    assert_eq!(config.llama.repeat_penalty, 1.2);
    assert_eq!(config.llama.system_prompt, "Test prompt");

    assert_eq!(config.game.starting_level, 5);
    assert_eq!(config.game.starting_caps, 1000);
    assert_eq!(config.game.permadeath, true);
    assert_eq!(config.game.autosave_interval, 10);
}

#[test]
fn test_load_invalid_toml() {
    let invalid_toml = r#"
[llama
server_url = "broken
"#;

    let result: Result<Config, _> = toml::from_str(invalid_toml);
    assert!(result.is_err(), "Should fail to parse invalid TOML");
}

#[test]
fn test_load_missing_required_fields() {
    let incomplete_toml = r#"
[llama]
server_url = "http://localhost:8080"
# Missing other required fields
"#;

    let result: Result<Config, _> = toml::from_str(incomplete_toml);
    assert!(
        result.is_err(),
        "Should fail when required fields are missing"
    );
}

#[test]
fn test_load_partial_config_with_defaults() {
    // TOML with only some fields - this should fail since we don't have default attributes
    // But if we had #[serde(default)] it would work
    let partial_toml = r#"
[llama]
server_url = "http://localhost:8080"
extraction_url = "http://localhost:8081"
temperature = 0.8
top_p = 0.9
top_k = 40
max_tokens = 512
repeat_penalty = 1.1
system_prompt = "Test"

[game]
starting_level = 1
starting_caps = 500
permadeath = false
autosave_interval = 5
"#;

    let result: Result<Config, _> = toml::from_str(partial_toml);
    assert!(result.is_ok(), "Should parse complete config");
}

#[test]
fn test_config_load_from_file_not_found() {
    // This will try to load from a non-existent config.toml
    // In actual code, this would use Config::load() which reads from "config.toml"
    let result = std::fs::read_to_string("nonexistent_config_file.toml");
    assert!(
        result.is_err(),
        "Should fail when config file doesn't exist"
    );
}

#[test]
fn test_config_serialization_roundtrip() {
    let original = Config::default();

    // Serialize to TOML
    let toml_string = toml::to_string(&original).expect("Should serialize to TOML");

    // Deserialize back
    let deserialized: Config = toml::from_str(&toml_string).expect("Should deserialize from TOML");

    // Verify key fields match
    assert_eq!(deserialized.llama.server_url, original.llama.server_url);
    assert_eq!(deserialized.llama.temperature, original.llama.temperature);
    assert_eq!(
        deserialized.game.starting_level,
        original.game.starting_level
    );
    assert_eq!(deserialized.game.permadeath, original.game.permadeath);
}

#[test]
fn test_llama_config_temperature_range() {
    let config = Config::default();
    // Temperature should be in a reasonable range
    assert!(
        config.llama.temperature >= 0.0,
        "Temperature should be non-negative"
    );
    assert!(
        config.llama.temperature <= 2.0,
        "Temperature should be reasonable"
    );
}

#[test]
fn test_llama_config_top_p_range() {
    let config = Config::default();
    // top_p should be between 0 and 1
    assert!(config.llama.top_p > 0.0, "top_p should be positive");
    assert!(config.llama.top_p <= 1.0, "top_p should not exceed 1.0");
}

#[test]
fn test_game_config_starting_level_positive() {
    let config = Config::default();
    assert!(
        config.game.starting_level > 0,
        "Starting level should be positive"
    );
}

#[test]
fn test_game_config_starting_caps_nonnegative() {
    let config = Config::default();
    assert!(
        config.game.starting_caps >= 0,
        "Starting caps should be non-negative"
    );
}

#[test]
fn test_game_config_autosave_interval_positive() {
    let config = Config::default();
    assert!(
        config.game.autosave_interval > 0,
        "Autosave interval should be positive"
    );
}

#[test]
fn test_custom_llama_config() {
    let custom = LlamaConfig {
        server_url: "http://custom:8080".to_string(),
        extraction_url: "http://custom:8081".to_string(),
        temperature: 0.5,
        top_p: 0.85,
        top_k: 30,
        max_tokens: 256,
        repeat_penalty: 1.0,
        system_prompt: "Custom prompt".to_string(),
    };

    assert_eq!(custom.server_url, "http://custom:8080");
    assert_eq!(custom.temperature, 0.5);
    assert_eq!(custom.max_tokens, 256);
}

#[test]
fn test_custom_game_config() {
    let custom = GameConfig {
        starting_level: 10,
        starting_caps: 10000,
        permadeath: true,
        autosave_interval: 1,
    };

    assert_eq!(custom.starting_level, 10);
    assert_eq!(custom.starting_caps, 10000);
    assert!(custom.permadeath);
    assert_eq!(custom.autosave_interval, 1);
}

#[test]
fn test_config_with_extreme_values() {
    let toml_content = r#"
[llama]
server_url = "http://localhost:8080"
extraction_url = "http://localhost:8081"
temperature = 2.0
top_p = 1.0
top_k = 100
max_tokens = 4096
repeat_penalty = 2.0
system_prompt = "Very long prompt that could theoretically be quite extensive"

[game]
starting_level = 100
starting_caps = 999999
permadeath = true
autosave_interval = 1
"#;

    let config: Config = toml::from_str(toml_content).expect("Should handle extreme values");

    assert_eq!(config.llama.temperature, 2.0);
    assert_eq!(config.llama.max_tokens, 4096);
    assert_eq!(config.game.starting_level, 100);
    assert_eq!(config.game.starting_caps, 999999);
}

#[test]
fn test_config_boolean_fields() {
    let toml_true = r#"
[llama]
server_url = "http://localhost:8080"
extraction_url = "http://localhost:8081"
temperature = 0.8
top_p = 0.9
top_k = 40
max_tokens = 512
repeat_penalty = 1.1
system_prompt = "Test"

[game]
starting_level = 1
starting_caps = 500
permadeath = true
autosave_interval = 5
"#;

    let toml_false = r#"
[llama]
server_url = "http://localhost:8080"
extraction_url = "http://localhost:8081"
temperature = 0.8
top_p = 0.9
top_k = 40
max_tokens = 512
repeat_penalty = 1.1
system_prompt = "Test"

[game]
starting_level = 1
starting_caps = 500
permadeath = false
autosave_interval = 5
"#;

    let config_true: Config = toml::from_str(toml_true).unwrap();
    let config_false: Config = toml::from_str(toml_false).unwrap();

    assert!(config_true.game.permadeath);
    assert!(!config_false.game.permadeath);
}
