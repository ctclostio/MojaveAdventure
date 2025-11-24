//! Game handlers - re-exports for main entry points
//!
//! The actual game loop runs in TUI mode via `tui_game_loop.rs`.
//! This module provides re-exports for character creation and game loading.

// Re-export for main.rs
pub use super::char_handlers::create_new_character;
pub use super::persistence::load_game;
