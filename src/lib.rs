//! # Fallout D&D - AI-Driven Wasteland Adventures
//!
//! A text-based RPG that combines the Fallout universe with tabletop RPG mechanics,
//! powered by a local AI dungeon master (llama.cpp).
//!
//! ## Features
//!
//! - **SPECIAL System**: Classic Fallout character attributes (Strength, Perception, Endurance, Charisma, Intelligence, Agility, Luck)
//! - **Turn-Based Combat**: Action points system with strategic combat encounters
//! - **AI Dungeon Master**: Dynamic storytelling powered by local LLM via llama.cpp
//! - **Persistent World**: Worldbook system tracks locations, NPCs, and events
//! - **Character Progression**: Level up, improve skills, and collect wasteland loot
//!
//! ## Module Overview
//!
//! - [`game`]: Core game logic including character management, combat, and world state
//! - [`ai`]: AI dungeon master integration with llama.cpp
//! - [`ui`]: Terminal user interface and display utilities
//! - [`config`]: Configuration management for game and AI settings
//! - [`error`]: Centralized error types and handling
//!
//! ## Quick Start
//!
//! ```no_run
//! use fallout_dnd::game::{GameState, character::Character};
//! use fallout_dnd::ai::AIDungeonMaster;
//! use fallout_dnd::config::LlamaConfig;
//!
//! #[tokio::main]
//! async fn main() -> anyhow::Result<()> {
//!     // Create a character
//!     let character = Character::new("Vault Dweller".to_string());
//!
//!     // Initialize game state
//!     let mut game = GameState::new(character);
//!
//!     // Set up AI dungeon master
//!     let config = LlamaConfig::default();
//!     let dm = AIDungeonMaster::new(config);
//!
//!     // Play the game...
//!     Ok(())
//! }
//! ```

pub mod ai;
pub mod config;
pub mod error;
pub mod game;
pub mod tui;
pub mod ui;
pub mod validation;
