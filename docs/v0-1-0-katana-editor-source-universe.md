# KatanA Editor Source Universe

## Status

This is the source-universe audit ledger for KLE v0.1.0.  It is incomplete
work, not a claim of parity completion.  In particular, the earlier 57-file
list was only an initial seed set: it omitted the reachable search input,
action dispatch, and document-effect paths found during the 2026-08-14 audit.

| Field | Value |
| --- | --- |
| Reference repository | `/Users/hiroyuki_furuno/works/private/katana` |
| Reference revision | `4f6a6287c650a38633c7baeb544a92e739c68567` |
| Direct editor implementation seed files | 28 |
| Direct editor integration-test seed files | 8 |
| Workspace-search modal seed files | 6 |
| Cross-module seed files initially recorded | 21 |
| Canonical source-file cardinality | Pending generated closure |
| Reference edit policy | read-only |

The canonical set is not a hand-maintained cardinality.  `katana-parity-check`
must generate a per-file SHA-256 manifest from the closure defined below and
fail when the current reference file list, file hash, action route, decisive
branch, or documented source universe differs.  A later KatanA revision
requires a new audit; it must never be silently accepted.

The machine-readable root declaration is
`docs/v0-1-0-source-closure-roots.json`.  It fixes the same KatanA revision,
expands only the two required direct-source directories, and lists roots in
mixed app/state/preview areas individually.  The workflow resolves that
declaration against the pinned checkout, sorts the resulting Rust files, and
records the expanded paths in `SourceClosureInput`; it must not substitute a
single editor file or include a mixed directory wholesale.

The generated files, branch catalog, leaf proof, execution record, and
Storybook artifact contract are specified by
`docs/v0-1-0-parity-manifest-schema.md`.  The six artifacts are one joined
release input: a manually maintained file count, a source marker, or media
without the same-run execution record cannot establish parity.

### Required Release Profiles

The fixed KatanA release workflow runs macOS, Windows, and Linux
(`.github/workflows/test-and-build.yml:33-35` and `:129-131`). Source closure
and every proof join therefore have three mandatory target profiles:
`macos-latest`, `windows-latest`, and `ubuntu-latest`. The generator records
the actual `rustc -vV` host triple, full `rustc --print cfg`, Cargo feature and
dependency graph, lockfile hash, active edges, and inactive `cfg` candidates
for each profile, then takes their classified union. A local target result,
shared default profile, unrecorded inactive branch, or profile skip cannot
establish source completeness.

The profile record is also the authority for KUC platform font evidence. Exact
`⭐️` VS16 requires an actual KUC-resolved color face and isolated RGBA crop
proof on all three profiles; `Apple Color Emoji` is only the macOS fact and
cannot stand in for Windows or Linux.

#### Verified Target-Specific Source Seeds

The following target branches are source-closure seeds, not implementation
permission for KLE. The generator must classify their active and inactive
edges per profile before it can decide whether a branch is an editor leaf,
host-only sibling, or test-only oracle.

| Source span | Profile branch | Required closure treatment |
| --- | --- | --- |
| `app/action/mod.rs:2-5` | `clipboard_image_macos` is mounted only on macOS | expand the module and its callers on macOS; record an explicit inactive module edge on Windows/Linux. KLE/KUC may not recreate the pasteboard parser. |
| `app/action/clipboard_image.rs:14-61` | image acquisition priority is arboard image, file image, file URL, then a macOS pasteboard fallback; non-macOS ends without that fallback | generate separate macOS-fallback and non-macOS-no-fallback clipboard host-boundary leaves, including each success/error/no-mutation outcome. A common "clipboard image" aggregate is invalid. |
| `app/action/process_helpers.rs:118-150` | reveal-in-OS invokes `open -R` on macOS, `explorer /select,` on Windows, or Linux explorer fallback candidates | classify this action as a source-spanned native host-only sibling unless an editor-root origin reaches it. Its external command behavior is never reproduced by KLE/KUC. |
| `app/action/process_update.rs:20-35` | update installer uses an app-bundle ancestor on macOS and the executable path elsewhere | classify as a source-spanned update host-only sibling. Record both active/inactive branches, but never add it to the KLE editor root. |
| `app/action/{dispatch_secondary,dispatch_tertiary}.rs` | production-only callback arms are excluded under `test`/`coverage` | preserve the `cfg(all(not(test), not(coverage)))` edge in each profile graph and reject a test-only closure that silently omits its production branch. |

KUC separately has a target-specific defect at
`katana-ui-core/src/render_model/typed_text.rs:114-128`: its current resolver
selects `Apple Color Emoji` only on macOS and `SansSerif` for every other
target. The KUC replacement must be a color-face resolver/provisioner verified
per profile; it is not a KLE compatibility branch.

### Boundary terminology

In this ledger, a KLE “request”, “mapping”, or “typed event” always means a
closed neutral envelope containing only an opaque host-issued target,
descriptor revision, correlation, and source. It never means a KatanA action,
document identity, path, range, line, offset, Markdown operation, URL,
diagnostic payload, or semantic lookup. KLE forwards the envelope exactly once;
the unchanged host alone validates the revision and resolves it to an existing
KatanA action. Any older wording in this ledger that suggests KLE maps a
semantic value is superseded by this rule.

## Direct Editor Sources

Every Rust file under the following current directory is required:

```text
crates/katana-ui/src/views/panels/editor/**
```

At the recorded revision this expands to:

```text
authoring.rs                     context_menu_tests.rs       logic_tests.rs
authoring_tests.rs               decorations.rs              mod.rs
authoring_utils.rs               diagnostics_hover.rs        paste.rs
code_block_menu.rs               diagnostics_popup.rs        row_diagnostics.rs
context_menu.rs                  diagnostics_ui.rs           syntax.rs
context_menu_image_ingest.rs     layout.rs                   text_edit.rs
line_numbers.rs                  logic.rs                    toolbar.rs
logic_colors.rs                  logic_scroll.rs             toolbar_popup.rs
logic_scroll_tests.rs            types.rs                    ui.rs
utils.rs
```

The source parser must enumerate every `if`/`else`, `match` arm, early return,
and guard that changes an editor-visible state, typed action, document effect,
or accessibility result.  A helper may be classified as non-leaf only when its
callers and absence of an independent visible/action effect are recorded.

### Initial Direct-File Classification

This is a measured file-level audit index, not the canonical AST leaf manifest.
The `Observed entry` names the currently inspected source surface; the generated
closure still has to record exact branch spans, callers, and every state guard.
No row below can close a parity leaf by itself.

| File | Observed entry | Initial classification | Required closure result |
| --- | --- | --- | --- |
| `authoring.rs` | `MarkdownAuthoringOps::{apply,apply_code_block}` | transform effect | every Markdown operation and cursor result is an individual host-effect leaf |
| `authoring_tests.rs` | transform unit cases | test oracle | registration/effect link only; never KLE E2E evidence |
| `authoring_utils.rs` | inline/line/block/snippet transform helpers | helper | caller-specific branch rationale and UTF-8 range safety route |
| `code_block_menu.rs` | `CodeBlockMenuOps`, `CodeBlockMenuPopupOps` | command presentation | all 17 kinds, open/select/close/focus/dismiss leaves |
| `context_menu.rs` | `EditorContextMenu::{render,render_inline,render_structure,render_headings,render_blocks,render_insert}` | command presentation | root order, enabled guards, all direct authoring leaves, close/focus result |
| `context_menu_image_ingest.rs` | image ingest submenu render | external-command presentation | file and clipboard visible leaves with host availability guards |
| `context_menu_tests.rs` | context menu unit cases | test oracle | registration and distinct physical KLE input cases |
| `decorations.rs` | cursor/hover/search decoration renderers | generic presentation | KUC semantic decoration records, no local KLE geometry |
| `diagnostics_hover.rs` | wave and single/all diagnostic action UI | diagnostics presentation/action | severity, hover suppression, fix/fix-all/docs routes |
| `diagnostics_popup.rs` | `DiagnosticsPopupOps::{show,show_content}` | diagnostics lifecycle | open/close/outside/focus/action leaves |
| `diagnostics_ui.rs` | diagnostic render entry | diagnostics composition helper | marker/popup caller linkage and AccessKit state |
| `layout.rs` | `EditorLayouter::{layout,append_highlighted_line}` | currently uninvoked syntax helper | the fixed-revision direct source search has no construction/call from `TextEditRenderer` or any production editor caller; generate an unreachable-helper record unless a resolved macro/extern edge is found. It is not a KUC syntax-span leaf by definition presence alone. |
| `line_numbers.rs` | `EditorLineNumbers::{render,render_row_number}` | gutter presentation/input | all visible/active/hover/click/accessibility leaves |
| `logic.rs` | `EditorLogicOps::apply_pending_cursor` | cursor restoration | authoring transform and next-frame cursor effect |
| `logic_colors.rs` | editor color resolution | presentation helper | semantic token mapping, not copied color literals |
| `logic_scroll.rs` | `handle_scroll_to_line`, `update_scroll_sync` | scroll coordinator | source suppression, target consumption, preview/editor convergence |
| `logic_scroll_tests.rs` | scroll unit cases | test oracle | registration/effect linkage only |
| `logic_tests.rs` | scroll synchronization unit cases | test oracle | branch-specific executable KLE/KatanA scenario requirement |
| `mod.rs` | editor module registration | mount/helper edge | root-to-module reachability only |
| `paste.rs` | image interception and `file://` recognition/decoding | host acquisition boundary | opaque KLE request; KatanA-only raw image/file-list/file-URL decision path |
| `row_diagnostics.rs` | `RowDiagnosticsRenderer::render` | gutter diagnostics presentation | priority, aggregation, official start-line and hit-target leaves |
| `syntax.rs` | `MarkdownSyntaxHighlighter::highlight` | currently uninvoked syntax provider | the fixed-revision source search finds the type definition and a `MarkdownEditorWidget::frame_config` test asserting default empty spans, but no production construction/call. Generate an unreachable-helper record unless a resolved macro/extern edge is found; do not add KLE/KUC syntax decoration to imitate an unused source helper. |
| `text_edit.rs` | `TextEditRenderer::render` | primary input/mutation path | typing, IME, selection, read-only, paste priority, buffer update, accessibility |
| `toolbar.rs` | `EditorToolbar::show` | floating command presentation | 12 independent toolbar leaves and selection guards |
| `toolbar_popup.rs` | `ToolbarPopup::show` | popup lifecycle/placement | selection anchor, clamp, suppression, focus/outside lifecycle |
| `types.rs` | `MarkdownEditorWidget` configuration | surface configuration helper | typography/theme/spacing ownership classified to KUC or host injection |
| `ui.rs` | `EditorContent::show` | editor mount/composition | all input/overlay/scroll route callers and action handoff |
| `utils.rs` | text-coordinate conversion, row-anchor extraction and bottom padding | generic text/layout semantics | byte/character/line/range/anchor/padding branches map to KUC generic records; no KLE coordinate or egui layout implementation |

The apparent local `file://` parser in `paste.rs` is a KatanA host reference,
not a transfer target. KUC and KLE must use its source branches for parity
inventory only and emit no file URL parsing logic of their own.

## Workspace Search Modal Sources

The workspace-search modal is an editor requirement, not an optional shell
search surface. Every Rust file under the following directory, and its modal
entry point, are required source-closure roots:

```text
crates/katana-ui/src/views/modals/search.rs
crates/katana-ui/src/views/modals/search_tabs/**
```

At the fixed revision the directory expands to `filename_tab.rs`, `history.rs`,
`md_tab.rs`, `mod.rs`, and `utils.rs`. The source closure must record each
modal-tab, query/filter/history/result branch individually. A document-search
root, a generic SearchStrip fixture, or an aggregate workspace-search counter
does not cover this directory.

## Slideshow Modal Sources

The slideshow overlay has native-window and editor-visible interaction branches
that cannot be inferred from the command route alone. The fixed-revision source
roots are:

```text
crates/katana-ui/src/preview_pane/slideshow/modal.rs
crates/katana-ui/src/preview_pane/slideshow/controls.rs
```

`modal.rs` owns Escape, keyboard page movement, visible-content pagination,
idle opacity and fullscreen-restoration decisions. `controls.rs` owns the
previous/next availability and close-control input route. The closure records
each branch independently; a `ToggleSlideshow` action origin, static overlay
fixture, or generic KUC fullscreen control does not cover either source file.

### Direct Editor Edge Seeds

The following read-only call edges were measured from the direct editor tree.
They are closure seeds, not a substitute for AST traversal. Every source span
and branch condition still has to be generated by `katana-parity-check`.

| Caller | Callee | Why the edge is behavior-bearing |
| --- | --- | --- |
| `ui.rs:EditorContent::show` | `text_edit.rs:TextEditRenderer::render` | actual editable/read-only input, all overlays, paste arbitration, and action emission are mounted through this call |
| `ui.rs:EditorContent::show` | `logic_scroll.rs:EditorLogicOps::update_scroll_sync` | scroll offset, preview consumption, echo suppression, source state, and anchors change editor/preview behavior after each visible frame |
| `text_edit.rs:TextEditRenderer::render` | `context_menu.rs:EditorContextMenu::render` | secondary-click command tree, enabled state, close behavior, and host actions are coupled to the actual text response |
| `text_edit.rs:TextEditRenderer::render` | `decorations.rs:EditorDecorations::{render_cursor_line,render_hovered_lines,render_search_matches}` | active/hover/search annotations are separate visible and accessibility state from text mutation |
| `text_edit.rs:TextEditRenderer::render` | `diagnostics_ui.rs:EditorDiagnostics::render_diagnostics` and `line_numbers.rs:EditorLineNumbers::render` | inline diagnostics and gutter marker/row/popup paths are independently visible and interactive |
| `line_numbers.rs:EditorLineNumbers::render` | `row_diagnostics.rs:RowDiagnosticsRenderer::render` -> `diagnostics_popup.rs:DiagnosticsPopupOps::show` | gutter number, official start-line aggregation, icon hit target, popup lifecycle and actions form one closure chain |
| `text_edit.rs:TextEditRenderer::render` | `toolbar_popup.rs:ToolbarPopup::show` | selection anchor, diagnostic suppression, focus/outside dismissal, and floating toolbar lifecycle are current-frame behavior |
| `toolbar_popup.rs:ToolbarPopup::show` | `toolbar.rs:EditorToolbar::show` and `code_block_menu.rs:CodeBlockMenuPopupOps` | the toolbar's direct actions and code-menu interaction/dismissal are distinct command leaves |
| `context_menu.rs:EditorContextMenu::render` | `context_menu_image_ingest.rs:EditorContextMenuImageIngestOps::render` and `code_block_menu.rs:CodeBlockMenuOps::show` | the Ingest submenu and nested 17-kind code submenu must not be omitted from the root context-menu proof |
| `text_edit.rs:TextEditRenderer::render` | `paste.rs:EditorPasteOps` and `logic.rs:EditorLogicOps` | focused pre/post-event clipboard priority determines whether text mutation or opaque host image acquisition occurs |
| `text_edit.rs:TextEditRenderer::render` and diagnostics/search/scroll callers | `utils.rs:EditorLogicOps` coordinate/anchor/padding helpers | source character/byte/line conversion, first-row anchors and bottom padding constrain diagnostics, selection, scroll and visible editor geometry |

The generated closure must discover and validate additional edges beyond this
table. A missing listed edge is an immediate seed failure; a generated-only
edge remains unresolved until it has a classification and leaf/helper rationale.

### Verified Static Syntax Non-Invocation Candidate

At the fixed source revision, `TextEditRenderer::render` constructs
`egui::TextEdit::multiline(buffer)` without a custom layouter. A read-only
search of all `katana-ui/src` Rust production sources finds
`EditorLayouter`/`MarkdownSyntaxHighlighter` only at their own definitions;
the sole `EditorConfig::syntax_highlighter` use is the `types.rs` test that
asserts the default highlighter produces an empty span list. This is a static
source fact, not yet a generated closure result. The generator must resolve
all macro/trait/external candidates before recording either helper as
`unreachable_helper`; any resolved invocation reopens the exact source branch.
Until then, no syntax-colored KLE/KUC parity leaf, local highlighter, or
syntax-only Storybook stage is admissible.

## Editor-Frame Sibling Seeds

`TabToolbar` mounts document tabs, document-frame navigation, and the URL
source control in the same editor frame. These routes are in the KLE parity
universe. They must not be silently discarded merely because their host effect
can create or select a document rather than mutate the active text buffer.

| Source route | Observed behavior-bearing branch family | Required KUC/KLE/KatanA boundary |
| --- | --- | --- |
| `views/app_frame/tab_toolbar.rs:16-109` -> `top_bar/tab_bar` | document tabs, URL source bar, virtual-document toolbar suppression, dirty indicator and document-search mount share the editor frame | document tabs/search remain existing KLE leaves; the sibling routes below receive their own generated leafs and cannot be covered by a tab aggregate. |
| `views/app_frame/breadcrumbs.rs:17-112` | virtual Demo/LinterDocs renders a non-navigable final label; no-workspace path is labels only; workspace paths build every prefix; each non-final prefix opens a tree-derived candidate menu and can emit a document action | KUC owns generic retained `BreadcrumbNavigator` layout, disclosure, menu, focus and AccessKit. KLE forwards a selected opaque host target once. KatanA retains workspace-tree lookup, path handling, document selection and filesystem state. |
| `views/top_bar/url_source.rs:10-45` | controlled address text, submit button, focus-loss Enter submit, whitespace rejection, empty/non-empty history visibility, and history selection | KUC owns generic retained `SourceAddressBar` text/focus/menu/AccessKit behavior. KLE projects only host descriptors; text edit, preedit and history interaction stay KUC-retained. On submit alone, KLE forwards one non-persistent typed submission directly to the host. It never validates, fetches, stores, parses or renders source payloads. |
| `app_action_types.rs:32 AppAction::OpenUrl` -> `app/action/dispatch_secondary.rs:20` -> `app/action/url_source.rs:12-167` | `file://` versus HTTP validation; invalid/local canonicalization/not-openable/document/HTML branches; pending-request, success, error, timeout and disconnect outcomes | KatanA owns URL validation, filesystem, network, history, source tabs, document identity, KRR browser session and all error/status state. A KLE host E2E starts only from a KUC event and invokes unchanged `OpenUrl`; it may not seed `UrlTabState` or inject a fetch result. |
| `views/app_frame/workspace_toolbar.rs:9-30` -> `WorkspaceTabBar` | switching/reordering a *workspace* shell tab | This is adjacent shell chrome, not a document-editor feature. The generator records this exact source-spanned host-only rationale and excludes it from KLE root input. It must fail if a document-tab source is classified as a workspace-tab sibling or vice versa. |

### Resolved Navigation And Source-Address Branch Seeds

The following is a fixed-revision source inventory, not a hand-maintained
completion matrix. The source-closure generator must derive one canonical leaf
per reachable branch below, including its predicate and final declared effect.
It must fail on an added, removed, or unclassified branch; it may not replace
this inventory with a source-file count, an `OpenUrl` aggregate, or a shared
keyboard trace.

| Family | Exact source path | Required individual branch candidates | Final owner and effect boundary |
| --- | --- | --- | --- |
| breadcrumb presentation | `views/app_frame/breadcrumbs.rs:17-112` | `Katana://Demo/` and `Katana://LinterDocs/` final-label-only; empty segment filtering; slash and backslash segmentation; no-workspace labels; workspace final-label-only; workspace non-final menu; missing/non-directory tree result empty menu | KUC owns generic retained layout, menu, focus, overflow and AX state. These are `kuc_retained_ui_effect` leaves with an unchanged bootstrapped KatanA observation. KLE has no path parsing, segment list, or tree lookup. |
| breadcrumb candidate traversal | `views/app_frame/breadcrumbs.rs:78-103` -> `views/panels/explorer/breadcrumb.rs:17-35` | nested-directory disclosure at every depth; selectable file child; selected file close-menu behavior; no selectable child | KatanA resolves the tree and source target. KUC emits only the host-projected opaque selection target; KLE forwards it once without path visibility. A selection is an `in_process_host_effect` only after unchanged `SelectDocument -> handle_action_select_document -> DocumentOps::handle_select_document` reaches the observed document/preview/search/diagnostic continuation. |
| source-address retained interaction | `views/top_bar/url_source.rs:10-45` | text/preedit change; submit-button activation; lost-focus Enter activation; blank/whitespace rejection; history absent; history open; each history selection; history selection without host submission | KUC `SourceAddressBar` owns edit/preedit/focus/history disclosure/AX records. Only submit-button and lost-focus Enter may emit a single non-Clone/non-Serialize opaque submission transport. KLE cannot inspect, normalize, persist, parse, hash, log, or retry its text. |
| URL admission | `app/action/dispatch_secondary.rs:20` -> `app/action/url_source.rs:12-115` | `file://` recognition; invalid non-file URL; local URL validation failure; canonicalization failure; non-openable local path; canonical URL failure; local document; local HTML read failure; local HTML source admission; HTTP fetch admission | The unchanged KatanA host owns URL parsing, filesystem checks, temporary-workspace selection, state/history updates and request creation. Each branch has an independent host observation or no-mutation/error observation; direct `UrlTabState` setup is invalid. |
| URL completion | `app/action/url_source.rs:127-167` -> `app/action/url_source/document.rs:8-108` -> `app/action/html_navigation.rs:94-166` | queued request pending; HTML success with browser-source construction failure; HTML success replacing an existing document; HTML success opening a new document; binary-document surface construction failure; binary document existing/new reference document; collector error; timeout; disconnected channel; multiple pending requests with loading retained | KatanA owns the real transport, source payload, source tabs, document identity, error/status and KDV/KRR session creation. HTTP leaves use a real loopback listener and actual bytes; local leaves use a real temporary `file://` path. KUC/KLE cannot inject a response, source payload, browser bitmap, KRR session, or URL-tab state. |
| selected-document continuation | `app/action/dispatch.rs:35-41` -> `app/action/process_document.rs:148-166` -> `app/document.rs:19-122` | existing/new document; activation/no activation; image replacement; preview refresh; document-format PreviewOnly transition; search-open refresh; search-modal close; diagnostics continuation; open/load failure; per-document split initialization; temporary-workspace branch reached from local source admission | KatanA alone owns `PathBuf`, document list/index, buffer, preview, scroll, workspace, filesystem and diagnostics state. This continuation is joined to breadcrumb/source selection evidence but never replicated in KLE/KUC. |

The source-address family also requires this state root:

```text
crates/katana-ui/src/state/url_tab.rs
```

It retains KatanA-owned URL history, pending request, active tab and error
state. KLE and KUC receive neither this state nor a synthetic substitute; every
history/loading/error branch joins the real host route.

For HTTP source acquisition, strict host E2E may use a real loopback HTTP
service that sends actual bytes over the real transport, but it must not mock,
stub or replace KatanA, `ehttp`, KRR or a browser session. The execution record
identifies the real listener endpoint, request/response hashes, KatanA action
and frame chain, and the resulting source-document/browser observation. The
`file://` route separately uses a real temporary file and unchanged KatanA
filesystem path. Both routes must reject direct `UrlTabState` mutation.

The text `17-kind` is an observation at the fixed reference revision, not a
hard-coded completion count. `CodeBlockMenuOps::show_items` derives its visible
items from `CodeBlockKind::all()`. The generated closure must resolve that
source inventory and produce one leaf candidate per current enum value; an
added/removed/reordered value, an unknown call target, or an unclassified item
fails the gate.

## Direct Integration Sources

Every Rust file under this current directory is required as KatanA reference
evidence only:

```text
crates/katana-ui/tests/integration/editor/**
```

```text
layout_persistence.rs
lint_fix_review_button.rs
mod.rs
navigation.rs
rendering.rs
toggle_view_modes.rs
ui.rs
view_modes.rs
```

These files are KatanA reference candidates only. Registration is itself a
source fact and must be proven from a concrete current test target; a file in
this directory is not automatically runnable. They are neither KLE public-input
evidence nor KLE-owned KatanA host-E2E evidence.

| Integration file | Current registration fact | Observed KatanA reference behavior | Required generated-oracle record |
| --- | --- | --- | --- |
| `layout_persistence.rs` | locally listed by `integration/editor/mod.rs`, but that module has no current root test-target reference | PreviewOnly/Split visibility and split-direction persistence | unresolved source test candidate; it cannot be counted as runnable KatanA evidence, and KLE host E2E independently proves the state effect |
| `lint_fix_review_button.rs` | registered separately by `tests/ui_integration_parallel.rs:60-61`, not by editor `mod.rs` | fixable diagnostics, review tab, multi-document/fix batches | diagnostic action branch and final review/buffer effect |
| `mod.rs` | lists `layout_persistence`, `rendering`, `toggle_view_modes`, and `view_modes`, but neither this file nor `integration/mod.rs` has a current root test-target reference | local module declaration only | unresolved registration edge; source parser must report it rather than treating it as runnable |
| `navigation.rs` | no current Rust registration reference was found at the fixed revision | workspace/tab open, switch, multiple document behavior | unresolved source test candidate; it cannot be counted as runnable KatanA evidence, and KLE host E2E must independently prove document identity/isolation |
| `rendering.rs` | locally listed by `integration/editor/mod.rs`, but the enclosing module is not a current root test target | line numbers/highlight, update buffer, save, TextEdit update | unresolved source test candidate; direct UI source plus runnable KLE RawInput and actual host effect remain required |
| `toggle_view_modes.rs` | locally listed by `integration/editor/mod.rs`, but the enclosing module is not a current root test target | toggle cycle | unresolved source test candidate; action dispatch/state transition host effect remains required |
| `ui.rs` | registered separately by `tests/ui_integration_parallel.rs:10-11`, not by editor `mod.rs` | editor click/input, code menu, file URL paste, line number, update buffer | physical input baseline plus separate KLE public input route |
| `view_modes.rs` | locally listed by `integration/editor/mod.rs`, but the enclosing module is not a current root test target | view mode and split behavior | unresolved source test candidate; state/layout coordinator host effect remains required |

The checker must fail when an included integration test has no corresponding
production branch in the closure, and also when a production branch relies on
an integration test as its only evidence.

## Direct Document Tab Sources

Every Rust file under the following current directory is required. Tab state
is editor behavior: an individual root file is not sufficient evidence for its
selection, close, grouping, drag, overflow, or persistence branches.

```text
crates/katana-ui/src/views/top_bar/tab_bar/**
```

The generated closure must classify every reachable tab branch individually.
Source presence never substitutes for KUC root interaction, KLE opaque transit,
or actual KatanA host effect evidence.

## Cross-Module Source Seeds

The following 21 existing files were the first cross-module seeds found from
the editor surface or its host effect.  They remain required, but are not the
complete source universe.

```text
crates/katana-ui/src/app/action/clipboard_file_url.rs
crates/katana-ui/src/app/action/clipboard_image.rs
crates/katana-ui/src/app/action/clipboard_image_macos.rs
crates/katana-ui/src/app/action/dispatch_secondary.rs
crates/katana-ui/src/app/action/image_ingest.rs
crates/katana-ui/src/app/action/process_authoring.rs
crates/katana-ui/src/app/action/process_document.rs
crates/katana-ui/src/app/action/process_markdown_formatting.rs
crates/katana-ui/src/app/action/refresh_content.rs
crates/katana-ui/src/app/doc_search.rs
crates/katana-ui/src/app/document_edit.rs
crates/katana-ui/src/editor_undo.rs
crates/katana-ui/src/markdown_authoring_op.rs
crates/katana-ui/src/shell_ui/shell_ui_shortcuts.rs
crates/katana-ui/src/state/app_state_impl.rs
crates/katana-ui/src/state/command_inventory/edit_commands.rs
crates/katana-ui/src/state/shortcut_context.rs
crates/katana-ui/src/views/app_frame/central_content.rs
crates/katana-ui/src/views/layout/split.rs
crates/katana-ui/src/views/panels/preview/content.rs
crates/katana-ui/src/views/panels/preview/content_html_browser.rs
crates/katana-ui/src/views/panels/preview/tangochou.rs
crates/katana-ui/src/views/top_bar/search.rs
```

The subsequent audit found these further reachable paths.  They are mandatory
closure seeds, not optional supporting implementation:

```text
crates/katana-ui/src/app/action/dispatch.rs
crates/katana-ui/src/app/action/dispatch_tertiary.rs
crates/katana-ui/src/app/action/process_diagnostics.rs
crates/katana-ui/src/app/action/process_helpers.rs
crates/katana-ui/src/app/document.rs
crates/katana-ui/src/app/diff_review.rs
crates/katana-ui/src/app/preview.rs
crates/katana-ui/src/app_action.rs
crates/katana-ui/src/app_action_types.rs
crates/katana-ui/src/app_state.rs
crates/katana-ui/src/linter_bridge.rs
crates/katana-ui/src/shell/constants.rs
crates/katana-ui/src/state/search.rs
crates/katana-ui/src/state/scroll_sync/mod.rs
crates/katana-ui/src/state/scroll_sync/types.rs
crates/katana-ui/src/theme_bridge/logic.rs
crates/katana-ui/src/theme_bridge/mod.rs
crates/katana-ui/src/theme_bridge/types.rs
crates/katana-ui/src/views/app_frame/tab_toolbar.rs
crates/katana-ui/src/views/top_bar/search_logic.rs
crates/katana-ui/src/views/app_frame/ui.rs
crates/katana-ui/src/views/top_bar/mod.rs
crates/katana-ui/src/views/top_bar/status_bar.rs
crates/katana-ui/src/views/top_bar/tab_bar/mod.rs
crates/katana-ui/src/app/action/process_tabs.rs
crates/katana-ui/src/views/panels/problems/mod.rs
crates/katana-ui/src/views/panels/problems/ui.rs
crates/katana-ui/src/views/panels/problems/scope.rs
crates/katana-ui/src/views/panels/problems/diagnostics_renderer.rs
crates/katana-ui/src/views/panels/problems/bulk_fixes.rs
crates/katana-ui/src/views/panels/problems/fix_preview_renderer.rs
crates/katana-ui/src/views/panels/problems/fix_preview_model.rs
crates/katana-ui/src/state/diagnostics.rs
```

### Closure expansion discovered after the initial seeds

The following continuations were found by tracing the currently recorded
routes. They demonstrate why a manually frozen source count cannot establish
coverage. The generator must either include each file and branch in the
closure, or reject it with a specific host-only/helper rationale; it must not
silently omit any entry below.

| Incoming route | Newly discovered continuation | Decisive fact to classify |
| --- | --- | --- |
| search next/previous, preview fallback, document jump | `crates/katana-ui/src/state/scroll.rs` | `ScrollState::{scroll_to_line,last_scroll_to_line,reset_for_document_change}` controls one-shot jumps and prevents repeat jitter. |
| search next/previous, `PreviewOnly` fallback | `crates/katana-ui/src/views/panels/preview/logic.rs` | `PreviewLogic::{compute_forced_offset,update_scroll_sync}` consumes the preview-side fallback and clears editor-origin sync. |
| `PreviewOnly` split rendering and scroll consumption | `crates/katana-ui/src/views/layout/split.rs` | `scroll_to_line` is consumed after preview processing when an editor is absent. |
| view shortcut inventory | `crates/katana-ui/src/state/command_inventory/view_commands.rs` | `view.toggle_split_mode` and `view.toggle_code_preview` map commands to action routes. |
| editor command inventories | `crates/katana-ui/src/state/command_inventory/{file,edit,view}_commands.rs` | every item/availability/shortcut is a source seed. `file.save`, `file.close_document`, `file.restore_closed`, `view.doc_search`, `view.refresh_document`, `edit.ingest_clipboard_image`, and every authoring shortcut are editor routes; workspace/application commands need an explicit source-spanned `host_only` rationale. |
| preview side rail and TOC | `crates/katana-ui/src/views/panels/preview/{side_panels,side_panel_hover,side_panel_toc,side_panel_toc_ops,side_panel_export,side_panel_story,side_panel_tools,side_panel_tools_inner}.rs`, `views/panels/toc/{mod,render,anchor_ops,anchor_lookup_ops}.rs`, `state/toc.rs` | rail/panel mechanics, TOC hierarchy/active anchor/scroll and every action route are direct editor behavior. KLE may not replace this closure with a panel, accordion, line/anchor mapping or host-state fixture. |
| select-and-jump | `crates/katana-ui/src/app_action_types.rs`, `crates/katana-ui/src/app/action/dispatch.rs`, `crates/katana-ui/src/app/action/process_document.rs` | `SelectDocumentAndJump` forces `CodeOnly` only from `PreviewOnly`, then supplies the scroll request. |
| diagnostic documentation action | `crates/katana-ui/src/app/action/process_linter.rs` | `OpenLinterDoc` dispatch reaches the host browser/documentation effect. |
| lint-fix review effect | `crates/katana-ui/src/app/document_edit.rs`, `crates/katana-ui/src/app/diff_review.rs` | `ApplyLintFixes` and file batches mutate buffer/review state and require separate effects. |
| document update and save | `crates/katana-ui/src/app/document_contract.rs`, `crates/katana-ui/src/app/action/mod.rs` | update/save contract and preview refresh continuation must be linked to `document.rs`, not inferred from dispatch alone. |
| theme-driven diagram refresh | `crates/katana-ui/src/shell_ui/shell_ui_update.rs:161-163` -> `AppAction::RefreshDiagrams` -> `app/action/refresh_content.rs:91-105` | theme refresh enqueues `RefreshDiagrams` only when no action is already pending; the handler clears all image/viewer caches, then either returns with no active document or refreshes the active preview. `RefreshDocument { is_manual: true }` also calls this handler only for a virtual `Katana://` document. These are editor host-continuation branches, not a KLE-visible refresh-diagrams control. |
| KatanA test oracles outside the direct integration directory | `crates/katana-ui/src/app/doc_search_tests.rs`, `crates/katana-ui/src/app/document_edit_tests.rs`, `crates/katana-ui/src/app/diff_review_source_tests.rs`, `crates/katana-ui/src/app/diff_review_reopen_tests.rs` | these can be test-oracle edges only after the production branch is included; they never substitute for KLE-owned host E2E. |

The preview, state, and app directories contain unrelated functionality. The
closure must traverse from a behavior-bearing source symbol, not include a
directory wholesale. Conversely, an edge through a shared state/action type
does not permit the generator to stop at the type definition: it must record
the dispatch and concrete handler that changes the editor-observable result.

An included KatanA source is a behavior reference, not a transfer instruction.
KUC owns generic text, command, context-menu, glyph, accessibility, layout,
and artifact behavior. KLE owns only neutral host-descriptor projection and
closed or one-shot forwarding; it has no KatanA presentation/action type
dependency. Host document, filesystem, clipboard acquisition, linter, preview,
and shortcut-router effects remain outside both KUC and KLE.

## Decisive UI-To-Host Routes

The following routes were confirmed from current KatanA source.  They are a
minimum branch catalog for the generated closure; a route row is not a single
leaf and must be expanded by state branch and input route.

| Route | Source start and decisive branch | Host continuation | Required KLE/KUC boundary |
| --- | --- | --- | --- |
| normal typing | `views/panels/editor/text_edit.rs::TextEditRenderer::render`: editable state; `response.changed()` after no image-paste result | `app/action/dispatch.rs::UpdateBuffer` -> `app/document.rs::handle_update_buffer` -> preview/search/diagnostic refresh | KUC owns text/IME/caret/selection. The changed value can cross KLE only as one nonpersistent, non-observed input transit; the host owns `UpdateBuffer`, dirty/preview/search/diagnostics mutation and the returned projection. |
| paste precedence | `ui.rs` frame-local document-buffer clone; `text_edit.rs` plus `paste.rs`: editor focus; paste event; raw image; file-list; image `file://`; text change; non-image/no-payload | `dispatch_secondary.rs::IngestClipboardImage` -> `image_ingest.rs::handle_action_ingest_clipboard_image` -> `process_image_ingest` | Raw image is intercepted before text mutation. A supported image `file://` can mutate only the frame-local clone, restores the pre-paste cursor, and emits no `UpdateBuffer`; KatanA host inserts Markdown into the unchanged document at the original selection. KUC emits a generic request only; KLE correlates a typed token; host alone acquires OS data, writes assets, and inserts Markdown. |
| file image | `toolbar.rs`, `context_menu_image_ingest.rs`, and `text_edit.rs` visible/enabled image command branches | `dispatch_secondary.rs::IngestImageFile` -> `image_ingest.rs::handle_action_ingest_image_file` | KUC owns the generic enabled command/UI route; KLE forwards one opaque host-issued target; file dialog/extension/read/write stay host-owned |
| Markdown authoring | `toolbar.rs`, `toolbar_popup.rs`, `context_menu.rs`, `code_block_menu.rs`: selection guard, toolbar lifecycle, command/code-kind menu branch | `dispatch_secondary.rs::AuthorMarkdown` -> `process_authoring.rs::handle_action_author_markdown` -> `authoring.rs::MarkdownAuthoringOps::apply` -> update buffer/cursor restore | KUC owns toolbar/menu/focus/placement/dropdown; KLE forwards only the host-issued opaque target/revision/correlation; KatanA host owns command resolution, transform and persisted cursor effect |
| Save and conditional format | `context_menu.rs`: Save always; Format only editable Markdown extension | `dispatch.rs::SaveDocument` -> `document.rs::handle_save_document`; `dispatch_secondary.rs::FormatMarkdownFile` -> `process_markdown_formatting.rs` | KUC owns generic menu enabled/disabled/AccessKit; KLE does not save/format; host owns file I/O, refresh, and undo record |
| diagnostics gutter and popup | `row_diagnostics.rs`: official metadata and start-line guard; severity priority; `diagnostics_popup.rs`: hover/click/open/outside/action | `dispatch.rs::ApplyLintFixes` -> `document_edit.rs` -> `diff_review.rs`; `dispatch_tertiary.rs::OpenLinterDoc` | KUC owns generic range marker, row bounds, hit test, popup, AccessKit; KLE forwards opaque diagnostic targets only; host owns lint/diff review/browser effect |
| document search | `views/app_frame/tab_toolbar.rs` -> `views/top_bar/search.rs` -> `search_logic.rs`: focus, Enter/Shift+Enter, arrows, Escape, zero-result enablement | `dispatch_secondary.rs` -> `process_helpers.rs` -> `doc_search.rs`, editor/preview scroll state | KUC owns the retained search strip input/focus/accessibility. Query/close text can cross KLE only as one nonpersistent, non-observed input transit; directions use host-projected opaque targets. The host owns KatanA query/match/navigation result and scroll mapping. |
| replace current / replace all | user-mandated visible KUC search controls; no confirmed KatanA editor-side control source | `app_action_types.rs::AppAction::ReplaceText` -> `dispatch.rs` -> `document_edit.rs::DocumentEditOps::handle_replace_text` | KUC owns generic replace controls. Replacement text can cross KLE only as one nonpersistent, non-observed transport without match/range visibility; KatanA host derives the exact current byte range, owns document mutation and refresh. Replace is never claimed as a KatanA-origin UI feature. |
| problems panel / lint-review actions | `diagnostics_hover.rs`, `diagnostics_ui.rs`, and editor integration lint-review source | `dispatch_secondary.rs::ToggleProblemsPanel` and `RefreshDiagnostics`; `dispatch.rs::ApplyLintFixes` -> `document_edit.rs` / `diff_review.rs` | KUC retains generic Problems open/scope/disclosure presentation as a `kuc_retained_ui_effect`; KLE emits no host panel action. Fix/jump/docs forward opaque diagnostic targets; KatanA owns review/document/browser effects. |
| Problems status-bar mount | `views/app_frame/ui.rs` -> `views/top_bar/mod.rs` -> `status_bar.rs` | the status-bar Problems button emits `ToggleProblemsPanel`; its mount, current count, state toggle and diagnostics effect cannot be inferred from the dispatch arm alone. |
| Problems panel mount | `views/app_frame/ui.rs` -> `views/panels/problems/{mod,ui}.rs` -> scope/renderer/preview/bulk-fix modules | the bottom panel owns its own visible state, scope, disclosure, fix and document-jump branches. Its KatanA rendering code is not a permitted KLE implementation route. |
| document tab-strip mount | `views/app_frame/tab_toolbar.rs` -> `views/top_bar/mod.rs` -> `tab_bar/mod.rs` | visible document tabs expose active selection, close, traversal, pin, reorder, group and restore routes. The closure must recurse through the tab-bar submodules that emit a document-changing action. |
| tab/document handlers | `app/action/process_tabs.rs`, document action dispatch/handlers and state/document paths | next/previous, direct selection, close variants, restore, pin, ordering and grouping must each reach their final document/workspace state or a source-spanned non-editor rationale. |
| tab navigation and document isolation | editor UI shortcut/input path plus tab navigation sources | `dispatch_secondary.rs::{SelectNextTab,SelectPrevTab}` -> `process_tabs.rs::{handle_action_next_tab,handle_action_prev_tab}`; `editor_undo.rs` | KUC owns editor focus/input facts only; KLE preserves typed document identity; host owns active document, per-document history, and state reset. |
| view, scroll, and shortcuts | `ui.rs`, `logic.rs`, `logic_scroll.rs`, `line_numbers.rs`; `edit_commands.rs`, `shortcut_context.rs`, `shell_ui_shortcuts.rs` | `dispatch_secondary.rs`, `state/app_state_impl.rs`, `views/layout/split.rs`, `views/panels/preview/content.rs` | KUC owns generic scroll/focus/bounds/row action; KLE forwards one opaque host source target or closed host request without coordinates; host owns document/view/split/shortcut arbitration state |

The clipboard routes deliberately include source that performs OS acquisition in
KatanA.  That code is a host-effect oracle only.  Copying it into KLE or KUC
would violate the ownership boundary and fails the release gate even if its
observed result matches KatanA.

KatanA text copy/cut/paste also has an egui-native portion. The closure must
record its exact `text_edit.rs` input branch and the eframe/egui boundary as an
explicit unresolved external interaction until KUC's corresponding actual
RawInput contract and the KLE-owned host-E2E effect are linked. It must not
infer successful text-clipboard parity from KUC's generic request type, an
`egui` default, or the image-ingest routes.

The exact currently audited external branch families and their KUC replacement
boundary are recorded in
`docs/v0-1-0-locked-ui-semantic-reference-audit.md`. This audit is a required
source-universe input, not an optional compatibility note or a static leaf
catalog.

## Locked External UI Semantic Dependency Hold

`TextEditRenderer::render` constructs `egui::TextEdit::multiline` without a
custom layouter and KatanA's own source therefore does not enumerate all normal
text-edit semantics. The fixed KatanA `Cargo.lock` resolves both `eframe` and
`egui` to `0.36.1`; the `egui` package checksum is
`c977ac91dfaa651633fd9722e4ce9ccb32cda4c748b89a5cb57e504036e37c13`.
Consequently copy, cut, ordinary text paste, undo/redo, selection navigation,
composition handling, and their platform output are source-closure inputs, not
unmentioned behavior outside the v0.1.0 goal.

The platform event bridge is also part of this universe:
`vendor/egui-winit/src/lib.rs` contains `State::on_window_event`, `on_ime`, and
`on_keyboard_input`, including OS-dependent composition-range conversion and
clipboard-to-paste event conversion. The fixed Cargo manifest patches
`egui-winit` to this vendor directory; its lock entry has no registry checksum.
Authenticate these sources as fixed KatanA Git blobs, not as the upstream
registry package. Follow its declared modules and retain the unresolved winit,
clipboard, and accessibility boundaries until their actual semantics and native
input outcomes are proven. Merely enumerating this source does not close IME.

The generator must create an `external_ui_semantic_dependency` record for each
reachable behavior-bearing external API. For the current root this starts with
`egui::TextEdit::multiline`, `TextEdit::load_state`, `TextEdit::store_state`,
`egui::Event::Paste`, and `InputState::consume_shortcut` reached through the
typed `Context::input_mut` closure. The record contains the
fixed lock-package name/version/source/checksum, the resolved crate-source file
hashes and symbols, the KatanA invoking span, and the KUC replacement leafs.
It cannot be treated like a native file-dialog or pasteboard host boundary,
because it directly changes editor text/selection/history semantics.

If the exact registry source or a source-equivalent executable oracle cannot be
resolved from the locked package, every affected leaf remains unresolved. KUC
must reproduce the observed behavior through its own generic text surface and
platform catalog, never by retaining an egui fallback in KLE. The equivalence
suite uses public KLE RawInput, a current opaque KUC root record/AccessKit
snapshot, and an actual KatanA host effect; an egui unit result alone is only a
reference oracle. For source UI-state branches that the KUC replacement owns,
the equivalence suite instead records `kuc_retained_ui_effect` and a
bootstrapped KatanA no-mutation observation; it never uses direct `AppState`
mutation as a false host effect.

## Closure Generation Contract

The generated source closure starts with every file in the two direct seeds and
the cross-module seeds above.  It must then recursively add a same-crate source
file whenever an included source has one of these behavior-bearing edges:

| Edge | Required traversal and recorded rationale |
| --- | --- |
| UI host edge | editor mounting, search-bar mounting, RawInput/key/focus/hover/pointer/AccessKit handling, state needed to decide enabled or visible UI |
| action edge | `AppAction` or typed editor request definition, every dispatch arm, and the handler implementing its document/state/file/external effect |
| data-refresh edge | buffer, dirty/save, selection/cursor, diagnostics, search, preview, view/split/scroll, undo, and image-ingest state that changes an editor observable result |
| generic presentation edge | source that establishes menu order, labels, icon semantics, colors, bounds, syntax spans, gutter, accessibility or retained UI state; classify the target KUC component and either its `kuc_retained_ui_effect` or a typed host request instead of copying egui code into KLE |
| test-oracle edge | a KatanA test that asserts a leaf-specific result, together with its registration path; it is source evidence only and cannot replace KLE host E2E |
| external UI semantic edge | a locked external UI API that directly changes text, selection, history, focus, input consumption, platform output, accessibility or visible layout; resolve its exact package/source/symbol or fail closed, rather than classifying it as a host-effect terminus |

The generator must retain the incoming edge, source `path:symbol`, source span,
and whether the target is a leaf or a helper. A helper is allowed only with a
specific no-independent-visible-or-effect rationale. Any unresolved module,
macro expansion, dynamic dispatch boundary, or external crate boundary is a
fail-closed audit item. A generic UI-state branch may terminate only at a
current KUC retained-UI effect plus unchanged-host observation; a native KatanA
host effect may terminate at a source-spanned host-owned rationale. An external
UI semantic edge instead needs the lock-pinned dependency record and a KUC
replacement/equivalence leaf.

For each `AppAction` route, the generated record must contain four different
symbols when they are distinct: action definition, dispatch arm, handler, and
the final state/file/external-effect call. For a KatanA host-only external
effect (file dialog, clipboard acquisition, filesystem write, browser launch),
the record ends at that host boundary and names the effect type. It must not
copy the host operation into KLE or KUC merely to make the closure executable.

For an external boundary, naming the boundary is necessary but insufficient for
leaf completion. The generated leaf manifest also classifies it as
`native_external_host_effect` and requires the same-run native proof in
`docs/v0-1-0-external-host-e2e-design.md`: actual KLE RawInput, current KUC
record/AccessKit node, mapped KatanA action, native dialog or pasteboard trace,
and final KatanA file/document/state observation. The current host bridge's
`ExternalHostIntentUnavailable`, `UnsupportedAction`, pending dialog, or
ignored live-clipboard test is an unresolved leaf, not a host-boundary
rationale.

## Verified Action-Route Seeds

This table records measured high-risk roots from the fixed KatanA revision. It
is deliberately incomplete and is not a generated leaf manifest. It prevents
the migration from collapsing distinct host effects into one generic callback.

| Behavior | Action definition and dispatch | Handler and final effect | Required KLE/KUC proof |
| --- | --- | --- | --- |
| ordinary editable text | `app_action_types.rs:40 AppAction::UpdateBuffer` -> `app/action/dispatch.rs:58` | `app/document.rs:126 DocumentOps::handle_update_buffer`: active document update, preview refresh, search refresh, diagnostics debounce | public KLE content event reaches this unchanged action exactly once; editable/no-active/no-change and per-document effects are separate leafs |
| replace current or all | `app_action_types.rs:41-44 AppAction::ReplaceText` -> `app/action/dispatch.rs:59-60` | `app/document_edit.rs:25 DocumentEditOps::handle_replace_text`: range replace, undo external change, preview/search/diagnostic refresh | user-mandated KUC SearchStrip intent -> one-shot KLE forwarding -> host-derived current range -> `ReplaceText`; current/all/no-match/read-only/stale/document-switch are individual host-E2E leafs |
| Markdown toolbar/context action | `app_action_types.rs:185 AppAction::AuthorMarkdown` -> `app/action/dispatch_secondary.rs:57` | `app/action/process_authoring.rs:12 KatanaApp::handle_action_author_markdown` -> `MarkdownAuthoringOps::apply` -> `handle_update_buffer` and pending cursor | each operation, selection/caret/read-only guard, transformed bytes, dirty/preview effect, and restored character cursor need distinct proof |
| raw Markdown insert | editor action route -> authoring handler | `app/action/process_authoring.rs:58 KatanaApp::handle_action_insert_raw_markdown` -> `handle_update_buffer` and pending cursor | link/table/rule/image snippet insertion must not be merged with direct authoring transformation leafs |
| lint fix | `app_action_types.rs:45-46` -> `app/action/dispatch.rs:62-64` | `app/document_edit.rs:81 DocumentEditOps::handle_apply_lint_fixes` and `:99 handle_apply_lint_fixes_for_files` | diagnostic UI/action, exact document delta, no-fix guard, and refresh/undo result are separate leafs |
| linter documentation | `app_action_types.rs:50 OpenLinterDoc` -> `app/action/dispatch_tertiary.rs:90-92` | `app/action/process_linter.rs:6 KatanaApp::handle_open_linter_doc` -> host browser boundary | actual native external host effect, including failure/no-mutation branch, is required; URL ownership stays in KatanA |
| file image ingest | `app_action_types.rs:186 IngestImageFile` -> `app/action/dispatch_secondary.rs:59` | `app/action/image_ingest.rs:5 handle_action_ingest_image_file` -> native picker -> `process_image_ingest` | non-headless picker selection plus same-run asset, Markdown, dirty/explorer result; headless pending-dialog is not proof |
| clipboard image ingest | `app_action_types.rs:187 IngestClipboardImage` -> `app/action/dispatch_secondary.rs:60` | `app/action/image_ingest.rs:38 handle_action_ingest_clipboard_image` -> KatanA clipboard acquisition -> `process_image_ingest` | opaque correlated KUC/KLE request; raw image/file-list/file-URL/text priority and no-payload/error/no-mutation leafs |
| search lifecycle | `OpenDocSearch`, `ToggleDocSearch`, query/next/prev action arms in `app/action/dispatch_secondary.rs:82-105` | `app/action/process_helpers.rs:22 handle_action_doc_search_changed`, `:36 handle_action_doc_search_next`, `:56 handle_action_doc_search_prev` | open/focus, query, previous/next/wrap, Escape/zero-result and KUC visible current records are individual leafs |
| view and split controls | `SetSplitDirection`, `SetPaneOrder`, `SetViewMode`, `ToggleSplitMode`, `ToggleCodePreview` in `app/action/dispatch_secondary.rs:61-63,123-133` | active view/split state mutation, including `Split -> PreviewOnly -> CodeOnly -> PreviewOnly` cycle | one KUC command record plus persisted actual KatanA state and document/navigation interaction for each direction/order/mode leaf |
| save and refresh | `SaveDocument`, `RefreshDiagrams`, `RefreshDocument` in `app/action/dispatch.rs:77-81` | `app/document.rs:156 DocumentOps::handle_save_document`; `app/action/refresh_content.rs:91 handle_action_refresh_diagrams`, `:108 handle_action_refresh_document` | dirty/clean, virtual/file/save-error, active document, preview/diagram cache and host filesystem result are separate leafs |
| theme-driven diagram refresh continuation | `shell_ui/shell_ui_update.rs:161-163`: theme update with `pending_action == None`; `refresh_content.rs:91-105`; manual virtual-document refresh `:145-152` | only no-pending theme update enqueues `RefreshDiagrams`; cache reset always precedes active-document early return or preview refresh; manual refresh calls it for virtual paths only | `editor_host_continuation`, joined to the host theme/manual-refresh source rather than a fabricated KLE command. The generated branch catalog contains pending/non-pending, active/no-active, and virtual-manual/non-manual cases; each has actual cache/preview/no-effect observation. |
| Markdown formatting | `FormatMarkdownFile` / `FormatWorkspaceMarkdown` -> `app/action/dispatch_secondary.rs:107-111`; context entry in `views/panels/editor/context_menu.rs:28` | `app/action/process_markdown_formatting.rs:16 KatanaApp::handle_action_format_markdown_file` and workspace handler | markdown/HTML/non-markdown, active/inactive, changed/unchanged/error and actual document refresh are separate leafs |
| document selection and jump | `SelectDocument` / `SelectDocumentAndJump` -> `app/action/dispatch.rs:35-41` | `app/action/process_document.rs:148 handle_action_select_document`, `:154 handle_action_select_and_jump` -> `app/document.rs:19 DocumentOps::handle_select_document` | document isolation, search-modal close, CodeOnly transition when required, target line scroll and dirty-buffer preservation are separate leafs |
| tab traversal | `SelectNextTab` / `SelectPrevTab` -> `app/action/dispatch_secondary.rs:134-135` | `app/action/process_tabs.rs:170 handle_action_next_tab`, `:181 handle_action_prev_tab` -> document selection | no-document, one-document, wrap, active document and per-tab editor state are separate leafs |
| problems and diagnostics refresh | `ToggleProblemsPanel` / `RefreshDiagnostics` -> `app/action/dispatch_secondary.rs:105-106`; status-bar trigger in `views/top_bar/status_bar.rs:90` | panel state mutation and `app/action/process_diagnostics.rs:4 handle_action_refresh_diagnostics` | visible panel, scope, diagnostic source update, gutter/popup current records and actual linter review outcomes are separate leafs |
| Problems panel detail | `views/panels/problems/{ui,scope,diagnostics_renderer,bulk_fixes,fix_preview_renderer,fix_preview_model}.rs` | panel close, OpenTabs/ActiveTab scope, expand/collapse, empty/official-only file list, file disclosure, location jump, docs, entry/file/visible-scope fixes, preview missing/invalid/truncated branches | KUC owns generic retained `DiagnosticsPanel`/`StatusStrip`/preview/AX presentation; KLE forwards opaque diagnostic descriptors and host targets only. KatanA owns linter semantics, visible document set, batch construction, document/review/browser effects. Every source action and visibility predicate is a separate leaf. |
| document tab strip and tab state | `views/app_frame/tab_toolbar.rs` -> `views/top_bar::TabBar` -> `tab_bar/{mod,nav,tab_item,tab_context_menu,drag,group_header,group_header_popup}` | direct tab select, previous/next traversal, close/restore, close scope, pin, reorder and grouping actions are editor-document state only when they alter the current/open document model | KUC owns a generic retained `TabStrip`/menu/drag/AX presentation; KLE forwards opaque tab/group/placement/swatch targets and one-shot group naming only. KatanA host owns documents, groups, persistence and filesystem side effects. Every emitted document-changing action is a separate leaf; workspace/URL-only siblings must be source-spanned non-editor helpers, not silently omitted. |

The generator must extend this table until it contains every reachable route.
Rows must be expanded into branch-level records, not treated as an approved
aggregate route list.

### Verified Document Tab-Strip Interaction Seeds

The document tab strip is part of the editor surface. It is distinct from
KatanA's workspace-tab shell. The source-closure generator must follow the
document `TabBar` path below, then source-span any workspace-only sibling as
out of the editor scope. It must not silently omit either category.

| KatanA source route | Exact observed branch family | Required generated leaf proof and owner boundary |
| --- | --- | --- |
| `views/top_bar/tab_bar/mod.rs::TabBar::show` | tab strip uses a horizontally scrollable path only when estimated tab/group width exceeds available strip width; a pending active-tab scroll is consumed once; nav buttons follow the strip | KUC generic `TabStrip` owns retained overflow, focus and scroll presentation. KLE supplies document descriptors only. Each overflow/non-overflow, pending/no-pending active scroll, keyboard and AccessKit traversal branch records the same opaque root/AX frame and the resulting host document selection. |
| `tab_item/mod.rs::TabItem::show` | a collapsed group suppresses its member; inactive primary activation selects the document; close of a pinned document toggles pin while ordinary close requests close; changelog/demo/lint-review title variants, dirty indicator, tooltip and context root vary by descriptor | KUC owns tab bounds, hover, close affordance, tooltip, hit testing and AX. KLE copies opaque non-UI document descriptors only. KatanA owns selection, close confirmation/force-close, dirty state and persistence. Suppressed, active, inactive, pinned, dirty, virtual and special-title branches are separate leaves. |
| `tab_context_menu.rs::TabContextMenu` | close, close-other, close-all, close-right and close-left are always offered; virtual documents stop before pin/group/restore; non-virtual documents expose pin/unpin; only unpinned documents expose group actions; restore appears only with a recently closed tab; group menu differs for existing membership, available non-demo groups and no existing group | KUC owns a generic tab context menu and disabled/hidden presentation; KLE forwards opaque tab capability targets/events. Every menu item and every visibility predicate has a pointer, keyboard and AccessKit leaf. KatanA host E2E proves pinned preservation, recently-closed stack, active-document update, group cleanup and workspace-state save. |
| `tab_bar/drag.rs::TabDragHandler` | drag resolves visual to physical index, derives an adjacent target group, forces virtual documents out of a group, emits reorder only for a true move, and separately emits an in-place group change when order is unchanged | KUC owns generic drag target geometry and accessible alternative activation. KLE forwards an opaque typed reorder/group event without computing indices or group membership. Host E2E proves no-op, reorder, add-to-group, remove-from-group, virtual-document and persisted-order outcomes. |
| `tab_bar/nav.rs` and `app/action/process_tabs.rs` | navigation buttons select the previous/next document around the current index; host handling has no-document, one-document and wrap branches | KUC owns generic navigation controls; KLE forwards only the closed traversal request. KatanA owns path selection and per-document editor state. Pointer/keyboard/AX input and each no-op/wrap effect are distinct leaves. |
| `tab_bar/group_header.rs` | primary activation toggles collapse; secondary activation opens a non-demo group popup; pending inline rename opens and clears the host flag; primary press outside both header and popup closes it | KUC owns generic group-header/popup/focus/outside-click interaction. KLE mechanically forwards host-projected opaque group descriptors and an accepted opaque target only. KatanA host E2E proves collapse/inline-rename state and no unintended document mutation. |
| `tab_bar/group_header_popup.rs` | editable group name requests focus during inline rename; Enter closes the popup; changed name/color dispatch rename/recolor; ungroup and close-group are explicit actions | KUC owns generic text field, color palette, popup lifecycle, validation and AccessKit. KLE does not use a local `TextEdit`, palette or popup. Host E2E proves rename, recolor, ungroup, close-group and persistence branches. |
| `app/action/{dispatch,dispatch_secondary,dispatch_tertiary,process_tabs}.rs` and tab/group handlers | `SelectDocument`, close variants, force close, next/previous, pin, restore, reorder, create/add/remove/rename/recolor/ungroup/close/collapse group use distinct handler and persistence paths | Generated closure must follow each action definition, dispatch arm, handler, state/file effect and save path. A direct action variant, handler existence, or fixture state change alone is not host-effect evidence. |

#### Dirty-close confirmation reachability hold

At the fixed source revision, `app/action/process_document.rs:179-198`
dispatches a document close to either `force_close_document` or a
`state.layout.pending_close_confirm = Some(idx)` state transition according to
the dirty-tab setting. `AppAction::ForceCloseDocument` consumes that state in
`app/action/dispatch.rs:54-57`. A repository-wide fixed-revision search finds
no view source that reads `pending_close_confirm`; it appears only in layout
state, shortcut-context suppression, and the two action paths.

This is an explicit source-closure hold, not permission to invent a dialog:

| Branch | Required KLE/KUC behavior | Required proof |
| --- | --- | --- |
| clean or confirmation-disabled close | KUC emits the generic opaque close request; KLE forwards it; unchanged KatanA closes through its existing handler. | Individual KUC/AX/request/action/document/persistence record. |
| dirty confirmation-enabled close | The same close request reaches unchanged KatanA, which retains the current pending-close state and does not close the document in the visible editor path. KLE/KUC must not render a local confirmation dialog. | Individual actual KatanA pending-state/no-document-mutation record plus source span proving no current renderer. |
| `ForceCloseDocument` | No KLE/KUC event is generated until the closure finds an actual fixed-revision editor UI or shortcut origin. | The generator classifies the action as unmounted from the KLE root and rejects any locally invented control. |

If a later KatanA revision adds a renderer or shortcut route, its source hash and
leaf inventory change must be reviewed before KLE exposes a generic dialog.

KUC's public generic `TabStrip` contract therefore accepts only generic tab,
group, capability and presentation descriptors and emits typed tab events. It
owns retained layout, overflow, drag, menu, popup, text editing, color
selection, focus, hit testing and AccessKit. KLE must not introduce local tab
geometry, `egui::TextEdit`, palette, popup, drag or tab-menu code; it maps
KatanA document and group values to the generic descriptors and maps generic
events back to KatanA-specific typed requests. KatanA retains document/group
state, close confirmation, filesystem effects and persistence.

### Verified Status And Problems-Panel Seeds

The editor-relevant Problems surface is mounted by `MainPanels::show` before
the tab toolbar and central content. It must be rendered through the same KUC
opaque full-editor root. The source closure classifies KatanA export activity
as an input-free app-shell helper, but retains it as a source-spanned status
descriptor rather than silently dropping the visible branch.

| KatanA source route | Exact observed branch family | Required generated leaf proof and owner boundary |
| --- | --- | --- |
| `views/top_bar/status_bar.rs::StatusBar::show` | no status uses ready text; error/warning/success/info select different icon/color; dirty marker is conditional; host activity is absent or lists one/more filenames; Problems button always formats current count and emits toggle | KUC generic `StatusStrip` owns text/icon layout, tooltip, hit testing, font raster, AccessKit and retained panel toggle state. KLE forwards host-projected status/activity/diagnostic descriptors only. KatanA owns severity, dirty calculation and activity. Each visible state and pointer/keyboard/AX toggle is a separate leaf; generic UI-only leaves prove an unchanged KatanA host observation. |
| `views/panels/problems/ui.rs::ProblemsPanel::show` | closed panel returns before allocation; open panel supports close, OpenTabs/ActiveTab scope, expand/collapse all, visible-scope fix-all only with batches, zero official-diagnostic state, file list scroll and a one-shot expand-all reset | KUC generic `DiagnosticsPanel` owns retained panel/scope/scroll/disclosure state and accessibility. KLE forwards opaque descriptors and host-request targets only. KatanA owns visible document data, official diagnostic collection and fix effect. Closed/open, each scope, empty/nonempty, capability, transient expand and no-mutation branches are individual leaves. |
| `problems/scope.rs::ProblemScopeOps` | source displays OpenTabs and ActiveTab labels, but converts the selected display string back into `ProblemsScope` | KUC contract uses stable opaque scope keys, never label comparison, and retains the selection locally. KLE provides file scope-membership descriptors. Generator retains both source choices and verifies localized-label changes cannot alter scope selection or mutate KatanA state. |
| `problems/diagnostics_renderer.rs` | non-official diagnostics are hidden; each file has a persisted disclosure; a file fix requires an applicable batch; each official row provides location jump, docs only with a URL, and fix only when current content permits it | KUC owns generic file/diagnostic disclosure and presentation. KLE passes opaque entry/file/fix handles and has no linter predicate. KatanA host E2E proves exact select-and-jump, browser, single/file fix, refresh/review/undo, hidden/disabled and no-op effects. |
| `problems/fix_preview_{renderer,model}.rs` | hover/focus preview distinguishes missing content, invalid one-based line range, removed/added rows and truncation beyond ten rows | KUC owns generic preview layout, tooltip/focus, platform text and AccessKit. It accepts generic rows or performs only generic line-preview calculation. KLE does not copy the model. Source presence/missing/invalid/empty/truncated branches require independent root-frame and no-mutation proof. |
| `problems/bulk_fixes.rs` and action handlers | visible-scope/file batches filter official applicable fixes, then `ApplyLintFixesForFiles` reaches KatanA review/document mutation | KUC emits only an opaque request; KLE forwards it without paths, fix payload or source text. The actual host E2E proves batch scope, stale/no-fix rejection, document/review/undo/diagnostics refresh and no cross-document mutation. |

The generic KUC status/diagnostics contract is defined in
`docs/v0-1-0-kuc-diagnostics-status-design.md`. KLE local status/panel/list,
`egui::Panel`, `SegmentedStringToggle`, linter batch building, preview line
calculation and browser URL handling are architecture violations.

### Verified Direct-Input Decision Seeds

`views/panels/editor/text_edit.rs:12 TextEditRenderer::render` is the observed
KatanA primary-input root. Its post-edit decision order is a non-negotiable
source-audit seed:

1. editable focused paste with an intercepted raw image selects
   `IngestClipboardImage`;
2. editable focused paste that resolves to an image request also selects
   `IngestClipboardImage`, restores the prior cursor when text changed, and
   must not commit the apparent pasted text;
3. every other changed text value selects `UpdateBuffer(buffer.clone())`.

The second decision is derived from before/after input-event snapshots through
`EditorLogicOps::editor_clipboard_image_paste_requested_from_event_snapshots`
in `views/panels/editor/logic.rs:49` and
`EditorPasteOps::should_ingest_clipboard_image_paste` in `paste.rs:35`.
The KUC TextSurface must own equivalent generic focus, event ordering, IME,
caret, selection, hit-testing, and rendering behavior. KLE may only carry a
one-time opaque transport without observing its content or result; it may not
inspect OS clipboard data, parse `file://`, reconstruct cursor pixels, or
reimplement this condition.

### Verified Locked TextEdit Standard-Operation Seeds

KatanA's direct call is plain `egui::TextEdit::multiline`; it does **not** call
`code_editor()` or `lock_focus(true)`. The locked external semantic dependency
is `egui 0.36.1`, resolved by KatanA's `Cargo.lock`. Its default
`EventFilter` has `horizontal_arrows = true`, `vertical_arrows = true`, and
`tab = false`. Consequently Tab and Shift+Tab are not editor indentation
routes in this KatanA revision. Any KLE/KUC implementation that inserts or
removes a tab character for this source-derived route is a behavior regression,
not an editor improvement. The generated closure must also resolve the
surrounding KatanA focus route; it may not assume which sibling receives focus.
The recorded registry checksum is
`c977ac91dfaa651633fd9722e4ce9ccb32cda4c748b89a5cb57e504036e37c13`.
For this audit snapshot, `widgets/text_edit/builder.rs` SHA-256 is
`1c2198aa1c397d40e0e384cb6e69252822a6bdf870b8b3d9ddafb2dfd5f98cbd`
and `text_selection/cursor_range.rs` SHA-256 is
`ba2d32c55f92c472a58ea51fa2df7798cf8dd95dbf3b9bb414fa3119edba0161`.
The canonical generator must calculate and compare these values; this record
is not a static allow-list.

| Fixed external source branch | Observed inherited KatanA behavior | Required generated leaf discipline |
| --- | --- | --- |
| `widgets/text_edit/builder.rs::events`, `Event::{Copy,Cut}` | copy writes only a non-empty selected text range; cut additionally deletes that range; empty selection is a no-op | non-empty/empty and editable/reference outcomes stay separate. KUC owns platform clipboard and selection mechanics; KLE never accesses an OS clipboard. |
| `builder.rs::events`, `Event::{Paste,Text}` | empty paste is a no-op; non-empty multiline paste replaces selection; ordinary text ignores empty/CR/LF because Enter owns newline insertion | each empty/non-empty, selection/no-selection, CR/LF-reject, multiline paste and regular text branch requires its own record/AX/host event. KatanA image-paste interception remains the prior higher-priority branch. |
| `builder.rs::events`, `Key::Enter` | focused multiline input replaces selection with one newline | `text.enter-newline` proves one semantic content mutation and exact Unicode-safe cursor/history outcome. |
| `builder.rs` default `EventFilter` | Tab is not filtered into the text-edit event handler; the direct KatanA call has no `code_editor`/`lock_focus(true)` override | `text.tab-not-editor-indent` and `text.shift-tab-not-editor-unindent` prove no document mutation and the separately resolved surrounding focus route. No indent/unindent leaf is admitted for this KatanA source. |
| `builder.rs::events`, command `Z`, command `Y`, shift-command `Z` | undo and both redo shortcuts operate only when the retained undo state has a matching snapshot | undo, redo-Y, redo-shift-Z and unavailable-history no-op are individual branches, with document identity isolation and no KLE history implementation. |
| `builder.rs::check_for_mutating_key_press` | Backspace/Delete distinguish selection, character, word and paragraph deletion; Ctrl-H/K/U/W add platform-specific delete variants | the AST closure expands each key/modifier/selection/OS branch separately. `text.delete-branch-family` is a generation root, not a passing aggregate. KUC owns Unicode-safe generic editing; KLE has no string/byte mutation helper. |
| `text_selection/cursor_range.rs::{on_event,move_single_cursor}` | command-A, collapsed-selection left/right, arrows, Home/End, shift extension, command/alt/ctrl word or document movement, and macOS Ctrl-P/N/B/F/A/E choose different selection/caret transitions | every static match arm and modifier predicate becomes a leaf. `text.cursor-navigation-family` cannot be satisfied by a count, a shared selector, or a plain ASCII case. Japanese, VS16 and ZWJ layout/caret cases are mandatory for each relevant class. |
| `builder.rs` pointer branch and `text_selection` pointer interaction closure | pointer hover sets text affordance; click/drag changes focus/selection and a moving pointer can preview a caret | source closure must enter the pointer helper and enumerate click/drag/multi-click/selection/focus branches before any pointer parity claim. KUC owns hit testing and selection geometry; KLE owns none. |
| `builder.rs` IME branch and rendered IME output | enabled/disabled, preedit, empty preedit/commit guards, CR/LF rejection, commit, DeleteSurrounding, active range, cursor rectangle and accessibility update have independent branches | IME state is KUC root state. Each branch requires current root raster/record/AccessKit evidence and exact, non-duplicated host mutation or no-mutation observation. |
| `builder.rs` relayout and `accesskit_text::update_accesskit_for_text_widget` | every text or selection mutation relayouts before the same frame is published; selection-only changes update accessible text-selection output | KUC must keep layout, caret, glyph raster and AX in one current root frame. Any KLE-side after-frame repair, glyph fallback or shape-count evidence is rejected. |

`views/panels/editor/logic_scroll.rs:6 EditorLogicOps::handle_scroll_to_line`
contains separate TOC-top alignment, ordinary center alignment,
already-consumed target suppression, and absent-line behavior. Its
`update_scroll_sync` at `:88` separately handles preview consumption,
no-content/echo suppression, pending-jump suppression, dead-zone comparison,
and editor-origin update. KUC owns the generic scroll and bounds contract;
KLE forwards only an opaque host-issued scroll target without local coordinate
arithmetic or source-state interpretation.

### Verified Primary Text Surface Configuration and Frame Ordering

`TextEditRenderer::render` is the current KatanA compositor for the editor
frame. The egui calls are reference-only, but their observable same-frame
ordering and configuration are source-derived requirements for the KUC retained
root. Host-projected neutral presentation reaches KUC through KLE without KLE
inspection or retention. KLE must not recreate any child surface, spacer,
layout order, action arbitration, or semantic state.

| Source branch | Observed KatanA behavior | Required KUC/KLE separation and leaf proof |
| --- | --- | --- |
| `text_edit.rs:15-72 TextEditRenderer::render` | allocates a dedicated line/diagnostic-gutter column; the multiline text surface is editable only when the document is not a reference, uses the document-specific undo identity, monospace font size, infinite width, initial visible-row target, no frame, and matching right margin | KUC owns retained text/gutter layout, style interpretation, focus identity, raster and AccessKit. The host projects neutral read-only/capability/presentation descriptors directly to KUC through KLE without KLE retention of document identity, typography, spacing, or content. Editable/reference, first mount, resize, initial rows, line-gutter reservation, font/scale change, focus/document switch and whole-frame bounds are distinct root-frame leaves. KLE must not retain the source's 52-pixel gutter/margin or egui builder calls. |
| `text_edit.rs:73-98` | after the text surface, KatanA attaches its context menu, carries the current cursor range, and applies a pending authoring cursor through the document-specific text identity | KUC owns same-frame context-menu attachment, selection/caret and focus update. KLE mechanically forwards the opaque host projection and accepted opaque target only; it neither reads nor stores cursor/selection/result values. Context target, no/current/pending cursor, document identity, authoring restore and reference read-only behavior require separate RawInput/AX/host leaves. |
| `text_edit.rs:99-108` | an editor click causes a logical-line target only when preview/editor synchronization is enabled and a cursor is present | KatanA owns the synchronization policy; KUC owns hit/caret geometry. sync enabled/disabled, click/no click, cursor present/absent, Unicode caret line and resulting preview/editor host state are separate leaves. |
| `text_edit.rs:109-166` | the root renders current-line decoration, preview-linked hover, search matches, inline diagnostics, line diagnostics/gutter, then selection toolbar; diagnostic hover suppresses the toolbar | KUC root owns this exact internal layer/lifetime order and produces one final frame/record/AccessKit tree. KLE has no child painter, z-order, diagnostic hit region or toolbar-suppression state. Each layer presence/absence, overlap, suppression, pointer/keyboard/AX route and document change has a distinct leaf. |
| `text_edit.rs:168-189` | image-paste arbitration occurs after the frame using before/after focus and event snapshots; image result wins over a text change, restores the prior cursor when needed, otherwise text change emits `UpdateBuffer`; only then does the source consume scroll target and extract anchors | KUC owns generic input transaction ordering and current surface facts; KLE only performs correlated nonpersistent opaque transit; KatanA host alone decides external image acquisition and buffer effects. Image/text/no-payload/read-only/focus transition/changed text/cursor restore/ordinary content update/scroll acknowledgement/anchor record are independent leafs. No local KLE event replay or post-frame aggregate repair is allowed. |

### Verified Gutter, Decoration, Layout, and Syntax Seeds

The following are source-derived behavior seeds from the fixed KatanA
revision.  They specify observable semantics, not an instruction to transplant
the egui implementation.  KUC is the sole owner of shaping, line geometry,
pixel placement, colors, hit testing, text rasterization, and AccessKit nodes.
KatanA supplies document semantics and syntax spans; the host projects opaque
generic presentation envelopes to KUC through a non-observed KLE transit.
KLE neither interprets nor maps those inputs.

| Source branch | Observed KatanA behavior | Required KUC/KLE separation and leaf proof |
| --- | --- | --- |
| `line_numbers.rs:28-94 EditorLineNumbers::render` | walks laid-out rows, emits a number only at a paragraph start, clips to the expanded viewport, and delegates the same visible row to diagnostic rendering before the number | KUC owns wrapped-row to logical-line derivation, clipping, ordered gutter records, and their AccessKit nodes. The host projects opaque current presentation descriptors through KLE without KLE document/diagnostic interpretation. Paragraph-start/non-start, visible/clipped, empty/final-line, and diagnostic-before-number ordering are distinct record and actual-input leafs. |
| `line_numbers.rs:116-149 render_row_number` | the active cursor row is visually distinct; a row click requests that logical line; hover exposes a pointing cursor | KUC owns active/hover/click state, geometry and pointer semantics. KLE forwards only the resulting opaque host target. Active/inactive, hover/leave, click, absent cursor, and no-op repeated target each need RawInput, record, AccessKit, and KatanA scroll-state proof. |
| `line_numbers.rs:152-165 row_number_label_rect` | reserves `ACTION_ICON_GUTTER_WIDTH`; a line-number hit rectangle cannot cover the diagnostic icon gutter | KUC owns this collision invariant as a generic `GutterLayout` constraint. It must be property-tested over widths, zoom/font metrics, RTL-safe bounds where supported, and marker/no-marker states; no KLE pixel constants or hit rectangles are permitted. |
| `decorations.rs:17-49 render_cursor_line` | a present cursor maps character index to logical line, updates active-editor-line state, and draws the full editor row; an absent cursor clears that state | KUC owns selection/caret geometry and generic `CurrentLine` decoration. KLE maps no cursor pixels or palette. Cursor present/absent, multi-byte Japanese/VS16/ZWJ caret positions, wrapped line, and document switch must prove semantic record, AX state, and host-visible scroll/current-line state. |
| `decorations.rs:52-85 render_hovered_lines` | every preview-derived logical line range is converted to character range; invalid ranges are skipped and valid ranges draw full-width hover rows | KUC owns range-to-layout conversion and hover decoration painting; KLE forwards only a host-projected opaque source target/revision. Empty/invalid, one/multi-line, overlapping, wrapped, and preview-origin-cleared cases are individual leaves. |
| `decorations.rs:87-185 render_search_matches` | every document-search character range is mapped across one or more layout rows; the active index uses a distinct semantic color and non-positive rectangles are suppressed | KUC owns multi-row match geometry, active/inactive semantic roles and suppression. KLE must never calculate match pixels or reassemble paint plans. Single/multi-row, active/inactive, range boundary, zero-width/stale range, scrolling, Japanese and emoji character ranges each need exact record/AX/host-E2E proof. |
| `layout.rs:15-55 EditorLayouter::layout` | extension lookup selects a syntax, falling back to Markdown; missing syntax yields base text formatting. Theme lookup first tries the current theme then an available theme. | KatanA remains the source of document extension/theme/syntax semantics; KUC owns layout and rendering of host-projected generic presentation descriptors. The closure must enumerate extension-known/unknown, theme-current/fallback/unavailable, wrap-width change, and plain-text fallback branches. KLE transparently transits the current opaque descriptor only and never receives an egui `LayoutJob`, font, theme cache, span, or syntax configuration. |
| `layout.rs:60-83 append_highlighted_line` | each highlighted line produces styled runs; a highlighter error renders that line using base text style rather than aborting the editor | KUC renders the host-projected generic presentation stream and has a documented generic invalid-input policy. KatanA owns highlighter errors; KLE forwards only the opaque current descriptor/correlation. Success/error adjacent-line offsets and continuous editing after an error require separate host-E2E leaves. |
| `syntax.rs:37-59 MarkdownSyntaxHighlighter::highlight` | Markdown syntax is selected with plain-text fallback; an unavailable theme returns an empty span list; a line highlighter error advances the byte offset without creating spans | KatanA owns this highlighter and byte-range semantics. KUC validates and consumes spans without interpreting Markdown, retains unstyled source where spans are empty, and exposes layout/accessibility state. Theme-empty, syntax fallback, successful spans, error-offset continuation, UTF-8 Japanese, `⭐️` VS16, and ZWJ boundaries are individual parity leaves. |
| `syntax.rs:63-84 style_to_token_kind/append_ranges` | bold maps to `Heading`, italic maps to `Comment`, all other styles map to `Default`; spans preserve source byte offsets | The host projection emits neutral semantic-span descriptors only when the reachability hold is resolved; KLE forwards them without KatanA type dependency or local highlighting. KUC maps token kinds to presentation through its theme contract. The generated closure must prove source-byte range validity, monotonic non-overlap, style-priority, and unchanged source text; it must reject malformed or lossy conversion. |

### Verified Text Coordinate, Anchor, and Padding Seeds

`utils.rs` is behavior-bearing source, not a license to retain a KLE text
layout helper. The fixed KatanA implementation mixes byte, character, logical
line, and laid-out-row values. The generated closure must retain each unit and
conversion boundary, while KUC owns the generic safe conversion, layout-anchor,
padding, raster and hit-test implementation.

| Source branch | Observed KatanA behavior | Required KUC/KLE separation and leaf proof |
| --- | --- | --- |
| `utils.rs:char_index_to_line` | counts Unicode scalar values up to a character offset and maps newline count to a zero-based logical line | KUC exposes an authoritative generic text-coordinate record. ASCII/Japanese/VS16/ZWJ/combining mark, before/after newline, EOF and out-of-range source requests are distinct conversion leaves. KLE must not reimplement the scan. |
| `utils.rs:line_to_byte_index` and `line_to_char_index` | line starts map separately to UTF-8 byte and character offsets; an exact trailing empty line resolves to end-of-buffer and an absent line returns `None` | KUC validates and records both coordinate units when required by generic text operations. KLE forwards no coordinate value and only relays an opaque host target where a host effect is required; trailing newline, empty buffer, Japanese/VS16/ZWJ, invalid line and byte/character disagreement need independent record and host-E2E tests. |
| `utils.rs:line_col_to_byte_index` and `line_col_to_char_index` | linter columns are one-based character positions; the conversion saturates the zero column, stops at newline and returns the terminal position at EOF | KatanA owns diagnostic line/column semantics; KUC owns generic conversion and range validation. The leaf catalog must distinguish column zero, first/last scalar, past-line, newline, EOF, multi-byte and malformed/stale diagnostic data without accepting a KLE local mapper. |
| `utils.rs:line_range_to_char_range` | inverted line ranges are rejected; otherwise the end is the line-end newline or document end | KUC owns logical-range-to-text-range conversion for generic decorations. Inverted, empty, one/multi-line, final unterminated line, final empty line and Unicode boundary cases require KUC record/AX and KatanA decoration/scroll evidence. |
| `utils.rs:extract_line_anchors` | records only the first laid-out row of each logical paragraph; subsequent wrapped rows do not create anchors and `ends_with_newline` controls the next anchor | KUC derives anchors from its one text layout and publishes them with the root frame. Wrapped/unwrapped, empty/final line, changing width/scale, Japanese/VS16/ZWJ, repeated layout and scroll synchronization must prove no duplicate or drifting anchors. KLE may not calculate row positions. |
| `utils.rs:render_editor_padding` | inserts positive host scroll ghost-space, then reserves half the clip height as bottom padding | KUC owns generic scroll-past-end layout and root bounds. KatanA supplies only typed scroll-policy facts; positive/zero ghost-space, viewport resize, repeated frame, focus/selection at final line and preview/editor sync need pixel/record/host effect leaves. No KLE `egui::Ui::add_space` or padding constant is permitted. |

### Verified Theme and Scroll Coordinator Seeds

| Source branch | Observed KatanA behavior | Required KUC/KLE separation and leaf proof |
| --- | --- | --- |
| `logic_colors.rs:resolve_editor_colors` | a KatanA theme record supplies code background/text/selection/current/hover/line-number roles when present; otherwise the source uses egui visual defaults and leaves optional semantic roles absent | Host-projected neutral theme roles reach KUC without KLE conversion or storage; KUC owns generic role resolution and rasterization. Theme-present/theme-absent, every optional role present/absent, and document/theme change require root record/pixel/AX leaves. KLE may not read egui temporary data or choose fallback colors. |
| `logic_colors.rs:current_line_highlight_color` | an explicit themed current-line color wins; otherwise dark mode uses white alpha 15 and light mode black alpha 15 | KUC owns the generic semantic fallback implementation with host-provided theme mode/token policy. Explicit/dark fallback/light fallback are separate pixel and semantic-record leaves; hard-coded KLE colors or alpha constants are prohibited. |
| `logic_colors.rs:hover_line_highlight_color` | an explicit themed hover color wins; otherwise dark mode uses white alpha 10 and light mode black alpha 10 | KUC owns the generic hover role with the same strict typed policy. Explicit/dark fallback/light fallback must not be merged with current-line behavior, because the source alpha and role differ. |
| `logic_scroll.rs:handle_scroll_to_line` and `scroll_to_line_with_align` | TOC target has priority and aligns to top; ordinary target aligns center, is deduplicated by `last_scroll_to_line`, and invalid logical lines do not scroll | KatanA owns TOC/ordinary request semantics and persistence. KUC owns line-to-layout-rect conversion and scrolling. TOC priority/top, ordinary/center, repeated target, new target, invalid target, empty/final line, Japanese/VS16/ZWJ target, and action acknowledgement are distinct RawInput/root/host leaves. |
| `logic_scroll.rs:update_scroll_sync` | always updates editor height/offset/anchors; preview consumption clears source, zero max/echo/pending jumps suppress propagation, and only movement beyond dead zone becomes Editor source | KatanA owns scroll synchronization state and source selection; KUC emits current layout/offset/anchor facts and applies requested scroll. Preview-consuming/non-consuming, zero/positive max, echo/non-echo, pending ordinary/TOC jump, same/different segment, at/below/above dead zone, resize and document switch need individual record and host-coordinator leaves. |

The fixed KatanA egui fallback colors and layout primitives are reference-only.
Their observable fallback branches must be reproduced through KUC semantic
tokens and KUC's platform font catalog; accepting egui glyph output, a KLE
font fallback, or a local KLE shape/pixel conversion fails the boundary gate.

### Verified Markdown Command and Authoring Seeds

The floating selection toolbar shown in the KatanA reference is one retained
KUC `CommandChrome` interaction, not a KLE-owned collection of buttons.  Its
commands, menu hierarchy, focus lifecycle, and host effects are all part of
parity.  The source labels/icons remain KatanA presentation data; KUC provides
the generic control, popup, submenu, disabled-state, input and accessibility
behavior.

| Source branch | Observed KatanA behavior | Required KUC/KLE separation and leaf proof |
| --- | --- | --- |
| `toolbar.rs:23-43 show` | command order is inline controls, separator, headings, separator, list/quote, separator, code menu, separator, image; all groups are centered and shrink to fit | KUC owns retained grouping, separator semantics, compact overflow/clamp, icon/button hit targets, keyboard navigation and AX tree. The host projects ordered opaque presentation/capability/target descriptors; KLE copies them without interpreting command semantics. Order, separators, narrow viewport, zoom, focus order, and every command locator are separate records; a screenshot-only group is not proof. |
| `toolbar.rs:55-91 inline_group` | Bold, Italic, Strikethrough and InlineCode are disabled unless a non-empty selection exists | KUC owns generic selection-gated command availability. The host projects selection capability; KLE forwards an accepted opaque target once. No-selection/selection/read-only/action/cancel keyboard cases must have separate RawInput, AX disabled/enabled state and KatanA transform proof. |
| `toolbar.rs:93-161 heading_group/list_group/block_group/image_group` | H1/H2/H3, bullet list, numbered list, block quote, code dropdown and file image use independent actions; headings/list/block commands are visually offered without the inline selection guard | KUC owns every generic command control and code submenu. KLE has neither command IDs nor KatanA action DTOs; it forwards an opaque host-issued target only. Each command needs an individual transformed-document or native-host proof, including caret/selection/read-only host rejection; code-kind inventory must be generated from `CodeBlockKind::all()` rather than frozen at an observed count. |
| `toolbar_popup.rs:20-80 ToolbarPopup::show` | toolbar appears only for editable cursor state without diagnostic suppression while editor/popup focus is retained; click/click-elsewhere closes it, code child state keeps it alive and popup input blocks the underlying editor | KUC owns popup state machine, focus transfer, outside dismissal, modal input occlusion, nested-menu lifetime and AX focus. KLE must not retain popup booleans or calculate rectangle input interception. Each transition, including toolbar child focus, diagnostic suppression, lost cursor, and close-after-command is a distinct leaf. |
| `toolbar_popup.rs:91-103 popup_position` | anchor is cursor-bottom plus gap, clamped inside viewport edges | KUC owns generic anchor and collision placement derived from its text layout, including DPI/zoom/font/IME geometry. KLE passes no cursor pixels, estimated dimensions or clamps. Four-edge clamping and live reflow are property and actual-input tests, not KLE constants. |
| `code_block_menu.rs:13-32 CodeBlockMenuOps::show_items` | each current `CodeBlockKind::all()` value is displayed and click emits its own `AuthorMarkdown(CodeBlock(kind))`, then closes | KUC owns menu rendering and close behavior; the host projects an opaque descriptor for each current item, and KLE forwards the selected target without carrying `CodeBlockKind`. Generated inventory order, each item, outside dismissal, keyboard activation, failed/cancelled selection and final Markdown/cursor state are separate leaves. |
| `code_block_menu.rs:39-81 CodeBlockMenuPopupOps` | code menu opens on click or pointer-down, uses click-outside close, labels its button for accessibility, and closes after selection | KUC owns pointer-down/click distinction, popup lifecycle and accessible button/menu roles. KLE has no egui memory IDs or popup state. The generated branch catalog must include open, repeat activation, child click, outside click, focus traversal and default-closed state. |
| `context_menu.rs:10-43 render` | root order is Save, conditional Format, separator, Edit submenu, Ingest submenu. Format exists only when editable and extension is `md` or `markdown`, case-insensitively. | KUC owns generic context-menu hierarchy, ordering, enabled state, shortcut/secondary-pointer/AX invocation and closing. Host-projected neutral capability descriptors reach KUC through KLE without semantic conversion. Save/format each require an actual KatanA host effect; source path extension parsing and file I/O remain host-owned. |
| `context_menu.rs:54-149 render_inline/render_structure/render_insert` | Edit contains selection-gated inline operations, then H1/H2/H3, bullet/numbered/quote/code/horizontal-rule, then link/table; accepted action closes the menu | KUC owns hierarchy and command affordances. KatanA owns Markdown operation semantics. Individual menu leaves must prove label/order/availability/activation/close plus the exact KatanA transformation. KLE may not implement `AuthoringUtils` or source byte-edit rules. |
| `context_menu_image_ingest.rs:6-33` | Ingest exposes file image and clipboard image; clipboard image is disabled when KatanA host payload availability is false, and accepted actions close the whole context menu | KUC owns a generic availability-gated ingest command. KLE forwards a selected opaque target once and carries no result/payload; only KatanA owns payload probing, native clipboard/file acquisition, asset insertion and errors. File/clipboard, enabled/disabled, accepted/cancelled/failed, no-mutation and menu-close must be distinct native-host leaves. |
| `authoring.rs:16-81 MarkdownAuthoringOps::apply` and `authoring_utils.rs` | KatanA clamps ordering/ranges then performs operation-specific inline wraps, prefixes, numbered lines, block fences, snippets, link/table behavior and byte cursor result | This transform is KatanA host domain logic. KUC only emits generic opaque-target activations and renders the resulting content/caret; KLE only forwards the target. Every operation, empty/non-empty selection, multi-line/empty-line, Unicode byte-boundary, code kind, and cursor restoration needs a source branch plus actual host effect. No KUC or KLE Markdown transformer is permitted. |

`KUC CommandChrome` must accept the complete opaque command descriptor and
semantic capability state as one root input and emit one-time opaque activation
records. It must not depend on KatanA types. KLE carries only the
host-projected opaque target/revision/correlation at this boundary and must not
retain toolbar/context-menu child state, compose child paint plans, render
fallback controls, or derive a command DTO.

#### Fixed-Revision Markdown Enumeration And Handler Branches

The following names and order are source-audit facts from
`markdown_authoring_op.rs`, not KLE/KUC enum or label requirements. The
generator parses these definitions and the `all()` construction directly. A
name/order/count change invalidates the generated action-origin and branch
catalog until a new host projection and proof are supplied.

| Source inventory | Fixed-revision values | Required generated closure |
| --- | --- | --- |
| `MarkdownAuthoringOp` | `Bold`, `Italic`, `Strikethrough`, `InlineCode`, `Heading1`, `Heading2`, `Heading3`, `BulletList`, `NumberedList`, `Blockquote`, `CodeBlock(kind)`, `HorizontalRule`, `InsertLink`, `InsertTable` | one action-origin and effect leaf for every operation and every physical toolbar/context-menu/shortcut construction; no KLE operation enum or command string. |
| `CodeBlockKind::ALL` order | `Text`, `Markdown`, `Bash`, `Zsh`, `Mermaid`, `Drawio`, `Plantuml`, `Json`, `Yaml`, `Toml`, `Rust`, `Typescript`, `Javascript`, `Python`, `Html`, `Css`, `Sql` | one generated code-menu leaf per source item and a separate `edit.code_block` shortcut leaf; KUC sees only host-projected opaque item descriptors, never this enum or `info_string`. |
| `CodeBlockKind::{info_string,display_label}` | each current label is derived from the source info string | label/order changes are source catalog changes. KUC displays only the host-projected presentation text and does not translate language identifiers. |

`process_authoring.rs:handle_action_author_markdown` adds non-presentation
branches that must be in the host-effect catalog: absent active document is a
no-mutation return; reference document is a no-mutation return; a current
cursor range is a host-only character-to-byte conversion while an absent range
uses the buffer end; every accepted operation flows through
`handle_update_buffer`; and the final cursor restoration is converted back to
character positions and stored for the next KatanA frame. Japanese, VS16 emoji,
ZWJ, empty/non-empty selection, multi-line selection, stale projection and
document switch must exercise these exact host branches. KLE receives none of
the source cursor/range values and must not reproduce the conversion.

### Verified Mount, Scroll, and Syntax-Reachability Seeds

| Source branch | Observed KatanA behavior | Required KUC/KLE separation and leaf proof |
| --- | --- | --- |
| `ui.rs:49-136 EditorContent::show` | no document produces no editor content. An open document is cloned into the editor frame; it combines the scroll surface, text edit, decorations, gutter, command popup, padding, anchors and post-frame scroll synchronization. | KUC owns the single opaque editor root and every internal child lifetime. KLE forwards host-projected opaque descriptors in and closed opaque events out; it must not clone/reconcile visual child state or compose artifacts. No-document, document mount, document switch, read-only reference and active document are individual actual-host leaves. |
| `ui.rs:77-90` | a pending editor or TOC target disables animated scrolling; preview-origin sync without an editor jump prepositions the editor from logical scroll state | KUC owns generic scroll animation and logical/physical conversion contract. KLE neither receives offsets nor interprets source/target state. Editor jump, TOC jump, preview consume, both target fields, and no target are separate state and visual-motion leaves. |
| `logic_scroll.rs:6-31 handle_scroll_to_line` | a TOC target has priority and aligns to top; ordinary target aligns center and is suppressed when already consumed | KUC owns logical-line target layout and one-shot suppression. KLE does not compute line character offsets or rectangles. TOC priority, ordinary target, repeated target, absent line, wrapped target, and clearing after host consumption need individual KUC record and KatanA state proof. |
| `logic_scroll.rs:88-125 update_scroll_sync` | records editor dimensions/anchors; consuming preview resets source; no scroll range, echo, pending editor/TOC jump and dead-zone movement suppress editor-origin updates | KUC owns its geometry and physical scroll state; KatanA host owns shared preview/document scroll coordination. KLE forwards no offset, anchor, segment, or source value. Each suppression branch and source transition must be generated and exercised through real host frames, including rapid bidirectional input. |
| `utils.rs:5-114` | logical line/column helpers distinguish bytes from characters, permit an empty final line and reject reversed line ranges | KatanA host/source semantics define document positions. KUC's text model must preserve equivalent Unicode-safe logical position behavior but may not delegate it to KLE. Japanese, combining marks, VS16 emoji and ZWJ must be exact position property cases; a byte/character conversion loss is a release failure. |
| `utils.rs:116-145` | first visual row per logical line becomes an anchor and editor padding includes mapper-provided ghost space plus half a viewport | KUC owns anchors and padding as generic scroll-surface behavior. KLE cannot put in pixel constants. Empty/one/many wrapped lines, mapper ghost-space zero/nonzero, resize and zoom require deterministic frame record and motion checks. |
| `logic_colors.rs:5-59` | KatanA supplies code semantic colors when theme data exists; otherwise it uses egui fallbacks. Current-line/hover colors also branch by dark/light mode when no themed color exists. | KUC owns color resolution from a generic theme token input and all rasterization. The host projection supplies neutral semantic tokens; KLE forwards them without KatanA color types. Each themed/fallback and dark/light case needs KUC record and image-pixel semantic assertions; no egui fallback palette or local KLE color code is allowed. |
| `types.rs:31-76 MarkdownEditorWidget::frame_config` | this adapter copies font size/dark-mode config into `EditorConfig`, whose default highlighter is `NoopSyntaxHighlighter`; the current local test explicitly asserts empty spans | This is KatanA adapter residue, not a KLE implementation template. KUC owns platform text configuration/catalog. The closure must resolve the actual configuration consumer; KLE may not recreate `EditorWidget`, inject a fallback font or use its empty-span test as syntax evidence. |

#### Syntax-Reachability Audit Hold

At the fixed revision, an all-Rust static reference search finds
`EditorLayouter` and `MarkdownSyntaxHighlighter` only at their definitions and
documentation references.  The actual `TextEditRenderer` constructs
`egui::TextEdit::multiline` without a layouter, while
`MarkdownEditorWidget::frame_config` retains the default no-op highlighter.
This is an observed reachability gap, not permission to discard either source
file or to claim syntax-highlighted output.

The generated closure must resolve every module/re-export/macro/dynamic edge
for these two definitions.  It may classify a definition as unreachable helper
only when the generated call graph proves that outcome at the recorded source
tree and records its hash/span/rationale.  Until then, all syntax presentation
claims remain open.  If an external or runtime edge is found, it creates its
own source-derived leaf and KUC semantic-span acceptance case.  KLE must not
fill this ambiguity by adding an independent highlighter.

### Verified Diagnostics and Paste Branch Seeds

| Source branch | Observed KatanA result | Required parity classification |
| --- | --- | --- |
| `diagnostics_ui.rs:7-68 EditorDiagnostics::render_diagnostics` | unofficial diagnostics are skipped; an official diagnostic's 1-based line/column range is converted to character indices before visual-row layout; unresolvable bounds produce no inline annotation | KatanA owns diagnostic semantic range data. KUC owns generic UTF-8-safe range validation, multi-row annotation records, pointer hit regions, semantic severity roles and AccessKit state. Official/unofficial, resolvable/unresolvable, single/multi-row, Japanese/VS16/ZWJ columns and stale-document range are separate leaves. |
| `diagnostics_ui.rs:70-153 paint_squiggly` and `diagnostics_hover.rs:7-34 draw_wave` | each positive-width visual-row fragment receives a severity-colored wave and hover target; non-positive/missing rows create neither. A hover displays the diagnostic UI with same-line grouping. | KUC owns generic diagnostic annotation geometry, wave/stroke raster and hover presentation. KatanA alone resolves semantic diagnostics and projects an opaque current descriptor; KLE transparently transits that descriptor without reading ranges, actions, or severity and never draws paths or hit rectangles. Fragment ordering, wrapped/multi-row continuation, hover enter/leave, zero-width suppression, severity and grouped popup lifecycle must have RawInput, KUC record/AX and KatanA effect proof. |
| `row_diagnostics.rs:RowDiagnosticsRenderer::action_icon_diagnostics` | only diagnostics with `official_meta` and `range.start_line == line_number` create a gutter action; non-fixable official diagnostics remain included | official/non-official, start/non-start, fixable/non-fixable, and empty-row are distinct KUC record/AccessKit/host-effect leafs |
| `RowDiagnosticsRenderer::render` severity mapping | `Error > Warning > Info` selects the marker semantic role; no matching diagnostic returns without a marker | KUC owns generic marker priority and raster/hit target; the host projects opaque current marker descriptors and KLE performs no diagnostic interpretation, geometry, or color computation |
| `diagnostics_popup.rs:DiagnosticsPopupOps::show` | icon hover/click opens; outside click closes unless popup hovered; selected action closes | hover/click/open/outside/action/focus interaction must have separate RawInput and AccessKit scenarios |
| `diagnostics_hover.rs:DiagnosticsHoverOps::{show_hover_ui,show_single_diagnostic_ui}` | same-line official diagnostics are grouped; applicable fix emits one/fix-all actions; documentation emits `OpenLinterDoc` | each diagnostic action requires its own actual KatanA buffer/review/native-browser proof; group rendering alone proves nothing |
| `paste.rs:EditorPasteOps::intercept_clipboard_image_paste` | focused raw-image paste consumes the paste event before TextEdit mutation | opaque KUC clipboard request followed by a KatanA host acquisition effect; text must not mutate on image success |
| `paste.rs:EditorPasteOps::should_ingest_clipboard_image_paste` | focused image `file://` paste wins; ordinary text paste or already changed text does not take the image route | image URL, non-image URL, multi-line mixed URL, no focus, changed text, and no payload are separate host transaction cases |
| `paste.rs:paste_text_references_image_file` and decoder | accepts only nonempty lines of image `file://` URLs, supports `localhost` and percent decoding, then checks known image extensions | KatanA host-source branch only. KLE/KUC must not parse URL, extension, or percent encoding; native host E2E proves the observable result |
| `ui.rs:65`, `text_edit.rs:81-184`, `process_authoring.rs:59-99`, `image_ingest.rs:101-104` | editor rendering uses a frame-local `doc.buffer` clone. For supported image `file://`, the clone may change inside `TextEdit`, then `cursor_range_out` is restored to the pre-edit selection and no `UpdateBuffer` action is emitted. Later host image ingest inserts Markdown through that original selection into the unchanged active document. | KUC must defer image/file URL paste mutation until host resolution and retain the original selection transactionally. KLE must not send the URL as content or derive its own replacement range. Raw-image and image-URL selection-preservation are independent actual host-E2E leaves; no-payload/error must leave both document and selection unchanged. |

### Verified Search, Document, History, and Shortcut Seeds

| Source branch | Observed KatanA result | Required parity classification |
| --- | --- | --- |
| `app/doc_search.rs:DocSearchOps::compute_matches` | empty/invalid query has no matches; case-insensitive search evaluates Markdown `Text` and `Code` events only, excludes URL/HTML metadata, converts byte ranges to character ranges, then sorts/deduplicates | KatanA semantic search remains host-owned; query/visible filtering/multibyte range/empty result are individually asserted through KUC search UI and host E2E |
| `views/top_bar/search.rs:DocSearchBar::{render_content,render_nav_buttons,render_match_count}` | close clears `doc_search_open`; next/previous buttons are disabled with no matches; non-empty results show one-based active index and total, while a non-empty zero-result query has error styling | KUC owns the generic retained search-strip presentation and state records. Close, zero-result disabled button, empty-query no-result, non-empty-query no-result, one-based count, and current-match count are distinct RawInput/AccessKit/root-frame leaves; host-projected search presentation and one-shot KUC input traverse KLE without KLE state/request mapping. |
| `views/top_bar/search_logic.rs:SearchLogic::handle_input_events` | a newly opened bar requests focus once; Enter and ArrowDown request next, Shift+Enter and ArrowUp request previous, a changed query requests refresh, and Escape closes only with the source response's lost-focus condition | The generated branch catalog must retain each condition exactly, including focus gating for arrow keys and the source's Escape condition. KUC must not infer a broader keyboard policy from label text; every key/focus combination receives a distinct actual-input and KatanA-host transition leaf. |
| `app/action/dispatch_secondary.rs:82-104` plus `process_helpers.rs:22-74` | `OpenDocSearch`, `ToggleDocSearch`, `DocSearchNext`, and `DocSearchPrev` are actions, but `DocSearchQueryChanged` has no query payload and reads `state.search.doc_search.query` already changed by `DocSearchBar`; close similarly mutates `doc_search_open` inside the UI callback | KLE cannot satisfy query or close by direct `AppState` mutation or by an action-only bridge. The host-E2E must mechanically relay the single-consumption KUC query/close transport to physical input on the bootstrapped unchanged KatanA `DocSearchBar`, then observe the exact KatanA refresh/close effect. No public UI relay is a fail-closed release blocker; it is not permission for a KatanA edit in this scope. |
| `DocSearchOps::{navigate_next,navigate_prev}` | no matches returns no navigation; matching navigation wraps and resolves target line from character index | zero-result disabled command, next/previous wrap, active index, KUC scroll request, and actual host scroll state are separate leafs |
| `app/action/process_document.rs:handle_action_select_and_jump` | selects the document, closes active search modal, converts only `PreviewOnly` to `CodeOnly`, then sets target line | document identity, modal behavior, PreviewOnly-only conversion, target-line effect, and no unnecessary mode change are separate host-state leafs |
| `editor_undo.rs:EditorUndoIdentity` and `EditorUndoOps::record_external_change` | external replace/fix stores previous content and character cursor under workspace/document scoped identity | KUC owns generic history UI/input; KatanA host owns external-change record. Cross-workspace/document isolation and undo restore require real host E2E |
| `text_edit.rs:TextEditRenderer::render` plus locked `egui 0.36.1` `TextEdit` semantics | KatanA delegates ordinary copy/cut/paste, local undo/redo, selection editing and IME behavior to its locked `TextEdit` dependency after its image-paste interception branch | closure includes the exact external UI semantic dependency and KUC has one retained text/history implementation. Copy, cut, text paste, undo, redo, selection, composition, read-only and document switch each require a distinct root/host leaf; no KLE egui fallback is permitted |
| `app/document.rs:DocumentOps::handle_update_buffer` | no active document or unchanged revision returns without refresh; a changed non-HTML active buffer refreshes preview, an open search refreshes matches, and diagnostics debounce is updated | KUC user mutation must reach KatanA `UpdateBuffer` once. No-active/no-change, HTML/non-HTML, search-open/closed, dirty, preview, diagnostics and active-document isolation are separate actual-host effects |
| `app/document_edit.rs:DocumentEditOps::handle_replace_text` | replacement calls `String::replace_range`, then records external undo and refreshes preview/search/diagnostics when content changes | The host validates its current search result and converts it to byte boundaries before `ReplaceText`; stale/misaligned ranges fail typed without mutation. KLE carries query/replacement text only as a one-shot non-observed transport and never receives a current-result/range. Current/all, Unicode, no-op and read-only leaves cannot use a KLE-local replace engine |
| `state/shortcut_context.rs:ShortcutContextResolver::resolve` | active priority is Recording, Modal, Editor, Preview, then Global; `Explorer` exists as a command context but is not currently returned by the resolver | every current resolver outcome and `context_allows` branch must be cataloged; a future enum value cannot be ignored by a static manifest |
| `shell_ui/shell_ui_shortcuts.rs:KatanaApp::handle_shortcuts` | Recording suppresses commands, allowed contexts filter inventory, more modifiers are sorted first, editor-reserved text shortcuts are not consumed, and a matched command becomes the pending action | KatanA owns global arbitration. KUC contributes same-run focus/reservation evidence only; KLE/KUC cannot recreate inventory ordering or modifier parsing, issue a request to this router, or claim mounted causality. Every active-context/rejection/first-match case needs host E2E. |
| `shell_ui/shortcut_keys.rs:ShortcutKeyOps::editor_keeps_shortcut` | arrows, Backspace, Delete, Enter, Tab, and primary A/B/C/I/K/U/V/X/Y/Z remain editor input; primary Shift+V is intentionally not retained when Alt is absent | KUC must process only the post-arbitration retained input. Every protected key, shifted image-ingest exception, modifier variation and no-double-dispatch result is a separate host-E2E leaf; neither KLE nor KUC implements the KatanA router |

#### Fixed-Revision File/Edit/View Command Inventory

The following is the complete measured `file`, `edit`, and `view` inventory at
the locked reference revision: 6 file items, 16 edit items, and 16 view items.
The number **38** is an audit observation, not a release threshold. The
generator reads the item constructors and fails closed when an item, action,
availability closure, declared shortcut, context, or resulting action route
changes. Every row below expands into individual availability, physical
shortcut-origin, reserved-input, matched-action, and no-double-dispatch leaves;
it is not permissible to accept a group row or a shared keyboard trace.

| Source item | Action at this revision | Context and declared shortcut(s) | Required source-closure outcome |
| --- | --- | --- | --- |
| `file.open_workspace` | `PickOpenWorkspace` | Global; `primary+O` | source-spanned shell/workspace classification and native host boundary; it cannot be silently omitted. |
| `file.open_file_current_workspace` | `PickOpenFileInCurrentWorkspace` | Global; none | source-spanned shell/workspace classification and availability proof. |
| `file.close_workspace` | `CloseWorkspace` | Global; `primary+Shift+W` | source-spanned shell/workspace classification plus workspace-present/absent branches. |
| `file.save` | `SaveDocument` | Global; `primary+S` | editor-document effect leaf through actual shortcut arbitration and save effect. |
| `file.close_document` | `CloseActiveDocument` | Global; `primary+W` | document-tab close/pending-dirty/no-mutation or final close effect leaf. |
| `file.restore_closed` | `RestoreClosedDocument` | Global; `primary+Shift+T` | document-tab restore availability and actual document-state effect leaf. |
| `edit.bold` | `AuthorMarkdown(Bold)` | Editor; `primary+Shift+B` | individual opaque authoring target through KUC, KLE, and KatanA transform/cursor effect. |
| `edit.italic` | `AuthorMarkdown(Italic)` | Editor; `primary+I` | individual opaque authoring target through KUC, KLE, and KatanA transform/cursor effect. |
| `edit.strikethrough` | `AuthorMarkdown(Strikethrough)` | Editor; `primary+Shift+X` | individual opaque authoring target through KUC, KLE, and KatanA transform/cursor effect. |
| `edit.inline_code` | `AuthorMarkdown(InlineCode)` | Editor; `primary+backtick` | individual opaque authoring target through KUC, KLE, and KatanA transform/cursor effect. |
| `edit.heading1` | `AuthorMarkdown(Heading1)` | Editor; `primary+1` | individual opaque authoring target through KUC, KLE, and KatanA transform/cursor effect. |
| `edit.heading2` | `AuthorMarkdown(Heading2)` | Editor; `primary+2` | individual opaque authoring target through KUC, KLE, and KatanA transform/cursor effect. |
| `edit.heading3` | `AuthorMarkdown(Heading3)` | Editor; `primary+3` | individual opaque authoring target through KUC, KLE, and KatanA transform/cursor effect. |
| `edit.bullet_list` | `AuthorMarkdown(BulletList)` | Editor; `primary+Shift+8` | individual opaque authoring target through KUC, KLE, and KatanA transform/cursor effect. |
| `edit.numbered_list` | `AuthorMarkdown(NumberedList)` | Editor; `primary+Shift+7` | individual opaque authoring target through KUC, KLE, and KatanA transform/cursor effect. |
| `edit.blockquote` | `AuthorMarkdown(Blockquote)` | Editor; `primary+Shift+9` | individual opaque authoring target through KUC, KLE, and KatanA transform/cursor effect. |
| `edit.code_block` | `AuthorMarkdown(CodeBlock(Text))` | Editor; `primary+Shift+C` | fixed item plus separately generated every `CodeBlockKind::all()` menu leaf. |
| `edit.horizontal_rule` | `AuthorMarkdown(HorizontalRule)` | Editor; `primary+Shift+H` | individual opaque authoring target through KUC, KLE, and KatanA transform/cursor effect. |
| `edit.insert_link` | `AuthorMarkdown(InsertLink)` | Editor; `primary+K` | individual opaque authoring target through KUC, KLE, and KatanA transform/cursor effect. |
| `edit.insert_table` | `AuthorMarkdown(InsertTable)` | Editor; `primary+Alt+T` | individual opaque authoring target through KUC, KLE, and KatanA transform/cursor effect. |
| `edit.ingest_image_file` | `IngestImageFile` | Editor; none | native file-picker host-effect leaf; KLE/KUC never receive path or image bytes. |
| `edit.ingest_clipboard_image` | `IngestClipboardImage` | Editor; `primary+Shift+V` | native clipboard-image leaf and reservation exception; no text double-dispatch. |
| `view.katana_command_palette` | `ToggleKatanaCommandPalette` | Global; `primary+Shift+P` | source-spanned shell/editor origin classification; a KLE command palette is not implied. |
| `view.command_palette` | `ToggleCommandPalette` | Global; `primary+P` | source-spanned shell/editor origin classification; a KLE command palette is not implied. |
| `view.explorer` | `ToggleExplorer` | Global; `primary+B` | source-spanned shell classification and availability proof. |
| `view.refresh_explorer` | `RefreshExplorer` | Global; `primary+Shift+R` | source-spanned shell classification plus workspace-present/absent branches. |
| `view.toggle_filter` | `ToggleExplorerFilter` | Global; none | source-spanned shell classification plus workspace-present/absent branches. |
| `view.close_all` | `CloseAllDocuments` | Global; none | document-tab host effect; individual active/empty/dirty branches. |
| `view.search_modal` | `ToggleSearchModal` | Global; `primary+Shift+F` | source-spanned global-search classification and source effect; no unexamined host-only exclusion. |
| `view.doc_search` | `ToggleDocSearch` | Global; `primary+F` | KUC SearchStrip opening plus actual KatanA document-search relay/effect leaves. |
| `view.refresh_document` | `RefreshDocument { is_manual: true }` | Global; `primary+R` | actual active-document refresh effect and unavailable branch. |
| `view.zoom_in` | `ZoomIn` | Global; `primary+Shift+Plus`, `primary+Plus` | source-spanned editor-frame versus shell classification, including each physical shortcut. |
| `view.zoom_out` | `ZoomOut` | Global; `primary+Shift+Minus`, `primary+Minus` | source-spanned editor-frame versus shell classification, including each physical shortcut. |
| `view.toggle_split_mode` | `ToggleSplitMode` | Global; `primary+Shift+backslash` | actual view-mode/split state effect from the physical router. |
| `view.toggle_code_preview` | `ToggleCodePreview` | Global; `primary+backslash` | actual view-mode cycle effect from the physical router. |
| `view.tab_next` | `SelectNextTab` | Global; `primary+Right`, `primary+Shift+]` | individual next traversal, wrap/boundary, and document-isolation host effect. |
| `view.tab_prev` | `SelectPrevTab` | Global; `primary+Left`, `primary+Shift+[` | individual previous traversal, wrap/boundary, and document-isolation host effect. |
| `view.toggle_slideshow` | `ToggleSlideshow` | Global; `primary+Alt+Enter`, `F5` | actual fullscreen/windowed close/restore path and no duplicate native-window request. |

`is_active_editable_markdown` is an availability closure shared by the sixteen
`edit.*` entries. The generator records it as a shared source edge but still
creates a separate available/unavailable proof for every item; reference,
virtual, missing, and editable Markdown documents cannot collapse into one
authoring result. Likewise, the Global context in this table does not make an
item host-only: `shell_ui_shortcuts` first resolves the active context and
arbitrates reserved text input. Final `editor_shortcut` versus `host_only`
classification comes from the physical origin and effect route, not from the
command group or context enum alone.

#### Document Search Host-Input Relay Gate

The fixed KatanA revision exposes no query-bearing document-search action:
`DocSearchQueryChanged` derives its result from UI-owned state, while the close
button and Escape branch mutate that state in `DocSearchBar`/`SearchLogic`.
Therefore the release harness needs a linear **host-input relay** only for this
source boundary:

```text
KUC SearchStrip RawInput
  -> one non-Clone/non-Serialize query or close transport
  -> KLE immediate forwarding without inspection or storage
  -> actual input on the bootstrapped unchanged KatanA DocSearchBar
  -> KatanA UI mutation -> existing action/handler/frame effect
```

The relay is an E2E host adapter, never a KLE search implementation. It may
record only event kind, revision, correlation, byte length, and SHA-256. It
must use an actual KatanA UI focus/input route and dynamic current-frame target,
not `app_state_mut`, a direct query assignment, fixed coordinates, an action
fixture, or a mock search state. Query open, query change, close button,
Escape lost-focus condition, next/previous pointer and keyboard branches, zero
result, wrap and editor/preview scroll outcomes each retain a separate leaf.
If the unchanged public KatanA surface cannot accept this relay, the leaf is
unexecuted and the release gate fails; no KatanA source edit is authorized by
this finding.

### Integration-Oracle Audit Holds

The current KatanA integration tests identify required externally visible
outcomes, but they do not meet the KLE v0.1.0 evidence standard by themselves.
The source-closure generator must record them as test-oracle edges and require
a separate KLE RawInput/KUC-record/actual-KatanA-host execution for each
corresponding leaf.

| KatanA oracle | Observed requirement | KLE acceptance constraint |
| --- | --- | --- |
| `tests/integration/editor/ui.rs:119-161` | a focused code-editor command popup exposes one Code Block button; choosing `text` changes the active buffer to include a `text` fence | KUC popup/AX record and KLE actual physical pointer/keyboard path must precede KatanA `AuthorMarkdown` handling. A direct `AppAction` injection or a text assertion alone is rejected. |
| `ui.rs:164-190` | a focused paste of percent-encoded image `file://` text queues clipboard image ingestion | On 2026-08-14, the registered `ui_integration_parallel` oracle passed, but it stops at pending action and emitted macOS automation `-1700` diagnostics. It is source evidence only. KLE must additionally use an actual host paste acquisition route and prove original-selection preservation, URL no-text-mutation, and final successful/cancelled/error asset/Markdown/dirty/explorer result. KUC/KLE must not duplicate KatanA URL decoding. |
| `ui.rs:192-238` | an open code-kind menu closes after an editor click outside it | KUC must prove real retained popup focus/occlusion, its exact current-frame record and AccessKit state. Fixed sleeps, simulator state, or a shared selector are not acceptable. |
| `ui.rs:240-283` | the current OS clipboard test is explicitly `#[ignore]` and relies on manually copied content | it is a non-evidence oracle only. KLE release validation requires unattended native fixture provisioning/trace and a real final KatanA asset/Markdown observation; manual setup cannot close the leaf. |
| `rendering.rs` and `ui.rs:285-351` | line numbers, buffer update and save are checked mainly through labels or directly triggered actions | KLE tests must use unique source-derived locators, KUC gutter/text records and final KatanA active-document/filesystem state. Label-only checks cannot establish geometry, edit path or persistence. |
| `lint_fix_review_button.rs` | gutter hover Fix opens a lint-review tab; reject-all restores the original document; all-tabs scope includes loaded and unloaded tabs; status count changes only while the problems panel is open | all diagnostic command leaves need physical input and the final KatanA review/tab/scope state. KUC must own only generic gutter/popup presentation; no direct test-state mutation or mock linter result is allowed in end-to-end coverage. |
| `layout_persistence.rs`, `toggle_view_modes.rs`, `view_modes.rs` | PreviewOnly/Split content is visible; directions persist; toggle sequence is `Split -> PreviewOnly -> CodeOnly -> PreviewOnly`; ToggleSplit always chooses Split | KLE must exercise public KUC command activation and assert actual KatanA per-document view/split state, rendered KUC root records and preview/editor scroll interaction. Each state transition is a separate leaf. |
| `navigation.rs` | workspace selection, tab open/close, active-tab switching and no-active-tab result are visible state | document identity/history/scroll/popup state must be isolated at the real host boundary. Test setup may create fixtures, but action direct injection is not KLE input evidence. |

KatanA harness `run_steps`, bounded sleeps and direct `trigger_action` calls are
reference-oracle mechanics only.  They are not permitted in KLE/KUC E2E as a
replacement for an observable readiness condition, actual public input or a
final host effect.  An ignored test remains a failing KLE leaf until the
strict unattended native scenario above exists.

## Required Leaf Families

| Family | Decisive source branches | Required leaf separation |
| --- | --- | --- |
| text/IME/selection | `text_edit.rs`, `types.rs`, `ui.rs` | editable/read-only, Japanese IME commit, `⭐️` VS16, ZWJ, selection/caret, authoring cursor restore |
| authoring toolbar | `toolbar.rs`, `toolbar_popup.rs`, `authoring.rs`, `markdown_authoring_op.rs` | 12 triggers, selection/caret anchor, focus retention, suppression, outside close, four-edge clamp |
| context menu | `context_menu.rs`, `context_menu_image_ingest.rs`, `code_block_menu.rs` | Save, conditional Format, 14 authoring leaves, 17 code kinds, file/clipboard image leaves, secondary/keyboard/AccessKit paths and disabled branches |
| clipboard/image ingest | `paste.rs`, `clipboard_*.rs`, `image_ingest.rs` | image-over-text priority, raw image, file list, image/non-image file URL, percent decode, macOS fallback, saved/unsaved/error/no-mutation effects |
| diagnostics/gutter | `decorations.rs`, `line_numbers.rs`, `row_diagnostics.rs`, `diagnostics_*.rs` | numbered/active/hovered rows, official start-line marker, same-line aggregation, popup lifecycle, fix/fix-all/docs review effect |
| search/replace | `doc_search.rs`, `text_edit.rs`, `views/top_bar/search.rs`, `views/top_bar/search_logic.rs`, `app/action/process_helpers.rs`, `app/action/dispatch_secondary.rs` | visible-text/code search, URL/HTML exclusion, query, next/previous wrap, zero result, focus/Escape; replace-current/replace-all remain user-mandated leaves |
| document/history | `document_edit.rs`, `refresh_content.rs`, `process_markdown_formatting.rs`, `editor_undo.rs` | dirty/save, preview/search/diagnostics refresh, active/non-active behavior, format, scoped undo, external change |
| navigation/view | `logic.rs`, `logic_scroll.rs`, `dispatch_secondary.rs`, `central_content.rs`, `split.rs`, `app_state_impl.rs`, `preview/content.rs`, `process_document.rs` | CodeOnly/PreviewOnly/Split, horizontal/vertical, persisted layout, scroll sync, select-and-jump, document switch |
| preview/KDV/KRR viewport | `central_content.rs`, `layout/{split,split_horizontal,split_vertical}.rs`, `panels/preview/content.rs`, `preview_pane/**` | Markdown sections, KDV document frame, KDV/KRR HTML browser, split resize/ratchet, pane order, scroll/hover/source target/task, fullscreen/slideshow, failures and external boundaries; every branch is distinct |
| shortcut arbitration | `edit_commands.rs`, `shortcut_context.rs`, `shell_ui_shortcuts.rs` | Recording/Modal/Editor/Preview/Global priority, availability, text-entry suppression, modifier specificity, first match |
| syntax/accessibility | `syntax.rs`, `layout.rs`, `logic_colors.rs`, `text_edit.rs` | injected span/decoration state, KUC TextSurface/AccessKit mapping, no local rendering fallback |

## Non-Evidence And Rejections

- `crates/katana-ui/tests/integration/editor/kle_downstream_adapter.rs` does
  not exist at the recorded revision.  It must not be deferred, listed, or
  used as a planned parity proof.
- A source marker, KatanA baseline test, KLE simulator, Storybook screen, or
  shared selector does not close a leaf.
- The retired `57-file` cardinality is not an acceptance threshold.  A static
  list that omits a reachable action dispatcher, handler, state mutation, or
  input path must fail even when its recorded count and hashes match.
- Every behavior leaf needs its own exact source branch, KUC current-frame
  record, AccessKit evidence, and an executed class-appropriate declared
  effect. `editor_direct_ui`, `editor_host_continuation`, and user-mandated
  extension leaves additionally require a KLE public
  `RawInput -> EguiLanguageEditor::show` locator. KatanA-owned
  `editor_shortcut` leaves instead require the fixed-revision physical KatanA
  RawInput/router/action trace correlated with same-run KUC root/AccessKit
  focus and retained-input evidence for the same document/revision. A shortcut
  record must not claim that KLE is mounted in or causes the KatanA router.
  Generic retained-UI leaves additionally prove an unchanged bootstrapped
  KatanA host observation; host-effect leaves prove the actual KatanA effect.
- KatanA must not be edited to make this inventory or its evidence pass.

## Verified Preview, Split, And KDV Boundary Seeds

| Source branch | Observed KatanA result | Required parity classification |
| --- | --- | --- |
| `views/app_frame/central_content.rs` | virtual documents select independent content; normal documents select `CodeOnly`, `PreviewOnly`, or `SplitMode`; preview side panels precede central content | exact virtual/no-active/code/preview/split decisions are separate source branches. KUC owns generic workspace composition; KLE forwards host-projected opaque descriptors only. |
| `views/layout/split.rs` | `SplitDirection::Horizontal` and `Vertical` select different split paths; `PreviewOnly` consumes an editor scroll request after rendering preview | axis, PreviewOnly/no-document and post-preview scroll-clear branches are separate leaves; KLE has no panel/scroll implementation. |
| `views/layout/split_horizontal.rs` | pane order chooses left/right preview; a resizable panel has min/max/default size; non-drag intrinsic growth is ratcheted back; editor receives the complementary panel | KUC `SplitViewport` owns geometry, resize hit test, clamp and ratchet prevention. pane order and host-owned view selection are individual requests; retained resize/persistence is a KUC effect with unchanged host observation. |
| `views/layout/split_vertical.rs` | pane order chooses top/bottom preview; height resize has default/max and ratchet protection | horizontal and vertical branches are independent leaves; no shared aggregate `split resize` selector. |
| `views/panels/preview/content.rs` | document surface and HTML browser bypass Markdown sections; otherwise preview has forced/requested scroll, synchronized scroll, hover source line, task action, floating back-to-top and linter-doc UI branches | source closure must expand each early return, scroll source, hover/click guard, task event and visible control. KUC renders a typed generic frame; KLE does not copy preview rendering. |
| `views/panels/preview/{side_panels,side_panel_hover,side_panel_toc,side_panel_toc_ops,side_panel_export,side_panel_story,side_panel_tools_inner}.rs` | fixed 32px rail conditionally exposes TOC, then refresh/search/export/story/tools/meta controls; export/story/tools have foreground hover panels and delayed sibling switching; TOC has pinned left/right and hover/cooldown/empty-source paths, and row selection mutates current scroll state directly | all rail, pinned/overlay panel, hover/cooldown, capability, empty-source and action branches are editor-source leaves. KUC owns generic rail/panel mechanics, KLE forwards opaque targets only, and host-target effects use unchanged KatanA routes. The generated closure must reject a KLE local panel, `TocPosition`, line/scroll calculation or action-only fixture. |
| `views/panels/toc/{mod,render,anchor_ops,anchor_lookup_ops}.rs` | TOC header expands/collapses hierarchy; leaf and parent header clicks have different disclosure/selection paths; active heading derives from CodeOnly editor anchor, PreviewOnly hover/viewport anchor, or Split source anchor; active nodes may scroll into view | generic KUC `OutlineNavigator` owns hierarchy/accordion/active presentation/scroll geometry. KatanA retains outline/anchor/source semantics and actual selected-item scroll behavior. KLE must not model outline index/level/line/anchor/force-open/current origin or copy the egui TOC. |
| `state/toc.rs` | TOC click immediately changes the current anchor and suppresses one editor/preview observation; viewport candidates require exact source-dependent eligibility and 25ms stability, then generation gates auto-scroll; document change resets all fields | this is host semantic state, not a KUC or KLE debounce implementation. The closure generates each candidate/suppression/stability/generation/reset branch. Host E2E drives current KatanA UI input frames with deterministic monotonic timestamps, not sleep/retry or direct `TocState` setup. |
| `preview_pane/ui.rs` and `section_show/**` | Markdown preview sections create anchors, source spans, hover state, search/highlight and task activation | KatanA CommonMark/egui renderer is a reference oracle. KatanA resolves all Markdown/KDV semantics and projects a generic opaque preview descriptor; KLE only transparently transits it and never depends on, maps, or projects KDV public types. |
| `preview_pane/document_surface/{pane,worker,render,painter/**}.rs` | KatanA opens KDV `DocumentSession`, applies typed commands, receives `DocumentFrame`, paints it, and distinguishes source/KDV worker/KDV surface/KatanA host failures | KDV remains renderer/domain owner. KUC owns a generic frame viewport; KLE forwards a host-projected neutral frame descriptor without geometry/raster code or a KDV dependency. Every command/frame/failure variant must be generated. |
| `preview_pane/image_html_surface*.rs` | KatanA translates KDV browser session source/viewport/input/update/navigation to egui and shows KDV/KRR-produced pixels | KUC owns a generic external-browser viewport; KLE forwards a host-projected opaque frame/input/navigation descriptor. Browser raster and native boundaries are not reimplemented. |
| `preview_pane/{fullscreen,slideshow/**}.rs` | fullscreen/slideshow controls and Escape/close may request OS fullscreen restoration | pure generic overlay/focus transitions are KUC retained effects; OS viewport effects are native external leaves. |

### Verified Preview Action And State Seeds

| Source branch | Exact observed branch | Required source-derived leaves and declared effect |
| --- | --- | --- |
| `state/command_inventory/view_commands.rs` -> `app/action/dispatch_secondary.rs::ToggleSplitMode` | `primary+Shift+\\` sets the active document's mode to `Split` | `view.command-toggle-split`, `in_process_host_effect`; editor-context reservation remains a separate shortcut leaf. |
| `state/command_inventory/view_commands.rs` -> `app/action/dispatch_secondary.rs::ToggleCodePreview` | `primary+\\` has three branches: `Split -> PreviewOnly`, `PreviewOnly -> CodeOnly`, `CodeOnly -> PreviewOnly` | three separately executed `view.command-toggle-code-preview.*` leaves, each an `in_process_host_effect`; a cycle aggregate is prohibited. |
| `views/panels/preview/side_panel_tools_inner.rs` | visible split toggle emits `SetViewMode(Split)` or `SetViewMode(PreviewOnly)`; Tangochou emits `SetViewMode(CodeOnly|PreviewOnly)` | `view.set-mode.*` leaves are distinct physical pointer/keyboard/AccessKit routes. KUC owns control geometry and focus; KLE forwards the current opaque host target once and the host resolves it. |
| `views/panels/preview/side_panel_tools_inner.rs` -> `dispatch_secondary.rs` | horizontal/vertical direction icon chooses the opposite direction; pane-order icon chooses the opposite order; sync switch writes `Some(true|false)` | each direction, pane-order and sync direction is a separate `in_process_host_effect`. KUC may retain viewport geometry but never derives source offsets or persists KatanA's per-document layout. |
| `views/panels/preview/content.rs` -> `app/action/{dispatch,mod}.rs::handle_toggle_task_list` | first actionable Markdown/HTML task emits `(global_index, new_state)` and the handler reparses the KatanA buffer before one transform/dirty/preview refresh | valid and stale-or-missing task leaves are separate. KLE/KUC carry only an opaque host semantic task request and may not copy CommonMark parsing, calculate `global_index`, or mutate text. |
| `views/panels/preview/logic.rs` -> `content.rs` | back-to-top writes a one-shot preview scroll request and consumes it after forcing zero offset | `preview.back-to-top` is a `kuc_retained_ui_effect` in the replacement: KUC root/AX transition plus bootstrapped KatanA no-mutation observation, never a direct KatanA state write. |
| `state/command_inventory/view_commands.rs` -> `app/action/process_helpers.rs::handle_action_toggle_slideshow` | open resets page, records prior OS fullscreen state, requests fullscreen only when needed; close conditionally restores windowed mode | `preview.slideshow.open.windowed`, `.open-already-fullscreen`, `.close-restore-windowed`, `.close-preserve-fullscreen` are individual `native_external_host_effect` leaves. Escape and close-control routes require their own physical-input evidence. |
| `views/panels/preview/side_panel_story.rs` -> `app/action/dispatch.rs` | hover-highlight and diagram-control settings toggle independently | both settings, and each direction, remain individually catalogued `in_process_host_effect` leaves. A generic KUC setting control must not silently substitute host state without actual host proof. |
| `views/panels/preview/side_panels.rs` -> `app/action/{dispatch,dispatch_secondary,dispatch_tertiary}.rs` | TOC is capability-gated; refresh is manual; search toggles the document-search bar; export/story/tools toggle hover panels; Info passes the active document path to metadata | generate individual capability/open/close/hover/target leaves. Panel mechanics are generic KUC retained effects; refresh/search/meta effects follow actual host routes. KLE never owns a panel name, path, hover time, geometry or direct host state. |
| `views/panels/preview/side_panel_export.rs` -> `app/action/dispatch_tertiary.rs` -> `app/export.rs` | the four physical items issue `ExportDocument(Html|Pdf|Png|Jpg)` | four independent export leaves. The host owns format, output path, renderer/tool and error outcome; KLE receives no format/path/payload. |
| `views/panels/preview/side_panel_toc_ops.rs` | a selected TOC row clears current source scroll fields and writes `toc_scroll_to_line` only outside `PreviewOnly` | every current TOC selection/view-mode guard is an actual-KatanA UI input relay leaf. KUC carries one opaque source target; KLE has no row/line/scroll mutation. |

### Current Adoption Boundary: Verified Integration-Claim Blocker

At this revision, `Cargo.toml` has no `katana-language-editor` dependency and
the three KatanA rendering routes directly instantiate the legacy editor:
`views/app_frame/central_content.rs`, `views/layout/split_horizontal.rs`, and
`views/layout/split_vertical.rs` each call `EditorContent::new`.  KatanA is
read-only in this scope, so no current actual KatanA frame can contain KLE.
This is a fact, not an inferred test gap. Therefore an execution record must
reject any claim that a KLE input was rendered inside an actual KatanA editor
frame until a separately approved KatanA adoption change exists. KLE v0.1.0
does not require a KatanA source change: its parity evidence instead joins the
independent KLE/KUC real-root interaction with a read-only KatanA
host-effect bridge that executes the corresponding unchanged action and
observes its document/state/file/native result. The two proofs must not be
relabelled as an already-adopted KatanA rendering integration.

The resolved KDV `PreviewRenderEngine` emits `PreviewSurfaceImage` through
`KdvPreviewSurfaceFactory` and export `SurfacePainter`. That image is not an
admissible KLE Markdown editor-preview renderer: it bypasses KUC platform text
raster, exact `⭐️` VS16 glyph proof, grapheme measure and hit testing. The
closure must prove a KDV semantic `ViewerInput`/`ViewerNodePlan` projection to
KUC structured preview input that discards KDV geometry and lets KUC lay out
all text. Opaque raster projection is limited to external document/PDF/media/
diagram/browser payloads with an accessibility alternative and exact runtime
fingerprint. The source branch remains open until this proof exists.

The KDV source is a read-only dependency audit input, not a KUC dependency.
For this KatanA revision the canonical input is the `Cargo.lock` resolved
registry package `katana-document-viewer 0.5.2`, checksum
`d5e6e3cb1791b7a6ceb836f5bd932fcce5600b7e5c163b9637e503260b455eb5`, with
`katana-render-runtime 0.4.15`; the local KDV `0.2.8` checkout is not API
evidence for this host. KDV already depends on KUC, so KUC-to-KDV coupling is a
release-blocking cycle. The generated closure must record the resolved package
checksum/version/API symbol and file hash used by every document/browser/
preview branch and fail if a KatanA visible branch has no public KDV-to-KUC
projection. See
`docs/v0-1-0-kuc-preview-viewport-design.md`.

## Automated Gate Design

`katana-parity-check` must derive and compare the six joined artifacts in
`docs/v0-1-0-parity-manifest-schema.md`: source closure, action origins, branch
catalog, leaf manifest, execution record, and Storybook artifact manifest. The check fails
closed for a missing/extra/changed file, a closure edge without a recorded
target or host-boundary rationale, a branch without a leaf or recorded helper
rationale, duplicate or aggregate leaf IDs, a nonexistent locator, a shared
actual-input selector, an unexecuted host-E2E, or an artifact without the
corresponding KUC record and AccessKit node.
