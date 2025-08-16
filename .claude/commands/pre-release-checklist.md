# リリース前チェックリスト

## 自動チェック項目（scripts/test-release.sh で実行）

- [ ] `cargo build --release` が成功
- [ ] `cargo test` が全パス
- [ ] `cargo clippy -- -D warnings` でwarningなし
- [ ] `cargo fmt --check` でフォーマット確認
- [ ] `cargo publish --dry-run` が成功

## 手動確認項目

### 1. バージョン確認
- [ ] Cargo.toml のバージョン更新
- [ ] src/main.rs の VERSION 定数更新
- [ ] CHANGELOG.md 更新

### 2. ドキュメント確認
- [ ] README.md が最新
- [ ] 新機能のドキュメント追加

### 3. Git状態確認
- [ ] 作業ディレクトリがクリーン
- [ ] mainブランチが最新
- [ ] 不要なファイルがコミットされていない

### 4. テスト実行
- [ ] 基本機能テスト: `./target/release/tree2md`
- [ ] オプションテスト: `./target/release/tree2md -c -e .rs`
- [ ] バージョン表示: `./target/release/tree2md --version`

### 5. プラットフォーム別ビルド確認（可能な範囲で）
- [ ] Linux x64
- [ ] macOS（現在のプラットフォーム）
- [ ] Windows（WSL or CI経由）

## リリース手順

1. **ローカルテスト実行**
   ```bash
   ./scripts/test-release.sh
   ```

2. **変更をコミット・プッシュ**
   ```bash
   git add .
   git commit -m "Release v0.x.x"
   git push origin main
   ```

3. **タグ作成・プッシュ**
   ```bash
   git tag -a v0.x.x -m "Release v0.x.x: 説明"
   git push origin v0.x.x
   ```

4. **GitHub Actions確認**
   ```bash
   gh run list --workflow=release.yml
   gh run watch <RUN_ID>
   ```

5. **リリース確認**
   ```bash
   gh release view v0.x.x
   ```

6. **crates.io公開確認**
   - 自動公開されるはず
   - 失敗時: `gh workflow run publish-crate.yml`