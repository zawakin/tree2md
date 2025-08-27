#!/usr/bin/env bash
set -euo pipefail

# DEPRECATED: This script is being phased out in favor of the Claude command /release
# which provides better release note generation and a more comprehensive process.
# 
# Usage: ./scripts/release.sh vX.Y.Z
# Non-interactive. Performs validation -> tag -> push.
# Prerequisites:
#  - Version already updated in Cargo.toml / src/main.rs / CHANGELOG.md
#  - gh not required (not used)
#  - main branch workflow

RED='\033[0;31m'; GREEN='\033[0;32m'; YELLOW='\033[0;33m'; NC='\033[0m'

die(){ echo -e "${RED}ERROR:${NC} $*" >&2; exit 1; }
ok(){  echo -e "${GREEN}✓${NC} $*"; }
warn(){ echo -e "${YELLOW}⚠${NC} $*"; }

TAG="${1-}"; [[ -n "$TAG" ]] || die "tag (e.g. v1.2.3) is required"
[[ "$TAG" =~ ^v[0-9]+\.[0-9]+\.[0-9]+$ ]] || die "tag must be vX.Y.Z"

# 1) Check git status
git rev-parse --is-inside-work-tree >/dev/null 2>&1 || die "not a git repo"
git fetch origin main --quiet
[[ "$(git branch --show-current)" == "main" ]] || warn "not on main branch"
git diff --quiet || die "working tree has changes"
git diff --cached --quiet || die "index has staged changes"
git diff --quiet HEAD origin/main || warn "local main differs from origin/main"

# 2) Version consistency check
CARGO_VERSION=$(grep '^version *= *"' Cargo.toml | sed 's/.*"\(.*\)".*/\1/')
MAIN_VERSION=$(grep 'const VERSION' src/main.rs | sed 's/.*"\(.*\)".*/\1/' || true)
[[ "v$CARGO_VERSION" == "$TAG" ]] || die "Cargo.toml version ($CARGO_VERSION) != tag ($TAG)"
[[ -z "$MAIN_VERSION" || "$MAIN_VERSION" == "$CARGO_VERSION" ]] || die "src/main.rs VERSION ($MAIN_VERSION) != Cargo.toml ($CARGO_VERSION)"
ok "Version consistency: $TAG"

# 3) Local quality gates
echo "Running fmt/clippy/tests..."
cargo fmt -- --check
cargo clippy -- -D warnings
cargo test
ok "Local checks passed"

# 4) Dry run (publish)
echo "cargo publish --dry-run..."
cargo publish --dry-run >/dev/null
ok "Publish dry-run passed"

# 5) Create tag & push
git tag -a "$TAG" -m "Release $TAG"
git push origin main
git push origin "$TAG"
ok "Pushed tag $TAG"

echo ""
ok "Release initiated. CI will build, attach assets, and publish to crates.io."