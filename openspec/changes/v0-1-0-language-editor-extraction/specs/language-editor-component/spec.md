## ADDED Requirements

### Requirement: opaque provider契約の定義元をneutral crateに一本化しなければならない

KLE MUST define `HostProjectionProvider` と `HostProjectionProviderError` をneutral
crateだけに置く。providerは関連型`Lease`/`Error`とretain/synchronize操作を持ち、
vendor型、payload表現、Clone/Debug/Serialize要件をneutral traitへ追加してはならない。
egui adapterは同一定義を再exportし、実KUC leaseとの型等価制約をMUST enforceする。
旧neutral DTO/controlの残存は別の未完了移行であり、一本化だけで全境界完了としない。

#### Scenario: adapterがprovider契約を再定義する

- **WHEN** 正規neutral module外のproduction Rustに同名trait又は型の定義を追加する
- **THEN** 通常のASTゲートはprivate/nested/raw identifierでも拒否する
- **AND** 同一定義のre-exportとコメント・文字列内の名前は拒否しない
- **AND** この構文検査を別名の意味的複製や機能互換性の証明として扱わない

#### Scenario: UI依存を持たないconsumerがprovider契約を実装する

- **WHEN** consumerがneutral crateだけに依存し、自身のopaque lease/error型を指定する
- **THEN** KUC/eguiを参照せずコンパイルでき、Clone/Debug等の不要なtrait実装を要求されない
- **THEN** このcompile-only確認をKatanA実入力や全機能互換の証拠にしない

#### Scenario: egui editorへ異なるlease型を渡す

- **WHEN** consumerがKUCのconcrete leaseと異なる関連型を指定したproviderを渡す
- **THEN** public editor/bindingの構築はコンパイルエラーになる
- **THEN** AnyやStringへの変換、同名の別trait、fallback実装で受け入れない

#### Scenario: 実KUC leaseを消費してframeを表示する

- **WHEN** public editorがproviderからretain又はsynchronize leaseを取得する
- **THEN** leaseを所有権移動し、同じleaseの再利用は型システムで拒否する
- **THEN** 既存のtyped stale/errorと一度だけのevent forwardingを保持し、実KUC回帰で検証する

### Requirement: LanguageEditor trait と SyntaxHighlighter 注入で言語非依存エディタを提供しなければならない

システムは、`LanguageEditor` trait（neutral interface）、`SyntaxHighlighter` trait（言語非依存ハイライト契約）、`EditorConfig`（`syntax_highlighter` を含む）、`HighlightedText` DTO を `katana-language-editor` neutral crate として提供し、ホストが `MarkdownSyntaxHighlighter` 等を実装して注入できるようにしなければならない（MUST）。

#### Scenario: KatanA が Markdown highlighter を注入する

- **WHEN** KatanA が `MarkdownSyntaxHighlighter` を実装し `EditorConfig::syntax_highlighter` に渡す
- **THEN** `katana-language-editor` は注入された highlighter を使ってシンタックスハイライトを適用する
- **THEN** `katana-language-editor` 自体は Markdown のドメイン知識を持たない

#### Scenario: katana-language-editor は egui に依存しない

- **WHEN** `cargo tree -p katana-language-editor` を実行する
- **THEN** `egui` は含まれない

### Requirement: katana-language-editor-egui が KUC-backed full editor implementation を提供しなければならない

システムは、retained KUC `TextSurface` / `CommandChrome` / `ContextMenu` root を使う editor widget と、行番号・syntax span・Japanese IME・exact `⭐️` VS16 color glyph・diagnostics・search/replace・AccessKit を `katana-language-editor-egui` impl crate として提供しなければならない（MUST）。KatanA は host presentation と typed host action を注入して widget を利用できる。KLE 独自の `egui::TextEdit` renderer、font workaround、又は fallback pixel path を持ってはならない（MUST NOT）。

#### Scenario: editor を egui 上に表示する

- **WHEN** ホストが `EguiLanguageEditor::show(ui)` を呼ぶ
- **THEN** KUC retained root surface が一度だけ描画される
- **THEN** 行番号・シンタックス span・IME/emoji・command chrome・context menu・AccessKit が同じ KUC frame record と composited artifact に現れる

#### Scenario: KUC platform text contract

- **WHEN** ユーザーが日本語入力 / カラー絵文字を使う
- **THEN** KUC platform text raster と IME contract が exact code-point sequence、selection/caret、color-glyph pixel、AccessKit を同じ frame で返す
- **THEN** `⭐️` を `☆` 又は monochrome substitute に置換してはならない

### Requirement: KLE configにgeneric UI設定の二重所有を残してはならない

KLE MUST NOT `EditorConfig` / `EditorConfigInput` に `typography` / `spacing` /
`settings` / `theme` / `strings` / `locale` を保持する。private fieldやOption、別名storeへの退避でも同じである。
theme/string/locale/font/spacingのgeneric実行責務はKUC root、semantic descriptorは
hostに置く。旧configの残存fieldや公開設定型も移行対象であり、fieldの除去だけで
config全体の責務解消と見なしてはならない。未接続の旧Storybook factoryも再利用しない。

#### Scenario: 旧設定fieldをconfigに再追加する

- **WHEN** neutral crateの対象config structへ禁止6fieldのいずれかを追加する
- **THEN** AST lintが実fieldの再導入を拒否する
- **THEN** raw identifierやprivate可視性でも拒否し、無関係なstructや文字列は誤検出しない

#### Scenario: 旧style DTOの再導入を拒否する

- **WHEN** consumerがneutral root又はtypes moduleからTypography又はSpacingをimportする
- **THEN** 各型・各パスの独立したcompile-fail検査で拒否する
- **AND** neutral内の同名struct/enum/type aliasの再導入はASTゲートで拒否する
- **AND** フォント・余白・計測・hit testの機能要件はKUC/hostで引き続き検証対象とする

#### Scenario: hostのpresentationを実KUC rootで扱う

- **WHEN** hostが既存KUC公開契約へrevisioned projectionを渡す
- **THEN** KLEはopaque relayだけを行い、KDV固有presetやKLE defaultで補完しない
- **THEN** 必要なdescriptor不足や未検証leafを構築成功だけで合格にしない

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

#### Scenario: v0.1.0 target API has no Unsupported escape

- **WHEN** host が v0.1.0 target API を呼ぶ
- **THEN** KUC/KLE/host の実装済み typed path が実行される
- **THEN** `EditorError::Unsupported`、future Floem、又は host fallback を成功結果として使用してはならない

### Requirement: Neutral interface に UI フレームワーク型を漏らしてはならない

`katana-language-editor` neutral crate の public API は、`egui` / `floem` / `cosmic-text` / `vello` などの実装型を引数・戻り値・関連型に含めてはならない（MUST NOT）。

#### Scenario: cargo tree で実装依存が漏れていない

- **WHEN** `cargo tree -p katana-language-editor` を実行する
- **THEN** `egui` / `floem` / `cosmic-text` / `vello` / `wgpu` が現れない
