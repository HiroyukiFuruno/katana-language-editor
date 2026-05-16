## Why

editor 実装を独立した crate として確立する。`katana-language-editor` は言語非依存の汎用テキストエディタ widget として設計する。KatanA は Markdown に特化した `SyntaxHighlighter` 実装を注入して使うが、`katana-language-editor` 自体はどの言語かを知らない。

加えて、v0.1.0 段階で **neutral interface 全体の DI 契約**（テーマ・i18n・タイポグラフィ・host 制御 API・設定・診断/装飾）を確定する。`v0.2.x` で Floem 実装へ差し替える際に neutral 層を二度書きしないため、egui MVP 段階から契約を固める。

## What Changes

### Neutral interface（v0.1.0 で確定）

- `katana-language-editor`（neutral interface、egui 非依存）に以下を定義する：
  - `SyntaxHighlighter` trait（言語非依存のハイライト契約）
  - `EditorConfig`（後述の theme / i18n / typography / autosave / shortcut / syntax_highlighter を **non-nullable** で集約）
  - `EditorWidget` / `LanguageEditor` trait（UI フレームワーク非依存のエディタ widget 契約）
  - `HighlightedText` / `EditorEvent` / `EditorOutput` / `CursorPosition` / `Selection` DTO
- **テーマ DI**（`Theme` / `ColorTokens`）
  - 全ての色表現は `Theme` を経由する。`Option<Theme>` を取らない。**preset は host (KDV) が必ず渡す**。
  - editor 実装でハードコードされた色（`Color::rgb`, `#fff` 等のリテラル）が現れないことを `kle-linter` の新ルール `prohibited-color-literal` で機械検証する。
- **i18n DI**（`Locale` / `Strings`）
  - editor 内で表示される全テキスト（メニュー、エラー、ガイド、アクセシビリティラベル等）は `Strings` を経由する。`Option<Strings>` を取らない。**preset (en) は host (KDV) が必ず渡す**。
- **タイポグラフィ / スペーシング DI**（`Typography` / `Spacing`）
  - font family / size / line-height / letter-spacing / radius / padding 等の token も `Option` を取らず、host preset を必須で受け取る。
- **Host 制御 API（公開）**
  - `EditorScrollControl`（外部から行/オフセット/位置までスクロール、可視領域取得）
  - `EditorWriteAccess`（外部からテキスト挿入・置換・全置換、batched edit、編集元タグ付け）
  - `EditorViewControl`（read-only 切替、focus/blur）
  - `EditorHistoryControl`（undo / redo / history barrier）
  - `EditorSelectionControl`（cursor / selection の programmatic 制御）
  - `EditorSearchControl`（find / replace / replace_all、case-sensitive・regex フラグ）
  - `EditorClipboardControl`（cut / copy / paste、host 提供のクリップボード backend 注入）
  - `EditorDiagnosticsSink`（host から LSP 風 diagnostics を流す）
  - `EditorDecorationsSink`（inline decoration / gutter marker / hover hint を host が注入）
  - `EditorAccessibility`（screen reader semantics / direction LTR・RTL）
- **設定 IF（`EditorSettings`）**
  - 自動保存 on/off、自動保存インターバル、ショートカット上書き、word_wrap、tab_size、line_numbers、font_size などを集約。
  - editor 実装は `EditorSettings` の更新通知を受け取り、内部状態を再構成する。
- **egui MVP 実装（`katana-language-editor-egui`）**
  - egui TextEdit ラップ、行番号、シンタックスハイライト描画
  - 上記 neutral 契約の egui バインディングを提供（egui 側で扱えない機能は `EditorError::Unsupported` を返し、Floem 移行で解消）
  - 絵文字フォント workaround（egui 制約の注記付き）
- **Floem 移行準備**
  - `katana-language-editor-floem` skeleton crate を workspace に追加し、依存は `floem = { git = "https://github.com/lapce/floem", rev = "<pinned>" }` で**必ず git 固定**にする（crates.io の `0.2.0` は古く、theme/styling/editor view API が main と乖離しているため使用しない）。
  - skeleton crate は v0.1.0 段階では実装空でよいが、neutral 契約がコンパイル可能であることを CI で保証する。
- KatanA は `MarkdownSyntaxHighlighter` を実装して `EditorConfig` に注入するだけ。
- `v0.1.0` として release tag を切る。

## Capabilities

### New Capabilities

- `language-editor-component`: 言語非依存 neutral interface + egui MVP 実装 + Floem skeleton + 共通 DTO
- `syntax-highlighter-injection`: 外部から言語固有の highlighter を注入できる設計
- `language-editor-theming`: 全色を host preset 経由で受け取り、ハードコード色を許さない
- `language-editor-i18n`: 全 UI 文字列を host preset 経由で受け取る（KDV が en preset を提供）
- `language-editor-host-control`: scroll / write / focus / undo / selection / search / clipboard / diagnostics / decorations の公開 API
- `language-editor-settings`: 自動保存・ショートカット・タイポグラフィ等の設定 IF
- `language-editor-ast-lint-color-rule`: kle-linter による色リテラル禁止ルール

## Known Constraints（egui MVP 段階）

- egui（epaint）の独自フォントアトラスは OS フォントフォールバックチェーンを無視するため、カラー絵文字をエディタ上で正しく表示できない。IME composition も不完全。根本解決は `katana-language-editor-floem` 実装時。
- egui 側で完全には実装できない host 制御 API（precise scroll pixel offset、decoration overlay、advanced clipboard backend など）は `EditorError::Unsupported` を返す。Floem 実装で全機能を満たす。
- `floem` は `0.2.0` (crates.io) が main と大きく乖離しているため、`katana-language-editor-floem` は **git dependency 固定**を必須ルールとする。crates.io 公開版は採用しない。

## Impact

- `crates/katana-language-editor/` — neutral interface crate（egui 非依存）
- `crates/katana-language-editor-egui/` — egui 実装 crate
- `crates/katana-language-editor-floem/` — Floem skeleton crate（v0.1.0 で workspace に追加、本実装は v0.2.x）
- `crates/kle-linter/` — 色リテラル禁止ルール追加
- `Cargo.toml` — `floem` を git dependency としてピン留め、`katana-language-editor-floem` を workspace member に追加
