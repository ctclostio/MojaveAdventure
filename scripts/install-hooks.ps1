# Install git hooks for this repository (Windows PowerShell version)
# Run this script after cloning: .\scripts\install-hooks.ps1

$hooksDir = ".git\hooks"
$preCommitPath = Join-Path $hooksDir "pre-commit"

Write-Host "Installing git hooks..." -ForegroundColor Cyan

# Pre-commit hook for formatting checks
$hookContent = @'
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
'@

Set-Content -Path $preCommitPath -Value $hookContent -NoNewline

Write-Host "✅ Pre-commit hook installed successfully" -ForegroundColor Green
Write-Host ""
Write-Host "The hook will run 'cargo fmt --all -- --check' before each commit."
Write-Host "To bypass the hook temporarily, use: git commit --no-verify"
