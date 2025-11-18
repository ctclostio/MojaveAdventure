//! # UI Module
//!
//! Terminal user interface and display utilities for the game.
//!
//! ## Overview
//!
//! This module provides all terminal UI functionality including:
//! - Colored output with ANSI escape codes
//! - Character sheet display with borders and formatting
//! - Combat status visualization with health bars
//! - Inventory management display
//! - DM response formatting with word wrapping
//! - User input prompts
//!
//! ## Design Philosophy
//!
//! The UI is designed to be:
//! - **Retro Terminal Aesthetic**: Uses box-drawing characters and colored text
//! - **Fallout-Themed**: Green and cyan colors reminiscent of Pip-Boy terminals
//! - **Responsive**: Word wrapping adapts to terminal width
//! - **Clear Information Hierarchy**: Different colors for different types of info
//!
//! ## Color Scheme
//!
//! - **Green**: Headers, DM responses, success messages
//! - **Cyan**: Character sheet borders and info
//! - **Red**: Combat, health warnings, errors
//! - **Yellow**: Action points, stat values
//! - **Blue**: Prompts, info messages
//!
//! ## Example
//!
//! ```no_run
//! use fallout_dnd::ui::UI;
//! use fallout_dnd::game::character::Character;
//!
//! let character = Character::new("Vault Dweller".to_string());
//!
//! UI::clear_screen();
//! UI::print_header();
//! UI::print_character_sheet(&character);
//! let input = UI::prompt("What do you do?");
//! ```

use colored::*;
use crossterm::{
    execute,
    terminal::{Clear, ClearType},
};
use std::io::{stdout, Write};

/// Terminal UI utilities for displaying game information.
///
/// All methods are static and organized around different display contexts:
/// - Screen management (clear, header)
/// - Character information (stats, inventory)
/// - Combat status (enemies, health bars)
/// - User interaction (prompts, messages)
pub struct UI;

impl UI {
    pub fn clear_screen() {
        execute!(stdout(), Clear(ClearType::All)).ok();
        execute!(stdout(), crossterm::cursor::MoveTo(0, 0)).ok();
    }

    pub fn print_header() {
        println!("{}", "═══════════════════════════════════════════════════════════════".green().bold());
        println!("{}", "    FALLOUT: WASTELAND ADVENTURES - AI Dungeon Master".green().bold());
        println!("{}", "═══════════════════════════════════════════════════════════════".green().bold());
        println!();
    }

    pub fn print_character_sheet(character: &crate::game::character::Character) {
        println!("{}", "╔════════════════════════════════════════╗".cyan());
        println!("{} {:<36} {}", "║".cyan(), format!("{} (Level {})", character.name, character.level).bold(), "║".cyan());
        println!("{}", "╠════════════════════════════════════════╣".cyan());

        println!("{} {:<36} {}", "║".cyan(), format!("HP: {}/{}", character.current_hp, character.max_hp).red(), "║".cyan());
        println!("{} {:<36} {}", "║".cyan(), format!("AP: {}/{}", character.current_ap, character.max_ap).yellow(), "║".cyan());
        println!("{} {:<36} {}", "║".cyan(), format!("XP: {} / {}", character.experience, (character.level + 1) * 1000).white(), "║".cyan());
        println!("{} {:<36} {}", "║".cyan(), format!("Caps: {}", character.caps).green(), "║".cyan());

        println!("{}", "╠════════════════════════════════════════╣".cyan());
        println!("{} {:<36} {}", "║".cyan(), "SPECIAL:".bold(), "║".cyan());
        println!("{} S:{} P:{} E:{} C:{} I:{} A:{} L:{} {}",
            "║".cyan(),
            format!("{:2}", character.special.strength).yellow(),
            format!("{:2}", character.special.perception).yellow(),
            format!("{:2}", character.special.endurance).yellow(),
            format!("{:2}", character.special.charisma).yellow(),
            format!("{:2}", character.special.intelligence).yellow(),
            format!("{:2}", character.special.agility).yellow(),
            format!("{:2}", character.special.luck).yellow(),
            "║".cyan()
        );
        println!("{}", "╚════════════════════════════════════════╝".cyan());
        println!();
    }

    pub fn print_dm_response(text: &str) {
        let term_size = crossterm::terminal::size().unwrap_or((80u16, 24u16));
        let term_width = term_size.0 as usize;
        let padding = 5usize;
        let max_width = term_width.saturating_sub(padding).min(70);

        // Simple word wrap ignoring original newlines for DM narrative flow
        let text_no_nl = text.replace('\n', " ");
        let words: Vec<&str> = text_no_nl.split_whitespace().collect();
        let mut wrapped_lines: Vec<String> = Vec::new();
        let mut current_line = String::new();

        for &word in &words {
            let test_line = if current_line.is_empty() {
                word.to_string()
            } else {
                format!("{} {}", current_line, word)
            };
            if test_line.chars().count() <= max_width {
                current_line = test_line;
            } else {
                if !current_line.is_empty() {
                    wrapped_lines.push(std::mem::take(&mut current_line));
                }
                current_line = word.to_string();
            }
        }
        if !current_line.is_empty() {
            wrapped_lines.push(current_line);
        }

        println!("{}", "┌─ DUNGEON MASTER ─────────────────────────".bright_green());
        for line in &wrapped_lines {
            println!("{} {}", "│".bright_green(), line);
        }
        println!("{}", "└──────────────────────────────────────────".bright_green());
        println!();
    }

    pub fn print_combat_status(combat: &crate::game::combat::CombatState) {
        if !combat.active {
            return;
        }

        println!("{}", format!("⚔  COMBAT - Round {} ⚔", combat.round).red().bold());
        println!();
        for (i, enemy) in combat.enemies.iter().enumerate() {
            if enemy.is_alive() {
                let health_bar = Self::health_bar(enemy.current_hp, enemy.max_hp);
                println!("  [{}] {} - HP: {} {}",
                    i + 1,
                    enemy.name.red(),
                    health_bar,
                    format!("({}/{})", enemy.current_hp, enemy.max_hp)
                );
            } else {
                println!("  [{}] {} - {}",
                    i + 1,
                    enemy.name.dimmed(),
                    "DEAD".red().bold()
                );
            }
        }
        println!();
    }

    fn health_bar(current: i32, max: i32) -> String {
        let percentage = (current as f32 / max as f32 * 100.0) as i32;
        let bars = (percentage / 10).min(10);
        let empty = 10 - bars;

        let filled = "█".repeat(bars as usize);
        let blank = "░".repeat(empty as usize);

        if percentage > 60 {
            format!("{}{}", filled.green(), blank.dimmed())
        } else if percentage > 30 {
            format!("{}{}", filled.yellow(), blank.dimmed())
        } else {
            format!("{}{}", filled.red(), blank.dimmed())
        }
    }

    pub fn print_inventory(items: &[crate::game::items::Item]) {
        println!("{}", "╔═══ INVENTORY ═══════════════════════════╗".cyan());
        if items.is_empty() {
            println!("{} {:<36} {}", "║".cyan(), "Empty".dimmed(), "║".cyan());
        } else {
            for (i, item) in items.iter().enumerate() {
                println!("{} [{}] {:<32} {}", "║".cyan(), i + 1, item.name, "║".cyan());
            }
        }
        println!("{}", "╚════════════════════════════════════════╝".cyan());
        println!();
    }

    pub fn prompt(message: &str) -> String {
        print!("{} ", message.bright_blue().bold());

        if let Err(e) = stdout().flush() {
            eprintln!("Fatal: Cannot flush stdout: {}", e);
            std::process::exit(1);
        }

        let mut input = String::new();
        if let Err(e) = std::io::stdin().read_line(&mut input) {
            eprintln!("Fatal: Cannot read from stdin: {}", e);
            std::process::exit(1);
        }

        input.trim().to_string()
    }

    pub fn print_error(message: &str) {
        println!("{} {}", "ERROR:".red().bold(), message);
        println!();
    }

    pub fn print_success(message: &str) {
        println!("{} {}", "✓".green().bold(), message);
        println!();
    }

    pub fn print_info(message: &str) {
        println!("{} {}", "ℹ".blue().bold(), message);
        println!();
    }

    pub fn wait_for_enter() {
        println!();
        Self::prompt("Press Enter to continue...");
    }
}
