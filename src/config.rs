use crate::error::{ConfigError, GameError};
use serde::{Deserialize, Serialize};
use std::fs;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub llama: LlamaConfig,
    pub game: GameConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlamaConfig {
    pub server_url: String,
    pub extraction_url: String,
    pub temperature: f32,
    pub top_p: f32,
    pub top_k: i32,
    pub max_tokens: i32,
    pub repeat_penalty: f32,
    pub system_prompt: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameConfig {
    pub starting_level: u32,
    pub starting_caps: u32,
    pub permadeath: bool,
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

    /// Validate configuration values
    pub fn validate(&self) -> Result<(), GameError> {
        // Validate temperature
        if !(0.0..=2.0).contains(&self.llama.temperature) {
            return Err(ConfigError::InvalidTemperature(self.llama.temperature).into());
        }

        // Validate top_p
        if !(0.0..=1.0).contains(&self.llama.top_p) {
            return Err(ConfigError::InvalidTopP(self.llama.top_p).into());
        }

        // Validate top_k
        if self.llama.top_k < 1 {
            return Err(ConfigError::InvalidTopK(self.llama.top_k).into());
        }

        // Validate max_tokens
        if !(1..=32000).contains(&self.llama.max_tokens) {
            return Err(ConfigError::InvalidMaxTokens(self.llama.max_tokens).into());
        }

        // Validate repeat_penalty
        if !(1.0..=2.0).contains(&self.llama.repeat_penalty) {
            return Err(ConfigError::InvalidRepeatPenalty(self.llama.repeat_penalty).into());
        }

        // Validate starting level
        if !(1..=50).contains(&self.game.starting_level) {
            return Err(ConfigError::InvalidStartingLevel(self.game.starting_level).into());
        }

        // Validate starting caps
        if self.game.starting_caps > 999_999 {
            return Err(ConfigError::InvalidStartingCaps(self.game.starting_caps).into());
        }

        tracing::debug!("Configuration validation passed");
        Ok(())
    }

    pub fn default() -> Self {
        Config {
            llama: LlamaConfig {
                server_url: "http://localhost:8080".to_string(),
                extraction_url: "http://localhost:8081".to_string(),
                temperature: 0.8,
                top_p: 0.9,
                top_k: 40,
                max_tokens: 2048,  // Increased for complex narratives
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
