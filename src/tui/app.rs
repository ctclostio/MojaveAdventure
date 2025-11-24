use crate::ai::extractor::ExtractedEntities;
use crate::game::GameState;
use crate::tui::animations::AnimationManager;
use crate::tui::theme::LoadingSpinner;
use crate::tui::worldbook_browser::WorldbookBrowser;
use std::collections::VecDeque;
use std::time::{SystemTime, UNIX_EPOCH};

/// Worldbook update message from background extraction
pub type WorldbookUpdate = (ExtractedEntities, String);

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

    /// Current streaming message being received from AI (full, including thinking)
    pub streaming_message: Option<String>,

    /// Filtered streaming message for display (thinking tokens removed)
    pub filtered_streaming_message: Option<String>,

    /// Line buffer for detecting thinking tokens during streaming
    thinking_line_buffer: String,

    /// Whether we're currently in a thinking block (line started with thinking indicator)
    in_thinking_mode: bool,

    /// Whether we're currently receiving a streaming response
    pub is_streaming: bool,

    /// Channel receiver for streaming tokens
    pub stream_receiver: Option<tokio::sync::mpsc::Receiver<Result<String, String>>>,

    /// Flicker state for retro CRT effect (toggles randomly)
    pub should_flicker: bool,

    /// Death information for game over screen
    pub death_info: Option<DeathInfo>,

    /// Last autosave timestamp (in seconds since UNIX_EPOCH)
    pub last_autosave_time: u64,

    /// Equipment menu state - selected item index
    pub equipment_selected_index: usize,

    /// Command history
    pub command_history: Vec<String>,

    /// Current position in command history (index)
    /// When equal to command_history.len(), we are at the "new" line
    pub history_index: usize,

    /// Channel sender for worldbook updates from background extraction
    pub worldbook_update_sender: tokio::sync::mpsc::Sender<WorldbookUpdate>,

    /// Channel receiver for worldbook updates from background extraction
    pub worldbook_update_receiver: tokio::sync::mpsc::Receiver<WorldbookUpdate>,
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
    Equipment, // Equipment menu for equipping/unequipping items
    #[allow(dead_code)]
    GameOver, // Player died - game over screen
}

impl App {
    pub fn new(game_state: GameState) -> Self {
        // Create channel for worldbook updates from background extraction
        let (worldbook_tx, worldbook_rx) = tokio::sync::mpsc::channel::<WorldbookUpdate>(16);

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
            filtered_streaming_message: None,
            thinking_line_buffer: String::new(),
            in_thinking_mode: false,
            is_streaming: false,
            stream_receiver: None,
            should_flicker: false,
            death_info: None,
            last_autosave_time: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            equipment_selected_index: 0,
            command_history: Vec::new(),
            history_index: 0,
            worldbook_update_sender: worldbook_tx,
            worldbook_update_receiver: worldbook_rx,
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
        self.filtered_streaming_message = Some(String::new());
        self.thinking_line_buffer.clear();
        self.in_thinking_mode = false;
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

    /// Append a token to the current streaming message with thinking-token filtering
    /// GPT-OSS-20B uses the "harmony format" with channel markers for thinking vs final response
    pub fn append_streaming_token(&mut self, token: String) {
        // Always store full message (including thinking) for extraction/debugging
        if let Some(ref mut msg) = self.streaming_message {
            msg.push_str(&token);
        }

        // Process token for filtered display
        self.thinking_line_buffer.push_str(&token);

        // Check if we've hit an end-of-thinking marker - this starts the actual response
        // Includes both OpenAI </think> tags and GPT-OSS harmony format markers
        let final_markers = [
            "</think>",
            "<|channel|>final<|message|>",
            "<|final|><|message|>",
        ];
        for marker in &final_markers {
            if let Some(pos) = self.thinking_line_buffer.find(marker) {
                // Found final marker - switch out of thinking mode and extract content after it
                self.in_thinking_mode = false;
                let after_marker = self.thinking_line_buffer[pos + marker.len()..].to_string();
                self.thinking_line_buffer = after_marker;

                // Add any content after the marker to filtered output
                if !self.thinking_line_buffer.is_empty() {
                    let cleaned = Self::strip_channel_markers(&self.thinking_line_buffer);
                    if !cleaned.is_empty() {
                        if let Some(ref mut filtered) = self.filtered_streaming_message {
                            filtered.push_str(&cleaned);
                        }
                    }
                    self.thinking_line_buffer.clear();
                }
                return;
            }
        }

        // Check if we're in thinking mode (analysis channel or emoji indicators)
        if Self::is_thinking_indicator(&self.thinking_line_buffer) {
            self.in_thinking_mode = true;
        }

        // If we're in thinking mode, don't add to filtered output
        if self.in_thinking_mode {
            // Keep buffering but don't display
            return;
        }

        // Process complete lines from the buffer for non-harmony format responses
        while let Some(newline_pos) = self.thinking_line_buffer.find('\n') {
            let line = self.thinking_line_buffer[..newline_pos].to_string();
            self.thinking_line_buffer = self.thinking_line_buffer[newline_pos + 1..].to_string();

            // Check if this line is a thinking line
            let trimmed = line.trim_start();
            let is_thinking = Self::is_thinking_indicator(trimmed);

            if is_thinking {
                self.in_thinking_mode = true;
                tracing::trace!("Filtered thinking line: {}", line);
            } else if !line.trim().is_empty() {
                self.in_thinking_mode = false;
                // Strip channel markers and add to filtered output
                let cleaned = Self::strip_channel_markers(&line);
                if !cleaned.is_empty() {
                    if let Some(ref mut filtered) = self.filtered_streaming_message {
                        if !filtered.is_empty() {
                            filtered.push('\n');
                        }
                        filtered.push_str(&cleaned);
                    }
                }
            }
        }
    }

    /// Check if a line is a thinking/reasoning line from GPT-OSS
    /// GPT-OSS uses the "harmony format" with channel markers, emoji indicators, and <think> tags
    fn is_thinking_indicator(line: &str) -> bool {
        // OpenAI/GPT-OSS thinking tags
        if line.contains("<think>") || line.starts_with("<think") {
            return true;
        }

        // GPT-OSS harmony format channel markers for analysis/thinking
        let harmony_thinking_markers = [
            "<|channel|>analysis",
            "<|analysis|>",
            "<|start|>assistant<|channel|>analysis",
        ];

        // Check for harmony format thinking markers anywhere in line
        for marker in &harmony_thinking_markers {
            if line.contains(marker) {
                return true;
            }
        }

        // GPT-OSS uses these emojis for chain-of-thought reasoning
        let thinking_prefixes = [
            "🤔", // Thinking face
            "💭", // Thought balloon
        ];

        for prefix in &thinking_prefixes {
            if line.starts_with(prefix) {
                return true;
            }
        }

        false
    }

    /// Strip harmony format channel markers and thinking tags from content
    fn strip_channel_markers(content: &str) -> String {
        let markers_to_strip = [
            "<|end|>",
            "<|start|>",
            "<|assistant|>",
            "<|channel|>",
            "<|analysis|>",
            "<|final|>",
            "<|message|>",
            "<|user|>",
            "<|system|>",
            "analysis>",
            "final>",
            "<|commentary|>",
            "<think>",
            "</think>",
        ];

        let mut result = content.to_string();
        for marker in &markers_to_strip {
            result = result.replace(marker, "");
        }

        // Clean up multiple spaces and trim
        result
            .split_whitespace()
            .collect::<Vec<&str>>()
            .join(" ")
            .trim()
            .to_string()
    }

    /// Finish the current streaming message and add it to the log
    /// Returns the completed message content (filtered, without thinking tokens)
    pub fn finish_streaming(&mut self) -> Option<String> {
        self.is_streaming = false;
        self.stream_receiver = None;

        // Process any remaining content in the line buffer
        if !self.thinking_line_buffer.is_empty() {
            let remaining = std::mem::take(&mut self.thinking_line_buffer);
            let trimmed = remaining.trim();
            if !trimmed.is_empty() && !Self::is_thinking_indicator(trimmed) {
                if let Some(ref mut filtered) = self.filtered_streaming_message {
                    if !filtered.is_empty() {
                        filtered.push('\n');
                    }
                    filtered.push_str(trimmed);
                }
            }
        }

        // Clean up thinking state
        self.in_thinking_mode = false;

        // Use filtered message for display (without thinking tokens)
        // Keep full message in streaming_message for extraction/debugging
        let _full_content = self.streaming_message.take(); // Keep for potential debugging

        if let Some(content) = self.filtered_streaming_message.take() {
            if !content.is_empty() {
                // Strip any stop tokens that may have been included in the response
                let cleaned_content = Self::strip_stop_tokens(&content);
                // Also do a final pass to strip any thinking content that made it through
                let final_content = Self::strip_thinking_content(&cleaned_content);
                if !final_content.is_empty() {
                    self.add_message(final_content.clone(), MessageType::DM);
                    // Add DM response to both conversation systems for continuity
                    self.game_state
                        .conversation
                        .add_dm_turn(final_content.clone());
                    self.game_state.story.add(format!("DM: {}", final_content)); // Legacy support
                    return Some(final_content);
                }
            }
        }
        None
    }

    /// Strip thinking content from response (final cleanup pass)
    /// Extracts only the final response from GPT-OSS harmony format or <think> tags
    fn strip_thinking_content(content: &str) -> String {
        // GPT-OSS harmony format: everything is on one line with channel markers
        // Format: thinking...<|channel|>analysis<|message|>...<|channel|>final<|message|>actual response
        // Or OpenAI format: <think>thinking...</think>actual response
        // We need to extract content after the "final" channel marker or </think> tag

        let mut result = content.to_string();

        // First check for </think> tag (OpenAI thinking format)
        if let Some(pos) = result.find("</think>") {
            // Extract everything after </think>
            result = result[pos + "</think>".len()..].to_string();
        } else {
            // Look for the final channel marker and extract content after it
            let final_markers = [
                "<|channel|>final<|message|>",
                "<|final|><|message|>",
                "<|channel|>final>",
            ];

            for marker in &final_markers {
                if let Some(pos) = result.find(marker) {
                    // Extract everything after the final marker
                    result = result[pos + marker.len()..].to_string();
                    break;
                }
            }
        }

        // If no markers found, try to filter by lines
        if result == content {
            result = content
                .lines()
                .filter(|line| {
                    let trimmed = line.trim();
                    !Self::is_thinking_indicator(trimmed)
                })
                .collect::<Vec<&str>>()
                .join("\n");
        }

        // Strip any remaining channel markers
        Self::strip_channel_markers(&result)
    }

    /// Strip stop tokens from the AI response
    /// These are tokens that should terminate generation but may be partially included
    fn strip_stop_tokens(content: &str) -> String {
        let stop_tokens = [
            ">>> PLAYER:",
            ">>> PLAYER",
            "\n>>> PLAYER:",
            "\n>>> PLAYER",
            "Player:",
            "\nPlayer:",
        ];

        let mut result = content.to_string();
        for token in &stop_tokens {
            if let Some(pos) = result.find(token) {
                result = result[..pos].to_string();
            }
        }
        result.trim().to_string()
    }

    /// Process any pending worldbook updates from background extraction
    /// Call this in the tick event to integrate extracted entities
    pub fn process_worldbook_updates(&mut self) {
        // Try to receive worldbook update without blocking
        while let Ok((entities, summary)) = self.worldbook_update_receiver.try_recv() {
            // Show brief status message
            self.add_info_message(format!("[Worldbook: {}]", summary));

            // Convert extracted entities to worldbook entries
            let (locations, npcs, events) = entities.to_worldbook_entries();
            let mut saved_count = 0;

            // Integrate locations
            for location in locations {
                let loc_id = location.id.clone();
                if self.game_state.worldbook.get_location(&loc_id).is_none() {
                    self.game_state.worldbook.add_location(location);
                    saved_count += 1;

                    if self.game_state.worldbook.current_location.is_none() {
                        self.game_state
                            .worldbook
                            .set_current_location(Some(loc_id.clone()));
                        self.game_state.worldbook.visit_location(&loc_id);
                    }
                }
            }

            // Integrate NPCs
            for npc in npcs {
                if self.game_state.worldbook.get_npc(&npc.id).is_none() {
                    self.game_state.worldbook.add_npc(npc);
                    saved_count += 1;
                }
            }

            // Always add events
            for event in events {
                self.game_state.worldbook.add_event(event);
                saved_count += 1;
            }

            tracing::info!("Worldbook updated: {} new entries saved", saved_count);
        }
    }

    /// Cancel the current streaming message
    pub fn cancel_streaming(&mut self) {
        self.is_streaming = false;
        self.streaming_message = None;
        self.filtered_streaming_message = None;
        self.thinking_line_buffer.clear();
        self.in_thinking_mode = false;
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
        self.equipment_selected_index = 0;

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

    /// Check if autosave should occur and perform it
    /// Returns true if autosave was performed
    pub fn check_and_perform_autosave(&mut self, autosave_interval_minutes: u32) -> bool {
        if autosave_interval_minutes == 0 {
            return false;
        }

        let current_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        let elapsed_seconds = current_time.saturating_sub(self.last_autosave_time);
        let interval_seconds = (autosave_interval_minutes as u64) * 60;

        if elapsed_seconds >= interval_seconds {
            if let Err(e) = crate::game::persistence::save_to_file(&self.game_state, "autosave") {
                self.add_system_message(format!("[Autosave failed: {}]", e));
                return false;
            }
            self.last_autosave_time = current_time;
            self.add_system_message("[Game autosaved]".to_string());
            return true;
        }

        false
    }

    /// Perform a manual save
    /// Returns true if save was successful
    pub fn perform_save(&mut self, save_name: Option<&str>) -> bool {
        let filename = match save_name {
            Some(name) if !name.is_empty() => name.to_string(),
            _ => "quicksave".to_string(),
        };

        match crate::game::persistence::save_to_file(&self.game_state, &filename) {
            Ok(_) => {
                let message = if filename == "quicksave" {
                    "Game saved to: saves/quicksave.json".to_string()
                } else {
                    format!("Game saved to: saves/{}.json", filename)
                };
                self.add_system_message(message);
                self.last_autosave_time = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs();
                true
            }
            Err(e) => {
                self.add_error_message(format!("Save failed: {}", e));
                false
            }
        }
    }

    /// Navigate up in command history (older commands)
    pub fn history_up(&mut self) {
        if self.command_history.is_empty() {
            return;
        }

        if self.history_index > 0 {
            self.history_index -= 1;
            self.input = self.command_history[self.history_index].clone();
            self.move_cursor_end();
        }
    }

    /// Navigate down in command history (newer commands)
    pub fn history_down(&mut self) {
        if self.command_history.is_empty() {
            return;
        }

        if self.history_index < self.command_history.len() {
            self.history_index += 1;
            if self.history_index == self.command_history.len() {
                self.input.clear();
            } else {
                self.input = self.command_history[self.history_index].clone();
            }
            self.move_cursor_end();
        }
    }

    /// Add a command to history
    pub fn add_to_history(&mut self, command: &str) {
        if command.trim().is_empty() {
            return;
        }

        // Don't add duplicates if it's the same as the last command
        if let Some(last) = self.command_history.last() {
            if last == command {
                self.history_index = self.command_history.len();
                return;
            }
        }

        self.command_history.push(command.to_string());
        self.history_index = self.command_history.len();
    }

    /// Tab completion for commands
    pub fn tab_complete(&mut self) {
        let input = self.input.trim();
        if input.is_empty() {
            return;
        }

        let commands = vec![
            "/help",
            "/quit",
            "/inventory",
            "/stats",
            "/worldbook",
            "/equip",
            "/save",
            "look",
            "status",
            "north",
            "south",
            "east",
            "west",
        ];

        // Find matches
        let matches: Vec<&str> = commands
            .iter()
            .filter(|cmd| cmd.starts_with(input))
            .cloned()
            .collect();

        if matches.len() == 1 {
            // Exact match or single completion
            self.input = matches[0].to_string();
            self.move_cursor_end();
        } else if matches.len() > 1 {
            // Multiple matches - show them in log?
            // For now, just cycle or pick first?
            // Simple implementation: pick first
            self.input = matches[0].to_string();
            self.move_cursor_end();

            // Optional: show available completions
            let completions = matches.join(", ");
            self.add_info_message(format!("Completions: {}", completions));
        }
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

    // ============================================================================
    // STOP TOKEN STRIPPING TESTS
    // ============================================================================

    #[test]
    fn test_strip_stop_tokens_with_player_prompt() {
        let content = "You see a large door ahead.\n\n>>> PLAYER:";
        let result = App::strip_stop_tokens(content);
        assert_eq!(result, "You see a large door ahead.");
    }

    #[test]
    fn test_strip_stop_tokens_with_partial_player() {
        let content = "You see a large door ahead.\n\n>>> PLAYER";
        let result = App::strip_stop_tokens(content);
        assert_eq!(result, "You see a large door ahead.");
    }

    #[test]
    fn test_strip_stop_tokens_with_player_colon() {
        let content = "The sheriff looks at you.\n\nPlayer:";
        let result = App::strip_stop_tokens(content);
        assert_eq!(result, "The sheriff looks at you.");
    }

    #[test]
    fn test_strip_stop_tokens_no_tokens() {
        let content = "This is a normal response without any stop tokens.";
        let result = App::strip_stop_tokens(content);
        assert_eq!(result, "This is a normal response without any stop tokens.");
    }

    #[test]
    fn test_strip_stop_tokens_empty_string() {
        let content = "";
        let result = App::strip_stop_tokens(content);
        assert_eq!(result, "");
    }

    #[test]
    fn test_strip_stop_tokens_only_stop_token() {
        let content = ">>> PLAYER:";
        let result = App::strip_stop_tokens(content);
        assert_eq!(result, "");
    }

    #[test]
    fn test_strip_stop_tokens_preserves_content_before() {
        let content =
            "The door creaks open, revealing a dark hallway. What do you do?\n>>> PLAYER:";
        let result = App::strip_stop_tokens(content);
        assert_eq!(
            result,
            "The door creaks open, revealing a dark hallway. What do you do?"
        );
    }
}
