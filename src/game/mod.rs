//! # Game Module
//!
//! Core game logic including character management, combat, and world state.
//!
//! ## Key Components
//!
//! - [`GameState`]: Main game state container that holds all game data
//! - [`character`]: Character creation, attributes (SPECIAL), and skill management
//! - [`combat`]: Turn-based combat system with action points
//! - [`worldbook`]: Persistent world knowledge tracking locations, NPCs, and events
//! - [`story_manager`]: Narrative context management for AI conversations
//! - [`items`]: Item system and inventory management
//! - [`rolls`]: Dice rolling mechanics for skill checks
//!
//! ## Game State Management
//!
//! The [`GameState`] struct is the central hub for all game data. It includes:
//! - Player character with SPECIAL stats and skills
//! - Combat state for managing encounters
//! - Story manager for AI conversation history
//! - Location and quest tracking
//! - Worldbook for persistent world knowledge
//!
//! ## Save/Load System
//!
//! Games can be saved and loaded as JSON files in the `saves/` directory.
//! All save operations are handled by the [`persistence`] module with comprehensive
//! security checks to prevent path traversal attacks.

pub mod char_handlers;
pub mod character;
pub mod combat;
pub mod conversation;
pub mod handlers;
pub mod items;
pub mod persistence;
pub mod rolls;
pub mod stat_allocator;
pub mod story_manager;
pub mod tui_game_loop;
pub mod worldbook;

use character::Character;
use combat::CombatState;
use conversation::ConversationManager;
use serde::{Deserialize, Serialize};
use story_manager::StoryManager;
use worldbook::Worldbook;

/// Default value for day counter (used for backward compatibility with old saves)
fn default_day() -> u32 {
    1
}

/// Default conversation manager (migrates from old story context for backward compatibility)
fn default_conversation() -> ConversationManager {
    ConversationManager::new()
}

/// Main game state container that holds all game data.
///
/// `GameState` is the central structure that manages:
/// - Player character progression and stats
/// - Combat encounters
/// - Conversation context for AI (via ConversationManager)
/// - Legacy story context (via StoryManager, kept for backward compatibility)
/// - Current location and quest progress
/// - Worldbook knowledge base
///
/// # Example
///
/// ```no_run
/// use fallout_dnd::game::{GameState, character::{Character, Special}, persistence};
///
/// let special = Special::new();
/// let character = Character::new("Vault Dweller".to_string(), special);
/// let mut game = GameState::new(character);
///
/// // Add conversation turns
/// game.conversation.add_player_turn("I enter the wasteland".to_string());
/// game.conversation.add_dm_turn("You see ruins ahead".to_string());
///
/// // Save the game
/// persistence::save_to_file(&game, "my_save").expect("Failed to save game");
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameState {
    /// The player's character with SPECIAL stats, skills, and inventory
    pub character: Character,

    /// Current combat state (active encounters, enemies, round counter)
    pub combat: CombatState,

    /// Structured conversation context manager for AI narrative (NEW: preferred method)
    #[serde(default = "default_conversation")]
    pub conversation: ConversationManager,

    /// Legacy story context manager (kept for backward compatibility)
    /// New code should use `conversation` instead
    pub story: StoryManager,

    /// Current location in the wasteland
    pub location: String,

    /// Active quests and objectives
    pub quest_log: Vec<String>,

    /// Persistent world knowledge (locations, NPCs, events)
    pub worldbook: Worldbook,

    /// Current day in the wasteland (starts at 1)
    #[serde(default = "default_day")]
    pub day: u32,
}

impl GameState {
    /// Creates a new game state with the given character.
    ///
    /// Initializes the game with:
    /// - Starting location: Vault 13 Entrance
    /// - Starting quest: Find the Water Chip
    /// - Empty combat state
    /// - Fresh worldbook
    /// - Empty story context
    ///
    /// # Arguments
    ///
    /// * `character` - The player's character to use for this game
    ///
    /// # Example
    ///
    /// ```no_run
    /// use fallout_dnd::game::{GameState, character::{Character, Special}};
    ///
    /// let special = Special::new();
    /// let character = Character::new("Lone Wanderer".to_string(), special);
    /// let game = GameState::new(character);
    /// assert_eq!(game.location, "Vault 13 Entrance");
    /// ```
    pub fn new(character: Character) -> Self {
        let mut worldbook = Worldbook::with_defaults();
        worldbook.set_current_location(Some("vault_13".into()));

        GameState {
            character,
            combat: CombatState::new(),
            conversation: ConversationManager::new(),
            story: StoryManager::new(),
            location: "Vault 13 Entrance".to_string(),
            quest_log: vec!["Find the Water Chip".to_string()],
            worldbook,
            day: 1,
        }
    }

    /// Migrate legacy story context to new conversation system
    ///
    /// This should be called once when loading old save files to convert
    /// the string-based story context to structured conversation turns.
    pub fn migrate_story_to_conversation(&mut self) {
        if self.conversation.is_empty() && !self.story.is_empty() {
            self.conversation =
                ConversationManager::from_legacy_story_context(self.story.get_all());
        }
    }
}
