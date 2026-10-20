## ADDED Requirements

### Requirement: 外部からスクロール制御を行える公開 API を提供しなければならない

`katana-language-editor` neutral crate は、host が editor を外部からスクロール制御するための `EditorScrollControl` trait を公開しなければならない（MUST）。最低限 `scroll_to_line(line: usize)` / `scroll_to_offset(offset: usize)` / `scroll_by_pixels(dx: f32, dy: f32)` / `scroll_into_view(range: Range)` / `visible_range() -> Range` を含む。

#### Scenario: host が editor を特定行までスクロールする

- **WHEN** host が `editor.scroll_to_line(42)` を呼ぶ
- **THEN** editor は行 42 が viewport に入るようにスクロールする
- **THEN** API は egui / Floem の実装型を引数に取らない（UI フレームワーク非依存）

#### Scenario: viewport 情報を取得する

- **WHEN** host が `editor.visible_range()` を呼ぶ
- **THEN** 現在表示中の `Range { start, end }`（行または文字オフセット）が返る
- **THEN** これは scroll synchronization の判断材料として host が使える

#### Scenario: gutter line click を host action として扱う

- **WHEN** editor gutter の行番号が click される
- **THEN** editor は `line_index` と表示用 `line_number` を分けた gutter line state を保持する
- **THEN** click は `ActivateGutterLine { line_index }` の `EditorActionRequest` として host に返る
- **THEN** host はこの action を preview/editor scroll synchronization に接続できる

#### Scenario: preview/editor scroll source と navigation intent を host action として扱う

- **WHEN** editor または preview が同期 scroll の source になる
- **THEN** editor は `EditorScrollSource` と `EditorLogicalScrollPosition` を含む `SyncScroll` action を発火できる
- **WHEN** host が select-and-jump を要求する
- **THEN** editor は `SelectAndJump` request に `preview_only_fallback_view_mode = CodeOnly` を保持し、対象行を scroll target として保持する
- **AND** host は現在の view mode が PreviewOnly の場合だけ fallback view mode を適用し、Split / CodeOnly は維持する
- **WHEN** document search next/previous が対象行へ移動する
- **THEN** editor は `NavigateDocumentSearch` request に `reset_last_scroll_target` と `preview_scroll_fallback` を保持する
- **THEN** Storybook contract はこれらの navigation action callback を検証する

### Requirement: 外部から editor 書き込みを行える公開 API を提供しなければならない

`katana-language-editor` neutral crate は、host が editor の本文を外部から書き換えるための `EditorWriteAccess` trait を公開しなければならない（MUST）。最低限 `insert_at(pos, text)` / `replace_range(range, text)` / `set_text(text)` / `apply_batch(edits)` / `with_origin(tag) -> WriteHandle` を含む。

#### Scenario: host が定型テキストを挿入する

- **WHEN** host が `editor.insert_at(pos, "## TODO\n")` を呼ぶ
- **THEN** editor は指定位置にテキストを挿入する
- **THEN** 挿入は通常のユーザー編集と同じく history / autosave / diagnostics 配信パイプラインを通る

#### Scenario: バッチ書き込みは単一の undo unit になる

- **WHEN** host が `apply_batch(vec![edit1, edit2, edit3])` を呼ぶ
- **THEN** 3 つの編集は 1 つの undo unit にまとまる
- **THEN** `with_origin("kme-resolver")` 等の origin タグを付けると、後で host がその編集を識別できる

#### Scenario: document state update は KatanA の dirty / refresh side effect を返す

- **WHEN** host が `EditorDocumentUpdate` を適用する
- **THEN** editor は `dirty`、`preview_refresh_required`、`search_refresh_required`、`diagnostics_refresh_required` を含む `EditorDocumentUpdateReport` を返す
- **THEN** virtual document は content が変わっても dirty にしない
- **WHEN** host が external change を適用する
- **THEN** editor は workspace/document scoped identity を持つ external undo record を pending として保持する
- **WHEN** host が save 完了を通知する
- **THEN** editor は dirty を clear し、diagnostics refresh を必要 side effect として返す

#### Scenario: 書き込み API は read-only モード時に拒否される

- **WHEN** editor が `read_only = true` の状態で `insert_at` を呼ぶ
- **THEN** `Result<_, EditorError::ReadOnly>` が返る

### Requirement: read-only / focus 制御 API を提供しなければならない

`katana-language-editor` neutral crate は、`EditorViewControl` trait を提供する（MUST）。最低限 `set_read_only(bool)` / `is_read_only()` / `focus()` / `blur()` / `is_focused()` を含む。

#### Scenario: host が viewer モードに切り替える

- **WHEN** host が `editor.set_read_only(true)` を呼ぶ
- **THEN** editor は以後のキー入力・書き込み API 呼び出しを拒否する
- **THEN** UI 上にカーソル選択は許可するが文字入力できない

### Requirement: undo / redo / selection / search / clipboard 公開 API を提供しなければならない

`katana-language-editor` neutral crate は、`EditorHistoryControl` / `EditorSelectionControl` / `EditorSearchControl` / `EditorClipboardControl` trait を提供する（MUST）。

#### Scenario: host が programmatic に undo する

- **WHEN** host が `editor.undo()` を呼ぶ
- **THEN** 直前の編集が取り消される
- **THEN** `can_undo()` の戻り値が更新される

#### Scenario: host が選択を programmatic に設定する

- **WHEN** host が `editor.set_selection(Selection { start, end })` を呼ぶ
- **THEN** caret と selection が指定位置に移動する

#### Scenario: authoring transform 後に pending cursor restore を適用する

- **WHEN** host が Markdown authoring などの外部 transform 後に `queue_cursor_restore(range)` を呼ぶ
- **THEN** editor は range を char-index selection として保持する
- **THEN** `apply_pending_cursor_restore()` または次の editor cycle で cursor / selection を復元し、selection change event を発火する

#### Scenario: clipboard backend を host が注入する

- **WHEN** host が `ClipboardBackend` trait の実装を `EditorConfig` に注入する
- **THEN** editor の programmatic cut / copy と headless text paste は host 提供 backend を経由する
- **THEN** editor は OS clipboard を直接触らない（テスタビリティ確保）

### Requirement: actual paste は controlled host resolution でなければならない

`katana-language-editor` MUST request actual UI paste as an
`EditorClipboardPasteRequest` event without reading the OS clipboard. The
request carries an opaque token, document identity, char-index `TextRange`, and
the requested `TextContent` snapshot.

#### Scenario: host が text paste を解決する

- **WHEN** KUC から editable な `ClipboardRequested(Paste)` が届く
- **THEN** KLE は本文を変更せず、token 付き `EditorClipboardPasteRequested` を一度だけ emit する
- **WHEN** host が同じ token の `Text` resolution を返す
- **THEN** editor は要求時 selection を一度だけ置換し、一つの content event を emit する

#### Scenario: host が image または失敗を解決する

- **WHEN** host が同じ token を `Image` として解決する
- **THEN** editor は本文を変更せず、`IngestClipboardImage` action request を一度だけ emit する
- **THEN** host は画像保存と Markdown 挿入後に `EditorDocumentUpdate::host_external_change` を適用する
- **WHEN** host が `NoPayload` または `Failed` として解決する
- **THEN** editor は本文、dirty state、content event を変更しない

#### Scenario: paste resolution が stale または重複する

- **WHEN** token、document identity、要求時 content snapshot、read-only 状態が一致しない、または token が既に解決済みである
- **THEN** editor は resolution を拒否し、本文、dirty state、content event、image action request を変更しない

### Requirement: diagnostics / decorations を host から注入できる sink を提供しなければならない

`katana-language-editor` neutral crate は、`EditorDiagnosticsSink` / `EditorDecorationsSink` trait を提供する（MUST）。LSP 風 diagnostics（range + severity + message + code）、inline decoration（text-range に対する装飾）、gutter marker（行に対するアイコン/色）、hover hint を host が push できる。

#### Scenario: host が KMM unresolved target を decoration で表示する

- **WHEN** host が KMM metadata sync 結果から unresolved target を取得し、editor に decoration を push する
- **THEN** editor は対応する行にマーカーを描画する
- **THEN** decoration は editor 内部状態と分離されており、host が任意に clear / replace できる

#### Scenario: host が診断 popup と fix/docs action を接続する

- **WHEN** host が `diagnostic_popup_for_gutter_line(line_number)` を呼ぶ
- **THEN** editor は該当 gutter line から診断 popup item を返す
- **THEN** fix、fix-all、docs は `EditorActionRequest` として host adapter に戻せる
- **THEN** linter 実行、差分 review、外部 docs 表示の実行責務は KatanA host 側に残る

### Requirement: accessibility（screen reader semantics, direction, reduced motion）を公開しなければならない

`katana-language-editor` neutral crate は `EditorAccessibility` trait を提供し、host が role / label / direction (LTR/RTL) / reduced motion を制御できるようにする（MUST）。

#### Scenario: host が screen reader label を上書きする

- **WHEN** host が `editor.set_accessibility(AccessibilityConfig { label: "Document body", .. })` を渡す
- **THEN** 同一 KUC frame の OS accessibility tree に対象 label が反映される

### Requirement: v0.1.0 target API は Unsupported を返してはならない

`katana-language-editor-egui` は、上記の全 v0.1.0 target trait を KUC generic contract と host typed request に接続しなければならない（MUST）。precise pixel scroll offset、decoration overlay、clipboard request、AccessKit 統合等を `EditorError::Unsupported` に置換してはならない（MUST NOT）。OS clipboard/image acquisition と file IO は KatanA host に残し、KLE/KUC は token 付き request/result を扱う。

#### Scenario: host control is completed through the actual path

- **WHEN** host が `EditorDecorationsSink::push_inline(...)` を呼ぶ
- **THEN** KUC frame record、paint/accessibility artifact、typed editor event が更新される
- **THEN** unresolved KUC support は release blocker として fail-closed になり、fallback へ切り替わらない
