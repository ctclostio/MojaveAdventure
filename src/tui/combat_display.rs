use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Paragraph},
    Frame,
};

use crate::game::{
    character::Character,
    combat::{CombatState, Enemy},
};
use crate::tui::theme::PipBoyTheme;

/// Render the tactical combat display
pub fn render_combat_display(
    frame: &mut Frame,
    area: Rect,
    character: &Character,
    combat: &CombatState,
    animation_manager: &crate::tui::animations::AnimationManager,
    should_flicker: bool,
) {
    use crate::tui::theme::RetroEffects;

    // Main combat border with title - Pip-Boy themed with optional flicker
    let title = format!(" ⚔ COMBAT - ROUND {} ", combat.round);
    let border_style = if should_flicker {
        RetroEffects::flicker_style()
    } else {
        PipBoyTheme::border_active()
    };

    let main_block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Double)
        .border_style(border_style)
        .title(title)
        .title_alignment(Alignment::Center);

    let inner = main_block.inner(area);
    frame.render_widget(main_block, area);

    // Split into player column (left) and enemy column (right)
    let columns = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(40), Constraint::Percentage(60)])
        .split(inner);

    // Render player panel on the left
    render_player_panel(frame, columns[0], character, animation_manager);

    // Render enemies panel on the right
    render_enemies_panel(frame, columns[1], &combat.enemies, animation_manager);

    // Render action bar at the bottom if there's room
    if area.height > 20 {
        let action_area = Rect {
            x: area.x,
            y: area.y + area.height - 2,
            width: area.width,
            height: 2,
        };
        render_action_bar(frame, action_area, character);
    }

    // Render dice roll animation if active (overlay in center)
    if let Some((is_rolling, display_value, final_result, modifier)) =
        animation_manager.get_dice_animation_state()
    {
        render_dice_roll_overlay(
            frame,
            area,
            is_rolling,
            display_value,
            final_result,
            modifier,
        );
    }
}

/// Render player character panel
fn render_player_panel(
    frame: &mut Frame,
    area: Rect,
    character: &Character,
    animation_manager: &crate::tui::animations::AnimationManager,
) {
    // Create vertical layout for player info
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // "YOU" header
            Constraint::Min(10),   // Player stats box
        ])
        .split(area);

    // "YOU" header
    let header = Paragraph::new("YOU")
        .style(
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )
        .alignment(Alignment::Center);
    frame.render_widget(header, chunks[0]);

    // Player stats box
    let player_box = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Green));

    let player_inner = player_box.inner(chunks[1]);
    frame.render_widget(player_box, chunks[1]);

    // Build player info content
    let mut lines = vec![];

    // Character name (centered)
    lines.push(Line::from(vec![Span::styled(
        format!(" {}", character.name),
        Style::default()
            .fg(Color::White)
            .add_modifier(Modifier::BOLD),
    )]));

    // HP bar - use animated value if animation is active
    let display_hp = animation_manager
        .get_animated_hp(character.current_hp)
        .unwrap_or(character.current_hp);

    lines.push(Line::from(""));
    lines.push(Line::from(create_hp_bar(display_hp, character.max_hp, 14)));
    lines.push(Line::from(vec![Span::styled(
        format!(" {}/{} HP", display_hp, character.max_hp),
        Style::default().fg(Color::White),
    )]));

    // AP bar
    lines.push(Line::from(""));
    lines.push(Line::from(create_ap_bar(
        character.current_ap,
        character.max_ap,
        14,
    )));
    lines.push(Line::from(vec![Span::styled(
        format!(" {}/{} AP", character.current_ap, character.max_ap),
        Style::default().fg(Color::Cyan),
    )]));

    // Equipped weapon
    lines.push(Line::from(""));
    let weapon_name = get_weapon_display_name(character);
    lines.push(Line::from(vec![
        Span::raw(" 🔫 "),
        Span::styled(weapon_name, Style::default().fg(Color::Yellow)),
    ]));

    let damage = character.get_equipped_damage();
    lines.push(Line::from(vec![Span::styled(
        format!("    {}", damage),
        Style::default().fg(Color::Gray),
    )]));

    let content = Paragraph::new(lines).alignment(Alignment::Left);
    frame.render_widget(content, player_inner);
}

/// Render enemies panel
fn render_enemies_panel(
    frame: &mut Frame,
    area: Rect,
    enemies: &[Enemy],
    animation_manager: &crate::tui::animations::AnimationManager,
) {
    // Create vertical layout
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // "ENEMIES" header
            Constraint::Min(10),   // Enemy boxes
            Constraint::Length(2), // Initiative order
        ])
        .split(area);

    // "ENEMIES" header
    let header = Paragraph::new("ENEMIES")
        .style(Style::default().fg(Color::Red).add_modifier(Modifier::BOLD))
        .alignment(Alignment::Center);
    frame.render_widget(header, chunks[0]);

    // Enemy boxes area - split horizontally for multiple enemies
    // Include enemies that are either alive OR currently fading out
    let visible_enemies: Vec<(usize, &Enemy)> = enemies
        .iter()
        .enumerate()
        .filter(|(idx, e)| e.is_alive() || animation_manager.is_enemy_fading(*idx))
        .collect();

    if visible_enemies.is_empty() {
        let no_enemies = Paragraph::new("All enemies defeated!")
            .style(Style::default().fg(Color::Green))
            .alignment(Alignment::Center);
        frame.render_widget(no_enemies, chunks[1]);
    } else {
        // Layout enemies horizontally (max 3 per row)
        let enemy_count = visible_enemies.len().min(3);
        let mut constraints = vec![];
        for _ in 0..enemy_count {
            constraints.push(Constraint::Percentage((100 / enemy_count) as u16));
        }

        let enemy_areas = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(constraints)
            .split(chunks[1]);

        // Render each enemy
        for (i, (original_idx, enemy)) in visible_enemies.iter().take(3).enumerate() {
            render_enemy_box(
                frame,
                enemy_areas[i],
                *original_idx + 1,
                enemy,
                *original_idx,
                animation_manager,
            );
        }

        // If more than 3 enemies, show count
        if visible_enemies.len() > 3 {
            let overflow = Paragraph::new(format!("  + {} more...", visible_enemies.len() - 3))
                .style(Style::default().fg(Color::Gray));
            frame.render_widget(overflow, enemy_areas[2]);
        }
    }

    // Initiative order
    render_initiative_order(frame, chunks[2], enemies);
}

/// Render a single enemy box
fn render_enemy_box(
    frame: &mut Frame,
    area: Rect,
    number: usize,
    enemy: &Enemy,
    enemy_index: usize,
    animation_manager: &crate::tui::animations::AnimationManager,
) {
    // Get opacity for fadeout effect (1.0 = fully visible, 0.0 = invisible)
    let opacity = animation_manager.get_enemy_opacity(enemy_index);

    // Apply opacity to colors - darken based on opacity
    let border_color = if opacity < 1.0 {
        Color::DarkGray // Fade to dark gray when dying
    } else {
        Color::Red
    };

    let enemy_box = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(border_color));

    let inner = enemy_box.inner(area);
    frame.render_widget(enemy_box, area);

    let mut lines = vec![];

    // Apply opacity to text colors
    let text_color = if opacity < 0.5 {
        Color::DarkGray
    } else if opacity < 1.0 {
        Color::Gray
    } else {
        Color::White
    };

    // Enemy name with number
    lines.push(Line::from(vec![Span::styled(
        format!(" [{}] {}", number, enemy.name),
        Style::default().fg(text_color).add_modifier(Modifier::BOLD),
    )]));

    // HP bar
    lines.push(Line::from(""));
    let bar_width = (inner.width.saturating_sub(4)).min(12) as usize;
    lines.push(Line::from(create_hp_bar(
        enemy.current_hp,
        enemy.max_hp,
        bar_width,
    )));
    lines.push(Line::from(vec![Span::styled(
        format!(" {}/{} HP", enemy.current_hp, enemy.max_hp),
        Style::default().fg(text_color),
    )]));

    // Status effects
    if !enemy.is_alive() && opacity > 0.0 {
        lines.push(Line::from(""));
        lines.push(Line::from(vec![Span::styled(
            " Defeated! ☠",
            Style::default().fg(Color::DarkGray),
        )]));
    } else if enemy.current_hp < enemy.max_hp / 4 {
        lines.push(Line::from(""));
        lines.push(Line::from(vec![Span::styled(
            " Wounded ⚠",
            Style::default().fg(if opacity < 1.0 {
                Color::DarkGray
            } else {
                Color::Yellow
            }),
        )]));
    }

    let content = Paragraph::new(lines).alignment(Alignment::Left);
    frame.render_widget(content, inner);
}

/// Render initiative order display
fn render_initiative_order(frame: &mut Frame, area: Rect, enemies: &[Enemy]) {
    let mut order_text = vec![Span::raw("Initiative: ")];

    // Player always first in current system
    order_text.push(Span::styled(
        "[YOU]",
        Style::default()
            .fg(Color::Green)
            .add_modifier(Modifier::BOLD),
    ));

    // Add alive enemies
    for (i, enemy) in enemies.iter().enumerate() {
        if enemy.is_alive() {
            order_text.push(Span::raw(" → "));
            order_text.push(Span::styled(
                format!("[{}]", i + 1),
                Style::default().fg(Color::Red),
            ));
        }
    }

    let initiative = Paragraph::new(Line::from(order_text)).alignment(Alignment::Center);
    frame.render_widget(initiative, area);
}

/// Render action bar with available commands
fn render_action_bar(frame: &mut Frame, area: Rect, _character: &Character) {
    let actions = vec![
        Span::raw("Actions: "),
        Span::styled("[A]", Style::default().fg(Color::Yellow)),
        Span::raw("ttack "),
        Span::styled("[U]", Style::default().fg(Color::Yellow)),
        Span::raw("se Item "),
        Span::styled("[R]", Style::default().fg(Color::Yellow)),
        Span::raw("un  |  AP Cost: "),
        Span::styled("Attack=4", Style::default().fg(Color::Cyan)),
        Span::raw(" "),
        Span::styled("Use=2", Style::default().fg(Color::Cyan)),
    ];

    let action_bar = Paragraph::new(Line::from(actions))
        .style(Style::default().fg(Color::Gray))
        .alignment(Alignment::Center);

    frame.render_widget(action_bar, area);
}

/// Create an HP bar visualization - Pip-Boy themed
fn create_hp_bar(current: i32, max: i32, width: usize) -> Vec<Span<'static>> {
    let percentage = if max > 0 {
        (current as f32 / max as f32).clamp(0.0, 1.0)
    } else {
        0.0
    };

    let filled = (width as f32 * percentage).round() as usize;
    let empty = width.saturating_sub(filled);

    // Pip-Boy themed HP colors
    let style = if percentage > 0.6 {
        PipBoyTheme::health_full()
    } else if percentage > 0.3 {
        PipBoyTheme::health_medium()
    } else {
        PipBoyTheme::health_low()
    };

    vec![
        Span::raw(" ❤ "),
        Span::styled("█".repeat(filled), style),
        Span::styled("░".repeat(empty), PipBoyTheme::text_very_dim()),
    ]
}

/// Create an AP bar visualization - Pip-Boy themed
fn create_ap_bar(current: i32, max: i32, width: usize) -> Vec<Span<'static>> {
    let percentage = if max > 0 {
        (current as f32 / max as f32).clamp(0.0, 1.0)
    } else {
        0.0
    };

    let filled = (width as f32 * percentage).round() as usize;
    let empty = width.saturating_sub(filled);

    vec![
        Span::raw(" ⚡ "),
        Span::styled("█".repeat(filled), PipBoyTheme::energy()),
        Span::styled("░".repeat(empty), PipBoyTheme::text_very_dim()),
    ]
}

/// Get display name for equipped weapon
fn get_weapon_display_name(character: &Character) -> String {
    if let Some(weapon_id) = &character.equipped_weapon {
        // Try to find weapon in inventory for display name
        for item in &character.inventory {
            if &item.id == weapon_id {
                return item.name.clone();
            }
        }
        // Fallback to formatted ID
        weapon_id
            .replace('_', " ")
            .split_whitespace()
            .map(|word| {
                let mut chars = word.chars();
                match chars.next() {
                    None => String::new(),
                    Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
                }
            })
            .collect::<Vec<_>>()
            .join(" ")
    } else {
        "Unarmed".to_string()
    }
}

/// Render dice roll animation overlay (appears in center of screen)
fn render_dice_roll_overlay(
    frame: &mut Frame,
    area: Rect,
    is_rolling: bool,
    display_value: u8,
    final_result: u8,
    modifier: i32,
) {
    // Create centered popup area
    let popup_width = 30;
    let popup_height = 7;
    let popup_x = area.x + (area.width.saturating_sub(popup_width)) / 2;
    let popup_y = area.y + (area.height.saturating_sub(popup_height)) / 2;

    let popup_area = Rect {
        x: popup_x,
        y: popup_y,
        width: popup_width,
        height: popup_height,
    };

    // Dice box with fancy border
    let dice_box = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Double)
        .border_style(Style::default().fg(if is_rolling {
            Color::Yellow
        } else {
            Color::Green
        }))
        .title(if is_rolling {
            " 🎲 ROLLING... "
        } else {
            " 🎲 RESULT "
        })
        .title_alignment(Alignment::Center);

    let inner = dice_box.inner(popup_area);
    frame.render_widget(dice_box, popup_area);

    // Build content
    let mut lines = vec![];

    lines.push(Line::from(""));

    if is_rolling {
        // Show spinning/changing number during roll
        lines.push(Line::from(vec![Span::styled(
            format!("        d20: {}", display_value),
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )]));
    } else {
        // Show final result
        let total = final_result as i32 + modifier;
        let result_color = if final_result == 20 {
            Color::Green // Critical success
        } else if final_result == 1 {
            Color::Red // Critical failure
        } else {
            Color::Cyan
        };

        lines.push(Line::from(vec![Span::styled(
            format!("      Roll: {}", final_result),
            Style::default()
                .fg(result_color)
                .add_modifier(Modifier::BOLD),
        )]));

        lines.push(Line::from(vec![Span::styled(
            format!(
                "  Modifier: {}",
                if modifier >= 0 {
                    format!("+{}", modifier)
                } else {
                    modifier.to_string()
                }
            ),
            Style::default().fg(Color::Gray),
        )]));

        lines.push(Line::from(""));

        lines.push(Line::from(vec![Span::styled(
            format!("     Total: {}", total),
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        )]));
    }

    let content = Paragraph::new(lines).alignment(Alignment::Left);
    frame.render_widget(content, inner);
}
