use ratatui::{
    style::{Color, Modifier, Style},
    text::{Line, Span},
};

/// Parse and format narrative text from the AI with enhanced typography
pub fn format_dm_narrative(content: &str, max_width: usize) -> Vec<Line<'static>> {
    let mut lines = Vec::new();

    // Ensure we have a minimum width to work with
    let max_width = max_width.max(20);

    // Add top border with "DUNGEON MASTER" header
    // Layout: "┌─ " (3) + "DUNGEON MASTER" (14) + " ─...─" (max_width-19) + "┐" (1) = max_width-1
    // We use max_width-1 to leave room for the border character
    lines.push(Line::from(vec![
        Span::styled("┌─ ", Style::default().fg(Color::Cyan)),
        Span::styled(
            "DUNGEON MASTER",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            format!(" {}", "─".repeat(max_width.saturating_sub(20))),
            Style::default().fg(Color::Cyan),
        ),
        Span::styled("─┐", Style::default().fg(Color::Cyan)),
    ]));

    // Parse the content and format it
    let parsed_sections = parse_narrative_content(content);

    for section in parsed_sections {
        match section {
            NarrativeSection::Text(text) => {
                // Add wrapped text with borders
                // Content width = max_width - 4 (for "│ " and " │" borders)
                let content_width = max_width.saturating_sub(4);
                let wrapped = wrap_text_advanced(&text, content_width);
                for wrapped_line in wrapped {
                    lines.push(build_bordered_line(
                        &wrapped_line,
                        max_width,
                        Style::default().fg(Color::White),
                    ));
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

                // Add dialogue text with speech bubble on first line
                let wrapped = wrap_text_advanced(&text, max_width.saturating_sub(10));
                for (i, wrapped_line) in wrapped.iter().enumerate() {
                    if i == 0 {
                        // First line: include speech bubble icon and speaker
                        let speaker_prefix = format!("💬 {}: ", speaker);
                        let prefix_len = speaker_prefix.chars().count() + 1; // +1 for opening quote
                        lines.push(Line::from(vec![
                            Span::styled("│ ", Style::default().fg(Color::Cyan)),
                            Span::styled("💬 ", Style::default()),
                            Span::styled(
                                format!("{}: ", speaker),
                                Style::default()
                                    .fg(Color::Green)
                                    .add_modifier(Modifier::ITALIC),
                            ),
                            Span::styled(
                                format!("\"{}\"", wrapped_line),
                                Style::default()
                                    .fg(Color::Cyan)
                                    .add_modifier(Modifier::ITALIC),
                            ),
                            Span::styled(
                                " ".repeat(
                                    max_width.saturating_sub(wrapped_line.len() + prefix_len + 3),
                                ),
                                Style::default(),
                            ),
                            Span::styled("│", Style::default().fg(Color::Cyan)),
                        ]));
                    } else {
                        // Subsequent lines: indent to align with first line's text
                        let indent_size = speaker.chars().count() + 5; // Align with text after "💬 Speaker: "
                        let indent = " ".repeat(indent_size);
                        lines.push(Line::from(vec![
                            Span::styled("│ ", Style::default().fg(Color::Cyan)),
                            Span::styled(indent.clone(), Style::default()),
                            Span::styled(
                                format!("\"{}\"", wrapped_line),
                                Style::default()
                                    .fg(Color::Cyan)
                                    .add_modifier(Modifier::ITALIC),
                            ),
                            Span::styled(
                                " ".repeat(
                                    max_width.saturating_sub(wrapped_line.len() + indent_size + 4),
                                ),
                                Style::default(),
                            ),
                            Span::styled("│", Style::default().fg(Color::Cyan)),
                        ]));
                    }
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
            let bullet_text = if let Some(text) = line.strip_prefix("• ") {
                text.trim().to_string()
            } else if let Some(text) = line.strip_prefix("- ") {
                text.trim().to_string()
            } else if let Some(text) = line.strip_prefix("* ") {
                text.trim().to_string()
            } else {
                // numbered list
                line.split_once('.')
                    .map_or(String::new(), |(_, after)| after.trim().to_string())
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

            // IMPORTANT: Only treat as dialogue if the "speaker" looks like an actual speaker name
            // - Should be relatively short (< 30 chars)
            // - Should not be a full sentence (no periods before the quote)
            // - Quoted text should be substantial (not just a short label like "V-13")
            if speaker.len() > 30 || before_quote.contains('.') || dialogue_text.len() < 5 {
                // This is likely a quoted term/label in narrative, not actual dialogue
                return None;
            }

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

            // Handle very long words that exceed max_width - split them
            if word_len > max_width {
                // Use char_indices for proper UTF-8 handling
                let chars: Vec<char> = word.chars().collect();
                let mut start = 0;
                while start < chars.len() {
                    let end = (start + max_width).min(chars.len());
                    let chunk: String = chars[start..end].iter().collect();
                    if start == 0 || end == chars.len() {
                        // First chunk or last chunk - might go on current/next line
                        if start == 0 {
                            lines.push(chunk);
                        } else {
                            current_line = chunk;
                            current_width = end - start;
                        }
                    } else {
                        lines.push(chunk);
                    }
                    start = end;
                }
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

/// Pad or truncate a line to a specific width
/// Ensures the returned string is EXACTLY the specified width
fn pad_line(text: &str, width: usize) -> String {
    if width == 0 {
        return String::new();
    }

    let chars: Vec<char> = text.chars().collect();
    let text_len = chars.len();

    if text_len >= width {
        // Truncate to exactly width characters
        chars.into_iter().take(width).collect()
    } else {
        // Pad with spaces to reach width
        format!("{}{}", text, " ".repeat(width - text_len))
    }
}

/// Build a bordered content line with proper width handling
/// Format: "│ " + content + " │" = max_width chars total
/// Content will be wrapped/truncated to fit exactly
fn build_bordered_line(content: &str, max_width: usize, style: Style) -> Line<'static> {
    // We need at least 4 chars for borders: "│ " (2) + " │" (2)
    let content_width = max_width.saturating_sub(4);
    let padded_content = pad_line(content, content_width);

    Line::from(vec![
        Span::styled("│ ", Style::default().fg(Color::Cyan)),
        Span::styled(padded_content, style),
        Span::styled(" │", Style::default().fg(Color::Cyan)),
    ])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wrap_text_advanced_basic() {
        let text = "Hello world this is a test";
        let wrapped = wrap_text_advanced(text, 15);

        assert_eq!(wrapped.len(), 2);
        assert_eq!(wrapped[0], "Hello world");
        assert_eq!(wrapped[1], "this is a test");
    }

    #[test]
    fn test_wrap_text_advanced_short_text() {
        let text = "Short";
        let wrapped = wrap_text_advanced(text, 20);

        assert_eq!(wrapped.len(), 1);
        assert_eq!(wrapped[0], "Short");
    }

    #[test]
    fn test_wrap_text_advanced_exact_width() {
        let text = "Exactly ten";
        let wrapped = wrap_text_advanced(text, 11);

        assert_eq!(wrapped.len(), 1);
        assert_eq!(wrapped[0], "Exactly ten");
    }

    #[test]
    fn test_wrap_text_advanced_long_word() {
        // Word longer than max_width should be split
        let text = "Supercalifragilisticexpialidocious";
        let wrapped = wrap_text_advanced(text, 10);

        assert!(!wrapped.is_empty());
        assert_eq!(wrapped[0], "Supercalif");
    }

    #[test]
    fn test_wrap_text_advanced_zero_width() {
        let text = "Some text";
        let wrapped = wrap_text_advanced(text, 0);

        assert_eq!(wrapped.len(), 1);
        assert_eq!(wrapped[0], "Some text");
    }

    #[test]
    fn test_wrap_text_advanced_single_word_per_line() {
        let text = "One Two Three Four";
        let wrapped = wrap_text_advanced(text, 5);

        assert_eq!(wrapped.len(), 4);
        assert_eq!(wrapped[0], "One");
        assert_eq!(wrapped[1], "Two");
        assert_eq!(wrapped[2], "Three");
        assert_eq!(wrapped[3], "Four");
    }

    #[test]
    fn test_pad_line_basic() {
        let padded = pad_line("Hello", 10);
        assert_eq!(padded, "Hello     ");
        assert_eq!(padded.chars().count(), 10);
    }

    #[test]
    fn test_pad_line_exact_width() {
        let padded = pad_line("Exactly", 7);
        assert_eq!(padded, "Exactly");
        assert_eq!(padded.chars().count(), 7);
    }

    #[test]
    fn test_pad_line_longer_than_width() {
        // Now truncates to fit exactly the specified width
        let padded = pad_line("TooLongText", 5);
        assert_eq!(padded, "TooLo");
        assert_eq!(padded.chars().count(), 5);
    }

    #[test]
    fn test_pad_line_empty() {
        let padded = pad_line("", 5);
        assert_eq!(padded, "     ");
        assert_eq!(padded.chars().count(), 5);
    }

    #[test]
    fn test_extract_dialogue_basic() {
        let line = "Guard: \"Halt! Who goes there?\"";
        let result = extract_dialogue(line);

        assert!(result.is_some());
        if let Some(NarrativeSection::Dialogue(speaker, text)) = result {
            assert_eq!(speaker, "Guard");
            assert_eq!(text, "Halt! Who goes there?");
        } else {
            panic!("Expected dialogue section");
        }
    }

    #[test]
    fn test_extract_dialogue_no_speaker() {
        let line = "\"Hello there!\"";
        let result = extract_dialogue(line);

        assert!(result.is_some());
        if let Some(NarrativeSection::Dialogue(speaker, text)) = result {
            assert_eq!(speaker, "Unknown");
            assert_eq!(text, "Hello there!");
        } else {
            panic!("Expected dialogue section");
        }
    }

    #[test]
    fn test_extract_dialogue_no_quotes() {
        let line = "Just regular text";
        let result = extract_dialogue(line);

        assert!(result.is_none());
    }

    #[test]
    fn test_extract_dialogue_incomplete_quotes() {
        let line = "Speaker: \"Missing closing quote";
        let result = extract_dialogue(line);

        assert!(result.is_none());
    }

    #[test]
    fn test_extract_speaker_text_basic() {
        let line = "Captain: The enemy approaches from the east";
        let result = extract_speaker_text(line);

        assert!(result.is_some());
        if let Some((speaker, text)) = result {
            assert_eq!(speaker, "Captain");
            assert_eq!(text, "The enemy approaches from the east");
        } else {
            panic!("Expected speaker/text pair");
        }
    }

    #[test]
    fn test_extract_speaker_text_long_speaker_name() {
        // Very long speaker name should be rejected
        let line = "This is a very long name that should not be considered a speaker: text";
        let result = extract_speaker_text(line);

        assert!(result.is_none());
    }

    #[test]
    fn test_extract_speaker_text_lowercase_start() {
        // Speaker name should start with uppercase
        let line = "lowercase: should not match";
        let result = extract_speaker_text(line);

        assert!(result.is_none());
    }

    #[test]
    fn test_extract_speaker_text_no_colon() {
        let line = "No colon here";
        let result = extract_speaker_text(line);

        assert!(result.is_none());
    }

    #[test]
    fn test_is_mechanic_text_roll() {
        assert!(is_mechanic_text("Make a Perception roll"));
        assert!(is_mechanic_text("You may roll for initiative"));
        assert!(is_mechanic_text("ROLL a d20"));
    }

    #[test]
    fn test_is_mechanic_text_check() {
        assert!(is_mechanic_text("Make a Stealth check"));
        assert!(is_mechanic_text("CHECK your inventory"));
    }

    #[test]
    fn test_is_mechanic_text_dc() {
        assert!(is_mechanic_text("DC 15 Lockpick"));
        assert!(is_mechanic_text("The DC is 12"));
    }

    #[test]
    fn test_is_mechanic_text_dice() {
        assert!(is_mechanic_text("Roll a d20"));
        assert!(is_mechanic_text("Take 2d6 damage"));
        assert!(is_mechanic_text("1d10+3"));
    }

    #[test]
    fn test_is_mechanic_text_bracket() {
        assert!(is_mechanic_text("[SKILL CHECK]"));
        assert!(is_mechanic_text("[Combat begins]"));
    }

    #[test]
    fn test_is_mechanic_text_regular_text() {
        assert!(!is_mechanic_text("The guard looks at you"));
        assert!(!is_mechanic_text("You see a door"));
        assert!(!is_mechanic_text("Normal dialogue here"));
    }

    #[test]
    fn test_parse_narrative_content_simple_text() {
        let content = "Simple text line";
        let sections = parse_narrative_content(content);

        assert_eq!(sections.len(), 1);
        match &sections[0] {
            NarrativeSection::Text(text) => assert_eq!(text, "Simple text line"),
            _ => panic!("Expected text section"),
        }
    }

    #[test]
    fn test_parse_narrative_content_bullet_list() {
        let content = "• First item\n• Second item\n• Third item";
        let sections = parse_narrative_content(content);

        assert_eq!(sections.len(), 1);
        match &sections[0] {
            NarrativeSection::BulletList(items) => {
                assert_eq!(items.len(), 3);
                assert_eq!(items[0], "First item");
                assert_eq!(items[1], "Second item");
                assert_eq!(items[2], "Third item");
            }
            _ => panic!("Expected bullet list section"),
        }
    }

    #[test]
    fn test_parse_narrative_content_numbered_list() {
        let content = "1. First\n2. Second\n3. Third";
        let sections = parse_narrative_content(content);

        assert_eq!(sections.len(), 1);
        match &sections[0] {
            NarrativeSection::BulletList(items) => {
                assert_eq!(items.len(), 3);
                assert_eq!(items[0], "First");
                assert_eq!(items[1], "Second");
                assert_eq!(items[2], "Third");
            }
            _ => panic!("Expected bullet list section"),
        }
    }

    #[test]
    fn test_parse_narrative_content_dialogue() {
        let content = "Guard: \"Who goes there?\"";
        let sections = parse_narrative_content(content);

        assert_eq!(sections.len(), 1);
        match &sections[0] {
            NarrativeSection::Dialogue(speaker, text) => {
                assert_eq!(speaker, "Guard");
                assert_eq!(text, "Who goes there?");
            }
            _ => panic!("Expected dialogue section"),
        }
    }

    #[test]
    fn test_parse_narrative_content_mechanic() {
        let content = "Make a DC 15 Lockpick check";
        let sections = parse_narrative_content(content);

        assert_eq!(sections.len(), 1);
        match &sections[0] {
            NarrativeSection::Mechanic(text) => {
                assert_eq!(text, "Make a DC 15 Lockpick check");
            }
            _ => panic!("Expected mechanic section"),
        }
    }

    #[test]
    fn test_parse_narrative_content_empty_lines() {
        let content = "Line 1\n\nLine 2\n\nLine 3";
        let sections = parse_narrative_content(content);

        assert_eq!(sections.len(), 5);
        match &sections[0] {
            NarrativeSection::Text(_) => {}
            _ => panic!("Expected text"),
        }
        match &sections[1] {
            NarrativeSection::EmptyLine => {}
            _ => panic!("Expected empty line"),
        }
    }

    #[test]
    fn test_parse_narrative_content_mixed_content() {
        let content = "Text line\n• Bullet 1\n• Bullet 2\nGuard: \"Hello\"\nMake a roll";
        let sections = parse_narrative_content(content);

        assert!(sections.len() >= 4);
        // First should be text
        match &sections[0] {
            NarrativeSection::Text(_) => {}
            _ => panic!("Expected text section first"),
        }
    }

    #[test]
    fn test_parse_narrative_content_dash_bullets() {
        let content = "- Item one\n- Item two";
        let sections = parse_narrative_content(content);

        assert_eq!(sections.len(), 1);
        match &sections[0] {
            NarrativeSection::BulletList(items) => {
                assert_eq!(items.len(), 2);
                assert_eq!(items[0], "Item one");
            }
            _ => panic!("Expected bullet list"),
        }
    }

    #[test]
    fn test_parse_narrative_content_asterisk_bullets() {
        let content = "* First\n* Second";
        let sections = parse_narrative_content(content);

        assert_eq!(sections.len(), 1);
        match &sections[0] {
            NarrativeSection::BulletList(items) => {
                assert_eq!(items.len(), 2);
            }
            _ => panic!("Expected bullet list"),
        }
    }

    #[test]
    fn test_format_dm_narrative_basic() {
        let content = "Hello, adventurer!";
        let lines = format_dm_narrative(content, 40);

        // Should have header, content line, and footer
        assert!(lines.len() >= 3);
        // First line should be header with "DUNGEON MASTER"
        assert!(lines[0]
            .spans
            .iter()
            .any(|span| span.content.contains("DUNGEON MASTER")));
    }

    #[test]
    fn test_format_dm_narrative_with_dialogue() {
        let content = "Guard: \"Stop right there!\"";
        let lines = format_dm_narrative(content, 60);

        assert!(lines.len() > 2);
        // Should contain the dialogue formatted properly
        assert!(lines.iter().any(|line| {
            line.spans
                .iter()
                .any(|span| span.content.contains("Guard") || span.content.contains("Stop"))
        }));
    }

    #[test]
    fn test_format_dm_narrative_empty() {
        let content = "";
        let lines = format_dm_narrative(content, 40);

        // Should still have header and footer
        assert!(lines.len() >= 2);
    }

    #[test]
    fn test_wrap_text_advanced_multiline() {
        let text = "This is a much longer text that should wrap across multiple lines when given a reasonable width constraint";
        let wrapped = wrap_text_advanced(text, 20);

        assert!(wrapped.len() > 3);
        // Each line should not exceed max width
        for line in &wrapped {
            assert!(line.chars().count() <= 20);
        }
    }

    #[test]
    fn test_extract_dialogue_with_punctuation() {
        let line = "Merchant: \"That'll cost you 50 caps, friend.\"";
        let result = extract_dialogue(line);

        assert!(result.is_some());
        if let Some(NarrativeSection::Dialogue(speaker, text)) = result {
            assert_eq!(speaker, "Merchant");
            assert_eq!(text, "That'll cost you 50 caps, friend.");
        }
    }

    #[test]
    fn test_is_mechanic_text_you_may() {
        assert!(is_mechanic_text("You may roll for Perception"));
        assert!(is_mechanic_text("You may check your Lockpick skill"));
    }

    #[test]
    fn test_parse_narrative_content_consecutive_bullets() {
        let content = "Introduction\n• Item 1\n• Item 2\nConclusion";
        let sections = parse_narrative_content(content);

        assert_eq!(sections.len(), 3);
        match &sections[0] {
            NarrativeSection::Text(t) => assert_eq!(t, "Introduction"),
            _ => panic!("Expected text"),
        }
        match &sections[1] {
            NarrativeSection::BulletList(_) => {}
            _ => panic!("Expected bullet list"),
        }
        match &sections[2] {
            NarrativeSection::Text(t) => assert_eq!(t, "Conclusion"),
            _ => panic!("Expected text"),
        }
    }

    #[test]
    fn test_format_dm_narrative_long_paragraph_wraps() {
        // Test case similar to the actual game output - a long narrative description
        let content = "The vault entrance is dimly lit by flickering fluorescent tubes, the rusted metal walls bearing faded slogans. A low hum from the old air filters rattles against your chest. The door itself is a sealed steel slab, its brass bolts corroded green with age.";

        // Use a typical terminal width of 80 chars
        let max_width = 80;
        let lines = format_dm_narrative(content, max_width);

        // Should have header, multiple content lines, and footer
        assert!(
            lines.len() >= 4,
            "Expected at least 4 lines (header, 2+ content, footer)"
        );

        // Verify each line is within max_width by checking span widths
        for (i, line) in lines.iter().enumerate() {
            let total_width: usize = line.spans.iter().map(|s| s.content.chars().count()).sum();
            assert!(
                total_width <= max_width,
                "Line {} exceeds max_width: {} chars (max {})",
                i,
                total_width,
                max_width
            );
        }
    }

    #[test]
    fn test_build_bordered_line_exact_width() {
        let max_width = 50;
        let line = build_bordered_line("Test content", max_width, Style::default());

        let total_width: usize = line.spans.iter().map(|s| s.content.chars().count()).sum();
        assert_eq!(total_width, max_width, "Line should be exactly max_width");
    }

    #[test]
    fn test_build_bordered_line_truncates_long_content() {
        let max_width = 20;
        let long_content = "This is a very long content that should be truncated";
        let line = build_bordered_line(long_content, max_width, Style::default());

        let total_width: usize = line.spans.iter().map(|s| s.content.chars().count()).sum();
        assert_eq!(
            total_width, max_width,
            "Line should be exactly max_width even with long content"
        );
    }
}
