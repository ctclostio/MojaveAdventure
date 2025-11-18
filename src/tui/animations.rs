use std::collections::HashMap;
use std::time::{Duration, Instant};

/// Animation duration constants (in milliseconds)
const HP_DRAIN_DURATION: u64 = 800;
const XP_FILL_DURATION: u64 = 600;
const DICE_ROLL_DURATION: u64 = 1200;
const ENEMY_FADEOUT_DURATION: u64 = 1000;

/// Represents different types of animations that can be active
#[derive(Debug, Clone)]
pub enum AnimationType {
    /// Health bar draining animation (old_hp, new_hp)
    HealthDrain { from: i32, to: i32 },
    /// XP bar filling animation (old_xp, new_xp)
    XpFill { from: u32, to: u32 },
    /// Dice rolling animation with final result
    DiceRoll { result: u8, modifier: i32 },
    /// Enemy fadeout animation
    EnemyFadeout {
        #[allow(dead_code)]
        enemy_index: usize,
    },
}

/// Tracks a single animation with timing information
#[derive(Debug, Clone)]
pub struct Animation {
    pub anim_type: AnimationType,
    start_time: Instant,
    duration: Duration,
}

impl Animation {
    /// Create a new animation
    pub fn new(anim_type: AnimationType) -> Self {
        let duration = match anim_type {
            AnimationType::HealthDrain { .. } => Duration::from_millis(HP_DRAIN_DURATION),
            AnimationType::XpFill { .. } => Duration::from_millis(XP_FILL_DURATION),
            AnimationType::DiceRoll { .. } => Duration::from_millis(DICE_ROLL_DURATION),
            AnimationType::EnemyFadeout { .. } => Duration::from_millis(ENEMY_FADEOUT_DURATION),
        };

        Animation {
            anim_type,
            start_time: Instant::now(),
            duration,
        }
    }

    /// Get the progress of this animation (0.0 to 1.0)
    pub fn progress(&self) -> f32 {
        let elapsed = self.start_time.elapsed();
        let progress = elapsed.as_secs_f32() / self.duration.as_secs_f32();
        progress.min(1.0)
    }

    /// Check if the animation is complete
    pub fn is_complete(&self) -> bool {
        self.progress() >= 1.0
    }

    /// Get eased progress using ease-out cubic function for smooth deceleration
    pub fn eased_progress(&self) -> f32 {
        let t = self.progress();
        // Ease-out cubic: 1 - (1-t)^3
        1.0 - (1.0 - t).powi(3)
    }
}

/// Manages all active animations in the application
#[derive(Debug, Default)]
pub struct AnimationManager {
    /// Current health animation (only one at a time)
    health_animation: Option<Animation>,
    /// Current XP animation (only one at a time)
    xp_animation: Option<Animation>,
    /// Current dice roll animation (only one at a time)
    dice_animation: Option<Animation>,
    /// Enemy fadeout animations (multiple enemies can fade at once)
    enemy_fadeouts: HashMap<usize, Animation>,
}

impl AnimationManager {
    pub fn new() -> Self {
        Self::default()
    }

    /// Start a health drain animation
    pub fn start_health_drain(&mut self, from: i32, to: i32) {
        // Only animate if there's an actual change
        if from != to {
            self.health_animation = Some(Animation::new(AnimationType::HealthDrain { from, to }));
        }
    }

    /// Start an XP fill animation
    pub fn start_xp_fill(&mut self, from: u32, to: u32) {
        // Only animate if there's an actual change
        if from != to {
            self.xp_animation = Some(Animation::new(AnimationType::XpFill { from, to }));
        }
    }

    /// Start a dice roll animation
    pub fn start_dice_roll(&mut self, result: u8, modifier: i32) {
        self.dice_animation = Some(Animation::new(AnimationType::DiceRoll { result, modifier }));
    }

    /// Start an enemy fadeout animation
    pub fn start_enemy_fadeout(&mut self, enemy_index: usize) {
        self.enemy_fadeouts.insert(
            enemy_index,
            Animation::new(AnimationType::EnemyFadeout { enemy_index }),
        );
    }

    /// Get the current interpolated HP value during animation
    pub fn get_animated_hp(&self, _current_hp: i32) -> Option<i32> {
        if let Some(anim) = &self.health_animation {
            if let AnimationType::HealthDrain { from, to } = anim.anim_type {
                let progress = anim.eased_progress();
                let interpolated = from as f32 + (to - from) as f32 * progress;
                return Some(interpolated.round() as i32);
            }
        }
        None
    }

    /// Get the current interpolated XP value during animation
    pub fn get_animated_xp(&self, _current_xp: u32) -> Option<u32> {
        if let Some(anim) = &self.xp_animation {
            if let AnimationType::XpFill { from, to } = anim.anim_type {
                let progress = anim.eased_progress();
                let interpolated = from as f32 + (to - from) as f32 * progress;
                return Some(interpolated.round() as u32);
            }
        }
        None
    }

    /// Get the current dice roll animation state
    /// Returns: (is_rolling, current_display_value, final_result, modifier)
    pub fn get_dice_animation_state(&self) -> Option<(bool, u8, u8, i32)> {
        if let Some(anim) = &self.dice_animation {
            if let AnimationType::DiceRoll { result, modifier } = anim.anim_type {
                let is_rolling = !anim.is_complete();

                // During rolling, show random numbers
                let display_value = if is_rolling {
                    // Generate pseudo-random values that change each frame
                    let seed = (anim.progress() * 20.0) as u32;
                    ((seed * 7 + 13) % 20 + 1) as u8
                } else {
                    result
                };

                return Some((is_rolling, display_value, result, modifier));
            }
        }
        None
    }

    /// Get the opacity for an enemy fadeout (1.0 = fully visible, 0.0 = invisible)
    pub fn get_enemy_opacity(&self, enemy_index: usize) -> f32 {
        if let Some(anim) = self.enemy_fadeouts.get(&enemy_index) {
            1.0 - anim.eased_progress()
        } else {
            1.0
        }
    }

    /// Check if an enemy is fading out
    pub fn is_enemy_fading(&self, enemy_index: usize) -> bool {
        self.enemy_fadeouts.contains_key(&enemy_index)
    }

    /// Update all animations and remove completed ones
    pub fn update(&mut self) {
        // Clean up completed health animation
        if let Some(anim) = &self.health_animation {
            if anim.is_complete() {
                self.health_animation = None;
            }
        }

        // Clean up completed XP animation
        if let Some(anim) = &self.xp_animation {
            if anim.is_complete() {
                self.xp_animation = None;
            }
        }

        // Clean up completed dice animation
        if let Some(anim) = &self.dice_animation {
            if anim.is_complete() {
                self.dice_animation = None;
            }
        }

        // Clean up completed enemy fadeouts
        self.enemy_fadeouts.retain(|_, anim| !anim.is_complete());
    }

    /// Check if any animations are currently active
    #[allow(dead_code)]
    pub fn has_active_animations(&self) -> bool {
        self.health_animation.is_some()
            || self.xp_animation.is_some()
            || self.dice_animation.is_some()
            || !self.enemy_fadeouts.is_empty()
    }

    /// Clear all animations (useful for scene transitions)
    #[allow(dead_code)]
    pub fn clear_all(&mut self) {
        self.health_animation = None;
        self.xp_animation = None;
        self.dice_animation = None;
        self.enemy_fadeouts.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;

    #[test]
    fn test_animation_progress() {
        let anim = Animation::new(AnimationType::HealthDrain { from: 100, to: 50 });
        // Just created, should be at or near 0.0 (allow for minimal CPU time)
        let initial = anim.progress();
        assert!(
            initial < 0.01,
            "Initial progress should be near 0, got {}",
            initial
        );

        thread::sleep(Duration::from_millis(100));
        assert!(anim.progress() > 0.0 && anim.progress() < 1.0);
    }

    #[test]
    fn test_health_animation() {
        let mut manager = AnimationManager::new();
        manager.start_health_drain(100, 50);

        // Should start near 100
        let animated = manager.get_animated_hp(50).unwrap();
        assert!(animated >= 90 && animated <= 100);
    }

    #[test]
    fn test_xp_animation() {
        let mut manager = AnimationManager::new();
        manager.start_xp_fill(0, 500);

        // Should start near 0
        let animated = manager.get_animated_xp(500).unwrap();
        assert!(animated <= 50);
    }

    #[test]
    fn test_no_animation_when_same_value() {
        let mut manager = AnimationManager::new();
        manager.start_health_drain(100, 100);
        assert!(manager.get_animated_hp(100).is_none());
    }
}
