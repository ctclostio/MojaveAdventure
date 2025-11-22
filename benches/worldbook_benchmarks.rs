//! Worldbook system benchmarks
//!
//! Measures performance of worldbook operations:
//! - Location lookups
//! - NPC queries
//! - Context building for AI prompts
//! - Serialization/deserialization

use divan::Bencher;
use fallout_dnd::game::worldbook::Worldbook;

fn main() {
    divan::main();
}

/// Benchmark creating a default worldbook
#[divan::bench]
fn worldbook_creation(bencher: Bencher) {
    bencher.bench_local(Worldbook::with_defaults);
}

/// Benchmark location lookup
#[divan::bench]
fn location_lookup(bencher: Bencher) {
    let wb = Worldbook::with_defaults();

    bencher.bench_local(|| wb.locations.get("vault_13"));
}

/// Benchmark NPC lookup
#[divan::bench]
fn npc_lookup(bencher: Bencher) {
    let wb = Worldbook::with_defaults();

    bencher.bench_local(|| wb.npcs.values().next());
}

/// Benchmark building worldbook context for AI
#[divan::bench]
fn build_context(bencher: Bencher) {
    let wb = Worldbook::with_defaults();

    bencher.bench_local(|| wb.build_context());
}

/// Benchmark worldbook serialization (save)
#[divan::bench]
fn worldbook_serialization(bencher: Bencher) {
    let wb = Worldbook::with_defaults();

    bencher.bench_local(|| serde_json::to_string(&wb));
}

/// Benchmark worldbook deserialization (load)
#[divan::bench]
fn worldbook_deserialization(bencher: Bencher) {
    bencher
        .with_inputs(|| {
            let wb = Worldbook::with_defaults();
            serde_json::to_string(&wb).unwrap()
        })
        .bench_values(|json_data| serde_json::from_str::<Worldbook>(&json_data));
}

/// Benchmark counting locations
#[divan::bench]
fn count_locations(bencher: Bencher) {
    let wb = Worldbook::with_defaults();

    bencher.bench_local(|| wb.locations.len());
}

/// Benchmark counting NPCs
#[divan::bench]
fn count_npcs(bencher: Bencher) {
    let wb = Worldbook::with_defaults();

    bencher.bench_local(|| wb.npcs.len());
}
