## ADDED Requirements

### Requirement: editor が表示する全 UI 文字列は Strings 経由で必ず受け取らなければならない

`katana-language-editor` および各実装 crate は、ユーザーに見える全ての文字列（メニュー、ボタン、ツールチップ、エラーメッセージ、ガイダンス、accessibility ラベル、context menu、find バー等）を `Strings` DI 経由で取得しなければならない（MUST）。impl crate 内に英語・日本語などの自然言語リテラルを直接含めてはならない（MUST NOT）。

#### Scenario: host が Strings を渡して editor を構築する

- **WHEN** host が `Strings`（KDV preset 由来）を `EditorConfig::strings` に渡す
- **THEN** editor は内部で表示する全 UI 文字列を `Strings` のキー経由で解決する
- **THEN** `egui::Label::new("Save")` のような直接リテラルが impl crate に存在しない

#### Scenario: Strings は Option ではない

- **WHEN** host が `EditorConfig` を構築する
- **THEN** `strings` フィールドは `Strings`（non-nullable）である
- **THEN** `Option<Strings>` / default fallback は提供されない

### Requirement: locale は LTR/RTL direction を含み、editor は direction に応じて配置を切り替えなければならない

`Locale { language: String, direction: TextDirection }` を neutral crate に定義し、`EditorConfig::locale` で必ず受け取る（MUST）。`TextDirection::Rtl` 指定時、行先頭・行末・gutter 配置を RTL 向けに反転する（MUST）。

#### Scenario: RTL locale で配置が反転する

- **WHEN** host が `Locale { language: "ar", direction: TextDirection::Rtl }` を渡す
- **THEN** gutter は行の右側に配置される
- **THEN** caret 移動キーは方向に応じて意味付けされる（egui 実装で部分対応の場合は `EditorError::Unsupported` を返す）

### Requirement: KLE crate 内に default 文字列を持たず、host (KDV) が en preset を必ず提供する

`katana-language-editor` crate は default `Strings` を持ってはならない（MUST NOT）。host (KDV) は最低限 `en` preset を `kdv-presets` で提供し、これを必須引数として渡す（MUST）。

#### Scenario: KDV en preset を渡す

- **WHEN** host が `kdv-presets::strings::en()` を呼んで `Strings` を取得し `EditorConfig::strings` に渡す
- **THEN** editor は KDV 由来の英語文字列で UI を構築する
- **THEN** KLE crate 内で `Strings::default()` / 英語フォールバックを grep しても見つからない

#### Scenario: 未訳キー検出

- **WHEN** host が独自言語の `Strings` を渡したが特定のキーが未定義の場合
- **THEN** editor は `Strings` 型の必須フィールド機構（Rust の構造体フィールド）でコンパイル時に検知する
- **THEN** 実行時の untranslated key を許容しない（key の集合は型レベルで閉じている）

### Requirement: Strings のキー集合は editor 機能と一対一で対応し、`#[non_exhaustive]` で拡張可能にしなければならない

`Strings` 構造体は editor が出す UI 文字列の semantic 単位ごとにフィールドを持ち、`#[non_exhaustive]` を付けて将来の追加が SemVer break にならないようにしなければならない（MUST）。

#### Scenario: 新規 UI 機能追加時の互換性

- **WHEN** editor に find バー新機能を追加し `Strings::find_placeholder` を増やす
- **THEN** host は `..` パターンや builder で残りフィールドを埋められる
- **THEN** 既存 host のコンパイルが壊れない（または意図的に壊して上書きを促す）
