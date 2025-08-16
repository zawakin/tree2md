#!/bin/bash

# Build script for creating cross-platform binaries
# Requires cross-compilation toolchains to be installed

set -e

VERSION=$(grep '^version' Cargo.toml | sed 's/.*"\(.*\)".*/\1/')
BINARY_NAME="tree2md"

echo "Building tree2md v${VERSION}..."

# Create dist directory
mkdir -p dist

# Build targets
TARGETS=(
    "x86_64-unknown-linux-gnu"
    "x86_64-unknown-linux-musl"
    "aarch64-unknown-linux-gnu"
    "x86_64-apple-darwin"
    "aarch64-apple-darwin"
    "x86_64-pc-windows-msvc"
)

for TARGET in "${TARGETS[@]}"; do
    echo "Building for ${TARGET}..."
    
    # Check if target is installed
    if rustup target list | grep -q "${TARGET} (installed)"; then
        cargo build --release --target "${TARGET}"
        
        # Determine file extension
        if [[ "${TARGET}" == *"windows"* ]]; then
            EXT=".exe"
        else
            EXT=""
        fi
        
        # Copy binary to dist with target name
        if [ -f "target/${TARGET}/release/${BINARY_NAME}${EXT}" ]; then
            cp "target/${TARGET}/release/${BINARY_NAME}${EXT}" "dist/${BINARY_NAME}-${TARGET}${EXT}"
            echo "  Created: dist/${BINARY_NAME}-${TARGET}${EXT}"
        fi
    else
        echo "  Skipping ${TARGET} - target not installed"
        echo "  Install with: rustup target add ${TARGET}"
    fi
done

echo ""
echo "Build complete! Binaries are in the dist/ directory"
echo ""
echo "To create archives for release:"
echo "  cd dist"
echo "  tar czf tree2md-linux-x64.tar.gz tree2md-x86_64-unknown-linux-gnu"
echo "  tar czf tree2md-linux-arm64.tar.gz tree2md-aarch64-unknown-linux-gnu"
echo "  tar czf tree2md-macos-x64.tar.gz tree2md-x86_64-apple-darwin"
echo "  tar czf tree2md-macos-arm64.tar.gz tree2md-aarch64-apple-darwin"
echo "  zip tree2md-windows-x64.zip tree2md-x86_64-pc-windows-msvc.exe"