use crate::ai::AIDungeonMaster;
use crate::config::Config;
use crate::game::GameState;
use crate::game::rolls::{parse_natural_roll_request, perform_roll, truncate_response_at_skill_check};
use crate::tui::{self, App, Event, EventHandler};
use crossterm::event::{KeyCode, KeyEvent, MouseEvent, MouseEventKind};
use std::io;

/// Run the game loop with the TUI interface
pub async fn run_game_with_tui(
    game_state: GameState,
    ai_dm: &AIDungeonMaster,
    _config: Config,
) -> io::Result<()> {
    // Initialize terminal
    let mut terminal = tui::init_terminal()?;

    // Create app state
    let mut app = App::new(game_state);

    // Create event handler with 50ms tick rate for smooth animations (20 FPS)
    let event_handler = EventHandler::new(50);

    // Main loop
    let result = run_app(&mut terminal, &mut app, &event_handler, ai_dm).await;

    // Restore terminal
    tui::restore_terminal(terminal)?;

    result
}

async fn run_app<B: ratatui::backend::Backend>(
    terminal: &mut ratatui::Terminal<B>,
    app: &mut App,
    event_handler: &EventHandler,
    ai_dm: &AIDungeonMaster,
) -> io::Result<()> {
    loop {
        // Update view mode based on combat status
        app.update_view_mode_for_combat();

        // Render UI
        terminal.draw(|f| tui::ui::render(f, app))?;

        // Handle events
        match event_handler.next()? {
            Event::Key(key) => {
                if let Err(e) = handle_key_event(app, key, ai_dm).await {
                    app.add_error_message(format!("Error: {}", e));
                }

                if app.should_quit {
                    // Render one final time to show exit message
                    terminal.draw(|f| tui::ui::render(f, app))?;
                    // Small delay to let user see the exit message
                    std::thread::sleep(std::time::Duration::from_millis(800));
                    break;
                }
            }
            Event::Mouse(mouse) => {
                handle_mouse_event(app, mouse);
            }
            Event::Resize => {
                // Terminal was resized, will automatically re-render
            }
            Event::Tick => {
                // Update animations on each tick
                app.animation_manager.update();

                // Update flicker state for retro CRT effect
                app.update_flicker();

                // Update loading spinner if waiting for AI
                if app.waiting_for_ai {
                    app.loading_spinner.next_frame();
                }

                // Process streaming tokens if available
                if app.is_streaming {
                    while let Some(result) = app.try_recv_token() {
                        match result {
                            Ok(token) => {
                                app.append_streaming_token(token);
                            }
                            Err(e) => {
                                app.cancel_streaming();
                                app.add_error_message(format!("Stream error: {}", e));
                                app.waiting_for_ai = false;
                                break;
                            }
                        }
                    }

                    // Check if stream has finished
                    if let Some(dm_response) = app.check_stream_finished() {
                        // Check if DM requested a skill check
                        if let Err(e) = handle_skill_check_if_needed(app, &dm_response, ai_dm).await {
                            app.add_error_message(format!("Skill check error: {}", e));
                        }
                        app.waiting_for_ai = false;
                        // TODO: Implement worldbook extraction from the completed response
                        // This would require passing the extraction AI client to this function
                        // and calling extract_and_save_entities() like in the classic mode
                    }
                }
            }
        }
    }

    Ok(())
}

async fn handle_key_event(
    app: &mut App,
    key: KeyEvent,
    ai_dm: &AIDungeonMaster,
) -> anyhow::Result<()> {
    // Don't process input while waiting for AI
    if app.waiting_for_ai {
        return Ok(());
    }

    // Special handling for Worldbook view mode
    if app.view_mode == crate::tui::app::ViewMode::Worldbook {
        return handle_worldbook_keys(app, key);
    }

    match key.code {
        // Quit
        KeyCode::Char('c')
            if key
                .modifiers
                .contains(crossterm::event::KeyModifiers::CONTROL) =>
        {
            app.should_quit = true;
        }

        // Navigation in view modes
        KeyCode::Esc => {
            if app.view_mode != crate::tui::app::ViewMode::Normal {
                app.set_view_mode(crate::tui::app::ViewMode::Normal);
            }
        }

        // Scroll message log
        KeyCode::PageUp => {
            app.scroll_up();
        }
        KeyCode::PageDown => {
            app.scroll_down();
        }

        // Enter key - submit input
        KeyCode::Enter => {
            if !app.input.is_empty() {
                let input = app.take_input();
                handle_player_input(app, &input, ai_dm).await?;
            }
        }

        // Backspace
        KeyCode::Backspace => {
            app.delete_char();
        }

        // Cursor movement
        KeyCode::Left => {
            app.move_cursor_left();
        }
        KeyCode::Right => {
            app.move_cursor_right();
        }
        KeyCode::Home => {
            app.move_cursor_start();
        }
        KeyCode::End => {
            app.move_cursor_end();
        }

        // Character input
        KeyCode::Char(c) => {
            app.enter_char(c);
        }

        _ => {}
    }

    Ok(())
}

/// Handle keyboard events when in worldbook view
fn handle_worldbook_keys(app: &mut App, key: KeyEvent) -> anyhow::Result<()> {
    use crate::tui::worldbook_browser::WorldbookTab;

    match key.code {
        // Quit worldbook view
        KeyCode::Char('q') | KeyCode::Char('Q') | KeyCode::Esc => {
            app.set_view_mode(crate::tui::app::ViewMode::Normal);
        }

        // Tab navigation (legacy - still works)
        KeyCode::Tab => {
            app.worldbook_browser.next_tab();
        }
        KeyCode::BackTab => {
            app.worldbook_browser.prev_tab();
        }

        // Arrow key navigation - behavior depends on focus
        KeyCode::Left => {
            if app.worldbook_browser.is_tab_bar_focused() {
                // Navigate to previous tab when focused on tab bar
                app.worldbook_browser.prev_tab();
            }
        }
        KeyCode::Right => {
            if app.worldbook_browser.is_tab_bar_focused() {
                // Navigate to next tab when focused on tab bar
                app.worldbook_browser.next_tab();
            }
        }

        KeyCode::Up => {
            if app.worldbook_browser.is_list_focused() {
                // Navigate up in list
                let max_items = match app.worldbook_browser.active_tab {
                    WorldbookTab::Locations => app.game_state.worldbook.locations.len(),
                    WorldbookTab::NPCs => app.game_state.worldbook.npcs.len(),
                    WorldbookTab::Events => app.game_state.worldbook.events.len(),
                    WorldbookTab::Search => 0,
                };

                // If at the top of the list, move focus to tab bar
                if app.worldbook_browser.selected_index == 0 {
                    app.worldbook_browser.focus_tab_bar();
                } else {
                    app.worldbook_browser.select_prev(max_items);
                }
            }
        }

        KeyCode::Down => {
            if app.worldbook_browser.is_tab_bar_focused() {
                // Move focus from tab bar to list
                app.worldbook_browser.focus_list();
            } else if app.worldbook_browser.is_list_focused() {
                // Navigate down in list
                let max_items = match app.worldbook_browser.active_tab {
                    WorldbookTab::Locations => app.game_state.worldbook.locations.len(),
                    WorldbookTab::NPCs => app.game_state.worldbook.npcs.len(),
                    WorldbookTab::Events => app.game_state.worldbook.events.len(),
                    WorldbookTab::Search => 0,
                };
                app.worldbook_browser.select_next(max_items);
            }
        }

        // Expand/collapse (for locations)
        KeyCode::Enter | KeyCode::Char(' ') => {
            if app.worldbook_browser.active_tab == WorldbookTab::Locations {
                let locations = app
                    .worldbook_browser
                    .get_sorted_locations(&app.game_state.worldbook);
                if app.worldbook_browser.selected_index < locations.len() {
                    let (location_id, _) = &locations[app.worldbook_browser.selected_index];
                    app.worldbook_browser.toggle_expansion(location_id);
                }
            }
        }

        // Detail scrolling
        KeyCode::PageDown => {
            app.worldbook_browser.scroll_detail_down(100); // Max scroll arbitrary for now
        }
        KeyCode::PageUp => {
            app.worldbook_browser.scroll_detail_up();
        }

        _ => {}
    }

    Ok(())
}

async fn handle_player_input(
    app: &mut App,
    input: &str,
    ai_dm: &AIDungeonMaster,
) -> anyhow::Result<()> {
    let input = input.trim();
    if input.is_empty() {
        return Ok(());
    }

    // Echo player input
    app.add_player_action(input);

    // Handle special commands
    match input.to_lowercase().as_str() {
        "quit" | "exit" => {
            app.add_system_message("═══════════════════════════════════════".to_string());
            app.add_system_message("  Exiting game... Returning to main menu.".to_string());
            app.add_system_message("═══════════════════════════════════════".to_string());
            app.should_quit = true;
            return Ok(());
        }
        "inventory" | "inv" | "i" => {
            app.set_view_mode(crate::tui::app::ViewMode::Inventory);
            return Ok(());
        }
        "stats" | "sheet" => {
            app.set_view_mode(crate::tui::app::ViewMode::Stats);
            return Ok(());
        }
        "worldbook" | "wb" => {
            app.set_view_mode(crate::tui::app::ViewMode::Worldbook);
            return Ok(());
        }
        "save" => {
            // TODO: Implement save in TUI mode
            app.add_system_message("Save functionality coming soon in TUI mode!".to_string());
            return Ok(());
        }
        "help" => {
            show_help(app);
            return Ok(());
        }
        "debug" | "context" => {
            show_debug_context(app);
            return Ok(());
        }
        _ => {}
    }

    // Handle combat commands if in combat
    if app.is_in_combat() {
        if let Some(result) = handle_combat_command(app, input) {
            return result;
        }
    }

    // Otherwise, send to AI DM with streaming
    app.waiting_for_ai = true;

    // Get AI response stream
    match ai_dm.generate_response_stream(&app.game_state, input).await {
        Ok(rx) => {
            // Start streaming - tokens will be processed in the tick event
            app.start_streaming(rx);
            // Add player input to both conversation systems
            app.game_state.conversation.add_player_turn(input.to_string());
            app.game_state.story.add(format!("Player: {}", input)); // Legacy support
            // Note: waiting_for_ai will be set to false when streaming completes
        }
        Err(e) => {
            app.waiting_for_ai = false;
            app.add_error_message(format!("AI Error: {}", e));
            app.add_system_message(
                "The AI is unavailable. Try a different action or check your connection."
                    .to_string(),
            );
        }
    }

    Ok(())
}

fn handle_combat_command(app: &mut App, input: &str) -> Option<anyhow::Result<()>> {
    use crate::game::combat::{attack_roll, calculate_damage, resolve_stat_modifiers};

    let lower = input.to_lowercase();

    // Attack command
    if lower.starts_with("attack") || lower.starts_with("a ") {
        // Parse target number
        let parts: Vec<&str> = input.split_whitespace().collect();
        if parts.len() >= 2 {
            if let Ok(target) = parts[1].parse::<usize>() {
                let target_idx = target.saturating_sub(1); // Convert to 0-based index
                if target_idx < app.game_state.combat.enemies.len() {
                    let enemy = &app.game_state.combat.enemies[target_idx];

                    if !enemy.is_alive() {
                        app.add_error_message("That enemy is already defeated!".to_string());
                        return Some(Ok(()));
                    }

                    let enemy_ac = enemy.armor_class;
                    let enemy_name = enemy.name.clone();
                    let weapon_damage = app.game_state.character.get_equipped_damage();
                    let resolved_damage = resolve_stat_modifiers(
                        &weapon_damage,
                        app.game_state.character.special.strength,
                    );
                    let skill = app.game_state.character.get_weapon_skill();

                    if !app.game_state.character.use_ap(4) {
                        app.add_error_message("Not enough AP!".to_string());
                        return Some(Ok(()));
                    }

                    let (hit, critical) = attack_roll(skill, enemy_ac);

                    // Trigger dice roll animation (d20 roll)
                    let modifier = (skill as i32 - 10) / 2;
                    app.animation_manager.start_dice_roll(
                        if critical {
                            20
                        } else if !hit {
                            1
                        } else {
                            10
                        }, // Approximate result
                        modifier,
                    );

                    if hit {
                        let damage = calculate_damage(&resolved_damage, 0, critical);
                        app.game_state.combat.enemies[target_idx].take_damage(damage);

                        // Check if enemy died and trigger fadeout animation
                        if !app.game_state.combat.enemies[target_idx].is_alive() {
                            app.animation_manager.start_enemy_fadeout(target_idx);

                            // Award XP
                            let old_xp = app.game_state.character.experience;
                            let xp_reward = app.game_state.combat.enemies[target_idx].xp_reward;
                            app.game_state.character.add_experience(xp_reward);

                            // Trigger XP fill animation
                            app.animation_manager
                                .start_xp_fill(old_xp, app.game_state.character.experience);

                            if critical {
                                app.add_combat_message(format!(
                                    "⚡ CRITICAL HIT! {} damage to {}! Enemy defeated! +{} XP",
                                    damage, enemy_name, xp_reward
                                ));
                            } else {
                                app.add_combat_message(format!(
                                    "→ Hit! {} damage to {}! Enemy defeated! +{} XP",
                                    damage, enemy_name, xp_reward
                                ));
                            }
                        } else {
                            if critical {
                                app.add_combat_message(format!(
                                    "⚡ CRITICAL HIT! {} damage to {}!",
                                    damage, enemy_name
                                ));
                            } else {
                                app.add_combat_message(format!(
                                    "→ Hit! {} damage to {}!",
                                    damage, enemy_name
                                ));
                            }
                        }
                    } else {
                        app.add_combat_message("✗ Missed!".to_string());
                    }

                    // Enemy turn
                    handle_enemy_turn(app);

                    // Check if combat ended
                    if app.game_state.combat.all_enemies_dead() {
                        app.add_combat_message("🎉 Victory! All enemies defeated!".to_string());
                        app.game_state.combat.active = false;
                        app.set_view_mode(crate::tui::app::ViewMode::Normal);
                    }

                    return Some(Ok(()));
                }
            }
        }
        app.add_error_message("Invalid target. Use: attack <number>".to_string());
        return Some(Ok(()));
    }

    // Run command
    if lower == "run" || lower == "flee" {
        use rand::Rng;
        let mut rng = rand::rng();
        if rng.random_bool(0.6) {
            // 60% chance to escape
            app.add_combat_message("You successfully fled from combat!".to_string());
            app.game_state.combat.active = false;
            app.set_view_mode(crate::tui::app::ViewMode::Normal);
        } else {
            app.add_combat_message("Failed to escape!".to_string());
            handle_enemy_turn(app);
        }
        return Some(Ok(()));
    }

    None
}

/// Handle enemy turn attacks
fn handle_enemy_turn(app: &mut App) {
    use crate::game::combat::{attack_roll, calculate_damage};

    // Collect enemy data first to avoid borrow checker issues
    let player_ac = 10 + app.game_state.character.special.agility as i32;
    let enemy_attacks: Vec<_> = app
        .game_state
        .combat
        .enemies
        .iter()
        .filter(|e| e.is_alive())
        .map(|enemy| {
            let (hit, critical) = attack_roll(enemy.skill, player_ac);
            (
                enemy.name.clone(),
                enemy.damage.clone(),
                enemy.strength,
                hit,
                critical,
            )
        })
        .collect();

    // Process attacks
    for (enemy_name, enemy_damage, enemy_strength, hit, critical) in enemy_attacks {
        if hit {
            // Store old HP for animation
            let old_hp = app.game_state.character.current_hp;

            let str_bonus = (enemy_strength / 2) as i32;
            let damage = calculate_damage(&enemy_damage, str_bonus, critical);
            app.game_state.character.take_damage(damage);

            // Trigger HP drain animation
            app.animation_manager
                .start_health_drain(old_hp, app.game_state.character.current_hp);

            if critical {
                app.add_combat_message(format!(
                    "⚠ {} lands a CRITICAL hit! {} damage!",
                    enemy_name, damage
                ));
            } else {
                app.add_combat_message(format!("← {} hits for {} damage!", enemy_name, damage));
            }

            // Check if player died
            if app.game_state.character.current_hp <= 0 {
                app.add_combat_message("☠ You have been defeated!".to_string());
                app.add_system_message("Game Over! (Note: Death mechanics WIP)".to_string());
                app.game_state.combat.active = false;
                app.set_view_mode(crate::tui::app::ViewMode::Normal);
                return;
            }
        } else {
            app.add_combat_message(format!("○ {} misses!", enemy_name));
        }
    }

    app.game_state.character.restore_ap();
    app.game_state.combat.next_round();
}

fn show_help(app: &mut App) {
    app.add_system_message("═══ COMMANDS ═══".to_string());
    app.add_info_message("inventory, inv, i  - View your inventory".to_string());
    app.add_info_message("stats, sheet       - View character stats".to_string());
    app.add_info_message("worldbook, wb      - View worldbook".to_string());
    app.add_info_message("save               - Save your game".to_string());
    app.add_info_message("help               - Show this help".to_string());
    app.add_info_message("debug, context     - Show AI conversation context".to_string());
    app.add_info_message("quit, exit         - Exit game".to_string());
    app.add_system_message("".to_string());
    app.add_system_message("In combat:".to_string());
    app.add_info_message("attack <number>    - Attack enemy".to_string());
    app.add_info_message("run, flee          - Attempt to flee".to_string());
    app.add_system_message("".to_string());
    app.add_system_message("Press ESC to return to main view".to_string());
    app.add_system_message("Use PageUp/PageDown to scroll messages".to_string());
}

fn show_debug_context(app: &mut App) {
    app.add_system_message("═══ CONVERSATION CONTEXT DEBUG ═══".to_string());
    app.add_info_message(format!("Total turns: {}", app.game_state.conversation.len()));
    app.add_info_message(format!("Max turns: {}", app.game_state.conversation.max_turns()));
    app.add_system_message("".to_string());
    app.add_system_message("Recent conversation history (last 10 turns):".to_string());

    // Collect turns into owned strings to avoid borrow checker issues
    let recent_turns: Vec<String> = app.game_state.conversation.get_recent_turns(10)
        .iter()
        .map(|turn| turn.format())
        .collect();

    if recent_turns.is_empty() {
        app.add_info_message("(No conversation history yet)".to_string());
    } else {
        for turn_text in recent_turns {
            app.add_info_message(turn_text);
        }
    }

    app.add_system_message("".to_string());
    app.add_system_message("Legacy story context:".to_string());
    app.add_info_message(format!("Total events: {}", app.game_state.story.len()));
}

/// Handle mouse events for scrolling and interaction
fn handle_mouse_event(app: &mut App, mouse: MouseEvent) {
    match mouse.kind {
        MouseEventKind::ScrollUp => {
            // Scroll up through message history (3 lines at a time for smooth scrolling)
            for _ in 0..3 {
                app.scroll_up();
            }
        }
        MouseEventKind::ScrollDown => {
            // Scroll down through message history (3 lines at a time for smooth scrolling)
            for _ in 0..3 {
                app.scroll_down();
            }
        }
        _ => {
            // Ignore other mouse events (clicks, drags, etc.)
        }
    }
}

/// Check if DM response contains a skill check request and handle it automatically
async fn handle_skill_check_if_needed(
    app: &mut App,
    dm_response: &str,
    ai_dm: &AIDungeonMaster,
) -> anyhow::Result<()> {
    // Check if the DM response contains a skill check request
    if let Some((skill_or_stat, dc)) = parse_natural_roll_request(dm_response) {
        // Truncate the response to remove any AI commentary after the skill check
        if let Some(truncated_response) = truncate_response_at_skill_check(dm_response) {
            // Replace the last DM turn in conversation with the truncated version
            // This prevents bad patterns from entering conversation history
            app.game_state.conversation.replace_last_dm_turn(truncated_response);
        }

        // Perform the roll automatically
        let result = perform_roll(&app.game_state.character, &skill_or_stat, dc);

        // Format roll result message
        let roll_msg = format!(
            "🎲 {} Check: Rolled {}+{} = {} vs DC {} - {}",
            result.skill_name,
            result.roll,
            result.modifier,
            result.total,
            result.dc,
            if result.critical {
                "CRITICAL SUCCESS!"
            } else if result.fumble {
                "CRITICAL FAILURE!"
            } else if result.success {
                "Success"
            } else {
                "Failure"
            }
        );

        // Display the roll result
        app.add_system_message(roll_msg);

        // Add roll to conversation history
        app.game_state.conversation.add_player_turn(format!(
            "rolled {} - {}",
            result.skill_name,
            if result.success { "Success" } else { "Failure" }
        ));

        // Request AI to narrate the outcome
        app.waiting_for_ai = true;

        let outcome_prompt = format!(
            "GAME SYSTEM UPDATE:\n\
            You requested a {} check (DC {}).\n\
            The dice have been rolled automatically by the game system.\n\
            Result: {} - d20 rolled {}, modifier +{}, total {} vs DC {}\n\n\
            NOW NARRATE: Describe what happens as a result of this {}. \
            DO NOT mention the dice roll or ask the player to roll - that already happened. \
            Just describe the outcome narratively. Be vivid and engaging.\n\n\
            Start your response immediately with the narrative outcome:",
            result.skill_name,
            result.dc,
            if result.success { "SUCCESS" } else { "FAILURE" },
            result.roll,
            result.modifier,
            result.total,
            result.dc,
            if result.critical {
                "CRITICAL SUCCESS"
            } else if result.fumble {
                "CRITICAL FAILURE"
            } else {
                "result"
            }
        );

        // Get AI response stream for the outcome
        match ai_dm.generate_response_stream(&app.game_state, &outcome_prompt).await {
            Ok(rx) => {
                app.start_streaming(rx);
            }
            Err(e) => {
                app.waiting_for_ai = false;
                app.add_error_message(format!("AI Error when narrating outcome: {}", e));
                app.add_system_message(
                    "The roll succeeded but the DM couldn't narrate the outcome.".to_string(),
                );
            }
        }
    }

    Ok(())
}
