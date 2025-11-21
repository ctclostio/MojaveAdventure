//! Combat system benchmarks
//!
//! Measures performance of combat operations:
//! - Damage calculations
//! - Attack rolls
//! - Combat state updates
//! - Enemy creation

use divan::Bencher;
use fallout_dnd::game::combat::{
    attack_roll, calculate_damage, resolve_stat_modifiers, roll_dice, CombatState, Enemy,
};

fn main() {
    divan::main();
}

/// Benchmark damage calculation
#[divan::bench]
fn damage_calculation(bencher: Bencher) {
    bencher.bench_local(|| calculate_damage("2d6+3", 5, false));
}

/// Benchmark critical damage calculation
#[divan::bench]
fn critical_damage_calculation(bencher: Bencher) {
    bencher.bench_local(|| calculate_damage("2d6+3", 5, true));
}

/// Benchmark attack roll
#[divan::bench]
fn attack_roll_bench(bencher: Bencher) {
    bencher.bench_local(|| attack_roll(50, 15));
}

/// Benchmark dice rolling
#[divan::bench]
fn dice_rolling(bencher: Bencher) {
    bencher.bench_local(|| roll_dice("2d6+3"));
}

/// Benchmark stat modifier resolution (hot path)
#[divan::bench]
fn stat_modifier_resolution(bencher: Bencher) {
    bencher.bench_local(|| resolve_stat_modifiers("1d8+STR", 6));
}

/// Benchmark stat modifier resolution without STR (should use Cow::Borrowed)
#[divan::bench]
fn stat_modifier_no_replacement(bencher: Bencher) {
    bencher.bench_local(|| resolve_stat_modifiers("2d6+3", 6));
}

/// Benchmark combat state creation with varying enemy counts
#[divan::bench(args = [1, 3, 5, 8])]
fn combat_state_creation(bencher: Bencher, enemy_count: usize) {
    bencher
        .with_inputs(|| {
            (0..enemy_count)
                .map(|i| Enemy::raider((i + 1) as u32))
                .collect::<Vec<_>>()
        })
        .bench_values(|enemies| {
            let mut combat = CombatState::new();
            combat.start_combat(enemies);
            combat
        });
}

/// Benchmark resolving a full combat turn
#[divan::bench]
fn resolve_combat_turn(bencher: Bencher) {
    bencher
        .with_inputs(|| {
            let mut combat = CombatState::new();
            let enemies = vec![Enemy::raider(3), Enemy::raider(2), Enemy::raider(4)];
            combat.start_combat(enemies);
            combat
        })
        .bench_values(|mut combat| {
            // Simulate player attack
            if let Some(enemy) = combat.enemies.first_mut() {
                let damage = calculate_damage("2d6+3", 5, false);
                enemy.current_hp -= damage;
            }
            combat.next_round();
            combat
        });
}

/// Benchmark enemy creation (tests Enemy::raider performance)
#[divan::bench]
fn enemy_creation(bencher: Bencher) {
    bencher.bench_local(|| Enemy::raider(5));
}

/// Benchmark taking damage
#[divan::bench]
fn enemy_damage_application(bencher: Bencher) {
    bencher
        .with_inputs(|| Enemy::raider(5))
        .bench_values(|mut enemy| {
            enemy.take_damage(10);
            enemy
        });
}
