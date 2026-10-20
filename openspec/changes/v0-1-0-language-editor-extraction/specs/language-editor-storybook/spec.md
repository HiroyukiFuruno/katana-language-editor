## ADDED Requirements

### Requirement: KLE は KDV 型の live Storybook harness を提供しなければならない

`katana-language-editor` workspace は `tools/kle-storybook` を提供し、KLE editor を interactive に操作できる Storybook harness を持たなければならない（MUST）。Storybook は static mock や screenshot-only の展示ではなく、props / state / event / action / callback を実際に通す live harness として実装しなければならない（MUST）。

#### Scenario: interactive Storybook を起動する

- **WHEN** 開発者が `just storybook` を実行する
- **THEN** KLE Storybook の interactive window が起動する
- **THEN** この recipe は smoke test alias ではなく、実際の操作画面を起動する

#### Scenario: headless smoke で window 描画を検証する

- **WHEN** 開発者が `just storybook-window-smoke` を実行する
- **THEN** Storybook は headless/smoke mode で起動し、editor fixture が描画されたことを検証する
- **THEN** 出力 artifact が空でないことを機械的に確認する

#### Scenario: interaction roundtrip を検証する

- **WHEN** 開発者が `just storybook-interaction-check` を実行する
- **THEN** Storybook は editor の入力、選択、scroll、search、diagnostics、clipboard action を state/event/action/callback 経由で検証する
- **THEN** UI 文字列や style_class の parse で action を復元しない

#### Scenario: pending cursor restore を検証する

- **WHEN** 開発者が `just storybook-contract-check` を実行する
- **THEN** Storybook は authoring transform 後に必要な pending cursor restore が selection/cursor event として復元されることを検証する
- **THEN** byte offset と char-index を混同した復元は contract failure になる

#### Scenario: KatanA editor action callback を検証する

- **WHEN** 開発者が `just storybook-contract-check` を実行する
- **THEN** Storybook は save、format、image ingest、authoring command、diagnostic fix、view mode、split direction の action request が `EditorEvent::ActionRequested` として流れることを検証する
- **THEN** Markdown 変換、file IO、preview 切替の実行責務は KatanA host adapter に残し、KLE core へ KatanA 固有依存を入れない

#### Scenario: diagnostic popup contract を検証する

- **WHEN** 開発者が `just storybook-contract-check` を実行する
- **THEN** Storybook は gutter line から diagnostic popup item が生成されることを検証する
- **THEN** fix、fix-all、docs の action request が `EditorEvent::ActionRequested` として流れることを検証する

#### Scenario: gutter line click contract を検証する

- **WHEN** 開発者が `just storybook-contract-check` を実行する
- **THEN** Storybook は active / diagnostic gutter line state を検証する
- **THEN** gutter line click が `ActivateGutterLine` action callback として流れることを検証する

### Requirement: Storybook は KUC の汎用 UI contract を bypass してはならない

KLE Storybook は、KUC が提供する text area / text span / theme / font / action / hit target / coordinate contract を利用しなければならない（MUST）。KLE Storybook 側で manual hit-test reconstruction、文字列 parse による action synthesis、KUC contract の bypass を実装してはならない（MUST NOT）。

#### Scenario: manual action synthesis を AST lint で禁止する

- **WHEN** Storybook 実装に manual hit-test や style string parse による action synthesis が混入する
- **THEN** `kle-linter` の Storybook contract rule が違反として報告する
- **THEN** `just storybook-contract-check` または `just ast-lint` が fail する

#### Scenario: KUC hit target を使って click を処理する

- **WHEN** Storybook window で editor gutter、selection、toolbar、diagnostic marker などの interactive target を click する
- **THEN** click は KUC/KLE の typed action contract を通って処理される
- **THEN** Storybook 固有の座標補正や action 再構築に依存しない

### Requirement: Storybook acceptance artifact を release readiness に含めなければならない

KLE は release readiness で Storybook の live acceptance artifact を生成し、interactive harness が release 時点で動作する証跡を残さなければならない（MUST）。

#### Scenario: release readiness で Storybook artifact を生成する

- **WHEN** release readiness gate を実行する
- **THEN** Storybook smoke / interaction / emoji / contract check が通る
- **THEN** live acceptance artifact が `target/acceptance` 配下へ生成される

### Requirement: MUST Storybook はフルエディターの実コンポーネントを同一 root で提供しなければならない

MUST NOT: KLE Storybook は部分的な text fixture、静的 gallery、又は操作を省いた MVP を
editor として提供してはならない（MUST NOT）。各 scene は KUC の retained full-editor
root を KLE の one-shot binding を通じて実行し、source-derived parity leaf を一つずつ
実入力で再生する live harness でなければならない（MUST）。KatanA が所有する副作用は
Storybook action callback で代用せず、同じ `step_id` を持つ KLE-owned actual KatanA
host E2E で独立して検証しなければならない（MUST）。

#### Scenario: full editor controls を操作する

- **WHEN** interactive 又は headless Storybook が full-editor scenario を実行する
- **THEN** 行番号・diagnostic gutter・検索・replace-current/replace-all・floating Markdown toolbar・context menu・17 code kinds・tab/group・breadcrumb/source address・Problems/status・preview/split の current root が同時に存在する
- **THEN** 各操作は Pointer、keyboard、又は AccessKit の物理入力から current KUC root record と AccessKit snapshot を更新する
- **THEN** KLE は selection、query、range、line、scroll、popup、tab/group、action payload、描画 child を保存・再構成しない

#### Scenario: source-derived leaf を省略しない

- **WHEN** acceptance manifest を生成する
- **THEN** `docs/v0-1-0-editor-requirements.md` の source-derived leaf と user-mandated replace leaf の各々に、一意の `step_id`、入力 origin、KUC record、AccessKit evidence、visible assertion、class-appropriate host-effect evidence がある
- **THEN** aggregate count、source marker、fixture 状態、pending action、又は Storybook-only callback は leaf の合格証拠にならない

#### Scenario: 外部UI入口の採取を全leaf解析の完了としない

- **WHEN** 固定KatanAが依存するeguiの入口定義と呼出しを採取したが、推移的な意味解析とreplacement leaf結合が未完了である
- **THEN** canonicalとdiagnosticの双方で未完了状態と理由を保持する
- **THEN** RC、KLE公開前、KatanA採用後のvalidatorは外部依存の欠落・未完了・未解決・replacement leaf不在を拒否する
- **THEN** 認証済みRust sourceのSHAは入口定義の有無で省略せず、全source採取を全symbol解析の証明としない

### Requirement: MUST Storybook の媒体成果物は KUC の実コンポジットから生成・照合されなければならない

MUST: Storybook は各 acceptance stage の KUC full-root final composite を唯一の画像出所として、
numbered PNG、contact sheet、GIF、MP4、manifest、SHA-256 を生成しなければならない
（MUST）。`egui::FullOutput.shapes`、shape count、minifb、手書き raster、fallback font、
fixture renderer、又は独自座標・glyph 再構成を媒体の出所としてはならない（MUST NOT）。

#### Scenario: 動画と連番フレームを機械照合する

- **WHEN** motion artifact gate を実行する
- **THEN** PNG header、dimension、non-empty、duplicate、sequence order、freshness、KUC compositor provenance を検証する
- **THEN** MP4 を decode した各フレーム hash は対応する PNG sequence と照合され、欠落・余剰・不一致は gate failure になる
- **THEN** exact `⭐️` VS16、Japanese、ZWJ、IME/caret の visual evidence は KUC PlatformFontCatalog の resolved-face fingerprint を含み、`☆`、mono fallback、code-point assertion は不合格になる

#### Scenario: fallback renderer を拒否する

- **WHEN** Storybook 又は release artifact の compiled dependency graph に fallback renderer、manual hit-test、shape-count acceptance、文字列/座標からの action synthesis が存在する
- **THEN** `kle-linter` の AST rule と runtime provenance gate が fail する
- **THEN** fallback path が未実行であっても release evidence としては不合格になる
