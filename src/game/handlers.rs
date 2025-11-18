use super::char_handlers::print_detailed_stats;
use super::combat_handlers::{
    end_combat, handle_enemy_turns, handle_player_attack, start_combat_encounter,
};
use super::items::ItemType;
use super::persistence::save_game;
use super::rolls::{parse_roll_request, perform_roll};
use super::worldbook::Worldbook;
use super::GameState;
use crate::ai::extractor::{ExtractedEntities, ExtractionAI};
use crate::ai::AIDungeonMaster;
use crate::config::Config;
use crate::ui::UI;
use colored::*;

// Re-export for main.rs
pub use super::char_handlers::create_new_character;
pub use super::persistence::load_game;

/// Main game loop handling player input and game state
pub async fn game_loop(mut game_state: GameState, ai_dm: &AIDungeonMaster, config: Config) {
    UI::clear_screen();

    // Initialize extraction AI for worldbook updates
    let extractor = ExtractionAI::new(config.llama.extraction_url.clone());

    // Display opening message
    show_opening_message(&ai_dm, &mut game_state).await;

    loop {
        // Display status
        display_status(&game_state);

        // Show action menu
        show_action_menu(game_state.combat.active);

        let input = UI::prompt(">");

        if input.is_empty() {
            continue;
        }

        let parts: Vec<&str> = input.split_whitespace().collect();
        let command = parse_numbered_command(&parts[0].to_lowercase(), game_state.combat.active);

        match command.as_str() {
            "quit" | "exit" => {
                if UI::prompt("Save before quitting? (y/n):").to_lowercase() == "y" {
                    save_game(&game_state);
                }
                break;
            }
            "save" => save_game(&game_state),
            "inventory" => {
                UI::print_inventory(&game_state.character.inventory);
                UI::wait_for_enter();
            }
            "stats" => {
                print_detailed_stats(&game_state.character);
                UI::wait_for_enter();
            }
            "worldbook" | "wb" => {
                display_worldbook_menu(&mut game_state);
            }
            "locations" => {
                display_locations(&game_state);
                UI::wait_for_enter();
            }
            "npcs" => {
                display_npcs(&game_state);
                UI::wait_for_enter();
            }
            "events" => {
                display_events(&game_state);
                UI::wait_for_enter();
            }
            "attack" if game_state.combat.active => {
                if parts.len() < 2 {
                    UI::print_error("Specify target: attack <number>");
                    continue;
                }

                if let Ok(target_idx) = parts[1].parse::<usize>() {
                    if target_idx > 0 && target_idx <= game_state.combat.enemies.len() {
                        handle_player_attack(&mut game_state, target_idx - 1);
                        handle_enemy_turns(&mut game_state);

                        if game_state.combat.all_enemies_dead() {
                            end_combat(&mut game_state);
                        } else if !game_state.character.is_alive() {
                            UI::print_error("You have died...");
                            break;
                        }
                    } else {
                        UI::print_error("Invalid target number");
                    }
                }
            }
            "use" => {
                if parts.len() < 2 {
                    UI::print_error("Specify item ID: use <item_id>");
                    UI::print_info("Available: stimpak, radaway");
                    continue;
                }

                let item_id = parts[1];
                match game_state.character.use_consumable(item_id) {
                    Ok(message) => {
                        UI::print_success(&message);
                        if game_state.combat.active {
                            handle_enemy_turns(&mut game_state);
                        }
                    }
                    Err(e) => UI::print_error(&e),
                }
            }
            "roll" if !game_state.combat.active => {
                if parts.len() < 2 {
                    UI::print_error("Describe what you're trying to do: roll <action>");
                    UI::print_info("Examples: 'roll pick the lock', 'roll persuade the guard'");
                    continue;
                }

                // Get the action description
                let action = parts[1..].join(" ");

                // Ask AI DM to determine skill and DC
                handle_skill_roll(&mut game_state, &ai_dm, &action).await;
            }
            "run" if game_state.combat.active => {
                UI::print_info("You attempt to flee from combat!");
                game_state.combat.end_combat();
                UI::print_success("Escaped from combat!");
            }
            "equip" => {
                handle_equip(&mut game_state);
            }
            _ => {
                // Send player action to AI DM
                handle_ai_action(&input, &mut game_state, &ai_dm, &extractor).await;
            }
        }
    }
}

/// Display opening message from AI DM
async fn show_opening_message(ai_dm: &AIDungeonMaster, game_state: &mut GameState) {
    println!();
    UI::print_info("The AI Dungeon Master is preparing your adventure...");
    println!();

    let opening = ai_dm.generate_response(&game_state, "I step out into the wasteland for the first time.").await
        .unwrap_or_else(|e| {
            UI::print_error(&format!("AI DM Error: {}", e));
            "You emerge from the vault into the harsh wasteland. The sun beats down mercilessly. What do you do?".to_string()
        });

    UI::clear_screen();
    UI::print_dm_response(&opening);
    game_state.story.add(format!("DM: {}", opening));
}

/// Display character and combat status
fn display_status(game_state: &GameState) {
    UI::print_character_sheet(&game_state.character);

    if game_state.combat.active {
        UI::print_combat_status(&game_state.combat);
    }
}

/// Show available actions based on game state
fn show_action_menu(combat_active: bool) {
    println!("{}", "ACTIONS:".bold());
    if combat_active {
        println!("  1. attack <target#> - Attack an enemy");
        println!("  2. use <item> - Use an item");
        println!("  3. run - Attempt to flee combat");
    } else {
        println!("  <text> - Describe your action or talk");
        println!("  1. roll <action> - Make a skill/stat check");
        println!("  2. inventory - View inventory");
        println!("  3. stats - View detailed stats");
        println!("  4. worldbook / wb - View worldbook");
        println!("  5. save - Save game");
        println!("  6. quit - Exit to main menu");
        println!("  7. equip - Equip weapon or armor");
    }
    println!();
}

/// Parse numbered shortcuts to command names
fn parse_numbered_command(command: &str, combat_active: bool) -> String {
    if combat_active {
        match command {
            "1" => "attack".to_string(),
            "2" => "use".to_string(),
            "3" => "run".to_string(),
            _ => command.to_string(),
        }
    } else {
        match command {
            "1" => "roll".to_string(),
            "2" => "inventory".to_string(),
            "3" => "stats".to_string(),
            "4" => "worldbook".to_string(),
            "5" => "save".to_string(),
            "6" => "quit".to_string(),
            "7" => "equip".to_string(),
            _ => command.to_string(),
        }
    }
}

/// Handle player action by sending to AI DM
async fn handle_ai_action(
    input: &str,
    game_state: &mut GameState,
    ai_dm: &AIDungeonMaster,
    extractor: &ExtractionAI,
) {
    game_state.story.add(format!("Player: {}", input));
    UI::print_info("The DM is thinking...");

    match ai_dm.generate_response(&game_state, &input).await {
        Ok(response) => {
            UI::clear_screen();
            UI::print_dm_response(&response);
            game_state.story.add(format!("DM: {}", response));

            // Extract entities from AI response in background
            extract_and_save_entities(&extractor, &response, game_state).await;

            // Check if DM initiated combat (simplified detection)
            check_and_start_combat(&response, game_state);
        }
        Err(e) => {
            UI::print_error(&format!("AI Error: {}", e));
            UI::print_info("Continue without AI response, or check llama.cpp server");
        }
    }
}

/// Check if narrative suggests combat and prompt to start
fn check_and_start_combat(response: &str, game_state: &mut GameState) {
    if !game_state.combat.active
        && (response.contains("attack")
            || response.contains("combat")
            || response.contains("fight"))
    {
        if UI::prompt("Start combat? (y/n):").to_lowercase() == "y" {
            start_combat_encounter(game_state);
        }
    }
}

/// Handle a skill/stat roll requested by the player
async fn handle_skill_roll(game_state: &mut GameState, ai_dm: &AIDungeonMaster, action: &str) {
    println!();
    UI::print_info(&format!("Rolling for: {}", action));

    // Build a special prompt for the AI to determine skill and DC
    let roll_prompt = format!(
        "The player wants to attempt: '{}'\n\n\
        CRITICAL: Your response MUST start with one of these exact formats:\n\
        SKILL: <skill_name> DC <number>\n\
        or\n\
        STAT: <stat_name> DC <number>\n\n\
        Available Skills: Small Guns, Big Guns, Energy Weapons, Melee Weapons, Unarmed, Speech, Sneak, Lockpick, Science, Repair\n\
        Available Stats: Strength, Perception, Endurance, Charisma, Intelligence, Agility, Luck\n\n\
        DC Guidelines:\n\
        - Easy: DC 5-10 (routine tasks)\n\
        - Medium: DC 11-15 (challenging tasks)\n\
        - Hard: DC 16-20 (difficult tasks)\n\
        - Very Hard: DC 21-25 (expert-level tasks)\n\
        - Nearly Impossible: DC 26+ (legendary tasks)\n\n\
        EXAMPLES:\n\
        Player: \"pick the lock\" → SKILL: Lockpick DC 12\n\
        Player: \"persuade the guard\" → SKILL: Speech DC 14\n\
        Player: \"use my intelligence to divine which way is west\" → STAT: Intelligence DC 10\n\
        Player: \"hack the terminal\" → SKILL: Science DC 15\n\
        Player: \"notice hidden details\" → STAT: Perception DC 12\n\
        Player: \"break down the door\" → STAT: Strength DC 18\n\n\
        After the SKILL/STAT line, you may add a brief description of what they're attempting.",
        action
    );

    UI::print_info("The DM is determining the challenge...");

    match ai_dm.generate_response(game_state, &roll_prompt).await {
        Ok(response) => {
            // Try to parse the skill/stat and DC from AI response
            if let Some((skill_or_stat, dc)) = parse_roll_request(&response) {
                println!();
                UI::print_dm_response(&response);
                println!();

                // Perform the roll
                let result = perform_roll(&game_state.character, &skill_or_stat, dc);

                // Display roll result with color
                let result_text = result.format();
                if result.critical {
                    println!("{} {}", result.emoji(), result_text.green().bold());
                } else if result.fumble {
                    println!("{} {}", result.emoji(), result_text.red().bold());
                } else if result.success {
                    println!("{} {}", result.emoji(), result_text.green());
                } else {
                    println!("{} {}", result.emoji(), result_text.red());
                }
                println!();

                // Ask AI to narrate the outcome
                let outcome_prompt = format!(
                    "The player attempted: {}\n\
                    Roll result: {} (rolled {}, total {} vs DC {})\n\n\
                    Narrate what happens as a result of this {}. Be descriptive and atmospheric.",
                    action,
                    if result.success { "SUCCESS" } else { "FAILURE" },
                    result.roll,
                    result.total,
                    result.dc,
                    if result.critical {
                        "CRITICAL SUCCESS"
                    } else if result.fumble {
                        "CRITICAL FAILURE"
                    } else if result.success {
                        "success"
                    } else {
                        "failure"
                    }
                );

                UI::print_info("The DM narrates the outcome...");

                match ai_dm.generate_response(game_state, &outcome_prompt).await {
                    Ok(outcome) => {
                        println!();
                        UI::print_dm_response(&outcome);
                        game_state
                            .story
                            .add(format!("Player rolled for: {}", action));
                        game_state.story.add(format!("Result: {}", result_text));
                        game_state.story.add(format!("DM: {}", outcome));
                    }
                    Err(e) => {
                        UI::print_error(&format!("AI Error: {}", e));
                    }
                }
            } else {
                UI::print_error("AI DM didn't provide a clear skill/DC. Try again or describe your action differently.");
                UI::print_info(&format!("DM Response: {}", response));
            }
        }
        Err(e) => {
            UI::print_error(&format!("AI Error: {}", e));
        }
    }

    println!();
    UI::wait_for_enter();
}

/// Display worldbook menu
fn display_worldbook_menu(game_state: &mut GameState) {
    loop {
        UI::clear_screen();
        println!("{}", "═══ WORLDBOOK ═══".bold().cyan());
        println!();
        println!("Total Locations: {}", game_state.worldbook.locations.len());
        println!("Total NPCs: {}", game_state.worldbook.npcs.len());
        println!("Total Events: {}", game_state.worldbook.events.len());
        println!();

        if let Some(current_loc_id) = &game_state.worldbook.current_location {
            if let Some(loc) = game_state.worldbook.get_location(current_loc_id) {
                println!("Current Location: {}", loc.name.green().bold());
                println!();
            }
        }

        println!("{}", "OPTIONS:".bold());
        println!("  1. View all locations");
        println!("  2. View all NPCs");
        println!("  3. View recent events");
        println!("  4. Search location");
        println!("  5. Search NPC");
        println!("  6. Back to game");
        println!();

        let choice = UI::prompt(">");

        match choice.as_str() {
            "1" => {
                display_locations(game_state);
                UI::wait_for_enter();
            }
            "2" => {
                display_npcs(game_state);
                UI::wait_for_enter();
            }
            "3" => {
                display_events(game_state);
                UI::wait_for_enter();
            }
            "4" => {
                let name = UI::prompt("Location name:");
                if let Some(loc) = find_location_by_name(game_state, &name) {
                    display_location_details(loc);
                    UI::wait_for_enter();
                } else {
                    UI::print_error("Location not found.");
                    UI::wait_for_enter();
                }
            }
            "5" => {
                let name = UI::prompt("NPC name:");
                if let Some(npc) = find_npc_by_name(game_state, &name) {
                    display_npc_details(npc);
                    UI::wait_for_enter();
                } else {
                    UI::print_error("NPC not found.");
                    UI::wait_for_enter();
                }
            }
            "6" | "back" | "exit" => break,
            _ => UI::print_error("Invalid choice"),
        }
    }
}

/// Display all locations
fn display_locations(game_state: &GameState) {
    UI::clear_screen();
    println!("{}", "═══ LOCATIONS ═══".bold().cyan());
    println!();

    if game_state.worldbook.locations.is_empty() {
        println!("No locations discovered yet.");
        return;
    }

    let mut locations: Vec<_> = game_state.worldbook.locations.values().collect();
    locations.sort_by_key(|l| &l.name);

    for (i, loc) in locations.iter().enumerate() {
        println!(
            "{}. {} ({})",
            i + 1,
            loc.name.green().bold(),
            loc.location_type
        );
        println!("   {}", loc.description);
        println!(
            "   Visits: {} | NPCs present: {}",
            loc.visit_count,
            loc.npcs_present.len()
        );
        println!();
    }
}

/// Display all NPCs
fn display_npcs(game_state: &GameState) {
    UI::clear_screen();
    println!("{}", "═══ NPCs ═══".bold().cyan());
    println!();

    if game_state.worldbook.npcs.is_empty() {
        println!("No NPCs met yet.");
        return;
    }

    let mut npcs: Vec<_> = game_state.worldbook.npcs.values().collect();
    npcs.sort_by_key(|n| &n.name);

    for (i, npc) in npcs.iter().enumerate() {
        let status = if npc.alive {
            "Alive".green()
        } else {
            "Dead".red()
        };
        let disposition_color = match npc.disposition {
            d if d >= 50 => "Allied".green(),
            d if d >= 10 => "Friendly".cyan(),
            d if d >= -10 => "Neutral".yellow(),
            d if d >= -50 => "Unfriendly".magenta(),
            _ => "Hostile".red(),
        };

        println!(
            "{}. {} ({}) - {}",
            i + 1,
            npc.name.green().bold(),
            npc.role,
            status
        );
        println!(
            "   Disposition: {} ({})",
            disposition_color, npc.disposition
        );

        if !npc.personality.is_empty() {
            println!("   Traits: {}", npc.personality.join(", "));
        }

        if let Some(location) = &npc.current_location {
            if let Some(loc) = game_state.worldbook.get_location(location) {
                println!("   Location: {}", loc.name);
            }
        }

        if !npc.notes.is_empty() {
            println!("   Notes: {}", npc.notes);
        }

        println!();
    }
}

/// Display recent events
fn display_events(game_state: &GameState) {
    UI::clear_screen();
    println!("{}", "═══ RECENT EVENTS ═══".bold().cyan());
    println!();

    if game_state.worldbook.events.is_empty() {
        println!("No events recorded yet.");
        return;
    }

    // Show last 20 events, most recent first
    let events: Vec<_> = game_state.worldbook.events.iter().rev().take(20).collect();

    for (i, event) in events.iter().enumerate() {
        let event_type_colored = match event.event_type.as_str() {
            "combat" => event.event_type.red(),
            "npc_met" => event.event_type.green(),
            "discovery" => event.event_type.cyan(),
            "dialogue" => event.event_type.yellow(),
            _ => event.event_type.normal(),
        };

        println!("{}. [{}] {}", i + 1, event_type_colored, event.description);

        if let Some(location) = &event.location {
            if let Some(loc) = game_state.worldbook.get_location(location) {
                println!("   Location: {}", loc.name);
            }
        }

        println!();
    }
}

/// Display location details
fn display_location_details(loc: &super::worldbook::Location) {
    UI::clear_screen();
    println!("{}", format!("═══ {} ═══", loc.name).bold().cyan());
    println!();
    println!("Type: {}", loc.location_type);
    println!("Description: {}", loc.description);
    println!("Visits: {}", loc.visit_count);

    if let Some(atmosphere) = &loc.atmosphere {
        println!("Atmosphere: {}", atmosphere);
    }

    if !loc.npcs_present.is_empty() {
        println!();
        println!("NPCs present: {}", loc.npcs_present.len());
    }

    if !loc.notes.is_empty() {
        println!();
        println!("{}", "Notes:".bold());
        for note in &loc.notes {
            println!("  - {}", note);
        }
    }

    if !loc.state.is_empty() {
        println!();
        println!("{}", "State:".bold());
        for (key, value) in &loc.state {
            println!("  {}: {}", key, value);
        }
    }
}

/// Display NPC details
fn display_npc_details(npc: &super::worldbook::NPC) {
    UI::clear_screen();
    println!("{}", format!("═══ {} ═══", npc.name).bold().cyan());
    println!();

    let status = if npc.alive {
        "Alive".green()
    } else {
        "Dead".red()
    };
    println!("Status: {}", status);
    println!("Role: {}", npc.role);
    println!("Disposition: {}", npc.disposition);

    if !npc.personality.is_empty() {
        println!("Personality: {}", npc.personality.join(", "));
    }

    if let Some(location) = &npc.current_location {
        println!("Current Location: {}", location);
    }

    if !npc.knowledge.is_empty() {
        println!();
        println!("{}", "Knowledge:".bold());
        for item in &npc.knowledge {
            println!("  - {}", item);
        }
    }

    if !npc.notes.is_empty() {
        println!();
        println!("{}", "Notes:".bold());
        println!("{}", npc.notes);
    }
}

/// Find location by name (case-insensitive partial match)
fn find_location_by_name<'a>(
    game_state: &'a GameState,
    name: &str,
) -> Option<&'a super::worldbook::Location> {
    let name_lower = name.to_lowercase();
    game_state
        .worldbook
        .locations
        .values()
        .find(|loc| loc.name_lowercase.contains(&name_lower))
}

/// Find NPC by name (case-insensitive partial match)
fn find_npc_by_name<'a>(
    game_state: &'a GameState,
    name: &str,
) -> Option<&'a super::worldbook::NPC> {
    let name_lower = name.to_lowercase();
    game_state
        .worldbook
        .npcs
        .values()
        .find(|npc| npc.name_lowercase.contains(&name_lower))
}

/// Extract entities from AI narrative and save to worldbook
async fn extract_and_save_entities(
    extractor: &ExtractionAI,
    narrative: &str,
    game_state: &mut GameState,
) {
    let entities = match extractor.extract_entities(narrative).await {
        Ok(e) if !e.is_empty() => e,
        Ok(_) => return, // No entities found
        Err(e) => {
            #[cfg(debug_assertions)]
            UI::print_error(&format!("Worldbook extraction error: {}", e));
            return;
        }
    };

    display_extracted_entities(&entities);

    let response = UI::prompt("Save to worldbook? (y/n/view):");
    match response.to_lowercase().as_str() {
        "y" | "yes" => save_entities_to_worldbook(entities, game_state),
        "view" => display_worldbook_summary(game_state),
        _ => UI::print_info("Skipped worldbook update."),
    }

    println!();
}

/// Display extracted entities to the user
fn display_extracted_entities(entities: &ExtractedEntities) {
    println!();
    UI::print_info(&format!("Worldbook: {}", entities.summary()));

    if !entities.locations.is_empty() {
        println!("{}", "  Locations:".yellow());
        for loc in &entities.locations {
            println!("    - {} ({})", loc.name, loc.location_type);
        }
    }

    if !entities.npcs.is_empty() {
        println!("{}", "  NPCs:".yellow());
        for npc in &entities.npcs {
            let traits = if npc.personality.is_empty() {
                String::new()
            } else {
                format!(" [{}]", npc.personality.join(", "))
            };
            println!("    - {} ({}){}", npc.name, npc.role, traits);
        }
    }

    if !entities.events.is_empty() {
        println!("{}", "  Events:".yellow());
        for event in &entities.events {
            println!("    - {}: {}", event.event_type, event.description);
        }
    }

    println!();
}

/// Save extracted entities to the worldbook
fn save_entities_to_worldbook(entities: ExtractedEntities, game_state: &mut GameState) {
    let (locations, npcs, events) = entities.to_worldbook_entries();
    let mut saved_count = 0;

    // Save locations
    for location in locations {
        let loc_id = location.id.clone();

        if game_state.worldbook.get_location(&loc_id).is_none() {
            // New location discovered
            game_state.worldbook.add_location(location);
            saved_count += 1;

            // Only set as current location if we don't have one
            if game_state.worldbook.current_location.is_none() {
                game_state
                    .worldbook
                    .set_current_location(Some(loc_id.clone()));
                game_state.worldbook.visit_location(&loc_id);
            }
        }
        // Don't auto-visit/set location if it already exists
        // Let the DM narrative determine when player moves
    }

    // Save NPCs
    for npc in npcs {
        if game_state.worldbook.get_npc(&npc.id).is_none() {
            game_state.worldbook.add_npc(npc);
            saved_count += 1;
        }
    }

    // Save events
    for event in events {
        game_state.worldbook.add_event(event);
        saved_count += 1;
    }

    if saved_count > 0 {
        UI::print_success(&format!("Saved {} entries to worldbook!", saved_count));
        save_worldbook_to_file(&game_state.worldbook);
    } else {
        UI::print_info("All entities already in worldbook.");
    }
}

/// Display a summary of worldbook contents
fn display_worldbook_summary(game_state: &GameState) {
    println!();
    UI::print_info("Worldbook contents:");
    println!("Locations: {}", game_state.worldbook.locations.len());
    println!("NPCs: {}", game_state.worldbook.npcs.len());
    println!("Events: {}", game_state.worldbook.events.len());
}

/// Save worldbook to file

/// Handle equipping weapons or armor from inventory
fn handle_equip(game_state: &mut GameState) {
    let character = &mut game_state.character;

    // Filter equippable items
    let _weapons: Vec<_> = character
        .inventory
        .iter()
        .filter(|item| matches!(item.item_type, ItemType::Weapon(_)))
        .cloned()
        .collect();

    let _armor: Vec<_> = character
        .inventory
        .iter()
        .filter(|item| matches!(item.item_type, ItemType::Armor(_)))
        .cloned()
        .collect();

    // TODO: Implement equipment menu UI
    UI::print_info("Equipment system not yet implemented");
}

/// Save worldbook to file
fn save_worldbook_to_file(worldbook: &Worldbook) {
    let worldbook_path = std::path::Path::new("saves/worldbook.json");
    if let Err(e) = worldbook.save_to_file(worldbook_path) {
        UI::print_error(&format!("Failed to save worldbook: {}", e));
    }
}
