use super::combat::{Enemy, attack_roll, calculate_damage, resolve_stat_modifiers};
use super::GameState;
use crate::ui::UI;
use colored::*;

/// Handle a player attack on a specific enemy
pub fn handle_player_attack(game_state: &mut GameState, target_idx: usize) {
    // Get enemy info without holding a reference
    let (enemy_alive, enemy_ac, enemy_name) = {
        let enemy = &game_state.combat.enemies[target_idx];
        (enemy.is_alive(), enemy.armor_class, enemy.name.clone())
    };

    if !enemy_alive {
        UI::print_error("That enemy is already dead!");
        return;
    }

    let weapon_damage = game_state.character.get_equipped_damage();
    // Resolve stat modifiers in damage string (e.g., "1d8+STR" -> "1d8+3")
    let resolved_damage = resolve_stat_modifiers(&weapon_damage, game_state.character.special.strength);

    // Use appropriate skill based on equipped weapon type
    let skill = game_state.character.get_weapon_skill();

    let (hit, critical) = attack_roll(skill, enemy_ac);

    if !game_state.character.use_ap(4) {
        UI::print_error("Not enough AP!");
        return;
    }

    if hit {
        // No additional stat bonus since it's already in the weapon damage
        let damage = calculate_damage(&resolved_damage, 0, critical);
        game_state.combat.enemies[target_idx].take_damage(damage);

        if critical {
            println!("{} Critical hit! {} damage to {}!",
                "⚡".yellow(), damage, enemy_name);
        } else {
            println!("{} Hit! {} damage to {}!",
                "→".green(), damage, enemy_name);
        }
    } else {
        println!("{} Missed!", "✗".red());
    }

    println!();
}

/// Handle enemy turns - each enemy attacks the player
pub fn handle_enemy_turns(game_state: &mut GameState) {
    for enemy in &mut game_state.combat.enemies {
        if !enemy.is_alive() {
            continue;
        }

        // Use enemy's actual skill and calculate damage with their strength bonus
        let enemy_skill = enemy.skill;
        let enemy_strength = enemy.strength;
        let enemy_damage = enemy.damage.clone();

        let player_ac = 10 + game_state.character.special.agility as i32;
        let (hit, critical) = attack_roll(enemy_skill, player_ac);

        if hit {
            // Apply enemy strength bonus to damage
            let str_bonus = (enemy_strength / 2) as i32;
            let damage = calculate_damage(&enemy_damage, str_bonus, critical);
            game_state.character.take_damage(damage);

            if critical {
                println!("{} {} lands a CRITICAL hit! {} damage!",
                    "⚠".red().bold(), enemy.name, damage);
            } else {
                println!("{} {} hits for {} damage!",
                    "←".red(), enemy.name, damage);
            }
        } else {
            println!("{} {} misses!", "○".dimmed(), enemy.name);
        }
    }

    game_state.character.restore_ap();
    game_state.combat.next_round();
    println!();
}

/// Start a random combat encounter based on player level
pub fn start_combat_encounter(game_state: &mut GameState) {
    use rand::Rng;
    let mut rng = rand::thread_rng();

    let level = game_state.character.level;
    let encounter_type = rng.gen_range(0..3);

    let enemies = match encounter_type {
        0 => {
            // Radroach encounter
            UI::print_info("You encounter some mutated insects!");
            vec![
                Enemy::radroach(level),
                Enemy::radroach(level),
                Enemy::radroach(level),
            ]
        }
        1 => {
            // Raider encounter
            UI::print_info("Raiders spot you and attack!");
            vec![
                Enemy::raider(level),
                Enemy::raider(level),
            ]
        }
        _ => {
            // Super Mutant encounter (harder)
            UI::print_info("A Super Mutant emerges from the ruins!");
            vec![
                Enemy::super_mutant(level),
            ]
        }
    };

    game_state.combat.start_combat(enemies);
    UI::print_success("Combat started!");
}

/// End combat and award XP
pub fn end_combat(game_state: &mut GameState) {
    let xp = game_state.combat.total_xp_reward();
    game_state.character.add_experience(xp);

    UI::print_success(&format!("Combat ended! Gained {} XP", xp));
    if game_state.character.experience >= (game_state.character.level + 1) * 1000 {
        UI::print_success(&format!("LEVEL UP! Now level {}", game_state.character.level));
    }

    game_state.combat.end_combat();
    UI::wait_for_enter();
}
