# Release Process

## 0. Update Version

- version in Cargo.toml
- VERSION constant in src/main.rs
- CHANGELOG.md

## 1. Local Validation & Tag & Push
```bash
./scripts/release.sh vX.Y.Z
```

## 2. Verify Results (Automatic)
- GitHub Actions: Release workflow starts
- Binaries for each OS attached to Release
- Published to crates.io
- Check GitHub Release and crates.io when successful
