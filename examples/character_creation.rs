//! Showcase the character builder and SPECIAL system
//!
//! This example demonstrates:
//! - Interactive SPECIAL point allocation
//! - Skill calculations based on SPECIAL stats
//! - Character sheet display with formatting
//! - Comparing different character builds
//!
//! Run: cargo run --example character_creation

use fallout_dnd::game::character::{Character, Special};

fn main() {
    println!("=== Fallout D&D - Character Creation Example ===\n");

    // Example 1: Balanced Character
    println!("▼ BUILD 1: Balanced Survivor");
    let balanced = create_balanced_character();
    display_character_sheet(&balanced);

    // Example 2: Combat Specialist
    println!("\n▼ BUILD 2: Combat Specialist");
    let combat = create_combat_specialist();
    display_character_sheet(&combat);

    // Example 3: Social Character
    println!("\n▼ BUILD 3: Silver Tongue");
    let social = create_social_character();
    display_character_sheet(&social);

    // Example 4: Tech Expert
    println!("\n▼ BUILD 4: Tech Savant");
    let tech = create_tech_expert();
    display_character_sheet(&tech);

    // Compare builds
    println!("\n=== BUILD COMPARISON ===\n");
    compare_builds(&[
        ("Balanced", &balanced),
        ("Combat", &combat),
        ("Social", &social),
        ("Tech", &tech),
    ]);
}

fn create_balanced_character() -> Character {
    let mut special = Special::new();
    special.strength = 5;
    special.perception = 5;
    special.endurance = 5;
    special.charisma = 5;
    special.intelligence = 5;
    special.agility = 5;
    special.luck = 5;

    Character::new("Balanced Survivor", special)
}

fn create_combat_specialist() -> Character {
    let mut special = Special::new();
    special.strength = 7; // High melee damage
    special.perception = 6; // Better accuracy
    special.endurance = 7; // More HP
    special.charisma = 3; // Low
    special.intelligence = 4; // Low
    special.agility = 8; // High AP and gun skills
    special.luck = 5; // Average

    Character::new("Wasteland Warrior", special)
}

fn create_social_character() -> Character {
    let mut special = Special::new();
    special.strength = 3; // Low
    special.perception = 5; // Average
    special.endurance = 4; // Low HP
    special.charisma = 9; // Excellent speech/barter
    special.intelligence = 7; // Good skills
    special.agility = 5; // Average
    special.luck = 7; // Good fortune

    Character::new("Silver Tongue", special)
}

fn create_tech_expert() -> Character {
    let mut special = Special::new();
    special.strength = 3; // Low
    special.perception = 7; // Good for lockpick/traps
    special.endurance = 4; // Low HP
    special.charisma = 4; // Low
    special.intelligence = 10; // Maximum science/repair
    special.agility = 6; // Decent
    special.luck = 6; // Decent

    Character::new("Tech Savant", special)
}

fn display_character_sheet(character: &Character) {
    println!("═══════════════════════════════════════════");
    println!("  Name: {}", character.name);
    println!(
        "  Level: {} | XP: {}",
        character.level, character.experience
    );
    println!("═══════════════════════════════════════════");

    println!("\n┌─ SPECIAL STATS ─────────────────────────┐");
    println!(
        "│ Strength:     {:2}  │  Charisma:    {:2}  │",
        character.special.strength, character.special.charisma
    );
    println!(
        "│ Perception:   {:2}  │  Intelligence:{:2}  │",
        character.special.perception, character.special.intelligence
    );
    println!(
        "│ Endurance:    {:2}  │  Agility:     {:2}  │",
        character.special.endurance, character.special.agility
    );
    println!(
        "│ Luck:         {:2}  │                   │",
        character.special.luck
    );
    println!("└──────────────────────────────────────────┘");

    println!("\n┌─ DERIVED STATS ─────────────────────────┐");
    println!(
        "│ Hit Points:    {:3}/{:3}               │",
        character.current_hp, character.max_hp
    );
    println!(
        "│ Action Points: {:3}/{:3}               │",
        character.current_ap, character.max_ap
    );
    println!("│ Caps:          {:5}                   │", character.caps);
    println!("└──────────────────────────────────────────┘");

    println!("\n┌─ COMBAT SKILLS ─────────────────────────┐");
    println!(
        "│ Small Guns:       {:3}%                │",
        character.skills.small_guns
    );
    println!(
        "│ Big Guns:         {:3}%                │",
        character.skills.big_guns
    );
    println!(
        "│ Energy Weapons:   {:3}%                │",
        character.skills.energy_weapons
    );
    println!(
        "│ Unarmed:          {:3}%                │",
        character.skills.unarmed
    );
    println!(
        "│ Melee Weapons:    {:3}%                │",
        character.skills.melee_weapons
    );
    println!("└──────────────────────────────────────────┘");

    println!("\n┌─ ACTIVE SKILLS ─────────────────────────┐");
    println!(
        "│ Sneak:            {:3}%                │",
        character.skills.sneak
    );
    println!(
        "│ Lockpick:         {:3}%                │",
        character.skills.lockpick
    );
    println!(
        "│ Steal:            {:3}%                │",
        character.skills.steal
    );
    println!(
        "│ Traps:            {:3}%                │",
        character.skills.traps
    );
    println!("└──────────────────────────────────────────┘");

    println!("\n┌─ KNOWLEDGE SKILLS ──────────────────────┐");
    println!(
        "│ Science:          {:3}%                │",
        character.skills.science
    );
    println!(
        "│ Repair:           {:3}%                │",
        character.skills.repair
    );
    println!(
        "│ First Aid:        {:3}%                │",
        character.skills.first_aid
    );
    println!(
        "│ Doctor:           {:3}%                │",
        character.skills.doctor
    );
    println!("└──────────────────────────────────────────┘");

    println!("\n┌─ SOCIAL SKILLS ─────────────────────────┐");
    println!(
        "│ Speech:           {:3}%                │",
        character.skills.speech
    );
    println!(
        "│ Barter:           {:3}%                │",
        character.skills.barter
    );
    println!("└──────────────────────────────────────────┘");

    println!("\n┌─ OTHER SKILLS ──────────────────────────┐");
    println!(
        "│ Gambling:         {:3}%                │",
        character.skills.gambling
    );
    println!(
        "│ Outdoorsman:      {:3}%                │",
        character.skills.outdoorsman
    );
    println!(
        "│ Throwing:         {:3}%                │",
        character.skills.throwing
    );
    println!("└──────────────────────────────────────────┘");

    println!("\n┌─ EQUIPMENT ─────────────────────────────┐");
    if let Some(weapon) = &character.equipped_weapon {
        println!("│ Weapon: {:<31} │", weapon);
    } else {
        println!("│ Weapon: None                            │");
    }
    if let Some(armor) = &character.equipped_armor {
        println!("│ Armor:  {:<31} │", armor);
    } else {
        println!("│ Armor:  None                            │");
    }
    println!("└──────────────────────────────────────────┘");
}

fn compare_builds(builds: &[(&str, &Character)]) {
    println!("Attribute Comparison:");
    println!("─────────────────────────────────────────────────────────");
    println!(
        "{:<15} {:>8} {:>8} {:>8} {:>8}",
        "Stat", builds[0].0, builds[1].0, builds[2].0, builds[3].0
    );
    println!("─────────────────────────────────────────────────────────");

    println!(
        "{:<15} {:>8} {:>8} {:>8} {:>8}",
        "Max HP", builds[0].1.max_hp, builds[1].1.max_hp, builds[2].1.max_hp, builds[3].1.max_hp
    );
    println!(
        "{:<15} {:>8} {:>8} {:>8} {:>8}",
        "Max AP", builds[0].1.max_ap, builds[1].1.max_ap, builds[2].1.max_ap, builds[3].1.max_ap
    );

    println!("\nKey Skill Comparison:");
    println!("─────────────────────────────────────────────────────────");
    println!(
        "{:<15} {:>8} {:>8} {:>8} {:>8}",
        "Skill", builds[0].0, builds[1].0, builds[2].0, builds[3].0
    );
    println!("─────────────────────────────────────────────────────────");

    println!(
        "{:<15} {:>7}% {:>7}% {:>7}% {:>7}%",
        "Small Guns",
        builds[0].1.skills.small_guns,
        builds[1].1.skills.small_guns,
        builds[2].1.skills.small_guns,
        builds[3].1.skills.small_guns
    );

    println!(
        "{:<15} {:>7}% {:>7}% {:>7}% {:>7}%",
        "Speech",
        builds[0].1.skills.speech,
        builds[1].1.skills.speech,
        builds[2].1.skills.speech,
        builds[3].1.skills.speech
    );

    println!(
        "{:<15} {:>7}% {:>7}% {:>7}% {:>7}%",
        "Science",
        builds[0].1.skills.science,
        builds[1].1.skills.science,
        builds[2].1.skills.science,
        builds[3].1.skills.science
    );

    println!(
        "{:<15} {:>7}% {:>7}% {:>7}% {:>7}%",
        "Lockpick",
        builds[0].1.skills.lockpick,
        builds[1].1.skills.lockpick,
        builds[2].1.skills.lockpick,
        builds[3].1.skills.lockpick
    );

    println!(
        "{:<15} {:>7}% {:>7}% {:>7}% {:>7}%",
        "Unarmed",
        builds[0].1.skills.unarmed,
        builds[1].1.skills.unarmed,
        builds[2].1.skills.unarmed,
        builds[3].1.skills.unarmed
    );

    println!("─────────────────────────────────────────────────────────");
}
