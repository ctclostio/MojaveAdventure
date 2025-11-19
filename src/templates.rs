//! Template engine for AI prompts using Tera
//!
//! This module provides a centralized template rendering system for all AI prompts,
//! separating prompt logic from code and making prompts easier to maintain.

use crate::error::GameError;
use once_cell::sync::Lazy;
use serde::Serialize;
use tera::Tera;

/// Global template engine instance
static TEMPLATES: Lazy<Tera> = Lazy::new(|| match Tera::new("templates/**/*.tera") {
    Ok(t) => t,
    Err(e) => {
        eprintln!("Template parsing error: {}", e);
        panic!("Failed to initialize template engine");
    }
});

/// Character context for templates
#[derive(Serialize)]
pub struct CharacterContext {
    pub name: String,
    pub level: u8,
    pub current_hp: i32,
    pub max_hp: i32,
    pub current_ap: u8,
    pub max_ap: u8,
    pub caps: u32,
    pub special: SpecialStats,
    pub skills: SkillsContext,
}

#[derive(Serialize)]
pub struct SpecialStats {
    pub strength: u8,
    pub perception: u8,
    pub endurance: u8,
    pub charisma: u8,
    pub intelligence: u8,
    pub agility: u8,
    pub luck: u8,
}

#[derive(Serialize)]
pub struct SkillsContext {
    pub small_guns: u8,
    pub speech: u8,
    pub lockpick: u8,
    pub science: u8,
    pub sneak: u8,
}

/// Enemy context for combat templates
#[derive(Serialize)]
pub struct EnemyContext {
    pub name: String,
    pub current_hp: i32,
    pub is_alive: bool,
}

/// Combat context for templates
#[derive(Serialize)]
pub struct CombatContext {
    pub round: u32,
    pub enemies: Vec<EnemyContext>,
}

/// Render the system prompt template
pub fn render_system_prompt() -> Result<String, GameError> {
    TEMPLATES
        .render("system_prompt.tera", &tera::Context::new())
        .map_err(|e| GameError::InvalidInput(format!("Failed to render system prompt: {}", e)))
}

/// Render the entity extraction prompt template
pub fn render_extractor_prompt(narrative: &str) -> Result<String, GameError> {
    let mut context = tera::Context::new();
    context.insert("narrative", narrative);

    TEMPLATES
        .render("extractor.tera", &context)
        .map_err(|e| GameError::InvalidInput(format!("Failed to render extractor prompt: {}", e)))
}

/// Render the game context template with character, inventory, combat, and conversation data
pub fn render_context(
    character: Option<&CharacterContext>,
    inventory: Option<&[String]>,
    combat: Option<&CombatContext>,
    conversation_history: Option<&[String]>,
) -> Result<String, GameError> {
    let mut context = tera::Context::new();

    if let Some(char_data) = character {
        context.insert("character", char_data);
    }

    if let Some(inv) = inventory {
        context.insert("inventory", inv);
    }

    if let Some(combat_data) = combat {
        context.insert("combat", combat_data);
    }

    if let Some(history) = conversation_history {
        context.insert("conversation_history", history);
    }

    TEMPLATES
        .render("context.tera", &context)
        .map_err(|e| GameError::InvalidInput(format!("Failed to render context: {}", e)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_render_system_prompt() {
        let result = render_system_prompt();
        assert!(result.is_ok());
        let prompt = result.unwrap();
        assert!(prompt.contains("Dungeon Master"));
        assert!(prompt.contains("Fallout universe"));
    }

    #[test]
    fn test_render_extractor_prompt() {
        let narrative = "You meet a friendly merchant named Bob.";
        let result = render_extractor_prompt(narrative);
        assert!(result.is_ok());
        let prompt = result.unwrap();
        assert!(prompt.contains(narrative));
        assert!(prompt.contains("entity extractor"));
    }

    #[test]
    fn test_render_context_minimal() {
        let result = render_context(None, None, None, None);
        assert!(result.is_ok());
    }

    #[test]
    fn test_render_context_with_character() {
        let character = CharacterContext {
            name: "Vault Dweller".to_string(),
            level: 5,
            current_hp: 50,
            max_hp: 75,
            current_ap: 8,
            max_ap: 10,
            caps: 250,
            special: SpecialStats {
                strength: 6,
                perception: 7,
                endurance: 5,
                charisma: 4,
                intelligence: 8,
                agility: 7,
                luck: 5,
            },
            skills: SkillsContext {
                small_guns: 45,
                speech: 38,
                lockpick: 42,
                science: 55,
                sneak: 40,
            },
        };

        let result = render_context(Some(&character), None, None, None);
        assert!(result.is_ok());
        let context = result.unwrap();
        assert!(context.contains("Vault Dweller"));
        assert!(context.contains("Level 5"));
    }

    #[test]
    fn test_render_context_with_inventory() {
        let inventory = vec!["10mm Pistol".to_string(), "Stimpak".to_string()];
        let result = render_context(None, Some(&inventory), None, None);
        assert!(result.is_ok());
        let context = result.unwrap();
        assert!(context.contains("10mm Pistol"));
        assert!(context.contains("Stimpak"));
    }

    #[test]
    fn test_render_context_with_combat() {
        let combat = CombatContext {
            round: 3,
            enemies: vec![
                EnemyContext {
                    name: "Raider".to_string(),
                    current_hp: 25,
                    is_alive: true,
                },
                EnemyContext {
                    name: "Feral Ghoul".to_string(),
                    current_hp: 0,
                    is_alive: false,
                },
            ],
        };

        let result = render_context(None, None, Some(&combat), None);
        assert!(result.is_ok());
        let context = result.unwrap();
        assert!(context.contains("Round 3"));
        assert!(context.contains("Raider"));
        // Dead enemies should not appear in output
        assert!(!context.contains("Feral Ghoul"));
    }
}
