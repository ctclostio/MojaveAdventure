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

        // Skip degenerate tokens early (before buffering) to avoid display garbage
        // These are tokens that are mostly punctuation/underscores/whitespace
        if Self::is_degenerate_token(&token) {
            return;
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

                // Add any content after the marker to filtered output (if not meta-commentary)
                if !self.thinking_line_buffer.is_empty() {
                    let cleaned = Self::strip_channel_markers(&self.thinking_line_buffer);
                    if !cleaned.is_empty() && !Self::is_meta_commentary(&cleaned) {
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

        // Check for narrative delimiters in buffer BEFORE processing lines
        // This handles cases where meta-commentary and narrative are on same line
        let narrative_delimiters = [
            "let's write:",
            "let me write:",
            "here's the narrative:",
            "here's the response:",
            "here it is:",
            "here's what happens:",
        ];

        let lower_buffer = self.thinking_line_buffer.to_lowercase();
        for delimiter in &narrative_delimiters {
            if let Some(pos) = lower_buffer.find(delimiter) {
                // Found narrative delimiter - skip everything before it
                let after_delimiter = pos + delimiter.len();
                if after_delimiter < self.thinking_line_buffer.len() {
                    let narrative_start = self.thinking_line_buffer[after_delimiter..]
                        .trim_start()
                        .to_string();
                    if !narrative_start.is_empty() {
                        // Clear meta-commentary, keep only narrative
                        tracing::trace!(
                            "Found narrative delimiter, extracted: {}",
                            narrative_start
                        );
                        self.thinking_line_buffer = narrative_start;
                    }
                }
                break;
            }
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
                // Strip channel markers first
                let cleaned = Self::strip_channel_markers(&line);
                if !cleaned.is_empty() {
                    // Check for meta-commentary - don't display during streaming
                    if Self::is_meta_commentary(&cleaned) {
                        tracing::trace!("Filtered meta-commentary line: {}", cleaned);
                        continue; // Skip this line entirely
                    }

                    self.in_thinking_mode = false;
                    if let Some(ref mut filtered) = self.filtered_streaming_message {
                        if !filtered.is_empty() {
                            filtered.push('\n');
                        }
                        filtered.push_str(&cleaned);
                    }
                }
            }
        }

        // Note: Content without newlines stays in buffer until finish_streaming
        // where it gets filtered through strip_thinking_content
    }

    /// Check if a line is a thinking/reasoning line from GPT-OSS
    /// GPT-OSS uses the "harmony format" with channel markers, emoji indicators, and <think> tags
    /// NOTE: Meta-commentary detection is NOT done here to avoid false positives on narrative text
    /// Meta-commentary is stripped only during the final cleanup pass in strip_thinking_content
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

        // NOTE: We do NOT check is_meta_commentary here because it would cause false positives
        // on legitimate narrative text containing phrases like "Let me show you" (dialogue)
        // or "We should explore" (NPC speech). Meta-commentary is only stripped in the final
        // cleanup pass via strip_meta_commentary_sentences.

        false
    }

    /// Check if text contains meta-commentary patterns (model planning what to write)
    /// These are phrases like "We should describe...", "The text says...", etc.
    /// Only matches patterns at the START of the text to avoid false positives on dialogue
    fn is_meta_commentary(text: &str) -> bool {
        let lower = text.to_lowercase().trim_start().to_string();

        // Skip leading asterisks/markdown formatting for pattern matching
        let check_text = lower.trim_start_matches('*').trim_start();

        // Meta-commentary indicators that must appear at the START of text
        // These indicate the model is planning/reasoning about what to write
        let start_patterns = [
            "we should",
            "we might",
            "we need to",
            "we could",
            "we can ",
            "we have ", // "We have to describe..."
            "we are ",  // "We are in a vault..." (context stating)
            "we're ",   // "We're going to describe..."
            "let's ",
            "let me ",
            "i should",
            "i'll ",
            "i will ",
            "i need to",
            "i have to",
            "the text says",
            "the prompt",
            "the player ", // "The player wants..."
            "the user ",   // "The user is asking..."
            "this requires",
            "this needs",
            "this is about",
            "then we",
            "first we",
            "first,",
            "now we",
            "now,",
            "now i",
            "okay,",
            "ok,",
            "so,",
            "so we",
            "alright,",
            "hmm",
            "let me think",
            "thinking about",
            // New patterns from actual broken output
            "what happens",   // "what happens as result of..."
            "need to be ",    // "Need to be descriptive"
            "just narrate",   // "Just narrate"
            "no dice",        // "no dice mention"
            "player sees",    // "player sees keycard"
            "they notice",    // "They notice details"
            "provide ",       // "Provide environment description"
            "produce ",       // "Let's produce a concise narrative"
            "maybe a ",       // "maybe a hidden panel"
            "something else", // "or something else"
            "concise narrative",
            "environment description",
            // More patterns from second screenshot
            "according to",      // "According to instructions:"
            "no skill check",    // "no skill check until needed"
            "need to set",       // "So need to set scene"
            "set scene",         // "need to set scene:"
            "mention maybe",     // "Mention maybe vault interior"
            "also mention",      // "Also mention"
            "gear door",         // specific meta-planning
            "flickering lights", // when used in meta context at start
            "vault interior:",   // "Mention maybe vault interior:"
            // More patterns from third screenshot
            "the first...", // "The first... etc.." (with ellipsis = truncated meta)
            "... etc",      // Truncated meta with etc
            "...",          // Starts with ellipsis (continuation of truncated text)
            "…",            // Unicode ellipsis at start
            // Assistant-style patterns (chatbot breaking roleplay)
            "sure thing",     // "Sure thing! Let's get your character..."
            "sure!",          // "Sure! I'll..."
            "of course!",     // "Of course! Let me..."
            "absolutely!",    // "Absolutely! Here's..."
            "happy to help",  // "Happy to help!"
            "glad to help",   // "Glad to help!"
            "i'd be happy",   // "I'd be happy to..."
            "i'd be glad",    // "I'd be glad to..."
            "certainly!",     // "Certainly! Here's..."
            "great question", // "Great question!"
            "good question",  // "Good question!"
            "no problem",     // "No problem!"
            "you got it",     // "You got it!"
            "here we go",     // "Here we go!" (when not narrative)
            "let's get",      // "Let's get your character ready"
            "let's do",       // "Let's do this!"
            "let's begin",    // "Let's begin!"
            "let's start",    // "Let's start!"
        ];

        for pattern in &start_patterns {
            if check_text.starts_with(pattern) {
                return true;
            }
        }

        // Additional patterns that can appear anywhere but are very specific to meta-commentary
        // These are unlikely to appear in normal narrative
        let anywhere_patterns = [
            "should describe",
            "might mention",
            "need to include",
            "describe environment",
            "mention doors",
            "should mention",
            "could mention",
            "will describe",
            "will mention",
            "the user wants",
            "the player wants",
            "respond with",
            "write a response",
            "no need to mention",
            "no need to describe",
            "no need to include",
            "don't need to mention",
            "don't mention",
            "just description",
            "just narration",
            "just narrative",
            "focus on description",
            "focus on the description",
            "but no need",
            "not necessary to mention",
            // New patterns from actual broken output
            "as result of",        // "what happens as result of Perception check"
            "next choices",        // "and next choices"
            "player sees",         // Can appear mid-sentence too
            "they notice details", // "They notice details about..."
            "maintenance hatches",
            "hidden panel",
            "dice mention",
            // More patterns from second screenshot
            "until needed",    // "no skill check until needed"
            "set scene",       // "need to set scene"
            "mention maybe",   // Can appear mid-sentence
            "also mention",    // Can appear mid-sentence
            "vault interior:", // Meta-planning about vault
            "gear door etc",   // Planning text
            // More patterns from third screenshot - truncated/repeated output
            "... etc", // Truncated meta ending with etc
            ".. etc",  // Truncated meta
            ". etc",   // Single period before etc
            "… …",     // Multiple ellipses (broken output)
            "... ...", // Multiple ASCII ellipses
            // Assistant-style anywhere patterns
            "let's get your character", // "Sure thing! Let's get your character ready"
            "get your character ready",
            "ready for action",      // "...ready for action:"
            "here's your character", // Direct assistant addressing
            "your character is:",    // Direct character stat listing
            "- hp:",                 // Character stat listing mid-narrative
        ];

        for pattern in &anywhere_patterns {
            if lower.contains(pattern) {
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
            // Skip if it's thinking indicator OR meta-commentary
            if !trimmed.is_empty()
                && !Self::is_thinking_indicator(trimmed)
                && !Self::is_meta_commentary(trimmed)
            {
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

        // For multi-line content, filter out lines that are purely thinking indicators
        // (emoji prefixes, harmony markers, etc.) but NOT meta-commentary
        // Meta-commentary is handled at the sentence level below
        if result.lines().count() > 1 {
            result = result
                .lines()
                .filter(|line| {
                    let trimmed = line.trim();
                    // Only filter lines with explicit thinking markers, not meta-commentary
                    !Self::has_explicit_thinking_markers(trimmed)
                })
                .collect::<Vec<&str>>()
                .join("\n");
        }

        // Strip meta-commentary sentences from the beginning
        // These are sentences like "We should describe...", "The text says...", etc.
        result = Self::strip_meta_commentary_sentences(&result);

        // Clean up degenerate/garbled punctuation patterns
        result = Self::clean_degenerate_punctuation(&result);

        // Detect and remove repetitive/looping output
        result = Self::remove_repetitions(&result);

        // Strip any remaining channel markers
        Self::strip_channel_markers(&result)
    }

    /// Clean up degenerate punctuation patterns that indicate broken model output
    /// Patterns like "....", "????", "**?**", excessive asterisks, etc.
    fn clean_degenerate_punctuation(content: &str) -> String {
        use regex::Regex;

        // Lazy static would be better but this is called infrequently (once per response)
        let patterns: Vec<(Regex, &str)> = vec![
            // Multiple periods (3+ in a row) -> ellipsis
            (Regex::new(r"\.{3,}").unwrap(), "..."),
            // Multiple question marks (2+ in a row) -> single
            (Regex::new(r"\?{2,}").unwrap(), "?"),
            // Multiple exclamation marks (2+ in a row) -> single
            (Regex::new(r"!{2,}").unwrap(), "!"),
            // Unicode ellipsis followed by periods or vice versa
            (Regex::new(r"…\.+|\.+…").unwrap(), "..."),
            // Multiple unicode ellipsis
            (Regex::new(r"…{2,}").unwrap(), "..."),
            // Orphaned markdown bold/italic markers with just punctuation inside: **?** or *?*
            (Regex::new(r"\*\*[?.!…\s]*\*\*").unwrap(), ""),
            (Regex::new(r"\*[?.!…\s]*\*").unwrap(), ""),
            // Multiple spaces
            (Regex::new(r" {2,}").unwrap(), " "),
            // Standalone punctuation clusters (just dots, question marks, etc. with spaces)
            (Regex::new(r"^\s*[.?!…\s]+\s*$").unwrap(), ""),
            // Leading/trailing garbage punctuation (not part of words)
            (Regex::new(r"^[.?!…*\s]+").unwrap(), ""),
        ];

        let mut result = content.to_string();
        for (regex, replacement) in &patterns {
            result = regex.replace_all(&result, *replacement).to_string();
        }

        // Also filter out "sentences" that are mostly punctuation (>50% non-alphanumeric)
        result = result
            .split(". ")
            .filter(|sentence| {
                let total = sentence.len();
                if total == 0 {
                    return false;
                }
                let alpha_count = sentence.chars().filter(|c| c.is_alphanumeric()).count();
                // Keep sentence if at least 30% is alphanumeric
                (alpha_count as f32 / total as f32) > 0.3
            })
            .collect::<Vec<&str>>()
            .join(". ");

        result.trim().to_string()
    }

    /// Detect and remove repeated content from looping model output
    /// When a model gets stuck in a loop, it may repeat the same sentences/paragraphs
    fn remove_repetitions(content: &str) -> String {
        // Split into sentences for analysis
        let sentences: Vec<&str> = content
            .split(['.', '!', '?'])
            .filter(|s| !s.trim().is_empty())
            .collect();

        // Not enough sentences to have meaningful repetition
        if sentences.len() < 4 {
            return content.to_string();
        }

        // Normalize sentences for comparison (lowercase, trim, collapse whitespace)
        let normalize = |s: &str| -> String {
            s.to_lowercase()
                .split_whitespace()
                .collect::<Vec<_>>()
                .join(" ")
        };

        let normalized: Vec<String> = sentences.iter().map(|s| normalize(s)).collect();

        // Find repeating patterns - look for sequences that repeat
        // Start with longer potential repeating units (more likely to be meaningful)
        // Include unit_len of 1 to catch single sentence repetition
        for unit_len in (1..=sentences.len() / 2).rev() {
            // Check if first `unit_len` sentences repeat throughout
            let first_unit: Vec<&str> = normalized
                .iter()
                .take(unit_len)
                .map(|s| s.as_str())
                .collect();

            let mut repeat_count = 0;
            let mut i = 0;
            while i + unit_len <= normalized.len() {
                let current_unit: Vec<&str> = normalized[i..i + unit_len]
                    .iter()
                    .map(|s| s.as_str())
                    .collect();
                if current_unit == first_unit {
                    repeat_count += 1;
                    i += unit_len;
                } else {
                    break;
                }
            }

            // If we found 3+ repeats of a meaningful unit, truncate to first occurrence
            if repeat_count >= 3 {
                tracing::warn!(
                    "Detected repetition: {} sentences repeated {} times, truncating",
                    unit_len,
                    repeat_count
                );

                // Reconstruct just the first unit with original punctuation
                // Find the end of the first unit in the original content
                let mut end_pos = 0;
                let mut sentence_count = 0;
                for (i, c) in content.char_indices() {
                    if c == '.' || c == '!' || c == '?' {
                        sentence_count += 1;
                        if sentence_count == unit_len {
                            end_pos = i + 1;
                            break;
                        }
                    }
                }

                if end_pos > 0 && end_pos < content.len() {
                    return content[..end_pos].trim().to_string();
                }
            }
        }

        // Also check for simple consecutive duplicate sentences
        let mut seen_sentences: std::collections::HashMap<String, usize> =
            std::collections::HashMap::new();
        let mut consecutive_dupes = 0;
        let mut last_sentence = String::new();

        for normalized_sentence in &normalized {
            if normalized_sentence.len() > 20 {
                // Only meaningful sentences
                *seen_sentences
                    .entry(normalized_sentence.clone())
                    .or_insert(0) += 1;

                if *normalized_sentence == last_sentence {
                    consecutive_dupes += 1;
                } else {
                    consecutive_dupes = 0;
                }

                // If same sentence appears 3+ times consecutively, it's a loop
                if consecutive_dupes >= 2 {
                    tracing::warn!("Detected consecutive duplicate sentence, truncating");
                    // Find first occurrence and cut there
                    if let Some(pos) = content.to_lowercase().find(&last_sentence) {
                        let end_pos = content[pos..]
                            .find(['.', '!', '?'])
                            .map(|p| pos + p + 1)
                            .unwrap_or(content.len());
                        if end_pos < content.len() {
                            return content[..end_pos].trim().to_string();
                        }
                    }
                }

                last_sentence = normalized_sentence.clone();
            }
        }

        content.to_string()
    }

    /// Check if a token is degenerate (mostly garbage punctuation/repetition)
    /// These tokens indicate the model is producing broken output and should be skipped
    fn is_degenerate_token(token: &str) -> bool {
        // Empty or whitespace-only tokens are not degenerate (just skip them)
        let trimmed = token.trim();
        if trimmed.is_empty() {
            return false;
        }

        // Count characters by type
        let total_chars = trimmed.chars().count();
        if total_chars == 0 {
            return false;
        }

        // Very short tokens (1-2 chars) are never degenerate - they're legitimate punctuation
        if total_chars <= 2 {
            return false;
        }

        let alphanumeric_count = trimmed.chars().filter(|c| c.is_alphanumeric()).count();
        let punctuation_count = trimmed
            .chars()
            .filter(|c| matches!(c, '.' | '?' | '!' | '…' | '-' | '_' | '*' | '>' | '<' | '|'))
            .count();

        // Token is degenerate if:
        // 1. It's mostly punctuation (>70% punctuation, <20% alphanumeric) AND longer than 3 chars
        // 2. It contains repetitive patterns like "....", "????", "____" (4+ repetitions)
        // 3. It's a long token that's just punctuation

        // Check for mostly punctuation with very little actual content (only for longer tokens)
        let punct_ratio = punctuation_count as f32 / total_chars as f32;
        let alpha_ratio = alphanumeric_count as f32 / total_chars as f32;

        // Only consider it degenerate if it's long AND mostly punctuation
        if total_chars >= 5 && punct_ratio > 0.7 && alpha_ratio < 0.2 {
            return true;
        }

        // Check for highly repetitive punctuation patterns (need 4+ chars of same pattern)
        let degenerate_patterns = [
            "....", "????", "!!!!", "____", "----", "….…", ".….", "??.?", "!.!", "._._", "-.-.",
            "****", ">>>>", "<<<<", "|>|>", "<|<|",
        ];

        for pattern in &degenerate_patterns {
            if trimmed.contains(pattern) {
                return true;
            }
        }

        // Check for mixed degenerate patterns (combinations of different punctuation)
        // These are highly specific patterns seen in broken model output
        let mixed_degenerate = [
            "..??", "??...", "…??", "??…", "._.", "-..", "..-", "__.", ".__", "**?", "?**",
        ];

        for pattern in &mixed_degenerate {
            if trimmed.contains(pattern) {
                return true;
            }
        }

        // Check for long runs of the same character (4+ in a row)
        let mut last_char = '\0';
        let mut run_length = 0;
        for c in trimmed.chars() {
            if c == last_char && !c.is_alphanumeric() && !c.is_whitespace() {
                run_length += 1;
                if run_length >= 4 {
                    return true;
                }
            } else {
                last_char = c;
                run_length = 1;
            }
        }

        false
    }

    /// Check if a line has explicit thinking markers (tags, emojis) but NOT meta-commentary
    /// This is used for line-based filtering to avoid filtering out mixed content
    fn has_explicit_thinking_markers(line: &str) -> bool {
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

        for marker in &harmony_thinking_markers {
            if line.contains(marker) {
                return true;
            }
        }

        // GPT-OSS emoji prefixes for chain-of-thought
        let thinking_prefixes = ["🤔", "💭"];

        for prefix in &thinking_prefixes {
            if line.starts_with(prefix) {
                return true;
            }
        }

        false
    }

    /// Strip meta-commentary sentences from the beginning of the response
    /// Keeps stripping sentences until we find one that isn't meta-commentary
    fn strip_meta_commentary_sentences(content: &str) -> String {
        let mut result = content.trim().to_string();

        // FIRST: Check for explicit "narrative start" delimiters
        // These are phrases where the model explicitly transitions to writing the actual content
        // Everything AFTER these delimiters is the real narrative
        let narrative_delimiters = [
            "let's write:",
            "let me write:",
            "here's the narrative:",
            "here's the response:",
            "here it is:",
            "here's what happens:",
            "actual response:",
            "final response:",
            "output:",
            // Note: "the response:" and "the narrative:" removed - too generic, causes false positives
        ];

        // Keep checking for delimiters until none are found
        // Must recompute lowercase each iteration since result changes
        loop {
            let lower_result = result.to_lowercase();
            let mut found_delimiter = false;

            for delimiter in &narrative_delimiters {
                if let Some(pos) = lower_result.find(delimiter) {
                    // Extract everything after the delimiter
                    let after_delimiter = pos + delimiter.len();
                    if after_delimiter < result.len() {
                        result = result[after_delimiter..].trim_start().to_string();
                        found_delimiter = true;
                        break; // Restart the loop with updated result
                    }
                }
            }

            if !found_delimiter {
                break;
            }
        }

        // Keep stripping meta-commentary sentences from the beginning
        loop {
            let trimmed = result.trim_start();
            if trimmed.is_empty() {
                break;
            }

            // Check for "etc " pattern (common delimiter in meta-commentary)
            // e.g., "Then we might mention doors, etc The actual content..."
            // This must come BEFORE sentence boundary check because "etc" often
            // appears in the middle of what looks like one long sentence
            if let Some(etc_pos) = trimmed.to_lowercase().find("etc ") {
                let before_etc = &trimmed[..etc_pos + 4]; // Include "etc "
                if Self::is_meta_commentary(before_etc) {
                    // Skip past "etc " and continue
                    result = trimmed[etc_pos + 4..].trim_start().to_string();
                    continue;
                }
            }

            // Find first sentence boundary (. or ? or !)
            let sentence_end = Self::find_sentence_boundary(trimmed);

            if let Some(end_pos) = sentence_end {
                let first_sentence = &trimmed[..=end_pos]; // Include the punctuation

                // Check if this sentence is meta-commentary
                if Self::is_meta_commentary(first_sentence) {
                    // Skip this sentence - find where to continue
                    let skip_to = end_pos + 1;
                    if skip_to < trimmed.len() {
                        result = trimmed[skip_to..].trim_start().to_string();
                    } else {
                        // Nothing left after this sentence
                        result = String::new();
                        break;
                    }
                } else {
                    // First non-meta sentence found, stop stripping
                    break;
                }
            } else {
                // No clear sentence boundary - check if entire remaining text is meta
                if Self::is_meta_commentary(trimmed) {
                    result = String::new();
                }
                break;
            }
        }

        result
    }

    /// Find the first sentence boundary (. ? ! or …) in text
    /// Returns the byte position of the LAST byte of the punctuation mark
    /// This is critical for correct UTF-8 slicing with multi-byte characters like …
    fn find_sentence_boundary(text: &str) -> Option<usize> {
        // Collect char_indices to properly navigate by character index
        let chars: Vec<(usize, char)> = text.char_indices().collect();

        for (char_idx, &(byte_pos, c)) in chars.iter().enumerate() {
            // Check for standard sentence-ending punctuation
            if c == '.' || c == '?' || c == '!' || c == '…' {
                // Get next character using character index (not byte index)
                let next_char = chars.get(char_idx + 1).map(|(_, ch)| *ch);
                // Valid boundary if followed by: end of string, space, newline, or asterisk (markdown)
                if next_char.is_none()
                    || next_char == Some(' ')
                    || next_char == Some('\n')
                    || next_char == Some('*')
                {
                    // Return byte position of the END of this character (last byte)
                    // This ensures slicing with [..=end_pos] includes the full character
                    return Some(byte_pos + c.len_utf8() - 1);
                }
            }
        }
        None
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

    // ============================================================================
    // META-COMMENTARY STRIPPING TESTS
    // ============================================================================

    #[test]
    fn test_is_meta_commentary_we_should() {
        // Pattern at start of text
        assert!(App::is_meta_commentary(
            "We should describe the environment."
        ));
        assert!(App::is_meta_commentary("we should mention the doors"));
    }

    #[test]
    fn test_is_meta_commentary_the_text_says() {
        // Pattern at start of text
        assert!(App::is_meta_commentary("The text says we need to respond"));
    }

    #[test]
    fn test_is_meta_commentary_then_we() {
        // Pattern at start of text
        assert!(App::is_meta_commentary("Then we might mention doors, etc"));
    }

    #[test]
    fn test_is_meta_commentary_negative() {
        // These should NOT be detected as meta-commentary
        assert!(!App::is_meta_commentary(
            "The vaulted concrete walls rise high above you."
        ));
        assert!(!App::is_meta_commentary("You see a door ahead."));
        assert!(!App::is_meta_commentary(
            "The wasteland stretches before you."
        ));
        // Patterns in the middle should NOT match (dialogue, etc.)
        assert!(!App::is_meta_commentary(
            "The guard says 'Let me show you the way.'"
        ));
        assert!(!App::is_meta_commentary(
            "\"We should explore this place,\" says Marcus."
        ));
        assert!(!App::is_meta_commentary(
            "You realize that we are at a crossroads."
        ));
    }

    #[test]
    fn test_strip_meta_commentary_sentences_basic() {
        let content = "We should describe environment. The vaulted concrete walls rise above you.";
        let result = App::strip_meta_commentary_sentences(content);
        assert_eq!(result, "The vaulted concrete walls rise above you.");
    }

    #[test]
    fn test_strip_meta_commentary_sentences_multiple() {
        let content = "The text says? We are at Vault 13. We should describe. Then we might mention doors. The vaulted walls rise above you.";
        let result = App::strip_meta_commentary_sentences(content);
        assert_eq!(result, "The vaulted walls rise above you.");
    }

    #[test]
    fn test_strip_meta_commentary_sentences_no_meta() {
        let content =
            "The vaulted concrete walls rise high above you, a testament to pre-war engineering.";
        let result = App::strip_meta_commentary_sentences(content);
        assert_eq!(result, content);
    }

    #[test]
    fn test_strip_thinking_content_with_meta() {
        // Simulates the actual output from GPT-OSS we saw in the screenshot
        let content = "The text says? We are at Vault 13 Entrance. We should describe environment. Then we might mention doors, etc The vaulted concrete walls of Vault 13 rise high above you, a testament to pre-war engineering.";
        let result = App::strip_thinking_content(content);
        assert!(
            result.starts_with("The vaulted concrete walls"),
            "Expected to start with 'The vaulted concrete walls', got: '{}'",
            result
        );
        assert!(!result.contains("We should"));
        assert!(!result.contains("The text says"));
    }

    #[test]
    fn test_find_sentence_boundary() {
        assert_eq!(App::find_sentence_boundary("Hello. World"), Some(5));
        assert_eq!(App::find_sentence_boundary("Hello? World"), Some(5));
        assert_eq!(App::find_sentence_boundary("Hello! World"), Some(5));
        assert_eq!(App::find_sentence_boundary("No boundary here"), None);
    }

    // ============================================================================
    // DEGENERATE TOKEN FILTERING TESTS
    // ============================================================================

    #[test]
    fn test_is_degenerate_token_repetitive_punctuation() {
        // Patterns seen in actual broken model output
        assert!(App::is_degenerate_token("..??...??")); // Mixed punctuation chaos
        assert!(App::is_degenerate_token("….???…")); // Unicode ellipsis mixed with ?
        assert!(App::is_degenerate_token("__________")); // Long runs of underscores
        assert!(App::is_degenerate_token("**?**")); // Asterisks with question mark
        assert!(App::is_degenerate_token("....")); // 4+ periods
        assert!(App::is_degenerate_token("??????")); // 4+ question marks
        assert!(App::is_degenerate_token("----")); // 4+ dashes
    }

    #[test]
    fn test_is_degenerate_token_mostly_punctuation() {
        // Tokens that are >70% punctuation AND 5+ chars
        assert!(App::is_degenerate_token("a..??..")); // Has ..?? pattern
        assert!(App::is_degenerate_token("x_.__.")); // Has ._. pattern
        assert!(App::is_degenerate_token(".....a")); // Long punctuation with tiny text
    }

    #[test]
    fn test_is_degenerate_token_normal_tokens() {
        // Normal tokens should NOT be filtered
        assert!(!App::is_degenerate_token("Hello"));
        assert!(!App::is_degenerate_token("world!"));
        assert!(!App::is_degenerate_token("The vault"));
        assert!(!App::is_degenerate_token(" ")); // whitespace-only returns false (not degenerate, just empty)
        assert!(!App::is_degenerate_token("")); // empty returns false
        assert!(!App::is_degenerate_token("We're")); // contractions are fine
    }

    #[test]
    fn test_is_degenerate_token_legitimate_punctuation() {
        // Single or double punctuation that's legitimate
        assert!(!App::is_degenerate_token(".")); // single period
        assert!(!App::is_degenerate_token("?")); // single question mark
        assert!(!App::is_degenerate_token("!")); // single exclamation
        assert!(!App::is_degenerate_token(",")); // comma
        assert!(!App::is_degenerate_token(";")); // semicolon
        assert!(!App::is_degenerate_token(":")); // colon
    }

    #[test]
    fn test_is_degenerate_token_mixed_content() {
        // Mixed content with some punctuation - should NOT be filtered if enough text
        assert!(!App::is_degenerate_token("Hello, world!"));
        assert!(!App::is_degenerate_token("What?"));
        assert!(!App::is_degenerate_token("Yes...")); // Valid ellipsis (only 3 dots)
        assert!(!App::is_degenerate_token("Wait...")); // Valid ellipsis with word
        assert!(!App::is_degenerate_token("...")); // Short - not filtered
    }

    // ============================================================================
    // NEW META-COMMENTARY PATTERN TESTS
    // ============================================================================

    #[test]
    fn test_is_meta_commentary_new_patterns() {
        // Patterns from the actual broken output in screenshot
        assert!(App::is_meta_commentary(
            "what happens as result of Perception check"
        ));
        assert!(App::is_meta_commentary("Need to be descriptive"));
        assert!(App::is_meta_commentary("Just narrate"));
        assert!(App::is_meta_commentary("no dice mention"));
        assert!(App::is_meta_commentary("player sees keycard"));
        assert!(App::is_meta_commentary(
            "They notice details about the latch"
        ));
        assert!(App::is_meta_commentary("Provide environment description"));
        // Patterns from second screenshot
        assert!(App::is_meta_commentary(
            "According to instructions: Provide narrative"
        ));
        assert!(App::is_meta_commentary("no skill check until needed"));
        assert!(App::is_meta_commentary("So need to set scene: Vault 13"));
        assert!(App::is_meta_commentary(
            "Mention maybe vault interior: dust"
        ));
        assert!(App::is_meta_commentary("Also mention the door"));
    }

    #[test]
    fn test_is_meta_commentary_anywhere_patterns() {
        // These patterns should be detected anywhere in the text
        assert!(App::is_meta_commentary("And also next choices"));
        assert!(App::is_meta_commentary("Text with as result of in middle"));
    }

    // ============================================================================
    // NARRATIVE DELIMITER TESTS
    // ============================================================================

    #[test]
    fn test_strip_meta_commentary_lets_write_delimiter() {
        // The exact pattern from the screenshot
        let content = "Let's write:You swallow a breath and scan the vault's dim interior.";
        let result = App::strip_meta_commentary_sentences(content);
        assert_eq!(
            result,
            "You swallow a breath and scan the vault's dim interior."
        );
    }

    #[test]
    fn test_strip_meta_commentary_lets_write_with_space() {
        let content = "Let's write: The vaulted walls rise above you.";
        let result = App::strip_meta_commentary_sentences(content);
        assert_eq!(result, "The vaulted walls rise above you.");
    }

    #[test]
    fn test_strip_meta_commentary_here_response() {
        let content = "Here's the response: You see a door ahead.";
        let result = App::strip_meta_commentary_sentences(content);
        assert_eq!(result, "You see a door ahead.");
    }

    #[test]
    fn test_strip_full_screenshot_pattern() {
        // Full pattern from the screenshot (simplified)
        let content = "We need to respond after success. Need to be descriptive. Let's write:You swallow a breath and scan the vault.";
        let result = App::strip_meta_commentary_sentences(content);
        assert_eq!(result, "You swallow a breath and scan the vault.");
    }

    #[test]
    fn test_strip_complex_meta_with_etc() {
        // Pattern with "etc" delimiter
        let content = "They notice details about latch, door, etc The faint flicker of the broken fluorescent strip now reveals more.";
        let result = App::strip_meta_commentary_sentences(content);
        assert_eq!(
            result,
            "The faint flicker of the broken fluorescent strip now reveals more."
        );
    }

    #[test]
    fn test_strip_according_to_instructions_pattern() {
        // Pattern from second screenshot - entire text is meta-commentary
        let content = "According to instructions: Provide narrative; no skill check until needed. So need to set scene: Vault 13 entrance, gear door etc. Mention maybe vault interior: dust, posters, flickering lights.";
        let result = App::strip_meta_commentary_sentences(content);
        // Should strip everything since it's all meta-commentary
        assert!(
            result.is_empty() || !result.to_lowercase().contains("according to"),
            "Should strip meta-commentary, got: '{}'",
            result
        );
    }

    #[test]
    fn test_is_meta_commentary_ellipsis_patterns() {
        // Patterns from third screenshot - truncated/repeated output
        assert!(App::is_meta_commentary("The first... etc.. something"));
        assert!(App::is_meta_commentary("... etc The actual content"));
        assert!(App::is_meta_commentary("...The vault entrance"));
        assert!(App::is_meta_commentary("… Unicode ellipsis start"));
    }

    #[test]
    fn test_is_meta_commentary_valid_narrative_with_ellipsis() {
        // Valid narrative that happens to contain ellipsis should NOT be filtered
        // (only filtered if starts with ellipsis)
        assert!(!App::is_meta_commentary(
            "The vault door creaks... then silence."
        ));
        assert!(!App::is_meta_commentary("You wait... nothing happens."));
    }

    #[test]
    fn test_is_meta_commentary_assistant_style() {
        // Assistant-style patterns that break roleplay immersion
        assert!(App::is_meta_commentary(
            "Sure thing! Let's get your character ready"
        ));
        assert!(App::is_meta_commentary("Sure! I'll help you with that"));
        assert!(App::is_meta_commentary("Of course! Here's what happens"));
        assert!(App::is_meta_commentary("Absolutely! The vault door opens"));
        assert!(App::is_meta_commentary(
            "Happy to help! You enter the vault"
        ));
        assert!(App::is_meta_commentary("Certainly! Your character walks"));
        assert!(App::is_meta_commentary(
            "Let's get started with the adventure"
        ));
        assert!(App::is_meta_commentary("Let's begin your adventure"));

        // These should still work - valid narrative that uses "let's" naturally
        assert!(!App::is_meta_commentary("The guard says \"Let's go!\""));
        assert!(!App::is_meta_commentary(
            "\"Sure thing,\" replies the merchant."
        ));
    }

    #[test]
    fn test_is_meta_commentary_anywhere_assistant_patterns() {
        // Anywhere patterns for assistant-style output
        assert!(App::is_meta_commentary(
            "Some text let's get your character ready for action"
        ));
        assert!(App::is_meta_commentary(
            "Here's your character: - HP: 44/44"
        ));
    }

    #[test]
    fn test_remove_repetitions_basic() {
        // Simple case - no repetition
        let input = "The vault door opens. You step inside. The lights flicker. Something moves.";
        assert_eq!(App::remove_repetitions(input), input);
    }

    #[test]
    fn test_remove_repetitions_consecutive_duplicates() {
        // Same sentence repeated consecutively
        let input = "The vault door opens. The vault door opens. The vault door opens. The vault door opens.";
        let result = App::remove_repetitions(input);
        // Should truncate to first occurrence
        assert!(result.len() < input.len());
        assert!(result.contains("The vault door opens"));
    }

    #[test]
    fn test_remove_repetitions_pattern_loop() {
        // Repeating pattern of multiple sentences
        let input = "The air is stale. You hear dripping. The air is stale. You hear dripping. The air is stale. You hear dripping.";
        let result = App::remove_repetitions(input);
        // Should detect the pattern and truncate
        assert!(result.len() < input.len());
    }
}
