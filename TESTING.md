# Testing Guide

This document describes the testing strategy and how to run tests for the Fallout D&D game.

## Overview

The testing infrastructure has been developed across multiple phases:
- **Phase 4**: Core test infrastructure (insta, proptest, serial_test)
- **Phase 5**: Narrative module tests (40 tests added)
- **Phase 6**: Benchmark suite (divan framework)
- **Phase 7**: Additional integration tests
- **Phase 8**: Regression tests and edge cases

## Test Structure

The project uses Rust's built-in test framework with several types of tests:

### Unit Tests
Located inline in source files (marked with `#[cfg(test)]`)
- `src/game/character.rs` - SPECIAL stats and character mechanics
- `src/game/combat.rs` - Dice rolling and combat calculations
- `src/game/rolls.rs` - Skill check parsing and rolls
- `src/game/worldbook.rs` - Worldbook entity tracking
- `src/game/story_manager.rs` - Story context FIFO management

### Integration Tests
Located in `tests/` directory:
- `helpers.rs` - Common test fixtures and builders
- `ai_test_helpers.rs` - Mock AI responses and test utilities
- `character_tests.rs` - Character leveling and XP system (19 tests)
- `item_tests.rs` - Items, weapons, armor, consumables (19 tests)
- `config_tests.rs` - Configuration loading and validation (16 tests)
- `combat_tests.rs` - Combat mechanics and enemy scaling (28 tests)
- `worldbook_tests.rs` - Worldbook locations, NPCs, events (27 tests)
- `persistence_tests.rs` - Save/load functionality (3 tests)
- `story_manager_tests.rs` - Story context management (29 tests)
- `stat_allocator_tests.rs` - SPECIAL point allocation UI (Phase 5)
- `handlers_tests.rs` - Game command handlers (Phase 5)
- `integration_tests.rs` - End-to-end gameplay flows (3 tests)
- `error_path_tests.rs` - Error handling and edge cases (8 tests)
- `tui_tests.rs` - Terminal UI component tests
- `validation_integration.rs` - Garde validation framework tests

### Property-Based Tests
Using `proptest` for invariant testing:
- `property_tests.rs` - Property-based tests for game invariants

### Regression Tests
Tests for previously fixed bugs:
- `regression_tests.rs` - Tests preventing bug regressions

### Benchmarks (Phase 6)
Located in `benches/` directory using **divan** framework:
- `combat_benchmarks.rs` - Combat system performance (damage, rolls, stat modifiers)
- `worldbook_benchmarks.rs` - Worldbook operations (lookups, serialization)
- `ai_benchmarks.rs` - AI prompt building and template rendering

See [PERFORMANCE.md](PERFORMANCE.md) for detailed benchmark results.

## Running Tests

### Run All Tests
```bash
cargo test
```

### Run Specific Test Suite
```bash
cargo test --test character_tests
cargo test --test combat_tests
```

### Run Specific Test
```bash
cargo test test_character_level_up
```

### Run Tests with Output
```bash
cargo test -- --nocapture
```

### Run Tests in Parallel
```bash
cargo test -- --test-threads=4
```

### Run Benchmarks
```bash
# Run all benchmarks
cargo bench

# Run specific benchmark suite
cargo bench --bench combat_benchmarks
cargo bench --bench worldbook_benchmarks
cargo bench --bench ai_benchmarks

# Save baseline for comparison
cargo bench -- --save-baseline my_baseline

# Compare against baseline
cargo bench -- --baseline my_baseline
```

## Test Infrastructure (Phase 4)

### Key Testing Dependencies

```toml
[dev-dependencies]
# Snapshot testing - verify complex outputs
insta = { version = "1.40", features = ["json", "yaml", "redactions"] }

# Property-based testing - verify invariants
proptest = "1.4"

# Serialize tests - prevent race conditions
serial_test = "3.2"

# Fast benchmarking framework
divan = "0.1"

# Temporary files for testing
tempfile = "3.8"
```

### insta - Snapshot Testing

**Purpose:** Verify complex structured outputs (JSON, YAML) without manual assertions.

**Example:**
```rust
use insta::{assert_json_snapshot, assert_yaml_snapshot};

#[test]
fn test_worldbook_serialization() {
    let worldbook = Worldbook::with_defaults();
    assert_json_snapshot!(worldbook);
}
```

**Benefits:**
- Automatic output comparison
- Easy to review changes (human-readable diffs)
- Redaction support for timestamps/UUIDs

**Review snapshots:**
```bash
cargo insta review
```

### proptest - Property-Based Testing

**Purpose:** Test invariants hold for random inputs.

**Example:**
```rust
use proptest::prelude::*;

proptest! {
    #[test]
    fn test_hp_never_negative(damage in 0i32..10000) {
        let mut character = create_test_character("Test");
        character.take_damage(damage);
        prop_assert!(character.current_hp >= 0);
    }
}
```

**Benefits:**
- Finds edge cases automatically
- Tests thousands of inputs per test
- Shrinks failing cases to minimal example

### serial_test - Test Serialization

**Purpose:** Run tests sequentially when they share resources.

**Example:**
```rust
use serial_test::serial;

#[test]
#[serial]
fn test_save_file() {
    // Tests that modify saves/ directory run sequentially
}
```

### divan - Benchmarking

**Purpose:** Fast, accurate performance measurements.

**Example:**
```rust
use divan::Bencher;

#[divan::bench]
fn damage_calculation(bencher: Bencher) {
    bencher.bench_local(|| calculate_damage("2d6+3", 5, false));
}
```

**Benefits:**
- Faster than Criterion (compile times)
- Can measure allocations
- Clean output format

## Code Coverage

### Generate Coverage Report
```bash
# Install tarpaulin
cargo install cargo-tarpaulin

# Generate HTML coverage report
cargo tarpaulin --out Html --output-dir coverage

# Open coverage/index.html in browser
```

### Coverage Goals
- **Core gameplay**: 80% line coverage
- **Game logic**: 70% line coverage
- **Overall project**: 60% line coverage

### CI/CD Integration
A GitHub Actions workflow file (`coverage.yml`) is available locally in `.github/workflows/`
but cannot be pushed via the GitHub App due to workflow permissions. To enable automated
coverage reporting:
1. Manually add the workflow file through GitHub's web interface, or
2. Push it using a git client with full repository access

The workflow will run tests with coverage on each push and upload reports to codecov.io.

## Test Organization

### Test Helpers (`tests/helpers.rs`)
Reusable test utilities:
- `create_test_character()` - Create character with default stats
- `create_custom_character()` - Create character with custom SPECIAL
- `create_test_game_state()` - Create full game state for testing
- `create_healing_item()` - Create consumable items
- `create_test_weapon()` - Create weapons for testing
- `create_weak_enemy()` - Create low-level enemy

### Property-Based Testing
Property tests verify invariants hold for all inputs:
- Dice rolls are always within expected bounds
- Character HP never goes negative
- AP usage respects constraints
- Enemy scaling is monotonic with level
- Story manager respects capacity limits

### Regression Tests
Tests for previously identified bugs:
- Multi-level HP bonus application
- HP cannot go negative
- AP usage validation
- Combat state reset
- XP reward calculation

## Test Coverage Areas

### High Coverage (>80%)
- ✅ Character creation and stats
- ✅ Experience and leveling
- ✅ Combat dice mechanics
- ✅ Item system
- ✅ Configuration loading
- ✅ Story manager

### Medium Coverage (60-80%)
- ✅ Worldbook operations
- ✅ Game persistence
- ✅ Combat encounters

### Low Coverage (<60%)
- ⚠️ TUI components (harder to test)
- ⚠️ AI integration (requires mocking)
- ⚠️ Main game loop

## Writing New Tests

### Unit Test Example
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_character_takes_damage() {
        let mut character = create_test_character("Test");
        let initial_hp = character.current_hp;

        character.take_damage(10);

        assert_eq!(character.current_hp, initial_hp - 10);
    }
}
```

### Integration Test Example
```rust
// tests/my_feature_tests.rs
mod helpers;
use helpers::*;

#[test]
fn test_my_feature() {
    let character = create_test_character("Hero");
    // Test logic here
    assert!(character.is_alive());
}
```

### Property Test Example
```rust
use proptest::prelude::*;

proptest! {
    #[test]
    fn test_hp_never_negative(damage in 0i32..10000) {
        let mut character = create_test_character("Test");
        character.take_damage(damage);
        prop_assert!(character.current_hp >= 0);
    }
}
```

## Continuous Integration

Tests run automatically on:
- Every push to main/master branches
- All pull requests
- See `.github/workflows/rust.yml` and `.github/workflows/coverage.yml`

## Test Organization by Phase

### Phase 4: Core Infrastructure
- Added `insta`, `proptest`, `serial_test`, `tempfile`
- Established test helpers in `tests/helpers.rs`
- Created property-based tests for invariants
- Set up snapshot testing for complex outputs

### Phase 5: Narrative Module Testing (40 tests)
- `stat_allocator_tests.rs` - SPECIAL point allocation UI
- `handlers_tests.rs` - Command processing and game state updates
- Enhanced combat and worldbook test coverage
- Added AI mock helpers for testing without llama.cpp

### Phase 6: Benchmark Suite
- Created `benches/` directory with divan framework
- Combat benchmarks: damage, rolls, stat modifiers
- Worldbook benchmarks: lookups, serialization
- AI benchmarks: template rendering, string allocations
- Identified and optimized hot path (`resolve_stat_modifiers`)

### Phase 7: Integration Tests
- End-to-end gameplay flows
- Save/load round-trip testing
- Multi-system integration (AI + game + TUI)

### Phase 8: Regression & Edge Cases
- Regression tests for fixed bugs
- Error path testing
- Validation integration tests (Garde framework)

## Known Issues

- TUI animation test (`test_animation_progress`) has timing issues
- AI integration tests require running llama.cpp server
- Some file I/O tests may fail on read-only filesystems
- Windows path separators may cause issues on Unix systems

## Best Practices

1. **Test behavior, not implementation** - Test what code does, not how
2. **Use descriptive test names** - `test_character_levels_up_at_1000_xp` not `test_1`
3. **One assertion per test** - Keep tests focused on single behavior
4. **Use test helpers** - DRY principle applies to tests too
5. **Test edge cases** - Zero values, negative numbers, empty collections
6. **Add regression tests** - When fixing bugs, add test to prevent recurrence

## Testing Metrics

### Current Test Count
- **Integration tests**: 150+ tests across 15 files
- **Unit tests**: 50+ inline tests
- **Property tests**: 10+ invariant tests
- **Benchmarks**: 20+ performance benchmarks

### Coverage Targets
- Core gameplay: 80%+ line coverage
- Game logic: 70%+ line coverage
- Overall project: 60%+ line coverage

### Test Execution Time
- All tests: ~5-10 seconds (without AI tests)
- Benchmarks: ~30 seconds (full suite)
- Property tests: ~1-2 seconds per test (1000+ cases each)

## Additional Resources

- [Rust Testing Guide](https://doc.rust-lang.org/book/ch11-00-testing.html)
- [Proptest Documentation](https://proptest-rs.github.io/proptest/)
- [Tarpaulin Documentation](https://github.com/xd009642/tarpaulin)
- [insta Snapshot Testing](https://insta.rs/)
- [divan Benchmarking](https://github.com/nvzqz/divan)
- [PERFORMANCE.md](PERFORMANCE.md) - Detailed benchmark results
- [ARCHITECTURE.md](ARCHITECTURE.md) - System design overview
