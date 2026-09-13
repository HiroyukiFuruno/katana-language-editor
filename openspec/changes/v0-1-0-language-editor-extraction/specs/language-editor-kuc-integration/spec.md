## ADDED Requirements

### Requirement: 汎用 UI 問題は KUC contract に寄せなければならない

KLE は、emoji、IME、grapheme caret、text span、font、theme token、spacing、icon、hit target、coordinate normalization など、KatanA 固有ではない UI 問題を KLE 内の ad hoc 実装として抱え込んではならない（MUST NOT）。汎用化した方がよい部分は KUC の generic contract として扱わなければならない（MUST）。

#### Scenario: emoji と IME を KUC text input contract と照合する

- **WHEN** KLE が emoji / IME / grapheme caret を扱う
- **THEN** KUC の `TextArea` / text entry / `UiTextSpan::emoji` contract と整合する API を使う
- **THEN** OS 依存の emoji 表示問題を egui fallback で隠さない

#### Scenario: KUC に不足する generic contract を追加する

- **WHEN** KLE 実装中に Katana 固有でない UI contract が KUC に不足していることが分かる
- **THEN** local KUC repository へ generic API と contract tests を追加する
- **THEN** Katana 専用 namespace、Katana 固有 enum、KLE 固有の fixture 前提を KUC core に入れない

### Requirement: KLE neutral crate は KUC runtime 型を public API に漏らしてはならない

KLE は KUC の汎用 contract を利用する場合でも、`katana-language-editor` neutral crate の public API に KUC runtime / Storybook / render backend の具象型を漏らしてはならない（MUST NOT）。KLE neutral crate は editor domain DTO と trait に閉じ、UI adapter / Storybook / host integration で KUC と接続しなければならない（MUST）。

#### Scenario: neutral crate の public API を検査する

- **WHEN** `cargo tree -p katana-language-editor` と AST lint を実行する
- **THEN** neutral crate に egui / Floem / KUC Storybook runtime の依存が現れない
- **THEN** public DTO は framework-neutral な型だけで構成される

### Requirement: KUC に寄せた挙動は KUC と KLE の両方で検証しなければならない

KUC へ追加または再利用した UI contract は、KUC 側の contract test と KLE 側の integration / Storybook test の両方で検証されなければならない（MUST）。KLE 側の合格は KUC 側の screenshot 表示だけを根拠にしてはならない（MUST NOT）。

#### Scenario: KUC TextArea contract を KLE editor で利用する

- **WHEN** KLE editor が IME commit、emoji grapheme delete、selection、paste を扱う
- **THEN** KUC 側 contract test が generic behavior を検証する
- **THEN** KLE 側 integration test が editor state/event/action として同じ behavior を検証する
- **THEN** `Paste` は KUC が payload を読まず `ClipboardRequested` を emit し、KLE は
  byte offset を char-index request range へ一箇所で写像する
- **THEN** KUC/KLE は `file://`、OS clipboard、画像 payload、画像保存を解析せず、
  KatanA host resolution のみがそれらを扱う

#### Scenario: theme token を使う

- **WHEN** KLE adapter が line number、active line、search match、diagnostic underline を描画する
- **THEN** 色は KUC/host 由来の semantic token から解決される
- **THEN** hard-coded color や `egui::Visuals::default()` fallback は AST lint で拒否される

### Requirement: Command chrome は KUC の additive wrapper contract のみを利用しなければならない

KLE SHALL compose KUC CommandChrome contracts and SHALL NOT render a generic command control locally.
KLE の authoring toolbar、code-block dropdown、document find/replace は、KUC
`kuc-command-chrome-runtime` の `CommandChrome` / `FloatingCommandToolbar` /
`CommandChromeSearchStrip` / SVG raster / egui adapter を compose しなければならない（MUST）。
KLE は既存 KUC `ToolbarAction` / `SearchControlStripAction` の public enum/struct を拡張する前提を
置いてはならず（MUST NOT）、KLE-owned `egui::Area`、button inventory、popup placement、SVG parser、
icon glyph fallback、検索 form state を実装してはならない（MUST NOT）。

#### Scenario: KLE authoring/search binding は KUC typed event だけを map する

- **WHEN** KLE egui adapter が authoring toolbar または find/replace control を表示する
- **THEN** KUC adapter へ host-provided `UiIconProps`、injected strings、editor state、anchor/viewport を渡す
- **THEN** 返る KUC typed event を `EditorActionRequest` / `EditorSearchControl` / `EditorEvent` へ map するだけである
- **THEN** KLE source/AST guard は local generic renderer、coordinate algorithm、SVG raster/cache、文字 icon fallback を拒否する

#### Scenario: host-injected command presentation is data-driven

- **WHEN** a host or Storybook supplies ordered opaque command/group/item ids,
  localized label/tooltip/accessibility text and adapter-bound KUC icon props
- **THEN** the KLE egui binding projects that presentation to KUC without a
  command-id-to-label/icon/group branch or `EditorConfig::Strings` fallback
- **THEN** a command unknown to KLE static code remains renderable and
  accessibly named when present in the injected neutral command collection
- **THEN** the neutral `katana-language-editor` crate exposes no KUC type

### Requirement: KLE は KUC command chrome 実装前に暫定 UI を完了扱いしてはならない

KLE SHALL reject temporary helper and fallback paths from every release-evidence command until the KUC migration is complete.
KUC command chrome の public runtime、component contract、egui adapter、real interaction test、frame-record
contract、guard が完了するまで、KLE の暫定 `authoring_helper.rs` または Storybook fallback renderer は
release evidence にならない（MUST NOT）。KLE は KUC implementation 後にそれらを削除し、thin binding へ
置換しなければならない（MUST）。

#### Scenario: 暫定 helper または fallback が release path に残る

- **WHEN** KLE release/storybook/parity gate が実行され、暫定 helper または fallback renderer が参照される
- **THEN** gate は KUC command chrome migration 未完了として fail する
- **THEN** screenshot、GIF、shape count、fixture canvas は合格根拠として扱われない

### Requirement: Generic editor text surface は KUC TextSurface adapter だけを利用しなければならない

KLE SHALL consume the KUC TextSurface composition and SHALL NOT implement a generic text surface in its own crate.
KLE egui binding は、multiline text、selection/caret、IME preedit/commit、generic gutter、range annotation、
scroll viewport、ContextMenu、DiagnosticsList、accessibility tree の実描画を KUC `TextSurface` と shared
`katana-ui-core-egui-adapter` の composition として利用しなければならない（MUST）。KLE は editor-domain
DTO/semantic range/host action を KUC props/event に map するだけであり（MUST）、local
`PlatformTextSurface`、`LineGutterModel`、egui texture/font/input renderer、manual hit-test、generic overlay
geometry を残してはならない（MUST NOT）。

#### Scenario: KLE local text surface が release path に残る

- **WHEN** KLE AST/dependency/release gate が local text surface、line-gutter renderer、egui TextEdit/font atlas path、または fallback frame source を検出する
- **THEN** gate は KUC TextSurface adapter migration 未完了として fail する
- **THEN** KLE public neutral crate には KUC/egui concrete type が漏れず、KLE binding は typed prop/event mapping だけを行う

### Requirement: automatic gutter facts SHALL come only from the KUC frame record

KLE SHALL obtain automatic-gutter active、hovered、display label、row bounds and marker presence only from the
returned KUC `TextSurfaceFrameRecord`. KLE MAY forward a logical-row hover request to the KUC controlled presentation,
but it MUST NOT derive row state or geometry from source text、cursor offset、diagnostic range or egui pointer state.

#### Scenario: public gutter control maps a KUC-rendered frame

- **WHEN** Japanese と `⭐️` VS16 を含む editor surface が actual RawInput により caret/hover/scroll/marker
  interaction を受ける
- **THEN** KLE は returned KUC gutter frame と AccessKit/artifact facts を neutral gutter DTO/action に map する
- **THEN** `LineGutterModel`、newline enumeration、local row geometry、local active/hover/diagnostic renderer は
  release path に存在しない
