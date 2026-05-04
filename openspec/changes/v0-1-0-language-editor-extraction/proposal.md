## Why

editor 実装を独立した crate として確立する。`katana-language-editor` は言語非依存の汎用テキストエディタ widget として設計する。KatanA は Markdown に特化した `SyntaxHighlighter` 実装を注入して使うが、`katana-language-editor` 自体はどの言語かを知らない。

## What Changes

- `katana-language-editor`（neutral interface、egui 非依存）に以下を定義する：
  - `SyntaxHighlighter` trait（言語非依存のハイライト契約）
  - `EditorConfig { syntax_highlighter: Box<dyn SyntaxHighlighter>, font_size, theme, ... }`
  - `EditorWidget` trait（UI フレームワーク非依存のエディタ widget 契約）
  - `HighlightedText` DTO
- `katana-language-editor-egui` に以下を実装する：
  - egui TextEdit ラップ、行番号、シンタックスハイライト描画
  - 絵文字フォント workaround（egui 制約の注記付き）
  - `EditorWidget` の egui 実装
- KatanA は `MarkdownSyntaxHighlighter` を実装して `EditorConfig` に注入するだけ
- `v0.1.0` として release tag を切る

## Capabilities

### New Capabilities

- `language-editor-component`: 言語非依存 neutral interface + egui MVP 実装
- `syntax-highlighter-injection`: 外部から言語固有の highlighter を注入できる設計

## Known Constraints（egui MVP 段階）

egui（epaint）の独自フォントアトラスは OS フォントフォールバックチェーンを無視するため、カラー絵文字をエディタ上で正しく表示できない。IME composition も不完全。根本解決は `katana-language-editor-floem` 実装時。

## Impact

- `crates/katana-language-editor/` — neutral interface crate（egui 非依存）
- `crates/katana-language-editor-egui/` — egui 実装 crate
