#!/bin/bash

# Semi-automated release script for tree2md
# Usage: ./scripts/release.sh [patch|minor|major]

set -e

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[0;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Check arguments
if [ $# -ne 1 ]; then
    echo "Usage: $0 [patch|minor|major]"
    echo "  patch: 0.1.0 -> 0.1.1"
    echo "  minor: 0.1.0 -> 0.2.0"
    echo "  major: 0.1.0 -> 1.0.0"
    exit 1
fi

BUMP_TYPE=$1

if [[ ! "$BUMP_TYPE" =~ ^(patch|minor|major)$ ]]; then
    echo -e "${RED}Error: Invalid bump type. Use patch, minor, or major.${NC}"
    exit 1
fi

echo -e "${BLUE}🚀 Starting release process...${NC}"
echo ""

# 1. Run pre-release tests
echo "Running pre-release tests..."
if ! ./scripts/test-release.sh; then
    echo -e "${RED}Pre-release tests failed. Fix issues before releasing.${NC}"
    exit 1
fi

echo ""
echo -e "${GREEN}✓ All tests passed!${NC}"
echo ""

# 2. Get current version
CURRENT_VERSION=$(grep "^version" Cargo.toml | sed 's/.*"\(.*\)".*/\1/')
echo "Current version: v$CURRENT_VERSION"

# 3. Calculate new version
IFS='.' read -ra VERSION_PARTS <<< "$CURRENT_VERSION"
MAJOR=${VERSION_PARTS[0]}
MINOR=${VERSION_PARTS[1]}
PATCH=${VERSION_PARTS[2]}

case $BUMP_TYPE in
    patch)
        PATCH=$((PATCH + 1))
        ;;
    minor)
        MINOR=$((MINOR + 1))
        PATCH=0
        ;;
    major)
        MAJOR=$((MAJOR + 1))
        MINOR=0
        PATCH=0
        ;;
esac

NEW_VERSION="$MAJOR.$MINOR.$PATCH"
echo "New version: v$NEW_VERSION"
echo ""

# 4. Confirm with user
echo -e "${YELLOW}This will:${NC}"
echo "  1. Update version in Cargo.toml and src/main.rs"
echo "  2. Update CHANGELOG.md"
echo "  3. Commit changes"
echo "  4. Create and push tag v$NEW_VERSION"
echo "  5. Trigger GitHub Actions release workflow"
echo ""
read -p "Continue? (y/N): " -n 1 -r
echo ""

if [[ ! $REPLY =~ ^[Yy]$ ]]; then
    echo "Release cancelled."
    exit 0
fi

# 5. Update version in files
echo "Updating version in Cargo.toml..."
sed -i.bak "s/^version = \".*\"/version = \"$NEW_VERSION\"/" Cargo.toml
rm Cargo.toml.bak

echo "Updating version in src/main.rs..."
sed -i.bak "s/const VERSION: &str = \".*\"/const VERSION: \&str = \"$NEW_VERSION\"/" src/main.rs
rm src/main.rs.bak

# 6. Update Cargo.lock
echo "Updating Cargo.lock..."
cargo update --package tree2md --quiet

# 7. Update CHANGELOG.md
echo "Updating CHANGELOG.md..."
TODAY=$(date +%Y-%m-%d)
sed -i.bak "s/## \[Unreleased\]/## [$NEW_VERSION] - $TODAY\n\n## [Unreleased]/" CHANGELOG.md 2>/dev/null || {
    echo -e "${YELLOW}Note: Please update CHANGELOG.md manually${NC}"
}
rm -f CHANGELOG.md.bak

# 8. Commit changes
echo "Committing changes..."
git add Cargo.toml src/main.rs Cargo.lock CHANGELOG.md
git commit -m "Release v$NEW_VERSION"

# 9. Create and push tag
echo "Creating tag..."
read -p "Enter release description (one line): " RELEASE_DESC
git tag -a "v$NEW_VERSION" -m "Release v$NEW_VERSION: $RELEASE_DESC"

# 10. Push to remote
echo "Pushing to remote..."
git push origin main
git push origin "v$NEW_VERSION"

echo ""
echo -e "${GREEN}🎉 Release v$NEW_VERSION initiated!${NC}"
echo ""
echo "Next steps:"
echo "1. Monitor GitHub Actions: gh run list --workflow=release.yml"
echo "2. Check release: gh release view v$NEW_VERSION"
echo "3. Verify crates.io: https://crates.io/crates/tree2md"
echo ""
echo "If any issues occur:"
echo "- Check workflow: gh run view <RUN_ID> --log-failed"
echo "- Manual crates.io publish: gh workflow run publish-crate.yml"