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
- **THEN** Violation は `path`、`line`、`column`、`literal`、`hint`（"Use theme.colors.* instead"）を含む
- **THEN** 開発者は出力だけで該当箇所を特定できる

### Requirement: 例外は preset crate とテストコードのみに限定しなければならない

`prohibited-color-literal` ルールは、`kdv-presets` 配下、`tests/` 配下、`#[cfg(test)]` 内、明示的に `#[allow(kle_lint::prohibited_color_literal)]` が付いたスコープでのみ色リテラルを許可する（MUST）。それ以外の crate では `Theme` 経由を強制する。

#### Scenario: kdv-presets では色リテラル可

- **WHEN** `kdv-presets/src/theme/dark.rs` が `Color::rgb(0x1e, 0x1e, 0x1e)` を含む
- **THEN** ルールは違反として報告しない
- **THEN** ただし対象ファイルが preset 用途であることがディレクトリで判定される

#### Scenario: 明示 allow

- **WHEN** やむを得ない理由で実装 crate に色リテラルを置く（例: GPU shader bind の placeholder）
- **THEN** 該当スコープに `#[allow(kle_lint::prohibited_color_literal)]` を付けると例外として扱われる
- **THEN** allow には PR レビューで根拠コメントを残すことを docs で要求する

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
- **THEN** `theme.colors.*` 経由への置き換えを要求する
