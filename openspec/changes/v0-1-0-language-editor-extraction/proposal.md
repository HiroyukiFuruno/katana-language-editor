## Why

editor 実装を独立した crate として確立する。`katana-language-editor` は言語非依存の汎用テキストエディタ widget として設計する。KatanA は Markdown に特化した `SyntaxHighlighter` 実装を注入して使うが、`katana-language-editor` 自体はどの言語かを知らない。

加えて、v0.1.0 段階で **neutral interface 全体の DI 契約**（テーマ・i18n・タイポグラフィ・host 制御 API・設定・診断/装飾）を確定し、KatanA editor の source-derived 機能を KUC-backed egui binding で完全に提供する。将来の Floem 実装は neutral 層の追加 consumer であり、v0.1.0 の機能、OS 依存 emoji/IME、または host 制御 API を未実装のままにする理由にはならない。

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
- **KUC-backed egui 実装（`katana-language-editor-egui`）**
  - retained KUC `TextSurface` / `CommandChrome` / `ContextMenu` root を一度だけ呼び出し、KLE は neutral DTO と typed event/action の変換だけを担う
  - 行番号、シンタックス span、選択、IME、emoji、診断、検索、toolbar、context menu、AccessKit、artifact root は KUC generic contract を組み合わせて提供する
  - KatanA source-derived feature を `EditorError::Unsupported`、egui font workaround、local fallback renderer、または将来の Floem 移行へ退避させない
- **KatanA editor parity 要件定義**
  - KatanA が現在持つ editor の編集、行番号、検索、診断、装飾、scroll、selection、clipboard、shortcut 接続を `docs/v0-1-0-editor-requirements.md` に可視化する。
  - workspace / file IO / virtual path / preview sync coordinator / Markdown 固有 adapter は host 側に残し、KLE は neutral contract と adapter 実装に閉じる。
- **KDV 型 Storybook live harness**
  - `tools/kle-storybook` を追加し、`just storybook` は interactive window を起動する。
  - Storybook は props / state / event / action / callback を実際に通す live harness とし、static mock や screenshot-only を合格条件にしない。
  - smoke / interaction / emoji / contract / acceptance artifact の自動検証を release readiness に含める。
- **KUC 汎用 UI contract への寄せ込み**
  - emoji / IME / grapheme caret / text span / font / theme token / hit target / coordinate normalization は KUC の generic contract と照合する。
  - 汎用化すべき不足 contract は local KUC repo に実装し、Katana 固有 namespace を KUC core に入れない。
- **Floem 追加 consumer の準備**
  - `katana-language-editor-floem` skeleton crate を workspace に追加し、依存は `floem = { git = "https://github.com/lapce/floem", rev = "<pinned>" }` で**必ず git 固定**にする（crates.io の `0.2.0` は古く、theme/styling/editor view API が main と乖離しているため使用しない）。
  - skeleton crate は neutral 契約の将来 consumer としてコンパイル可能でよいが、KUC-backed egui binding の v0.1.0 完全性を弱めない。
- KatanA は `MarkdownSyntaxHighlighter` を実装して `EditorConfig` に注入するだけ。
- `v0.1.0` として release tag を切る。

## Capabilities

### New Capabilities

- `language-editor-component`: 言語非依存 neutral interface + KUC-backed egui 実装 + Floem skeleton + 共通 DTO
- `syntax-highlighter-injection`: 外部から言語固有の highlighter を注入できる設計
- `language-editor-theming`: 全色を host preset 経由で受け取り、ハードコード色を許さない
- `language-editor-i18n`: 全 UI 文字列を host preset 経由で受け取る（KDV が en preset を提供）
- `language-editor-host-control`: scroll / write / focus / undo / selection / search / clipboard / diagnostics / decorations の公開 API
- `language-editor-settings`: 自動保存・ショートカット・タイポグラフィ等の設定 IF
- `language-editor-ast-lint-color-rule`: kle-linter による色リテラル禁止ルール
- `language-editor-storybook`: KDV 型 live Storybook harness と release readiness artifact
- `language-editor-kuc-integration`: KUC 汎用 UI contract への寄せ込みと boundary 検証

## v0.1.0 Completion Constraints

- OS 依存のカラー絵文字、IME composition、grapheme caret、scroll/overlay、clipboard request、AccessKit は KUC generic contract の実装対象である。KLE/KDV に fallback、font workaround、又は duplicate renderer を置かない。
- KatanA source-derived feature と user-mandated visible replace/replace-all は KUC/KLE/host の actual path を持つまで release blocker であり、`EditorError::Unsupported` を返して通過してはならない。
- `floem` は将来 consumer のため git dependency 固定を維持してよいが、v0.1.0 parity 判定には使用しない。

## Impact

- `crates/katana-language-editor/` — neutral interface crate（egui 非依存）
- `crates/katana-language-editor-egui/` — egui 実装 crate
- `crates/katana-language-editor-floem/` — Floem skeleton crate（v0.1.0 で workspace に追加、本実装は v0.2.x）
- `crates/kle-linter/` — 色リテラル禁止ルール追加
- `tools/kle-storybook/` — KDV 型 Storybook live harness
- `docs/v0-1-0-editor-requirements.md` — KatanA editor parity 要件定義
- `Cargo.toml` — `floem` を git dependency としてピン留め、`katana-language-editor-floem` を workspace member に追加
