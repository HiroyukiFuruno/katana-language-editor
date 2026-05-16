## ADDED Requirements

### Requirement: 全色表現は Theme 経由で必ず受け取らなければならない

`katana-language-editor` および `katana-language-editor-egui` / `katana-language-editor-floem` は、editor 内で使う全ての色（前景・背景・選択・キャレット・ガター・行ハイライト・diagnostics underline・decoration overlay 等）を `Theme` から取得しなければならない（MUST）。色のハードコードを行ってはならない（MUST NOT）。

#### Scenario: host が Theme を渡して描画する

- **WHEN** host が `Theme` を `EditorConfig::theme` に渡して editor を構築する
- **THEN** editor の全描画は `Theme::colors` のいずれかから色を引く
- **THEN** `egui::Color32::WHITE` や `Color::rgb(...)` 直書きが impl crate に存在しない

#### Scenario: Theme は Option ではない

- **WHEN** host が `EditorConfig` を構築する
- **THEN** `theme` フィールドは `Theme`（non-nullable）である
- **THEN** `Option<Theme>` や default fallback は提供されない

### Requirement: テーマ preset は host (KDV) が提供し KLE 内に default を持たない

`katana-language-editor` crate は default テーマ（dark/light preset）を持ってはならない（MUST NOT）。default preset は `kdv-presets` 側で実装され、host が必須引数として KLE に渡す。

#### Scenario: KDV preset を渡す

- **WHEN** host が `kdv-presets::theme::dark()` などを呼んで `Theme` を取得し `EditorConfig::theme` に渡す
- **THEN** editor は KDV preset の色で描画する
- **THEN** KLE crate 内で `Theme::default()` 相当の関数を grep しても見つからない

#### Scenario: ダーク/ライト切替は host が制御する

- **WHEN** host が UI 操作で dark/light を切り替える
- **THEN** host は新しい `Theme` を生成して editor に `apply_theme(theme)` で渡す
- **THEN** editor は theme 切替の意思決定を内部で行わない

### Requirement: 色トークンは意味付き alias を含まなければならない

`ColorTokens` は raw な palette だけでなく、editor 用途のセマンティック alias（`text_primary` / `text_muted` / `selection_bg` / `caret` / `gutter_fg` / `current_line_bg` / `diagnostic_error` / `diagnostic_warn` / `diagnostic_info` / `decoration_accent` 等）を必ず公開しなければならない（MUST）。

#### Scenario: 実装は alias を使って描画する

- **WHEN** editor 実装が diagnostics underline を描画する
- **THEN** `theme.colors.diagnostic_error` のような alias 経由で色を取る
- **THEN** raw palette index への直接アクセスは禁止される
