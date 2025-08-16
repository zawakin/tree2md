# リリース手順

## 1. CHANGELOG.md 更新
```bash
# CHANGELOG.md を作成/更新
```

## 2. バージョン更新
```bash
# Cargo.toml の version を更新
# src/main.rs の VERSION 定数を更新
```

## 3. ビルドテスト
```bash
cargo build --release
cargo test
```

## 4. コミット
```bash
git add .
git commit -m "Release v0.1.x"
```

## 5. タグ作成とプッシュ
```bash
git tag -a v0.1.x -m "Release v0.1.x: 機能説明"
git push origin main
git push origin v0.1.x
```

## 6. GitHub Actions 確認
- リリースワークフロー実行確認
- バイナリ生成確認

## 7. crates.io 公開 (optional)
```bash
cargo publish
```