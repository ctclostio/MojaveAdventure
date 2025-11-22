//! Show worldbook extraction and context building
//!
//! This example demonstrates:
//! - Creating and populating a worldbook
//! - Adding locations, NPCs, and events
//! - Building context for AI prompts
//! - Querying worldbook data
//! - JSON serialization/deserialization
//!
//! Run: cargo run --example worldbook_demo

use colored::Colorize;
use fallout_dnd::game::worldbook::{Location, WorldEvent, Worldbook, NPC};
use smartstring::alias::String as SmartString;
use std::collections::HashMap;

fn main() {
    println!("{}", "=== Fallout D&D - Worldbook Demo ===\n".bold().cyan());

    // Create a new worldbook with defaults
    let mut worldbook = Worldbook::with_defaults();

    println!("{}", "Initial Worldbook State:".bold());
    println!("  Known Locations: {}", worldbook.locations.len());
    println!("  Known NPCs: {}", worldbook.npcs.len());
    println!("  Recorded Events: {}", worldbook.events.len());
    println!();

    // Add Megaton settlement
    println!("{}", "Adding Location: Megaton".bright_green());
    let megaton = Location {
        id: SmartString::from("megaton_01"),
        name: SmartString::from("Megaton"),
        name_lowercase: SmartString::from("megaton"),
        description: SmartString::from(
            "A fortified settlement built in a bomb crater around an undetonated atomic bomb. \
             Scrap metal walls and salvaged aircraft parts form a makeshift fortress."
        ),
        location_type: SmartString::from("settlement"),
        npcs_present: vec![SmartString::from("lucas_simms_01"), SmartString::from("moira_brown_01")],
        atmosphere: Some(SmartString::from(
            "Bustling with activity. The sound of hammering metal and traders haggling fills the air."
        )),
        first_visited: None,
        last_visited: None,
        visit_count: 0,
        notes: vec![],
        state: HashMap::new(),
    };
    worldbook.add_location(megaton);
    println!("  ✓ Added settlement 'Megaton'");
    println!();

    // Add the Wasteland location
    println!("{}", "Adding Location: Capital Wasteland".bright_green());
    let wasteland = Location {
        id: SmartString::from("capital_wasteland_01"),
        name: SmartString::from("Capital Wasteland"),
        name_lowercase: SmartString::from("capital wasteland"),
        description: SmartString::from(
            "The irradiated ruins of Washington D.C. and surrounding areas. \
             Crumbling buildings, toxic water, and dangerous creatures lurk everywhere.",
        ),
        location_type: SmartString::from("wasteland"),
        npcs_present: vec![],
        atmosphere: Some(SmartString::from(
            "Desolate and dangerous. The wind carries radioactive dust.",
        )),
        first_visited: None,
        last_visited: None,
        visit_count: 0,
        notes: vec![],
        state: HashMap::new(),
    };
    worldbook.add_location(wasteland);
    println!("  ✓ Added wasteland area");
    println!();

    // Add NPCs
    println!("{}", "Adding NPCs:".bright_cyan());

    let lucas_simms = NPC {
        id: SmartString::from("lucas_simms_01"),
        name: SmartString::from("Lucas Simms"),
        name_lowercase: SmartString::from("lucas simms"),
        role: SmartString::from("sheriff"),
        personality: vec![
            SmartString::from("stern"),
            SmartString::from("protective"),
            SmartString::from("lawful"),
        ],
        current_location: Some(SmartString::from("megaton_01")),
        disposition: 50, // Neutral-friendly
        knowledge: vec![
            SmartString::from("Runs Megaton's security"),
            SmartString::from("Concerned about the bomb in town center"),
        ],
        notes: SmartString::from("Sheriff of Megaton. Wears a duster and keeps order in town."),
        alive: true,
    };
    worldbook.add_npc(lucas_simms);
    println!(
        "  ✓ Lucas Simms (Sheriff) - Disposition: {} (Neutral-Friendly)",
        "50".bright_cyan()
    );

    let moira_brown = NPC {
        id: SmartString::from("moira_brown_01"),
        name: SmartString::from("Moira Brown"),
        name_lowercase: SmartString::from("moira brown"),
        role: SmartString::from("merchant"),
        personality: vec![
            SmartString::from("optimistic"),
            SmartString::from("curious"),
            SmartString::from("eccentric"),
        ],
        current_location: Some(SmartString::from("megaton_01")),
        disposition: 70, // Friendly
        knowledge: vec![
            SmartString::from("Runs Craterside Supply"),
            SmartString::from("Writing a wasteland survival guide"),
        ],
        notes: SmartString::from("Energetic merchant with unusual enthusiasm for the wasteland."),
        alive: true,
    };
    worldbook.add_npc(moira_brown);
    println!(
        "  ✓ Moira Brown (Merchant) - Disposition: {} (Friendly)",
        "70".bright_green()
    );

    let raider_boss = NPC {
        id: SmartString::from("gristle_01"),
        name: SmartString::from("Gristle"),
        name_lowercase: SmartString::from("gristle"),
        role: SmartString::from("raider_boss"),
        personality: vec![
            SmartString::from("violent"),
            SmartString::from("ruthless"),
            SmartString::from("greedy"),
        ],
        current_location: Some(SmartString::from("capital_wasteland_01")),
        disposition: -80, // Hostile
        knowledge: vec![SmartString::from("Leads a raider gang")],
        notes: SmartString::from("Dangerous raider leader. Attacks travelers on sight."),
        alive: true,
    };
    worldbook.add_npc(raider_boss);
    println!(
        "  ✓ Gristle (Raider Boss) - Disposition: {} (Hostile)",
        "-80".bright_red()
    );
    println!();

    // Record events
    println!("{}", "Recording Events:".bright_yellow());

    let event1 = WorldEvent {
        timestamp: SmartString::from("2277-10-23T08:00:00Z"),
        location: Some(SmartString::from("vault_13")),
        event_type: SmartString::from("discovery"),
        description: SmartString::from(
            "Emerged from Vault 13 into the wasteland for the first time.",
        ),
        entities: vec![],
    };
    worldbook.add_event(event1);
    println!("  ✓ Event 1: Emerged from Vault 13");

    let event2 = WorldEvent {
        timestamp: SmartString::from("2277-10-23T10:30:00Z"),
        location: Some(SmartString::from("megaton_01")),
        event_type: SmartString::from("npc_met"),
        description: SmartString::from(
            "Met Sheriff Lucas Simms at Megaton's gates. He welcomed us cautiously.",
        ),
        entities: vec![SmartString::from("lucas_simms_01")],
    };
    worldbook.add_event(event2);
    println!("  ✓ Event 2: Met Lucas Simms");

    let event3 = WorldEvent {
        timestamp: SmartString::from("2277-10-23T11:00:00Z"),
        location: Some(SmartString::from("megaton_01")),
        event_type: SmartString::from("dialogue"),
        description: SmartString::from(
            "Moira Brown asked us to help with her Wasteland Survival Guide.",
        ),
        entities: vec![SmartString::from("moira_brown_01")],
    };
    worldbook.add_event(event3);
    println!("  ✓ Event 3: Talked with Moira Brown");

    let event4 = WorldEvent {
        timestamp: SmartString::from("2277-10-23T14:00:00Z"),
        location: Some(SmartString::from("capital_wasteland_01")),
        event_type: SmartString::from("combat"),
        description: SmartString::from("Fought off Gristle and his raider gang in the wasteland."),
        entities: vec![SmartString::from("gristle_01")],
    };
    worldbook.add_event(event4);
    println!("  ✓ Event 4: Combat with raiders");
    println!();

    // Visit locations
    worldbook.visit_location("vault_13");
    worldbook.visit_location("megaton_01");
    worldbook.visit_location("megaton_01"); // Visit twice
    worldbook.set_current_location(Some(SmartString::from("megaton_01")));

    // Display worldbook statistics
    println!(
        "{}",
        "═══════════════════════════════════════════".bright_cyan()
    );
    println!(
        "{}",
        "           WORLDBOOK STATISTICS            "
            .bright_cyan()
            .bold()
    );
    println!(
        "{}",
        "═══════════════════════════════════════════".bright_cyan()
    );
    println!();

    println!("{}", "Locations:".bold());
    for (id, location) in &worldbook.locations {
        println!(
            "  • {} [{}]",
            location.name.bright_yellow(),
            location.location_type.bright_black()
        );
        println!("    ID: {}", id.bright_black());
        println!("    Type: {}", location.location_type);
        println!("    Visits: {}", location.visit_count);
        if let Some(desc) = location
            .description
            .chars()
            .take(80)
            .collect::<String>()
            .into()
        {
            println!("    Description: {}...", desc);
        }
        println!();
    }

    println!("{}", "NPCs:".bold());
    for (id, npc) in &worldbook.npcs {
        let disposition_color = if npc.disposition >= 50 {
            "friendly".green()
        } else if npc.disposition >= 0 {
            "neutral".yellow()
        } else {
            "hostile".red()
        };

        println!(
            "  • {} [{}]",
            npc.name.bright_yellow(),
            npc.role.bright_black()
        );
        println!("    ID: {}", id.bright_black());
        println!(
            "    Status: {}",
            if npc.alive {
                "Alive".green()
            } else {
                "Dead".red()
            }
        );
        println!(
            "    Disposition: {} ({})",
            npc.disposition, disposition_color
        );
        println!("    Personality: {}", npc.personality.join(", "));
        if let Some(loc) = &npc.current_location {
            if let Some(location) = worldbook.get_location(loc) {
                println!("    Location: {}", location.name);
            }
        }
        println!();
    }

    println!("{}", "Recent Events:".bold());
    for event in &worldbook.events {
        let event_icon = match event.event_type.as_str() {
            "combat" => "⚔",
            "npc_met" => "👤",
            "dialogue" => "💬",
            "discovery" => "🗺",
            _ => "•",
        };

        println!(
            "  {} [{}] {}",
            event_icon,
            event.timestamp.bright_black(),
            event.description
        );
        if let Some(loc) = &event.location {
            if let Some(location) = worldbook.get_location(loc) {
                println!("    Location: {}", location.name.bright_cyan());
            }
        }
        if !event.entities.is_empty() {
            print!("    Entities: ");
            for entity_id in &event.entities {
                if let Some(npc) = worldbook.get_npc(entity_id) {
                    print!("{} ", npc.name.bright_yellow());
                }
            }
            println!();
        }
        println!();
    }

    // Build AI context
    println!(
        "{}",
        "═══════════════════════════════════════════".bright_magenta()
    );
    println!(
        "{}",
        "           AI CONTEXT BUILDING             "
            .bright_magenta()
            .bold()
    );
    println!(
        "{}",
        "═══════════════════════════════════════════".bright_magenta()
    );
    println!();

    let context = worldbook.build_context();
    println!("{}", "Generated AI Context:".bold());
    println!(
        "{}",
        "─────────────────────────────────────────────────────────".bright_black()
    );
    println!("{}", context);
    println!(
        "{}",
        "─────────────────────────────────────────────────────────".bright_black()
    );
    println!();

    // Show token count estimate
    let token_estimate = context.len() / 4; // Rough estimate: 1 token ≈ 4 chars
    println!("Context Length: {} characters", context.len());
    println!("Estimated Tokens: ~{}", token_estimate);
    println!();

    // Query examples
    println!("{}", "Query Examples:".bright_green().bold());
    println!();

    println!("  • NPCs at Megaton:");
    let npcs_at_megaton = worldbook.get_npcs_at_location("megaton_01");
    for npc in npcs_at_megaton {
        println!("    - {} ({})", npc.name.bright_yellow(), npc.role);
    }
    println!();

    println!("  • Recent events at Megaton:");
    let megaton_events = worldbook.get_location_events("megaton_01", 5);
    for event in megaton_events {
        println!("    - {}", event.description);
    }
    println!();

    println!("  • Current Location:");
    if let Some(current_loc) = &worldbook.current_location {
        if let Some(location) = worldbook.get_location(current_loc) {
            println!(
                "    {} - {}",
                location.name.bright_yellow(),
                location.description
            );
        }
    }
    println!();

    println!("{}", "Worldbook demo complete!".bright_cyan().bold());
}
