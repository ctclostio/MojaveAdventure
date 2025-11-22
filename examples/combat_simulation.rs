//! Demonstrate combat mechanics without AI
//!
//! This example demonstrates:
//! - Turn-based combat system
//! - Action point mechanics
//! - Attack rolls and damage calculation
//! - Enemy AI behavior
//! - Combat log with colorized output
//!
//! Run: cargo run --example combat_simulation

use colored::Colorize;
use fallout_dnd::game::character::{Character, Special};
use fallout_dnd::game::combat::{
    attack_roll, calculate_damage, resolve_stat_modifiers, CombatState, Enemy,
};

fn main() {
    println!(
        "{}",
        "=== Fallout D&D - Combat Simulation Example ===\n"
            .bold()
            .cyan()
    );

    // Create player character
    let mut special = Special::new();
    special.strength = 6;
    special.perception = 6;
    special.endurance = 6;
    special.agility = 7;
    special.luck = 5;

    let mut player = Character::new("Wasteland Warrior", special);

    println!("{}", "Creating Player Character:".bold());
    println!("  Name: {}", player.name);
    println!("  HP: {}/{}", player.current_hp, player.max_hp);
    println!("  AP: {}/{}", player.current_ap, player.max_ap);
    println!("  Equipped: {}", player.equipped_weapon.as_ref().unwrap());
    println!("  Weapon Skill: {}%", player.get_weapon_skill());
    println!();

    // Create enemies
    let enemies = vec![Enemy::raider(2), Enemy::radroach(1)];

    println!("{}", "Enemy Forces:".bold());
    for (i, enemy) in enemies.iter().enumerate() {
        println!(
            "  {}. {} - HP: {}/{}, AC: {}, Skill: {}%",
            i + 1,
            enemy.name.bright_red(),
            enemy.current_hp,
            enemy.max_hp,
            enemy.armor_class,
            enemy.skill
        );
    }
    println!();

    // Start combat
    let mut combat = CombatState::new();
    combat.start_combat(enemies);

    println!(
        "{}",
        "╔════════════════════════════════════════╗".bright_yellow()
    );
    println!(
        "{}",
        "║         COMBAT INITIATED!              ║"
            .bright_yellow()
            .bold()
    );
    println!(
        "{}",
        "╚════════════════════════════════════════╝".bright_yellow()
    );
    println!();

    // Combat loop
    while combat.active {
        println!(
            "{}",
            format!("▼ ROUND {} ▼", combat.round).bold().bright_cyan()
        );
        println!(
            "{}",
            "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".bright_cyan()
        );

        // Player turn
        println!("{}", "\n[PLAYER TURN]".green().bold());
        player.restore_ap();
        println!("  AP: {}/{}", player.current_ap, player.max_ap);
        println!();

        // Attack each living enemy once
        for i in 0..combat.enemies.len() {
            if !combat.enemies[i].is_alive() {
                continue;
            }

            if player.current_ap < 4 {
                println!("  {} Not enough AP to attack!", "→".yellow());
                break;
            }

            player.use_ap(4);

            let enemy = &mut combat.enemies[i];
            let player_skill = player.get_weapon_skill();
            let enemy_ac = 10 + enemy.armor_class;

            println!(
                "  {} Attacking {} (AC: {})...",
                "→".yellow(),
                enemy.name.bright_red(),
                enemy_ac
            );

            let (hit, critical) = attack_roll(player_skill, enemy_ac);

            if hit {
                let weapon_damage = player.get_equipped_damage();
                let resolved_damage =
                    resolve_stat_modifiers(&weapon_damage, player.special.strength);
                let damage = calculate_damage(&resolved_damage, 0, critical);

                enemy.take_damage(damage);

                if critical {
                    println!(
                        "    {} CRITICAL HIT! Dealt {} damage!",
                        "✦".bright_yellow().bold(),
                        damage.to_string().bright_yellow().bold()
                    );
                } else {
                    println!(
                        "    {} Hit! Dealt {} damage.",
                        "✓".green(),
                        damage.to_string().green()
                    );
                }

                if !enemy.is_alive() {
                    println!(
                        "    {} {} eliminated! (+{} XP)",
                        "✖".red().bold(),
                        enemy.name.red(),
                        enemy.xp_reward.to_string().cyan()
                    );
                } else {
                    println!(
                        "    {} has {}/{} HP remaining.",
                        enemy.name.bright_red(),
                        enemy.current_hp,
                        enemy.max_hp
                    );
                }
            } else {
                println!("    {} Miss!", "✗".red());
            }
            println!();
        }

        // Check if all enemies defeated
        if combat.all_enemies_dead() {
            let total_xp = combat.total_xp_reward();
            println!(
                "{}",
                "╔════════════════════════════════════════╗".bright_green()
            );
            println!(
                "{}",
                "║         VICTORY!                       ║"
                    .bright_green()
                    .bold()
            );
            println!(
                "{}",
                "╚════════════════════════════════════════╝".bright_green()
            );
            println!("\n{} All enemies defeated!", "✓".green().bold());
            println!("  Total XP gained: {}", total_xp.to_string().cyan());
            player.add_experience(total_xp);
            println!("  Character XP: {}", player.experience);

            combat.end_combat();
            break;
        }

        // Enemy turn
        println!("{}", "[ENEMY TURN]".red().bold());

        for enemy in &mut combat.enemies {
            if !enemy.is_alive() {
                continue;
            }

            println!(
                "  {} {} attacks! (Skill: {}%)",
                "→".red(),
                enemy.name.bright_red(),
                enemy.skill
            );

            let player_ac = 10 + (player.special.agility as i32);
            let (hit, critical) = attack_roll(enemy.skill, player_ac);

            if hit {
                let resolved_damage = resolve_stat_modifiers(&enemy.damage, enemy.strength);
                let damage = calculate_damage(&resolved_damage, 0, critical);

                player.take_damage(damage);

                if critical {
                    println!(
                        "    {} CRITICAL HIT! Took {} damage!",
                        "✦".bright_red().bold(),
                        damage.to_string().bright_red().bold()
                    );
                } else {
                    println!(
                        "    {} Hit! Took {} damage.",
                        "✓".red(),
                        damage.to_string().red()
                    );
                }

                if !player.is_alive() {
                    println!("    {} You have been defeated!", "✖".red().bold());
                    combat.end_combat();
                    break;
                } else {
                    println!("    Player HP: {}/{}", player.current_hp, player.max_hp);
                }
            } else {
                println!("    {} Miss!", "✗".green());
            }
            println!();
        }

        // Check if player died
        if !player.is_alive() {
            println!(
                "{}",
                "╔════════════════════════════════════════╗".bright_red()
            );
            println!(
                "{}",
                "║         DEFEAT...                      ║"
                    .bright_red()
                    .bold()
            );
            println!(
                "{}",
                "╚════════════════════════════════════════╝".bright_red()
            );
            break;
        }

        combat.next_round();
        println!();
    }

    println!("\n{}", "=== Combat Statistics ===".bold());
    println!("  Final Player HP: {}/{}", player.current_hp, player.max_hp);
    println!("  Player XP: {}", player.experience);
    println!("  Combat lasted {} rounds", combat.round);
    println!();
    println!("{}", "Combat simulation complete!".bright_cyan());
}
