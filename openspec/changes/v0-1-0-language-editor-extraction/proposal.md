## Why

KatanA v0.27.0 で `katana-ui` から editor 実装を切り出す。切り出し先をこの repo とし、KatanA は git dependency として consume するだけにする。KatanA の検証範囲から editor の実装詳細を除く。

**`katana-language-editor` は言語非依存の汎用テキストエディタ widget。** KatanA は Markdown に特化した `SyntaxHighlighter` 実装を注入して使うが、`katana-language-editor` 自体はどの言語かを知らない。将来 Markdown 以外の言語にも同じ widget を流用できる。

将来の独自 UI フレームワーク導入時（egui 脱却）も、`katana-language-editor-egui` を差し替えるだけで KatanA 側は無影響にする。

## What Changes

- `katana-language-editor`（neutral interface）に以下を定義する：
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

egui（epaint）の独自フォントアトラスは OS フォントフォールバックチェーンを無視するため、カラー絵文字（Apple Color Emoji 等）をエディタ上で正しく表示できない。IME composition も不完全。これらの根本解決は独自 UI フレームワーク導入時に `katana-language-editor-egui` を差し替えることで対応する。

## Impact

- `crates/katana-language-editor/` — neutral interface crate（egui 非依存）
- `crates/katana-language-editor-egui/` — egui 実装 crate
