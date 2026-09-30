# KUC Diagnostics Panel And Status Strip Design

## Status

This is the KUC-only design required for KLE v0.1.0 editor parity. The first
KUC-only `EguiStatusBarAdapter` and `EguiDiagnosticsListAdapter` slices are
implemented locally. They consume the generic core models, use
`PlatformTextRasterizer` for visible text, emit existing generic core events,
publish AccessKit actions, and produce independently composable paint plans.
The retained KUC root composes both children into the same artifact order,
frame hash, AccessKit evidence ledger, interaction locator, and one-shot event
transport. `StatusDiagnosticsProjectionLease` mounts them only through the
opaque host-projection lease consumed by the public root facade; the
`WorkspaceTabs` full-workbench scenario and KLE Storybook relay only that
closed root frame and receipt. Diagnostic severity, item, and quick-fix
locators use stable opaque identities rather than labels, paths, or diagnostic
IDs. The generic status slice renders progress, icon, tooltip, and interactive
popover lifecycle; the diagnostics slice keeps a virtualized retained viewport
and generic disclosure state whose locator and AccessKit tree contain only
visible rows. Generic per-entry code-diff fix preview and generic scope
selection are implemented through pointer, keyboard, and AccessKit interactions.
The full Problems panel and fixed-host evidence remain unimplemented. KatanA
remains read-only. KLE must not render a status bar,
Problems panel, diagnostic list, segmented scope toggle, fix preview, tooltip,
scrollbar, collapse control or local accessibility tree.

## Source Boundary

KatanA mounts `StatusBar` and `ProblemsPanel` from
`views/app_frame/ui.rs`. The editor-relevant source paths are:

| Source | Observed behavior | Boundary |
| --- | --- | --- |
| `views/top_bar/status_bar.rs` | status message/severity icon, dirty marker, Problems count/action, host activity text | KUC renders generic status items and emits one typed status event. KLE maps host-provided semantic presentation. KatanA owns status, dirty, export activity and Problems state. |
| `views/panels/problems/ui.rs` | open/close, scope, expand/collapse, file groups, zero-state, panel/file/bulk fix | KUC renders generic diagnostic panel state. KLE maps diagnostics and events. KatanA owns diagnostic data, visible path set and lint mutations. |
| `problems/{scope,diagnostics_renderer,fix_preview_*}.rs` | active/open-tab scope, official-diagnostic filter, file collapse, jump, docs, fix, preview and truncation | KUC owns generic input/presentation/accessibility. KatanA-specific linter semantics and all host effects stay outside KUC. |

The KatanA export spinner and filenames are application activity, not a
language-editor mutation route. The source-closure generator retains their mount
and input-free presentation as source-spanned non-editor helpers. KUC still
accepts generic status activity descriptors so KLE Storybook can render the
same status-strip state without creating a KLE-specific widget.

## Generic Inputs

The same opaque full-editor root receives a host-authoritative
`StatusDiagnosticsInput` with a stable root identity and increasing revision.
Conceptually it contains:

| Input | Required generic content | Explicitly not KUC-owned |
| --- | --- | --- |
| `StatusStripInput` | localized status text, severity token, optional dirty marker, generic activity descriptors, diagnostic count, Problems command capability and accessible labels | KatanA status classification, export task and document dirty calculation; KUC owns retained Problems visibility |
| `DiagnosticsPanelInput` | allowed stable scope keys with localized labels, file diagnostic groups, generic visible text/icons/tokens, explicit capabilities and diagnostic summary | linter rule evaluation, document/path discovery, file content acquisition and fix applicability |
| `DiagnosticFileInput` | opaque file key/label, scope-membership keys, diagnostics, collapse identity, fix-file capability and opaque host fix handle | KatanA path parsing, filesystem access, source snapshots and batch construction |
| `DiagnosticEntryInput` | opaque entry key, generic severity token, rule label/message, source-location label, select/docs/fix capabilities, optional generic preview input and AX strings | KatanA `MarkdownDiagnostic`, official-meta interpretation, browser URL ownership, byte/character conversion and actual lint fix |
| `TextDiffPreviewInput` | prevalidated original/replacement text range or rows, truncation facts and localized labels | a KLE-local diff model or a KatanA linter type in KUC |

KUC may expose a generic line-replacement-preview helper that accepts generic
text, one-based logical line range, replacement and output limit. It returns
typed invalid-range/no-content results. KLE maps KatanA data once into that
generic request or receives prevalidated rows from the host; it may not clone
the current `fix_preview_model` algorithm. The fixed source behavior of at
most ten removed/added rows, missing-content state and invalid-range no-preview
remain source-derived leaves until the closure proves every condition.

Status, diagnostic membership, fixability and host-action capabilities are
host-authoritative. KUC owns generic retained editor-presentation state:
panel visibility, selected stable scope key, expanded file keys, scroll, hover,
active tooltip and current focus. The source KatanA UI currently stores some of
these generic states directly in `AppState`; that is reference behavior, not a
reason for KLE to mutate KatanA state. KUC selects visible generic file groups
from host-supplied scope membership and records all state transitions in the
same opaque root frame. KLE supplies no local mirror state.

### Scope Selector Contract

KUC represents every selectable diagnostic set with `DiagnosticScopeInput`: an
opaque stable key, a host-localized visible label, and an accessible label. Each
`DiagnosticItem` provides only a set of those stable keys. KUC has no
`OpenTabs`, `ActiveTab`, path discovery, tab identity, linter rule, or
label-to-scope conversion. The host supplies membership on every authoritative
input revision; KUC filters the already supplied generic list and retains only
the selected opaque key. A missing selected key resets to the first supplied
scope in deterministic input order, and an empty scope collection hides the
selector without inventing a fallback scope.

KUC emits `ScopeSelected { scope_key }` only for a current, visible enabled
selector target. Pointer, keyboard, and AccessKit activation share the same
target resolver. The resolver hashes the key before it enters an interaction
locator, rejects stale/off-screen/replayed targets, and never emits the
localized label. A scope selection is a retained-UI event, so it changes the
same root frame, visible item count, artifact hash, and AccessKit tree while
the host observation remains unchanged. File/visible-scope fix requests remain
separate opaque capabilities and must not be inferred from membership.

Required KUC contract coverage is: zero/one/multiple scopes; localized
Japanese and exact `U+2B50 U+FE0F` labels; deterministic default and removed
selected key; pointer/keyboard/AccessKit selection; stale, hidden, disabled,
and replay rejection; matching visible-list, raster, artifact, and AccessKit
evidence. KLE tests may prove only opaque one-shot relay. The actual KatanA
OpenTabs/ActiveTab host effect remains a fixed-host E2E leaf and is not proven
by this contract.

## Generic Event Contract

Each event carries root identity, input revision, opaque transaction ID and
physical source (`pointer`, `keyboard`, or `accesskit`). No event may contain a
KatanA path, browser URL, linter fix payload or clipboard/file payload.

| Event family | Required individual events |
| --- | --- |
| KUC retained UI | toggle-panel, close-panel, select-scope, expand-all, collapse-all, toggle-file-expanded |
| diagnostic navigation | select-diagnostic-location |
| diagnostic host requests | request-file-fix, request-visible-scope-fix, request-entry-fix, request-diagnostic-docs |
| retained presentation | preview-opened, preview-closed, scroll acknowledgement, no-op/disabled activation |

KLE forwards only diagnostic host-request events. KUC retained-UI events update
the same KUC root state and are recorded as `kuc_retained_ui_effect`; they do
not become KatanA `AppAction` or a direct `AppState` mutation. The actual
KatanA host adapter turns only select/jump, fix and docs requests into
`SelectDocumentAndJump`, `ApplyLintFixesForFiles`, or `OpenLinterDoc`. KLE
cannot construct a lint batch, derive a scope from a label, calculate a byte
range, open a URL, mutate a document or complete a request locally.

## Root Integration And Evidence

`TextCommandSurfaceFrameRecord` includes same-frame `status_strip` and
`diagnostics_panel` records with semantic node IDs, revision, visibility,
selected scope key, visible count, expanded-file keys, active tooltip and hit
target. The same opaque root artifact includes their rendered pixels. The KUC
composition order is private and validates status/panel layer presence against
the authoritative input. KLE and Storybook see only the root frame and generic
event batch.

KUC typed errors include missing/duplicate diagnostic key, stale status or
diagnostic event, capability mismatch, missing current-frame semantic node and
invalid generic preview input. No error may fall back to a KLE panel, a static
fixture, an empty artifact or a previous root frame.

### Current-Source Integration Sequence

The KUC core models already exist as `StatusBar` and `DiagnosticsList`; this
work does not create an editor-specific replacement. The implementation must
add retained egui adapters that consume those generic models using the existing
KUC platform-raster and artifact-plan path. The integration order is fixed:

1. Add `EguiStatusStripAdapter` and `EguiDiagnosticsPanelAdapter` in KUC.
   `EguiStatusBarAdapter` and `EguiDiagnosticsListAdapter` are the in-progress
   first slices. Their current RawInput/AccessKit/raster/compositor contracts
   cover generic status text/segment press and diagnostic severity selection,
   item selection, quick-fix activation, and standard keyboard selection.
   Status progress/icon/tooltip/popover, diagnostic scope/disclosure/code-diff
   preview/virtualization, retained root placement, and opaque transport are
   implemented local KUC slices. The remaining source-derived Problems panel
   behaviors remain required work. Both adapters own retained layout, physical input, current-frame
   AccessKit evidence, generic paint plans and event collection. They cannot
   accept KatanA paths, URLs, linter types, document content or host action
   enums.
2. Add optional KUC-owned slots to `EguiTextCommandSurface`, its adapter and
   `EguiTextCommandSurfaceRoot`. The root owns all placement and focus order;
   KLE receives no child model, bounds, paint plan or event payload.
3. Extend the closed root child order, paint-plan reference and compositor
   validation. A status or diagnostics child must have exactly one same-frame
   plan and AccessKit record; missing, duplicate or mismatched pairs are typed
   KUC errors.
4. Extend the generic one-shot root transport and interaction locator. Retained
   panel visibility/scope/disclosure transitions stay in KUC. Location, fix and
   docs requests are opaque host capabilities only; KLE forwards them once and
   never decodes a `StatusBarAction`, `DiagnosticsListAction` or diagnostic key.
5. Add RawInput pointer/keyboard/AccessKit tests before Storybook composition,
   then add a KUC scenario and only opaque KLE relay evidence. Fixed KatanA
   host effects and three-platform emoji proof remain separate release gates.

## Required Automated Proof

- Actual KUC RawInput runs cover status severity/dirty/activity, zero/nonzero
  diagnostic count, pointer/keyboard/AccessKit Problems activation, panel
  close, both source scopes, expand/collapse-all, individual file collapse,
  empty state, every diagnostic severity, and disabled/capability-hidden paths.
- File, visible-scope and single-entry fix requests; location jump; docs;
  preview open/close; missing content; invalid range; and ten-row truncation
  each produce one current root record, event and AccessKit assertion.
- Japanese, exact `U+2B50 U+FE0F`, and ZWJ strings are exercised in status,
  diagnostic title/message, tooltip and preview through the shared KUC font
  catalog. The exact-star rule in the font-catalog design applies unchanged.
- Every interaction is tested against current revision and again with stale or
  mismatched descriptor keys; invalid operations produce a typed KUC error with
  no root event or document mutation.
- KUC retained-UI cases prove current frame/record/AX state and unchanged host
  observation. KLE actual host E2E separately proves only document-jump,
  fix/review/browser and no-mutation effects. Storybook media is attached to
  the same root frame/event/host step but is never a substitute.
