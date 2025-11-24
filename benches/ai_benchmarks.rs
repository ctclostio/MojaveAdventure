//! AI system benchmarks
//!
//! Measures performance of AI operations:
//! - Template rendering
//! - String allocations (smartstring vs String)

use divan::Bencher;

fn main() {
    divan::main();
}

/// Benchmark prompt template rendering
#[divan::bench]
fn template_rendering(bencher: Bencher) {
    bencher.bench_local(fallout_dnd::templates::render_system_prompt);
}

/// Benchmark large prompt construction
#[divan::bench]
fn large_prompt_building(bencher: Bencher) {
    let wb = fallout_dnd::game::worldbook::Worldbook::with_defaults();

    bencher.bench_local(|| {
        let mut prompt = String::with_capacity(4096);
        prompt.push_str("System: You are a Fallout DM\n\n");
        prompt.push_str(&wb.build_context());
        prompt.push_str("\n\nPlayer action: I explore the area");
        prompt
    });
}

/// Benchmark string allocation patterns (String)
#[divan::bench(args = [10, 50, 100])]
fn string_allocations(bencher: Bencher, count: usize) {
    bencher.bench_local(|| {
        let mut strings = Vec::with_capacity(count);
        for i in 0..count {
            strings.push(format!("String number {}", i));
        }
        strings
    });
}

/// Benchmark smartstring usage
#[divan::bench(args = [10, 50, 100])]
fn smartstring_allocations(bencher: Bencher, count: usize) {
    bencher.bench_local(|| {
        let mut strings = Vec::with_capacity(count);
        for i in 0..count {
            let s: smartstring::alias::String = format!("String {}", i).into();
            strings.push(s);
        }
        strings
    });
}
