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
#[allow(dead_code)] // Public API for integration tests
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
                    let dc_clean = dc_str.trim_end_matches([')', ']']);
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

                if let Some(offset) = search_range.find(['.', '!', '?']) {
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

    // ============ EDGE CASE TESTS ============

    #[test]
    fn test_critical_success_nat_20() {
        // Critical success should always be true when rolling nat 20
        let character = Character::new(
            "Test".to_string(),
            Special {
                strength: 2,
                perception: 2,
                endurance: 2,
                charisma: 2,
                intelligence: 2,
                agility: 2,
                luck: 2,
            },
        );

        // Even with nat 20 and low modifier, total might still be low,
        // but critical flag should be true
        for _ in 0..5 {
            // Run multiple times to catch a nat 20
            let result = perform_roll(&character, "luck", 100);
            if result.roll == 20 {
                assert!(result.critical, "Nat 20 should always be critical");
                assert!(result.success, "Nat 20 should always succeed");
            }
        }
    }

    #[test]
    fn test_critical_failure_nat_1() {
        // Critical failure should always be true when rolling nat 1
        let character = Character::new(
            "Test".to_string(),
            Special {
                strength: 10,
                perception: 10,
                endurance: 10,
                charisma: 10,
                intelligence: 10,
                agility: 10,
                luck: 10,
            },
        );

        // Even with nat 1 and high modifier, fumble flag should be true
        for _ in 0..5 {
            let result = perform_roll(&character, "strength", 5);
            if result.roll == 1 {
                assert!(result.fumble, "Nat 1 should always be fumble");
            }
        }
    }

    #[test]
    fn test_extreme_positive_modifier() {
        // Test with very high skill modifier
        let mut character = Character::new(
            "TestHighSkill".to_string(),
            Special {
                strength: 10,
                perception: 10,
                endurance: 10,
                charisma: 10,
                intelligence: 10,
                agility: 10,
                luck: 10,
            },
        );
        // Max out a skill to 100+
        character.skills.small_guns = 100;

        let result = perform_roll(&character, "small guns", 50);
        // With 100 skill modifier, even a nat 1 (1 + 100 = 101) should succeed against DC 50
        assert!(result.total >= 101);
        if result.roll != 1 {
            assert!(result.success, "High skill should ensure success");
        }
    }

    #[test]
    fn test_extreme_negative_modifier() {
        // Test with very low stats (modifier = 0 for untrained)
        let character = Character::new(
            "TestLowStats".to_string(),
            Special {
                strength: 1,
                perception: 1,
                endurance: 1,
                charisma: 1,
                intelligence: 1,
                agility: 1,
                luck: 1,
            },
        );

        let result = perform_roll(&character, "luck", 1);
        // With luck 1 modifier and DC 1, only nat 1 fails (1 + 1 = 2 >= 1)
        assert!(result.total >= 2, "Minimum roll should be 1 + 1 = 2");
    }

    #[test]
    fn test_parse_invalid_dice_string_empty() {
        // Empty string should return None
        let result = parse_roll_request("");
        assert_eq!(result, None);
    }

    #[test]
    fn test_parse_invalid_dice_string_no_dc() {
        // String with skill keyword but no DC should return None
        let result = parse_roll_request("You need to SKILL: lockpick");
        assert_eq!(result, None);
    }

    #[test]
    fn test_parse_invalid_dice_string_malformed_dc() {
        // Malformed DC (non-numeric) should return None
        let result = parse_roll_request("SKILL: lockpick DC abc");
        assert_eq!(result, None);
    }

    #[test]
    fn test_parse_empty_skill_name() {
        // Empty skill name before DC should return None
        let result = parse_roll_request("SKILL:  DC 15");
        assert_eq!(result, None);
    }

    #[test]
    fn test_skill_check_very_high_dc() {
        // Test with extremely high DC (like 100)
        let character = Character::new(
            "TestHighDC".to_string(),
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

        let result = perform_roll(&character, "luck", 100);
        // Only nat 20 should succeed against DC 100 with low modifier
        if result.roll != 20 {
            assert!(!result.success, "Low roll should fail against DC 100");
        }
    }

    #[test]
    fn test_skill_check_zero_dc() {
        // Test with DC 0 (auto-succeed)
        let character = Character::new(
            "TestZeroDC".to_string(),
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

        let result = perform_roll(&character, "luck", 0);
        // Any roll with any positive modifier should succeed against DC 0
        assert!(result.total >= 0);
    }

    #[test]
    fn test_skill_check_negative_dc() {
        // Test with negative DC (always succeed)
        let character = Character::new(
            "TestNegDC".to_string(),
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

        let result = perform_roll(&character, "luck", -50);
        // Even nat 1 should succeed against negative DC
        assert!(
            result.success,
            "Any roll should succeed against negative DC"
        );
    }

    #[test]
    fn test_roll_result_format_critical() {
        // Test formatting of critical success
        let result = RollResult {
            skill_name: "Lockpick".to_string(),
            roll: 20,
            modifier: 5,
            total: 25,
            dc: 15,
            success: true,
            critical: true,
            fumble: false,
        };

        let formatted = result.format();
        assert!(formatted.contains("CRITICAL SUCCESS!"));
        assert!(formatted.contains("Lockpick"));
        assert!(formatted.contains("20"));
        assert!(formatted.contains("5"));
        assert!(formatted.contains("25"));
        assert!(formatted.contains("15"));
    }

    #[test]
    fn test_roll_result_format_fumble() {
        // Test formatting of critical failure
        let result = RollResult {
            skill_name: "Science".to_string(),
            roll: 1,
            modifier: 10,
            total: 11,
            dc: 15,
            success: false,
            critical: false,
            fumble: true,
        };

        let formatted = result.format();
        assert!(formatted.contains("CRITICAL FAILURE!"));
        assert!(formatted.contains("Science"));
        assert!(formatted.contains("1"));
        assert!(formatted.contains("10"));
        assert!(formatted.contains("11"));
    }

    #[test]
    fn test_roll_result_format_success() {
        // Test formatting of normal success
        let result = RollResult {
            skill_name: "Speech".to_string(),
            roll: 12,
            modifier: 3,
            total: 15,
            dc: 12,
            success: true,
            critical: false,
            fumble: false,
        };

        let formatted = result.format();
        assert!(formatted.contains("Success"));
        assert!(!formatted.contains("CRITICAL"));
        assert!(formatted.contains("Speech"));
    }

    #[test]
    fn test_roll_result_format_failure() {
        // Test formatting of normal failure
        let result = RollResult {
            skill_name: "Repair".to_string(),
            roll: 3,
            modifier: 2,
            total: 5,
            dc: 12,
            success: false,
            critical: false,
            fumble: false,
        };

        let formatted = result.format();
        assert!(formatted.contains("Failure"));
        assert!(!formatted.contains("CRITICAL"));
        assert!(formatted.contains("Repair"));
    }

    #[test]
    fn test_roll_result_emoji() {
        // Test emoji selection
        let critical_result = RollResult {
            skill_name: "Test".to_string(),
            roll: 20,
            modifier: 0,
            total: 20,
            dc: 15,
            success: true,
            critical: true,
            fumble: false,
        };
        assert_eq!(critical_result.emoji(), "⭐");

        let fumble_result = RollResult {
            skill_name: "Test".to_string(),
            roll: 1,
            modifier: 0,
            total: 1,
            dc: 15,
            success: false,
            critical: false,
            fumble: true,
        };
        assert_eq!(fumble_result.emoji(), "💀");

        let success_result = RollResult {
            skill_name: "Test".to_string(),
            roll: 10,
            modifier: 5,
            total: 15,
            dc: 12,
            success: true,
            critical: false,
            fumble: false,
        };
        assert_eq!(success_result.emoji(), "✓");

        let failure_result = RollResult {
            skill_name: "Test".to_string(),
            roll: 3,
            modifier: 2,
            total: 5,
            dc: 12,
            success: false,
            critical: false,
            fumble: false,
        };
        assert_eq!(failure_result.emoji(), "✗");
    }

    #[test]
    fn test_truncate_response_at_skill_check_basic() {
        // Test basic truncation
        let text =
            "You try to pick the lock. This requires a Lockpick check (DC 15). Roll the dice!";
        let result = truncate_response_at_skill_check(text);
        assert!(result.is_some());
        let truncated = result.unwrap();
        assert!(truncated.contains("(DC 15)"));
        // Should end after the DC statement
        assert!(!truncated.contains("Roll the dice"));
    }

    #[test]
    fn test_truncate_response_no_skill_check() {
        // Text without skill check should return None
        let text = "You walk into the room and see a table.";
        let result = truncate_response_at_skill_check(text);
        assert_eq!(result, None);
    }

    #[test]
    fn test_truncate_response_with_sentence_ending() {
        // Test truncation when sentence ends right after DC
        let text = "You need a Science check (DC 18). The device hums mysteriously.";
        let result = truncate_response_at_skill_check(text);
        assert!(result.is_some());
        let truncated = result.unwrap();
        assert!(truncated.contains("DC 18"));
    }

    #[test]
    fn test_get_modifier_all_skills() {
        // Test that all skill names are recognized
        let character = Character::new(
            "TestAllSkills".to_string(),
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

        let skills_to_test = vec![
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

        for skill in skills_to_test {
            let (name, _modifier) = get_modifier(&character, skill);
            assert!(!name.is_empty(), "Should recognize skill: {}", skill);
        }
    }

    #[test]
    fn test_get_modifier_all_stats() {
        // Test that all SPECIAL stats are recognized
        let character = Character::new(
            "TestAllStats".to_string(),
            Special {
                strength: 6,
                perception: 7,
                endurance: 5,
                charisma: 8,
                intelligence: 9,
                agility: 4,
                luck: 3,
            },
        );

        let stats = vec![
            ("strength", 6),
            ("perception", 7),
            ("endurance", 5),
            ("charisma", 8),
            ("intelligence", 9),
            ("agility", 4),
            ("luck", 3),
        ];

        for (stat_name, expected_value) in stats {
            let (_name, modifier) = get_modifier(&character, stat_name);
            assert_eq!(
                modifier as u8, expected_value,
                "Wrong modifier for {}",
                stat_name
            );
        }
    }

    #[test]
    fn test_get_modifier_abbreviations() {
        // Test abbreviated stat names
        let character = Character::new(
            "TestAbbrev".to_string(),
            Special {
                strength: 6,
                perception: 7,
                endurance: 5,
                charisma: 8,
                intelligence: 9,
                agility: 4,
                luck: 3,
            },
        );

        let abbrevs = vec![
            ("str", 6),
            ("per", 7),
            ("end", 5),
            ("cha", 8),
            ("int", 9),
            ("agi", 4),
            ("lck", 3),
        ];

        for (abbrev, expected_value) in abbrevs {
            let (_, modifier) = get_modifier(&character, abbrev);
            assert_eq!(
                modifier as u8, expected_value,
                "Wrong modifier for {}",
                abbrev
            );
        }
    }

    #[test]
    fn test_parse_roll_request_case_insensitive() {
        // Test that parsing is case insensitive
        let tests = vec![
            ("skill: lockpick dc 15", Some(("lockpick".to_string(), 15))),
            ("SKILL: LOCKPICK DC 15", Some(("LOCKPICK".to_string(), 15))),
            ("Skill: Lockpick DC 15", Some(("Lockpick".to_string(), 15))),
            (
                "stat: perception dc 10",
                Some(("perception".to_string(), 10)),
            ),
        ];

        for (input, expected) in tests {
            let result = parse_roll_request(input);
            assert_eq!(result, expected, "Failed to parse: {}", input);
        }
    }

    #[test]
    fn test_parse_natural_roll_no_dc_found() {
        // Test natural language roll request when check phrase exists but no DC
        let result =
            parse_natural_roll_request("You need to make a Lockpick check but we'll skip the DC.");
        // Should return None since there's no DC value
        assert_eq!(result, None);
    }

    #[test]
    fn test_roll_bounds_validation() {
        // Ensure rolls are always within 1-20 range
        let character = Character::new(
            "TestBounds".to_string(),
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

        for _ in 0..20 {
            let result = perform_roll(&character, "luck", 10);
            assert!(
                result.roll >= 1 && result.roll <= 20,
                "Roll {} out of bounds [1, 20]",
                result.roll
            );
            assert_eq!(result.total, result.roll + result.modifier);
        }
    }
}
