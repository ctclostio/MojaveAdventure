/// Pip-Boy themed styling for retro terminal aesthetic
use ratatui::style::{Color, Modifier, Style};

/// Pip-Boy color palette - monochrome green phosphor display
#[allow(dead_code)]
pub struct PipBoyTheme;

#[allow(dead_code)]
impl PipBoyTheme {
    // Primary colors - green phosphor variations
    pub const PHOSPHOR_BRIGHT: Color = Color::Rgb(0, 255, 0); // Bright green
    pub const PHOSPHOR_NORMAL: Color = Color::Rgb(0, 200, 0); // Normal green
    pub const PHOSPHOR_DIM: Color = Color::Rgb(0, 150, 0); // Dim green
    pub const PHOSPHOR_DARK: Color = Color::Rgb(0, 100, 0); // Dark green
    pub const PHOSPHOR_VERY_DARK: Color = Color::Rgb(0, 50, 0); // Very dark green
    pub const BACKGROUND: Color = Color::Rgb(0, 20, 0); // Almost black with green tint

    // Accent colors (still green-tinted)
    pub const AMBER_HIGHLIGHT: Color = Color::Rgb(255, 200, 0); // Amber warning
    pub const RED_DANGER: Color = Color::Rgb(200, 50, 0); // Danger (reddish but muted)

    // Styles for different UI elements

    /// Main text style - normal phosphor green
    pub fn text() -> Style {
        Style::default().fg(Self::PHOSPHOR_NORMAL)
    }

    /// Bright text for emphasis
    pub fn text_bright() -> Style {
        Style::default()
            .fg(Self::PHOSPHOR_BRIGHT)
            .add_modifier(Modifier::BOLD)
    }

    /// Dim text for secondary info
    pub fn text_dim() -> Style {
        Style::default().fg(Self::PHOSPHOR_DIM)
    }

    /// Very dim text for tertiary/background info
    pub fn text_very_dim() -> Style {
        Style::default().fg(Self::PHOSPHOR_VERY_DARK)
    }

    /// Header/title style
    pub fn header() -> Style {
        Style::default()
            .fg(Self::PHOSPHOR_BRIGHT)
            .add_modifier(Modifier::BOLD | Modifier::UNDERLINED)
    }

    /// Border style - dim green
    pub fn border() -> Style {
        Style::default().fg(Self::PHOSPHOR_DIM)
    }

    /// Border style for active/focused elements
    pub fn border_active() -> Style {
        Style::default().fg(Self::PHOSPHOR_BRIGHT)
    }

    /// Health bar - full
    pub fn health_full() -> Style {
        Style::default().fg(Self::PHOSPHOR_BRIGHT)
    }

    /// Health bar - medium
    pub fn health_medium() -> Style {
        Style::default().fg(Self::AMBER_HIGHLIGHT)
    }

    /// Health bar - low
    pub fn health_low() -> Style {
        Style::default().fg(Self::RED_DANGER)
    }

    /// AP/energy bar
    pub fn energy() -> Style {
        Style::default().fg(Self::PHOSPHOR_NORMAL)
    }

    /// Player action/input
    pub fn player_action() -> Style {
        Style::default()
            .fg(Self::PHOSPHOR_BRIGHT)
            .add_modifier(Modifier::ITALIC)
    }

    /// DM/narrative text
    pub fn dm_text() -> Style {
        Style::default().fg(Self::PHOSPHOR_NORMAL)
    }

    /// Combat message
    pub fn combat() -> Style {
        Style::default()
            .fg(Self::AMBER_HIGHLIGHT)
            .add_modifier(Modifier::BOLD)
    }

    /// System message
    pub fn system() -> Style {
        Style::default().fg(Self::PHOSPHOR_DIM)
    }

    /// Info message
    pub fn info() -> Style {
        Style::default().fg(Self::PHOSPHOR_NORMAL)
    }

    /// Success message
    pub fn success() -> Style {
        Style::default()
            .fg(Self::PHOSPHOR_BRIGHT)
            .add_modifier(Modifier::BOLD)
    }

    /// Error message
    pub fn error() -> Style {
        Style::default()
            .fg(Self::RED_DANGER)
            .add_modifier(Modifier::BOLD)
    }

    /// Warning/wounded status
    pub fn warning() -> Style {
        Style::default().fg(Self::AMBER_HIGHLIGHT)
    }

    /// Selected/highlighted item
    pub fn selected() -> Style {
        Style::default()
            .fg(Self::PHOSPHOR_BRIGHT)
            .bg(Self::PHOSPHOR_VERY_DARK)
            .add_modifier(Modifier::BOLD)
    }

    /// Loading/waiting indicator
    pub fn loading() -> Style {
        Style::default()
            .fg(Self::PHOSPHOR_DIM)
            .add_modifier(Modifier::SLOW_BLINK)
    }
}

/// Scanline effect overlay - creates retro CRT appearance
#[allow(dead_code)]
pub struct ScanlineEffect;

#[allow(dead_code)]
impl ScanlineEffect {
    /// Get scanline characters for a given row
    /// Alternates between normal and slightly dimmed to create scanline effect
    pub fn get_overlay(row: u16) -> Option<char> {
        // Every other row gets a subtle overlay
        if row.is_multiple_of(2) {
            Some('▒') // Light shade for scanline effect
        } else {
            None
        }
    }

    /// Check if this row should have a scanline
    pub fn is_scanline_row(row: u16) -> bool {
        row.is_multiple_of(2)
    }

    /// Get scanline style (very subtle dark green)
    pub fn style() -> Style {
        Style::default()
            .fg(PipBoyTheme::PHOSPHOR_VERY_DARK)
            .bg(PipBoyTheme::BACKGROUND)
    }
}

/// Loading spinner frames for AI response waiting
#[allow(dead_code)]
pub struct LoadingSpinner {
    frames: Vec<&'static str>,
    current_frame: usize,
}

#[allow(dead_code)]
impl LoadingSpinner {
    /// Create a new loading spinner
    pub fn new() -> Self {
        Self {
            frames: vec!["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"],
            current_frame: 0,
        }
    }

    /// Get the next frame
    pub fn next_frame(&mut self) -> &'static str {
        let frame = self.frames[self.current_frame];
        self.current_frame = (self.current_frame + 1) % self.frames.len();
        frame
    }

    /// Get current frame without advancing
    pub fn current(&self) -> &'static str {
        self.frames[self.current_frame]
    }

    /// Reset to first frame
    pub fn reset(&mut self) {
        self.current_frame = 0;
    }
}

impl Default for LoadingSpinner {
    fn default() -> Self {
        Self::new()
    }
}

/// Terminal sound effects using system bell
#[allow(dead_code)]
pub struct TerminalSound;

#[allow(dead_code)]
impl TerminalSound {
    /// Play terminal bell
    fn play_bell() {
        print!("\x07");
        use std::io::{self, Write};
        let _ = io::stdout().flush();
    }

    /// Sound for critical hit
    pub fn critical_hit() {
        Self::play_bell();
        std::thread::sleep(std::time::Duration::from_millis(50));
        Self::play_bell();
    }

    /// Sound for level up
    pub fn level_up() {
        for _ in 0..3 {
            Self::play_bell();
            std::thread::sleep(std::time::Duration::from_millis(100));
        }
    }

    /// Sound for enemy defeated
    pub fn enemy_defeated() {
        Self::play_bell();
    }

    /// Sound for player damage
    pub fn player_damaged() {
        Self::play_bell();
    }

    /// Sound for error/invalid action
    pub fn error() {
        for _ in 0..2 {
            Self::play_bell();
            std::thread::sleep(std::time::Duration::from_millis(80));
        }
    }

    /// Sound for success action
    pub fn success() {
        Self::play_bell();
    }
}

/// Pip-Boy ASCII art header
#[allow(dead_code)]
pub struct PipBoyHeader;

#[allow(dead_code)]
impl PipBoyHeader {
    pub fn get_header() -> Vec<&'static str> {
        vec![
            "╔═══════════════════════════════════════════════════════════════════╗",
            "║              ┏━┓╻┏━┓   ┏┓ ┏━┓╻ ╻   ┏━┓┏━┓┏━┓┏━┓                 ║",
            "║              ┣━┛┃┣━┛╺━╸┣┻┓┃ ┃┗┳┛   ┏━┛┃ ┃┃ ┃┃ ┃                 ║",
            "║              ╹  ╹╹      ┗━┛┗━┛ ╹    ┗━╸┗━┛┗━┛┗━┛                 ║",
            "║                    VAULT-TEC PERSONAL TERMINAL                   ║",
            "╚═══════════════════════════════════════════════════════════════════╝",
        ]
    }

    pub fn get_boot_sequence() -> Vec<&'static str> {
        vec![
            "",
            "  ROBCO INDUSTRIES UNIFIED OPERATING SYSTEM",
            "  COPYRIGHT 2075-2077 ROBCO INDUSTRIES",
            "  -Server 1-",
            "",
            "  > RUN DEBUG/ACCOUNTS.F",
            "  > LOAD VAULT_SECURITY_PROTOCOLS",
            "  > INITIALIZE PIP-BOY 3000",
            "",
            "  WELCOME TO VAULT-TEC!",
            "",
        ]
    }
}

/// Retro terminal effects and decorations
#[allow(dead_code)]
pub struct RetroEffects;

#[allow(dead_code)]
impl RetroEffects {
    /// Get a decorative line separator
    pub fn separator_line(width: usize) -> String {
        "─".repeat(width)
    }

    /// Get a decorative double line separator
    pub fn separator_double(width: usize) -> String {
        "═".repeat(width)
    }

    /// Get retro-style box corners
    pub fn box_top_left() -> char {
        '┌'
    }

    pub fn box_top_right() -> char {
        '┐'
    }

    pub fn box_bottom_left() -> char {
        '└'
    }

    pub fn box_bottom_right() -> char {
        '┘'
    }

    /// Get a flicker effect (randomly returns true/false for subtle text flicker)
    pub fn should_flicker(intensity: f32) -> bool {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        rng.gen::<f32>() < intensity
    }

    /// Get terminal flicker style (slightly dimmed)
    pub fn flicker_style() -> Style {
        Style::default().fg(PipBoyTheme::PHOSPHOR_DIM)
    }
}
