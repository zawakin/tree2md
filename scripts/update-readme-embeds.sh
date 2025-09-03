#!/usr/bin/env bash

# Change to project root
cd "$(dirname "$0")/.."

# Build the project
cargo build --release

# Check for --dry-run flag
DRY_RUN=""
if [[ "$1" == "--dry-run" ]]; then
    DRY_RUN="--dry-run"
fi

# Update markdown-style embed
./target/release/tree2md . \
    -L 2 \
    --output md \
    --tag-start "<!-- tree2md-md:start -->" \
    --tag-end "<!-- tree2md-md:end -->" \
    --inject README.md \
    $DRY_RUN

# Update tree-style embed  
./target/release/tree2md . \
    -L 2 \
    --output tty \
    --tag-start "<!-- tree2md-tree:start -->" \
    --tag-end "<!-- tree2md-tree:end -->" \
    --inject README.md \
    $DRY_RUN