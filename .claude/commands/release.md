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
git commit -m "Release v0.x.x"
```

## 5. タグ作成とプッシュ
```bash
git tag -a v0.x.x -m "Release v0.x.x: 機能説明"
git push origin main
git push origin v0.x.x
```

## 6. GitHub Actions 確認
- リリースワークフロー自動実行確認
- バイナリ生成確認
- GitHub Release ページ確認

## 7. crates.io 公開
### 7.1. Dry Run (推奨)
- GitHub Actions → publish-crate.yml → Run workflow
- `dry_run`: true (デフォルト)で実行
- 成功確認

### 7.2. 実際の公開
- GitHub Actions → publish-crate.yml → Run workflow  
- `dry_run`: false に変更して実行
- crates.io で公開確認

注: release.yml は GitHub Release のみ作成。crates.io への公開は別途手動実行。