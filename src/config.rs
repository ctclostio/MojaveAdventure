use crate::error::{ConfigError, GameError};
use garde::Validate;
use serde::{Deserialize, Serialize};
use std::fs;

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct Config {
    #[garde(dive)]
    pub llama: LlamaConfig,
    #[garde(dive)]
    pub game: GameConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct LlamaConfig {
    #[garde(skip)]
    pub server_url: String,
    #[garde(skip)]
    pub extraction_url: String,
    #[garde(range(min = 0.0, max = 2.0))]
    pub temperature: f32,
    #[garde(range(min = 0.0, max = 1.0))]
    pub top_p: f32,
    #[garde(range(min = 1))]
    pub top_k: i32,
    #[garde(range(min = 1, max = 32000))]
    pub max_tokens: i32,
    #[garde(range(min = 512, max = 131072))]
    pub context_window: i32,
    #[garde(range(min = 1.0, max = 2.0))]
    pub repeat_penalty: f32,
    #[garde(skip)]
    pub system_prompt: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct GameConfig {
    #[garde(range(min = 1, max = 50))]
    pub starting_level: u32,
    #[garde(range(max = 999999))]
    pub starting_caps: u32,
    #[garde(skip)]
    pub permadeath: bool,
    #[garde(skip)]
    pub autosave_interval: u32,
}

impl Config {
    pub fn load() -> Result<Self, GameError> {
        tracing::debug!("Loading configuration from config.toml");
        let config_str = fs::read_to_string("config.toml")?;
        let config: Config = toml::from_str(&config_str)?;

        // Validate configuration
        config.validate()?;

        tracing::info!("Configuration loaded and validated successfully");
        Ok(config)
    }

    /// Load configuration with environment variable overrides
    pub fn load_with_env() -> Result<Self, GameError> {
        let mut config = Self::load().unwrap_or_else(|_| {
            tracing::warn!("Failed to load config.toml, using defaults");
            Self::default()
        });

        // Override with environment variables if present
        if let Ok(url) = std::env::var("LLAMA_SERVER_URL") {
            tracing::info!("Overriding narrative AI URL from environment: {}", url);
            config.llama.server_url = url;
        }

        if let Ok(url) = std::env::var("EXTRACTION_AI_URL") {
            tracing::info!("Overriding extraction AI URL from environment: {}", url);
            config.llama.extraction_url = url;
        }

        config.validate()?;
        Ok(config)
    }

    /// Validate configuration values using garde
    pub fn validate(&self) -> Result<(), GameError> {
        <Self as Validate>::validate(self).map_err(|e| {
            // Map garde validation errors to specific ConfigError types
            let error_msg = e.to_string();

            if error_msg.contains("temperature") {
                ConfigError::InvalidTemperature(self.llama.temperature).into()
            } else if error_msg.contains("top_p") {
                ConfigError::InvalidTopP(self.llama.top_p).into()
            } else if error_msg.contains("top_k") {
                ConfigError::InvalidTopK(self.llama.top_k).into()
            } else if error_msg.contains("max_tokens") {
                ConfigError::InvalidMaxTokens(self.llama.max_tokens).into()
            } else if error_msg.contains("context_window") {
                ConfigError::InvalidContextWindow(self.llama.context_window).into()
            } else if error_msg.contains("repeat_penalty") {
                ConfigError::InvalidRepeatPenalty(self.llama.repeat_penalty).into()
            } else if error_msg.contains("starting_level") {
                ConfigError::InvalidStartingLevel(self.game.starting_level).into()
            } else if error_msg.contains("starting_caps") {
                ConfigError::InvalidStartingCaps(self.game.starting_caps).into()
            } else {
                GameError::InvalidInput(format!("Configuration validation failed: {}", e))
            }
        })?;

        tracing::debug!("Configuration validation passed");
        Ok(())
    }
}

impl Default for Config {
    fn default() -> Self {
        Config {
            llama: LlamaConfig {
                server_url: "http://localhost:8080".to_string(),
                extraction_url: "http://localhost:8081".to_string(),
                temperature: 0.8,
                top_p: 0.9,
                top_k: 40,
                max_tokens: 2048,     // Increased for complex narratives
                context_window: 8192, // Standard context window for most llama.cpp models
                repeat_penalty: 1.1,
                system_prompt: "You are a Fallout universe DM.".to_string(),
            },
            game: GameConfig {
                starting_level: 1,
                starting_caps: 500,
                permadeath: false,
                autosave_interval: 5,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn get_valid_config() -> Config {
        Config::default()
    }

    #[test]
    fn test_validate_valid_config() {
        let config = get_valid_config();
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_validate_invalid_temperature() {
        let mut config = get_valid_config();
        config.llama.temperature = -1.0;
        assert!(config.validate().is_err());
        config.llama.temperature = 3.0;
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_validate_invalid_top_p() {
        let mut config = get_valid_config();
        config.llama.top_p = -1.0;
        assert!(config.validate().is_err());
        config.llama.top_p = 2.0;
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_validate_invalid_top_k() {
        let mut config = get_valid_config();
        config.llama.top_k = 0;
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_validate_invalid_max_tokens() {
        let mut config = get_valid_config();
        config.llama.max_tokens = 0;
        assert!(config.validate().is_err());
        config.llama.max_tokens = 100_000;
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_validate_invalid_repeat_penalty() {
        let mut config = get_valid_config();
        config.llama.repeat_penalty = 0.0;
        assert!(config.validate().is_err());
        config.llama.repeat_penalty = 3.0;
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_validate_invalid_starting_level() {
        let mut config = get_valid_config();
        config.game.starting_level = 0;
        assert!(config.validate().is_err());
        config.game.starting_level = 100;
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_validate_invalid_starting_caps() {
        let mut config = get_valid_config();
        config.game.starting_caps = 1_000_000;
        assert!(config.validate().is_err());
    }
}
