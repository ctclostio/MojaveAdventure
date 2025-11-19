#!/bin/bash
# Install git hooks for this repository
# Run this script after cloning: ./scripts/install-hooks.sh

HOOKS_DIR=".git/hooks"
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

echo "Installing git hooks..."

# Pre-commit hook for formatting checks
cat > "$HOOKS_DIR/pre-commit" << 'EOF'
#!/bin/sh
# Pre-commit hook to ensure code is formatted before committing

echo "Running cargo fmt check..."

if ! cargo fmt --all -- --check >/dev/null 2>&1; then
    echo "❌ Code formatting check failed!"
    echo ""
    echo "Your code is not formatted according to rustfmt standards."
    echo "Please run: cargo fmt --all"
    echo ""
    echo "Or to automatically format and continue:"
    echo "  cargo fmt --all && git add -u && git commit"
    exit 1
fi

echo "✅ Formatting check passed"
exit 0
EOF

chmod +x "$HOOKS_DIR/pre-commit"

echo "✅ Pre-commit hook installed successfully"
echo ""
echo "The hook will run 'cargo fmt --all -- --check' before each commit."
echo "To bypass the hook temporarily, use: git commit --no-verify"
