use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Paragraph},
    Frame,
};

use super::app::App;
use super::settings_editor::SettingsEditor;

/// Render the settings view
pub fn render_settings(f: &mut Frame, app: &App, area: Rect) {
    let editor = &app.settings_editor;

    // Main settings border
    let block = Block::default()
        .title(vec![
            Span::raw("⚙ "),
            Span::styled(
                " LLAMA.CPP SERVER SETTINGS ",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(" ⚙"),
        ])
        .title_alignment(Alignment::Center)
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan))
        .border_type(BorderType::Double);

    let inner = block.inner(area);
    f.render_widget(block, area);

    // Split into sections: Settings list | Help text
    let sections = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(0),     // Settings list
            Constraint::Length(5),  // Help text
        ])
        .split(inner);

    // Render settings list
    render_settings_list(f, editor, sections[0]);

    // Render help/status bar
    render_settings_help(f, editor, sections[1]);
}

/// Render the list of settings
fn render_settings_list(f: &mut Frame, editor: &SettingsEditor, area: Rect) {
    let fields = SettingsEditor::get_fields();
    let mut lines = Vec::with_capacity(fields.len() * 4 + 5); // Pre-allocate for all fields

    lines.push(Line::from(""));
    lines.push(Line::from(vec![
        Span::styled(
            "  Use ↑↓ to navigate, Enter to edit, 's' to save to config.toml",
            Style::default().fg(Color::DarkGray).add_modifier(Modifier::ITALIC),
        ),
    ]));
    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        "  ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━",
        Style::default().fg(Color::DarkGray),
    )));
    lines.push(Line::from(""));

    for (idx, field) in fields.iter().enumerate() {
        let is_selected = idx == editor.selected_index;
        let is_editing = is_selected && editor.editing;

        let name = SettingsEditor::field_name(*field);
        let description = SettingsEditor::field_description(*field);
        let value = if is_editing {
            editor.edit_buffer.clone()
        } else {
            editor.get_value(*field)
        };

        // Selected indicator
        let indicator = if is_selected { "►" } else { " " };

        // Field name line
        let name_style = if is_selected {
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::Cyan)
        };

        lines.push(Line::from(vec![
            Span::styled(format!("  {} ", indicator), Style::default().fg(Color::Yellow)),
            Span::styled(name, name_style),
        ]));

        // Description line
        lines.push(Line::from(vec![
            Span::raw("     "),
            Span::styled(description, Style::default().fg(Color::DarkGray).add_modifier(Modifier::ITALIC)),
        ]));

        // Value line
        let value_style = if is_editing {
            Style::default()
                .fg(Color::Green)
                .add_modifier(Modifier::BOLD | Modifier::UNDERLINED)
        } else if is_selected {
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::White)
        };

        let value_prefix = if is_editing { "     [EDITING] " } else { "     Value: " };
        let cursor_indicator = if is_editing { "▊" } else { "" };

        lines.push(Line::from(vec![
            Span::styled(value_prefix, Style::default().fg(Color::DarkGray)),
            Span::styled(value, value_style),
            Span::styled(cursor_indicator, Style::default().fg(Color::Green)),
        ]));

        lines.push(Line::from(""));
    }

    let paragraph = Paragraph::new(lines);
    f.render_widget(paragraph, area);
}

/// Render help/status text
fn render_settings_help(f: &mut Frame, editor: &SettingsEditor, area: Rect) {
    let mut lines = vec![
        Line::from(Span::styled(
            "  ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━",
            Style::default().fg(Color::DarkGray),
        )),
        Line::from(""),
    ];

    if let Some(ref status) = editor.status_message {
        lines.push(Line::from(vec![
            Span::styled("  Status: ", Style::default().fg(Color::Cyan)),
            Span::styled(status, Style::default().fg(Color::Yellow)),
        ]));
    } else if editor.editing {
        lines.push(Line::from(vec![
            Span::styled("  ", Style::default()),
            Span::styled(
                "Editing mode: Type your changes, Enter to save, Esc to cancel",
                Style::default().fg(Color::Green),
            ),
        ]));
    } else {
        lines.push(Line::from(vec![
            Span::styled("  ", Style::default()),
            Span::styled(
                "↑↓: Navigate  │  Enter: Edit  │  's': Save  │  Esc: Exit",
                Style::default().fg(Color::DarkGray),
            ),
        ]));
    }

    lines.push(Line::from(""));

    let paragraph = Paragraph::new(lines);
    f.render_widget(paragraph, area);
}
