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

#### Scenario: clipboard backend を host が注入する

- **WHEN** host が `ClipboardBackend` trait の実装を `EditorConfig` に注入する
- **THEN** editor の cut / copy / paste は host 提供 backend を経由する
- **THEN** editor は OS clipboard を直接触らない（テスタビリティ確保）

### Requirement: diagnostics / decorations を host から注入できる sink を提供しなければならない

`katana-language-editor` neutral crate は、`EditorDiagnosticsSink` / `EditorDecorationsSink` trait を提供する（MUST）。LSP 風 diagnostics（range + severity + message + code）、inline decoration（text-range に対する装飾）、gutter marker（行に対するアイコン/色）、hover hint を host が push できる。

#### Scenario: host が KMM unresolved target を decoration で表示する

- **WHEN** host が KMM metadata sync 結果から unresolved target を取得し、editor に decoration を push する
- **THEN** editor は対応する行にマーカーを描画する
- **THEN** decoration は editor 内部状態と分離されており、host が任意に clear / replace できる

### Requirement: accessibility（screen reader semantics, direction, reduced motion）を公開しなければならない

`katana-language-editor` neutral crate は `EditorAccessibility` trait を提供し、host が role / label / direction (LTR/RTL) / reduced motion を制御できるようにする（MUST）。

#### Scenario: host が screen reader label を上書きする

- **WHEN** host が `editor.set_accessibility(AccessibilityConfig { label: "Document body", .. })` を渡す
- **THEN** OS の accessibility tree に対象 label が反映される（実装によっては Unsupported 返却可）

### Requirement: egui MVP で未対応の API は Unsupported を返して契約を満たさなければならない

`katana-language-editor-egui` は、上記の全 trait を実装するが、egui で実装困難な機能（precise pixel scroll offset、独自 decoration overlay、advanced clipboard backend、AccessKit 統合等）は `EditorError::Unsupported` を返して契約を満たす（MUST）。Floem 実装でこれらを完全対応する。

#### Scenario: egui 実装が Unsupported を返す

- **WHEN** host が egui 実装上で `EditorDecorationsSink::push_inline(...)` を呼ぶ
- **THEN** `Err(EditorError::Unsupported("inline decorations require Floem backend"))` が返る
- **THEN** host は Unsupported を検知して fallback できる
