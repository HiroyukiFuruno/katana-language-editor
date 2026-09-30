# KLE v0.1.0 KatanA Editor Full Parity Audit

## Scope Correction

KLE v0.1.0 first release targets every function currently provided by the KatanA editor area, not only the current egui MVP text-edit subset. A passing KLE release gate is not enough unless each KatanA editor feature below has contract, implementation, Storybook, and downstream KatanA integration evidence.

2026-07-11 の独立再監査で、従来の完了表記には次の欠落が判明した。`tools/kle-storybook` の motion artifact は `EguiLanguageEditor::show` を実行する一方、動画 pixels を別の `StorybookFallbackRenderer` から生成していたため live editor の視覚根拠ではない。これは KUC migration 前の履歴であり、2026-08-13 時点では KUC-owned TextSurface/egui adapter、CommandChrome、SVG/text raster、ContextMenu の component contract は実装・実入力検証済みである。ただし opaque composited root artifact は未完成であり、KLE の `PlatformTextSurface` / `LineGutterModel` / fallback Storybook も残る。KLE thin binding、実エディター Storybook、KLE-owned actual `KatanaApp` host E2E が未完了である。さらに syntax highlighting、text clipboard、shortcut、undo/redo、AccessKit、document replace UI、native input は独立 feature evidence が不足し、`editor/rendering.rs` の test は KLE runnable host-E2E target に登録されていない。以下の既存行は KLE/KatanA integration と release blocker が解消されるまで provisional であり、v0.1.0 parity は未達である。

2026-08-21 の固定参照 `4f6a6287c650a38633c7baeb544a92e739c68567` 再監査では、上記の matrix を release 判定に使うこと自体が不十分であると確定した。`clipboard-text` は cut/copy/paste と payload-none/error/read-only を個別 leaf にする。shortcut は Recording > Modal > Editor > Preview > Global、text-entry preservation、modifier specificity、first-match dispatch を個別 leaf にする。Markdown authoring は 14 操作、toolbar は 12 trigger、code block は 17 kind、image ingest は file/clipboard-image/file-URL と payload/error/read-only を個別 leaf にする。search の Enter/Shift+Enter/Escape、selection restore、authoring 後の undo/refresh、syntax layout の実到達性、diagnostics/gutter の row/hover/fix/docs、AccessKit と IME の日本語・ZWJ・exact `⭐️` VS16 も、各々 source branch、KUC root record、actual input、actual KatanA host effect を結合しなければならない。生成コード、source marker、Storybook callback、画面、動画、aggregate count はいずれもこの結合の代替ではない。2026-08-26 に detached fixed source tree から `macos-latest` の source-closure capture は生成したが、Windows/Linux の native capture と 3 profile の provenance/assembly が未取得であるため、leaf coverage は引き続き未証明である。

## Source Of Truth

The authoritative reference is
`docs/v0-1-0-katana-editor-source-universe.md` at KatanA revision
`4f6a6287c650a38633c7baeb544a92e739c68567`.  Its generated source closure,
not this document's historical table or a fixed file count, defines the source
universe.  This audit maps the generated source branches to feature and leaf
requirements; it must not retain a nonexistent source file as a deferred
adapter or invent a KatanA test target.

The source-of-truth paths used by the parity checker are
`katana-ui/src/views/panels/editor/**`,
`katana-ui/tests/integration/editor/**`,
`katana-ui/src/app/action/process_authoring.rs`,
`katana-ui/src/app/document_edit.rs`,
`katana-ui/src/app/action/image_ingest.rs`,
`katana-ui/src/app/action/dispatch_secondary.rs`, and
`katana-ui/src/editor_undo.rs`. These paths are reference-only evidence; the
fixed-revision checkout is not modified; historical planned path is not evidence.
KLE/KatanA integration remains an explicit blocker until a
KLE-owned actual `KatanaApp` host-E2E target proves the host effect.

### Re-audit additions (2026-08-13)

The capability inventory additionally requires: Markdown visible-text/code-only
search with URL/HTML-attribute exclusion; search focus and
Enter/Shift+Enter/Up/Down/Escape transitions; image-payload precedence over
text paste with payload-none/error no-mutation; official diagnostic
start-line-only, same-line aggregation, and non-fixable display; and
context-menu enablement for editable Markdown format, inline selection, and
clipboard-image payload availability. These are KatanA host semantics and are
not satisfied by a KLE control being merely visible. The parity gate must retain
each as an individual actual-input plus host-transition requirement.

## Evidence Revalidation (2026-08-13)

KatanA reference checkout を read-only で再検証したところ、`master` は clean だが
`katana-language-editor` / `katana-language-editor-egui` の dependency、
`crates/katana-ui/tests/integration/editor/kle_downstream_adapter.rs`、
`editor_kle_downstream_adapter` test module は存在しなかった。したがって本書に残る
それらの path、test 名、又は「real/actual KatanA adapter passed」とする過去の記述は、
v0.1.0 の evidence として無効である。KLE-only downstream simulator、KatanA baseline
test、又は source inventory は、実 KatanA editor と KLE の接続証明へ昇格させない。

このスコープで KatanA は変更しない。KLE/KUC の移行が完了するまで、KLE は local
read-only `katana-ui` を path dependency とする actual host E2E で互換性を検証する。
KatanA 内 adapter test の存在は将来の adoption evidence であり、KLE v0.1.0 の
compatibility evidence と混同しない。以下の `Current authoritative status` が、後続の
historical prose より優先する。

## Capability Evidence Quality Re-audit (2026-08-13)

KatanA editor source を read-only で追跡し、既存の 26-row matrix を feature 名だけで
閉じないことを確認した。v0.1.0 の全機能要件には、source path/line、KUC/KLE/KatanA の
所有者、KLE public actual-input evidence、KLE-owned actual KatanA host E2E evidence を
一対一で記録する capability manifest が必要である。source file、KLE-only contract、
Storybook、KatanA baseline test、又は将来作る予定の KatanA test 名の存在だけでは、該当行を
完了にできない。

| Capability group | KatanA source evidence | Required final evidence | Current gap |
| --- | --- | --- | --- |
| multiline/read-only, selection/cursor restore, IME | `editor/text_edit.rs`, `app/action/process_authoring.rs` | Japanese/`⭐️` VS16/ZWJ actual input through KUC, KLE, and KLE-owned actual `KatanaApp` buffer state | KLE-owned actual `KatanaApp` host E2E absent |
| gutter, diagnostics, accessibility | `editor/decorations.rs`, `editor/row_diagnostics.rs`, `editor/diagnostics_ui.rs` | actual pointer/hover/focus/AccessKit/artifact plus KatanA fix-review routing | KLE generic renderer and missing KLE host-E2E evidence |
| floating toolbar and code menu | `editor/toolbar.rs`, `editor/toolbar_popup.rs`, `editor/code_block_menu.rs` | 5 groups / 12 toolbar triggers (inline 4, heading 3, list-quote 3, code dropdown 1, image 1), 17 code kinds, actual pointer/keyboard/disabled/outside/focus/placement evidence | KUC generic runtime is partial; KLE typed binding, all action evidence, Storybook, opaque root artifact, and KLE-owned host E2E remain incomplete |
| context menu and ingest | `editor/context_menu.rs`, `context_menu_image_ingest.rs`, `editor/paste.rs`, `app/action/image_ingest.rs` | save, editable Markdown format condition, 14 authoring actions, file/clipboard image, file URL paste, disabled/payload-none cases | KUC component behavior has partial evidence; KLE injected tree/event binding, Storybook, opaque root artifact, and KLE-owned host E2E are absent |
| find/navigation/highlight | `views/top_bar/search.rs`, `app/doc_search.rs` | query/count/highlight/previous/next/wrap/close with actual editor scroll and KatanA host state | KLE/KatanA integration absent |
| replace/replace-all | **user explicit v0.1.0 requirement**; editor-surface source root was not found in this read-only audit | visible KUC CommandChrome replace controls, content effect, actual input/AccessKit/artifact, KLE-owned actual `KatanaApp` host E2E | must remain an additional mandatory requirement, not be mislabeled as confirmed KatanA source parity |
| lifecycle/history/shortcuts/syntax | `app/document_edit.rs`, `editor_undo.rs`, `editor/layout.rs` | dirty/save/format/refresh/scoped undo, clipboard and shortcut/read-only conflict, injected syntax layout through KLE-owned actual `KatanaApp` host E2E | source-only or KLE-only evidence remains |

The current matrix includes command strings for historical
`editor_kle_downstream_adapter` tests that do not exist in the reference
checkout. They are rejected historical residue. The future manifest gate must
reject them and instead resolve each leaf to a KLE-owned host-E2E target that
path-depends on the unchanged reference checkout. Missing KLE-owned execution
or an unresolvable reference source branch is a release blocker, not a skipped
check.

`find-replace-ui` is retained because the user explicitly required text
replacement and replace-all in the full Storybook/editor surface. The current
read-only source audit found confirmed find/navigation/highlight behavior but
did not find a generic editor replace UI. This is therefore classified as
"user-mandated additional capability" until a KatanA source/test reference is
identified; it must still receive the same KUC/KLE/KatanA runnable evidence and
may not be deleted to make the parity matrix pass.

### Exact control and runner correction (2026-08-13)

The required capability manifest keeps the 26 feature groups, but groups with
enumerable user controls must contain strict leaf cases. KatanA
`editor/toolbar.rs` contains five groups and 12 clickable toolbar triggers:
inline 4, heading 3, list/quote 3, one code dropdown trigger, and one image
trigger. The context-menu authoring subtree contains 14 top-level authoring
triggers, its code submenu contains 17 `CodeBlockKind` values, and image ingest
has three distinct routes: file, clipboard image payload, and clipboard file
URL paste. A check that only counts a generic "toolbar" or "image ingest" row
does not establish parity.

The only confirmed integration registration in the reference checkout is the
direct `#[path = "integration/editor/ui.rs"] mod editor_ui;` declaration in
`crates/katana-ui/tests/ui_integration_parallel.rs`. The historical
`editor_kle_downstream_adapter` module is absent, and modules merely reachable
from `tests/integration/editor/mod.rs` are not runnable evidence until the
actual integration target registers them. Accordingly, the existing
KLE-only simulator and KatanA baseline tests remain diagnostic evidence only;
they cannot satisfy a KLE-owned actual `KatanaApp` host-E2E leaf case.

## Source Inventory Feature IDs

- `markdown-authoring-ops`
- `code-block-menu`
- `authoring-toolbar`
- `context-menu-actions`
- `image-ingest`
- `line-gutter`
- `diagnostics-problems`
- `view-modes`
- `scroll-sync`
- `text-editing`
- `select-and-jump`
- `kuc-text-input`
- `document-search`
- `dirty-save-refresh`
- `cursor-selection-restore`
- `document-scoped-undo`
- `multi-document-state`
- `syntax-highlighting`
- `clipboard-text`
- `editor-shortcuts`
- `native-text-input`
- `command-chrome`
- `text-surface`
- `editing-history`
- `accessibility-tree`
- `find-replace-ui`

## Source Inventory

| File | Mapped requirement rows | Integration function(s) | Notes |
| --- | --- | --- | --- |
| `crates/katana-ui/src/views/panels/editor/authoring.rs` | `markdown-authoring-ops` | - | authoring command inventory and toolbar/menu action source |
| `crates/katana-ui/src/views/panels/editor/authoring_tests.rs` | `markdown-authoring-ops` | - | authoring contract tests |
| `crates/katana-ui/src/views/panels/editor/authoring_utils.rs` | `markdown-authoring-ops` | - | markdown transform + toolbar helper utilities |
| `crates/katana-ui/src/views/panels/editor/code_block_menu.rs` | `code-block-menu`, `authoring-toolbar`, `command-chrome` | - | code block kind menu lifecycle |
| `crates/katana-ui/src/views/panels/editor/context_menu.rs` | `context-menu-actions` | - | editor context-menu action contract |
| `crates/katana-ui/src/views/panels/editor/context_menu_image_ingest.rs` | `context-menu-actions`, `image-ingest` | - | image ingest action sources |
| `crates/katana-ui/src/views/panels/editor/context_menu_tests.rs` | `context-menu-actions`, `image-ingest`, `markdown-authoring-ops` | - | context menu contract tests |
| `crates/katana-ui/src/views/panels/editor/decorations.rs` | `line-gutter`, `diagnostics-problems`, `text-surface` | - | gutter decorations and diagnostics markers |
| `crates/katana-ui/src/views/panels/editor/diagnostics_hover.rs` | `diagnostics-problems` | `show_hover_ui`, `show_single_diagnostic_ui` | same-line aggregation and fix/fix-all presentation; source evidence only until the KLE/KatanA host-effect route exists |
| `crates/katana-ui/src/views/panels/editor/diagnostics_popup.rs` | `diagnostics-problems` | `show`, `show_content` | diagnostic popup action model and popup lifecycle; source evidence only |
| `crates/katana-ui/src/views/panels/editor/diagnostics_ui.rs` | `diagnostics-problems` | `render_diagnostics` | diagnostics sink/event conversion; source evidence only |
| `crates/katana-ui/src/views/panels/editor/layout.rs` | `view-modes`, `line-gutter`, `scroll-sync`, `syntax-highlighting` | - | pane and split controls; syntax-layout definitions remain reachability-held until generated closure resolves their actual editor route |
| `crates/katana-ui/src/views/panels/editor/line_numbers.rs` | `line-gutter`, `text-surface` | - | line number and gutter interactions |
| `crates/katana-ui/src/views/panels/editor/logic.rs` | `text-editing`, `scroll-sync`, `select-and-jump` | - | editor core request/state logic |
| `crates/katana-ui/src/views/panels/editor/logic_colors.rs` | `kuc-text-input`, `text-editing` | - | color/token mapping and style contract support |
| `crates/katana-ui/src/views/panels/editor/logic_scroll.rs` | `scroll-sync`, `select-and-jump` | - | scroll source and scroll request mapping |
| `crates/katana-ui/src/views/panels/editor/logic_scroll_tests.rs` | `scroll-sync` | - | scroll logic test fixture |
| `crates/katana-ui/src/views/panels/editor/logic_tests.rs` | `text-editing`, `kuc-text-input`, `editor-shortcuts` | - | editor unit behavior and shortcut coverage |
| `crates/katana-ui/src/views/panels/editor/mod.rs` | `text-editing` | - | editor panel wiring for required features |
| `crates/katana-ui/src/views/panels/editor/paste.rs` | `image-ingest`, `clipboard-text` | - | clipboard text/image paste entry points |
| `crates/katana-ui/src/views/panels/editor/row_diagnostics.rs` | `diagnostics-problems` | - | gutter-line diagnostics items |
| `crates/katana-ui/src/views/panels/editor/syntax.rs` | `syntax-highlighting` | - | injected syntax highlighter integration points |
| `crates/katana-ui/src/views/panels/editor/text_edit.rs` | `text-editing`, `document-search`, `dirty-save-refresh`, `clipboard-text`, `native-text-input`, `text-surface`, `editing-history`, `accessibility-tree` | - | text edit, input, and clipboard state |
| `crates/katana-ui/src/views/panels/editor/toolbar.rs` | `authoring-toolbar`, `context-menu-actions`, `command-chrome` | - | toolbar command dispatch |
| `crates/katana-ui/src/views/panels/editor/toolbar_popup.rs` | `authoring-toolbar`, `command-chrome` | - | authoring popup anchor + lifecycle |
| `crates/katana-ui/src/views/panels/editor/types.rs` | `text-editing`, `cursor-selection-restore` | - | state transfer data structures |
| `crates/katana-ui/src/views/panels/editor/ui.rs` | `cursor-selection-restore`, `authoring-toolbar`, `code-block-menu`, `image-ingest`, `context-menu-actions`, `text-surface`, `accessibility-tree` | - | host-facing UI event wiring |
| `crates/katana-ui/src/views/panels/editor/utils.rs` | `text-editing`, `cursor-selection-restore` | - | editor utility helpers |
| `crates/katana-ui/tests/integration/editor/layout_persistence.rs` | `view-modes`, `scroll-sync` | `test_regression_preview_content_visible_in_preview_only_mode`, `test_regression_preview_content_visible_in_split_mode`, `test_split_direction_setting_toggles_correctly` | view-split persistence integration |
| `crates/katana-ui/tests/integration/editor/lint_fix_review_button.rs` | `diagnostics-problems` | `editor_diagnostic_fix_button_opens_lint_fix_review_tab`, `lint_fix_review_tab_shows_cancel_all_button`, `problems_fix_all_detected_opens_review_for_every_open_tab`, `problems_fix_all_keeps_unloaded_open_tab_in_review`, `problems_status_count_follows_scope_only_while_panel_open` | lint-fix review and Problems scope/state behavior |
| `crates/katana-ui/tests/integration/editor/mod.rs` | `kuc-text-input`, `text-editing` | - | integration module registration |
| `crates/katana-ui/tests/integration/editor/navigation.rs` | `multi-document-state`, `select-and-jump` | `test_integration_workspace_and_tabs_navigation`, `test_integration_open_multiple_documents_and_switch` | multi-document + jump behavior |
| `crates/katana-ui/tests/integration/editor/rendering.rs` | `line-gutter`, `dirty-save-refresh`, `document-search`, `syntax-highlighting` | `test_integration_editor_line_numbers_and_highlight`, `test_integration_update_buffer`, `test_integration_save_document`, `test_integration_text_edit_triggers_update_buffer` | source-only until registered in a runnable KatanA test target |
| `crates/katana-ui/tests/integration/editor/toggle_view_modes.rs` | `view-modes` | `test_integration_toggle_view_modes` | split direction + mode transitions |
| `crates/katana-ui/tests/integration/editor/ui.rs` | `view-modes`, `code-block-menu`, `image-ingest`, `line-gutter`, `accessibility-tree` | `click_code_editor_input`, `test_integration_view_modes`, `clipboard_image_file_url_paste_queues_image_ingest_action`, `code_block_kind_menu_closes_when_editor_is_clicked`, `live_clipboard_image_shortcut_inserts_markdown_from_current_os_clipboard`, `test_integration_editor_line_numbers_visibility`, `test_integration_update_buffer` | core integration wrapper source; `accesskit::Role` query is a decisive baseline accessibility anchor, not KLE adapter evidence |
| `crates/katana-ui/tests/integration/editor/view_modes.rs` | `view-modes`, `select-and-jump` | `test_integration_split_mode_with_document` | view mode integration cases |
| `crates/katana-ui/src/app/doc_search.rs` | `document-search` | - | document search behavior and target-line search contracts |
| `crates/katana-ui/src/app/action/process_authoring.rs` | `markdown-authoring-ops`, `cursor-selection-restore`, `authoring-toolbar` | - | authoring action adapters |
| `crates/katana-ui/src/app/document_edit.rs` | `text-editing`, `dirty-save-refresh`, `document-search` | - | document state, save and dirty integration |
| `crates/katana-ui/src/app/action/image_ingest.rs` | `image-ingest` | - | image ingest host actions |
| `crates/katana-ui/src/app/action/dispatch_secondary.rs` | `view-modes`, `scroll-sync`, `select-and-jump` | - | secondary host action routing |
| `crates/katana-ui/src/app/action/clipboard_image.rs` | `image-ingest`, `clipboard-text` | `has_image_payload`, `read_image_payload` | host-only acquisition priority begins with raw image then file-list; KLE/KUC must only request and resolve a controlled transaction |
| `crates/katana-ui/src/app/action/clipboard_file_url.rs` | `image-ingest`, `clipboard-text` | `read_image_payload` | host-only supported `file://` image acquisition; it is not a visible ContextMenu item |
| `crates/katana-ui/src/app/action/clipboard_image_macos.rs` | `image-ingest` | `macos_pasteboard_has_image`, `read_macos_pasteboard_image` | conditional macOS host fallback; neither KUC nor KLE may own it |
| `crates/katana-ui/src/app/action/process_markdown_formatting.rs` | `dirty-save-refresh`, `document-scoped-undo` | `handle_action_format_markdown_file`, `refresh_after_format` | host format/file IO, external scoped undo and refresh fan-out; source evidence only |
| `crates/katana-ui/src/app/action/refresh_content.rs` | `dirty-save-refresh`, `document-search`, `diagnostics-problems` | `apply_refreshed_content`, `handle_action_refresh_document` | host dirty-refresh, preview/search/diagnostics lifecycle; source evidence only |
| `crates/katana-ui/src/editor_undo.rs` | `document-scoped-undo`, `multi-document-state`, `editing-history` | - | workspace/document scoped undo + external change state |
| `crates/katana-ui/src/state/command_inventory/edit_commands.rs` | `editor-shortcuts`, `markdown-authoring-ops` | `get` | editable Markdown command availability and authoring shortcut inventory; source evidence only |
| `crates/katana-ui/src/state/shortcut_context.rs` | `editor-shortcuts`, `editing-history` | `resolve`, `context_allows` | Recording > Modal > Editor > Preview > Global context arbitration; source evidence only |
| `crates/katana-ui/src/shell_ui/shell_ui_shortcuts.rs` | `editor-shortcuts`, `editing-history`, `image-ingest` | `handle_shortcuts`, `command_shortcut_consumed` | text-entry preservation, modifier specificity and first-match dispatch; source evidence only |
| `crates/katana-ui/src/views/top_bar/search.rs` | `document-search`, `find-replace-ui` | - | query, previous/next, count, close controls; KLE also requires visible replace/replace-all |

## Required Feature Inventory

**Status classification:** 既存行の `Implemented` は、記載された KLE neutral contract と KatanA host action/state evidence が確認済みであることだけを表す。KUC `TextSurface` / shared egui adapter / `CommandChrome` / ContextMenu の generic component と real-egui interaction は完了しているが、KLE がそれらを薄い binding として実消費し、same-frame Storybook artifact、AST ownership guard、KLE-owned actual `KatanaApp` host E2E を満たすまで、UI parity/release readiness は未完了である。host behavior evidence と UI component evidence の片方だけで完全互換を主張してはならない。

### Current authoritative status

The prior inventory contains stale assertions about an actual KatanA adapter
file. They are superseded by this table; no row is release-complete.

| Requirement IDs | Current status | Missing evidence before it can become complete |
| --- | --- | --- |
| `text-editing`, `cursor-selection-restore`, `multi-document-state`, `view-modes`, `scroll-sync`, `select-and-jump`, `context-menu-actions`, `image-ingest`, `document-search`, `diagnostics-problems` | Partial | KLE must first consume KUC components; then real KLE component input, typed host-event mapping, and the KLE-owned actual `KatanaApp` E2E must be tested. An absent KatanA adapter file cannot be cited. |
| `line-gutter`, `dirty-save-refresh`, `kuc-text-input`, `text-surface`, `accessibility-tree` | Blocked | KUC controlled TextSurface/automatic gutter migration, KLE local renderer deletion, same-surface RawInput/AccessKit/artifact evidence, and runnable downstream parity evidence. |
| `authoring-toolbar`, `markdown-authoring-ops`, `code-block-menu`, `command-chrome`, `find-replace-ui` | Blocked | KUC controlled CommandChrome/search/self-measured floating placement, KLE thin event binding, local helper deletion, full actual-input Storybook, and downstream parity evidence. |
| `syntax-highlighting`, `clipboard-text`, `editor-shortcuts`, `native-text-input`, `editing-history` | Blocked | Explicit KLE feature implementation and real-input tests for the KatanA behavior; `Unsupported`, injected events, host hooks, and source-only tests are insufficient. |

| Feature | KatanA evidence | KLE v0.1.0 status |
| --- | --- | --- |
| Multiline text editing, changed-buffer action, read-only/reference edit blocking | `editor/text_edit.rs`, `editor/rendering.rs` | Blocked. KLE-owned actual `KatanaApp` host E2E is absent. Neutral contracts and Storybook do not prove real KatanA routing. |
| Workspace/document-scoped text edit identity and undo for external changes | `editor_undo.rs`, `app/document_edit.rs` | Blocked. KLE-owned actual `KatanaApp` host E2E is absent. |
| Cursor and selection capture, pending cursor restore after authoring transforms | `editor/ui.rs`, `editor/text_edit.rs`, `process_authoring.rs` | Blocked. Host hooks and injected selection are not actual KatanA/KLE host-E2E evidence. KatanA integration blocker: a KLE-owned actual `KatanaApp` host-E2E target is absent. |
| Line numbers, active line, hover line, clickable line numbers | `editor/line_numbers.rs`, `editor/decorations.rs`, `editor/rendering.rs`, `crates/katana-ui/tests/integration/editor/ui.rs` | Blocked. A KatanA baseline test exists, but the relevant `editor/rendering.rs` coverage is source-only and KLE still owns `LineGutterModel` instead of consuming the KUC TextSurface gutter. The runnable KLE/KUC/KatanA same-surface evidence is absent. |
| Diagnostic gutter icon, hover suppression, diagnostic popup, lint fix action | `editor/row_diagnostics.rs`, `editor/diagnostics_ui.rs`, `editor/diagnostics_popup.rs`, diff-review lint fix actions | Blocked. KLE/KUC contracts and downstream probes are not KLE-owned actual `KatanaApp` host-E2E evidence; the KatanA integration blocker remains because the actual host-E2E target is absent. |
| Authoring toolbar popup anchored to cursor and constrained to viewport | `editor/toolbar_popup.rs`, `editor/toolbar.rs`, `editor/ui.rs`, `crates/katana-ui/tests/integration/editor/ui.rs` | Blocked. KatanA baseline tests establish the reference behavior, but KLE currently renders its own egui `Area`/`Frame`/buttons through `authoring_helper.rs`. The KUC `FloatingCommandToolbar` adapter/artifact and KLE typed-event binding remain unimplemented. |
| Markdown authoring operations: bold, italic, strikethrough, inline code, headings, lists, blockquote, code block kinds, horizontal rule, link, table | `editor/authoring.rs`, `markdown_authoring_op.rs`, `editor/authoring_tests.rs` | Partial. KatanA host transformation inventory and expected buffer effects are reference evidence; the KLE/KUC command-surface binding that exposes every operation through actual input is not proven. |
| Code block menu lifecycle and close-on-editor-click behavior | `editor/code_block_menu.rs`, `editor/ui.rs` | Blocked. KatanA baseline menu behavior is known, but KLE has no KUC `CommandChromeToolbar` split-dropdown binding and actual shared-adapter pointer/keyboard/dismiss behavior is not proven. |
| Editor context menu: save, format markdown, edit actions, image ingest actions | `editor/context_menu.rs`, `context_menu_image_ingest.rs` | Blocked. KatanA baseline tests establish the reference behavior, but KLE-owned actual `KatanaApp` host E2E is absent. KLE contracts and Storybook callback intent are not live KatanA integration evidence. |
| Image ingest entry points from file, clipboard image, and clipboard file URL paste | `image_ingest.rs`, `editor/paste.rs`, `editor/ui.rs` | Blocked. Request callbacks and KLE-only probes are insufficient; KLE-owned actual `KatanaApp` host E2E is absent. |
| Document search match highlighting, next/previous navigation, scroll-to-line | `app/doc_search.rs`, `editor/text_edit.rs`, `editor/decorations.rs` | Blocked. Search contracts and target-line metadata are not live KLE host-E2E evidence; the KatanA integration blocker remains because the actual host-E2E target is absent. |
| Preview/editor scroll sync and source ownership | `editor/logic.rs`, `editor/logic_scroll.rs`, `preview/content.rs`, `layout/split.rs` | Blocked. KatanA baseline coordinator behavior is reference evidence only; KLE-owned actual `KatanaApp` host E2E is absent. |
| View modes: PreviewOnly, Split, CodeOnly; toggle cycle; split direction | `toggle_view_modes.rs`, `view_modes.rs`, `layout_persistence.rs`, `dispatch_secondary.rs` | Blocked. KatanA baseline view-mode tests are not KLE integration evidence. KLE-owned actual `KatanaApp` host E2E is absent. |
| Select document and jump to line applies PreviewOnly fallback when needed | `process_document.rs` | Blocked. KLE-owned actual `KatanaApp` host E2E is absent; host hooks and KLE request contracts are not live KatanA integration evidence. |
| Save, dirty state, preview refresh, diagnostics timestamp, doc search refresh after edits | `app/document_edit.rs`, `editor/rendering.rs` | Blocked. Host-side baseline behavior is recorded, but the `editor/rendering.rs` dirty/save coverage is source-only and no KLE-owned actual `KatanaApp` host-E2E target proves the complete path. |
| Multi-document tab navigation and scoped editor state | `navigation.rs`, `editor_undo.rs` | Blocked. KatanA baseline navigation remains reference evidence; KLE-owned actual `KatanaApp` host E2E is absent. |
| KUC-backed emoji/IME/text-area behavior and OS font fallback | KUC TextArea tests, KUC contract test suite (`kuc-contract-check`), KUC TextSurface artifact | Partial. KUC now proves actual platform-raster Japanese/`⭐️` text, IME, selection, gutter, and AccessKit on its TextSurface. KLE still uses a local surface and the Katana evidence does not use native KatanA input through the KUC-consumed KLE surface, so it cannot close the parity row. |
| Syntax highlighting through host injection | `editor/layout.rs`, `editor/syntax.rs`, `crates/katana-ui/tests/integration/editor/rendering.rs` | Reachability hold. The fixed KatanA revision contains syntax-related definitions, but the static audit has not resolved an actual editor call path from `TextEditRenderer` to either syntax route. No syntax-rendered parity, and no KLE-local highlighter, may be claimed until the generated closure classifies the path. |
| Text clipboard: copy, cut, paste, and error/read-only paths | `editor/paste.rs`, `editor/text_edit.rs`, `editor/context_menu.rs` | Partial. KLE exposes `ClipboardBackend`, but KatanA text clipboard behavior is not proven by KLE-owned actual `KatanaApp` host E2E; current evidence concentrates on image ingest. |
| Editor shortcut mapping and conflict rejection | `editor/logic_tests.rs`, `editor/context_menu.rs`, `editor/ui.rs` | Partial. KLE defines `ShortcutMap`, but KatanA command routing, duplicate binding rejection, and read-only behavior are not proven by runnable parity evidence. |
| Native multi-byte text input and composition | `editor/text_edit.rs`, `editor/ui.rs`, `process_authoring.rs` | Partial. Earlier KLE event-injection evidence includes `TextAreaEvent::ImeCommit("⭐️".to_string())` and `TextAreaEvent::Change("⭐️".to_string())`; that is not native input. Japanese/`⭐️` commit and selection restore are not proven through a runnable KatanA native input path. |
| Shared KUC TextSurface: visible text, caret/selection, IME, generic gutter/annotation, accessibility, same-frame artifact | KLE `platform_text_surface.rs`, `line_gutter.rs`; KUC `TextSurface`, shared egui adapter, platform text-raster; KatanA `editor/text_edit.rs` | Blocked. KUC TextSurface and its actual-egui/AccessKit/artifact path are implemented and independently tested. KLE consumption is unimplemented: `platform_text_surface.rs`, `LineGutterModel`, and local scroll behavior remain, and its fallback artifact remains disallowed. |
| Command chrome: cursor toolbar, code dropdown, icon SVG, find/replace controls | KatanA `toolbar_popup.rs`, `toolbar.rs`, `code_block_menu.rs`, `views/top_bar/search.rs`; KLE `authoring_helper.rs`; KUC command chrome/shared egui adapter | Blocked. KUC has generic toolbar/SVG raster support and an in-progress immutable paint plan, but floating toolbar, search/replace composition, artifact encoder, and the KLE typed-event binding are incomplete. The current KLE helper is a boundary violation. |
| Undo/redo and history shortcut behavior | KatanA `editor_undo.rs`, `TextEdit` default history, shortcut context; KLE `unsupported_controls.rs` | Partial. External format undo identity is proven, but KLE returns `Unsupported` for undo/redo and no actual KatanA keyboard/redo parity target exists. |
| Accessibility/AccessKit editor tree | KatanA editor UI integration and AccessKit assertions; KLE `unsupported_controls.rs`; KUC TextSurface accessibility model | Partial. KUC TextSurface publishes and tests an actual AccessKit bridge, but KLE does not map its editor accessibility state to that surface and still returns `Unsupported` for required controls. |
| Find/replace user interface | KatanA document-search source/highlights; KLE `EditorSearchControl`; KUC search strip | Partial. Find/navigation/highlight are covered; visible replace/replace-all and close UI must use KUC CommandChromeSearchStrip and real interaction tests. |

## Unresolved Parity Corrections

| Feature | Current evidence | Required completion evidence |
| --- | --- | --- |
| `syntax-highlighting` | KatanA `layout.rs` and `syntax.rs` contain syntax-related definitions, but the fixed-revision static audit has not established whether either contributes to the actual `TextEditRenderer` path. The KLE matrix has no independent verified feature. | Generated source closure must first classify the actual runtime/macro/extern path or prove the definitions unreachable with hash/span/rationale. Only a reachable branch may become a KUC span-record, KLE typed-binding, and KLE-owned actual `KatanaApp` host-E2E leaf. KLE must not add a local highlighter in either outcome. |
| `clipboard-text` | KLE exposes a `ClipboardBackend`, but the parity matrix has no feature row and the observed KatanA tests emphasize image paste. | KLE-owned actual `KatanaApp` host E2E must exercise text copy/cut/paste, empty/error clipboard paths, selection behavior, and read-only rejection. |
| `editor-shortcuts` | `ShortcutMap` is a KLE contract but no matrix feature proves KatanA shortcut routing or conflict rejection. | KLE-owned actual `KatanaApp` host E2E must cover map conflicts, command dispatch, and read-only/disabled behavior. |
| `editor-shortcuts` source inventory | KatanA `edit_commands.rs`, `shortcut_context.rs`, and `shell_ui_shortcuts.rs` define editable availability, Recording > Modal > Editor > Preview > Global priority, text-entry shortcut preservation, modifier specificity, and first-match dispatch, but none is a required source-inventory row. | Add all three source files and their decisive functions to `katana-parity-check`; KLE-owned actual `KatanaApp` host E2E must cover context priority, availability, modifier specificity, and no double dispatch. |
| `image-ingest` payload acquisition | The current three-leaf trigger inventory covers file, clipboard-image and file-URL entry points, but does not enumerate KatanA `clipboard_image.rs` raw image/file-list/file-URL/macOS acquisition, conversion, supported extension, payload-none/error, or read-only behavior. The current KLE host bridge also rejects external intent or ends at a pending dialog. | Classify all external leaves and implement the isolated native execution protocol in `v0-1-0-external-host-e2e-design.md`: actual KLE RawInput, real KatanA dialog/pasteboard acquisition, same-run asset/link/dirty/explorer or no-mutation assertion, clipboard restoration, and no KatanA source change. Ignored live OS-clipboard tests remain diagnostic only. |
| `native-text-input` | Japanese/`⭐️` commit is injected through a KUC/KLE event helper; selection restore uses a host-state hook. | KLE-owned actual `KatanaApp` host E2E must prove multi-byte selection, authoring transform cursor restore, IME composition, and `⭐️` variation-selector preservation. |
| `line-gutter` and `dirty-save-refresh` | `editor/rendering.rs` declares four relevant tests but no runnable KatanA test target registers them. | Add a KLE-owned actual `KatanaApp` host E2E using the read-only KatanA path dependency and make `katana-parity-check` reject source tests that have no runnable KLE registration. |
| `dirty-save-refresh` lifecycle source inventory | `process_markdown_formatting.rs` and `refresh_content.rs` define open/unopened formatting, scoped external undo, dirty-refresh suppression, preview/search/diagnostics refresh, and deferred HTML refresh, but the inventory records only broad editor UI/rendering files. | Add both host action sources and decisive lifecycle functions to the inventory; KLE-owned actual `KatanaApp` host E2E must observe active/non-active documents, no mutation for dirty external refresh, format/save, refresh fan-out, and scoped undo. |
| `kuc-text-input` and motion artifact | KUC TextSurface owns platform-raster selection/caret/IME/gutter/annotation/AccessKit and emits a deterministic actual-egui artifact. KLE local surface and `StorybookFallbackRenderer` remain. | Remove KLE generic surface/fallback ownership, consume the KUC surface in KLE and KDV, then prove the actual Katana editor path and artifact use the same frame record. |
| `command-chrome` | KLE `authoring_helper.rs` directly renders egui text buttons/Area. KUC toolbar/SVG raster exists, but floating/search artifact composition and consumer integration are incomplete. | Complete the KUC CommandChrome shared adapter and artifact path, inject `UiIconProps`/opaque actions, map only typed events in KLE, delete the local helper, and prove actual toolbar/search/replace interaction. |
| `editing-history` | KLE external format undo is covered but `EguiLanguageEditor` returns `Unsupported` for undo/redo; KatanA shortcut policy has no adapter parity proof. | KUC TextSurface emits generic history requests; KLE/KatanA real keyboard tests prove undo/redo, conflicts, read-only suppression, and document-scoped history semantics. |
| `accessibility-tree` | KUC TextSurface adapter publishes AccessKit from its frame record, but KLE returns `Unsupported` and does not consume the bridge. | Actual KatanA/KLE tests must query focus, Japanese selection, read-only, gutter and menu semantics through the shared KUC surface. |
| `find-replace-ui` | KLE API can find/replace, and KatanA document search proves find/navigation/highlight, but there is no full find/replace component. | KUC CommandChromeSearchStrip real interaction tests prove query/options/previous/next/count/close/replace/replace-all, then KLE-owned actual `KatanaApp` host E2E proves state/highlight/content effects. |

## Release Blockers

- Every row in the inventory must have both (a) KUC/KLE actual UI component evidence where it has UI and (b) a KLE neutral contract with concrete KLE-owned actual KatanA host-E2E evidence where it has host side effects. One side alone is not complete.
- Every correction in `Unresolved Parity Corrections` must have a runnable check. A source-file match, host-state test hook, KLE-only event injection, KUC contract test, KatanA baseline test, or fallback-renderer artifact alone is insufficient.
- `tools/katana-parity-check` must derive the source closure in `v0-1-0-katana-editor-source-universe.md` and enforce one unique source-derived leaf. It must reject the nonexistent `kle_downstream_adapter.rs`, every KatanA-repository patch plan, a static file count, source-only evidence, shared selector/harness, or missing KLE host E2E.
- The historical 26-row inventory is a top-level grouping, not a replacement for exact leaf counts. The gate must require all 12 toolbar triggers, 14 context authoring triggers, 17 code kinds, three image-ingest routes, text/IME/history, diagnostics, search, view/split, and shortcut leaves with separately resolvable evidence.
- `just katana-downstream-check` remains a simulator/probe for typed event mapping only. It is supplementary and cannot satisfy a host-effect leaf.
- KLE-owned `tools/katana-host-e2e` path-depends on the unchanged local KatanA crate, drives public KLE RawInput and real KatanA UI frames, and asserts the exact KatanA buffer/state/file effect per leaf. KatanA source, dependency manifest, test registration, branch, and worktree remain untouched.
- KatanA-specific Markdown/file IO behavior remains KatanA host-owned, but the editor action surface and KLE-owned host-E2E evidence remain part of v0.1.0 readiness.
- KatanA source-derived features and user-mandated replace/replace-all cannot return `EditorError::Unsupported` in a release path.
- Storybook must cover live KLE/KUC behavior for the generic UI contracts used by these features. Static fixtures are not enough.

## KatanA Baseline Integration Test Inventory

Read-only KatanA baseline checks were run from `/Users/hiroyuki_furuno/works/private/katana` on 2026-07-03. They prove the current KatanA editor behavior still passes, but they do **not** prove KLE integration because KatanA did not yet depend on local KLE.

The historical adapter commands below record an invalid prior approach. The
2026-08-13 reference checkout has no local KLE dependencies, adapter module,
adapter source file, or runnable adapter target. They MUST NOT be added,
executed, or used as a release gate in this scope. KLE-owned
`tools/katana-host-e2e` is the replacement evidence path; KatanA baseline
commands remain reference-only behavior evidence.

| KatanA command | Result | Relevance |
| --- | --- | --- |
| `cargo test -p katana-ui --test ui_integration_parallel editor_ui -- --list` | PASS, 7 runnable editor UI tests listed | Establishes the currently wired integration wrapper for view modes, update buffer, line numbers, clipboard image paste, and code-block menu lifecycle. |
| `cargo test -p katana-ui --test ui_integration_parallel editor_lint_fix_review_button -- --list` | PASS, 5 runnable diagnostics/problems tests listed | Establishes the currently wired integration wrapper for lint-fix review and Problems panel scope/status behavior. |
| `cargo test -p katana-ui --test ui_integration_parallel editor_ui::test_integration_view_modes -- --nocapture` | PASS | Baseline for KLE view-mode adapter expectations. |
| `cargo test -p katana-ui --test ui_integration_parallel editor_ui::test_integration_update_buffer -- --nocapture` | PASS | Baseline for KLE dirty/update-buffer adapter expectations. |
| `cargo test -p katana-ui toolbar_popup -- --nocapture` | PASS (from actual wired KatanA repo) | Verifies authoring toolbar popup position clamp, anchor, focus retention, and menu lifecycle unit behavior in the real KatanA crate. |
| `cargo test -p katana-ui --test ui_integration_parallel editor_ui::test_integration_editor_line_numbers_visibility -- --nocapture` | PASS (from actual wired KatanA repo) | Verifies line-number gutter rendering and click/hover visibility behavior in the KatanA editor integration path used by this parity baseline. |
| `cargo test -p katana-ui --test ui_integration_parallel editor_ui::input_assist_code_block_button_updates_editor_buffer -- --nocapture` | PASS (from actual wired KatanA repo) | Verifies code block menu open/select/update behavior and editor buffer updates in KatanA integration after toolbar input assist selection, covering the code-block menu lifecycle contract. |
| `cargo test -p katana-ui --test ui_integration_parallel editor_ui::code_block_kind_menu_closes_when_editor_is_clicked -- --nocapture` | PASS (from actual wired KatanA repo) | Verifies close-on-editor-click behavior for the code block kind menu in KatanA integration, completing runnable coverage for the code-block-menu release gate. |
| `cargo test -p katana-ui --test ui_integration_parallel editor_lint_fix_review_button::problems_status_count_follows_scope_only_while_panel_open -- --nocapture` | PASS | Baseline for KLE Problems panel scope/status adapter expectations. |
| `cargo test -p katana-ui --test ui_integration_parallel editor_ui::editor_context_menu_authoring_heading1_updates_editor_state_via_real_ui_routing -- --nocapture` | PASS (3 tests) | Verifies context-menu routing for markdown heading insertion, save buffer persistence, and format button behavior against real KatanA UI state and `KatanaApp` file-backed persistence checks. |
| `cargo test -p katana-ui --test ui_integration_parallel editor_ui::editor_context_menu_save_button_persists_editor_buffer_state -- --nocapture` | PASS (3 tests) | Verifies context-menu save action path against real KatanA UI state and `KatanaApp` file-backed persistence checks. |
| `cargo test -p katana-ui --test ui_integration_parallel editor_ui::editor_context_menu_format_button_updates_formatted_markdown_buffer -- --nocapture` | PASS (3 tests) | Verifies context-menu format button action path against real KatanA UI state and `KatanaApp` markdown-format roundtrip expectations. |
| `cargo test -p katana-ui --test ui_integration_parallel editor_kle_downstream_adapter::kle_document_search_markdown_aware_matches_and_refreshes_via_kle_adapter -- --nocapture` | REJECTED | It requires an out-of-scope KatanA dependency/module/source change and is not a KLE v0.1.0 evidence command. |
| `cargo test -p katana-ui --test ui_integration_parallel editor_kle_downstream_adapter::kle_kuc_text_area_ime_commit_star_reaches_real_katana_buffer -- --nocapture` | REJECTED | It requires an out-of-scope KatanA dependency/module/source change and is not a KLE v0.1.0 evidence command. |
| `cargo test -p katana-ui --test ui_integration_parallel editor_navigation -- --nocapture` | PASS (2) | Verifies multi-document navigation and active tab state transitions in real KatanA integration. |
| `cargo test -p katana-ui --test ui_integration_parallel editor_view_modes -- --nocapture` | PASS (2) | Verifies view-mode behavior for split/preview/code views in real KatanA integration. |
| `cargo test -p katana-ui --test ui_integration_parallel editor_ui::editor_select_and_jump -- --nocapture` | PASS (from actual wired KatanA repo) | Verifies select-and-jump coordinator behavior for `PreviewOnly` fallback to `CodeOnly`, target-line index/number wiring, `last_scroll_to_line`, and accesskit marker/line-number observability against real `KatanaApp` editor frame state. |
| `cargo test -p katana-ui --test ui_integration_parallel editor_toggle_view_modes -- --nocapture` | PASS (1) | Verifies view-mode toggle behavior and cycle transitions in real KatanA integration. |
| `cargo test -p katana-ui --test ui_integration_parallel editor_layout_persistence -- --nocapture` | PASS (3) | Verifies preview visibility and split-direction persistence behavior in real KatanA integration. |
| `cargo test -p katana-ui --test ui_integration_parallel editor_ui::editor_scroll_sync -- --nocapture` | PASS (from actual wired KatanA repo) | Verifies scroll coordinator behavior when host sets preview source, including `ScrollSource::Preview`, `ScrollSource::Neither`, `editor_y >= 0`, `editor_max > 0`, and non-empty `editor_line_anchors` after sync under real KatanA editor frame state. |
| `cargo test -p katana-ui --test ui_integration_parallel editor_kle_downstream_adapter::kle_event_action_stream_updates_real_katana_editor_state -- --nocapture` | REJECTED | It requires an out-of-scope KatanA dependency/module/source change and is not a KLE v0.1.0 evidence command. |
| `cargo test -p katana-ui --test ui_integration_parallel editor_kle_downstream_adapter -- --nocapture` | REJECTED | It must not be cited or reintroduced; the valid target lives in KLE `tools/katana-host-e2e` and uses KatanA read-only. |

`crates/katana-ui/tests/integration/editor/rendering.rs` is currently a source file for this audit, but its four tests are not registered by `ui_integration_parallel.rs` or `ui_integration_serial.rs`. The existing runnable evidence for navigation/view/scroll does not prove these rendering tests. Registration and runnable execution are release blockers.
