//! # Conversation Manager
//!
//! Manages structured conversation history between the player and AI Dungeon Master.
//!
//! This module provides a more robust alternative to simple string-based story tracking,
//! with clear speaker attribution and better context management for AI prompts.

use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

/// Maximum number of conversation turns to keep in context
const MAX_CONVERSATION_TURNS: usize = 20;

/// Identifies who is speaking in a conversation turn
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Speaker {
    /// The AI Dungeon Master (narration, NPCs, world responses)
    DM,
    /// The player character
    Player,
}

/// A single turn in the conversation between player and DM
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversationTurn {
    /// Who is speaking
    pub speaker: Speaker,
    /// What was said/done
    pub message: String,
    /// Optional turn number for tracking (0 = first turn)
    #[serde(default)]
    pub turn_number: u32,
}

impl ConversationTurn {
    /// Create a new conversation turn
    pub fn new(speaker: Speaker, message: String, turn_number: u32) -> Self {
        ConversationTurn {
            speaker,
            message,
            turn_number,
        }
    }

    /// Format this turn as a string for display or prompts
    pub fn format(&self) -> String {
        match self.speaker {
            Speaker::Player => format!("Player: {}", self.message),
            Speaker::DM => format!("DM: {}", self.message),
        }
    }

    /// Format this turn with a clear visual separator for prompts
    pub fn format_for_prompt(&self) -> String {
        match self.speaker {
            Speaker::Player => format!(">>> PLAYER: {}", self.message),
            Speaker::DM => format!(">>> DM (YOU): {}", self.message),
        }
    }
}

/// Manages the conversation history for the game
///
/// This struct maintains a structured record of all interactions between
/// the player and the AI Dungeon Master, providing better context management
/// than simple string-based tracking.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversationManager {
    /// FIFO queue of conversation turns
    turns: VecDeque<ConversationTurn>,

    /// Maximum number of turns to keep (default: 20)
    #[serde(default = "default_max_turns")]
    max_turns: usize,

    /// Current turn number (increments with each turn)
    #[serde(default)]
    current_turn: u32,
}

fn default_max_turns() -> usize {
    MAX_CONVERSATION_TURNS
}

impl Default for ConversationManager {
    fn default() -> Self {
        Self::new()
    }
}

impl ConversationManager {
    /// Create a new ConversationManager with default settings
    pub fn new() -> Self {
        ConversationManager {
            turns: VecDeque::new(),
            max_turns: MAX_CONVERSATION_TURNS,
            current_turn: 0,
        }
    }

    /// Create a new ConversationManager with custom max turns
    #[allow(dead_code)]
    pub fn with_capacity(max_turns: usize) -> Self {
        ConversationManager {
            turns: VecDeque::with_capacity(max_turns),
            max_turns,
            current_turn: 0,
        }
    }

    /// Add a player's action/speech to the conversation
    pub fn add_player_turn(&mut self, message: String) {
        let turn = ConversationTurn::new(Speaker::Player, message, self.current_turn);
        self.add_turn(turn);
        self.current_turn += 1;
    }

    /// Add a DM response to the conversation
    pub fn add_dm_turn(&mut self, message: String) {
        let turn = ConversationTurn::new(Speaker::DM, message, self.current_turn);
        self.add_turn(turn);
        self.current_turn += 1;
    }

    /// Add a raw conversation turn
    fn add_turn(&mut self, turn: ConversationTurn) {
        self.turns.push_back(turn);

        // Keep only the most recent turns
        while self.turns.len() > self.max_turns {
            self.turns.pop_front();
        }
    }

    /// Get the most recent N turns
    pub fn get_recent_turns(&self, count: usize) -> Vec<&ConversationTurn> {
        let skip_count = self.turns.len().saturating_sub(count);
        self.turns.iter().skip(skip_count).collect()
    }

    /// Get all turns as a slice
    #[allow(dead_code)]
    pub fn get_all_turns(&self) -> &VecDeque<ConversationTurn> {
        &self.turns
    }

    /// Get the number of turns in the conversation
    pub fn len(&self) -> usize {
        self.turns.len()
    }

    /// Check if the conversation is empty
    pub fn is_empty(&self) -> bool {
        self.turns.is_empty()
    }

    /// Clear all conversation history
    #[allow(dead_code)]
    pub fn clear(&mut self) {
        self.turns.clear();
        self.current_turn = 0;
    }

    /// Remove the last DM turn from conversation
    /// Returns true if a DM turn was removed, false otherwise
    pub fn remove_last_dm_turn(&mut self) -> bool {
        // Find the last DM turn by iterating backwards
        if let Some(pos) = self
            .turns
            .iter()
            .rposition(|turn| turn.speaker == Speaker::DM)
        {
            self.turns.remove(pos);
            return true;
        }
        false
    }

    /// Replace the last DM turn with a new message
    /// Returns true if a turn was replaced, false otherwise
    pub fn replace_last_dm_turn(&mut self, new_message: String) -> bool {
        if self.remove_last_dm_turn() {
            self.add_dm_turn(new_message);
            true
        } else {
            false
        }
    }

    /// Build a prompt section from recent conversation history
    ///
    /// This creates a well-formatted section for AI prompts that clearly
    /// identifies who said what, with emphasis on the DM being "YOU".
    pub fn build_prompt_section(&self, include_last_n: usize) -> String {
        if self.turns.is_empty() {
            return String::new();
        }

        let recent_turns = self.get_recent_turns(include_last_n);
        if recent_turns.is_empty() {
            return String::new();
        }

        // Pre-allocate: ~100 bytes per turn + headers
        let mut section = String::with_capacity(200 + recent_turns.len() * 100);

        section.push_str("=== CONVERSATION HISTORY ===\n");
        section.push_str("(You are the DM. The player is the other speaker.)\n");
        section.push_str("(>>> marks turn boundaries for clarity)\n\n");

        for turn in recent_turns {
            section.push_str(&turn.format_for_prompt());
            section.push('\n');
        }

        section.push_str("\n=== END HISTORY ===\n\n");
        section
    }

    /// Build a simple prompt section (backward compatible with old system)
    #[allow(dead_code)]
    pub fn build_simple_prompt_section(&self, include_last_n: usize) -> String {
        if self.turns.is_empty() {
            return String::new();
        }

        let recent_turns = self.get_recent_turns(include_last_n);
        if recent_turns.is_empty() {
            return String::new();
        }

        let mut section = String::with_capacity(50 + recent_turns.len() * 100);
        section.push_str("Recent events:\n");

        for turn in recent_turns {
            section.push_str(&turn.format());
            section.push('\n');
        }

        section.push('\n');
        section
    }

    /// Get conversation summary for debugging
    #[allow(dead_code)]
    pub fn get_summary(&self) -> String {
        format!(
            "Conversation: {} turns, current turn: {}",
            self.turns.len(),
            self.current_turn
        )
    }

    /// Set the maximum number of turns to keep
    #[allow(dead_code)]
    pub fn set_max_turns(&mut self, max: usize) {
        self.max_turns = max;

        // Trim if necessary
        while self.turns.len() > self.max_turns {
            self.turns.pop_front();
        }
    }

    /// Get the maximum number of turns
    pub fn max_turns(&self) -> usize {
        self.max_turns
    }
}

// ==================== MIGRATION HELPERS ====================

impl ConversationManager {
    /// Create a ConversationManager from old-style story context
    ///
    /// This helps migrate from the old string-based system to the new structured system.
    /// Parses strings like "Player: message" and "DM: message".
    pub fn from_legacy_story_context(story_context: &VecDeque<String>) -> Self {
        let mut manager = ConversationManager::new();

        for (idx, msg) in story_context.iter().enumerate() {
            if let Some(player_msg) = msg.strip_prefix("Player: ") {
                manager.turns.push_back(ConversationTurn::new(
                    Speaker::Player,
                    player_msg.to_string(),
                    idx as u32,
                ));
            } else if let Some(dm_msg) = msg.strip_prefix("DM: ") {
                manager.turns.push_back(ConversationTurn::new(
                    Speaker::DM,
                    dm_msg.to_string(),
                    idx as u32,
                ));
            } else {
                // If no prefix, assume it's a DM message
                manager.turns.push_back(ConversationTurn::new(
                    Speaker::DM,
                    msg.clone(),
                    idx as u32,
                ));
            }
        }

        manager.current_turn = story_context.len() as u32;
        manager
    }

    /// Export to legacy format for backward compatibility
    #[allow(dead_code)]
    pub fn to_legacy_story_context(&self) -> VecDeque<String> {
        self.turns.iter().map(|turn| turn.format()).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_conversation() {
        let conv = ConversationManager::new();
        assert_eq!(conv.len(), 0);
        assert!(conv.is_empty());
        assert_eq!(conv.max_turns(), MAX_CONVERSATION_TURNS);
    }

    #[test]
    fn test_add_turns() {
        let mut conv = ConversationManager::new();
        conv.add_player_turn("I explore the vault".to_string());
        conv.add_dm_turn("You see rusted machinery".to_string());

        assert_eq!(conv.len(), 2);
        assert!(!conv.is_empty());

        let turns = conv.get_all_turns();
        assert_eq!(turns[0].speaker, Speaker::Player);
        assert_eq!(turns[1].speaker, Speaker::DM);
    }

    #[test]
    fn test_fifo_behavior() {
        let mut conv = ConversationManager::with_capacity(3);

        conv.add_player_turn("Turn 1".to_string());
        conv.add_dm_turn("Turn 2".to_string());
        conv.add_player_turn("Turn 3".to_string());
        conv.add_dm_turn("Turn 4".to_string()); // Should push out Turn 1

        assert_eq!(conv.len(), 3);
        let turns = conv.get_all_turns();
        assert_eq!(turns[0].message, "Turn 2");
        assert_eq!(turns[2].message, "Turn 4");
    }

    #[test]
    fn test_get_recent_turns() {
        let mut conv = ConversationManager::new();
        for i in 1..=5 {
            conv.add_player_turn(format!("Turn {}", i));
        }

        let recent = conv.get_recent_turns(3);
        assert_eq!(recent.len(), 3);
        assert_eq!(recent[0].message, "Turn 3");
        assert_eq!(recent[2].message, "Turn 5");
    }

    #[test]
    fn test_build_prompt_section() {
        let mut conv = ConversationManager::new();
        conv.add_player_turn("I enter the vault".to_string());
        conv.add_dm_turn("You see darkness ahead".to_string());

        let prompt = conv.build_prompt_section(10);
        assert!(prompt.contains(">>> PLAYER: I enter the vault"));
        assert!(prompt.contains(">>> DM (YOU): You see darkness ahead"));
        assert!(prompt.contains("=== CONVERSATION HISTORY ==="));
    }

    #[test]
    fn test_legacy_migration() {
        let mut old_story = VecDeque::new();
        old_story.push_back("Player: I explore".to_string());
        old_story.push_back("DM: You find a door".to_string());

        let conv = ConversationManager::from_legacy_story_context(&old_story);
        assert_eq!(conv.len(), 2);
        assert_eq!(conv.turns[0].speaker, Speaker::Player);
        assert_eq!(conv.turns[1].speaker, Speaker::DM);
        assert_eq!(conv.turns[0].message, "I explore");
        assert_eq!(conv.turns[1].message, "You find a door");
    }

    #[test]
    fn test_turn_formatting() {
        let turn = ConversationTurn::new(Speaker::Player, "Test message".to_string(), 1);
        assert_eq!(turn.format(), "Player: Test message");
        assert_eq!(turn.format_for_prompt(), ">>> PLAYER: Test message");
    }

    #[test]
    fn test_clear() {
        let mut conv = ConversationManager::new();
        conv.add_player_turn("Test".to_string());
        conv.add_dm_turn("Response".to_string());

        conv.clear();

        assert_eq!(conv.len(), 0);
        assert!(conv.is_empty());
        assert_eq!(conv.current_turn, 0);
    }
}
