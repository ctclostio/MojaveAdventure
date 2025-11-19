//! Worldbook UI rendering functions

use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, List, ListItem, Paragraph},
    Frame,
};

use super::app::App;
use crate::tui::worldbook_browser::*;

/// Main worldbook renderer
pub fn render_worldbook(f: &mut Frame, app: &App, area: Rect) {
    let browser = &app.worldbook_browser;

    // Main layout: [Tab bar | Content area]
    let main_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Tab bar
            Constraint::Min(0),    // Content
        ])
        .split(area);

    // Render tab bar
    render_worldbook_tabs(f, browser, main_chunks[0]);

    // Split content into list and detail panels
    let content_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(40), // List panel
            Constraint::Percentage(60), // Detail panel
        ])
        .split(main_chunks[1]);

    // Render based on active tab
    match browser.active_tab {
        WorldbookTab::Locations => {
            render_locations_list(f, app, content_chunks[0]);
            render_location_detail(f, app, content_chunks[1]);
        }
        WorldbookTab::NPCs => {
            render_npcs_list(f, app, content_chunks[0]);
            render_npc_detail(f, app, content_chunks[1]);
        }
        WorldbookTab::Events => {
            render_events_list(f, app, content_chunks[0]);
            render_event_detail(f, app, content_chunks[1]);
        }
        WorldbookTab::Search => {
            render_search_view(f, app, main_chunks[1]);
        }
    }

    // Render help bar
    render_worldbook_help(f, area);
}

/// Render tab bar
fn render_worldbook_tabs(f: &mut Frame, browser: &WorldbookBrowser, area: Rect) {
    let tabs = [
        WorldbookTab::Locations,
        WorldbookTab::NPCs,
        WorldbookTab::Events,
        WorldbookTab::Search,
    ];

    let is_tab_focused = browser.is_tab_bar_focused();

    let mut tab_spans = vec![Span::styled("  ", Style::default())];

    for (i, tab) in tabs.iter().enumerate() {
        let is_active = *tab == browser.active_tab;
        let style = if is_active {
            Style::default()
                .fg(Color::Black)
                .bg(Color::Cyan)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::DarkGray)
        };

        if i > 0 {
            tab_spans.push(Span::styled("  ", Style::default()));
        }

        tab_spans.push(Span::styled(format!(" {} ", tab.as_str()), style));
    }

    // Add focus indicator if tab bar is focused
    if is_tab_focused {
        tab_spans.push(Span::styled(
            "  ◄ Use ←→ arrows to navigate tabs",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::DIM),
        ));
    }

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(if is_tab_focused {
            Style::default().fg(Color::Yellow)
        } else {
            Style::default().fg(Color::Cyan)
        })
        .border_type(BorderType::Rounded);

    let paragraph = Paragraph::new(Line::from(tab_spans)).block(block);
    f.render_widget(paragraph, area);
}

/// Render locations list
fn render_locations_list(f: &mut Frame, app: &App, area: Rect) {
    let worldbook = &app.game_state.worldbook;
    let browser = &app.worldbook_browser;
    let is_list_focused = browser.is_list_focused();

    let block = Block::default()
        .title("📍 Locations")
        .borders(Borders::ALL)
        .border_style(if is_list_focused {
            Style::default().fg(Color::Yellow)
        } else {
            Style::default().fg(Color::Green)
        })
        .border_type(BorderType::Rounded);

    let inner_area = block.inner(area);
    f.render_widget(block, area);

    if worldbook.locations.is_empty() {
        let text = Paragraph::new("No locations discovered yet.")
            .style(Style::default().fg(Color::DarkGray))
            .alignment(Alignment::Center);
        f.render_widget(text, inner_area);
        return;
    }

    let locations = browser.get_sorted_locations(worldbook);
    let mut items = vec![];

    for (i, (_id, location)) in locations.iter().enumerate() {
        let is_selected = i == browser.selected_index;
        let is_current = Some(&location.id) == worldbook.current_location.as_ref();

        let visit_status = get_visit_status(location);
        let prefix = if browser.is_expanded(&location.id) {
            "▾"
        } else {
            "▸"
        };

        // Enhanced highlighting when list is focused and item is selected
        let (style, selector) = if is_selected && is_list_focused {
            (
                Style::default()
                    .fg(Color::Black)
                    .bg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
                "► ",
            )
        } else if is_selected {
            (
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
                "  ",
            )
        } else if is_current {
            (Style::default().fg(Color::Cyan), "  ")
        } else {
            (Style::default().fg(Color::White), "  ")
        };

        let line = format!(
            "{}{} {:<28} {:>15}",
            selector,
            prefix,
            truncate_string(&location.name, 26),
            visit_status
        );

        items.push(ListItem::new(line).style(style));

        // Show children if expanded
        if browser.is_expanded(&location.id) && !location.npcs_present.is_empty() {
            for npc_id in &location.npcs_present {
                if let Some(npc) = worldbook.npcs.get(npc_id) {
                    let child_line = format!("      └─ {}", npc.name);
                    let child_style = Style::default().fg(Color::DarkGray);
                    items.push(ListItem::new(child_line).style(child_style));
                }
            }
        }
    }

    let list = List::new(items);
    f.render_widget(list, inner_area);
}

/// Render location detail
fn render_location_detail(f: &mut Frame, app: &App, area: Rect) {
    let worldbook = &app.game_state.worldbook;
    let browser = &app.worldbook_browser;

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Green))
        .border_type(BorderType::Rounded);

    let inner_area = block.inner(area);
    f.render_widget(block, area);

    let locations = browser.get_sorted_locations(worldbook);
    if browser.selected_index >= locations.len() {
        return;
    }

    let (_id, location) = locations[browser.selected_index];

    let mut lines = vec![
        Line::from(Span::styled(
            location.name.as_str(),
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(Span::styled(
            format!("Type: {}", location.location_type),
            Style::default().fg(Color::Yellow),
        )),
        Line::from(""),
    ];

    // Description
    let wrapped_desc = wrap_text(&location.description, inner_area.width as usize - 2);
    for line in wrapped_desc {
        lines.push(Line::from(line));
    }
    lines.push(Line::from(""));

    // Atmosphere
    if let Some(atmosphere) = &location.atmosphere {
        lines.push(Line::from(vec![
            Span::styled("Atmosphere: ", Style::default().fg(Color::DarkGray)),
            Span::styled(atmosphere.as_str(), Style::default().fg(Color::White)),
        ]));
        lines.push(Line::from(""));
    }

    // Visit info
    if location.visit_count > 0 {
        lines.push(Line::from(Span::styled(
            "═══ Visit History ═══",
            Style::default().fg(Color::DarkGray),
        )));
        lines.push(Line::from(format!("Visits: {}", location.visit_count)));
        if let Some(first) = &location.first_visited {
            lines.push(Line::from(format!(
                "First: {}",
                format_relative_time(first)
            )));
        }
        if let Some(last) = &location.last_visited {
            lines.push(Line::from(format!("Last: {}", format_relative_time(last))));
        }
        lines.push(Line::from(""));
    }

    // NPCs present
    if !location.npcs_present.is_empty() {
        lines.push(Line::from(Span::styled(
            "═══ NPCs Present ═══",
            Style::default().fg(Color::DarkGray),
        )));
        for npc_id in &location.npcs_present {
            if let Some(npc) = worldbook.npcs.get(npc_id) {
                let (disp_text, _emoji) = get_disposition_string(npc.disposition);
                lines.push(Line::from(vec![
                    Span::styled(" • ", Style::default().fg(Color::Yellow)),
                    Span::styled(npc.name.as_str(), Style::default().fg(Color::White)),
                    Span::styled(
                        format!(" ({}, {})", npc.role, disp_text),
                        Style::default().fg(Color::DarkGray),
                    ),
                ]));
            }
        }
        lines.push(Line::from(""));
    }

    // Notes
    if !location.notes.is_empty() {
        lines.push(Line::from(Span::styled(
            "═══ Notes ═══",
            Style::default().fg(Color::DarkGray),
        )));
        for note in &location.notes {
            lines.push(Line::from(format!(" • {}", note)));
        }
    }

    let paragraph = Paragraph::new(lines);
    f.render_widget(paragraph, inner_area);
}

/// Render NPCs list
fn render_npcs_list(f: &mut Frame, app: &App, area: Rect) {
    let worldbook = &app.game_state.worldbook;
    let browser = &app.worldbook_browser;
    let is_list_focused = browser.is_list_focused();

    let block = Block::default()
        .title("👥 NPCs")
        .borders(Borders::ALL)
        .border_style(if is_list_focused {
            Style::default().fg(Color::Yellow)
        } else {
            Style::default().fg(Color::Green)
        })
        .border_type(BorderType::Rounded);

    let inner_area = block.inner(area);
    f.render_widget(block, area);

    if worldbook.npcs.is_empty() {
        let text = Paragraph::new("No NPCs met yet.")
            .style(Style::default().fg(Color::DarkGray))
            .alignment(Alignment::Center);
        f.render_widget(text, inner_area);
        return;
    }

    let npcs = browser.get_sorted_npcs(worldbook);
    let items: Vec<ListItem> = npcs
        .iter()
        .enumerate()
        .map(|(i, (_id, npc))| {
            let is_selected = i == browser.selected_index;

            // Enhanced highlighting when list is focused and item is selected
            let (style, selector) = if is_selected && is_list_focused {
                (
                    Style::default()
                        .fg(Color::Black)
                        .bg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                    "► ",
                )
            } else if is_selected {
                (
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                    "  ",
                )
            } else if !npc.alive {
                (Style::default().fg(Color::DarkGray), "  ")
            } else {
                (Style::default().fg(Color::White), "  ")
            };

            let status = if !npc.alive { " [DEAD]" } else { "" };
            let disp_indicator = match npc.disposition {
                d if d >= 75 => "💚",
                d if d >= 25 => "💛",
                d if d >= -25 => "🟠",
                d if d >= -75 => "🔴",
                _ => "💀",
            };

            let line = format!(
                "{}{} {:<23} {:<10}{}",
                selector,
                disp_indicator,
                truncate_string(&npc.name, 21),
                npc.role,
                status
            );

            ListItem::new(line).style(style)
        })
        .collect();

    let list = List::new(items);
    f.render_widget(list, inner_area);
}

/// Render NPC detail
fn render_npc_detail(f: &mut Frame, app: &App, area: Rect) {
    let worldbook = &app.game_state.worldbook;
    let browser = &app.worldbook_browser;

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Green))
        .border_type(BorderType::Rounded);

    let inner_area = block.inner(area);
    f.render_widget(block, area);

    let npcs = browser.get_sorted_npcs(worldbook);
    if browser.selected_index >= npcs.len() {
        return;
    }

    let (_id, npc) = npcs[browser.selected_index];

    let (disp_text, emoji) = get_disposition_string(npc.disposition);

    let mut lines = vec![
        Line::from(vec![
            Span::styled(
                npc.name.as_str(),
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                if !npc.alive { " [DECEASED]" } else { "" },
                Style::default().fg(Color::Red),
            ),
        ]),
        Line::from(Span::styled(
            format!("Role: {}", npc.role),
            Style::default().fg(Color::Yellow),
        )),
        Line::from(""),
    ];

    // Disposition
    lines.push(Line::from(vec![
        Span::styled("Disposition: ", Style::default().fg(Color::DarkGray)),
        Span::styled(
            format!("{} {} ({}/100)", emoji, disp_text, npc.disposition),
            Style::default().fg(Color::White),
        ),
    ]));
    lines.push(Line::from(""));

    // Location
    if let Some(loc_id) = &npc.current_location {
        if let Some(location) = worldbook.locations.get(loc_id) {
            lines.push(Line::from(vec![
                Span::styled("Location: ", Style::default().fg(Color::DarkGray)),
                Span::styled(location.name.as_str(), Style::default().fg(Color::White)),
            ]));
            lines.push(Line::from(""));
        }
    }

    // Personality
    if !npc.personality.is_empty() {
        lines.push(Line::from(Span::styled(
            "═══ Personality ═══",
            Style::default().fg(Color::DarkGray),
        )));
        let personality_text = npc.personality.join(", ");
        lines.push(Line::from(personality_text));
        lines.push(Line::from(""));
    }

    // Knowledge
    if !npc.knowledge.is_empty() {
        lines.push(Line::from(Span::styled(
            "═══ Knowledge ═══",
            Style::default().fg(Color::DarkGray),
        )));
        for knowledge in &npc.knowledge {
            lines.push(Line::from(format!(" • {}", knowledge)));
        }
        lines.push(Line::from(""));
    }

    // Notes
    if !npc.notes.is_empty() {
        lines.push(Line::from(Span::styled(
            "═══ Notes ═══",
            Style::default().fg(Color::DarkGray),
        )));
        let wrapped_notes = wrap_text(&npc.notes, inner_area.width as usize - 2);
        for line in wrapped_notes {
            lines.push(Line::from(line));
        }
    }

    let paragraph = Paragraph::new(lines);
    f.render_widget(paragraph, inner_area);
}

/// Render events list
fn render_events_list(f: &mut Frame, app: &App, area: Rect) {
    let worldbook = &app.game_state.worldbook;
    let browser = &app.worldbook_browser;
    let is_list_focused = browser.is_list_focused();

    let block = Block::default()
        .title("📅 Events")
        .borders(Borders::ALL)
        .border_style(if is_list_focused {
            Style::default().fg(Color::Yellow)
        } else {
            Style::default().fg(Color::Green)
        })
        .border_type(BorderType::Rounded);

    let inner_area = block.inner(area);
    f.render_widget(block, area);

    if worldbook.events.is_empty() {
        let text = Paragraph::new("No events recorded yet.")
            .style(Style::default().fg(Color::DarkGray))
            .alignment(Alignment::Center);
        f.render_widget(text, inner_area);
        return;
    }

    let events = browser.get_sorted_events(worldbook);
    let items: Vec<ListItem> = events
        .iter()
        .enumerate()
        .map(|(i, event)| {
            let is_selected = i == browser.selected_index;

            // Enhanced highlighting when list is focused and item is selected
            let (style, selector) = if is_selected && is_list_focused {
                (
                    Style::default()
                        .fg(Color::Black)
                        .bg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                    "► ",
                )
            } else if is_selected {
                (
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                    "  ",
                )
            } else {
                (Style::default().fg(Color::White), "  ")
            };

            let icon = match event.event_type.as_str() {
                "npc_met" => "👥",
                "combat" => "⚔",
                "discovery" => "🔍",
                "dialogue" => "💬",
                _ => "📌",
            };

            let time = format_relative_time(&event.timestamp);
            let line = format!(
                "{}{} {:<15} {}",
                selector,
                icon,
                time,
                truncate_string(&event.description, 28)
            );

            ListItem::new(line).style(style)
        })
        .collect();

    let list = List::new(items);
    f.render_widget(list, inner_area);
}

/// Render event detail
fn render_event_detail(f: &mut Frame, app: &App, area: Rect) {
    let worldbook = &app.game_state.worldbook;
    let browser = &app.worldbook_browser;

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Green))
        .border_type(BorderType::Rounded);

    let inner_area = block.inner(area);
    f.render_widget(block, area);

    let events = browser.get_sorted_events(worldbook);
    if browser.selected_index >= events.len() {
        return;
    }

    let event = events[browser.selected_index];

    let mut lines = vec![
        Line::from(Span::styled(
            format!("Event: {}", event.event_type),
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(Span::styled(
            format!("Time: {}", format_relative_time(&event.timestamp)),
            Style::default().fg(Color::Yellow),
        )),
        Line::from(""),
    ];

    // Location
    if let Some(loc_id) = &event.location {
        if let Some(location) = worldbook.locations.get(loc_id) {
            lines.push(Line::from(vec![
                Span::styled("Location: ", Style::default().fg(Color::DarkGray)),
                Span::styled(location.name.as_str(), Style::default().fg(Color::White)),
            ]));
            lines.push(Line::from(""));
        }
    }

    // Description
    let wrapped_desc = wrap_text(&event.description, inner_area.width as usize - 2);
    for line in wrapped_desc {
        lines.push(Line::from(line));
    }
    lines.push(Line::from(""));

    // Entities involved
    if !event.entities.is_empty() {
        lines.push(Line::from(Span::styled(
            "═══ Involved ═══",
            Style::default().fg(Color::DarkGray),
        )));
        for entity_id in &event.entities {
            // Try to find as NPC first, then location
            if let Some(npc) = worldbook.npcs.get(entity_id) {
                lines.push(Line::from(format!(" 👤 {}", npc.name)));
            } else if let Some(location) = worldbook.locations.get(entity_id) {
                lines.push(Line::from(format!(" 📍 {}", location.name)));
            } else {
                lines.push(Line::from(format!(" • {}", entity_id)));
            }
        }
    }

    let paragraph = Paragraph::new(lines);
    f.render_widget(paragraph, inner_area);
}

/// Render search view
fn render_search_view(f: &mut Frame, _app: &App, area: Rect) {
    let block = Block::default()
        .title("🔍 Search Worldbook")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Green))
        .border_type(BorderType::Rounded);

    let inner_area = block.inner(area);
    f.render_widget(block, area);

    let text = Paragraph::new(
        "Search functionality coming soon!\n\nPress Tab to switch tabs or Q to close.",
    )
    .style(Style::default().fg(Color::DarkGray))
    .alignment(Alignment::Center);
    f.render_widget(text, inner_area);
}

/// Render help bar at bottom
fn render_worldbook_help(f: &mut Frame, area: Rect) {
    let help_area = Rect {
        x: area.x,
        y: area.y + area.height - 1,
        width: area.width,
        height: 1,
    };

    let help_text = "←→: Switch Tabs  ↑↓: Navigate/Focus  Enter: Expand  Q: Close";
    let paragraph = Paragraph::new(help_text)
        .style(Style::default().fg(Color::Black).bg(Color::DarkGray))
        .alignment(Alignment::Center);
    f.render_widget(paragraph, help_area);
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
