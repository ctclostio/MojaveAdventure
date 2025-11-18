# Testing Guide

This document describes the testing strategy and how to run tests for the Fallout D&D game.

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
- `character_tests.rs` - Character leveling and XP system (19 tests)
- `item_tests.rs` - Items, weapons, armor, consumables (19 tests)
- `config_tests.rs` - Configuration loading and validation (16 tests)
- `combat_tests.rs` - Combat mechanics and enemy scaling (28 tests)
- `worldbook_tests.rs` - Worldbook locations, NPCs, events (27 tests)
- `persistence_tests.rs` - Save/load functionality (3 tests)
- `story_manager_tests.rs` - Story context management (29 tests)
- `integration_tests.rs` - End-to-end gameplay flows (3 tests)
- `error_path_tests.rs` - Error handling and edge cases (8 tests)

### Property-Based Tests
Using `proptest` for invariant testing:
- `property_tests.rs` - Property-based tests for game invariants

### Regression Tests
Tests for previously fixed bugs:
- `regression_tests.rs` - Tests preventing bug regressions

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

## Known Issues

- TUI animation test (`test_animation_progress`) has timing issues
- AI integration tests require running llama.cpp server
- Some file I/O tests may fail on read-only filesystems

## Best Practices

1. **Test behavior, not implementation** - Test what code does, not how
2. **Use descriptive test names** - `test_character_levels_up_at_1000_xp` not `test_1`
3. **One assertion per test** - Keep tests focused on single behavior
4. **Use test helpers** - DRY principle applies to tests too
5. **Test edge cases** - Zero values, negative numbers, empty collections
6. **Add regression tests** - When fixing bugs, add test to prevent recurrence

## Additional Resources

- [Rust Testing Guide](https://doc.rust-lang.org/book/ch11-00-testing.html)
- [Proptest Documentation](https://proptest-rs.github.io/proptest/)
- [Tarpaulin Documentation](https://github.com/xd009642/tarpaulin)
