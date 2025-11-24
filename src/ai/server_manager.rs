//! AI Server Process Manager
//!
//! Manages the lifecycle of llama.cpp server processes for the AI Dungeon Master.
//! Handles automatic startup, health checking, and cleanup of server processes.

use crate::error::GameError;
use anyhow::Result;
use std::path::PathBuf;
use std::process::{Child, Command};
use std::time::Duration;
use tokio::time::sleep;

/// Configuration for a single AI server instance
#[derive(Debug, Clone)]
pub struct ServerConfig {
    /// Path to the llama-server executable
    pub executable: PathBuf,
    /// Path to the model file
    pub model_path: PathBuf,
    /// Port number to run on
    pub port: u16,
    /// Context size for the model
    pub ctx_size: usize,
    /// Number of threads to use
    pub threads: usize,
    /// Number of layers to offload to GPU (0 = CPU only, 99 = all layers)
    pub gpu_layers: usize,
    /// Server URL (e.g., "http://localhost:8080")
    pub url: String,
    /// Human-readable name for this server
    pub name: String,
    // ===== SPEED OPTIMIZATIONS =====
    /// Enable Flash Attention for faster GPU inference
    pub flash_attention: bool,
    /// Enable continuous batching for better throughput
    pub continuous_batching: bool,
    /// Keep KV cache in VRAM (faster but uses more VRAM)
    pub no_kv_offload: bool,
    /// Memory-map the model file for faster loading
    pub mmap: bool,
    /// Lock model in RAM to prevent swapping
    pub mlock: bool,
    /// Batch size for prompt processing
    pub batch_size: i32,
    /// Micro batch size for parallelism
    pub ubatch_size: i32,
    /// KV cache quantization type for K (q8_0, q4_0, f16, f32)
    pub cache_type_k: String,
    /// KV cache quantization type for V (q8_0, q4_0, f16, f32)
    pub cache_type_v: String,
}

/// Manages AI server processes
pub struct ServerManager {
    narrative_config: Option<ServerConfig>,
    extraction_config: Option<ServerConfig>,
    narrative_process: Option<Child>,
    extraction_process: Option<Child>,
}

impl ServerManager {
    /// Create a new server manager with the given configurations
    pub fn new(
        narrative_config: Option<ServerConfig>,
        extraction_config: Option<ServerConfig>,
    ) -> Self {
        Self {
            narrative_config,
            extraction_config,
            narrative_process: None,
            extraction_process: None,
        }
    }

    /// Start both servers if they're not already running
    pub async fn ensure_servers_running(&mut self) -> Result<()> {
        // Start narrative server if configured
        if let Some(config) = &self.narrative_config {
            if !self.is_server_running(&config.url).await {
                tracing::info!("Starting narrative AI server on port {}", config.port);
                self.narrative_process = Some(self.start_server(config)?);
            } else {
                tracing::info!("Narrative AI server already running at {}", config.url);
            }
        }

        // Start extraction server if configured
        if let Some(config) = &self.extraction_config {
            if !self.is_server_running(&config.url).await {
                tracing::info!("Starting extraction AI server on port {}", config.port);
                self.extraction_process = Some(self.start_server(config)?);
            } else {
                tracing::info!("Extraction AI server already running at {}", config.url);
            }
        }

        // Wait for servers to be ready
        if self.narrative_process.is_some() || self.extraction_process.is_some() {
            tracing::info!("Waiting for AI servers to initialize...");
            self.wait_for_servers_ready().await?;
        }

        Ok(())
    }

    /// Start a single server process
    fn start_server(&self, config: &ServerConfig) -> Result<Child> {
        // Convert paths to absolute paths
        let current_dir = std::env::current_dir().map_err(|e| {
            GameError::AIConnectionError(format!("Failed to get current directory: {}", e))
        })?;

        let executable = if config.executable.is_absolute() {
            config.executable.clone()
        } else {
            current_dir.join(&config.executable)
        };

        let model_path = if config.model_path.is_absolute() {
            config.model_path.clone()
        } else {
            current_dir.join(&config.model_path)
        };

        tracing::info!(
            "Starting {} server: exe={:?}, model={:?}",
            config.name,
            executable,
            model_path
        );

        // Redirect stdout/stderr to log files for debugging
        let log_dir = current_dir.join("llama-cpp");
        std::fs::create_dir_all(&log_dir).ok();

        let log_file_name = format!("server-{}.log", config.port);
        let log_path = log_dir.join(log_file_name);

        let log_file = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&log_path)
            .map_err(|e| GameError::AIConnectionError(format!("Failed to open log file: {}", e)))?;

        // Build command with required parameters
        let mut cmd = Command::new(&executable);
        cmd.arg("-m")
            .arg(&model_path)
            .arg("--port")
            .arg(config.port.to_string())
            .arg("-c")
            .arg(config.ctx_size.to_string())
            .arg("--threads")
            .arg(config.threads.to_string())
            .arg("-ngl") // GPU layers parameter
            .arg(config.gpu_layers.to_string());

        // ===== SPEED OPTIMIZATIONS =====
        let mut speedhacks: Vec<&str> = Vec::new();

        // Flash Attention for faster GPU inference
        if config.flash_attention {
            cmd.arg("-fa");
            speedhacks.push("flash-attn");
        }

        // Continuous batching for better throughput
        if config.continuous_batching {
            cmd.arg("-cb");
            speedhacks.push("cont-batch");
        }

        // Keep KV cache in VRAM (faster inference, uses more VRAM)
        if config.no_kv_offload {
            cmd.arg("--no-kv-offload");
            speedhacks.push("no-kv-offload");
        }

        // Memory-map is enabled by default in modern llama.cpp
        // No need to pass --mmap flag (it's not supported in newer versions)
        if config.mmap {
            speedhacks.push("mmap(default)");
        }

        // Lock model in RAM to prevent swapping
        if config.mlock {
            cmd.arg("--mlock");
            speedhacks.push("mlock");
        }

        // Batch size for prompt processing
        if config.batch_size > 0 {
            cmd.arg("-b").arg(config.batch_size.to_string());
        }

        // Micro batch size for parallelism
        if config.ubatch_size > 0 {
            cmd.arg("-ub").arg(config.ubatch_size.to_string());
        }

        // KV cache quantization
        if !config.cache_type_k.is_empty() && config.cache_type_k != "f16" {
            cmd.arg("--cache-type-k").arg(&config.cache_type_k);
            speedhacks.push("kv-quant");
        }
        if !config.cache_type_v.is_empty() && config.cache_type_v != "f16" {
            cmd.arg("--cache-type-v").arg(&config.cache_type_v);
        }

        tracing::info!(
            "{} server speedhacks enabled: [{}]",
            config.name,
            speedhacks.join(", ")
        );

        let child = cmd
            .stdout(log_file.try_clone().unwrap())
            .stderr(log_file)
            .spawn()
            .map_err(|e| {
                GameError::AIConnectionError(format!(
                    "Failed to start {} server: {}. Make sure llama-server exists at {:?}",
                    config.name, e, executable
                ))
            })?;

        tracing::info!(
            "Started {} server (PID: {}) - logs: {:?}",
            config.name,
            child.id(),
            log_path
        );

        Ok(child)
    }

    /// Check if a server is responding at the given URL
    async fn is_server_running(&self, url: &str) -> bool {
        let health_url = format!("{}/health", url);
        let client = reqwest::Client::new();

        match client
            .get(&health_url)
            .timeout(Duration::from_secs(2))
            .send()
            .await
        {
            Ok(response) => response.status().is_success(),
            Err(_) => false,
        }
    }

    /// Wait for all started servers to be ready to accept requests
    async fn wait_for_servers_ready(&self) -> Result<()> {
        let max_wait = Duration::from_secs(60); // Wait up to 60 seconds
        let check_interval = Duration::from_secs(2);
        let start = std::time::Instant::now();

        // Determine which servers we need to wait for
        let mut servers_to_check = Vec::new();

        if self.narrative_process.is_some() {
            if let Some(config) = &self.narrative_config {
                servers_to_check.push((&config.url, &config.name));
            }
        }

        if self.extraction_process.is_some() {
            if let Some(config) = &self.extraction_config {
                servers_to_check.push((&config.url, &config.name));
            }
        }

        if servers_to_check.is_empty() {
            return Ok(());
        }

        let mut ready_servers = vec![false; servers_to_check.len()];

        while start.elapsed() < max_wait {
            // Check each server
            for (i, (url, name)) in servers_to_check.iter().enumerate() {
                if !ready_servers[i] && self.is_server_running(url).await {
                    tracing::info!("{} server is ready", name);
                    ready_servers[i] = true;
                }
            }

            // If all servers are ready, we're done
            if ready_servers.iter().all(|&ready| ready) {
                tracing::info!("All AI servers are ready");
                return Ok(());
            }

            sleep(check_interval).await;
        }

        // Check which servers failed to start
        let failed_servers: Vec<&str> = servers_to_check
            .iter()
            .enumerate()
            .filter_map(|(i, (_, name))| {
                if !ready_servers[i] {
                    Some(name.as_str())
                } else {
                    None
                }
            })
            .collect();

        Err(GameError::AIConnectionError(format!(
            "Timeout waiting for servers to start: {:?}. Check that model files exist and llama-server is compatible.",
            failed_servers
        ))
        .into())
    }

    /// Stop all running servers
    pub fn stop_servers(&mut self) {
        if let Some(mut process) = self.narrative_process.take() {
            tracing::info!("Stopping narrative AI server (PID: {})", process.id());
            let _ = process.kill();
        }

        if let Some(mut process) = self.extraction_process.take() {
            tracing::info!("Stopping extraction AI server (PID: {})", process.id());
            let _ = process.kill();
        }
    }
}

impl Drop for ServerManager {
    fn drop(&mut self) {
        self.stop_servers();
    }
}
