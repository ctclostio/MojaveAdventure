use crate::config::LlamaConfig;

/// Settings editor state
#[derive(Debug, Clone)]
pub struct SettingsEditor {
    /// Currently selected setting index
    pub selected_index: usize,

    /// Whether we're in edit mode
    pub editing: bool,

    /// Current input buffer when editing
    pub edit_buffer: String,

    /// Cursor position in edit buffer
    pub edit_cursor: usize,

    /// Current settings being edited (working copy)
    pub working_config: LlamaConfig,

    /// Status message to show to user
    pub status_message: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SettingField {
    ServerUrl,
    ExtractionUrl,
    Temperature,
    TopP,
    TopK,
    MaxTokens,
    RepeatPenalty,
}

impl SettingsEditor {
    pub fn new(config: LlamaConfig) -> Self {
        Self {
            selected_index: 0,
            editing: false,
            edit_buffer: String::new(),
            edit_cursor: 0,
            working_config: config,
            status_message: None,
        }
    }

    /// Get all available settings fields in order
    pub fn get_fields() -> Vec<SettingField> {
        vec![
            SettingField::ServerUrl,
            SettingField::ExtractionUrl,
            SettingField::Temperature,
            SettingField::TopP,
            SettingField::TopK,
            SettingField::MaxTokens,
            SettingField::RepeatPenalty,
        ]
    }

    /// Get the currently selected field
    pub fn current_field(&self) -> SettingField {
        let fields = Self::get_fields();
        fields[self.selected_index.min(fields.len() - 1)]
    }

    /// Get field display name
    pub fn field_name(field: SettingField) -> &'static str {
        match field {
            SettingField::ServerUrl => "DM Server URL",
            SettingField::ExtractionUrl => "Extractor Server URL",
            SettingField::Temperature => "Temperature",
            SettingField::TopP => "Top P",
            SettingField::TopK => "Top K",
            SettingField::MaxTokens => "Max Tokens",
            SettingField::RepeatPenalty => "Repeat Penalty",
        }
    }

    /// Get field description
    pub fn field_description(field: SettingField) -> &'static str {
        match field {
            SettingField::ServerUrl => "URL for the narrative DM llama.cpp server",
            SettingField::ExtractionUrl => "URL for the entity extraction llama.cpp server",
            SettingField::Temperature => "Randomness (0.0 = deterministic, 2.0 = very random)",
            SettingField::TopP => "Nucleus sampling threshold (0.0-1.0)",
            SettingField::TopK => "Top-K sampling (number of tokens to consider)",
            SettingField::MaxTokens => "Maximum response length (1-32000)",
            SettingField::RepeatPenalty => "Penalty for repeating tokens (1.0-2.0)",
        }
    }

    /// Get current value as string
    pub fn get_value(&self, field: SettingField) -> String {
        match field {
            SettingField::ServerUrl => self.working_config.server_url.clone(),
            SettingField::ExtractionUrl => self.working_config.extraction_url.clone(),
            SettingField::Temperature => format!("{:.2}", self.working_config.temperature),
            SettingField::TopP => format!("{:.2}", self.working_config.top_p),
            SettingField::TopK => self.working_config.top_k.to_string(),
            SettingField::MaxTokens => self.working_config.max_tokens.to_string(),
            SettingField::RepeatPenalty => format!("{:.2}", self.working_config.repeat_penalty),
        }
    }

    /// Move selection up
    pub fn select_previous(&mut self) {
        if self.selected_index > 0 {
            self.selected_index -= 1;
        }
    }

    /// Move selection down
    pub fn select_next(&mut self) {
        let max_index = Self::get_fields().len() - 1;
        if self.selected_index < max_index {
            self.selected_index += 1;
        }
    }

    /// Start editing the current field
    pub fn start_editing(&mut self) {
        if !self.editing {
            self.editing = true;
            let current_value = self.get_value(self.current_field());
            self.edit_buffer = current_value;
            self.edit_cursor = self.edit_buffer.len();
            self.status_message = Some("Editing... Press Enter to save, Esc to cancel".to_string());
        }
    }

    /// Cancel editing
    pub fn cancel_editing(&mut self) {
        self.editing = false;
        self.edit_buffer.clear();
        self.edit_cursor = 0;
        self.status_message = Some("Edit cancelled".to_string());
    }

    /// Save the current edit
    pub fn save_edit(&mut self) -> Result<(), String> {
        if !self.editing {
            return Ok(());
        }

        let field = self.current_field();
        let value = self.edit_buffer.trim();

        // Validate and save the value
        match field {
            SettingField::ServerUrl => {
                if value.is_empty() {
                    return Err("Server URL cannot be empty".to_string());
                }
                self.working_config.server_url = value.to_string();
            }
            SettingField::ExtractionUrl => {
                if value.is_empty() {
                    return Err("Extraction URL cannot be empty".to_string());
                }
                self.working_config.extraction_url = value.to_string();
            }
            SettingField::Temperature => {
                let temp: f32 = value.parse()
                    .map_err(|_| "Invalid number format".to_string())?;
                if !(0.0..=2.0).contains(&temp) {
                    return Err("Temperature must be between 0.0 and 2.0".to_string());
                }
                self.working_config.temperature = temp;
            }
            SettingField::TopP => {
                let top_p: f32 = value.parse()
                    .map_err(|_| "Invalid number format".to_string())?;
                if !(0.0..=1.0).contains(&top_p) {
                    return Err("Top P must be between 0.0 and 1.0".to_string());
                }
                self.working_config.top_p = top_p;
            }
            SettingField::TopK => {
                let top_k: i32 = value.parse()
                    .map_err(|_| "Invalid number format".to_string())?;
                if top_k < 1 {
                    return Err("Top K must be at least 1".to_string());
                }
                self.working_config.top_k = top_k;
            }
            SettingField::MaxTokens => {
                let max_tokens: i32 = value.parse()
                    .map_err(|_| "Invalid number format".to_string())?;
                if max_tokens < 1 || max_tokens > 32000 {
                    return Err("Max tokens must be between 1 and 32000".to_string());
                }
                self.working_config.max_tokens = max_tokens;
            }
            SettingField::RepeatPenalty => {
                let penalty: f32 = value.parse()
                    .map_err(|_| "Invalid number format".to_string())?;
                if !(1.0..=2.0).contains(&penalty) {
                    return Err("Repeat penalty must be between 1.0 and 2.0".to_string());
                }
                self.working_config.repeat_penalty = penalty;
            }
        }

        self.editing = false;
        self.edit_buffer.clear();
        self.edit_cursor = 0;
        self.status_message = Some("Value saved! Press 's' to save to config.toml".to_string());
        Ok(())
    }

    /// Handle character input while editing
    pub fn edit_char(&mut self, c: char) {
        self.edit_buffer.insert(self.edit_cursor, c);
        self.edit_cursor += 1;
    }

    /// Handle backspace while editing
    pub fn edit_backspace(&mut self) {
        if self.edit_cursor > 0 {
            self.edit_buffer.remove(self.edit_cursor - 1);
            self.edit_cursor -= 1;
        }
    }

    /// Move edit cursor left
    pub fn edit_cursor_left(&mut self) {
        if self.edit_cursor > 0 {
            self.edit_cursor -= 1;
        }
    }

    /// Move edit cursor right
    pub fn edit_cursor_right(&mut self) {
        if self.edit_cursor < self.edit_buffer.len() {
            self.edit_cursor += 1;
        }
    }

    /// Move edit cursor to start
    pub fn edit_cursor_home(&mut self) {
        self.edit_cursor = 0;
    }

    /// Move edit cursor to end
    pub fn edit_cursor_end(&mut self) {
        self.edit_cursor = self.edit_buffer.len();
    }

    /// Clear status message
    pub fn clear_status(&mut self) {
        self.status_message = None;
    }
}
