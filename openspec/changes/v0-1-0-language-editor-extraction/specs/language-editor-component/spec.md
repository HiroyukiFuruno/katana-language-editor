## ADDED Requirements

### Requirement: LanguageEditor trait と SyntaxHighlighter 注入で言語非依存エディタを提供しなければならない

システムは、`LanguageEditor` trait（neutral interface）、`SyntaxHighlighter` trait（言語非依存ハイライト契約）、`EditorConfig`（`syntax_highlighter` を含む）、`HighlightedText` DTO を `katana-language-editor` neutral crate として提供し、ホストが `MarkdownSyntaxHighlighter` 等を実装して注入できるようにしなければならない（MUST）。

#### Scenario: KatanA が Markdown highlighter を注入する

- **WHEN** KatanA が `MarkdownSyntaxHighlighter` を実装し `EditorConfig::syntax_highlighter` に渡す
- **THEN** `katana-language-editor` は注入された highlighter を使ってシンタックスハイライトを適用する
- **THEN** `katana-language-editor` 自体は Markdown のドメイン知識を持たない

#### Scenario: katana-language-editor は egui に依存しない

- **WHEN** `cargo tree -p katana-language-editor` を実行する
- **THEN** `egui` は含まれない

### Requirement: katana-language-editor-egui が egui MVP 実装を提供しなければならない

システムは、egui TextEdit ベースの editor widget と行番号・シンタックスハイライト描画・絵文字フォント workaround を `katana-language-editor-egui` impl crate として提供しなければならない（MUST）。KatanA は `EditorConfig` を構築して widget を配置するだけで editor を利用できる。

#### Scenario: editor を egui 上に表示する

- **WHEN** ホストが `EguiLanguageEditor::show(ui)` を呼ぶ
- **THEN** egui ui に TextEdit が描画される
- **THEN** 行番号・シンタックスハイライトが表示される

#### Scenario: egui MVP の既知制約

- **WHEN** ユーザーが日本語入力 / カラー絵文字を使う
- **THEN** egui TextEdit の IME 不完全とカラー絵文字非対応の制約が適用される
- **THEN** 制約は docs に明記され、`katana-language-editor-floem` 実装で根本解決する
