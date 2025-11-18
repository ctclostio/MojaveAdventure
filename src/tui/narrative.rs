use ratatui::{
    style::{Color, Modifier, Style},
    text::{Line, Span},
};

/// Parse and format narrative text from the AI with enhanced typography
pub fn format_dm_narrative(content: &str, max_width: usize) -> Vec<Line<'static>> {
    let mut lines = Vec::new();

    // Add top border with "DUNGEON MASTER" header
    lines.push(Line::from(vec![
        Span::styled("┌─ ", Style::default().fg(Color::Cyan)),
        Span::styled(
            "DUNGEON MASTER",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            format!(" {}", "─".repeat(max_width.saturating_sub(18))),
            Style::default().fg(Color::Cyan),
        ),
        Span::styled("┐", Style::default().fg(Color::Cyan)),
    ]));

    // Parse the content and format it
    let parsed_sections = parse_narrative_content(content);

    for section in parsed_sections {
        match section {
            NarrativeSection::Text(text) => {
                // Add wrapped text with borders
                let wrapped = wrap_text_advanced(&text, max_width.saturating_sub(4));
                for wrapped_line in wrapped {
                    lines.push(Line::from(vec![
                        Span::styled("│ ", Style::default().fg(Color::Cyan)),
                        Span::styled(
                            pad_line(&wrapped_line, max_width.saturating_sub(4)),
                            Style::default().fg(Color::White),
                        ),
                        Span::styled(" │", Style::default().fg(Color::Cyan)),
                    ]));
                }
            }
            NarrativeSection::BulletList(items) => {
                for item in items {
                    let wrapped = wrap_text_advanced(&item, max_width.saturating_sub(7));
                    for (i, wrapped_line) in wrapped.iter().enumerate() {
                        let prefix = if i == 0 { " • " } else { "   " };
                        lines.push(Line::from(vec![
                            Span::styled("│", Style::default().fg(Color::Cyan)),
                            Span::styled(prefix, Style::default().fg(Color::Yellow)),
                            Span::styled(
                                pad_line(wrapped_line, max_width.saturating_sub(7)),
                                Style::default().fg(Color::White),
                            ),
                            Span::styled(" │", Style::default().fg(Color::Cyan)),
                        ]));
                    }
                }
            }
            NarrativeSection::Dialogue(speaker, text) => {
                // Add empty line before dialogue for spacing
                lines.push(Line::from(vec![
                    Span::styled("│", Style::default().fg(Color::Cyan)),
                    Span::styled(" ".repeat(max_width.saturating_sub(2)), Style::default()),
                    Span::styled("│", Style::default().fg(Color::Cyan)),
                ]));

                // Add dialogue marker
                lines.push(Line::from(vec![
                    Span::styled("│ ", Style::default().fg(Color::Cyan)),
                    Span::styled("💬 ", Style::default()),
                    Span::styled(
                        format!("{}:", speaker),
                        Style::default()
                            .fg(Color::Green)
                            .add_modifier(Modifier::ITALIC),
                    ),
                    Span::styled(
                        " ".repeat(max_width.saturating_sub(speaker.len() + 7)),
                        Style::default(),
                    ),
                    Span::styled("│", Style::default().fg(Color::Cyan)),
                ]));

                // Add dialogue text
                let wrapped = wrap_text_advanced(&text, max_width.saturating_sub(8));
                for wrapped_line in wrapped {
                    lines.push(Line::from(vec![
                        Span::styled("│    ", Style::default().fg(Color::Cyan)),
                        Span::styled(
                            format!("\"{}\"", wrapped_line),
                            Style::default()
                                .fg(Color::Cyan)
                                .add_modifier(Modifier::ITALIC),
                        ),
                        Span::styled(
                            " ".repeat(max_width.saturating_sub(wrapped_line.len() + 8)),
                            Style::default(),
                        ),
                        Span::styled("│", Style::default().fg(Color::Cyan)),
                    ]));
                }
            }
            NarrativeSection::Mechanic(text) => {
                // Add empty line before mechanic for spacing
                lines.push(Line::from(vec![
                    Span::styled("│", Style::default().fg(Color::Cyan)),
                    Span::styled(" ".repeat(max_width.saturating_sub(2)), Style::default()),
                    Span::styled("│", Style::default().fg(Color::Cyan)),
                ]));

                // Add mechanic marker
                let wrapped = wrap_text_advanced(&text, max_width.saturating_sub(7));
                for (i, wrapped_line) in wrapped.iter().enumerate() {
                    let prefix = if i == 0 { "🎲 " } else { "   " };
                    lines.push(Line::from(vec![
                        Span::styled("│ ", Style::default().fg(Color::Cyan)),
                        Span::styled(prefix, Style::default()),
                        Span::styled(
                            pad_line(wrapped_line, max_width.saturating_sub(7)),
                            Style::default().fg(Color::Magenta),
                        ),
                        Span::styled(" │", Style::default().fg(Color::Cyan)),
                    ]));
                }
            }
            NarrativeSection::EmptyLine => {
                lines.push(Line::from(vec![
                    Span::styled("│", Style::default().fg(Color::Cyan)),
                    Span::styled(" ".repeat(max_width.saturating_sub(2)), Style::default()),
                    Span::styled("│", Style::default().fg(Color::Cyan)),
                ]));
            }
        }
    }

    // Add bottom border
    lines.push(Line::from(vec![
        Span::styled("└", Style::default().fg(Color::Cyan)),
        Span::styled(
            "─".repeat(max_width.saturating_sub(2)),
            Style::default().fg(Color::Cyan),
        ),
        Span::styled("┘", Style::default().fg(Color::Cyan)),
    ]));

    lines
}

#[derive(Debug)]
enum NarrativeSection {
    Text(String),
    BulletList(Vec<String>),
    Dialogue(String, String), // (speaker, text)
    Mechanic(String),
    EmptyLine,
}

/// Parse narrative content into structured sections
fn parse_narrative_content(content: &str) -> Vec<NarrativeSection> {
    let mut sections = Vec::new();
    let mut current_bullets = Vec::new();
    let lines: Vec<&str> = content.lines().collect();

    let mut i = 0;
    while i < lines.len() {
        let line = lines[i].trim();

        // Empty line
        if line.is_empty() {
            // Flush any pending bullets
            if !current_bullets.is_empty() {
                sections.push(NarrativeSection::BulletList(current_bullets.clone()));
                current_bullets.clear();
            }
            sections.push(NarrativeSection::EmptyLine);
            i += 1;
            continue;
        }

        // Bullet point (•, -, *, or numbered)
        if line.starts_with("• ")
            || line.starts_with("- ")
            || line.starts_with("* ")
            || (line.len() > 2
                && line.chars().next().unwrap().is_numeric()
                && line.chars().nth(1) == Some('.'))
        {
            let bullet_text =
                if line.starts_with("• ") || line.starts_with("- ") || line.starts_with("* ") {
                    line[2..].trim().to_string()
                } else {
                    // numbered list
                    line.splitn(2, '.').nth(1).unwrap_or("").trim().to_string()
                };
            current_bullets.push(bullet_text);
            i += 1;
            continue;
        }

        // Flush any pending bullets before processing other content
        if !current_bullets.is_empty() {
            sections.push(NarrativeSection::BulletList(current_bullets.clone()));
            current_bullets.clear();
        }

        // Dialogue detection (quotes or "Speaker:" pattern)
        if line.contains('"') {
            // Extract dialogue
            if let Some(dialogue) = extract_dialogue(line) {
                sections.push(dialogue);
                i += 1;
                continue;
            }
        }

        // Check for speaker pattern like "Name: text"
        if let Some((speaker, text)) = extract_speaker_text(line) {
            sections.push(NarrativeSection::Dialogue(speaker, text));
            i += 1;
            continue;
        }

        // Mechanic/roll detection (contains "roll", "check", "DC", etc.)
        if is_mechanic_text(line) {
            sections.push(NarrativeSection::Mechanic(line.to_string()));
            i += 1;
            continue;
        }

        // Regular text
        sections.push(NarrativeSection::Text(line.to_string()));
        i += 1;
    }

    // Flush any remaining bullets
    if !current_bullets.is_empty() {
        sections.push(NarrativeSection::BulletList(current_bullets));
    }

    sections
}

/// Extract dialogue from a line containing quotes
fn extract_dialogue(line: &str) -> Option<NarrativeSection> {
    // Try to find speaker before quotes
    if let Some(quote_start) = line.find('"') {
        let before_quote = line[..quote_start].trim();
        let after_quote_start = &line[quote_start + 1..];

        if let Some(quote_end) = after_quote_start.find('"') {
            let dialogue_text = after_quote_start[..quote_end].to_string();

            // Extract speaker name (everything before the quote, remove trailing :)
            let speaker = if !before_quote.is_empty() {
                before_quote.trim_end_matches(':').trim().to_string()
            } else {
                "Unknown".to_string()
            };

            return Some(NarrativeSection::Dialogue(speaker, dialogue_text));
        }
    }
    None
}

/// Extract speaker and text from "Speaker: text" pattern
fn extract_speaker_text(line: &str) -> Option<(String, String)> {
    if let Some(colon_pos) = line.find(':') {
        let potential_speaker = line[..colon_pos].trim();
        // Check if it looks like a speaker name (short, starts with capital, no spaces at ends)
        if potential_speaker.len() < 30
            && potential_speaker.chars().next()?.is_uppercase()
            && !potential_speaker.contains('"')
        {
            let text = line[colon_pos + 1..].trim().to_string();
            return Some((potential_speaker.to_string(), text));
        }
    }
    None
}

/// Check if text is a game mechanic instruction
fn is_mechanic_text(line: &str) -> bool {
    let lower = line.to_lowercase();
    lower.contains("roll")
        || lower.contains("check")
        || lower.contains("dc ")
        || lower.contains("d20")
        || lower.contains("d6")
        || lower.contains("d10")
        || (lower.contains("you may") && (lower.contains("roll") || lower.contains("check")))
        || lower.starts_with("[")
}

/// Advanced text wrapping that preserves words and handles special characters
fn wrap_text_advanced(text: &str, max_width: usize) -> Vec<String> {
    if max_width == 0 {
        return vec![text.to_string()];
    }

    let mut lines = Vec::new();
    let mut current_line = String::new();
    let mut current_width = 0;

    for word in text.split_whitespace() {
        let word_len = word.chars().count();

        // Check if adding this word would exceed the width
        if current_width + word_len + (if current_width > 0 { 1 } else { 0 }) > max_width {
            if !current_line.is_empty() {
                lines.push(current_line.clone());
                current_line.clear();
                current_width = 0;
            }

            // Handle very long words that exceed max_width
            if word_len > max_width {
                lines.push(word[..max_width].to_string());
                current_line = word[max_width..].to_string();
                current_width = current_line.chars().count();
                continue;
            }
        }

        if current_width > 0 {
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

/// Pad a line to a specific width
fn pad_line(text: &str, width: usize) -> String {
    let text_len = text.chars().count();
    if text_len >= width {
        text.to_string()
    } else {
        format!("{}{}", text, " ".repeat(width - text_len))
    }
}
