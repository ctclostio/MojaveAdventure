use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Gauge, List, ListItem, Paragraph},
    Frame,
};

use super::app::{App, MessageType, ViewMode};
use super::narrative;
use crate::game::character::Character;

/// Main render function
pub fn render(f: &mut Frame, app: &App) {
    let size = f.area();

    // Create root layout: [Main area | Status bar]
    let root_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(0),    // Main area (sidebar + content + input)
            Constraint::Length(3), // Status bar at bottom
        ])
        .split(size);

    // Create main layout: [Character Status | Main Content]
    let main_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length(35), // Character status sidebar
            Constraint::Min(0),     // Main content area
        ])
        .split(root_chunks[0]);

    // Render character status sidebar
    render_character_status(f, app, main_chunks[0]);

    // Split main content into message area and input
    let content_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(0),    // Message log
            Constraint::Length(3), // Input bar
        ])
        .split(main_chunks[1]);

    // Render main content based on view mode
    match app.view_mode {
        ViewMode::Normal => {
            render_message_log(f, app, content_chunks[0]);
        }
        ViewMode::Combat => {
            // Show tactical combat display if in active combat
            if app.game_state.combat.active {
                crate::tui::combat_display::render_combat_display(
                    f,
                    content_chunks[0],
                    &app.game_state.character,
                    &app.game_state.combat,
                    &app.animation_manager,
                    app.should_flicker,
                );
            } else {
                render_message_log(f, app, content_chunks[0]);
            }
        }
        ViewMode::Inventory => {
            render_inventory(f, app, content_chunks[0]);
        }
        ViewMode::Stats => {
            render_detailed_stats(f, app, content_chunks[0]);
        }
        ViewMode::Worldbook => {
            crate::tui::worldbook_ui::render_worldbook(f, app, content_chunks[0]);
        }
        ViewMode::GameOver => {
            render_game_over(f, app, content_chunks[0]);
        }
    }

    // Render input bar
    render_input_bar(f, app, content_chunks[1]);

    // Render status bar at the bottom
    render_status_bar(f, app, root_chunks[1]);
}

/// Render character status sidebar
fn render_character_status(f: &mut Frame, app: &App, area: Rect) {
    let character = &app.game_state.character;

    // Split sidebar into sections
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(7),  // Name, level, location
            Constraint::Length(6),  // HP, AP, XP
            Constraint::Length(10), // SPECIAL stats
            Constraint::Min(0),     // Quick items / status
        ])
        .split(area);

    // Character name and location
    render_character_header(f, app, chunks[0]);

    // HP, AP, XP bars
    render_vitals(f, character, &app.animation_manager, chunks[1]);

    // SPECIAL stats
    render_special_stats(f, character, chunks[2]);

    // Quick status
    render_quick_status(f, app, chunks[3]);
}

/// Render character header (name, level, location)
fn render_character_header(f: &mut Frame, app: &App, area: Rect) {
    use crate::tui::theme::{PipBoyTheme, RetroEffects};
    let character = &app.game_state.character;

    // Apply subtle flicker to decorative borders
    let border_style = if app.should_flicker {
        RetroEffects::flicker_style()
    } else {
        PipBoyTheme::border()
    };

    let text = vec![
        Line::from(vec![
            Span::styled("┌─ ", border_style),
            Span::styled(
                character.name.as_str(),
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(" ─┐", border_style),
        ]),
        Line::from(vec![
            Span::styled("│ ", border_style),
            Span::styled(
                format!("Level {} Vault Dweller", character.level),
                Style::default().fg(Color::Cyan),
            ),
        ]),
        Line::from(Span::styled(
            "├─────────────────────────────────┤",
            border_style,
        )),
        Line::from(vec![
            Span::styled("│ ", Style::default().fg(Color::Green)),
            Span::styled("📍 ", Style::default().fg(Color::Yellow)),
            Span::styled(
                truncate_string(&app.game_state.location, 25),
                Style::default().fg(Color::White),
            ),
        ]),
        Line::from(vec![
            Span::styled("│ ", Style::default().fg(Color::Green)),
            Span::styled("💰 ", Style::default().fg(Color::Yellow)),
            Span::styled(
                format!("{} Caps", character.caps),
                Style::default().fg(Color::Green),
            ),
        ]),
    ];

    let paragraph = Paragraph::new(text).style(Style::default());
    f.render_widget(paragraph, area);
}

/// Render HP, AP, XP vitals
fn render_vitals(
    f: &mut Frame,
    character: &Character,
    animation_manager: &crate::tui::animations::AnimationManager,
    area: Rect,
) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(2), // HP
            Constraint::Length(2), // AP
            Constraint::Length(2), // XP
        ])
        .split(area);

    // HP bar - use animated value if animation is active
    let display_hp = animation_manager
        .get_animated_hp(character.current_hp)
        .unwrap_or(character.current_hp);

    let hp_ratio = display_hp as f64 / character.max_hp as f64;
    let hp_color = if hp_ratio > 0.6 {
        Color::Green
    } else if hp_ratio > 0.3 {
        Color::Yellow
    } else {
        Color::Red
    };

    let hp_gauge = Gauge::default()
        .block(Block::default().borders(Borders::NONE))
        .gauge_style(Style::default().fg(hp_color))
        .label(format!("❤ HP {}/{}", display_hp, character.max_hp))
        .ratio(hp_ratio);
    f.render_widget(hp_gauge, chunks[0]);

    // AP bar
    let ap_ratio = character.current_ap as f64 / character.max_ap as f64;
    let ap_gauge = Gauge::default()
        .block(Block::default().borders(Borders::NONE))
        .gauge_style(Style::default().fg(Color::Yellow))
        .label(format!(
            "⚡ AP {}/{}",
            character.current_ap, character.max_ap
        ))
        .ratio(ap_ratio);
    f.render_widget(ap_gauge, chunks[1]);

    // XP bar - use animated value if animation is active
    let display_xp = animation_manager
        .get_animated_xp(character.experience)
        .unwrap_or(character.experience);

    let xp_for_next_level = (character.level * 1000) as i32;
    let xp_in_current_level = display_xp % 1000;
    let xp_ratio = xp_in_current_level as f64 / 1000.0;
    let xp_gauge = Gauge::default()
        .block(Block::default().borders(Borders::NONE))
        .gauge_style(Style::default().fg(Color::Cyan))
        .label(format!(
            "⭐ XP {}/{}",
            xp_in_current_level, xp_for_next_level
        ))
        .ratio(xp_ratio);
    f.render_widget(xp_gauge, chunks[2]);
}

/// Render SPECIAL stats
fn render_special_stats(f: &mut Frame, character: &Character, area: Rect) {
    let special = &character.special;

    let text = vec![
        Line::from(Span::styled(
            "├─ S.P.E.C.I.A.L. ────────────────┤",
            Style::default().fg(Color::Green),
        )),
        Line::from(vec![
            Span::styled("│ S ", Style::default().fg(Color::Green)),
            Span::styled(
                create_stat_bar(special.strength),
                Style::default().fg(Color::Yellow),
            ),
            Span::styled(
                format!(" {:2}", special.strength),
                Style::default().fg(Color::White),
            ),
        ]),
        Line::from(vec![
            Span::styled("│ P ", Style::default().fg(Color::Green)),
            Span::styled(
                create_stat_bar(special.perception),
                Style::default().fg(Color::Yellow),
            ),
            Span::styled(
                format!(" {:2}", special.perception),
                Style::default().fg(Color::White),
            ),
        ]),
        Line::from(vec![
            Span::styled("│ E ", Style::default().fg(Color::Green)),
            Span::styled(
                create_stat_bar(special.endurance),
                Style::default().fg(Color::Yellow),
            ),
            Span::styled(
                format!(" {:2}", special.endurance),
                Style::default().fg(Color::White),
            ),
        ]),
        Line::from(vec![
            Span::styled("│ C ", Style::default().fg(Color::Green)),
            Span::styled(
                create_stat_bar(special.charisma),
                Style::default().fg(Color::Yellow),
            ),
            Span::styled(
                format!(" {:2}", special.charisma),
                Style::default().fg(Color::White),
            ),
        ]),
        Line::from(vec![
            Span::styled("│ I ", Style::default().fg(Color::Green)),
            Span::styled(
                create_stat_bar(special.intelligence),
                Style::default().fg(Color::Yellow),
            ),
            Span::styled(
                format!(" {:2}", special.intelligence),
                Style::default().fg(Color::White),
            ),
        ]),
        Line::from(vec![
            Span::styled("│ A ", Style::default().fg(Color::Green)),
            Span::styled(
                create_stat_bar(special.agility),
                Style::default().fg(Color::Yellow),
            ),
            Span::styled(
                format!(" {:2}", special.agility),
                Style::default().fg(Color::White),
            ),
        ]),
        Line::from(vec![
            Span::styled("│ L ", Style::default().fg(Color::Green)),
            Span::styled(
                create_stat_bar(special.luck),
                Style::default().fg(Color::Yellow),
            ),
            Span::styled(
                format!(" {:2}", special.luck),
                Style::default().fg(Color::White),
            ),
        ]),
        Line::from(Span::styled(
            "└───────────────────────────────────┘",
            Style::default().fg(Color::Green),
        )),
    ];

    let paragraph = Paragraph::new(text);
    f.render_widget(paragraph, area);
}

/// Render quick status (weapon, combat status, etc.)
fn render_quick_status(f: &mut Frame, app: &App, area: Rect) {
    let character = &app.game_state.character;
    let mut lines = vec![];

    // Combat status
    if app.game_state.combat.active {
        lines.push(Line::from(vec![
            Span::styled("⚔ ", Style::default().fg(Color::Red)),
            Span::styled(
                format!("COMBAT - Round {}", app.game_state.combat.round),
                Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
            ),
        ]));
        lines.push(Line::from(""));

        // Show enemies
        for (i, enemy) in app.game_state.combat.enemies.iter().enumerate() {
            if enemy.is_alive() {
                let hp_percent = (enemy.current_hp as f64 / enemy.max_hp as f64 * 100.0) as u32;
                lines.push(Line::from(vec![
                    Span::styled(format!("[{}] ", i + 1), Style::default().fg(Color::Yellow)),
                    Span::styled(enemy.name.as_str(), Style::default().fg(Color::Red)),
                    Span::styled(
                        format!(" {}%", hp_percent),
                        Style::default().fg(Color::DarkGray),
                    ),
                ]));
            }
        }
    } else {
        // Show equipped weapon
        if let Some(weapon_id) = &character.equipped_weapon {
            if let Some(weapon) = character
                .inventory
                .iter()
                .find(|item| &item.id == weapon_id)
            {
                lines.push(Line::from(vec![
                    Span::styled("🔫 ", Style::default().fg(Color::Yellow)),
                    Span::styled(weapon.name.as_str(), Style::default().fg(Color::White)),
                ]));

                // Show damage if it's a weapon
                if let crate::game::items::ItemType::Weapon(ref stats) = weapon.item_type {
                    lines.push(Line::from(vec![
                        Span::styled("   ", Style::default()),
                        Span::styled(
                            format!("DMG: {}", stats.damage),
                            Style::default().fg(Color::DarkGray),
                        ),
                    ]));
                }
            }
        } else {
            lines.push(Line::from(vec![
                Span::styled("👊 ", Style::default().fg(Color::Yellow)),
                Span::styled("Unarmed", Style::default().fg(Color::White)),
            ]));
        }
    }

    let block = Block::default()
        .borders(Borders::TOP)
        .border_style(Style::default().fg(Color::Green))
        .border_type(BorderType::Plain);

    let paragraph = Paragraph::new(lines).block(block);
    f.render_widget(paragraph, area);
}

/// Render message log
fn render_message_log(f: &mut Frame, app: &App, area: Rect) {
    let title = if app.waiting_for_ai {
        "⏳ Dungeon Master (thinking...)"
    } else if app.is_in_combat() {
        "⚔ Combat Log"
    } else {
        "📜 Adventure Log"
    };

    let block = Block::default()
        .title(title)
        .title_alignment(Alignment::Left)
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Green))
        .border_type(BorderType::Rounded);

    let inner_area = block.inner(area);
    f.render_widget(block, area);

    // Calculate available height for messages
    let available_height = inner_area.height as usize;

    // Get visible messages
    let messages = app.get_visible_messages(available_height);

    // Create lines with proper formatting
    let mut lines: Vec<Line> = vec![];

    for msg in messages {
        // Use enhanced formatting for DM messages
        if msg.message_type == MessageType::DM {
            let narrative_lines =
                narrative::format_dm_narrative(&msg.content, inner_area.width as usize);
            lines.extend(narrative_lines);
            // Add spacing after DM message
            lines.push(Line::from(""));
        } else {
            // Use simple formatting for other message types
            let style = match msg.message_type {
                MessageType::DM => Style::default().fg(Color::Green),
                MessageType::Player => Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::ITALIC),
                MessageType::Combat => Style::default().fg(Color::Red),
                MessageType::System => Style::default().fg(Color::DarkGray),
                MessageType::Info => Style::default().fg(Color::Blue),
                MessageType::Success => Style::default().fg(Color::Green),
                MessageType::Error => Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
            };

            // Wrap text to fit width
            let wrapped_lines = wrap_text(&msg.content, inner_area.width as usize - 2);
            for line_text in wrapped_lines {
                lines.push(Line::from(Span::styled(line_text, style)));
            }

            // Add spacing between messages
            lines.push(Line::from(""));
        }
    }

    // Add streaming message if present
    if let Some(ref streaming_msg) = app.streaming_message {
        if !streaming_msg.is_empty() {
            let narrative_lines =
                narrative::format_dm_narrative(streaming_msg, inner_area.width as usize);
            lines.extend(narrative_lines);

            // Add typing indicator
            lines.push(Line::from(vec![Span::styled(
                "▊",
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD),
            )]));
        } else {
            // Show "DM is typing..." before any tokens arrive
            lines.push(Line::from(vec![
                Span::styled(
                    "DM is typing",
                    Style::default()
                        .fg(Color::Green)
                        .add_modifier(Modifier::ITALIC),
                ),
                Span::styled("...", Style::default().fg(Color::Green)),
            ]));
        }
    }

    let paragraph = Paragraph::new(lines);
    f.render_widget(paragraph, inner_area);

    // Show scroll indicator if scrolled
    if app.scroll_offset > 0 {
        let scroll_text = format!("↑ Scrolled up {} messages", app.scroll_offset);
        let scroll_area = Rect {
            x: inner_area.x + inner_area.width - scroll_text.len() as u16 - 2,
            y: inner_area.y,
            width: scroll_text.len() as u16 + 2,
            height: 1,
        };
        let scroll_widget = Paragraph::new(scroll_text)
            .style(Style::default().fg(Color::Yellow).bg(Color::DarkGray));
        f.render_widget(scroll_widget, scroll_area);
    }
}

/// Render inventory view
fn render_inventory(f: &mut Frame, app: &App, area: Rect) {
    let character = &app.game_state.character;

    let block = Block::default()
        .title("🎒 Inventory")
        .title_alignment(Alignment::Left)
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan))
        .border_type(BorderType::Rounded);

    let inner_area = block.inner(area);
    f.render_widget(block, area);

    if character.inventory.is_empty() {
        let text = Paragraph::new("Your inventory is empty.")
            .style(Style::default().fg(Color::DarkGray))
            .alignment(Alignment::Center);
        f.render_widget(text, inner_area);
        return;
    }

    let items: Vec<ListItem> = character
        .inventory
        .iter()
        .map(|item| {
            let is_equipped = character.equipped_weapon.as_ref() == Some(&item.id)
                || character.equipped_armor.as_ref() == Some(&item.id);
            let equipped = if is_equipped { " [E]" } else { "" };
            let style = if is_equipped {
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::White)
            };

            // Show item details based on type
            let details = match &item.item_type {
                crate::game::items::ItemType::Weapon(stats) => format!(" - DMG: {}", stats.damage),
                crate::game::items::ItemType::Armor(stats) => {
                    format!(" - AC: +{}", stats.armor_class)
                }
                crate::game::items::ItemType::Consumable(_) => format!(" x{}", item.quantity),
                crate::game::items::ItemType::Misc => String::new(),
            };

            ListItem::new(format!("• {}{}{}", item.name, equipped, details)).style(style)
        })
        .collect();

    let list = List::new(items);
    f.render_widget(list, inner_area);
}

/// Render detailed stats view with beautiful visual hierarchy
fn render_detailed_stats(f: &mut Frame, app: &App, area: Rect) {
    let character = &app.game_state.character;

    // Main character sheet border
    let block = Block::default()
        .title(vec![
            Span::raw("╔"),
            Span::styled(
                " CHARACTER ",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw("╗"),
        ])
        .title_alignment(Alignment::Center)
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Green))
        .border_type(BorderType::Double);

    let inner = block.inner(area);
    f.render_widget(block, area);

    // Split into sections: Header | Vitals | SPECIAL | Skills
    let sections = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),  // Character name and level
            Constraint::Length(6),  // Vitals (HP, AP, XP, Caps)
            Constraint::Length(11), // SPECIAL stats
            Constraint::Min(0),     // Skills
        ])
        .split(inner);

    // Section 1: Character Name and Level
    render_sheet_header(f, character, sections[0]);

    // Section 2: Vitals with progress bars
    render_sheet_vitals(f, character, sections[1]);

    // Section 3: SPECIAL with bars and benefits
    render_sheet_special(f, character, sections[2]);

    // Section 4: Top Skills
    render_sheet_skills(f, character, sections[3]);
}

/// Render character sheet header (name, level)
fn render_sheet_header(f: &mut Frame, character: &Character, area: Rect) {
    let text = vec![
        Line::from(""),
        Line::from(vec![
            Span::raw("  "),
            Span::styled(
                character.name.to_uppercase(),
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(format!(
                "{}Level {}",
                " ".repeat(area.width.saturating_sub(character.name.len() as u16 + 12) as usize),
                character.level
            )),
        ]),
        Line::from(Span::styled(
            "  ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━",
            Style::default().fg(Color::DarkGray),
        )),
    ];

    let paragraph = Paragraph::new(text);
    f.render_widget(paragraph, area);
}

/// Render vitals section with progress bars
fn render_sheet_vitals(f: &mut Frame, character: &Character, area: Rect) {
    let hp_ratio = character.current_hp as f64 / character.max_hp as f64;
    let ap_ratio = character.current_ap as f64 / character.max_ap as f64;
    let xp_next_level = (character.level + 1) * 1000;
    let xp_ratio = character.experience as f64 / xp_next_level as f64;

    let lines = vec![
        Line::from(""),
        Line::from(vec![
            Span::styled("  ❤ Health  ", Style::default().fg(Color::Red)),
            Span::raw(format!("{}/{}", character.current_hp, character.max_hp)),
            Span::raw("  "),
            Span::raw(make_progress_bar(hp_ratio, 20, Color::Red, Color::DarkGray)),
        ]),
        Line::from(vec![
            Span::styled("  ⚡ Action  ", Style::default().fg(Color::Yellow)),
            Span::raw(format!("{}/{}", character.current_ap, character.max_ap)),
            Span::raw("    "),
            Span::raw(make_progress_bar(
                ap_ratio,
                20,
                Color::Yellow,
                Color::DarkGray,
            )),
        ]),
        Line::from(vec![
            Span::styled("  ⭐ XP     ", Style::default().fg(Color::Magenta)),
            Span::raw(format!("{}/{}", character.experience, xp_next_level)),
            Span::raw("  "),
            Span::raw(make_progress_bar(
                xp_ratio,
                20,
                Color::Magenta,
                Color::DarkGray,
            )),
        ]),
        Line::from(vec![
            Span::styled("  💰 Caps   ", Style::default().fg(Color::Green)),
            Span::raw(format!("{}", character.caps)),
        ]),
    ];

    let paragraph = Paragraph::new(lines);
    f.render_widget(paragraph, area);
}

/// Render SPECIAL stats with bars and benefits
fn render_sheet_special(f: &mut Frame, character: &Character, area: Rect) {
    let special = &character.special;

    let lines = vec![
        Line::from(""),
        Line::from(vec![
            Span::raw("  ┌─ "),
            Span::styled(
                "S.P.E.C.I.A.L.",
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(" ───────────────────────────────────┐"),
        ]),
        make_special_line(
            "Strength",
            special.strength,
            "Melee DMG +",
            special.strength as i32,
        ),
        make_special_line(
            "Perception",
            special.perception,
            "Accuracy +",
            special.perception as i32,
        ),
        make_special_line(
            "Endurance",
            special.endurance,
            "HP +",
            (special.endurance * 2) as i32,
        ),
        make_special_line(
            "Charisma",
            special.charisma,
            if special.charisma >= 5 {
                "Barter +"
            } else {
                "Barter "
            },
            if special.charisma >= 5 {
                (special.charisma - 5) as i32 * 5
            } else {
                -((5 - special.charisma) as i32 * 10)
            },
        ),
        make_special_line(
            "Intelligence",
            special.intelligence,
            "Skill Pts +",
            (special.intelligence * 2) as i32,
        ),
        make_special_line(
            "Agility",
            special.agility,
            "AC ",
            10 + special.agility as i32,
        ),
        make_special_line("Luck", special.luck, "Critical +", special.luck as i32),
        Line::from("  └────────────────────────────────────────────────────┘"),
    ];

    let paragraph = Paragraph::new(lines);
    f.render_widget(paragraph, area);
}

/// Render top skills section
fn render_sheet_skills(f: &mut Frame, character: &Character, area: Rect) {
    let skills = &character.skills;

    // Get top 4 skills
    let mut skill_list = vec![
        ("Small Guns", skills.small_guns),
        ("Big Guns", skills.big_guns),
        ("Energy Weapons", skills.energy_weapons),
        ("Melee Weapons", skills.melee_weapons),
        ("Unarmed", skills.unarmed),
        ("Speech", skills.speech),
        ("Science", skills.science),
        ("Lockpick", skills.lockpick),
        ("Repair", skills.repair),
        ("Barter", skills.barter),
        ("Sneak", skills.sneak),
        ("Doctor", skills.doctor),
    ];
    skill_list.sort_by(|a, b| b.1.cmp(&a.1));
    let top_skills: Vec<_> = skill_list.into_iter().take(4).collect();

    let mut lines = vec![
        Line::from(""),
        Line::from(vec![
            Span::raw("  ┌─ "),
            Span::styled(
                "TOP SKILLS",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(" ──────────────────────────────────┐"),
        ]),
    ];

    for (name, value) in top_skills {
        lines.push(make_skill_line(name, value));
    }

    lines.push(Line::from(
        "  └────────────────────────────────────────────────────┘",
    ));

    let paragraph = Paragraph::new(lines);
    f.render_widget(paragraph, area);
}

/// Helper: Create a SPECIAL stat line with bar and benefit
fn make_special_line(
    name: &str,
    value: u8,
    benefit_label: &str,
    benefit_value: i32,
) -> Line<'static> {
    let bar = make_stat_bar(value, 10);
    let benefit = if benefit_value >= 0 {
        format!("{}  {}", benefit_label, benefit_value)
    } else {
        format!("{}{}", benefit_label, benefit_value)
    };

    Line::from(vec![
        Span::raw(format!("  │ {:<13} ", name)),
        Span::styled(bar, Style::default().fg(Color::Green)),
        Span::raw(format!(" {:2}   ", value)),
        Span::styled(
            format!("{:<15}", benefit),
            Style::default().fg(Color::DarkGray),
        ),
        Span::raw("│"),
    ])
}

/// Helper: Create a skill line with bar and rating
fn make_skill_line(name: &str, value: u8) -> Line<'static> {
    let ratio = value as f64 / 100.0;
    let bar = make_progress_bar(ratio, 20, Color::Cyan, Color::DarkGray);

    let rating = match value {
        0..=20 => ("Novice", Color::Red),
        21..=40 => ("Poor", Color::LightRed),
        41..=60 => ("Fair", Color::Yellow),
        61..=80 => ("Good", Color::LightGreen),
        81..=95 => ("Excellent", Color::Green),
        _ => ("Master", Color::Magenta),
    };

    Line::from(vec![
        Span::raw(format!("  │ {:<15} ", name)),
        Span::raw(bar),
        Span::raw(format!("  {:3}  ", value)),
        Span::styled(format!("{:<10}", rating.0), Style::default().fg(rating.1)),
        Span::raw("│"),
    ])
}

/// Helper: Make a stat bar (for values 1-10)
fn make_stat_bar(value: u8, max: u8) -> String {
    let filled = value.min(max);
    let empty = max - filled;
    format!(
        "[{}{}]",
        "█".repeat(filled as usize),
        "░".repeat(empty as usize)
    )
}

/// Helper: Make a progress bar
fn make_progress_bar(
    ratio: f64,
    width: usize,
    _filled_color: Color,
    _empty_color: Color,
) -> String {
    let filled_count = (ratio * width as f64).round() as usize;
    let empty_count = width.saturating_sub(filled_count);
    format!("{}{}", "█".repeat(filled_count), "░".repeat(empty_count))
}

/// Render input bar - Pip-Boy themed with loading spinner
fn render_input_bar(f: &mut Frame, app: &App, area: Rect) {
    use crate::tui::theme::PipBoyTheme;

    let (input_text, style) = if app.waiting_for_ai {
        // Show animated loading spinner
        let spinner_frame = app.loading_spinner.current();
        let text = format!("{} Waiting for Dungeon Master...", spinner_frame);
        (text, PipBoyTheme::loading())
    } else {
        (format!("> {}", app.input), PipBoyTheme::text())
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(PipBoyTheme::border())
        .border_type(BorderType::Double);

    let paragraph = Paragraph::new(input_text).style(style).block(block);

    f.render_widget(paragraph, area);

    // Set cursor position
    if !app.waiting_for_ai {
        f.set_cursor_position((area.x + app.cursor_position as u16 + 3, area.y + 1));
    }
}

/// Create a visual stat bar (e.g., "████████░░" for 8/10)
fn create_stat_bar(value: u8) -> String {
    let filled = value.min(10);
    let empty = 10 - filled;
    format!(
        "{}{}",
        "█".repeat(filled as usize),
        "░".repeat(empty as usize)
    )
}

/// Wrap text to fit within a given width
fn wrap_text(text: &str, width: usize) -> Vec<String> {
    if width == 0 {
        return vec![text.to_string()];
    }

    let mut lines = Vec::new();
    let mut current_line = String::new();
    let mut current_width = 0;

    for word in text.split_whitespace() {
        let word_len = word.chars().count();

        if current_width + word_len + 1 > width && !current_line.is_empty() {
            lines.push(current_line);
            current_line = String::new();
            current_width = 0;
        }

        if !current_line.is_empty() {
            current_line.push(' ');
            current_width += 1;
        }

        current_line.push_str(word);
        current_width += word_len;
    }

    if !current_line.is_empty() {
        lines.push(current_line);
    }

    if lines.is_empty() {
        lines.push(String::new());
    }

    lines
}

/// Truncate string to fit within a given length
fn truncate_string(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len.saturating_sub(3)])
    }
}

/// Render game over screen
fn render_game_over(f: &mut Frame, app: &App, area: Rect) {
    if let Some(ref death_info) = app.death_info {
        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Red))
            .border_type(BorderType::Double);

        let inner = block.inner(area);
        f.render_widget(block, area);

        let lines = vec![
            Line::from(""),
            Line::from(""),
            Line::from(""),
            Line::from(vec![Span::styled(
                "═══════════════════════════════════════════════════",
                Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
            )]),
            Line::from(vec![Span::styled(
                "                    GAME OVER",
                Style::default()
                    .fg(Color::LightRed)
                    .add_modifier(Modifier::BOLD),
            )]),
            Line::from(vec![Span::styled(
                "═══════════════════════════════════════════════════",
                Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
            )]),
            Line::from(""),
            Line::from(""),
            Line::from(vec![Span::styled(
                format!("☠ {} ☠", death_info.cause),
                Style::default().fg(Color::Red),
            )]),
            Line::from(""),
            Line::from(vec![Span::styled(
                "─────────────────────────────────────────────────",
                Style::default().fg(Color::DarkGray),
            )]),
            Line::from(""),
            Line::from(vec![
                Span::styled("Location: ", Style::default().fg(Color::Yellow)),
                Span::styled(&death_info.location, Style::default().fg(Color::White)),
            ]),
            Line::from(vec![
                Span::styled("Day:      ", Style::default().fg(Color::Yellow)),
                Span::styled(
                    format!("{}", death_info.day),
                    Style::default().fg(Color::White),
                ),
            ]),
            Line::from(vec![
                Span::styled("Level:    ", Style::default().fg(Color::Yellow)),
                Span::styled(
                    format!("{}", death_info.level),
                    Style::default().fg(Color::White),
                ),
            ]),
            Line::from(""),
            Line::from(vec![Span::styled(
                "─────────────────────────────────────────────────",
                Style::default().fg(Color::DarkGray),
            )]),
            Line::from(""),
            Line::from(""),
            Line::from(vec![Span::styled(
                "Press 'R' to restart or 'Q' to quit",
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            )]),
        ];

        let paragraph = Paragraph::new(lines)
            .alignment(Alignment::Center)
            .block(Block::default());
        f.render_widget(paragraph, inner);
    }
}

/// Render status bar (always visible at bottom of screen)
fn render_status_bar(f: &mut Frame, app: &App, area: Rect) {
    let character = &app.game_state.character;
    let location = &app.game_state.location;
    let day = app.game_state.day;

    // Calculate HP color based on health percentage
    let hp_ratio = character.current_hp as f64 / character.max_hp as f64;
    let hp_color = if hp_ratio > 0.6 {
        Color::Green
    } else if hp_ratio > 0.3 {
        Color::Yellow
    } else {
        Color::Red
    };

    // Build status bar content
    let status_content = vec![
        Span::raw(" "),
        Span::styled(
            truncate_string(location, 25),
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw("  │  "),
        Span::styled("❤", Style::default().fg(hp_color)),
        Span::raw(" "),
        Span::styled(
            format!("{}/{}", character.current_hp, character.max_hp),
            Style::default().fg(Color::White),
        ),
        Span::raw("  "),
        Span::styled("⚡", Style::default().fg(Color::Yellow)),
        Span::raw(" "),
        Span::styled(
            format!("{}/{}", character.current_ap, character.max_ap),
            Style::default().fg(Color::White),
        ),
        Span::raw("  │  "),
        Span::styled(format!("Day {}", day), Style::default().fg(Color::Cyan)),
        Span::raw("  │  "),
        Span::styled("💰", Style::default().fg(Color::Yellow)),
        Span::raw(" "),
        Span::styled(
            format!("{}", character.caps),
            Style::default().fg(Color::Green),
        ),
        Span::raw(" "),
    ];

    // Create border characters
    let border_width = area.width as usize;
    let top_border = "═".repeat(border_width);
    let bottom_border = "═".repeat(border_width);

    let text = vec![
        Line::from(Span::styled(top_border, Style::default().fg(Color::Green))),
        Line::from(status_content),
        Line::from(Span::styled(
            bottom_border,
            Style::default().fg(Color::Green),
        )),
    ];

    let paragraph = Paragraph::new(text)
        .style(Style::default().bg(Color::Black))
        .alignment(Alignment::Left);

    f.render_widget(paragraph, area);
}
