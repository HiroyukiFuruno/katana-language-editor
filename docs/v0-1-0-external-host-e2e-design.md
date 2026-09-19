# KLE v0.1.0 External Host E2E Design

## Decision

This document defines the only acceptable proof for source-derived leaves that
cross the KatanA host boundary. It is a release design, not evidence that the
design has been implemented.

KatanA at `4f6a6287c650a38633c7baeb544a92e739c68567` remains read-only. KLE
and KUC must not copy filesystem, native-dialog, clipboard, image decoding, or
asset insertion logic to make a test easier. A KLE host E2E may drive only the
unchanged KatanA public application loop and its source-derived physical UI
input routes. It must not add a KatanA adapter, test hook, feature flag, patch,
source file, direct `AppAction` injection, or `pending_action` assignment.

## Measured Current Gap

The existing `tools/katana-host-e2e` is partial and is not release evidence for
external interaction leaves:

| Fact | Measured source | Consequence |
| --- | --- | --- |
| KLE RawInput paste reaches public `EguiLanguageEditor::show` and emits a deferred clipboard request without changing KLE content | `tools/katana-host-e2e/src/kle_input.rs:46-105` | This is valid KLE-side input evidence only. |
| The host runner translates a request, assigns `KatanaApp::pending_action`, then runs one real KatanA UI frame | `tools/katana-host-e2e/src/host.rs:179-189`; KatanA `crates/katana-ui/src/shell/mod.rs:129-132` | This is direct action injection and is rejected. Replace it with a correlation-only bridge plus the source-derived physical KatanA UI-input relay that emits the action naturally. |
| The RawInput runner executes a real KatanA frame but discards its `FullOutput` after clearing texture deltas | `tools/katana-host-e2e/src/host.rs:77-86` | It cannot discover and record a same-frame dynamic AccessKit target, frame hash, or bounds before input. Retain only the required snapshot hashes and fail the leaf when semantic target discovery is unavailable. |
| The file-URL paste tests create a markdown document fixture and accept `pending_action_for_test()` as their primary result | `tools/katana-host-e2e/tests/file_url_paste_actual_host.rs:41-75,84-125` | These are migration residues, not release evidence. Bootstrap through physical workspace/document UI routes and assert the final source-derived document/asset effect; pending action is diagnostic-only and cannot satisfy a leaf. |
| The current action bridge has both `ExternalHostIntentUnavailable` and `UnsupportedAction` outcomes | `tools/katana-host-e2e/src/action_bridge/action_bridge_error.rs:8-40`; `action_bridge_translation.rs:57-80` | A rejected action is a release failure, not a legitimate expected test outcome. |
| Headless file ingest records `IngestImageFile` and opens KatanA's fallback dialog; the selected file is consumed later by KatanA UI update code | KatanA `app/action/image_ingest.rs:5-9`; `shell_ui/shell_ui_update.rs:88-115` | Request-to-picker is observable, but current KLE test code does not select a file or assert final asset/link/dirty effects in the same run. |
| KatanA clipboard-image ingest reads the actual host clipboard and then calls its private image processing path | KatanA `app/action/image_ingest.rs:37-105`; `app/action/clipboard_image.rs` | KLE/KUC cannot replace this with a fixture payload. The current KatanA live clipboard test is ignored, so it is not release evidence. |
| Public KatanA test hooks expose app state but no file-selection or clipboard-payload injection | KatanA `shell/test_hooks.rs:8-172` | A pure in-process fixture cannot complete native acquisition without modifying KatanA, which is prohibited. |
| Current KLE host fixture constructs `AppState`, pushes a `Document` into `open_documents`, sets `active_doc_idx`, and sets view state through `app_state_mut()` | `tools/katana-host-e2e/src/host.rs:37-61` | It can exercise a later handler, but it does not prove the KatanA workspace/document/view transition that produced that state. It is rejected as release host-E2E setup. |

## Three Required Effect Classes

Every source-derived leaf declares exactly one class. Each class requires the
same source revision/hash, a physical KLE RawInput locator, a current KUC root
record plus AccessKit assertion, and an exact declared effect. Host-request
classes additionally require one typed KLE request. The class determines how
the effect boundary is exercised.

| Class | Examples | Required execution |
| --- | --- | --- |
| `kuc_retained_ui_effect` | Problems open/close, stable scope selection, disclosure and expand/collapse | Public KLE RawInput changes only current retained KUC root state. The execution records root/AX transition and a read-only, already-bootstrapped KatanA host observation proving no document, diagnostic-data or workspace mutation. It must not synthesize a KatanA action or mutate `AppState`. |
| `in_process_host_effect` | authoring transform, save, format, diagnostics action, document search, view/split, history, tab/document state | Public KLE RawInput produces an opaque one-time target/transport. The bridge records only its correlation and exact source UI locator. A host input driver then exercises the equivalent source-derived pointer/key/IME/AccessKit route on unchanged KatanA; KatanA itself emits its action, handler, and real UI frames until the named document/state/file effect is asserted. Direct `AppAction`/`pending_action` mutation is invalid. `editor_shortcut` uses the separate physical-router mode below. General replace/replace-all is not listed here because the fixed KatanA source has no general document replace UI/action route. |
| `native_external_host_effect` | image-file picker, clipboard raw image, clipboard file-list image, percent-encoded `file://` image | Public KLE RawInput produces the opaque one-time target/transport; the bridge records correlation only. The host input driver invokes unchanged KatanA's source-derived UI route and a deterministic native driver supplies the real external input. The same execution asserts KatanA's file, Markdown, dirty, explorer-refresh, and no-mutation effects. |

No class permits a source test, a state hook, a bridge translation unit
test, a simulated downstream adapter, or a Storybook frame to stand in for the
declared effect.

### KatanA-Owned Shortcut Router Evidence

The effect class does not transfer ownership of `shell_ui_shortcuts` to KLE.
For an `editor_shortcut` origin, unchanged KatanA receives the physical key
`RawInput` and proves its own active context, command availability,
modifier-specific ordering, reserved-text decision, exactly-one pending action,
handler, and effect. The same execution run also records the current KUC root
frame and AccessKit focus/input state for the same bootstrapped document and
revision. These two observations are joined by immutable execution correlation,
not by replaying a KLE event into KatanA or by asserting that KLE is mounted in
KatanA.

KatanA currently constructs legacy `EditorContent` directly. Therefore a
shortcut record is invalid if it states or implies `KLE RawInput ->
shell_ui_shortcuts` causality. Conversely, a KatanA-only shortcut test without
the correlated KUC focus/reservation record does not prove component parity.
The generated manifest must label this mode `katana_shortcut_router`; it has no
typed KLE request. A separately approved KatanA adoption change would be
required before the stronger mounted-frame claim becomes valid.

### Document Search Input Relay

The KatanA document-search source is intentionally not action-complete:
`DocSearchQueryChanged` consumes `SearchState` after `DocSearchBar` has mutated
it, and close uses a UI callback rather than a close action. A release harness
must not repair this by assigning `SearchState` or `AppState`. For every query
and close leaf it consumes KUC's one-shot transport exactly once and drives the
equivalent text/IME or close/Escape interaction on the actual, bootstrapped
KatanA search UI. The same trace records the KUC frame/AX event, KLE forwarding
fingerprint, KatanA current-frame target and real match/close/scroll outcome;
raw query bytes remain absent from artifacts.

This relay is limited to the host-E2E boundary and owns no query parsing,
matching, range conversion, retry, or state cache. It must use current semantic
or dynamic-frame targeting, never a fixed coordinate. If the unchanged KatanA
surface cannot be driven this way, the relevant query/close source leaves fail
closed. This finding does not authorize a KatanA repository change.

### Preview Rail And Outline Relay

`PreviewSidePanels` and the TOC body are direct editor behavior in the fixed
KatanA source, but their responsibilities divide at the KUC root. Rail button
hit testing, hover delay, sibling exclusion, pinned/overlay placement,
outside/Escape dismissal, outline disclosure, expand/collapse-all, active
presentation, generic scroll-into-view, and empty presentation are
`kuc_retained_ui_effect` leaves. Their proof ends with a KUC root/AccessKit
transition and a read-only observation that the already-bootstrapped KatanA
document, diagnostics, workspace, and TOC semantic state did not mutate.

Refresh, document-search toggle, export, story, tools, metadata, slideshow,
view-setting, and an outline-row selection instead start with one KUC RawInput
event carrying an opaque target/revision/correlation. KLE forwards it once.
The host resolves the target against its current KatanA projection and proves
the unchanged action/handler/final effect. KLE must not branch on target name
or synthesize `ToggleToc`, `ToggleExportPanel`, `ExportDocument`, a line,
anchor, scroll request, or an `AppState` change.

TOC selection and active-anchor leaves require an additional host-input relay.
The harness drives the actual bootstrapped KatanA TOC control and records the
same monotonic input-frame timestamps used by `state/toc.rs`; it must exercise
first/repeated/replaced candidates, below/at stability threshold, click
suppression, editor versus preview observations, document reset, and
PreviewHover fallback. Fixed sleeps, retry loops, direct `TocState` mutation,
preloaded active anchors, and action-only fixtures are prohibited. An
unreachable unchanged KatanA TOC UI route is a release failure for that leaf,
not a reason to add a KLE implementation or modify KatanA.

## Host Bootstrap Contract

The host fixture itself is part of the execution chain. The existing helper
that pushes a `Document` into `AppState::document.open_documents`, assigns
`active_doc_idx`, or sets a target view through `app_state_mut()` cannot seed a
release case. It bypasses the same KatanA document lifecycle that the leaf is
supposed to prove.

Each host-E2E case creates an isolated physical workspace and document files,
constructs only the ordinary KatanA application service configuration, then
drives unchanged public KatanA actions and UI frames to establish the case:

1. `KatanaApp::new` starts without a directly injected document.
2. The runner uses the actual source-derived KatanA workspace-open UI route for
   the physical workspace and waits for KatanA's event/state completion through
   observable state changes, not a fixed sleep, direct `OpenWorkspace` action,
   or manually populated document list.
3. It uses the actual source-derived KatanA document-selection and view/split/
   search UI routes for every prerequisite state. A required CodeOnly,
   PreviewOnly, Split, active-tab, or search condition must therefore have its
   own recorded physical KatanA UI transition, never an injected action.
4. Only after that path has produced the active document may the exact public
   KLE RawInput trace begin. The event bridge and action bridge record all
   resulting KatanA frames/effects.

Existing read-only hooks such as `app_state_for_test()` may observe an already
executed KatanA effect when no public query exists. They may not construct,
mutate, select, save, format, search, scroll, inject clipboard content, or
stand alone as the effect oracle. `app_state_mut()`, direct `Document` creation
inside host E2E, direct `open_documents`/`active_doc_idx` mutation, and direct
view/search/diagnostic state mutation are forbidden in release host-E2E source
and must be rejected by an AST rule. The isolated native-external class also
retains its stricter non-headless requirement.

### Bootstrap Execution Slices

The bootstrap implementation proceeds in observable slices. A later slice may
not reuse a state fixture from an earlier test.

1. Fixed-source selection resolves the generated manifest, lockfile, and
   canonical metadata fingerprint before host code is compiled.
2. The fixed harness constructs ordinary KatanA services and calls
   `KatanaApp::new` with no document, workspace, view, search, diagnostics, or
   action state injected. It enables AccessKit and records the first real host
   frame. This proves only startup construction and current-frame observability.
3. A dynamic AccessKit target driver performs the actual onboarding and
   workspace-open UI inputs, recording each target from the immediately prior
   frame. Native-dialog selection and document activation are separate slices.
4. Only after slices 1-3 produce an active document can a KUC RawInput and
   KLE opaque transit begin. A first-frame success, a state hook, or a generated
   manifest cannot stand in for workspace/document bootstrap.

### Executable Physical-Bootstrap Plan

The in-process `FixedHostSession` is intentionally limited to slices 1-2. It
does not own a native window, operating-system event queue, or a native dialog,
so it is not eligible for slice 3 or any release leaf which claims a physical
host interaction. The next implementation is a separate process runner in
`tools/katana-host-e2e`, with these fixed boundaries:

1. It starts the unchanged `KatanA` binary from the fixed checkout using
   `cargo run --manifest-path <fixed-source>/Cargo.toml -p katana-ui --bin
   KatanA`. The runner sets a unique `CARGO_TARGET_DIR` and
   `KATANA_CONFIG_DIR` under its execution sandbox. It neither changes the
   fixed checkout nor reads or writes the user's normal KatanA configuration
   directory.
2. It identifies the launched application and its window through the operating
   system accessibility API, not by process title alone, fixed screen
   coordinates, or a timing delay. A missing, duplicate, disabled, unnamed, or
   non-owned window/control is a typed failure. The source manifest supplies
   the expected semantic role and the sorted, duplicate-free set of
   accessible-name digests generated from every bundled locale's
   `menu.open_workspace` value; raw control labels and locale values are not
   emitted into the result artifact.
3. The runner emits a real pointer/key/IME event only after resolving that
   control in the immediately preceding accessibility snapshot. It waits for a
   semantic accessibility notification or an independently observed filesystem
   effect. Polling sleeps, coordinate fallbacks, and direct `AppAction`,
   `pending_action`, `AppState`, document, workspace, or search mutation are
   forbidden.
4. Workspace selection is split into two proofs: (a) the KatanA owned UI
   control naturally starts its `rfd::FileDialog::pick_folder` route, then (b)
   the native driver resolves and operates the resulting system dialog. A
   selected path must equal the execution workspace root. After the source
   owned asynchronous workspace scan has completed, the driver resolves the
   pre-created Markdown tree entry in a new current AX snapshot and invokes
   that physical control. The later active KatanA document must be that exact
   file. Workspace completion alone is insufficient because the fixed source
   does not activate a document for a fresh workspace. Either phase failing
   invalidates the complete bootstrap chain.
5. Each bootstrap record contains the fixed revision, canonical generated
   manifest/lock fingerprints, child process identity, pre/post accessibility
   snapshot digests, source locator digest, input kind, configuration sandbox
   digest, workspace fixture digest, and final workspace/document effect. It
   contains neither coordinates, raw typed text, KLE request content, nor a
   mutable KatanA-state serialization.

The first implementation leaf is macOS workspace bootstrap only. It must
produce a real active document by the sequence above and prove isolation of the
configuration sandbox before any KUC/KLE token, Storybook artifact, markdown
toolbar, document search, replace blocker, or emoji parity case is attempted. Windows and Linux
require their own native-driver implementations and profiles; macOS success is
not a cross-platform substitute.

### Source-Derived Native Target Input

The physical driver may not receive an accessible role, accessible name,
process identifier, coordinate, or path through an environment variable, test
argument, fixture, or user-supplied configuration. A release case instead
loads one fixed-revision source-closure target record containing only the
source span, role digest, sorted duplicate-free accessible-name digest set
generated from every bundled locale's `menu.open_workspace` value, and
source-closure/profile fingerprints. The record is generated from the fixed
source and is validated before the child is started. The native driver
constructs its locator directly from those digests and never needs the raw
label or locale. A missing, stale, mismatched, or non-generated target record
is a failed case, not a prompt for manual environment setup. The child PID
remains internal to the native-driver module; only an execution-local digest
may enter evidence.

## Native External Driver Contract

The native driver is test infrastructure in `tools/katana-host-e2e`; it is not
product code in KLE, KUC, or KatanA. It must run in an isolated macOS test
session and fail closed before a leaf starts when the required operating-system
capability is absent. It may use Accessibility APIs to operate the real native
file dialog and AppKit pasteboard APIs to install and restore test clipboard
contents. Coordinate-only automation, fixed sleeps, and a manual operator are
prohibited.

### Preconditions and containment

1. Verify macOS version, Accessibility authorization, Pasteboard access, and
   the target app/window accessibility identity before changing any external
   state. Missing authorization is a failed required environment check, never
   a skip.
2. Create a unique temporary workspace, saved Markdown document, deterministic
   PNG fixture, and a manifest recording paths and SHA-256 values. The driver
   must validate that the selected file and resulting asset are inside this
   workspace.
3. Snapshot the complete clipboard item type set and data before the case.
   Restore it after each case and verify restoration. Failure to restore fails
   the run and preserves diagnostic artifacts.
4. Drive file-picker controls by queried accessibility role, label, value, and
   window ownership. Never use hard-coded screen coordinates or delays. An
   unexpected picker, multiple matching controls, or wrong selected path fails
   the case.
5. The file-picker case invokes the unchanged KatanA source-derived UI route in
   non-headless mode. The native driver waits on Accessibility notifications
   from a separate test worker while the real dialog is blocked, selects the
   generated path by accessibility identity, and waits for the named KatanA
   host effect. It may not set `KATANA_HEADLESS`, inject an action, inspect
   `pending_dialog_action` as final evidence, or use a sleep/retry loop to guess
   dialog readiness. Missing expected accessibility events fail the case through
   the test runner's normal event-driven deadline.

### Required native leaf sequences

| Leaf | Native setup | Same-run KatanA assertions |
| --- | --- | --- |
| `image.file` | Correlate the KLE opaque request, invoke KatanA's real source-derived image-picker UI route, and select the generated PNG through its accessibility tree | Exact fixture SHA-256 is written under the configured asset directory; Markdown link resolves to that asset; active document becomes dirty; `RefreshExplorer` is scheduled; no unrelated document changes. |
| `image.clipboard-image` | Put a valid deterministic PNG on the macOS pasteboard, correlate the KLE opaque paste request, then invoke the actual KatanA paste UI/input route | Same asset/link/dirty/explorer assertions as file ingest, plus exactly one image intent, no text insertion, and insertion/replacement at the pre-paste selection. |
| `image.clipboard-file-list` | Put exactly one supported image file item and unrelated file items on the pasteboard | The first supported item is selected by KatanA; its exact bytes are written and linked. |
| `image.clipboard-file-url` | Put a percent-encoded image `file://` string on the pasteboard | KatanA decodes the intended image, writes exact bytes, never inserts the URL as editor text, and inserts/replaces Markdown at the pre-paste selection. |
| no payload/error/reference/unsaved | Install no image, invalid image, unreadable file URL, reference document, or unsaved document as appropriate | Buffer, dirty state, asset tree, cursor, and pending action remain unchanged except for the documented KatanA error status. |

The file-list and URL variants may share the same generic KUC paste request but
must have distinct native setup, source branch, final-effect assertion, leaf ID,
and recorded evidence. The test driver must not inspect payload data on behalf
of KLE or KUC; it only prepares the operating-system input and observes the
unchanged KatanA host result.

The source boundary is explicit: KatanA
`views/panels/editor/paste.rs::EditorPasteOps` currently recognizes image
`file://` text, percent-decodes it, and determines supported image extensions
before routing to `IngestClipboardImage`. KatanA
`app/action/clipboard_image.rs` then orders raw image, file-list image, and
file-URL acquisition; `app/action/image_ingest.rs` writes the asset and inserts
Markdown. Those are reference host operations. The native driver supplies only
the OS fixture needed to exercise them and must not reimplement their decision
logic in KLE/KUC test code.

## Bridge And Manifest Rules

1. The correlation bridge accepts only an opaque request/one-shot transport
   captured from the current public KLE RawInput case. It may neither construct
   a request nor convert it to `AppAction`/`pending_action`; it records only the
   host-issued correlation, revision, and source UI locator used by the
   physical KatanA input driver.
2. Every release leaf reaches exactly one KatanA action naturally emitted by
   its source-derived physical UI route, or an explicit non-action text
   synchronization contract. The compiled host-E2E graph may not contain an
   expected `UnsupportedAction`, `ExternalHostIntentUnavailable`, direct action
   injection, or `pending_action` assignment for a required leaf.
3. For every correlation/physical-input/action sequence the generated manifest
   records input trace hash, KUC frame hash, AccessKit node ID, source closure
   branch span, bridge correlation span, KatanA physical-input/action/handler/
   effect spans, and exact effect locator. The parity gate rejects an unmapped,
   reused, stale, non-executed, direct-injected, or coordinate-only locator.
4. The harness must assert KatanA reference revision, source hashes, and clean
   KatanA worktree before and after every run. A changed reference invalidates
   all cases.
5. The harness is a compatibility proof for the KLE action surface. It must not
   claim that unchanged KatanA has adopted KLE as its production widget. That
   adoption would require a separate, explicitly authorized KatanA change.

### Physical Host-Input Driver Contract

The source-derived host input driver is test infrastructure only. For every
host-effect leaf it captures the current unchanged KatanA frame and discovers
the target in that frame's AccessKit tree by the source-manifested role and
host-projected accessible-name hash. It records the KatanA frame hash, node ID,
role, name hash, and bounds hash before delivering the pointer/key/IME event.
The target bounds are used only from this same current frame; they may not be
fixed, reused after a frame change, inferred by KLE/KUC, or reconstructed from
document coordinates.

The driver never receives a KatanA action type, document path, line, range,
outline item, URL, clipboard payload, or semantic operation from KLE. The host
source manifest resolves the opaque target/revision/correlation to one physical
KatanA UI source locator and expected target role/name. The driver emits the
event, then records the action/handler only if unchanged KatanA emits it in the
following UI cycle. Missing, duplicate, stale, disabled, ambiguous, or
non-AccessKit target discovery is a fail-closed leaf result. Keyboard/IME
leaves additionally record the same current focused AccessKit target and may
not fall back to an action call.

## Current Action-Bridge Census

The legacy bridge is incomplete and direct-injecting by measured code, so this
table is a migration backlog, not a claim that any action is implemented. In
the `Current bridge result` column, `mapped` means a measured legacy direct
action mapping and is a red condition, never a passing route. The KLE action
set is defined in `crates/katana-language-editor/src/actions.rs:27-68`; the
current direct mapping branches are in
`tools/katana-host-e2e/src/action_bridge/action_bridge_translation.rs:26-60`.
The required route names the action that must instead be naturally emitted by
the source-derived physical KatanA UI input driver.

| KLE action family | Current bridge result | Required unchanged-KatanA route or rule |
| --- | --- | --- |
| save and format | mapped | `SaveDocument` and `FormatMarkdownFile(active_path)` with actual file/dirty/refresh observations. |
| Markdown authoring and 17 code kinds | mapped | `AuthorMarkdown` and exact buffer/cursor restore assertions for every source-derived command. |
| view mode, split toggle, code/preview toggle, split direction | mapped | corresponding view `AppAction`; state transition and rendered-coordinator effects stay separate leaves. |
| document-search next/previous | mapped only for those directions | `DocSearchNext` / `DocSearchPrev`; open/query/focus/zero-result/scroll must have their own KUC and host paths. |
| document-search query and close | not represented by a valid action-only bridge | `DocSearchQueryChanged` has no payload and reads the query that KatanA `DocSearchBar` already changed; close mutates UI state directly. Relay the current KUC one-shot query/close transport to an actual bootstrapped KatanA search UI input route, then observe refresh/close. Direct `SearchState`/`AppState` mutation, action-only fixture, fixed coordinate, mock, or raw query artifact is rejected. |
| document TabStrip select/traverse/close/pin/restore/reorder/group operations | not represented by the current bridge table | Map each closed opaque tab/group/placement/swatch request to its exact unchanged KatanA action and handler. KUC-only overflow/menu/popup/edit/palette transitions are retained effects. Group-name text is a non-persistent one-shot host submission; KLE carries no path, index, membership, hex color, or persisted tab state. Dirty confirmation-enabled close records the observed pending-state/no-document-mutation branch; `ForceCloseDocument` has no current KLE-root UI origin and must not be invented. |
| breadcrumb open/close and no selection | KUC retained UI | `BreadcrumbNavigator` retained menu/focus/overflow state records a `kuc_retained_ui_effect` plus unchanged bootstrapped KatanA observation. KLE does not derive paths, tree nodes or candidates. |
| breadcrumb candidate selection | not represented by the current bridge table | Map one current opaque document target, descriptor revision, and correlation to the host-authoritative `SelectDocument` request. KatanA owns path/tree resolution, document activation, persisted state and any filesystem effect. |
| source-address edit/history/menu | KUC retained UI | `SourceAddressBar` retains text focus/preedit/history-menu state and proves an unchanged host observation. KLE never stores URL history or parses text. |
| source-address submit | not represented by the current bridge table | This source-address raw-text crossing is a non-Clone/non-Serialize one-shot submission record, consumed immediately by the host bridge and passed unchanged to `OpenUrl`. KLE never stores, retries, parses, logs, or artifacts the text. The separate generic TabStrip group-name submission follows the same containment rule. The host E2E separately proves blank rejection, invalid input, real local-file handling, and real HTTP request/result/browser paths without direct `UrlTabState` setup or an `ehttp`/KRR mock. |
| toggle Problems panel, close, scope and disclosure | KUC retained UI | KUC generic StatusStrip/DiagnosticsPanel changes only same-root retained state. It records the `kuc_retained_ui_effect` and proves unchanged KatanA host state; KLE never maps this to `ToggleProblemsPanel` or direct `AppState`. |
| preview side-rail panel mechanics | KUC retained UI | KUC generic PreviewSideRail changes only rail/panel hover/open/focus/dismiss state. It records `kuc_retained_ui_effect` and proves unchanged bootstrapped KatanA host state; KLE never maps a panel name to KatanA state or mutates `AppState`. |
| preview TOC outline mechanics | KUC retained UI | KUC generic OutlineNavigator changes only hierarchy disclosure, expand/collapse, active presentation and generic scroll-into-view state. It records `kuc_retained_ui_effect` with unchanged host observation; KLE never maps outline index/level/anchor/source priority or mutates host TOC state. |
| preview side-rail refresh/search/export/story/tools/meta targets | not represented by the current bridge table | Forward one opaque host-issued target/revision/correlation. Refresh/export/story/tools/meta must reach their exact unchanged action/handler/effect; search follows the actual bootstrapped KatanA search-UI input relay. KLE has no format/path/line/panel/timer implementation. |
| preview TOC row selection | not represented by the current bridge table | Forward one opaque source target to the actual bootstrapped KatanA TOC UI input route and observe the current view-mode guarded scroll transition. Direct scroll-state mutation, target-to-line conversion, fixed coordinates and action-only fixtures are rejected. |
| file image, clipboard image, clipboard file URL | rejected as `ExternalHostIntentUnavailable` | Map the request to KatanA's own `IngestImageFile` or `IngestClipboardImage` only after the native driver prepares host input. KLE never carries raw bytes, file lists, or URLs. |
| diagnostic fix, fix-all, documentation | rejected as `UnsupportedAction` | Resolve the KatanA-held diagnostic/fix/documentation data at the host boundary, then execute `ApplyLintFixes`, `ApplyLintFixesForFiles`, or `OpenLinterDoc`; every final review/browser/buffer result is a separate leaf. |
| Problems fixes, location jump and documentation | rejected as `UnsupportedAction` | Resolve the KatanA-held diagnostic/fix/documentation data at the host boundary, then execute `SelectDocumentAndJump`, `ApplyLintFixesForFiles`, or `OpenLinterDoc`. Scope is KUC retained UI, while KLE must not add a local linter, batch, path or URL implementation. |
| gutter activation | rejected as `UnsupportedAction` | Resolve row/document/scroll target in KatanA host state and assert actual jump/selection/scroll outcome. |
| scroll sync and select-and-jump | rejected as `UnsupportedAction` | Route to KatanA's scroll coordinator and `SelectDocumentAndJump`; prove source suppression and PreviewOnly fallback separately. |
| close transient UI and host command | rejected as `UnsupportedAction` | Replace broad string command handling with a source-derived explicit typed host route, or remove the KLE-visible capability until a KUC/KLE/KatanA route exists. A stringly fallback is prohibited. |
| normal buffer update, text clipboard, undo/redo | not represented by the current bridge table | Use the actual KLE content/clipboard/history event transaction and KatanA `UpdateBuffer` / host shortcut routes. A fabricated `EditorActionRequest` cannot prove editor input. |
| user-mandated replace / replace-all | fixed KatanA source route absent | `docs/v0-1-0-editor-interaction-source-audit.md:36-37,61` is the controlling source fact. `AppAction::ReplaceText` is a lint/review single-span route, not general replace evidence. Keep the user requirement as a release blocker that needs an additional KatanA host specification decision; KLE must not add local query/range/mutation semantics, and KUC must not receive a Katana-specific API. |

`EditorAction::IngestClipboardFileUrls { urls }` is a current boundary defect.
The URLs originate in a host clipboard acquisition outcome, so KLE must not
receive, store, inspect, or translate them. Spark must replace it with the
payload-free request/resolution transaction: KUC reports paste intent, KLE
issues a correlated opaque host request, KatanA alone decides text/image/file
URL priority, then KLE receives only an accepted host-external document update
or a no-mutation/error result. `HostCommand { command }`, arbitrary authoring
command strings, and diagnostic documentation URLs are the same category of
defect: they must be replaced by source-derived closed variants or opaque host
targets. The parity and AST gates must reject every legacy string/URL-carrying
release route after the replacement is live. Raw user input is permitted only
through the source-derived single-consumption transports defined in
`docs/v0-1-0-neutral-host-request-design.md` (document edit, document search,
viewport/gutter, source address, and generic TabStrip group name). User-mandated
replacement remains a separate release blocker until the KatanA host追加仕様判断
defines a general route. Each accepted transport is consumed by the host bridge
and never serialized or retained by KLE.

## Superseding Neutral Event-Bridge Boundary

This section supersedes older `EditorEvent` / `EditorActionRequest` bridge
wording in this document. KLE does not own a typed editor-event queue, content,
origin, cursor, selection, diagnostics, action, clipboard, or save state. For
a direct-root leaf, KUC creates the public RawInput/AccessKit record and emits
either a closed opaque host target or one non-persistent, single-consumption
input transport. KLE forwards it once without inspection, storage,
serialization, range conversion, retry, or action construction. The unchanged
host resolves it against its own current state. `editor_shortcut` remains the
separate physical KatanA-router mode, joined only to same-run KUC focus/input
evidence and never claimed as KLE-mounted causality.

## Host Projection Provider Implementation Boundary

The concrete `HostProjectionProvider` for a release case lives only in
`tools/katana-host-e2e`. It is test-host infrastructure which depends on the
fixed, read-only `katana-ui` checkout; it is not a KLE library implementation
and it must not be exported by `katana-language-editor-egui`.

1. After the physical bootstrap has produced an active KatanA document, the
   harness captures the current KatanA frame and its AccessKit update. The
   host projection is derived from that current host observation and its
   source-closure locator, never from a KLE fixture, action, content cache, or
   reconstructed editor model.
2. The harness alone calls KUC's
   `EguiTextCommandSurfaceHostProjectionEncoder` with a host-owned opaque
   target and an execution-local monotonic revision. It retains the resulting
   token internally and supplies it once to KLE's public provider contract.
   KLE accepts and forwards the token without constructing, decoding,
   logging, cloning, serializing, or comparing its payload.
3. Every synchronized token is tied to the same execution correlation, fixed
   KatanA revision, source-closure record, KatanA frame hash, and AccessKit
   target record. The KUC root PNG and manifest are written only through the
   KUC artifact writer. The provider never exposes presentation data, target
   bytes, raw pixels, child geometry, or an AccessKit tree to KLE.
4. A bootstrap failure, source revision mismatch, missing/ambiguous/disabled
   current-frame target, stale revision, or missing required KatanA host fact
   yields a typed failed case. It may not issue a token, fall back to an
   aggregate, instantiate `EguiLanguageEditor`, synthesize an action, or
   substitute a fixture. This remains true for Storybook media generation.
5. The provider is evidence of KUC/KLE compatibility with an unchanged host;
   it does not claim that KatanA has mounted KLE in its production frame.
   Such adoption requires a separately approved KatanA change and is outside
   this release.

### Fixed Read-Only Build Selection

The checked-in development manifest may not be used as release evidence when
its `katana-ui` path resolves to a mutable working checkout. The physical
release harness derives the fixed KatanA checkout from the compiled KLE
repository layout after the protected bootstrap installs it at the standard
sibling location. It accepts neither a caller-supplied source path nor a CLI
or test-fixture source selector.

1. The runner validates that standard checkout before Cargo metadata or
   compilation: detached `HEAD` equals
   `4f6a6287c650a38633c7baeb544a92e739c68567`, its worktree is clean, and the
   source-closure profile checksum for that revision is present.
2. The protected bootstrap and compiled repository layout are the only source
   selectors. The execution record retains canonical source provenance hashes,
   never raw source paths. Implementation-specific temporary build metadata is
   canonicalized before hashing and is not a source artifact.
3. A normal `tools/katana-host-e2e/Cargo.toml` build is a development-only
   compile check. It cannot create a canonical host-E2E result, frame manifest,
   Storybook artifact, GIF, or MP4.
4. A dirty source, movable branch, mismatched dependency source, absent
   profile checksum, generated-manifest write failure, or metadata mismatch is
   a typed source-selection failure. The runner does not fall back to the
   ordinary local KatanA checkout or silently rewrite a release manifest.

## Native Test Input Boundary and Protected Runner

The historical physical test interface accepted `KATANA_FIXED_SOURCE`,
`SOURCE_CLOSURE_PROFILE`, `KATANA_NATIVE_TARGET_RECORD`, and
`KATANA_WORKSPACE_FIXTURE` as environment paths. That interface has been
removed from the physical test path because it allowed an external path, target
description, or fixture to select the application under test. Reintroducing it
is a release-gate failure.

The implemented standard input boundary and remaining execution requirements
are:

1. The native test derives one standard run root from the compiled manifest
   root. Its only accepted inputs are the checked-in/generated artifacts below
   the standard `target/source-closure` subtree and the fixed source revision
   installed by the protected run bootstrap. No profile path, target-record
   path, workspace-fixture path, raw target field, PID, coordinate, label,
   locale, or user configuration is accepted through an environment variable,
   CLI option, test fixture argument, or KatanA configuration.
2. The protected bootstrap installs the fixed detached KatanA source and the
   same-run assembled three-OS source-closure artifact under the standard run
   root. The native test creates its workspace fixture internally from the
   canonical workspace layout and a unique execution identifier. The fixture
   path is never an input or public evidence field; only its generated content
   digest and execution-local correlation are retained.
3. The source-derived target record is loaded only from the canonical
   `target/source-closure` location, checked against the fixed revision,
   profile-matrix fingerprint, source span, and generated schema, and then
   converted to an internal locator. Raw target data remains private to the
   driver. Any absent, stale, mismatched, duplicated, or externally supplied
   artifact is a failure, never a skip or fallback.
4. The three `macos-latest`, `windows-latest`, and `ubuntu-latest` captures are
   assembled in the same workflow run before native execution. The assembled
   artifact is transferred to a protected macOS self-hosted runner, which
   executes only the fixed native release harness and validates the artifact
   provenance before starting KatanA. PR-controlled arbitrary code must never
   execute on that runner; the protected dispatch is limited to release or
   otherwise explicitly protected runs.
5. Missing runner protection, Accessibility/input authorization, fixed-source
   verification, artifact transfer, profile-matrix agreement, target lookup,
   or final host-effect evidence is a failed case. The workflow may not turn
   any of these conditions into a skipped test, a warning, or a Storybook-only
   result.

Until this boundary is implemented and exercised, the physical bootstrap is a
design/implementation blocker only. Its lower-level library tests, source
closure tests, screenshots, video, Storybook output, and an in-process host
frame do not satisfy the native release evidence requirement.

## Event Bridge Requirement

An action-only bridge is insufficient. The current host runner receives an
`EditorActionRequest`, calls `KatanaApp::trigger_action`, and runs a KatanA
frame, but it has no consumer for ordinary `EditorEvent::ContentChanged` or
`ContentChangedWithOrigin`. That leaves normal typing outside the actual
KatanA `UpdateBuffer -> handle_update_buffer` route even when its source marker
is recorded.

Spark must replace the action-only runner with a leaf-scoped neutral relay. It
joins the exact public KUC root record, one KLE transit record, and the
class-appropriate unchanged-host route. It handles each source-derived input
class by an explicit direction and effect rule:

| Source-derived input | Required bridge behavior |
| --- | --- |
| ordinary user text transport | transit once without KLE retention; the host input driver replays the actual KatanA text/IME UI route, which naturally emits unchanged `UpdateBuffer`, then asserts dirty, preview/search/diagnostics refresh and active-document isolation |
| opaque authoring/command/diagnostic/navigation target | transit once; the host uses the target/revision/correlation only to select its source-derived physical KatanA UI route. KatanA naturally emits the action/handler. KLE cannot construct or enumerate `AppAction`. |
| host external/disk update | host-to-KUC-root projection only; KLE cannot replay an already applied KatanA change into `UpdateBuffer`. |
| clipboard paste request | execute the correlated host resolution transaction through the actual KatanA paste UI/input route; no KLE mutation, token state, selection/range conversion, payload handling, or direct action injection before accepted host resolution. |
| cursor, selection, diagnostics facts | KUC retained root/AccessKit and host observable facts only. They are not KLE event payloads and cannot be silently fabricated or discarded. |
| save/refresh/format host target | select one source-derived physical KatanA UI route from the host-issued opaque target and assert the naturally emitted action's named file/cache/document effect. |

The host-E2E manifest records every root input, KLE transit, host resolution,
and final effect. It rejects an unhandled, duplicated, cross-document,
wrong-direction, direct-action-injected, or locally completed transport. The
existing downstream simulator and Storybook counters do not satisfy this
requirement.

## 置換要件の release blocker

`docs/v0-1-0-editor-interaction-source-audit.md:36-37,61` を一次事実とする。
固定 KatanA revision では、一般文書の Replace / Replace All を起動する
検索bar、検索modal、context menu、keyboard、AccessKit route は確認されていない。
`AppAction::ReplaceText` と `DocumentEditOps::handle_replace_text` は
lint/review 等が渡す単一 byte span の置換 route であり、一般検索置換や
Replace All の完了根拠にしてはならない。

ユーザー明示の「テキスト置換」は削除しない。ただし現時点では
`source parity` と `user-mandated extension` が衝突しているため、
KatanA host 側で追加仕様を判断するまで release blocker として扱う。
KLE は query、match、range、byte 変換、document mutation、replace-all
loop、undo policy を実装しない。KUC は generic search UI / input /
AccessKit / focus だけを所有し、Katana 固有の `ReplaceText` API や
document semantics を持ち込まない。

既存の `crates/katana-language-editor-egui/src/search_control.rs` の
local `replace` / `replace_all` content mutation は release path として
不合格である。修正方針は local semantics の移植ではなく、追加仕様が
承認された後に KatanA host が所有する一般置換 route と KUC generic
SearchStrip を、KLE の一回限り transit だけで接続することである。
追加仕様が未確定の間は、replace / replace-all の visible support、
Storybook counter、callback、`Unsupported` 期待 test、または
`ReplaceText` single-span action の存在を完了証拠として受理しない。

## KUC/KLE root integration gate

KUC 側には generic root と platform text/raster の所有境界が存在するが、
現状の KLE main editor `show` は、その root を実消費して KatanA host
effect へ接続する証拠をまだ持っていない。Storybook の counter-only
forwarder、fixture callback、shape count、または screenshot/video 単体は
root consumption evidence ではない。

root integration は design、implementation、evidence の順に閉じる。
KUC が generic root、platform raster、AccessKit、font/emoji fallback を
所有する。KLE は opaque token、`show` の受け渡し、一回限り transit だけを
所有し、child render、font/emoji fallback、local search/replace semantics、
host state mutation を持たない。成功条件は actual main `show`、counter で
ない real host forwarder、同一 correlation で結合された KUC root /
AccessKit / host effect である。source inventory の全 leaf がこの形で
満たされるまで完了扱いにしない。

## Required Automated Artifacts

For each native case, retain the generated input fixture hash, pre/post host
state record, KUC root-frame hash, KatanA effect record, native accessibility
trace, and a numbered final KUC-frame PNG. Generate GIF/MP4 only from the
already validated KUC frames. Video helps review the sequence but never replaces
the source/KUC/host assertions.

## Release Rejection Conditions

The release gate rejects the run when any required external leaf is skipped,
ignored, manual, fixture-injected inside KLE/KUC, routed to a KatanA patch,
accepted as `UnsupportedAction` or `ExternalHostIntentUnavailable`, missing a
same-run final host effect, missing clipboard restoration verification, or
missing the recorded source/KUC/AccessKit/action/effect chain.
