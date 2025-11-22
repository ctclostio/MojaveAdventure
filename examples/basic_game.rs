//! Minimal example showing how to start a game programmatically
//!
//! This example demonstrates:
//! - Creating a character with SPECIAL stats
//! - Initializing a game state
//! - Accessing basic game information
//!
//! Run: cargo run --example basic_game

use fallout_dnd::game::character::{Character, Special};
use fallout_dnd::game::GameState;

fn main() {
    println!("=== Fallout D&D - Basic Game Example ===\n");

    // Create SPECIAL attributes for our character
    let mut special = Special::new();
    special.strength = 6;
    special.perception = 5;
    special.endurance = 5;
    special.charisma = 6;
    special.intelligence = 7;
    special.agility = 5;
    special.luck = 6;

    println!("Creating character with SPECIAL stats:");
    println!(
        "  S: {} | P: {} | E: {} | C: {}",
        special.strength, special.perception, special.endurance, special.charisma
    );
    println!(
        "  I: {} | A: {} | L: {}",
        special.intelligence, special.agility, special.luck
    );
    println!();

    // Create a character
    let character = Character::new("Vault Dweller", special);

    println!("Character created: {}", character.name);
    println!("Level: {}", character.level);
    println!("HP: {}/{}", character.current_hp, character.max_hp);
    println!("AP: {}/{}", character.current_ap, character.max_ap);
    println!("Caps: {}", character.caps);
    println!();

    // Show some derived skills
    println!("Key Skills:");
    println!("  Small Guns: {}", character.skills.small_guns);
    println!("  Speech: {}", character.skills.speech);
    println!("  Science: {}", character.skills.science);
    println!("  Lockpick: {}", character.skills.lockpick);
    println!("  Sneak: {}", character.skills.sneak);
    println!();

    // Show starting inventory
    println!("Starting Inventory:");
    for item in &character.inventory {
        println!("  - {} (x{})", item.name, item.quantity);
    }
    println!();

    // Create game state
    let game_state = GameState::new(character);

    println!("Game initialized!");
    println!("Starting Location: {}", game_state.location);
    println!("Current Day: {}", game_state.day);
    println!("Active Quests:");
    for quest in &game_state.quest_log {
        println!("  - {}", quest);
    }
    println!();

    // Show worldbook info
    println!("Worldbook Status:");
    println!(
        "  Known Locations: {}",
        game_state.worldbook.locations.len()
    );
    println!("  Known NPCs: {}", game_state.worldbook.npcs.len());
    println!("  Recorded Events: {}", game_state.worldbook.events.len());
    println!();

    println!("Ready to explore the wasteland!");
}
