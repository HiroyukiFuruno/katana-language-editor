## ADDED Requirements

### Requirement: kle-linter は色リテラル禁止ルール `prohibited-color-literal` を提供しなければならない

`kle-linter` crate は `prohibited-color-literal` ルールを実装し、`crates/katana-language-editor*` 配下の Rust ソースに色リテラルが現れた場合に違反として報告しなければならない（MUST）。検出対象には少なくとも以下を含む：

- `egui::Color32::*` のフィールド参照 / `egui::Color32::from_rgb(...)` 系コンストラクタ
- `floem::peniko::Color::rgb(...)` / `Color::rgba(...)` / `Color::WHITE` 等の named constant / コンストラクタ
- `#xxx` / `#xxxxxx` / `#xxxxxxxx` の hex 文字列リテラル
- `rgb(...)` / `rgba(...)` / `hsl(...)` / `hsla(...)` の文字列リテラル

#### Scenario: 実装 crate に色リテラルがあると違反として検出される

- **WHEN** `katana-language-editor-egui/src/lib.rs` に `egui::Color32::WHITE` が現れる
- **THEN** `kle-linter` が `prohibited-color-literal` 違反として報告する
- **THEN** CI が exit code 非 0 で fail する

#### Scenario: 違反は file_path:line:column と該当リテラルを示す

- **WHEN** ルールが違反を検出する
- **THEN** Violationはpath/line/column、該当literal、hintを含み、hintはKUCがhost presentationの色を解決する境界を示す
- **THEN** 開発者は出力だけで該当箇所を特定できる

#### Scenario: 日本語や複数行を含むsourceのliteralを報告する

- **WHEN** 同一SourceFile由来のASTで色constant/call/stringの違反を検出する
- **THEN** literalは対応spanの元ソースを保持し、constructorは引数を含むcall全体を示す
- **AND** 日本語・emoji・CRLF・raw string・複数行でもbyte/columnを混同しない
- **AND** reportの補足literal/hintは改行等をescapeして別の診断行を偽装させない

#### Scenario: 診断spanがsource範囲外である

- **WHEN** AST spanから元ソースを取得できない
- **THEN** DiagnosticSpanエラーを返し、空literalや部分一覧を成功として返さない
- **AND** literal/hintを持たない既存ruleの診断表示は変更しない

### Requirement: allowやpath名によるproduction検査回避を許してはならない

色ruleはneutral/egui/floemのsrc境界内を検査し、allow又はtests/kdv-presetsというpath componentで除外してはならない（MUST NOT）。既存の明示的cfg(test) module/functionのunit fixture処理のみを維持し、KUC/hostの色処理をKLEに複製しない。

#### Scenario: checkout祖先とsubmodule名で検査を回避できない

- **WHEN** 対象crateのsrc内又はcheckout祖先にtests/kdv-presetsというpath名がある
- **THEN** productionの色literalを検出する
- **AND** 対象外crateを走査しただけでKLE libraryの色ruleを適用することはない

#### Scenario: 色専用allowも拒否する

- **WHEN** file/itemにallow(kle_lint::prohibited_color_literal)を付ける
- **THEN** attribute ruleはallow自体を拒否し、color ruleもfileを検査対象から除外しない

### Requirement: ルールは KLE workspace の CI / Justfile から自動実行されなければならない

`prohibited-color-literal` を含む `kle-linter` の全ルールは、`just lint` / CI で実行され、違反があれば fail させる（MUST）。

#### Scenario: CI で違反検出

- **WHEN** PR が `katana-language-editor-egui` に色リテラルを混入させる
- **THEN** GitHub Actions の `kle-linter` ジョブが fail する
- **THEN** マージは block される

### Requirement: ルールは将来の Floem 実装にも継続適用しなければならない

`katana-language-editor-floem` が `v0.2.x` で本実装に進む際も、`prohibited-color-literal` ルールは継続適用される（MUST）。Floem 固有の色型（`floem::peniko::Color` / `floem::style::Style::color(...)` 等）も検出対象に含む。

#### Scenario: Floem 実装でも違反検出

- **WHEN** `katana-language-editor-floem/src/lib.rs` に `Color::rgb8(255, 255, 255)` が現れる
- **THEN** `kle-linter` が違反として報告する
- **THEN** KUC/host所有の色処理へ修正するよう案内し、KLEのTheme再導入を勧めない
