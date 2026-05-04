# Tasks: katana-language-editor v0.1.0

## Branch Rule

- **標準ブランチ**: `release/v0.1.0`
- **作業ブランチ**: `feature/v0.1.0-task-x`

---

## 1. neutral interface を確定する

- [ ] 1.1 `LanguageEditor` trait と DTO（`TextContent` / `EditorConfig` / `CursorPosition` / `Selection` / `EditorEvent` / `EditorOutput`）を本実装向けに拡張する
- [ ] 1.2 `katana-language-editor` の `cargo tree` に `egui` が含まれないことを確認する
- [ ] 1.3 KatanA が interface crate のみを依存しても型エラーが出ないことを確認する

---

## 2. KatanA editor 実装を移管する

- [ ] 2.1 KatanA `katana-ui` の editor コードを `katana-language-editor-egui` へ移管する
- [ ] 2.2 `EditorConfig` 経由の設定注入を確立し、`katana-ui` のグローバル設定依存を排除する
- [ ] 2.3 絵文字フォント管理（macOS / Windows プラットフォーム差異）を egui crate へカプセル化する
- [ ] 2.4 シンタックスハイライトを egui crate へ移管する（Mermaid / Draw.io block のハイライトは構文認識のみ、描画は kcf 経由）

---

## 3. v0.1.0 release

- [ ] 3.1 `cargo fmt` / `cargo clippy --workspace -- -D warnings` / `cargo test --workspace` が通る
- [ ] 3.2 release tag `v0.1.0` を切り GitHub Release を作成する
- [ ] 3.3 KatanA v0.27.0 が `katana-language-editor = { git = "...", tag = "v0.1.0" }` でビルドできることを確認する
