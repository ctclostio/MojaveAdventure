//! # Consumables Module
//!
//! Consumable item effects for the Fallout RPG system.
//!
//! ## Effect Types
//!
//! - **Healing**: Restores hit points (Stimpaks, food)
//! - **RadAway**: Removes accumulated radiation
//! - **StatBuff**: Temporary SPECIAL stat increases (chems)
//! - **Addiction**: Warning effect for addictive substances
//!
//! ## Common Consumables
//!
//! | Item | Effect | Value |
//! |------|--------|-------|
//! | Stimpak | Healing(30) | 50 caps |
//! | Super Stimpak | Healing(60) | 100 caps |
//! | RadAway | RadAway(50) | 75 caps |
//! | Buffout | +2 STR, 300 rounds | 100 caps |
//! | Mentats | +2 INT, 300 rounds | 100 caps |
//! | Jet | +2 AGI, 100 rounds | 80 caps |

use serde::{Deserialize, Serialize};
use smartstring::alias::String as SmartString;

/// Effects that consumable items can have when used.
///
/// Consumables are single-use items that provide immediate benefits.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ConsumableEffect {
    /// Restores hit points
    ///
    /// # Example
    /// ```
    /// use fallout_dnd::game::items::ConsumableEffect;
    ///
    /// let stimpak_effect = ConsumableEffect::Healing(30);
    /// ```
    Healing(i32),

    /// Removes accumulated radiation
    ///
    /// # Example
    /// ```
    /// use fallout_dnd::game::items::ConsumableEffect;
    ///
    /// let radaway_effect = ConsumableEffect::RadAway(50);
    /// ```
    RadAway(i32),

    /// Temporarily boosts a SPECIAL stat
    ///
    /// # Fields
    /// * `stat` - Name of the SPECIAL stat to boost
    /// * `amount` - Bonus amount added to the stat
    /// * `duration` - Number of combat rounds the effect lasts
    ///
    /// # Example
    /// ```
    /// use fallout_dnd::game::items::ConsumableEffect;
    /// use smartstring::alias::String as SmartString;
    ///
    /// let buffout = ConsumableEffect::StatBuff {
    ///     stat: SmartString::from("strength"),
    ///     amount: 2,
    ///     duration: 300,
    /// };
    /// ```
    StatBuff {
        stat: SmartString,
        amount: i32,
        duration: u32,
    },

    /// Addiction warning effect (for flavor/roleplay)
    ///
    /// Used to indicate that a substance is addictive and may
    /// have negative consequences with repeated use.
    Addiction { effect: SmartString },
}

// Test-only helper methods
#[cfg(test)]
impl ConsumableEffect {
    pub fn healing(amount: i32) -> Self {
        Self::Healing(amount)
    }

    pub fn rad_away(amount: i32) -> Self {
        Self::RadAway(amount)
    }

    pub fn stat_buff(stat: &str, amount: i32, duration: u32) -> Self {
        Self::StatBuff {
            stat: SmartString::from(stat),
            amount,
            duration,
        }
    }

    pub fn addiction(effect: &str) -> Self {
        Self::Addiction {
            effect: SmartString::from(effect),
        }
    }

    pub fn description(&self) -> String {
        match self {
            Self::Healing(amount) => format!("Restores {} HP", amount),
            Self::RadAway(amount) => format!("Removes {} rads", amount),
            Self::StatBuff {
                stat,
                amount,
                duration,
            } => {
                format!("+{} {} for {} rounds", amount, stat, duration)
            }
            Self::Addiction { effect } => format!("Warning: {}", effect),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_healing_effect() {
        let effect = ConsumableEffect::Healing(30);
        match effect {
            ConsumableEffect::Healing(amount) => assert_eq!(amount, 30),
            _ => panic!("Expected Healing effect"),
        }
    }

    #[test]
    fn test_radaway_effect() {
        let effect = ConsumableEffect::RadAway(50);
        match effect {
            ConsumableEffect::RadAway(amount) => assert_eq!(amount, 50),
            _ => panic!("Expected RadAway effect"),
        }
    }

    #[test]
    fn test_stat_buff_effect() {
        let effect = ConsumableEffect::StatBuff {
            stat: SmartString::from("strength"),
            amount: 2,
            duration: 300,
        };

        match effect {
            ConsumableEffect::StatBuff {
                stat,
                amount,
                duration,
            } => {
                assert_eq!(stat, "strength");
                assert_eq!(amount, 2);
                assert_eq!(duration, 300);
            }
            _ => panic!("Expected StatBuff effect"),
        }
    }

    #[test]
    fn test_addiction_effect() {
        let effect = ConsumableEffect::Addiction {
            effect: SmartString::from("Psycho Addiction: -1 STR when not using"),
        };

        match effect {
            ConsumableEffect::Addiction { effect } => {
                assert!(effect.contains("Psycho"));
            }
            _ => panic!("Expected Addiction effect"),
        }
    }

    #[test]
    fn test_helper_constructors() {
        let healing = ConsumableEffect::healing(25);
        assert_eq!(healing, ConsumableEffect::Healing(25));

        let radaway = ConsumableEffect::rad_away(40);
        assert_eq!(radaway, ConsumableEffect::RadAway(40));

        let buff = ConsumableEffect::stat_buff("agility", 3, 100);
        match buff {
            ConsumableEffect::StatBuff {
                stat,
                amount,
                duration,
            } => {
                assert_eq!(stat, "agility");
                assert_eq!(amount, 3);
                assert_eq!(duration, 100);
            }
            _ => panic!("Expected StatBuff"),
        }

        let addiction = ConsumableEffect::addiction("Test addiction");
        match addiction {
            ConsumableEffect::Addiction { effect } => {
                assert_eq!(effect, "Test addiction");
            }
            _ => panic!("Expected Addiction"),
        }
    }

    #[test]
    fn test_effect_description() {
        assert_eq!(
            ConsumableEffect::Healing(30).description(),
            "Restores 30 HP"
        );
        assert_eq!(
            ConsumableEffect::RadAway(50).description(),
            "Removes 50 rads"
        );
        assert_eq!(
            ConsumableEffect::stat_buff("strength", 2, 300).description(),
            "+2 strength for 300 rounds"
        );
        assert!(ConsumableEffect::addiction("Bad stuff")
            .description()
            .contains("Warning"));
    }

    #[test]
    fn test_consumable_effect_serialization() {
        let effects = vec![
            ConsumableEffect::Healing(30),
            ConsumableEffect::RadAway(50),
            ConsumableEffect::stat_buff("intelligence", 2, 300),
            ConsumableEffect::addiction("Test"),
        ];

        for effect in effects {
            let json = serde_json::to_string(&effect).unwrap();
            let deserialized: ConsumableEffect = serde_json::from_str(&json).unwrap();
            assert_eq!(deserialized, effect);
        }
    }

    #[test]
    fn test_consumable_effect_equality() {
        let heal1 = ConsumableEffect::Healing(30);
        let heal2 = ConsumableEffect::Healing(30);
        let heal3 = ConsumableEffect::Healing(50);

        assert_eq!(heal1, heal2);
        assert_ne!(heal1, heal3);
        assert_ne!(heal1, ConsumableEffect::RadAway(30));
    }
}
