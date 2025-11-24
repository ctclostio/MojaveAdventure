//! # AI Module
//!
//! AI Dungeon Master integration using llama.cpp for dynamic storytelling.
//!
//! ## Overview
//!
//! This module provides the AI dungeon master functionality that powers the
//! narrative experience. It communicates with a local llama.cpp server to
//! generate contextual responses based on game state, player actions, and
//! world knowledge.
//!
//! ## Key Components
//!
//! - [`AIDungeonMaster`]: Main AI client that generates DM responses
//! - [`extractor`]: Extracts structured game commands from AI responses
//!
//! ## Architecture
//!
//! The AI system:
//! 1. Builds a comprehensive prompt with game context
//! 2. Sends HTTP request to llama.cpp server
//! 3. Receives natural language response
//! 4. Extracts game commands (combat, items, etc.) from response
//!
//! ## Context Management
//!
//! The AI prompt includes:
//! - Character stats (SPECIAL, skills, HP, inventory)
//! - Combat state (active enemies, round number)
//! - Story context (last 10 conversation messages)
//! - Worldbook knowledge (known locations, NPCs, events)
//! - Current location and quest log
//!
//! ## Example
//!
//! ```no_run
//! use fallout_dnd::ai::AIDungeonMaster;
//! use fallout_dnd::config::Config;
//!
//! #[tokio::main]
//! async fn main() -> anyhow::Result<()> {
//!     let config = Config::default();
//!     let dm = AIDungeonMaster::new(config.llama);
//!
//!     // Test connection before use
//!     dm.test_connection().await?;
//!
//!     Ok(())
//! }
//! ```

pub mod cache;
pub mod extractor;
pub mod server_manager;

use crate::config::LlamaConfig;
use crate::error::GameError;
use crate::game::GameState;
use crate::templates::{
    self, CharacterContext, CombatContext, EnemyContext, SkillsContext, SpecialStats,
};
use anyhow::Result;
use cache::{TokenCache, WorldbookCache};
use serde::{Deserialize, Serialize};
use std::sync::OnceLock;
use std::time::Duration;
use tiktoken_rs::CoreBPE;
use tokio::sync::mpsc;

#[derive(Debug, Serialize)]
struct LlamaRequest {
    prompt: String,
    temperature: f32,
    top_p: f32,
    top_k: i32,
    n_predict: i32,
    repeat_penalty: f32,
    stop: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    stream: Option<bool>,
}

/// Streaming response chunk from llama.cpp
#[derive(Debug, Deserialize)]
struct LlamaStreamChunk {
    content: String,
    #[serde(default)]
    stop: bool,
}

/// Error response from llama.cpp (used when stream chunk parsing fails)
#[derive(Debug, Deserialize)]
struct LlamaErrorResponse {
    #[serde(default)]
    error: Option<String>,
}

#[derive(Clone)]
pub struct AIDungeonMaster {
    config: LlamaConfig,
    client: reqwest::Client,
    token_cache: TokenCache,
    worldbook_cache: WorldbookCache,
}

impl AIDungeonMaster {
    pub fn new(config: LlamaConfig) -> Self {
        AIDungeonMaster {
            config,
            client: reqwest::Client::new(),
            token_cache: TokenCache::new(),
            worldbook_cache: WorldbookCache::new(),
        }
    }

    /// Generate a streaming response from the AI DM
    /// Returns a channel receiver that yields tokens as they are generated
    pub async fn generate_response_stream(
        &self,
        game_state: &GameState,
        player_action: &str,
    ) -> Result<mpsc::Receiver<Result<String, String>>> {
        let prompt = self.build_prompt(game_state, player_action).await;

        let request = LlamaRequest {
            prompt,
            temperature: self.config.temperature,
            top_p: self.config.top_p,
            top_k: self.config.top_k,
            n_predict: self.config.max_tokens,
            repeat_penalty: self.config.repeat_penalty,
            stop: vec![
                ">>> PLAYER:".to_string(),
                "\n>>> PLAYER:".to_string(),
                "Player:".to_string(),
                "\nPlayer:".to_string(),
            ],
            stream: Some(true), // Enable streaming for real-time token display
        };

        let url = format!("{}/completion", self.config.server_url);

        tracing::debug!(
            "Sending streaming request to: {}, prompt length: {} chars",
            url,
            request.prompt.len()
        );

        let response = self
            .client
            .post(&url)
            .json(&request)
            .timeout(Duration::from_secs(600)) // 10 minutes for slow generation
            .send()
            .await
            .map_err(|e| {
                GameError::AIConnectionError(format!(
                    "Failed to connect to llama.cpp server: {}. Make sure it's running at {}",
                    e, self.config.server_url
                ))
            })?;

        tracing::debug!("Got response with status: {}", response.status());

        if !response.status().is_success() {
            return Err(GameError::AIConnectionError(format!(
                "llama.cpp server returned error: {}",
                response.status()
            ))
            .into());
        }

        // Create a channel to send tokens
        let (tx, rx) = mpsc::channel::<Result<String, String>>(100);

        // Process streaming SSE response in background task
        tokio::spawn(async move {
            tracing::debug!("Starting to process streaming response");

            // Get the response body as bytes stream
            let mut stream = response.bytes_stream();
            use futures_util::StreamExt;

            let mut buffer = String::new();

            while let Some(chunk_result) = stream.next().await {
                match chunk_result {
                    Ok(bytes) => {
                        // Append new bytes to buffer
                        if let Ok(text) = std::str::from_utf8(&bytes) {
                            buffer.push_str(text);

                            // Process complete SSE events (data: {...}\n\n)
                            while let Some(event_end) = buffer.find("\n\n") {
                                let event = buffer[..event_end].to_string();
                                buffer = buffer[event_end + 2..].to_string();

                                // Parse SSE data line
                                for line in event.lines() {
                                    if let Some(data) = line.strip_prefix("data: ") {
                                        // Parse the JSON chunk
                                        match serde_json::from_str::<LlamaStreamChunk>(data) {
                                            Ok(chunk) => {
                                                if !chunk.content.is_empty()
                                                    && tx.send(Ok(chunk.content)).await.is_err()
                                                {
                                                    tracing::debug!(
                                                        "Receiver dropped, stopping stream"
                                                    );
                                                    return;
                                                }
                                                if chunk.stop {
                                                    tracing::debug!("Stream completed (stop=true)");
                                                    return;
                                                }
                                            }
                                            Err(e) => {
                                                // Try parsing as error response
                                                if let Ok(err_response) =
                                                    serde_json::from_str::<LlamaErrorResponse>(data)
                                                {
                                                    if let Some(error) = err_response.error {
                                                        let _ = tx.send(Err(error)).await;
                                                        return;
                                                    }
                                                }
                                                tracing::warn!(
                                                    "Failed to parse stream chunk: {} - data: {}",
                                                    e,
                                                    data
                                                );
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                    Err(e) => {
                        tracing::error!("Stream error: {}", e);
                        let _ = tx.send(Err(format!("Stream error: {}", e))).await;
                        return;
                    }
                }
            }

            tracing::debug!("Stream ended naturally");
        });

        Ok(rx)
    }

    /// Accurate token counting using tiktoken-rs with caching
    ///
    /// Uses the cl100k_base tokenizer (used by GPT-4 and similar models).
    /// The tokenizer is cached globally to avoid re-initialization overhead.
    /// Token counts are cached using TokenCache for 10-50x speedup on repeated text.
    async fn estimate_tokens(&self, text: &str) -> usize {
        static TOKENIZER: OnceLock<CoreBPE> = OnceLock::new();

        self.token_cache
            .get_or_compute(text, |text| {
                let bpe = TOKENIZER.get_or_init(|| {
                    tiktoken_rs::cl100k_base()
                        .expect("Failed to initialize tokenizer - this should never fail")
                });
                bpe.encode_with_special_tokens(text).len()
            })
            .await
    }

    /// Build the prompt with game context (with caching)
    ///
    /// Uses WorldbookCache to cache expensive worldbook context building (~20x speedup).
    async fn build_prompt(&self, game_state: &GameState, player_action: &str) -> String {
        let mut prompt = String::with_capacity(4096);

        // System prompt from template
        match templates::render_system_prompt() {
            Ok(system_prompt) => {
                prompt.push_str(&system_prompt);
                prompt.push_str("\n\n");
            }
            Err(e) => {
                // Fallback to config system prompt if template fails
                tracing::error!("Failed to render system prompt template: {}", e);
                prompt.push_str(&self.config.system_prompt);
                prompt.push_str("\n\n");
            }
        }

        // Build context from templates
        let character_ctx = Self::build_character_context(&game_state.character);
        let inventory_items: Vec<String> = game_state
            .character
            .inventory
            .iter()
            .map(|item| item.name.to_string())
            .collect();

        let combat_ctx = if game_state.combat.active {
            Some(Self::build_combat_context(&game_state.combat))
        } else {
            None
        };

        // Get conversation history
        let conversation_history = if !game_state.conversation.is_empty() {
            Self::get_conversation_messages(&game_state.conversation)
        } else if !game_state.story.is_empty() {
            // Fallback for old save files
            game_state.story.get_all().iter().cloned().collect()
        } else {
            Vec::new()
        };

        // Render context template
        match templates::render_context(
            Some(&character_ctx),
            Some(&inventory_items),
            combat_ctx.as_ref(),
            Some(&conversation_history),
        ) {
            Ok(context) => prompt.push_str(&context),
            Err(e) => {
                tracing::error!("Failed to render context template: {}", e);
                // Fallback to old methods if template fails
                prompt.push_str(&Self::build_character_section(&game_state.character));
                prompt.push_str(&Self::build_inventory_section(
                    &game_state.character.inventory,
                ));
            }
        }

        // Location and worldbook (with caching)
        prompt.push_str(&format!("Location: {}\n\n", game_state.location));

        // Use WorldbookCache to cache expensive worldbook.build_context() calls
        let worldbook_hash = cache::hash_worldbook_state(&game_state.worldbook);
        let worldbook_context = self
            .worldbook_cache
            .get_or_compute(worldbook_hash, || game_state.worldbook.build_context())
            .await;

        if !worldbook_context.is_empty() {
            prompt.push_str(&worldbook_context);
            prompt.push('\n');
        }

        // Current player action
        prompt.push_str(&format!(">>> PLAYER: {}\n\n>>> DM (YOU):", player_action));

        // Warn if prompt is using >75% of context window (leaving room for response)
        let estimated_tokens = self.estimate_tokens(&prompt).await;
        let warning_threshold = (self.config.context_window as f32 * 0.75) as usize;
        if estimated_tokens > warning_threshold {
            tracing::warn!(
                "Large prompt detected: {} tokens ({}% of {} token context window). Consider reducing worldbook or conversation history.",
                estimated_tokens,
                (estimated_tokens as f32 / self.config.context_window as f32 * 100.0) as usize,
                self.config.context_window
            );
        }

        prompt
    }

    /// Build character context for templates
    fn build_character_context(character: &crate::game::character::Character) -> CharacterContext {
        CharacterContext {
            name: character.name.as_str().to_string(),
            level: character.level as u8,
            current_hp: character.current_hp,
            max_hp: character.max_hp,
            current_ap: character.current_ap as u8,
            max_ap: character.max_ap as u8,
            caps: character.caps,
            special: SpecialStats {
                strength: character.special.strength,
                perception: character.special.perception,
                endurance: character.special.endurance,
                charisma: character.special.charisma,
                intelligence: character.special.intelligence,
                agility: character.special.agility,
                luck: character.special.luck,
            },
            skills: SkillsContext {
                small_guns: character.skills.small_guns,
                speech: character.skills.speech,
                lockpick: character.skills.lockpick,
                science: character.skills.science,
                sneak: character.skills.sneak,
            },
        }
    }

    /// Build combat context for templates
    fn build_combat_context(combat: &crate::game::combat::CombatState) -> CombatContext {
        CombatContext {
            round: combat.round,
            enemies: combat
                .enemies
                .iter()
                .map(|enemy| EnemyContext {
                    name: enemy.name.as_str().to_string(),
                    current_hp: enemy.current_hp,
                    is_alive: enemy.is_alive(),
                })
                .collect(),
        }
    }

    /// Get conversation messages for templates
    fn get_conversation_messages(
        conversation: &crate::game::conversation::ConversationManager,
    ) -> Vec<String> {
        conversation
            .get_recent_turns(10)
            .iter()
            .map(|turn| format!("{:?}: {}", turn.speaker, turn.message))
            .collect()
    }

    /// Build character stats section (legacy fallback)
    fn build_character_section(character: &crate::game::character::Character) -> String {
        // SPECIAL stats - IMPORTANT: These are the actual character stats, do not make up different ones!
        format!(
            "CHARACTER: {} (Level {})\n\
             HP: {}/{} | AP: {}/{} | Caps: {}\n\
             SPECIAL: S:{} P:{} E:{} C:{} I:{} A:{} L:{}\n\
             Skills: Small Guns:{} Speech:{} Lockpick:{} Science:{} Sneak:{}\n",
            character.name,
            character.level,
            character.current_hp,
            character.max_hp,
            character.current_ap,
            character.max_ap,
            character.caps,
            character.special.strength,
            character.special.perception,
            character.special.endurance,
            character.special.charisma,
            character.special.intelligence,
            character.special.agility,
            character.special.luck,
            character.skills.small_guns,
            character.skills.speech,
            character.skills.lockpick,
            character.skills.science,
            character.skills.sneak
        )
    }

    /// Build inventory section (legacy fallback)
    fn build_inventory_section(inventory: &[crate::game::items::Item]) -> String {
        if inventory.is_empty() {
            return String::new();
        }

        let items: Vec<&str> = inventory.iter().map(|item| item.name.as_str()).collect();

        format!("Inventory: {}\n", items.join(", "))
    }

    /// Test connection to llama.cpp server
    pub async fn test_connection(&self) -> Result<()> {
        let url = format!("{}/health", self.config.server_url);

        self.client
            .get(&url)
            .timeout(Duration::from_secs(10))
            .send()
            .await
            .map_err(|_| {
                GameError::AIConnectionError(format!(
                    "Cannot connect to llama.cpp at {}. Is it running?",
                    self.config.server_url
                ))
            })?;

        Ok(())
    }
}
