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
//! use fallout_dnd::game::{GameState, character::{Character, Special}};
//!
//! #[tokio::main]
//! async fn main() -> anyhow::Result<()> {
//!     let config = Config::default();
//!     let dm = AIDungeonMaster::new(config.llama);
//!
//!     // Test connection
//!     dm.test_connection().await?;
//!
//!     // Generate a response
//!     let special = Special::new();
//!     let character = Character::new("Wanderer".to_string(), special);
//!     let game = GameState::new(character);
//!     let response = dm.generate_response(&game, "I explore the ruins").await?;
//!     println!("DM: {}", response);
//!
//!     Ok(())
//! }
//! ```

pub mod cache;
pub mod extractor;

use crate::config::LlamaConfig;
use crate::error::GameError;
use crate::game::GameState;
use crate::templates::{
    self, CharacterContext, CombatContext, EnemyContext, SkillsContext, SpecialStats,
};
use anyhow::Result;
use futures_util::StreamExt;
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

#[derive(Debug, Deserialize)]
struct LlamaResponse {
    content: String,
    #[serde(default)]
    error: Option<String>,
}

#[derive(Clone)]
pub struct AIDungeonMaster {
    config: LlamaConfig,
    client: reqwest::Client,
}

impl AIDungeonMaster {
    pub fn new(config: LlamaConfig) -> Self {
        AIDungeonMaster {
            config,
            client: reqwest::Client::new(),
        }
    }

    /// Generate a response from the AI DM
    pub async fn generate_response(
        &self,
        game_state: &GameState,
        player_action: &str,
    ) -> Result<String> {
        let prompt = self.build_prompt(game_state, player_action);

        let request = LlamaRequest {
            prompt,
            temperature: self.config.temperature,
            top_p: self.config.top_p,
            top_k: self.config.top_k,
            n_predict: self.config.max_tokens,
            repeat_penalty: self.config.repeat_penalty,
            stop: vec!["\nPlayer:".to_string(), "\n>".to_string()],
            stream: None,
        };

        let url = format!("{}/completion", self.config.server_url);

        let response = self
            .client
            .post(&url)
            .json(&request)
            .timeout(Duration::from_secs(60))
            .send()
            .await
            .map_err(|e| {
                GameError::AIConnectionError(format!(
                    "Failed to connect to llama.cpp server: {}. Make sure it's running at {}",
                    e, self.config.server_url
                ))
            })?;

        if !response.status().is_success() {
            return Err(GameError::AIConnectionError(format!(
                "llama.cpp server returned error: {}",
                response.status()
            ))
            .into());
        }

        let llama_response: LlamaResponse = response.json().await?;

        if let Some(error) = llama_response.error {
            return Err(GameError::AIConnectionError(format!("llama.cpp error: {}", error)).into());
        }

        Ok(llama_response.content.trim().to_string())
    }

    /// Generate a streaming response from the AI DM
    /// Returns a channel receiver that yields tokens as they are generated
    pub async fn generate_response_stream(
        &self,
        game_state: &GameState,
        player_action: &str,
    ) -> Result<mpsc::Receiver<Result<String, String>>> {
        let prompt = self.build_prompt(game_state, player_action);

        let request = LlamaRequest {
            prompt,
            temperature: self.config.temperature,
            top_p: self.config.top_p,
            top_k: self.config.top_k,
            n_predict: self.config.max_tokens,
            repeat_penalty: self.config.repeat_penalty,
            stop: vec!["\nPlayer:".to_string(), "\n>".to_string()],
            stream: Some(true),
        };

        let url = format!("{}/completion", self.config.server_url);

        let response = self
            .client
            .post(&url)
            .json(&request)
            .timeout(Duration::from_secs(120))
            .send()
            .await
            .map_err(|e| {
                GameError::AIConnectionError(format!(
                    "Failed to connect to llama.cpp server: {}. Make sure it's running at {}",
                    e, self.config.server_url
                ))
            })?;

        if !response.status().is_success() {
            return Err(GameError::AIConnectionError(format!(
                "llama.cpp server returned error: {}",
                response.status()
            ))
            .into());
        }

        // Create a channel to send tokens
        let (tx, rx) = mpsc::channel::<Result<String, String>>(100);

        // Spawn a task to process the stream
        tokio::spawn(async move {
            let mut stream = response.bytes_stream();
            let mut buffer = String::new();

            while let Some(chunk_result) = stream.next().await {
                match chunk_result {
                    Ok(chunk) => {
                        // Parse the SSE data
                        if let Ok(text) = String::from_utf8(chunk.to_vec()) {
                            buffer.push_str(&text);

                            // Process complete SSE events
                            while let Some(event_end) = buffer.find("\n\n") {
                                let event = buffer[..event_end].to_string();
                                buffer = buffer[event_end + 2..].to_string();

                                // Extract the data field
                                for line in event.lines() {
                                    if let Some(data) = line.strip_prefix("data: ") {
                                        // Skip the [DONE] message
                                        if data == "[DONE]" {
                                            continue;
                                        }

                                        // Try to parse as JSON
                                        if let Ok(json) =
                                            serde_json::from_str::<serde_json::Value>(data)
                                        {
                                            if let Some(content) =
                                                json.get("content").and_then(|v| v.as_str())
                                            {
                                                if !content.is_empty()
                                                    && tx
                                                        .send(Ok(content.to_string()))
                                                        .await
                                                        .is_err()
                                                {
                                                    return; // Receiver dropped
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                    Err(e) => {
                        let _ = tx.send(Err(format!("Stream error: {}", e))).await;
                        return;
                    }
                }
            }
        });

        Ok(rx)
    }

    /// Accurate token counting using tiktoken-rs
    ///
    /// Uses the cl100k_base tokenizer (used by GPT-4 and similar models).
    /// The tokenizer is cached globally to avoid re-initialization overhead.
    fn estimate_tokens(text: &str) -> usize {
        static TOKENIZER: OnceLock<CoreBPE> = OnceLock::new();

        let bpe = TOKENIZER.get_or_init(|| {
            tiktoken_rs::cl100k_base()
                .expect("Failed to initialize tokenizer - this should never fail")
        });

        bpe.encode_with_special_tokens(text).len()
    }

    /// Build the prompt with game context
    fn build_prompt(&self, game_state: &GameState, player_action: &str) -> String {
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

        // Location and worldbook
        prompt.push_str(&format!("Location: {}\n\n", game_state.location));
        let worldbook_context = game_state.worldbook.build_context();
        if !worldbook_context.is_empty() {
            prompt.push_str(&worldbook_context);
            prompt.push('\n');
        }

        // Current player action
        prompt.push_str(&format!(">>> PLAYER: {}\n\n>>> DM (YOU):", player_action));

        // Warn if prompt is using >75% of context window (leaving room for response)
        let estimated_tokens = Self::estimate_tokens(&prompt);
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

    /// Build combat status section (legacy fallback)
    #[allow(dead_code)]
    fn build_combat_section(combat: &crate::game::combat::CombatState) -> String {
        // Pre-allocate: ~100 bytes header + ~50 bytes per enemy
        let mut section = String::with_capacity(100 + combat.enemies.len() * 50);

        section.push_str(&format!("IN COMBAT - Round {}\n", combat.round));
        section.push_str("Enemies:\n");
        for enemy in &combat.enemies {
            if enemy.is_alive() {
                section.push_str(&format!("  - {} (HP: {})\n", enemy.name, enemy.current_hp));
            }
        }
        section.push('\n');

        section
    }

    /// Build recent story context section
    ///
    /// DEPRECATED: This uses the old StoryManager. New code should use build_conversation_section.
    #[allow(dead_code)]
    fn build_story_section(story_context: &std::collections::VecDeque<String>) -> String {
        if story_context.is_empty() {
            return String::new();
        }

        let skip_count = story_context.len().saturating_sub(10);
        let num_events = story_context.len() - skip_count;

        // Pre-allocate: ~100 bytes header + ~100 bytes per story event
        let mut section = String::with_capacity(100 + num_events * 100);
        section.push_str("=== CONVERSATION HISTORY ===\n");
        section.push_str("(You are the DM. The player is the other speaker.)\n\n");

        // Take last 10 messages (or fewer if not enough)
        for msg in story_context.iter().skip(skip_count) {
            section.push_str(&format!("{}\n", msg));
        }
        section.push_str("\n=== END HISTORY ===\n\n");

        section
    }

    /// Build conversation context section using structured ConversationManager
    ///
    /// This is the preferred method for building conversation context.
    #[allow(dead_code)]
    fn build_conversation_section(
        conversation: &crate::game::conversation::ConversationManager,
    ) -> String {
        conversation.build_prompt_section(10)
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
