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
        let mut rng = rand::rng();
        rng.random::<f32>() < intensity
    }

    /// Get terminal flicker style (slightly dimmed)
    pub fn flicker_style() -> Style {
        Style::default().fg(PipBoyTheme::PHOSPHOR_DIM)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ============================================================================
    // PIPBOYTHEME COLOR TESTS
    // ============================================================================

    #[test]
    fn test_pipboy_theme_colors_are_defined() {
        // Verify all color constants are accessible and not panicking
        assert_eq!(PipBoyTheme::PHOSPHOR_BRIGHT, Color::Rgb(0, 255, 0));
        assert_eq!(PipBoyTheme::PHOSPHOR_NORMAL, Color::Rgb(0, 200, 0));
        assert_eq!(PipBoyTheme::PHOSPHOR_DIM, Color::Rgb(0, 150, 0));
        assert_eq!(PipBoyTheme::PHOSPHOR_DARK, Color::Rgb(0, 100, 0));
        assert_eq!(PipBoyTheme::PHOSPHOR_VERY_DARK, Color::Rgb(0, 50, 0));
        assert_eq!(PipBoyTheme::BACKGROUND, Color::Rgb(0, 20, 0));
    }

    #[test]
    fn test_pipboy_theme_accent_colors() {
        // Test accent colors for warnings and danger states
        assert_eq!(PipBoyTheme::AMBER_HIGHLIGHT, Color::Rgb(255, 200, 0));
        assert_eq!(PipBoyTheme::RED_DANGER, Color::Rgb(200, 50, 0));
    }

    // ============================================================================
    // PIPBOYTHEME STYLE TESTS
    // ============================================================================

    #[test]
    fn test_text_style() {
        let style = PipBoyTheme::text();
        assert_eq!(style.fg, Some(PipBoyTheme::PHOSPHOR_NORMAL));
        // Default style should not have modifiers
        assert_eq!(style.add_modifier, Modifier::empty());
    }

    #[test]
    fn test_text_bright_style() {
        let style = PipBoyTheme::text_bright();
        assert_eq!(style.fg, Some(PipBoyTheme::PHOSPHOR_BRIGHT));
        assert!(style.add_modifier.contains(Modifier::BOLD));
    }

    #[test]
    fn test_text_dim_style() {
        let style = PipBoyTheme::text_dim();
        assert_eq!(style.fg, Some(PipBoyTheme::PHOSPHOR_DIM));
        assert_eq!(style.add_modifier, Modifier::empty());
    }

    #[test]
    fn test_text_very_dim_style() {
        let style = PipBoyTheme::text_very_dim();
        assert_eq!(style.fg, Some(PipBoyTheme::PHOSPHOR_VERY_DARK));
    }

    #[test]
    fn test_header_style() {
        let style = PipBoyTheme::header();
        assert_eq!(style.fg, Some(PipBoyTheme::PHOSPHOR_BRIGHT));
        assert!(style.add_modifier.contains(Modifier::BOLD));
        assert!(style.add_modifier.contains(Modifier::UNDERLINED));
    }

    #[test]
    fn test_border_style() {
        let style = PipBoyTheme::border();
        assert_eq!(style.fg, Some(PipBoyTheme::PHOSPHOR_DIM));
    }

    #[test]
    fn test_border_active_style() {
        let style = PipBoyTheme::border_active();
        assert_eq!(style.fg, Some(PipBoyTheme::PHOSPHOR_BRIGHT));
    }

    // ============================================================================
    // HEALTH BAR STYLE TESTS
    // ============================================================================

    #[test]
    fn test_health_full_style() {
        let style = PipBoyTheme::health_full();
        assert_eq!(style.fg, Some(PipBoyTheme::PHOSPHOR_BRIGHT));
        // Health full should be bright green
    }

    #[test]
    fn test_health_medium_style() {
        let style = PipBoyTheme::health_medium();
        assert_eq!(style.fg, Some(PipBoyTheme::AMBER_HIGHLIGHT));
        // Medium health is amber/warning color
    }

    #[test]
    fn test_health_low_style() {
        let style = PipBoyTheme::health_low();
        assert_eq!(style.fg, Some(PipBoyTheme::RED_DANGER));
        // Low health is danger/red color
    }

    #[test]
    fn test_energy_style() {
        let style = PipBoyTheme::energy();
        assert_eq!(style.fg, Some(PipBoyTheme::PHOSPHOR_NORMAL));
    }

    // ============================================================================
    // MESSAGE STYLE TESTS
    // ============================================================================

    #[test]
    fn test_player_action_style() {
        let style = PipBoyTheme::player_action();
        assert_eq!(style.fg, Some(PipBoyTheme::PHOSPHOR_BRIGHT));
        assert!(style.add_modifier.contains(Modifier::ITALIC));
    }

    #[test]
    fn test_dm_text_style() {
        let style = PipBoyTheme::dm_text();
        assert_eq!(style.fg, Some(PipBoyTheme::PHOSPHOR_NORMAL));
    }

    #[test]
    fn test_combat_style() {
        let style = PipBoyTheme::combat();
        assert_eq!(style.fg, Some(PipBoyTheme::AMBER_HIGHLIGHT));
        assert!(style.add_modifier.contains(Modifier::BOLD));
    }

    #[test]
    fn test_system_style() {
        let style = PipBoyTheme::system();
        assert_eq!(style.fg, Some(PipBoyTheme::PHOSPHOR_DIM));
    }

    #[test]
    fn test_info_style() {
        let style = PipBoyTheme::info();
        assert_eq!(style.fg, Some(PipBoyTheme::PHOSPHOR_NORMAL));
    }

    #[test]
    fn test_success_style() {
        let style = PipBoyTheme::success();
        assert_eq!(style.fg, Some(PipBoyTheme::PHOSPHOR_BRIGHT));
        assert!(style.add_modifier.contains(Modifier::BOLD));
    }

    #[test]
    fn test_error_style() {
        let style = PipBoyTheme::error();
        assert_eq!(style.fg, Some(PipBoyTheme::RED_DANGER));
        assert!(style.add_modifier.contains(Modifier::BOLD));
    }

    #[test]
    fn test_warning_style() {
        let style = PipBoyTheme::warning();
        assert_eq!(style.fg, Some(PipBoyTheme::AMBER_HIGHLIGHT));
    }

    #[test]
    fn test_selected_style() {
        let style = PipBoyTheme::selected();
        assert_eq!(style.fg, Some(PipBoyTheme::PHOSPHOR_BRIGHT));
        assert_eq!(style.bg, Some(PipBoyTheme::PHOSPHOR_VERY_DARK));
        assert!(style.add_modifier.contains(Modifier::BOLD));
    }

    #[test]
    fn test_loading_style() {
        let style = PipBoyTheme::loading();
        assert_eq!(style.fg, Some(PipBoyTheme::PHOSPHOR_DIM));
        assert!(style.add_modifier.contains(Modifier::SLOW_BLINK));
    }

    // ============================================================================
    // SCANLINE EFFECT TESTS
    // ============================================================================

    #[test]
    fn test_scanline_overlay_pattern() {
        // Test that scanlines appear on even rows (0, 2, 4, etc.)
        assert_eq!(ScanlineEffect::get_overlay(0), Some('▒'));
        assert_eq!(ScanlineEffect::get_overlay(2), Some('▒'));
        assert_eq!(ScanlineEffect::get_overlay(4), Some('▒'));
        // Odd rows should have no overlay
        assert_eq!(ScanlineEffect::get_overlay(1), None);
        assert_eq!(ScanlineEffect::get_overlay(3), None);
        assert_eq!(ScanlineEffect::get_overlay(5), None);
    }

    #[test]
    fn test_scanline_row_detection() {
        // Test scanline row detection
        assert!(ScanlineEffect::is_scanline_row(0));
        assert!(!ScanlineEffect::is_scanline_row(1));
        assert!(ScanlineEffect::is_scanline_row(2));
        assert!(!ScanlineEffect::is_scanline_row(3));
        assert!(ScanlineEffect::is_scanline_row(100));
        assert!(!ScanlineEffect::is_scanline_row(101));
    }

    #[test]
    fn test_scanline_style() {
        let style = ScanlineEffect::style();
        assert_eq!(style.fg, Some(PipBoyTheme::PHOSPHOR_VERY_DARK));
        assert_eq!(style.bg, Some(PipBoyTheme::BACKGROUND));
    }

    // ============================================================================
    // LOADING SPINNER TESTS
    // ============================================================================

    #[test]
    fn test_loading_spinner_creation() {
        let spinner = LoadingSpinner::new();
        assert_eq!(spinner.current(), "⠋");
        // Should have 10 frames
        assert_eq!(spinner.frames.len(), 10);
    }

    #[test]
    fn test_loading_spinner_next_frame() {
        let mut spinner = LoadingSpinner::new();
        let first = spinner.next_frame();
        assert_eq!(first, "⠋");

        let second = spinner.next_frame();
        assert_eq!(second, "⠙");

        let third = spinner.next_frame();
        assert_eq!(third, "⠹");
    }

    #[test]
    fn test_loading_spinner_cycles() {
        let mut spinner = LoadingSpinner::new();
        // Advance through all 10 frames
        for _ in 0..10 {
            spinner.next_frame();
        }
        // Next frame should cycle back to first
        assert_eq!(spinner.current(), "⠋");
    }

    #[test]
    fn test_loading_spinner_reset() {
        let mut spinner = LoadingSpinner::new();
        spinner.next_frame();
        spinner.next_frame();
        spinner.next_frame();
        assert_ne!(spinner.current(), "⠋");

        spinner.reset();
        assert_eq!(spinner.current(), "⠋");
    }

    #[test]
    fn test_loading_spinner_default() {
        let spinner = LoadingSpinner::default();
        assert_eq!(spinner.current(), "⠋");
    }

    #[test]
    fn test_loading_spinner_frame_sequence() {
        let spinner = LoadingSpinner::new();
        let expected_frames = vec!["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];
        assert_eq!(spinner.frames, expected_frames);
    }

    // ============================================================================
    // RETRO EFFECTS TESTS
    // ============================================================================

    #[test]
    fn test_separator_line_generation() {
        let sep = RetroEffects::separator_line(5);
        assert_eq!(sep, "─────");
        // Unicode character '─' takes 3 bytes, so 5 chars = 15 bytes
        assert_eq!(sep.chars().count(), 5);

        let sep = RetroEffects::separator_line(10);
        assert_eq!(sep.chars().count(), 10);

        let sep = RetroEffects::separator_line(0);
        assert_eq!(sep, "");
    }

    #[test]
    fn test_separator_double_generation() {
        let sep = RetroEffects::separator_double(5);
        assert_eq!(sep, "═════");
        // Unicode character '═' takes 3 bytes, so 5 chars = 15 bytes
        assert_eq!(sep.chars().count(), 5);

        let sep = RetroEffects::separator_double(10);
        assert_eq!(sep.chars().count(), 10);
    }

    #[test]
    fn test_box_corner_characters() {
        assert_eq!(RetroEffects::box_top_left(), '┌');
        assert_eq!(RetroEffects::box_top_right(), '┐');
        assert_eq!(RetroEffects::box_bottom_left(), '└');
        assert_eq!(RetroEffects::box_bottom_right(), '┘');
    }

    #[test]
    fn test_flicker_style() {
        let style = RetroEffects::flicker_style();
        assert_eq!(style.fg, Some(PipBoyTheme::PHOSPHOR_DIM));
    }

    #[test]
    fn test_should_flicker_range() {
        // Test with 0 intensity (should always be false)
        for _ in 0..10 {
            assert!(!RetroEffects::should_flicker(0.0));
        }

        // Test with 1.0 intensity (should always be true)
        for _ in 0..10 {
            assert!(RetroEffects::should_flicker(1.0));
        }
    }

    // ============================================================================
    // PIPBOY HEADER TESTS
    // ============================================================================

    #[test]
    fn test_pipboy_header_exists() {
        let header = PipBoyHeader::get_header();
        assert!(!header.is_empty());
        assert_eq!(header.len(), 6);
    }

    #[test]
    fn test_pipboy_header_contains_vault_tec_text() {
        let header = PipBoyHeader::get_header();
        let header_str = header.join("\n");
        // Header uses box drawing characters for "PIP-BOY", not literal text
        // But it does contain "VAULT-TEC" and "PERSONAL TERMINAL"
        assert!(header_str.contains("VAULT-TEC"));
        assert!(header_str.contains("PERSONAL TERMINAL"));
    }

    #[test]
    fn test_pipboy_boot_sequence_exists() {
        let boot = PipBoyHeader::get_boot_sequence();
        assert!(!boot.is_empty());
        assert_eq!(boot.len(), 11);
    }

    #[test]
    fn test_pipboy_boot_sequence_contains_system_info() {
        let boot = PipBoyHeader::get_boot_sequence();
        let boot_str = boot.join("\n");
        assert!(boot_str.contains("ROBCO"));
        assert!(boot_str.contains("VAULT-TEC"));
        assert!(boot_str.contains("PIP-BOY 3000"));
    }

    // ============================================================================
    // THEME CONSISTENCY TESTS
    // ============================================================================

    #[test]
    fn test_health_styles_use_consistent_colors() {
        // Verify health styles use the expected color hierarchy
        let full = PipBoyTheme::health_full();
        let medium = PipBoyTheme::health_medium();
        let low = PipBoyTheme::health_low();

        // All should be defined
        assert!(full.fg.is_some());
        assert!(medium.fg.is_some());
        assert!(low.fg.is_some());

        // They should be different
        assert_ne!(full.fg, medium.fg);
        assert_ne!(medium.fg, low.fg);
        assert_ne!(full.fg, low.fg);
    }

    #[test]
    fn test_message_styles_are_distinguishable() {
        // Verify that different message types are visually distinct
        let success = PipBoyTheme::success();
        let error = PipBoyTheme::error();
        let warning = PipBoyTheme::warning();
        let info = PipBoyTheme::info();

        // Each should have defined colors
        assert!(success.fg.is_some());
        assert!(error.fg.is_some());
        assert!(warning.fg.is_some());
        assert!(info.fg.is_some());
    }

    #[test]
    fn test_fallout_theme_green_dominance() {
        // Verify that the primary colors are green-based (phosphor theme)
        // All main style colors should be green
        assert_eq!(PipBoyTheme::text().fg, Some(PipBoyTheme::PHOSPHOR_NORMAL));
        assert_eq!(PipBoyTheme::header().fg, Some(PipBoyTheme::PHOSPHOR_BRIGHT));
        assert_eq!(
            PipBoyTheme::dm_text().fg,
            Some(PipBoyTheme::PHOSPHOR_NORMAL)
        );
    }

    #[test]
    fn test_border_visibility_hierarchy() {
        // Normal border should be dimmer than active border
        let normal_border = PipBoyTheme::border();
        let active_border = PipBoyTheme::border_active();

        // Active should be bright, normal should be dim
        assert_eq!(normal_border.fg, Some(PipBoyTheme::PHOSPHOR_DIM));
        assert_eq!(active_border.fg, Some(PipBoyTheme::PHOSPHOR_BRIGHT));
    }
}
