use anyhow::{anyhow, Context, Result};
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
    pub fn load() -> Result<Self> {
        tracing::debug!("Loading configuration from config.toml");
        let config_str = fs::read_to_string("config.toml").context("Failed to read config.toml")?;
        let config: Config = toml::from_str(&config_str)
            .context("Failed to parse config.toml - check TOML syntax")?;

        // Validate configuration
        config.validate()?;

        tracing::info!("Configuration loaded and validated successfully");
        Ok(config)
    }

    /// Load configuration with environment variable overrides
    pub fn load_with_env() -> Result<Self> {
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
    pub fn validate(&self) -> Result<()> {
        // Validate temperature
        if !(0.0..=2.0).contains(&self.llama.temperature) {
            return Err(anyhow!(
                "Invalid temperature: {}. Must be between 0.0 and 2.0",
                self.llama.temperature
            ));
        }

        // Validate top_p
        if !(0.0..=1.0).contains(&self.llama.top_p) {
            return Err(anyhow!(
                "Invalid top_p: {}. Must be between 0.0 and 1.0",
                self.llama.top_p
            ));
        }

        // Validate top_k
        if self.llama.top_k < 1 {
            return Err(anyhow!(
                "Invalid top_k: {}. Must be at least 1",
                self.llama.top_k
            ));
        }

        // Validate max_tokens
        if self.llama.max_tokens < 1 || self.llama.max_tokens > 32000 {
            return Err(anyhow!(
                "Invalid max_tokens: {}. Must be between 1 and 32000",
                self.llama.max_tokens
            ));
        }

        // Validate starting level
        if self.game.starting_level < 1 || self.game.starting_level > 50 {
            return Err(anyhow!(
                "Invalid starting_level: {}. Must be between 1 and 50",
                self.game.starting_level
            ));
        }

        // Validate starting caps
        if self.game.starting_caps > 999999 {
            return Err(anyhow!(
                "Invalid starting_caps: {}. Must be less than 1000000",
                self.game.starting_caps
            ));
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
                max_tokens: 512,
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
