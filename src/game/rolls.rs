use super::character::Character;
use rand::Rng;

/// Represents a skill check result
#[derive(Debug, Clone)]
pub struct RollResult {
    pub skill_name: String,
    pub roll: i32,
    pub modifier: i32,
    pub total: i32,
    pub dc: i32,
    pub success: bool,
    pub critical: bool,
    pub fumble: bool,
}

impl RollResult {
    /// Format the roll result for display
    pub fn format(&self) -> String {
        let outcome = if self.critical {
            "CRITICAL SUCCESS!"
        } else if self.fumble {
            "CRITICAL FAILURE!"
        } else if self.success {
            "Success"
        } else {
            "Failure"
        };

        format!(
            "{} Check: Rolled {}+{} = {} vs DC {} - {}",
            self.skill_name, self.roll, self.modifier, self.total, self.dc, outcome
        )
    }

    /// Get a color-coded emoji for the result
    pub fn emoji(&self) -> &str {
        if self.critical {
            "⭐"
        } else if self.fumble {
            "💀"
        } else if self.success {
            "✓"
        } else {
            "✗"
        }
    }
}

/// Parse skill/stat from AI response
/// Expected format: "SKILL: lockpick DC 15" or "STAT: perception DC 10"
/// This parser is designed to be forgiving of minor format variations
pub fn parse_roll_request(text: &str) -> Option<(String, i32)> {
    let lower = text.to_lowercase();

    // Primary pattern: Look for "SKILL: name DC number" or "STAT: name DC number"
    if let Some(skill_start) = lower.find("skill:").or_else(|| lower.find("stat:")) {
        let after_skill = &text[skill_start + 6..]; // Skip "skill:" or "stat:"

        // Extract skill name (everything before "DC")
        if let Some(dc_pos) = after_skill.to_lowercase().find("dc") {
            let skill_name = after_skill[..dc_pos].trim().to_string();

            // Skip empty skill names
            if skill_name.is_empty() {
                return None;
            }

            // Extract DC number - look for the first number after "DC"
            let after_dc = &after_skill[dc_pos + 2..];
            let dc_part = after_dc.trim();

            // Try multiple DC extraction strategies
            // Strategy 1: First whitespace-separated token
            if let Some(dc_str) = dc_part.split_whitespace().next() {
                if let Ok(dc) = dc_str.parse::<i32>() {
                    return Some((skill_name, dc));
                }
            }

            // Strategy 2: Extract first sequence of digits
            let digits: String = dc_part
                .chars()
                .take_while(|c| c.is_ascii_digit() || c.is_whitespace())
                .filter(|c| c.is_ascii_digit())
                .collect();

            if !digits.is_empty() {
                if let Ok(dc) = digits.parse::<i32>() {
                    return Some((skill_name, dc));
                }
            }
        }
    }

    None
}

/// Parse natural language skill check requests from DM responses
/// Detects patterns like "make a Science check", "roll Lockpick", etc.
pub fn parse_natural_roll_request(text: &str) -> Option<(String, i32)> {
    let lower = text.to_lowercase();

    // List of all possible skills and stats to detect
    let skills = [
        "small guns",
        "big guns",
        "energy weapons",
        "melee weapons",
        "unarmed",
        "speech",
        "sneak",
        "lockpick",
        "science",
        "repair",
        "barter",
        "explosives",
        "medicine",
        "survival",
        "throwing",
        "first aid",
        "doctor",
        "outdoorsman",
    ];

    let stats = [
        "strength",
        "perception",
        "endurance",
        "charisma",
        "intelligence",
        "agility",
        "luck",
    ];

    // Common trigger phrases that indicate a skill check request
    let check_phrases = [
        "roll",
        "check",
        "make a",
        "requires a",
        "need a",
        "needs a",
        "attempt a",
        "try a",
        "successful",
        "roll under your",
        "requires an",
        "needs an",
        "make an",
        "attempt an",
    ];

    // Check if the text contains any check trigger phrases
    let has_check_phrase = check_phrases.iter().any(|phrase| lower.contains(phrase));

    if !has_check_phrase {
        return None;
    }

    // Try to find a skill or stat name in the text
    let mut found_skill = None;

    // Check for skills first (they're more specific)
    for skill in &skills {
        if lower.contains(skill) {
            found_skill = Some(skill.to_string());
            break;
        }
    }

    // If no skill found, check for stats
    if found_skill.is_none() {
        for stat in &stats {
            if lower.contains(stat) {
                found_skill = Some(stat.to_string());
                break;
            }
        }
    }

    // If we found a skill/stat, try to extract the DC
    if let Some(skill_name) = found_skill {
        // Strategy 1: Look for "DC" followed by a number (most common)
        if let Some(dc_pos) = lower.find("dc") {
            let after_dc = &text[dc_pos + 2..];

            // Skip any whitespace, colons, or equal signs
            let trimmed =
                after_dc.trim_start_matches(|c: char| c.is_whitespace() || c == ':' || c == '=');

            // Extract consecutive digits
            let digits: String = trimmed.chars().take_while(|c| c.is_ascii_digit()).collect();

            if !digits.is_empty() {
                if let Ok(dc) = digits.parse::<i32>() {
                    return Some((skill_name, dc));
                }
            }
        }

        // Strategy 2: Look for "(DC number)" or "[DC number]" patterns
        for pattern in &["(dc ", "[dc ", "(DC ", "[DC "] {
            if let Some(paren_dc) = text.find(pattern) {
                let after_dc = &text[paren_dc + pattern.len()..];
                if let Some(dc_str) = after_dc.split_whitespace().next() {
                    // Remove trailing ) or ]
                    let dc_clean = dc_str.trim_end_matches(|c| c == ')' || c == ']');
                    if let Ok(dc) = dc_clean.parse::<i32>() {
                        return Some((skill_name, dc));
                    }
                }
            }
        }

        // Strategy 3: Look for "difficulty X" or "difficulty of X" patterns
        if let Some(diff_pos) = lower.find("difficulty") {
            let after_diff = &text[diff_pos + 10..];
            let trimmed = after_diff.trim_start_matches(|c: char| {
                c.is_whitespace() || c == ':' || c == 'o' || c == 'f'
            });
            let digits: String = trimmed.chars().take_while(|c| c.is_ascii_digit()).collect();

            if !digits.is_empty() {
                if let Ok(dc) = digits.parse::<i32>() {
                    return Some((skill_name, dc));
                }
            }
        }

        // Strategy 4: Look for "against DC" pattern specifically
        if let Some(against_pos) = lower.find("against dc") {
            let after_against = &text[against_pos + 10..];
            let trimmed = after_against.trim_start();
            let digits: String = trimmed.chars().take_while(|c| c.is_ascii_digit()).collect();

            if !digits.is_empty() {
                if let Ok(dc) = digits.parse::<i32>() {
                    return Some((skill_name, dc));
                }
            }
        }
    }

    None
}

/// Truncate a DM response at the skill check statement
/// Returns the response trimmed to end right after the DC statement
/// This prevents AI commentary after skill checks from entering conversation history
pub fn truncate_response_at_skill_check(text: &str) -> Option<String> {
    // First, check if there's a skill check in this text
    if let Some((_skill, dc)) = parse_natural_roll_request(text) {
        // Look for DC patterns in the text
        let dc_patterns = [
            format!("(dc {})", dc),
            format!("(DC {})", dc),
            format!("[dc {}]", dc),
            format!("[DC {}]", dc),
            format!("dc {}", dc),
            format!("DC {}", dc),
        ];

        // Find the first matching DC pattern
        for pattern in &dc_patterns {
            if let Some(pos) = text.find(pattern) {
                let end_of_dc = pos + pattern.len();

                // Look for sentence ending within next 15 characters
                let search_range = &text[end_of_dc..].chars().take(15).collect::<String>();

                if let Some(offset) = search_range.find(|c| c == '.' || c == '!' || c == '?') {
                    // Found sentence ending nearby, include it
                    let final_pos = end_of_dc + offset + 1;
                    return Some(text[..final_pos].trim().to_string());
                } else {
                    // No sentence ending found nearby, add a period
                    return Some(format!("{}.", text[..end_of_dc].trim()));
                }
            }
        }
    }

    None
}

/// Perform a skill or stat check
pub fn perform_roll(character: &Character, skill_or_stat: &str, dc: i32) -> RollResult {
    let mut rng = rand::rng();
    let roll = rng.random_range(1..=20);

    // Determine modifier based on skill or stat name
    let (skill_name, modifier) = get_modifier(character, skill_or_stat);

    let total = roll + modifier;
    let success = total >= dc || roll == 20;
    let critical = roll == 20;
    let fumble = roll == 1;

    RollResult {
        skill_name,
        roll,
        modifier,
        total,
        dc,
        success,
        critical,
        fumble,
    }
}

/// Get the appropriate modifier for a skill or stat
fn get_modifier(character: &Character, name: &str) -> (String, i32) {
    let lower = name.to_lowercase();

    // Check skills first
    if lower.contains("small") || lower.contains("gun") || lower.contains("firearms") {
        return ("Small Guns".to_string(), character.skills.small_guns as i32);
    }
    if lower.contains("big") || lower.contains("heavy") {
        return ("Big Guns".to_string(), character.skills.big_guns as i32);
    }
    if lower.contains("energy") {
        return (
            "Energy Weapons".to_string(),
            character.skills.energy_weapons as i32,
        );
    }
    if lower.contains("melee") {
        return (
            "Melee Weapons".to_string(),
            character.skills.melee_weapons as i32,
        );
    }
    if lower.contains("unarmed") || lower.contains("fist") {
        return ("Unarmed".to_string(), character.skills.unarmed as i32);
    }
    if lower.contains("speech") || lower.contains("persuade") || lower.contains("charisma check") {
        return ("Speech".to_string(), character.skills.speech as i32);
    }
    if lower.contains("sneak") || lower.contains("stealth") {
        return ("Sneak".to_string(), character.skills.sneak as i32);
    }
    if lower.contains("lockpick") || lower.contains("lock") {
        return ("Lockpick".to_string(), character.skills.lockpick as i32);
    }
    if lower.contains("science") || lower.contains("hack") || lower.contains("computer") {
        return ("Science".to_string(), character.skills.science as i32);
    }
    if lower.contains("repair") || lower.contains("fix") {
        return ("Repair".to_string(), character.skills.repair as i32);
    }

    // Check SPECIAL stats
    if lower.contains("strength") || lower.contains("str") {
        return ("Strength".to_string(), character.special.strength as i32);
    }
    if lower.contains("perception") || lower.contains("per") {
        return (
            "Perception".to_string(),
            character.special.perception as i32,
        );
    }
    if lower.contains("endurance") || lower.contains("end") {
        return ("Endurance".to_string(), character.special.endurance as i32);
    }
    if lower.contains("charisma") || lower.contains("cha") {
        return ("Charisma".to_string(), character.special.charisma as i32);
    }
    if lower.contains("intelligence") || lower.contains("int") {
        return (
            "Intelligence".to_string(),
            character.special.intelligence as i32,
        );
    }
    if lower.contains("agility") || lower.contains("agi") {
        return ("Agility".to_string(), character.special.agility as i32);
    }
    if lower.contains("luck") || lower.contains("lck") {
        return ("Luck".to_string(), character.special.luck as i32);
    }

    // Default to Luck if we can't determine
    ("Luck".to_string(), character.special.luck as i32)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::character::Special;

    #[test]
    fn test_parse_roll_request() {
        let result = parse_roll_request("You need to make a SKILL: lockpick DC 15 check.");
        assert_eq!(result, Some(("lockpick".to_string(), 15)));

        let result = parse_roll_request("This requires a STAT: perception DC 10 roll.");
        assert_eq!(result, Some(("perception".to_string(), 10)));

        let result = parse_roll_request("SKILL: Speech DC 12");
        assert_eq!(result, Some(("Speech".to_string(), 12)));
    }

    #[test]
    fn test_get_modifier() {
        let character = Character::new(
            "Test".to_string(),
            Special {
                strength: 6,
                perception: 8,
                endurance: 5,
                charisma: 7,
                intelligence: 9,
                agility: 5,
                luck: 5,
            },
        );

        let (name, modifier) = get_modifier(&character, "lockpick");
        assert_eq!(name, "Lockpick");
        assert!(modifier >= 0);

        let (name, modifier) = get_modifier(&character, "strength");
        assert_eq!(name, "Strength");
        assert_eq!(modifier, 6);
    }

    #[test]
    fn test_perform_roll() {
        let character = Character::new(
            "Test".to_string(),
            Special {
                strength: 5,
                perception: 5,
                endurance: 5,
                charisma: 5,
                intelligence: 5,
                agility: 5,
                luck: 5,
            },
        );

        let result = perform_roll(&character, "lockpick", 15);
        assert!(result.roll >= 1 && result.roll <= 20);
        assert_eq!(result.dc, 15);
    }

    #[test]
    fn test_parse_natural_roll_request() {
        // Test natural language patterns
        let result = parse_natural_roll_request(
            "You'll need a successful Science skill check for this, Gaunt. This requires a Science check (DC 15)."
        );
        assert_eq!(result, Some(("science".to_string(), 15)));

        let result = parse_natural_roll_request(
            "This requires a Lockpick roll against DC 18 to open the safe.",
        );
        assert_eq!(result, Some(("lockpick".to_string(), 18)));

        let result =
            parse_natural_roll_request("Make a Perception check DC 12 to notice the trap.");
        assert_eq!(result, Some(("perception".to_string(), 12)));

        let result = parse_natural_roll_request(
            "You need to roll Intelligence [DC 20] to hack this terminal.",
        );
        assert_eq!(result, Some(("intelligence".to_string(), 20)));

        // Test the exact pattern from the screenshot
        let result = parse_natural_roll_request(
            "This requires a Speech check against DC 15. Roll the dice and I'll tell you the result."
        );
        assert_eq!(result, Some(("speech".to_string(), 15)));

        // Test difficulty pattern
        let result =
            parse_natural_roll_request("This needs a Science roll with difficulty 14 to succeed.");
        assert_eq!(result, Some(("science".to_string(), 14)));

        // Test DC: pattern with colon
        let result = parse_natural_roll_request("Make a Lockpick check DC: 16");
        assert_eq!(result, Some(("lockpick".to_string(), 16)));

        // Test that non-skill check text doesn't trigger
        let result = parse_natural_roll_request("You walk into the room and see a desk.");
        assert_eq!(result, None);
    }
}
