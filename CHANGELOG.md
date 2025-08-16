# Changelog

## [Unreleased] - Rust Version

### Added
- Rust版として完全書き直し
- クロスプラットフォームバイナリ配布
- GitHub Actions CI/CD
- crates.io サポート

### Changed
- Go実装からRust実装へ移行
- より高速な実行
- より堅牢な.gitignore処理（ignoreクレート使用）

### Maintained
- すべてのコマンドラインオプション互換性維持
- 同一の出力フォーマット

## [0.1.6] - 2025-01-22 (Go Version)

### Added
- `--respect-gitignore` フラグ追加
- .gitignore パターンに基づくファイル除外機能
- ディレクトリ、ワイルドカード、否定パターンのサポート

## [0.1.5] - 2025-01-15 (Go Version)

### Added
- `--max-lines` オプション追加
- ファイル内容の行数制限機能
- truncation情報の詳細表示

### Fixed
- バージョン文字列の更新

## [0.1.4] - 2024-12-31 (Go Version)

### Added
- バージョン情報表示機能（`-v`, `--version`）

## [0.1.3] - 2024-12-31 (Go Version)

### Added
- HTML言語サポート

## [0.1.2] - 2024-12-08 (Go Version)

### Changed
- デフォルトモードを変更

## [0.1.1] - 2024-12-08 (Go Version)

### Added
- MITライセンス追加

## [0.1.0] - 2024-12-08 (Go Version)

### Initial Release
- ディレクトリ構造のMarkdown出力
- コードブロック表示機能
- 拡張子フィルタリング
- 隠しファイル対応
- 多言語サポート（英語/日本語）