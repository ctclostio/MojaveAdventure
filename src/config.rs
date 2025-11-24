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

fn default_narrative_ctx_size() -> i32 {
    8192
}

fn default_extraction_ctx_size() -> i32 {
    4096
}

fn default_narrative_threads() -> i32 {
    8
}

fn default_extraction_threads() -> i32 {
    6
}

fn default_narrative_gpu_layers() -> i32 {
    99 // Offload all layers to GPU by default (99 means "all available")
}

fn default_extraction_gpu_layers() -> i32 {
    99 // Offload all layers to GPU by default
}

fn default_flash_attention() -> bool {
    true // Enable Flash Attention by default for faster GPU inference
}

fn default_continuous_batching() -> bool {
    true // Enable continuous batching by default for better throughput
}

fn default_no_kv_offload() -> bool {
    true // Keep KV cache in VRAM for faster inference (uses more VRAM)
}

fn default_mmap() -> bool {
    true // Memory-map the model file for faster loading
}

fn default_mlock() -> bool {
    false // Don't lock model in RAM by default (can cause issues on some systems)
}

fn default_batch_size() -> i32 {
    2048 // Large batch size for faster prompt processing
}

fn default_ubatch_size() -> i32 {
    512 // Micro batch size for parallelism
}

fn default_cache_type_k() -> String {
    "q8_0".to_string() // Quantize K cache for speed
}

fn default_cache_type_v() -> String {
    "q8_0".to_string() // Quantize V cache for speed
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
    // Server auto-start configuration
    #[garde(skip)]
    #[serde(default)]
    pub auto_start: bool,
    #[garde(skip)]
    #[serde(default)]
    pub llama_server_path: Option<String>,
    #[garde(skip)]
    #[serde(default)]
    pub narrative_model_path: Option<String>,
    #[garde(skip)]
    #[serde(default)]
    pub extraction_model_path: Option<String>,
    #[garde(range(min = 512, max = 131072))]
    #[serde(default = "default_narrative_ctx_size")]
    pub narrative_ctx_size: i32,
    #[garde(range(min = 512, max = 131072))]
    #[serde(default = "default_extraction_ctx_size")]
    pub extraction_ctx_size: i32,
    #[garde(range(min = 1, max = 32))]
    #[serde(default = "default_narrative_threads")]
    pub narrative_threads: i32,
    #[garde(range(min = 1, max = 32))]
    #[serde(default = "default_extraction_threads")]
    pub extraction_threads: i32,
    #[garde(range(min = 0, max = 200))]
    #[serde(default = "default_narrative_gpu_layers")]
    pub narrative_gpu_layers: i32,
    #[garde(range(min = 0, max = 200))]
    #[serde(default = "default_extraction_gpu_layers")]
    pub extraction_gpu_layers: i32,
    /// Enable Flash Attention for faster GPU inference (requires CUDA)
    #[garde(skip)]
    #[serde(default = "default_flash_attention")]
    pub flash_attention: bool,
    /// Enable continuous batching for better throughput
    #[garde(skip)]
    #[serde(default = "default_continuous_batching")]
    pub continuous_batching: bool,
    /// Keep KV cache in VRAM for faster inference (uses more VRAM)
    #[garde(skip)]
    #[serde(default = "default_no_kv_offload")]
    pub no_kv_offload: bool,
    /// Memory-map the model file for faster loading
    #[garde(skip)]
    #[serde(default = "default_mmap")]
    pub mmap: bool,
    /// Lock model in RAM to prevent swapping
    #[garde(skip)]
    #[serde(default = "default_mlock")]
    pub mlock: bool,
    /// Batch size for prompt processing (higher = faster but more VRAM)
    #[garde(range(min = 1, max = 8192))]
    #[serde(default = "default_batch_size")]
    pub batch_size: i32,
    /// Micro batch size for parallelism
    #[garde(range(min = 1, max = 4096))]
    #[serde(default = "default_ubatch_size")]
    pub ubatch_size: i32,
    /// KV cache quantization type for K (q8_0, q4_0, f16, f32)
    #[garde(skip)]
    #[serde(default = "default_cache_type_k")]
    pub cache_type_k: String,
    /// KV cache quantization type for V (q8_0, q4_0, f16, f32)
    #[garde(skip)]
    #[serde(default = "default_cache_type_v")]
    pub cache_type_v: String,
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
                auto_start: true,
                llama_server_path: Some("llama-cpp/llama-server.exe".to_string()),
                narrative_model_path: Some(
                    // GPT-OSS-20B: OpenAI's open-weight model (requires llama.cpp b6096+)
                    "llama-cpp/models/gpt-oss-20b-q4_k_m.gguf".to_string(),
                ),
                extraction_model_path: Some(
                    "llama-cpp/models/Hermes-2-Pro-Llama-3-8B-Q4_K_M.gguf".to_string(),
                ),
                narrative_ctx_size: 8192,
                extraction_ctx_size: 4096,
                narrative_threads: 8,
                extraction_threads: 6,
                narrative_gpu_layers: 99,  // Offload all layers to GPU
                extraction_gpu_layers: 99, // Offload all layers to GPU
                // ===== SPEED OPTIMIZATIONS =====
                flash_attention: true, // Flash Attention for faster inference
                continuous_batching: true, // Continuous batching for throughput
                no_kv_offload: true,   // Keep KV cache in VRAM (faster)
                mmap: true,            // Memory-map for faster loading
                mlock: false,          // Don't lock in RAM by default
                batch_size: 2048,      // Large batch for fast prompt processing
                ubatch_size: 512,      // Micro batch for parallelism
                cache_type_k: "q8_0".to_string(), // Quantized K cache
                cache_type_v: "q8_0".to_string(), // Quantized V cache
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
