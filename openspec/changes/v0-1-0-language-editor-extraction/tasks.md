# Tasks: katana-language-editor v0.1.0

## Branch Rule

- **標準ブランチ**: `release/v0.1.0`
- **作業ブランチ**: `feature/v0.1.0-task-x`

---

## 1. neutral interface を確定する

- [ ] 1.1 `LanguageEditor` trait と DTO（`TextContent` / `EditorConfig` / `CursorPosition` / `Selection` / `EditorEvent` / `EditorOutput`）を本実装向けに拡張する
- [ ] 1.2 `katana-language-editor` の `cargo tree` に `egui` が含まれないことを確認する
- [ ] 1.3 KatanA が interface crate のみを依存しても型エラーが出ないことを確認する
- [ ] 1.4 `EditorConfig` のフィールドは theme / strings / typography / spacing / settings / syntax_highlighter を `Option` ではなく必須として持つ
- [ ] 1.5 全 DTO に `#[non_exhaustive]` を付けて将来追加可能にしつつ、必須フィールドの追加で SemVer break が起きないこと（または起きる場合は意図的であること）を docs に明示する

---

## 2. テーマ DI を定義する

- [ ] 2.1 `Theme { colors: ColorTokens, dark: bool }` と `ColorTokens`（前景・背景・選択・キャレット・ガター・ハイライト各種 / セマンティック alias 含む）を neutral crate に定義する
- [ ] 2.2 `EditorConfig::theme: Theme` を **non-nullable** で受け取る（`Option<Theme>` を許容しない）
- [ ] 2.3 host (KDV) 側で `kdv-presets` を提供する前提を docs に明記し、KLE 内に default preset を持たないことを確認する
- [ ] 2.4 egui 実装は注入された `Theme` のみで描画し、`egui::Visuals::default()` 直参照を持たない

---

## 3. i18n DI を定義する

- [ ] 3.1 `Locale`（言語コード + direction LTR/RTL）と `Strings`（editor が出す全 UI テキストのキー集合）を neutral crate に定義する
- [ ] 3.2 `EditorConfig::strings: Strings` / `EditorConfig::locale: Locale` を **non-nullable** で受け取る
- [ ] 3.3 host (KDV) が **en preset** を必須で渡す前提を docs に明記し、KLE 内に default 文字列を持たないことを確認する
- [ ] 3.4 editor 実装にユーザー可視文字列リテラルが残らないことを CI で検査する（手動レビューでも可、後続 task でルール化）

---

## 4. タイポグラフィ / スペーシング DI を定義する

- [ ] 4.1 `Typography`（font family / size / weight / line-height / letter-spacing）と `Spacing`（padding / radius / gap）を neutral crate に定義する
- [ ] 4.2 `EditorConfig::typography: Typography` / `EditorConfig::spacing: Spacing` を **non-nullable** で受け取る
- [ ] 4.3 host (KDV) preset を必須で渡す前提を docs に明記する

---

## 5. Host 制御 API を公開する

- [ ] 5.1 `EditorScrollControl`（`scroll_to_line` / `scroll_to_offset` / `scroll_by_pixels` / `visible_range` / `scroll_into_view`）trait を neutral crate に定義する
- [ ] 5.2 `EditorWriteAccess`（`insert_at` / `replace_range` / `set_text` / `apply_batch` / `with_origin(tag)`）trait を neutral crate に定義する
- [ ] 5.3 `EditorViewControl`（`set_read_only` / `is_read_only` / `focus` / `blur` / `is_focused`）trait を neutral crate に定義する
- [ ] 5.4 `EditorHistoryControl`（`undo` / `redo` / `can_undo` / `can_redo` / `push_history_barrier`）trait を定義する
- [ ] 5.5 `EditorSelectionControl`（`set_cursor` / `set_selection` / `selections`）trait を定義する
- [ ] 5.6 `EditorSearchControl`（`find` / `find_next` / `replace` / `replace_all`、`case_sensitive` / `regex` / `whole_word` フラグ）trait を定義する
- [ ] 5.7 `EditorClipboardControl`（`cut` / `copy` / `paste`、host が `ClipboardBackend` を注入できる形）trait を定義する
- [ ] 5.8 `EditorDiagnosticsSink` / `EditorDecorationsSink`（LSP 風 diagnostics、inline decoration、gutter marker、hover hint を host が push できる形）trait を定義する
- [ ] 5.9 `EditorAccessibility`（screen reader role / label、direction LTR・RTL、reduced motion）trait を定義する
- [ ] 5.10 egui 実装で部分対応のものは `EditorError::Unsupported` を返し、Floem 実装で完成させる旨を docs に明記する

---

## 6. 設定 IF を定義する

- [ ] 6.1 `EditorSettings { autosave: AutosavePolicy, shortcuts: ShortcutMap, word_wrap, tab_size, line_numbers, font_scale, .. }` を neutral crate に定義する
- [ ] 6.2 `AutosavePolicy { enabled: bool, interval_ms: Option<u64> }` を定義し、`enabled = true` 時のみ `interval_ms` が意味を持つ contract を docs に明記する
- [ ] 6.3 `ShortcutMap` は host から既定キーバインドを上書きできる形にする（Save / Find / Replace / Undo / Redo / Toggle Comment 等の semantic action key を介する）
- [ ] 6.4 `EditorConfig::settings: EditorSettings` を **non-nullable** で受け取る
- [ ] 6.5 settings 更新を runtime に反映する `apply_settings(&mut self, new: EditorSettings)` API を定義する

---

## 7. ast-lint 拡張: 色リテラル禁止ルール

- [ ] 7.1 `kle-linter` に `prohibited-color-literal` ルールを追加する
  - 検出対象（最低限）: `egui::Color32::*`、`#xxx` / `#xxxxxx` / `rgb(...)` / `rgba(...)` 文字列リテラル、`Color::rgb(...)` 系コンストラクタ、Floem 移行後の `floem::peniko::Color` リテラル
  - 例外: `kdv-presets` 配下、テストコード、明示的 `#[allow(kle_lint::prohibited_color_literal)]`
- [ ] 7.2 `crates/katana-language-editor*` 配下に新ルールを適用し、違反ゼロにする
- [ ] 7.3 `kle-linter` の CI が新ルール込みで通ることを確認する

---

## 8. Floem 移行準備（git dependency 固定）

- [x] 8.1 `crates/katana-language-editor-floem/` skeleton crate を新規追加し、workspace member に加える
- [x] 8.2 ルート `Cargo.toml` の `[workspace.dependencies]` に `floem = { git = "https://github.com/lapce/floem", rev = "<pinned-sha>" }` を追加し、**crates.io の floem は使用しない**ルールを docs / project.md に明文化する
- [x] 8.3 skeleton crate は neutral 契約がコンパイルできることだけを保証し、本実装は v0.2.x で行う旨を README に明記する
- [x] 8.4 `cargo tree -p katana-language-editor-floem | grep floem` が git source を指していることを確認する

---

## 9. KatanA editor 実装を移管する（egui MVP）

- [ ] 9.1 KatanA `katana-ui` の editor コードを `katana-language-editor-egui` へ移管する
- [ ] 9.2 `EditorConfig` 経由の設定注入を確立し、`katana-ui` のグローバル設定依存を排除する
- [ ] 9.3 絵文字フォント管理（macOS / Windows プラットフォーム差異）を egui crate へカプセル化する
- [ ] 9.4 シンタックスハイライトを egui crate へ移管する（Mermaid / Draw.io block のハイライトは構文認識のみ、描画は kcf 経由）
- [ ] 9.5 egui crate 内に色リテラルが残らないことを `kle-linter` で検証する

---

## 10. v0.1.0 release

- [ ] 10.1 `cargo fmt` / `cargo clippy --workspace -- -D warnings` / `cargo test --workspace` が通る
- [ ] 10.2 `kle-linter` の `prohibited-color-literal` を含む全ルールで違反ゼロ
- [ ] 10.3 release tag `v0.1.0` を切り GitHub Release を作成する
- [ ] 10.4 KatanA v0.27.0 が `katana-language-editor = { git = "...", tag = "v0.1.0" }` でビルドできることを確認する
- [ ] 10.5 KDV 側に theme / i18n / typography / settings の preset がそろっていない場合は KDV 側に Issue を起票し、v0.1.0 リリース後すぐ整備する
