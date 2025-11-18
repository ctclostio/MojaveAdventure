use super::character::{Character, Special};
use super::GameState;
use super::stat_allocator::allocate_stats_interactive;
use crate::ui::UI;
use crate::config::Config;
use colored::*;

/// Create a new character through interactive character creation
pub fn create_new_character(_config: &Config) -> GameState {
    UI::clear_screen();
    UI::print_header();

    println!("{}", "CHARACTER CREATION".bold().green());
    println!();

    let name = UI::prompt("Enter your character's name:");

    println!();
    println!("{}", "Prepare for SPECIAL allocation...".yellow());
    println!("Press any key to continue...");
    let _ = UI::prompt("");

    // Use the new interactive stat allocator
    let special = match allocate_stats_interactive() {
        Ok(stats) => stats,
        Err(e) => {
            UI::print_error(&format!("Error during stat allocation: {}", e));
            Special::new() // Fall back to default stats
        }
    };

    let character = Character::new(name, special);
    UI::clear_screen();
    UI::print_success("Character created!");
    UI::print_character_sheet(&character);
    UI::wait_for_enter();

    GameState::new(character)
}

/// Print detailed character statistics
pub fn print_detailed_stats(character: &Character) {
    println!("{}", "═══ DETAILED STATS ═════════════════════════".cyan().bold());
    println!();
    println!("{}", "SKILLS:".bold());
    println!("  Small Guns: {}", character.skills.small_guns);
    println!("  Big Guns: {}", character.skills.big_guns);
    println!("  Energy Weapons: {}", character.skills.energy_weapons);
    println!("  Melee Weapons: {}", character.skills.melee_weapons);
    println!("  Unarmed: {}", character.skills.unarmed);
    println!("  Speech: {}", character.skills.speech);
    println!("  Sneak: {}", character.skills.sneak);
    println!("  Lockpick: {}", character.skills.lockpick);
    println!("  Science: {}", character.skills.science);
    println!("  Repair: {}", character.skills.repair);
    println!();
}
