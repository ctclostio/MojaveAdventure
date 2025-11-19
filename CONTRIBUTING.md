# Contributing to FalloutDnD

Thank you for your interest in contributing to FalloutDnD! This document provides guidelines and instructions for contributing to the project.

## Development Setup

1. **Clone the repository**
   ```bash
   git clone https://github.com/ctclostio/MojaveAdventure.git
   cd MojaveAdventure
   ```

2. **Install git hooks**

   For Linux/macOS:
   ```bash
   ./scripts/install-hooks.sh
   ```

   For Windows (PowerShell):
   ```powershell
   .\scripts\install-hooks.ps1
   ```

   This installs a pre-commit hook that automatically checks code formatting.

3. **Verify your setup**
   ```bash
   cargo build
   cargo test
   cargo fmt --all -- --check
   ```

## Code Style

We use `rustfmt` to maintain consistent code formatting across the project.

### Before Committing

**Important**: Always run `cargo fmt --all` before committing your changes.

If you've installed the git hooks (step 2 above), this will be checked automatically. If formatting issues are found, the commit will be blocked with a helpful error message.

### Manual Formatting Check

```bash
# Check if code is formatted correctly
cargo fmt --all -- --check

# Automatically format all code
cargo fmt --all
```

## Pull Request Process

1. **Create a feature branch**
   ```bash
   git checkout -b feature/your-feature-name
   ```

2. **Make your changes**
   - Write clear, concise commit messages
   - Add tests for new functionality
   - Update documentation as needed

3. **Ensure quality checks pass**
   ```bash
   cargo build         # Must compile without errors
   cargo test          # All tests must pass
   cargo fmt --all     # Code must be formatted
   cargo clippy        # Should have no warnings
   ```

4. **Push and create a pull request**
   ```bash
   git push origin feature/your-feature-name
   ```

   Then create a PR on GitHub with a clear description of your changes.

## CI/CD

All pull requests must pass the following checks:

- ✅ **Formatting**: `cargo fmt --all -- --check`
- ✅ **Linting**: `cargo clippy --all-targets --all-features -- -D warnings`
- ✅ **Build**: `cargo build --verbose --all-features`
- ✅ **Tests**: `cargo test --verbose --all-features`

## Commit Message Guidelines

We follow conventional commit format:

```
<type>: <description>

[optional body]

[optional footer]
```

### Types
- `feat`: New feature
- `fix`: Bug fix
- `improve`: Code improvement/refactoring
- `docs`: Documentation changes
- `test`: Adding or updating tests
- `chore`: Maintenance tasks

### Examples

```
feat: add combat encounter system

Implement random combat encounters with difficulty scaling
based on player level.
```

```
fix: resolve dice rolling edge case

Fixed issue where negative modifiers could result in
invalid dice rolls.
```

## Testing

- Write unit tests for new functionality
- Ensure all existing tests pass
- Add integration tests for complex features
- Test edge cases and error conditions

## Questions?

If you have questions about contributing, please:
- Open an issue on GitHub
- Check existing issues and pull requests
- Review the codebase for examples

## License

By contributing to FalloutDnD, you agree that your contributions will be licensed under the same license as the project.
