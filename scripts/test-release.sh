#!/bin/bash

# Pre-release test script for tree2md
# This script performs comprehensive tests before releasing

set -e

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[0;33m'
NC='\033[0m' # No Color

echo "🔍 Starting pre-release tests for tree2md..."
echo ""

# Function to print colored output
print_status() {
    if [ $1 -eq 0 ]; then
        echo -e "${GREEN}✓${NC} $2"
    else
        echo -e "${RED}✗${NC} $2"
        exit 1
    fi
}

print_warning() {
    echo -e "${YELLOW}⚠${NC} $1"
}

# 1. Check if we're in the right directory
if [ ! -f "Cargo.toml" ]; then
    echo -e "${RED}Error: Cargo.toml not found. Run this script from the project root.${NC}"
    exit 1
fi

echo "📋 Running automated checks..."
echo ""

# 2. Clean build
echo "Building release version..."
cargo clean
cargo build --release --quiet
print_status $? "Release build successful"

# 3. Run tests
echo "Running tests..."
cargo test --quiet
print_status $? "All tests passed"

# 4. Check formatting
echo "Checking code formatting..."
cargo fmt -- --check
print_status $? "Code formatting is correct"

# 5. Run clippy
echo "Running clippy..."
cargo clippy -- -D warnings 2>/dev/null
print_status $? "No clippy warnings"

# 6. Check if it can be published
echo "Testing crates.io publish (dry run)..."
cargo publish --dry-run 2>&1 | grep -q "warning: aborting upload due to dry run"
print_status $? "Package ready for crates.io"

echo ""
echo "🧪 Running functional tests..."
echo ""

# 7. Test basic functionality
echo "Testing basic tree output..."
./target/release/tree2md sample > /dev/null 2>&1
print_status $? "Basic tree generation works"

# 8. Test with content flag
echo "Testing content output..."
./target/release/tree2md sample -c > /dev/null 2>&1
print_status $? "Content output works"

# 9. Test extension filter
echo "Testing extension filter..."
./target/release/tree2md sample -e .py,.go > /dev/null 2>&1
print_status $? "Extension filter works"

# 10. Test version flag
echo "Testing version output..."
VERSION_OUTPUT=$(./target/release/tree2md --version)
if [[ $VERSION_OUTPUT == *"tree2md"* ]]; then
    print_status 0 "Version output works: $VERSION_OUTPUT"
else
    print_status 1 "Version output failed"
fi

echo ""
echo "📦 Checking version consistency..."
echo ""

# 11. Check version consistency
CARGO_VERSION=$(grep "^version" Cargo.toml | sed 's/.*"\(.*\)".*/\1/')
RUST_VERSION=$(grep "const VERSION" src/main.rs | sed 's/.*"\(.*\)".*/\1/')

if [ "$CARGO_VERSION" = "$RUST_VERSION" ]; then
    print_status 0 "Version consistency: v$CARGO_VERSION"
else
    echo -e "${RED}✗${NC} Version mismatch!"
    echo "  Cargo.toml: $CARGO_VERSION"
    echo "  src/main.rs: $RUST_VERSION"
    exit 1
fi

# 12. Check git status
echo ""
echo "📋 Checking git status..."
echo ""

# Check if working directory is clean (allowing untracked files)
if git diff --quiet && git diff --cached --quiet; then
    print_status 0 "No uncommitted changes"
else
    print_warning "There are uncommitted changes"
    echo "  Run: git status"
fi

# Check if we're on main branch
CURRENT_BRANCH=$(git branch --show-current)
if [ "$CURRENT_BRANCH" = "main" ]; then
    print_status 0 "On main branch"
else
    print_warning "Not on main branch (current: $CURRENT_BRANCH)"
fi

# Check if main is up to date with origin
git fetch origin main --quiet
if git diff --quiet HEAD origin/main; then
    print_status 0 "Main branch is up to date"
else
    print_warning "Main branch differs from origin/main"
    echo "  Run: git pull origin main"
fi

echo ""
echo "🎉 All automated tests passed!"
echo ""
echo "📝 Next steps:"
echo "1. Review CHANGELOG.md"
echo "2. Update version if needed"
echo "3. Commit changes: git commit -m 'Release vX.X.X'"
echo "4. Create tag: git tag -a vX.X.X -m 'Release vX.X.X: description'"
echo "5. Push: git push origin main && git push origin vX.X.X"
echo ""
echo "For full checklist, see: .claude/commands/pre-release-checklist.md"