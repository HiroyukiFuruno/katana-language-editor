# KUC Text-Command Root Design

## Status

This is the required KUC-side design for KLE v0.1.0. It is not implemented
and must be implemented by Spark in the local KUC repository before KLE
consumer migration starts. KatanA is reference-only and no KatanA type, source
parser, Markdown transformation, file dialog, clipboard acquisition, linter,
preview, or shortcut-router behavior belongs in this design.

## Measured Current Boundary Violation

At the reviewed KUC tree:

- `text_command_surface/types.rs::EguiTextCommandSurfaceOutput` publicly
  exposes child text/chrome/context-menu outputs and `artifact_order()`.
- `text_command_surface/artifact.rs::artifact_paint_plans()` publicly turns
  that output back into child `ArtifactPaintPlanRef` values and uses `expect`
  for missing child artifacts.
- `text_command_surface/composition.rs::EguiTextCommandSurfaceAdapter::show`
  produces child outputs and the order but no final root RGBA artifact.
- therefore KLE and Storybook can reconstruct a generic surface. That is an
  ownership breach, not a consumer extension point.

## Required Public Contract

`EguiTextCommandSurfaceAdapter::show` MUST either return one complete
same-frame root result or a typed `EguiTextCommandSurfaceError`. It must not
publish a partial child result.

The exact Rust names may vary, but the public shape must be equivalent to:

```rust
pub struct EguiTextCommandSurfaceOutput {
    pub frame: EguiTextCommandSurfaceFrame,
    pub events: EguiTextCommandSurfaceEvents,
}

pub struct EguiTextCommandSurfaceFrame {
    pub root_id: UiStateId,
    pub bounds: UiRect,
    pub rgba8: Arc<[u8]>,
    pub dimensions: UiSize,
    pub pixel_sha256: FrameHash,
    pub plan_sha256: FrameHash,
    pub provenance: TextCommandSurfaceFrameProvenance,
    pub record: TextCommandSurfaceFrameRecord,
    pub accesskit: TextCommandSurfaceAccessKitSnapshot,
}
```

`TextCommandSurfaceFrameProvenance` expresses the validated KUC layer roles
and same-frame identities but does **not** reveal a paint plan, a child
artifact, an egui shape, a raster cache, a texture handle, or child geometry.
`TextCommandSurfaceFrameRecord` is an aggregated generic KUC record for a
document tab strip, status strip, diagnostics panel, text, selection,
gutter/annotations, command chrome, search and context menu. It
may expose semantic node IDs and bounds needed for an external assertion, but
not coordinate algorithms or child composition APIs.

`EguiTextCommandSurfaceEvents` is an ordered KUC-generic event batch. KLE may
map its event variants to KatanA-specific DTOs. KLE must not infer an event
from labels, pixels, child order, raw egui input, or an AccessKit tree.

The following must be private to KUC production consumers:

- `EguiTextCommandSurfaceChild` and child order;
- `artifact_order_for_root`;
- `artifact_paint_plans` and every `ArtifactPaintPlanRef` path;
- child adapter output required only to produce the root frame;
- egui UI IDs, popup memory, cursor rectangles, clipping, layer geometry and
  font/raster/texture state.

KUC-internal tests may use an explicitly test-only inspection interface. That
interface cannot be exported through KLE or be used by KLE Storybook.

## Generic CommandChrome And Authoring-Target Contract

`CommandChrome` is a generic retained KUC child, not a KLE authoring widget. A
host projection supplies an ordered collection of opaque item and group keys,
availability, localized presentation text, icon props, and one opaque host
target for each activatable item. KUC renders and accesses those descriptors
without parsing a command ID, shortcut, code kind, Markdown operation, URL, or
KatanA type.

KUC owns floating-toolbar and nested-dropdown visibility, focus transfer,
outside/Escape dismissal, disabled behavior, anchor measurement, viewport
clamping, keyboard/pointer routing, and AccessKit. The anchor is derived from
the root's own TextSurface state; no cursor coordinate, popup state, or lifecycle
object enters KLE. A successful activation emits only the current opaque host
target, root revision, and correlation. KLE forwards that exact tuple once.
The unchanged host validates it against the projection it issued and resolves
the target to its source-derived action such as `AppAction::AuthorMarkdown`.

Consequently, `EditorAuthoringCommand`, `EditorCodeBlockKind`,
`EditorAuthoringMenuState`, `EditorAuthoringMenuLifecycle`, command-id strings,
code-kind/info-string conversion, and a KLE-side Markdown-operation enum are
all forbidden from the release path. The generated KatanA closure proves the
complete authoring inventory and its action mapping; KUC proves generic
interaction for each injected opaque item. An unknown or stale target is a typed
error/no-op according to the current descriptor, never a fallback command.

### Root Read-only Policy

`SanitizedDocumentRootInput` carries a generic host-authoritative `readonly`
attribute as part of its complete revisioned snapshot. KUC maps it directly to
the existing `TextSurface` input policy: physical text, IME commit/preedit,
paste, cut, undo, and redo must not create a content mutation while read-only;
selection, scrolling, focus traversal, and the AccessKit read-only state remain
available. A change to `readonly` at the same root revision is a typed revision
conflict, never an implicit local policy update.

Read-only does not mean that KUC may infer that every projected command is an
authoring command. Top, floating, and context command availability remains the
opaque host projection's per-item responsibility. The host disables or omits
commands whose semantics would mutate its document; KUC preserves the generic
behavior of any enabled command without parsing its target. This keeps KatanA
document semantics out of KUC while making the root's text-input policy strict
and independently verifiable.

### Closed Floating Artifact Boundary

The production `SanitizedDocumentRootFrame` and its public record expose only
root identity/revision, root dimensions, and opaque integrity hashes. They may
not expose a floating command label, target bytes or fingerprint, child bounds,
child artifact/texture/RGBA bytes, child paint plan, or an AccessKit node list.
The root-level hashes remain allowed precisely because they are not reversible
child readback. Test-only current-frame geometry may exist behind `#[cfg(test)]`
to drive physical input, but it is not a production method, field, Debug value,
transport payload, or Storybook/KLE contract.

The release guard must open a floating command through the real root selection
path and prove that public Debug/record/forwarding values do not reveal the
injected localized label or opaque target. It must also reject source/API
changes that make test-only child geometry available in non-test builds. This
is a KUC boundary proof only; it cannot stand in for a KLE consumer or KatanA
host effect.

## Generic Text, Clipboard, and History Transaction

The root owns generic text editing, selection, IME, text-clipboard presentation
and retained history. It does not own KatanA document state, Markdown parsing,
image/file clipboard acquisition, filesystem access, or the host shortcut
router. The public contract must make these two categories impossible to
confuse.

For each stable opaque `UiStateId`, the root receives a host-authoritative
content snapshot with a monotonically increasing revision and emits only these
generic outcomes:

| Outcome | KUC responsibility | Prohibited consumer behavior |
| --- | --- | --- |
| user content mutation | one ordered `ContentMutation` with transaction ID, base revision, resulting UTF-8 text, selection, and origin (`typing`, `paste_text`, `cut`, `undo`, or `redo`) | KLE must consume it once into the host bridge without re-editing, cloning, serializing, coalescing, or keeping a parallel undo/content/origin state. |
| host content synchronization | atomically adopt the acknowledged host snapshot and preserve/record the generic history transition exactly once | KLE must not replay it as user input or synthesize `UpdateBuffer`. |
| text clipboard copy/cut | KUC issues its own generic framework clipboard-write effect from the selected UTF-8 text; cut also emits the one content mutation | KLE and Storybook must never inspect, cache, serialize, or write the copied text through a local clipboard implementation. |
| paste request | KUC emits one correlated opaque paste intent and waits for the host resolution before changing text | KLE must not parse `file://`, inspect clipboard MIME/file data, or decide image precedence. |
| paste resolution | `text` performs one `paste_text` mutation; `host_owned_image`, empty, error, stale, or read-only resolves with no text mutation | KUC must not acquire raw image/file data; KLE must only forward the correlation token; KatanA alone decides image/file acquisition. |
| history request | KUC applies retained undo/redo to the exact `UiStateId` and emits one resulting content mutation | KLE must not return `Unsupported`, map it to a host-global shortcut, or share history between documents/workspaces. |

The exact Rust names may vary, but a content mutation must carry the surface
identity, base revision, transaction ID, origin, full result text, and
character-safe selection/caret state. It is a single-consumption input
transport at the KLE boundary: the transport type is not `Clone`, `Serialize`,
or `Deserialize`, and KLE/Storybook manifests record only kind, revision,
length and SHA-256. A host acknowledgement must carry the same transaction ID
or a distinct host-origin revision. Missing, duplicate, out-of-order,
cross-surface, stale, or read-only mutations are typed errors; they may not be
repaired by a consumer fallback.

KUC may use its generic framework adapter to issue a text clipboard write, but
the platform operation is opaque to KLE. Raw image/file-list/file-URL clipboard
acquisition remains an unchanged KatanA host operation. The actual KatanA
shortcut router keeps its own context, availability, modifier-specificity and
first-match arbitration; KUC receives only the input that survives that router
and must not recreate its command inventory.

KatanA currently records host-origin external transformations in an egui
`TextEdit` undo state. That source behavior is an oracle, not a license for two
independent retained histories. Before consumer migration, Spark must prove the
single authoritative KUC history transition for a host-authoring, replace,
format, lint-fix, and external-refresh acknowledgement, then show the
equivalent KatanA document effect through actual host E2E. Until that proof
exists, history parity is blocked.

## Generic Search And Replacement Contract

The opaque root owns a generic `SearchStrip` child by extending/composing the
existing KUC `structured::SearchControlStrip` and command-chrome search adapter
internally. It retains query and
replacement text editing, IME/preedit, focus, open/close, result list
presentation, current-result visual state, disabled controls, keyboard,
pointer and AccessKit. It has no Markdown, regex engine, document text parser,
match range, file path, linter, URL, KatanA action, or byte-offset model.

Each query update, replacement update, replace-current, replace-all,
next/previous, and close event carries the root identity, descriptor revision,
opaque transaction and opaque current-result target where relevant. Query and
replacement text use single-consumption non-Clone/non-Serialize transports; the
other events are closed typed events. KLE forwards the transport or opaque
event exactly once and has no `SearchQuery`, match list, active index, regex,
range, line, scroll target, or local content mutation state.

The host validates the revision and resolves the current document snapshot,
Markdown-aware query semantics, match ranges, active result, character/byte
conversion, read-only/stale/document-switch behavior, undo, dirty/refresh and
scroll/preview effects. A no-match or disabled KUC event is a retained UI
transition; it must not construct a host action. An accepted replace event
reaches unchanged KatanA `ReplaceText` only after the host derives its exact
current byte range. The host returns the next opaque SearchStrip descriptor;
KUC never optimistically edits host content or retains a synthetic match list.

For the fixed KatanA document-search route, query and close require a stricter
host boundary than next/previous. `AppAction::DocSearchQueryChanged` carries no
query and reads KatanA UI state already updated by `DocSearchBar`; close is also
a UI-state branch. KUC therefore emits a linear query or close transport with
the current root revision/correlation. KLE forwards it without inspection,
copying, serialization, or persistence. The host integration must drive the
same value or close interaction through the bootstrapped unchanged KatanA
search UI and then return the authoritative result descriptor. It may not
assign `SearchState`, construct a KatanA query state, or substitute an
action-only fixture. A host unable to perform that physical UI relay rejects
the leaf; KUC/KLE do not add a Markdown search engine as a fallback.

KUC tests must cover physical query/replacement input, Japanese/`⭐️` VS16/ZWJ,
IME, open/close/focus, Enter/Shift+Enter/Up/Down/Escape, disabled/no-match,
current/all request, stale revision and AccessKit. KLE host E2E then proves
the corresponding actual KatanA filter/navigation/replace/no-mutation effect.
Neither a KLE search engine or second search UI nor a Storybook
callback/counter is admissible. The current KUC core's serializable search
model must not cross the opaque root as KLE-owned persisted state: raw query
and replacement values leave only through the defined single-consumption
transport.

## Generic Retained TabStrip Contract

This is an extension of KUC's existing structured tab and diagnostics building
blocks, not permission to create parallel KLE widgets. No current KUC
`CloseableTabStrip` alias was found; `WorkspaceTabBar` is the reusable
substrate. Spark must extract/generalize it into `TabStrip` so the new root
input/output contains no `Workspace*`, KatanA, filesystem, Markdown, or
document-path domain concept. The existing KUC `StatusBar` and
`DiagnosticsList` similarly become internal root children. Their retained
state/event contracts must be preserved and extended where the source-derived
leaf catalog requires it; KLE may not wrap them in its own panel, palette,
layout, or event state.

The root accepts a host-authoritative generic `TabStripInput` on every frame.
It contains a stable surface identity and monotonically increasing descriptor
revision, generic tab descriptors, generic group descriptors, a
recently-closed capability, and presentation strings/icons/tokens. A tab
descriptor uses an opaque stable key, title, tooltip/accessibility text,
active/dirty/pinned state and capability flags. A group descriptor uses an
opaque stable key, label, color token, collapsed state and capability flags.
The flags express what is currently allowed; KUC must not infer KatanA concepts
such as virtual paths, demo documents, workspace roots or filesystem state.

KUC owns only retained generic interaction state: horizontal overflow position,
current focus, hover, open menu/popup, pending inline-editor focus, drag source
and candidate drop target. It never mutates the authoritative tab order,
document selection, dirty/pinned state, group membership, group name/color or
recently-closed collection locally. A KUC event is followed by a host action
and then by a new `TabStripInput` revision. Optimistic authority or local
rollback is prohibited.

The public generic event union must represent individual operations, not an
untyped command string or an aggregate tab event:

| Generic event family | Required event variants |
| --- | --- |
| selection and navigation | direct select, previous, next |
| close lifecycle | close, close-other, close-all, close-right, close-left, restore-closed |
| arrangement | toggle-pinned, reorder, add-to-group, remove-from-group |
| group lifecycle | create-group, rename-group, recolor-group, close-group, ungroup, toggle-collapse, cancel-inline-rename |
| retained presentation | popup closed, focus moved, scroll-to-active acknowledged, drag cancelled/no-op |

Every event contains the root surface identity, descriptor revision, stable tab
and/or group key where relevant, an opaque transaction ID and its physical
source (`pointer`, `keyboard`, `accesskit`, or `drag-accessible-alternative`).
KLE forwards that opaque generic event once but must not derive an index, group
membership, route, label, capability, or KatanA request from its own local
state. The unchanged host resolves the event and is the only authority for dirty-close
confirmation, force-close, document loading, persistence and filesystem side
effects.

Rename is a payload-bearing single-consumption group-name transport with only
the current opaque group key, root revision, transaction ID and edited UTF-8
text. It is not `Clone`, `Serialize`, or `Deserialize`; KLE forwards it once to
the host and may record only length/SHA-256. Recolor emits a host-projected
opaque swatch ID, never a color hex value. The KLE projection has no group
membership, name, color, index, drag geometry, or popup state.

The fixed KatanA revision does not provide a document-editor renderer or
shortcut origin for `ForceCloseDocument` after a dirty close. Therefore the
KLE full-editor projection must omit a force-close capability and no KUC event
from this root may map to it. A generic KUC component may support an optional
close-confirmation capability for another future consumer, but that capability
is absent from the KLE descriptor revision until a reviewed KatanA source
revision provides an origin.

## Generic Editor-Frame Navigation Contract

The opaque root also owns generic retained `BreadcrumbNavigator` and
`SourceAddressBar` children. These are generic controls; their public KUC
contracts contain no path, filesystem, workspace, URL, browser, KRR, KDV,
KatanA or Markdown type.

`BreadcrumbNavigatorInput` carries a surface/revision key, ordered opaque
segment descriptors, final/non-final presentation capability, and opaque child
candidate descriptors for each currently exposable non-final segment. KUC owns
segment layout, overflow, disclosure, candidate-menu lifecycle, focus, pointer,
keyboard and AccessKit nodes. It emits only `BreadcrumbSegmentOpened`,
`BreadcrumbCandidateSelected`, and retained close/focus events with stable
opaque IDs and the input revision. KLE forwards an enabled candidate selection
once without deriving a host request; it does not split paths, derive prefixes, inspect
filesystem data, filter candidates or calculate geometry.

`SourceAddressBarInput` is a host-controlled descriptor with a revision, current
display value, opaque bounded history descriptors, submit capability and host
status/error presentation. KUC retains only input focus, preedit, history-menu
visibility and transient selection. It emits value-change and submit (button or
Enter) single-consumption transports, opaque history selection, and retained
lifecycle events. Empty submission is a KUC no-host-event transition. KLE
immediately forwards the transport or opaque ID and stores neither; URL parsing,
history deduplication, local-file access, networking, source bytes, document
identity and browser state remain host operations.

Every current generic source-address/breadcrumb event has a root record and
AccessKit node. KUC test fixtures use opaque descriptor IDs and never fabricate
an URL response, filesystem tree, browser frame or KatanA action.

`TextCommandSurfaceFrameRecord` includes a `tab_strip` record for the same
root frame: descriptor revision, visible tab/group semantic node IDs, active
key, overflow/focus/popup/drag state, capability state and current hit target.
It may expose assertion bounds but no layout algorithm or child rectangles.
The root AccessKit snapshot exposes equivalent select, close, navigation,
pin/group/menu and drag-alternative actions. Pointer drag may never be the only
way to reorder or alter a group.

The generic presentation accepts host-provided localized strings, icon props,
color tokens, dirty/pin markers and close/menu capabilities. KUC owns layout,
overflow thresholding, clipping, drag targets, palette rendering, popup
lifecycle, text input shaping/rasterization and accessibility. KLE supplies
only typed presentation values; it may not recreate any tab geometry, text
input, palette, popup or context menu to imitate the source UI.

The KUC root returns `TabDescriptorMismatch` when references are absent or
capabilities do not permit an operation, `StaleTabEvent` when an event revision
is not current, and `TabStripAccessibilityMismatch` when a visible interactive
tab/group lacks a same-frame semantic node. These errors are publish-time root
errors, never KLE fallbacks.

Required KUC root cases include: no tab; one active tab; multiple tabs with
and without overflow; Japanese, exact `⭐️` VS16 and ZWJ titles/tooltips; active
and inactive selection; dirty close confirmation request; pinned close as a
pin event; virtual/capability-hidden actions; every close scope; restore
available/unavailable; drag move, in-place group change, no-op and cancellation;
collapsed/expanded group; group create/add/remove/rename/recolor/ungroup/close;
inline rename focus/Enter/outside close; pointer/keyboard/AccessKit equivalence;
and stale/invalid event rejection. These KUC tests prove retained root behavior
only. The matching KLE host E2E must additionally prove the actual KatanA state
and persistence effect for every source-derived leaf.

## Generic Retained PreviewViewport Contract

The same opaque full-editor root includes a generic `WorkspaceViewport` with
`CodeOnly`, `PreviewOnly`, and `Split` composition. `SplitViewportInput` is
host-authoritative for axis, pane order, stable layout key and initial extent;
`PreviewViewportInput` is host-authoritative for stable identity, revision,
typed frame, search/scroll request and capability facts. KUC owns retained
split extent, resize drag, focus, clipping, preview scroll, hover, overlays,
keyboard/AccessKit navigation and all associated geometry/raster output.

KUC does not depend on KDV and does not accept KatanA/Markdown/parser/browser
types. Its generic frame union contains only structured preview nodes, document
frame cells/pages, external-browser raster/input metadata, explicit failure,
and explicit empty state. KLE is allowed to project public KDV DTOs into that
union, but may not draw, calculate geometry, parse Markdown, cache browser
pixels or synthesize an action. KDV already depends on KUC, so a KUC-to-KDV
dependency is a release-blocking cycle.

The root record adds current view mode, split axis/order/extent, preview
viewport bounds, scroll/anchor/source-target facts, hover/focus/overlay state,
typed preview events and equivalent AccessKit nodes. It must expose no child
preview artifact, egui panel/state, geometry algorithm or raw browser frame.
Missing or stale revision, impossible geometry, duplicate source target,
capability mismatch, unsupported generic frame or stale command correlation is
a typed KUC root error, never a KLE/Storybook fallback.

Generic preview event families are individual and typed: view mode, split
axis/order, resize acknowledgement, scroll/anchor request, hover target,
activate task/source target, document-frame command, browser input/navigation,
fullscreen and slideshow control. KUC does not mutate KatanA documents,
browser sessions or OS viewport state. KLE forwards only the current opaque
target/revision/correlation and waits for a new authoritative input revision. Pure resize/overlay/focus state
uses `kuc_retained_ui_effect` plus unchanged bootstrapped KatanA observation;
host document/view/native effects require the source-derived declared-effect
class and actual host evidence.

Required KUC cases are CodeOnly/PreviewOnly/Split; horizontal/vertical and both
pane orders; min/max resize and ratchet rejection; no document and failure;
Japanese, exact `⭐️` VS16 and ZWJ structured content; scroll/hover/anchor;
document-frame capability and stale-command rejection; browser frame/input
correlation; fullscreen/slideshow focus/close; pointer, keyboard and AccessKit
equivalence; stale/mixed identity/revision rejection; deterministic same-frame
hashes; and opaque-root artifact/record/AccessKit consistency. The detailed
KDV projection gate and source leaf minimum are in
`docs/v0-1-0-kuc-preview-viewport-design.md`.

### Generic Retained PreviewSideRail Contract

The same root includes a generic `PreviewSideRail` beside `WorkspaceViewport`.
Its input is only a revisioned ordered collection of opaque rail-item and panel
targets, icon/label/accessibility presentation, availability/capability and
host-projected content descriptors. KUC owns rail sizing, button hit testing,
focus, tooltip, overlay anchor, panel clipping, hover-switch delay, sibling
exclusion, outside/Escape dismissal, retained panel state, AccessKit and final
artifact. It does not receive `toc`, `export`, `story`, `tools`, Markdown,
format, path, document, KatanA, KDV, KRR, browser or slideshow types.

KUC supports generic overlay and host-projected pinned leading/trailing panel
placement; the placement is not a KatanA TOC enum. It emits either a purely
retained panel transition or one opaque target with the input
revision/correlation. KLE forwards that target once without panel name
branching, geometry, timer, toggle state, export format, document path, TOC
line or host-state mutation. The host resolves targets such as refresh,
search, export, slideshow setting, view setting and metadata against its own
current state. A TOC row is an opaque source target; KLE never calculates its
line or scroll request. Generic rail/panel pinned/hover/open/close/cooldown/focus/empty-content leaves
are `kuc_retained_ui_effect` plus an unchanged bootstrapped KatanA observation;
host-target leaves require the source-derived host effect. A local KLE side
rail, popup, `Area`, panel, hover clock or overlay artifact is prohibited.

`PreviewSideRail` internally composes a generic `OutlineNavigator` for any
hierarchical side-panel content. `OutlineNavigator` accepts only revisioned
opaque item targets, nesting relations, current active target, generic guide
capability and presentation strings. KUC owns item layout/truncation,
accordion disclosure, expand/collapse-all, active styling, generic
scroll-into-view, empty state and AccessKit. The host alone determines the
current active target from document/anchor/scroll semantics and resolves a
selected opaque target; KLE must not receive an outline index, level, line,
anchor, source priority, `TocState`, `TocCurrentOrigin`, or force-open flag.
Every active-target update and selection is revision/correlation checked.

Spark must build these components by extending/composing KUC's existing generic
`structured::CollapsiblePanel` (leading/trailing/pinned/overlay state),
`disclosure::HoverCard`/`Popover` (delayed overlay/focus/dismissal), and
`TreeView`/disclosure substrate (hierarchy/disclosure). The file-specific
`FileTree` public contract is not copied into the root. A second panel,
popover, hover scheduler, accordion, tree hit-test, or renderer in KLE is
prohibited; a parallel KUC implementation without an explicit incompatibility
test is also rejected.

## Same-Frame Composition Algorithm

1. Allocate the one actual KUC root in the incoming egui UI.
2. Render/synchronize retained generic children in KUC's ordering and input
   policy: document tab strip, breadcrumb navigator, source address bar, top
   chrome, search, workspace viewport (editor and preview), preview side rail,
   diagnostics panel, status strip, floating chrome, context menu. The generic `TabStrip`,
   `BreadcrumbNavigator`, `SourceAddressBar`, `WorkspaceViewport`,
   `PreviewSideRail`, `DiagnosticsPanel` and `StatusStrip` own their retained interaction and
   AccessKit behavior; KLE transparently forwards host-issued opaque descriptors
   and current opaque events once. This is internal KUC behavior, not public
   consumer policy.
3. Collect every required child artifact, frame record and AccessKit evidence.
   Validate its root identity, frame generation, dimensions, alpha format,
   layer role and order before composition.
4. KUC's own `ArtifactCompositor` combines the validated internal plans on the
   exact allocated root canvas. It creates `RGBA8`, hashes and aggregated
   semantic record.
5. Publish the immutable `EguiTextCommandSurfaceFrame` and the KUC-generic
   event batch only after all validation succeeds.

No error path may fall back to a child frame, an empty canvas, an alternate
renderer, a stale preceding frame or a consumer repair path.

## Typed Error Contract

The error enum must distinguish at least:

| Error | Required condition |
| --- | --- |
| `MissingArtifact` | a visible/ordered KUC child has no artifact |
| `UnexpectedArtifact` | an artifact exists for a child absent from the root state |
| `DuplicateLayer` | one semantic layer appears more than once |
| `InvalidLayerOrder` | the internally calculated order and assembled artifacts differ |
| `RootBoundsMismatch` | final canvas does not equal the current root bounds |
| `FrameGenerationMismatch` | records/artifacts/AX snapshot do not share the current frame identity |
| `PixelBufferMismatch` | RGBA8 length, dimensions, alpha or hash is inconsistent |
| `AccessibilityMismatch` | required current-frame KUC semantic node cannot be related to the frame |
| `PlatformTextFailure` | catalog/raster/shaping reports an explicit error |
| `TabDescriptorMismatch` | a tab/group key or currently allowed operation is absent from the current descriptor revision |
| `StaleTabEvent` | tab interaction revision does not match the current root input |
| `TabStripAccessibilityMismatch` | a visible interactive tab/group has no same-frame semantic node |

`expect`, panic recovery, a missing optional field, a stale root reuse and an
`Unsupported` consumer fallback are invalid substitutions for these errors.

## Boundary With KLE And Storybook

| Operation | KUC | KLE binding | KLE Storybook |
| --- | --- | --- | --- |
| text/IME/selection/gutter/menu rendering | owns | forwards opaque descriptor and single-consumption event only | drives public input only |
| document tab strip/menu/drag/accessibility | owns generic retained component | forwards typed document-tab descriptors/events without local state | drives public input and captures root record only |
| status strip/diagnostics panel/fix preview | owns generic retained component | maps typed diagnostic/status descriptors/events | drives public input and captures root record only |
| layout/hit-test/popup/focus/AX | owns | reads opaque current record only | asserts aggregate record/AX snapshot |
| final RGBA composition | owns | retains/forwards opaque frame | encodes frame bytes only |
| command activation | emits opaque target/revision/correlation event batch | forwards the same opaque host request only | captures same event trace |
| text clipboard/history transaction | owns generic protocol and retained state | forwards correlated transport/result only | captures root record and actual host effect |
| Markdown transform/save/format/lint/image/shortcut | never owns | maps request only | asserts actual KatanA host effect |
| platform font catalog/glyph/emoji | owns | never caches/falls back | never caches/substitutes |

KLE and Storybook are forbidden from importing `ArtifactCompositor`, child
paint plans, `artifact_order`, `egui::FullOutput::shapes`, `minifb`, local
glyph rasterization, local RGBA-to-window conversion or a fallback editor
surface. These are AST-lint release failures.

## Required KUC Tests

- RawInput drives actual retained root scenarios for text only, combined
  tab-strip/chrome/search/text/status/diagnostics, selected floating toolbar,
  every context menu state, and all layers present.
- Each successful result has non-zero root bounds, current generation identity,
  exact RGBA byte length, deterministic hashes, aggregate record and AccessKit
  nodes from that same frame.
- Negative tests independently prove every typed error above. No test may
  manufacture a passing final frame from child output.
- A Storybook or consumer test may not configure `EguiLanguageEditor` through
  `set_content`, `set_cursor`, `replace_diagnostics`, or
  `set_hovered_gutter_lines` and then treat `latest_kuc_frame` as current
  evidence. The 2026-08-14 gutter failure demonstrates that this can observe a
  stale frame. The only admissible replacement is a KUC-root RawInput scenario
  whose same-frame root record and AccessKit snapshot identify the gutter node
  before and after the actual event.
- Keyboard/pointer/AccessKit invoke the same root for editing, IME,
  command/menu activation with opaque injected targets, tab selection/close/reorder/pin/group events,
  search, diagnostics/status activation, gutter marker, focus return and
  scroll acknowledgement.
- Text copy, cut, paste, undo and redo each exercise the transaction table:
  selected/unselected, editable/read-only, empty/error/stale resolution,
  Japanese/VS16/ZWJ text, one mutation per transaction, per-surface history
  isolation, and host-origin acknowledgement without replay.
- The exact locked external reference branch families for selection, ordinary
  input, Tab/Enter, deletion, IME and history are covered by generated KUC
  root cases as defined in `v0-1-0-locked-ui-semantic-reference-audit.md`.
  That document is an oracle only; KUC must not import or delegate to egui
  text editing to satisfy a case.
- Image/file paste priority is tested only through an opaque KatanA-owned host
  resolution. KUC tests prove intent/no-text-mutation behavior but do not use a
  KUC or KLE image payload fixture, URL parser, or clipboard reader.
- Japanese IME, exact `⭐️` VS16 color glyph and ZWJ text are verified in the
  final root pixels and semantic record using the shared `PlatformFontCatalog`.
- Repeated same-input frames have deterministic frame hashes; a changed input
  changes only the expected current record/frame sequence. Tests prove catalog
  sharing and per-surface cache isolation as specified in
  `v0-1-0-kuc-font-catalog-design.md`.
- Status/Problems root behavior follows the generic input, event, ownership,
  typed-error and current-frame proof contract in
  `v0-1-0-kuc-diagnostics-status-design.md`.

## Migration Checkpoint

Only after these KUC tests pass may Spark remove KLE's
`EguiKucArtifactAggregate` and Storybook composition/fallback paths in the
same dependency-ordered migration. Existing child plan APIs must be absent
from the compiled KLE and Storybook graphs after that migration. Physical
legacy-file deletion occurs only at the separately recorded deletion
checkpoint, after the replacement path has strict coverage.
