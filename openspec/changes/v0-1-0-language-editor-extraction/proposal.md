## Why

KatanA v0.27.0 で `katana-ui` から editor 実装を切り出す。切り出し先をこの repo とし、KatanA は git dependency として consume するだけにする。KatanA の検証範囲から editor の実装詳細を除く。将来の独自 input surface 導入時も、この repo の egui 実装を差し替えるだけで KatanA 側は無影響にする。

## What Changes

- KatanA `katana-ui` の editor コード（TextEdit ラップ、行番号、シンタックスハイライト、絵文字フォント管理等）を `katana-language-editor-egui` へ移管する。
- `katana-language-editor`（neutral interface）の `LanguageEditor` trait と DTO を確定する。
- `EditorConfig` 経由の設定注入で `katana-ui` のグローバル設定依存を排除する。
- `v0.1.0` として release tag を切る。

## Capabilities

### New Capabilities

- `language-editor-component`: neutral interface + egui MVP 実装。

## Impact

- `crates/katana-language-editor/` — neutral interface crate
- `crates/katana-language-editor-egui/` — egui 実装 crate
