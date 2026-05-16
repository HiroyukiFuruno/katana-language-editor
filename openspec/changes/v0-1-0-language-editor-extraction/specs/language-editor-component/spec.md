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

### Requirement: EditorConfig は必須 DI フィールドだけで構成されなければならない

`EditorConfig` は `theme` / `strings` / `locale` / `typography` / `spacing` / `settings` / `syntax_highlighter` を **non-nullable** で保持しなければならない（MUST）。`Option<Theme>` などのフォールバックを許してはならない（MUST NOT）。preset は host (KDV) が提供する。

#### Scenario: host が preset を渡さない構築は型レベルで失敗する

- **WHEN** host が `EditorConfig` を `theme` 無しで構築しようとする
- **THEN** Rust の型システムがコンパイルエラーで拒否する
- **THEN** KLE crate 内に default preset は存在せず、host が必ず渡す

#### Scenario: KLE crate 内に default preset が存在しない

- **WHEN** `katana-language-editor` の crate を検索する
- **THEN** `Theme::default()` / `Strings::default()` / `Typography::default()` 相当の default 実装が公開されていない
- **THEN** default は `kdv-presets` 側に存在する

### Requirement: Floem 実装は git dependency 固定で取り込まなければならない

`katana-language-editor-floem` crate は `floem` を **git dependency**（`{ git = "https://github.com/lapce/floem", rev = "<pinned-sha>" }`）として取り込まなければならない（MUST）。crates.io 公開版（`floem = "0.2.0"` 等）は main 進行と乖離しており、theme / editor view / styling API を満たさないため使用してはならない（MUST NOT）。

#### Scenario: workspace に Floem skeleton crate が存在する

- **WHEN** `cargo metadata` を実行する
- **THEN** `katana-language-editor-floem` が workspace member として登録されている
- **THEN** v0.1.0 段階では skeleton（コンパイル可能）であり、本実装は v0.2.x で完成する

#### Scenario: Floem が crates.io 由来にならない

- **WHEN** `cargo tree -p katana-language-editor-floem` を実行する
- **THEN** `floem` の source は `git+https://github.com/lapce/floem` を指す
- **THEN** `crates.io-index` 由来の `floem` は依存ツリーに現れない

### Requirement: Neutral interface は host 制御 API を公開しなければならない

`katana-language-editor` neutral crate は、host が editor を外部制御するための trait 群を公開しなければならない（MUST）。最低限以下を含む：`EditorScrollControl`、`EditorWriteAccess`、`EditorViewControl`（read-only / focus）、`EditorHistoryControl`、`EditorSelectionControl`、`EditorSearchControl`、`EditorClipboardControl`、`EditorDiagnosticsSink`、`EditorDecorationsSink`、`EditorAccessibility`。

#### Scenario: host が外部から editor をスクロール・書き込みできる

- **WHEN** host が `EditorScrollControl::scroll_to_line(42)` と `EditorWriteAccess::insert_at(pos, "...")` を呼ぶ
- **THEN** editor は対応する位置までスクロールし、指定位置にテキストを挿入する
- **THEN** これらの API は public で、UI フレームワーク非依存の DTO のみを引数に取る

#### Scenario: egui MVP で未対応の API は明示的に Unsupported を返す

- **WHEN** host が egui 実装上で `EditorDecorationsSink::push(...)` のような未実装 API を呼ぶ
- **THEN** `Result<_, EditorError::Unsupported>` を返す
- **THEN** Floem 実装で全機能を提供する旨が docs に書かれている

### Requirement: Neutral interface に UI フレームワーク型を漏らしてはならない

`katana-language-editor` neutral crate の public API は、`egui` / `floem` / `cosmic-text` / `vello` などの実装型を引数・戻り値・関連型に含めてはならない（MUST NOT）。

#### Scenario: cargo tree で実装依存が漏れていない

- **WHEN** `cargo tree -p katana-language-editor` を実行する
- **THEN** `egui` / `floem` / `cosmic-text` / `vello` / `wgpu` が現れない
