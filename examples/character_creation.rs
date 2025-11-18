//! Character Creation Example
//!
//! Demonstrates how to create a character with SPECIAL stats and skills.
//!
//! Run with: `cargo run --example character_creation`

use fallout_dnd::game::character::{Character, Special};
use fallout_dnd::validation;

fn main() {
    println!("=== Fallout D&D Character Creation Example ===\n");

    // Create a new character with SPECIAL stats
    let mut special = Special::new();

    // Distribute SPECIAL points (total of 28 points)
    special.strength = 6;
    special.perception = 5;
    special.endurance = 6;
    special.charisma = 3;
    special.intelligence = 8;
    special.agility = 5;
    special.luck = 5;

    println!("SPECIAL Stats:");
    println!("  Strength:     {}", special.strength);
    println!("  Perception:   {}", special.perception);
    println!("  Endurance:    {}", special.endurance);
    println!("  Charisma:     {}", special.charisma);
    println!("  Intelligence: {}", special.intelligence);
    println!("  Agility:      {}", special.agility);
    println!("  Luck:         {}", special.luck);
    println!("  Total:        {}\n", special.total_points());

    // Validate character name
    let char_name = "Vault Dweller";
    match validation::validate_character_name(char_name) {
        Ok(_) => println!("✓ Character name '{}' is valid\n", char_name),
        Err(e) => println!("✗ Invalid name: {}\n", e),
    }

    // Create the character
    let mut character = Character::new(char_name.to_string(), special);

    println!("Character Created!");
    println!("  Name:     {}", character.name);
    println!("  Level:    {}", character.level);
    println!("  HP:       {}/{}", character.current_hp, character.max_hp);
    println!("  AP:       {}/{}", character.current_ap, character.max_ap);
    println!("  Caps:     {}", character.caps);
    println!();

    println!("Derived Skills:");
    println!("  Small Guns:     {}", character.skills.small_guns);
    println!("  Science:        {}", character.skills.science);
    println!("  Speech:         {}", character.skills.speech);
    println!("  Lockpick:       {}", character.skills.lockpick);
    println!("  Repair:         {}", character.skills.repair);
    println!();

    // Add some experience
    character.add_experience(500);
    println!("Added 500 XP. Current XP: {}", character.experience);

    // Level up if possible
    if character.can_level_up() {
        character.level_up();
        println!("Leveled up! New level: {}", character.level);
        println!("  HP: {}/{}", character.current_hp, character.max_hp);
    }

    println!("\n=== Example Complete ===");
}
