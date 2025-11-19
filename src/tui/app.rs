use crate::game::GameState;
use crate::tui::animations::AnimationManager;
use crate::tui::theme::LoadingSpinner;
use crate::tui::worldbook_browser::WorldbookBrowser;
use std::collections::VecDeque;

/// Information about player death for game over screen
#[derive(Debug, Clone)]
pub struct DeathInfo {
    pub location: String,
    pub day: u32,
    pub level: u32,
    pub cause: String,
}

/// Main application state for the TUI
pub struct App {
    /// Whether the app should quit
    pub should_quit: bool,

    /// Current input buffer (what the user is typing)
    pub input: String,

    /// Input cursor position
    pub cursor_position: usize,

    /// Game state reference
    pub game_state: GameState,

    /// Message log (AI DM responses, combat messages, etc.)
    pub message_log: VecDeque<LogMessage>,

    /// Maximum number of messages to keep in the log
    max_log_size: usize,

    /// Current view mode (normal, inventory, worldbook, etc.)
    pub view_mode: ViewMode,

    /// Scroll offset for the message log
    pub scroll_offset: usize,

    /// Whether we're waiting for AI response
    pub waiting_for_ai: bool,

    /// Worldbook browser state
    pub worldbook_browser: WorldbookBrowser,

    /// Animation manager for smooth transitions
    pub animation_manager: AnimationManager,

    /// Loading spinner for AI responses
    pub loading_spinner: LoadingSpinner,

    /// Current streaming message being received from AI
    pub streaming_message: Option<String>,

    /// Whether we're currently receiving a streaming response
    pub is_streaming: bool,

    /// Channel receiver for streaming tokens
    pub stream_receiver: Option<tokio::sync::mpsc::Receiver<Result<String, String>>>,

    /// Flicker state for retro CRT effect (toggles randomly)
    pub should_flicker: bool,

    /// Death information for game over screen
    pub death_info: Option<DeathInfo>,
}

#[derive(Debug, Clone)]
pub struct LogMessage {
    pub content: String,
    pub message_type: MessageType,
}

#[derive(Debug, Clone, PartialEq)]
pub enum MessageType {
    DM,     // AI DM narrative
    Player, // Player action echo
    Combat, // Combat message
    System, // System message (saves, errors, etc.)
    Info,   // Info message
    #[allow(dead_code)]
    Success, // Success message
    Error,  // Error message
}

#[derive(Debug, Clone, PartialEq)]
pub enum ViewMode {
    Normal,    // Regular gameplay
    Inventory, // Viewing inventory
    Stats,     // Viewing character stats
    Worldbook, // Viewing worldbook
    Combat,    // In combat
    #[allow(dead_code)]
    GameOver, // Player died - game over screen
}

impl App {
    pub fn new(game_state: GameState) -> Self {
        let mut app = Self {
            should_quit: false,
            input: String::new(),
            cursor_position: 0,
            game_state,
            message_log: VecDeque::new(),
            max_log_size: 100,
            view_mode: ViewMode::Normal,
            scroll_offset: 0,
            waiting_for_ai: false,
            worldbook_browser: WorldbookBrowser::new(),
            animation_manager: AnimationManager::new(),
            loading_spinner: LoadingSpinner::new(),
            streaming_message: None,
            is_streaming: false,
            stream_receiver: None,
            should_flicker: false,
            death_info: None,
        };

        // Add welcome message
        app.add_message(
            "Welcome to Fallout D&D! You are standing at the entrance to Vault 13.".to_string(),
            MessageType::DM,
        );
        app.add_message(
            "Type your actions and press Enter to proceed. Use 'help' for available commands."
                .to_string(),
            MessageType::System,
        );

        app
    }

    /// Add a message to the log
    pub fn add_message(&mut self, content: String, message_type: MessageType) {
        self.message_log.push_back(LogMessage {
            content,
            message_type,
        });

        // Keep log size under control
        while self.message_log.len() > self.max_log_size {
            self.message_log.pop_front();
        }

        // Auto-scroll to bottom when new messages arrive
        self.scroll_offset = 0;
    }

    /// Add a player action echo to the log
    pub fn add_player_action(&mut self, action: &str) {
        self.add_message(format!("> {}", action), MessageType::Player);
    }

    /// Add a DM response to the log
    #[allow(dead_code)]
    pub fn add_dm_response(&mut self, response: String) {
        self.add_message(response, MessageType::DM);
    }

    /// Add a combat message to the log
    pub fn add_combat_message(&mut self, message: String) {
        self.add_message(message, MessageType::Combat);
    }

    /// Add a system message to the log
    pub fn add_system_message(&mut self, message: String) {
        self.add_message(message, MessageType::System);
    }

    /// Add an info message to the log
    pub fn add_info_message(&mut self, message: String) {
        self.add_message(message, MessageType::Info);
    }

    /// Add a success message to the log
    #[allow(dead_code)]
    pub fn add_success_message(&mut self, message: String) {
        self.add_message(message, MessageType::Success);
    }

    /// Add an error message to the log
    pub fn add_error_message(&mut self, message: String) {
        self.add_message(message, MessageType::Error);
    }

    /// Handle character input
    pub fn enter_char(&mut self, c: char) {
        self.input.insert(self.cursor_position, c);
        self.cursor_position += 1;
    }

    /// Handle backspace
    pub fn delete_char(&mut self) {
        if self.cursor_position > 0 {
            self.input.remove(self.cursor_position - 1);
            self.cursor_position -= 1;
        }
    }

    /// Move cursor left
    pub fn move_cursor_left(&mut self) {
        if self.cursor_position > 0 {
            self.cursor_position -= 1;
        }
    }

    /// Move cursor right
    pub fn move_cursor_right(&mut self) {
        if self.cursor_position < self.input.len() {
            self.cursor_position += 1;
        }
    }

    /// Move cursor to start
    pub fn move_cursor_start(&mut self) {
        self.cursor_position = 0;
    }

    /// Move cursor to end
    pub fn move_cursor_end(&mut self) {
        self.cursor_position = self.input.len();
    }

    /// Get the current input and clear it
    pub fn take_input(&mut self) -> String {
        let input = self.input.clone();
        self.input.clear();
        self.cursor_position = 0;
        input
    }

    /// Scroll the message log up
    pub fn scroll_up(&mut self) {
        if self.scroll_offset < self.message_log.len().saturating_sub(1) {
            self.scroll_offset += 1;
        }
    }

    /// Scroll the message log down
    pub fn scroll_down(&mut self) {
        if self.scroll_offset > 0 {
            self.scroll_offset -= 1;
        }
    }

    /// Get visible messages based on scroll offset and available height
    pub fn get_visible_messages(&self, height: usize) -> Vec<&LogMessage> {
        let total_messages = self.message_log.len();
        if total_messages == 0 {
            return vec![];
        }

        // Calculate maximum useful scroll offset
        // We can't scroll back further than (total_messages - height) because
        // that would mean trying to show messages that don't exist
        let max_scroll = total_messages.saturating_sub(height);

        // Clamp scroll_offset to prevent showing incomplete screens
        let effective_offset = self.scroll_offset.min(max_scroll);

        let start_idx = total_messages.saturating_sub(height + effective_offset);
        let end_idx = total_messages.saturating_sub(effective_offset);

        self.message_log.range(start_idx..end_idx).collect()
    }

    /// Set view mode
    pub fn set_view_mode(&mut self, mode: ViewMode) {
        self.view_mode = mode;
    }

    /// Check if in combat
    pub fn is_in_combat(&self) -> bool {
        self.game_state.combat.active
    }

    /// Update view mode based on combat status
    pub fn update_view_mode_for_combat(&mut self) {
        if self.game_state.combat.active && self.view_mode != ViewMode::Combat {
            self.view_mode = ViewMode::Combat;
        } else if !self.game_state.combat.active && self.view_mode == ViewMode::Combat {
            self.view_mode = ViewMode::Normal;
        }
    }

    /// Start a new streaming message with a receiver channel
    pub fn start_streaming(
        &mut self,
        receiver: tokio::sync::mpsc::Receiver<Result<String, String>>,
    ) {
        self.is_streaming = true;
        self.streaming_message = Some(String::new());
        self.stream_receiver = Some(receiver);
        self.scroll_offset = 0; // Auto-scroll to bottom when streaming
    }

    /// Try to receive the next token from the stream (non-blocking)
    /// Returns Some(Ok(token)) for new tokens, Some(Err(error)) for errors, or None when done/empty
    pub fn try_recv_token(&mut self) -> Option<Result<String, String>> {
        if let Some(ref mut rx) = self.stream_receiver {
            match rx.try_recv() {
                Ok(result) => Some(result),
                Err(tokio::sync::mpsc::error::TryRecvError::Empty) => None,
                Err(tokio::sync::mpsc::error::TryRecvError::Disconnected) => {
                    // Stream finished - this will be detected by is_stream_finished()
                    None
                }
            }
        } else {
            None
        }
    }

    /// Check if the stream has finished and clean up if so
    /// Returns Some(response) if streaming finished, None if still streaming
    pub fn check_stream_finished(&mut self) -> Option<String> {
        if let Some(ref mut rx) = self.stream_receiver {
            if rx.is_closed() {
                return self.finish_streaming();
            }
        }
        None
    }

    /// Append a token to the current streaming message
    pub fn append_streaming_token(&mut self, token: String) {
        if let Some(ref mut msg) = self.streaming_message {
            msg.push_str(&token);
        }
    }

    /// Finish the current streaming message and add it to the log
    /// Returns the completed message content
    pub fn finish_streaming(&mut self) -> Option<String> {
        self.is_streaming = false;
        self.stream_receiver = None;
        if let Some(content) = self.streaming_message.take() {
            if !content.is_empty() {
                self.add_message(content.clone(), MessageType::DM);
                // Add DM response to both conversation systems for continuity
                self.game_state.conversation.add_dm_turn(content.clone());
                self.game_state.story.add(format!("DM: {}", content)); // Legacy support
                return Some(content);
            }
        }
        None
    }

    /// Cancel the current streaming message
    pub fn cancel_streaming(&mut self) {
        self.is_streaming = false;
        self.streaming_message = None;
        self.stream_receiver = None;
    }

    /// Update flicker state for retro CRT effect
    /// Call this on each tick to create subtle text flicker
    pub fn update_flicker(&mut self) {
        use crate::tui::theme::RetroEffects;
        // Very low intensity (1-2% chance per tick) for subtle effect
        self.should_flicker = RetroEffects::should_flicker(0.015);
    }

    /// Trigger game over when player dies
    #[allow(dead_code)]
    pub fn trigger_game_over(&mut self, cause: String) {
        self.death_info = Some(DeathInfo {
            location: self.game_state.location.clone(),
            day: self.game_state.day,
            level: self.game_state.character.level,
            cause,
        });
        self.view_mode = ViewMode::GameOver;
        self.waiting_for_ai = false;
        self.cancel_streaming();
    }

    /// Reset the game to start a new playthrough
    #[allow(dead_code)]
    pub fn restart_game(&mut self) {
        // Create a new character with the same name and SPECIAL stats
        let character_name = self.game_state.character.name.to_string();
        let special = self.game_state.character.special.clone();
        let character = crate::game::character::Character::new(character_name, special);

        // Create new game state
        self.game_state = GameState::new(character);

        // Reset UI state
        self.message_log.clear();
        self.view_mode = ViewMode::Normal;
        self.death_info = None;
        self.input.clear();
        self.cursor_position = 0;
        self.scroll_offset = 0;
        self.waiting_for_ai = false;
        self.cancel_streaming();

        // Add welcome message
        self.add_message(
            "Welcome to Fallout D&D! You are standing at the entrance to Vault 13.".to_string(),
            MessageType::DM,
        );
        self.add_message(
            "Type your actions and press Enter to proceed. Use 'help' for available commands."
                .to_string(),
            MessageType::System,
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::{character::Special, GameState};

    fn create_test_app() -> App {
        let special = Special {
            strength: 5,
            perception: 5,
            endurance: 5,
            charisma: 5,
            intelligence: 5,
            agility: 5,
            luck: 5,
        };
        let character =
            crate::game::character::Character::new("Test Character".to_string(), special);
        let game_state = GameState::new(character);
        App::new(game_state)
    }

    #[test]
    fn test_add_message() {
        let mut app = create_test_app();
        app.add_message("Test message".to_string(), MessageType::System);
        assert_eq!(app.message_log.len(), 3); // 2 welcome + 1 new
        let last_message = app.message_log.back().unwrap();
        assert_eq!(last_message.content, "Test message");
        assert_eq!(last_message.message_type, MessageType::System);
    }

    #[test]
    fn test_message_log_max_size() {
        let mut app = create_test_app();
        for i in 0..105 {
            app.add_message(format!("Message {}", i), MessageType::Info);
        }
        assert_eq!(app.message_log.len(), 100);
        assert_eq!(app.message_log.front().unwrap().content, "Message 5");
        assert_eq!(app.message_log.back().unwrap().content, "Message 104");
    }

    #[test]
    fn test_input_handling() {
        let mut app = create_test_app();
        app.enter_char('a');
        app.enter_char('b');
        app.enter_char('c');
        assert_eq!(app.input, "abc");
        assert_eq!(app.cursor_position, 3);

        app.move_cursor_left();
        assert_eq!(app.cursor_position, 2);

        app.delete_char();
        assert_eq!(app.input, "ac");
        assert_eq!(app.cursor_position, 1);

        let input = app.take_input();
        assert_eq!(input, "ac");
        assert_eq!(app.input, "");
        assert_eq!(app.cursor_position, 0);
    }

    #[test]
    fn test_scrolling() {
        let mut app = create_test_app();
        for i in 0..20 {
            app.add_message(format!("Message {}", i), MessageType::DM);
        }

        app.scroll_up();
        assert_eq!(app.scroll_offset, 1);

        app.scroll_down();
        assert_eq!(app.scroll_offset, 0);

        // Cannot scroll below 0
        app.scroll_down();
        assert_eq!(app.scroll_offset, 0);

        // Cannot scroll past the last message
        for _ in 0..30 {
            app.scroll_up();
        }
        assert_eq!(app.scroll_offset, 21);
    }

    #[test]
    fn test_view_mode() {
        let mut app = create_test_app();
        assert_eq!(app.view_mode, ViewMode::Normal);

        app.set_view_mode(ViewMode::Inventory);
        assert_eq!(app.view_mode, ViewMode::Inventory);

        app.game_state.combat.active = true;
        app.update_view_mode_for_combat();
        assert_eq!(app.view_mode, ViewMode::Combat);

        app.game_state.combat.active = false;
        app.update_view_mode_for_combat();
        assert_eq!(app.view_mode, ViewMode::Normal);
    }

    #[test]
    fn test_get_visible_messages() {
        let mut app = create_test_app();
        app.message_log.clear(); // Clear welcome messages
        for i in 0..50 {
            app.add_message(format!("Message {}", i), MessageType::System);
        }

        // Test with full height
        let visible = app.get_visible_messages(20);
        assert_eq!(visible.len(), 20);
        assert_eq!(visible[0].content, "Message 30");

        // Test with scrolling
        app.scroll_offset = 10;
        let visible_scrolled = app.get_visible_messages(20);
        assert_eq!(visible_scrolled.len(), 20);
        assert_eq!(visible_scrolled[0].content, "Message 20");
    }

    #[test]
    fn test_app_creation() {
        let app = create_test_app();
        assert!(!app.should_quit);
        assert_eq!(app.view_mode, ViewMode::Normal);
        assert_eq!(app.message_log.len(), 2); // Welcome messages
    }
}
