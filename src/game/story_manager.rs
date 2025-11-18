//! # Story Manager
//!
//! Manages the story/conversation context for AI narrative generation.
//!
//! The story manager maintains a sliding window of recent story events
//! to provide context to the AI Dungeon Master for narrative continuity.

use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

/// Maximum number of story entries to keep in context
const MAX_STORY_CONTEXT: usize = 20;

/// Manages the narrative story context for the game
///
/// This struct keeps track of the conversation history between the player
/// and the AI Dungeon Master, maintaining a rolling window of recent events
/// to provide context for narrative generation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoryManager {
    /// FIFO queue of story events (player actions and DM responses)
    context: VecDeque<String>,

    /// Maximum number of entries to keep (default: 20)
    #[serde(default = "default_max_context")]
    max_context: usize,
}

fn default_max_context() -> usize {
    MAX_STORY_CONTEXT
}

impl Default for StoryManager {
    fn default() -> Self {
        Self::new()
    }
}

impl StoryManager {
    /// Create a new StoryManager with default settings
    pub fn new() -> Self {
        StoryManager {
            context: VecDeque::new(),
            max_context: MAX_STORY_CONTEXT,
        }
    }

    /// Create a new StoryManager with custom max context size
    #[allow(dead_code)]
    pub fn with_capacity(max_context: usize) -> Self {
        StoryManager {
            context: VecDeque::with_capacity(max_context),
            max_context,
        }
    }

    /// Add a new entry to the story context
    ///
    /// Automatically removes old entries if the context exceeds max_context.
    /// Uses a FIFO (First In, First Out) strategy.
    ///
    /// # Example
    /// ```
    /// let mut story = StoryManager::new();
    /// story.add("Player: I walk into the tavern".to_string());
    /// story.add("DM: You see a crowded room".to_string());
    /// ```
    pub fn add(&mut self, message: String) {
        self.context.push_back(message);

        // Keep only the most recent entries
        while self.context.len() > self.max_context {
            self.context.pop_front(); // O(1) operation with VecDeque
        }
    }

    /// Get the most recent N story entries
    ///
    /// Returns a slice of the most recent entries, up to `count` items.
    /// If there are fewer than `count` entries, returns all entries.
    ///
    /// # Example
    /// ```
    /// let recent = story.get_recent(10);
    /// ```
    #[allow(dead_code)]
    pub fn get_recent(&self, count: usize) -> Vec<&String> {
        let skip_count = self.context.len().saturating_sub(count);
        self.context.iter().skip(skip_count).collect()
    }

    /// Get all story entries as a slice
    pub fn get_all(&self) -> &VecDeque<String> {
        &self.context
    }

    /// Get the number of entries in the story context
    #[allow(dead_code)]
    pub fn len(&self) -> usize {
        self.context.len()
    }

    /// Check if the story context is empty
    #[allow(dead_code)]
    pub fn is_empty(&self) -> bool {
        self.context.is_empty()
    }

    /// Clear all story context
    #[allow(dead_code)]
    pub fn clear(&mut self) {
        self.context.clear();
    }

    /// Set the maximum context size
    ///
    /// If the new max is smaller than current size, old entries are removed.
    #[allow(dead_code)]
    pub fn set_max_context(&mut self, max: usize) {
        self.max_context = max;

        // Trim if necessary
        while self.context.len() > self.max_context {
            self.context.pop_front();
        }
    }

    /// Get the maximum context size
    #[allow(dead_code)]
    pub fn max_context(&self) -> usize {
        self.max_context
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_story_manager() {
        let story = StoryManager::new();
        assert_eq!(story.len(), 0);
        assert!(story.is_empty());
        assert_eq!(story.max_context(), MAX_STORY_CONTEXT);
    }

    #[test]
    fn test_add_and_get() {
        let mut story = StoryManager::new();
        story.add("Event 1".to_string());
        story.add("Event 2".to_string());

        assert_eq!(story.len(), 2);
        assert!(!story.is_empty());

        let all = story.get_all();
        assert_eq!(all.len(), 2);
        assert_eq!(all[0], "Event 1");
        assert_eq!(all[1], "Event 2");
    }

    #[test]
    fn test_fifo_behavior() {
        let mut story = StoryManager::with_capacity(3);

        story.add("Event 1".to_string());
        story.add("Event 2".to_string());
        story.add("Event 3".to_string());
        story.add("Event 4".to_string()); // Should push out Event 1

        assert_eq!(story.len(), 3);
        let all = story.get_all();
        assert_eq!(all[0], "Event 2");
        assert_eq!(all[1], "Event 3");
        assert_eq!(all[2], "Event 4");
    }

    #[test]
    fn test_get_recent() {
        let mut story = StoryManager::new();
        for i in 1..=5 {
            story.add(format!("Event {}", i));
        }

        let recent = story.get_recent(3);
        assert_eq!(recent.len(), 3);
        assert_eq!(*recent[0], "Event 3");
        assert_eq!(*recent[1], "Event 4");
        assert_eq!(*recent[2], "Event 5");
    }

    #[test]
    fn test_get_recent_more_than_available() {
        let mut story = StoryManager::new();
        story.add("Event 1".to_string());
        story.add("Event 2".to_string());

        let recent = story.get_recent(10);
        assert_eq!(recent.len(), 2);
    }

    #[test]
    fn test_clear() {
        let mut story = StoryManager::new();
        story.add("Event 1".to_string());
        story.add("Event 2".to_string());

        story.clear();

        assert_eq!(story.len(), 0);
        assert!(story.is_empty());
    }

    #[test]
    fn test_set_max_context() {
        let mut story = StoryManager::new();
        for i in 1..=10 {
            story.add(format!("Event {}", i));
        }

        story.set_max_context(5);

        assert_eq!(story.len(), 5);
        assert_eq!(story.max_context(), 5);

        let all = story.get_all();
        assert_eq!(all[0], "Event 6"); // Oldest entries removed
        assert_eq!(all[4], "Event 10");
    }
}
