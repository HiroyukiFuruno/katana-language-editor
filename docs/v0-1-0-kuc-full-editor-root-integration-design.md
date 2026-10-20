# KUC Full Editor Root Integration Design

## Decision

KLE v0.1.0 の full-spec Storybook は、KUC の generic retained full-editor root
だけを表示する。KLE は opaque projection lease を `show` に渡し、KUC root の
one-shot event batch を一度だけ host forwarder へ渡す。KLE に tab、breadcrumb、
source address、status、diagnostics、preview、layout、font、emoji、AccessKit、
coordinate、Markdown/search/replace semantics を追加しない。

固定 KatanA は read-only である。KatanA 固有の action、document、path、URL、
Markdown transform、検索/置換、file/clipboard、dirty/undo/preview effect は KUC
にも KLE にも置かず、fixed-host E2E の host effect port だけが解決する。

## Root Contract

KUC は一つの stable root identity と monotonically increasing presentation revision
を持つ `FullEditorRootProjection` を公開する。projection は全て host-issued opaque
target と localized presentation value で構成し、child component の structural ID、
geometry、raw event、font path、raster bytes を公開しない。

root layout region は次の順序で固定する。

1. document tab strip
2. breadcrumb and generic source-address strip
3. primary/floating command chrome and search strip
4. retained document/preview/split viewport
5. status strip and optional diagnostics panel
6. context menu, dropdown, tooltip, and modal overlays

同一 root が focus order、overlay z-order、viewport clamp、scroll、retained popover
state、AccessKit tree を所有する。child component を KLE が別 widget として描画した
り、複数 root の record を合成したりしてはならない。

## Generic Projection And Event Boundary

KUC projection は以下の generic family を持つ。各 family は revision、opaque
target、localized label/tooltip/accessibility text、capability、presentation state だけを
受け取る。

| Family | KUC owns | Explicitly excluded |
| --- | --- | --- |
| Tab strip | selection, horizontal overflow scroll/active reveal, close/pin/group/drag retained UI | document path, tab index, persistence semantics |
| Breadcrumb/source address | text input, IME, history disclosure, candidate/menu UI | URL/path parsing, filesystem/network, response payload |
| Text command | toolbar, dropdown, floating selection UI, search strip | Markdown transform, query/range/replace semantics |
| Viewport | split/layout/scroll/focus/overlay presentation | Markdown parser, browser/KDV raster, KatanA view state |
| Status/diagnostics | status, panel, scope/filter/collapse/preview retained UI | linter rule evaluation, file discovery, fix payload, browser URL |

Every physical pointer, keyboard, or AccessKit activation is converted inside
KUC to a current-revision `FullEditorRootEvent`. The event contains only root
identity, revision, opaque transaction/correlation, input origin, and generic
event class. KUC retained-UI events are resolved inside the root. Host-request
events are forwarded exactly once. Missing capability, stale revision, duplicate
consumption, invalid focus target, hidden/disabled activation, and forwarder
failure are typed fail-closed outcomes with no replay.

## Source Address Contract

`SourceAddressStrip` is a KUC-retained component. Its public projection has
localized visible/tooltip/accessibility presentation only; a history or
candidate entry's host target is private KUC state and has no getter, `Clone`,
`Debug`, or serialization transport. Selection changes the local draft and
emits a targetless generic event. It does not reveal an entry's target to KLE
or select a KatanA document by itself.

Direct submitted input is the only value-bearing boundary. It is represented by
a non-Clone, non-Debug, non-Serialize, consume-only submission value, travels
outside `EditorAction` and a frame/manifest/log, and is consumed exactly once
by the host forwarder. KLE does not inspect, store, validate, parse, deduplicate,
or serialize it. The fixed KatanA host alone decides whether that text has an
`OpenUrl` effect.

The KUC adapter must cover typed/IME input, Enter/button submission, focus and
blur, history/candidate opening and selection, disabled and stale state, and
AccessKit equivalents. Its root tests must prove that visual labels can be
rendered while opaque targets and submitted values never appear in records,
debug output, artifacts, or generic events.

## Existing Component Reuse

| Component | Existing KUC foundation | Required KUC work before root mount |
| --- | --- | --- |
| Workspace tab strip | `workspace_tab_bar` and `closeable_tab_strip_adapter` | opaque root projection, root event envelope, retained focus/overlay integration |
| Text viewport | `TextSurfaceViewport` and `EguiTextSurfaceAdapter` | root-region sizing and split/preview ownership; no Markdown renderer in KLE |
| Breadcrumb | generic `Breadcrumb` render model | egui adapter, AccessKit, opaque selection event, root retained state |
| Source address | none | new generic input/history/candidate model; no URL/path type or parser |
| Status bar | `StatusBar` model/state/events | egui adapter, AccessKit, root event projection |
| Diagnostics list | `DiagnosticsList` model/state/events | egui adapter, root bounds/hit/focus/event projection |
| Preview | generic viewport primitives only | generic semantic preview slot; no KDV/KatanA rendering dependency |

## Implementation Order

1. Define `FullEditorRootProjection`, closed root frame, generic event envelope,
   revision/correlation validation, and root layout-region records in KUC.
2. Mount the existing workspace tab strip in the KUC root and test retained
   select/close/horizontal-overflow-scroll/keyboard/AccessKit behavior from physical input.
3. Add the generic breadcrumb and source-address component. Verify IME,
   history/candidate lifecycle, focus return, stale/disabled handling, and
   no path/URL leakage.
4. Add status and diagnostics egui adapters, then integrate their retained state,
   overlay/focus ordering, and one-shot host requests in the root.
5. Add the generic split/preview viewport slot and prove root-owned sizing,
   scroll, resize, and overlay clipping.
6. Expand the full-surface KUC scenario catalogue so every region has physical
   pointer, keyboard, and AccessKit stages plus same-frame root/AX/artifact
   evidence. Keep Replace and Replace All visible but disabled until a fixed
   KatanA host contract exists.
7. Only after KUC steps 1-6 pass, make KLE consume the opaque full-root lease.
   Then run fixed KatanA host E2E for every source-derived effect; Storybook
   media remains review evidence only.

## Source Address Root Integration Plan

The first mount is deliberately a KUC-only vertical slice. It adds a generic
`SourceAddress` child class to the existing retained root, without exposing a
new KLE surface or a source-address-specific host callback.

Source address is explicitly **not** added to the JSON wire presentation.
Its history/candidate target and direct submitted draft are value-bearing and
must never be reconstructable from a serialized token. Instead, KUC defines a
non-Clone/non-Debug/non-Serialize `SourceAddressProjectionLease` that consumes
presentation-only entries together with KUC-private opaque targets. The host
may hand that lease to the retained KUC root exactly once; KLE only transits an
opaque root lease and has no source-address constructor, accessor, record, or
forwarder implementation. A missing, stale, duplicate, or incompatible lease
fails closed before render.

1. Add an optional non-wire `SourceAddressProjectionLease` to the KUC host-root
   factory and retain its `SourceAddressStrip` and egui adapter in the same root
   identity/revision lifetime as the existing text-command children. The JSON
   wire token remains source-address-free.
2. Render that child in the root-owned top strip before command/search/text
   regions. The KUC layout contract owns slot bounds and rejects overlap; KLE
   cannot position or separately render this child.
3. Extend the sealed root batch with a private sixth `SourceAddress` child
   payload. The legacy public five-child dispatcher and its receipt remain
   source-compatible; the generic root context exposes only a source-submission
   count. Opaque entry targets and submitted draft bytes are never serialized
   or exposed.
4. Keep submitted draft separate from `KucRootEffectRouter` and from the public
   five-child `KucRootEventBatchDispatcher`. The non-wire source-address lease
   owns a dedicated opaque submission port. The sealed root transport dispatches
   the legacy five child classes unchanged, then consumes source submissions at
   that port exactly once. A missing port, a port failure, or a repeated
   transport is typed fail-closed; no default method may discard a submission.
   This keeps existing dispatcher implementations source-compatible while
   preventing KLE from receiving or interpreting the value.
5. Do not implement URL validation, history deduplication, document selection,
   path conversion, or `OpenUrl` here. The fixed host is the only later layer
   permitted to decide those effects.

### Host Facade And KLE Transit Boundary

`EguiTextCommandSurfaceHostProjectionLease` is the sole cross-repository value
that KLE may accept, retain, or transit for this slice. KUC may consume an
optional `SourceAddressProjectionLease` inside that opaque host lease when it
retains or synchronizes its root. KLE must not name, construct, destructure,
or implement `SourceAddressProjectionLease`, `SourceAddressSubmissionPort`,
`SourceAddressSubmission`, `SourceAddressStrip`, or the source-address egui
adapter. The same prohibition applies to KLE tests, Storybook, receipts,
debug output, and manifests.

The KUC facade contract has two distinct obligations. First, a retained root
must preserve its KUC-private submission port across distinct one-shot frame
transports; two physical submissions must be forwarded exactly once each.
Second, no public facade record, debug representation, event context, or
forwarding receipt may reveal either submitted draft. The facade test may
observe received drafts only inside its KUC-private test port. It must drive
real egui input through the retained host root rather than calling core
submission actions directly.

KLE's AST gate rejects all source-address-specific symbols above, URL/path
parser and `OpenUrl` construction in KLE/Storybook, and direct source-address
egui widget construction. The only allowed KUC spelling in KLE for this
feature is the existing opaque `EguiTextCommandSurfaceHostProjectionLease`.
This is a boundary gate, not evidence that a fixed KatanA host can resolve a
submission.

The KUC root gate must exercise pointer, keyboard/IME, and AccessKit input for
history/candidate disclosure, selection, focus/blur, disabled state, and submit.
It must also prove root revision and duplicate/stale lease rejection, exact-one
forwarding, failure consumption, no overlap with the existing child regions,
and no URL/path/draft/opaque-target leakage in Debug, serialized frame data,
fingerprints, AccessKit, or artifacts. Existing roots without the optional
projection remain byte-compatible in their five current child classes.

The source-address adapter must use the same root-owned `PlatformFontCatalog`,
text raster, and artifact compositor as the existing text-command children.
Direct `egui::TextEdit`, `egui::Button`, platform font fallback, or a separate
font catalog is prohibited for this root slot. A standalone native-egui adapter
may be used only as an exploratory contract and cannot be mounted, used in
Storybook evidence, or treated as a v0.1.0 implementation until its input and
label raster path is replaced by the KUC runtime.

## Tab Strip Root Integration Plan

`CloseableTabStrip` and its egui adapter are reusable KUC components, but the
current implementation is mounted only by the separate
`SanitizedDocumentRoot`. That root must not be exposed to KLE or composed next
to `EguiTextCommandSurfaceHostRoot`: doing so would create two retained roots,
two event transports, and a second layout authority. The migration reuses the
generic child adapter and its opaque route table only.

KUC adds an optional non-wire `TabStripProjectionLease` to
`EguiTextCommandSurfaceHostProjectionLease`. The lease consumes generic tab and
group presentation, KUC-private opaque targets, and a private one-shot tab
event port. It is neither part of the JSON presentation token nor readable from
KLE. `EguiTextCommandSurfaceHostRoot` owns the retained strip and renders it as
the first root region, before source navigation and text-command regions. The
root alone calculates bounds, overflow, focus order, drag state, and artifact
composition.

The tab port is distinct from `KucRootEffectRouter` and from the five legacy
public child classes. Selection, close, pin/restore, group, reorder, and
context-menu events resolve their opaque targets inside KUC and are consumed
once by the private port. Public KUC record/context/receipt may expose only a
generic sealed-event count; target bytes, tab IDs, group IDs, labels, colors,
indices, paths, and persisted layout state never cross into KLE. A missing or
rejecting port, duplicate transport, stale lease, disabled target, or invalid
drag state fails closed without replay or local KLE mutation.

Implementation is split in this order.

1. Extract the generic child adapter and route-table behavior from
   `SanitizedDocumentRoot` without importing that root into the public host-root
   path. Add a non-wire lease and retained child state to the existing host-root
   factory/process.
2. Extend root composition, closed frame record, AccessKit, and artifact plan
   so the strip is root-owned and cannot overlap navigation, command, text, or
   overlay regions. Keep the existing no-tab root byte-compatible.
3. Add a sealed private tab-event port to the root transport. KLE continues to
   forward the opaque batch once and does not name tab event types or targets.
4. Add a KUC-owned generic `WorkspaceTabs` scenario to the full Storybook
   catalogue. Its physical pointer, keyboard, drag, and AccessKit stages remain
   opaque to KLE; KLE checks only the existing closed root receipt and artifact
   coherence.
5. Prove every KatanA-required tab leaf separately: select, previous/next,
   horizontal overflow scroll/active reveal, close variants, pin/restore, drag/reorder, group create/add/remove/
   rename/recolor/ungroup/close/collapse, inline rename, context menu, disabled,
   stale, duplicate, and no-leak. A partial select/close demonstration is not
   completion of the tab requirement.

Fixed KatanA persistence and action effects remain a later host-only gate. A
generic tab scenario, Storybook frame, or opaque KUC port is not evidence that
KatanA persisted the requested tab/group change.

### Reconciliation of the Existing KUC Tab Component

The current KUC `CloseableTabStrip` is an implementation substrate, not the
root contract to expose. Its public aliases still name `Workspace*`, carry
serializable string identifiers and colors, and let its event family disclose
tab/group identity, order, and mutation detail. Those properties are useful
inside the existing component but are incompatible with the opaque KLE root
boundary. `SanitizedDocumentRoot` currently solves only a narrow subset by
mapping generated structural IDs to private targets; it renders in a second
root and forwards only selection, close request, and group collapse. It is not
a valid host-root transport and must not be reused as one.

The implementation therefore has two explicitly separated layers:

1. KUC replaces `Workspace*` aliases at the new root boundary with generic
   `TabStripProjection`, `TabStripTabDescriptor`, `TabStripGroupDescriptor`,
   and opaque tab/group capability targets. The existing strip may remain an
   internal renderer while this extraction is in progress, but no new
   host-root API, receipt, record, or KLE import may contain `Workspace`,
   `CloseableTab*`, structural IDs, hexadecimal colors, tab indices, paths, or
   raw group-name strings.
2. `TabStripProjectionLease` is a non-wire, consuming KUC lease. It contains
   the revisioned generic projection and a private one-shot `TabStripEventPort`.
   The port accepts only sealed KUC events after route lookup. It has no read,
   clone, serialization, debug-payload, or replay API. A group-name commit is
   a separate non-Clone/non-Serialize one-shot value that the port consumes
   together with the already-resolved opaque group capability. A palette choice
   is an opaque host-issued swatch target, never a color string outside KUC.

The root maps KUC-only structural widget IDs to opaque targets for the current
revision. It must never derive a target from label, order, tree position, tab
state, or drag geometry. Root replacement discards the old map before accepting
the newer one. Any event from a missing, stale, disabled, duplicate, or
unrecognized mapping is rejected without a root event, host effect, retained
mutation, or KLE-visible difference other than the sealed failure receipt.

### Source-Derived Tab Operation Matrix

The fixed KatanA tab bar is the reference for this matrix. KUC owns the generic
interaction and presentation column; KLE only relays the opaque lease and
closed receipt. The final KatanA action/persistence column is host-only and
remains unproved until physical fixed-host E2E exists.

| Reference source | Required generic KUC capability | Required KUC evidence | Fixed-host evidence still required |
| --- | --- | --- | --- |
| `tab_bar/mod.rs`, `items.rs`, `strip_renderer.rs` | active selection, pinned/grouped/ungrouped ordering, horizontal overflow scroll, active-tab reveal | root-first layout, non-overlap, pointer/keyboard/AX select, clipping/reveal and deterministic redraw | selected document changes through the unchanged handler; strip scroll remains KUC-retained |
| `tab_bar/nav.rs` | previous and next navigation with disabled state | physical pointer, keyboard, AX, disabled no-event | cyclic KatanA document selection and scroll request |
| `tab_bar/tab_context_menu.rs` | close, close others/all/left/right, restore, pin/unpin, create/add/remove group | every menu branch, close lifecycle, disabled/virtual restrictions, sealed exact-once port | every corresponding KatanA document/group mutation or source no-op |
| `tab_bar/group_header.rs`, `group_header_popup.rs` | collapse, context menu, inline rename, opaque swatch choice, ungroup, close group | pointer/keyboard/AX, IME commit/Enter/Escape/outside close/focus return, no raw-name/color leak | KatanA group persistence and visible re-render |
| `tab_bar/tab_item/drag.rs`, `drag.rs`, `tab_ghost.rs` | drag lifecycle, ghost, reorder, group move, cancelled/invalid drop | separate press/move/release frames, current-frame locator, stale/disabled/invalid drop rejection, AX alternative | KatanA reorder/group result and no-op branches |

KatanA currently represents one-level groups, while the generic KUC descriptor
must keep its existing ability to represent nested groups without flattening or
inventing a KatanA hierarchy. The parity test corpus includes the KatanA
one-level cases and separate generic nested-group integrity cases. Neither
corpus is allowed to reinterpret a group relationship in KLE.

### Root Integration Contract

`EguiTextCommandSurfaceHostProjectionLease` gains an optional
`TabStripProjectionLease`, parallel to but independent from source address. The
factory and synchronization path consume it exactly once and attach it to the
same `EguiTextCommandSurfaceHostRoot`; there is no public `show` for a child
strip and no second event transport. The root composition order is tab strip,
source navigation (when present), command chrome/search, text surface, then
overlays. Root record, AccessKit snapshot, artifact plan, and motion catalogue
are extended as root-owned output only. Their public forms expose a sealed tab
event count and hashes, never a child geometry, label, target, state, or
operation kind.

The five existing public child classes remain unchanged. Tab events are not
coerced into text, toolbar, floating, search, or context-menu events. During
`dispatch_once`, the KUC-private tab port consumes its sealed events before the
public receipt is created. The receipt can report only total root event
cardinality and sealed-tab count. A missing port is a fail-closed configuration
error; a rejecting port produces a fail-closed dispatch error; neither case may
fall back to the generic root router or a local retained mutation.

### Proposal and Confirmation Semantics

The existing KUC tab model mutates selected and collapsed state when its local
action is applied. That behavior cannot be mounted directly in the host root:
KatanA remains authoritative for document selection, group collapse, pinning,
close confirmation, and persistence. A local mutation before the host accepts
and republishes the projection would display a state that neither the host nor
the next revision owns.

Every document-affecting interaction therefore follows this transaction:

1. KUC captures physical input and resolves a current-revision structural
   widget ID through its private route table. It constructs one sealed
   `TabStripProposal` carrying only opaque capability targets and, for rename,
   a non-clone one-shot text value.
2. KUC may retain purely transient UI state such as pointer capture, drag
   ghost, hover, focus, popup visibility, and IME preedit. It must not commit
   active-tab, collapsed-group, ordering, pin, membership, color, close, or
   restored-tab presentation state from the interaction itself.
3. The private port consumes the proposal once for that root transport. Port
   acceptance means only that the host received the proposal; it is not an
   acknowledgement of a state change. Rejection, stale correlation, missing
   port, duplicate proposal, disabled capability, invalid drop, or failed host
   effect immediately restores the last accepted projection without emitting a
   public child event.
4. Only a newer host projection lease, with the same root identity and a
   strictly newer revision, confirms a document-affecting visible state. KUC
   then discards the prior route table and derives the new retained renderer
   state from that lease. Equal revision with different projection is a
   conflict; an unchanged or absent follow-up lease never authorizes local
   commit.

The private port is one-shot per sealed proposal/transport, not permanently
one-shot for an entire retained root. It needs a KUC-private proposal nonce or
equivalent correlation binding so two independent current-frame interactions
can each be forwarded once, while a replay of either proposal is rejected. A
single global `consumed: bool` on the root or port handle is invalid because it
would drop later legitimate tab actions.

This contract changes the a2/a3 order: root integration first mounts only a
renderer capable of proposal extraction and rollback; operation-family routing
is added only when every core action can be represented as a proposal without
committing host state. Tests must prove pointer/keyboard/AccessKit select and
collapse leave the accepted projection unchanged before the newer lease,
rejection returns to the accepted projection in the same frame, and a newer
lease alone changes the final root RGBA/AccessKit/record. The same rules apply
to drag/reorder, close/pin/restore, group membership, rename, swatch, and
horizontal overflow/reveal scroll.

### Implementation Checkpoints

1. Extract a generic root-boundary model from the existing `Workspace*` aliases
   without changing KatanA or exposing those aliases through the new API. Add
   compile-time and source-scanning tests proving that the root/KLE public
   boundary contains none of the forbidden names or raw payload forms.
2. Add the non-wire lease, retained private route table, and root-first layout.
   Keep the no-tab root byte-compatible and reject same-revision projection
   drift, stale lease, duplicate lease, and changed root identity.
3. Add the sealed event port and all operation families before declaring any
   tab leaf complete. The core can internally evolve from its current event
   model, but no raw core event may enter a root transport.
4. Add `WorkspaceTabs` as a KUC-owned full-surface scenario. Its fixture and
   all physical coordinates remain KUC private; KLE Storybook accepts only the
   generic scenario ID, opaque continuation, and closed receipt.
5. Run the matrix above across RawInput, keyboard, drag, AccessKit, artifact,
   no-leak, stale/duplicate/disabled, and fixed-host categories. A row is not
   complete until its fixed-host column has real KatanA evidence.

### Tab Strip Retained-State Design

The current `EguiTextCommandSurfaceRootEventBatch` is intentionally not the
TabStrip transport. It is a root-frame one-shot for the five existing child
classes, whereas a tab interaction is proposal-specific and may occur more
than once during the lifetime of one retained root. Extending that batch would
either make tab proposals observable to KLE or incorrectly use one permanent
`consumed` flag for the whole root.

The KUC implementation therefore uses the following private ownership model:

```text
EguiTextCommandSurfaceHostProjectionLease
  token + generic root effect router + source-address lease
  optional non-wire TabStripProjectionLease

HostRootProcess
  EguiTextCommandSurfaceRoot
    optional TabStripRetainedState
      last accepted TabStripProjection
      current projection revision and correlation
      KUC-private route table
      transient popup, focus, IME preedit, pointer-capture, and drag-ghost state
      proposal-specific consumed correlation set
      proposal port
```

`TabStripProjectionLease` is consumed in the same factory and synchronization
transactions as the root token but is not encoded in it. A newer root lease
must replace the accepted TabStrip projection atomically; a same-revision but
different projection is a revision conflict, and a stale or different-identity
lease is rejected before the renderer or route table changes. An omitted tab
lease represents no mounted strip and preserves no hidden tab state.

The renderer is an extracted KUC-only adapter, not the existing mutating
`SanitizedTabProjectionAdapter`. It creates KUC-private structural widget IDs
only for the current accepted projection, resolves an input to an opaque
capability target, and creates a sealed `TabStripProposal`. It never calls the
legacy `WorkspaceTabBar` state transition to confirm selection, collapse,
pinning, order, membership, color, close, or restore. The retained state may
change only the transient state listed above. Rejected, stale, duplicate,
disabled, unknown, or invalid-drop proposals discard that transient state and
redraw `last accepted TabStripProjection`.

The proposal port is separate from the existing root event batch. It is
one-shot per proposal correlation, consumes a sealed proposal without a
readback API, and yields a sealed receipt only. A host port acceptance means
delivery, not state confirmation. Only the next strictly newer accepted lease
may change the displayed document-affecting state. The public root frame,
record, Debug output, artifacts, and KLE transit contain at most aggregate
sealed-proposal facts; they never contain operation kind, target, label,
structural ID, color, path, order, geometry, or pending state.

The sealed proposal operation set is source-derived and has no ambiguous
toggle or index form: select previous/next/specific; request and
confirm close separately; close others/all/left/right; restore; explicit
`SetPinned`; create, move into, and remove from groups; set collapse; rename;
recolor by opaque swatch; ungroup; close group; reorder tab/group through an
opaque before/after/group/end placement; and start/finish/cancel/hover drag.
The renderer may construct only operations whose capability is present in the
accepted projection. It must reject absent capabilities rather than infer an
operation from position, label, color, or local state. `AddTab` is not included
because the fixed KatanA tab bar does not expose a document-creation action;
document creation remains outside this editor region.

Root composition gives the tab strip a dedicated first rectangle before source
address, command chrome, text, search, and overlays. The adapter owns both the
render and artifact plan for that rectangle. Tests must prove exact
non-overlap, AccessKit parent-child order, no-tab byte compatibility, and that
the same physical interaction changes no accepted presentation until a newer
lease arrives.

### Current Bounded Implementation Status

As of 2026-08-23, KUC has consumed the optional non-wire
`TabStripProjectionLease` through the existing host lease, attached it to the
same retained `EguiTextCommandSurfaceHostRoot`, allocated its first root
rectangle, and included its KUC-owned raster paint plan in the final artifact
composition. Physical pointer selection and group-collapse labels produce a
single sealed proposal through the private proposal port; the renderer does
not mutate active or collapsed presentation locally. A root test proves one
physical select proposal, unchanged accepted artifact pixels before a newer
projection, and a nonempty same-root composite.

The same root now renders generic SVG/currentColor previous/next controls when
the host supplies opaque localized navigation presentation. A physical-input
test proves enabled previous forwards exactly one `SelectPrevious` proposal and
disabled next forwards none. These controls do not locally accept document
presentation.

KUC also renders the host-projected trailing affordance for closeable tabs. A
physical-input test proves the normal-tab affordance yields `RequestClose` and
the pinned-tab affordance yields `SetPinned(false)`; neither changes displayed
tabs before a newer host lease. Dirty confirmation, close scopes, restore,
context menus, and host effects are intentionally not implied by that test.

The accepted projection is compiled into a KUC-private route table before the
renderer sees physical input. The table owns the copied opaque capability
target and correlation for select, collapse, previous/next, close, and unpin;
the renderer registers each interactable `egui::Response` ID, actual bounds,
localized accessibility label, and disabled state for the current frame and
resolves activation through that response ID rather than a descriptor path. It
cannot construct a proposal from a descriptor. Pointer, Enter/Space on an
egui-focused route, and an AccessKit Click use the same entry; a disabled entry
is a fail-closed no-op and a pointer/key combination in one frame emits one
proposal. The generic KUC button evidence helper publishes the actual bounds
and role, while the closed root hashes all current-frame AccessKit evidence
without exposing target bytes. Physical group-header press/release now proves
one collapse proposal and an unchanged accepted artifact before host republish.
A strictly newer plain root token clears a prior tab-strip lease and reports
that visible-state change to the host facade. Focus traversal, stale response
invalidation after root replacement, route visibility facts, and AccessKit
parent-child ordering remain unimplemented, so this is not the complete
route-table gate.

KUC now renders tab/group items inside one horizontal, scrollbar-hidden
`ScrollArea` and fixes previous/next controls to the trailing root area. The
generic `TabStripScrollPresentation::request_active_reveal` flag is copied to
private retained state and centers the active tab without emitting a proposal.
Constrained-width physical frame tests prove both active reveal and horizontal
wheel input change only the clipped KUC artifact. Keyboard scrolling,
resize/replacement focus safety, and explicit route visibility records remain
unimplemented.

This is only the initial root-mount slice. It does **not** implement or prove
the remaining KatanA tab operations, route-table rejection/rollback semantics,
revision conflict behavior for tab leases, no-tab byte compatibility, bounds
non-overlap, focus traversal, overflow/menu/inline rename/palette/drag UI,
group-name IME, video evidence, three-platform emoji evidence, KLE opaque
consumption, or any fixed-KatanA host effect. The KUC generic Enter/Space and
AccessKit Click coverage is not a KatanA keyboard-parity claim. None of those
items may be inferred from this mount result.

### Full TabStrip Descriptor And Retained-State Plan

Before further renderer work, KUC must extend the generic projection with an
opaque, localized `TabStripControlPresentation`. It supplies presentation-only
text for previous/next/restore, tab context-menu leaves, group popup
leaves, the add-to-group submenu heading, rename field accessibility text, and
all tooltips. It carries no KatanA action, document identity, group name as a
readback API, color value, index, path, placement coordinate, or serialized
state. Capability targets remain the sole effect-bearing values.

`TabStripRetainedState` owns these transient states only:

| Transient state | Physical entry | Required exit | Must not mutate |
| --- | --- | --- | --- |
| focused control and hover | pointer, keyboard, AccessKit focus | focus move, Escape, projection replacement | active tab, pin, group collapse, order |
| tab/group context popup | secondary click or keyboard equivalent | outside click, Escape, operation delivery/rejection | host-confirmed close, membership, color |
| rename preedit | group popup rename action | Enter delivery, Escape/outside cancel, rejection/replacement | group name in accepted projection |
| palette popup | group popup recolor action | swatch delivery, Escape/outside, rejection/replacement | selected swatch/color in accepted projection |
| drag capture/ghost/drop indicator | primary press then threshold move | valid delivery, invalid release, Escape, replacement | order, membership, active tab |
| overflow/reveal scroll | viewport width and accepted active projection | resize, active reveal, replacement, or KUC-owned pointer/keyboard scroll | tab/group order or host state |

Every document-affecting control creates exactly one sealed proposal after a
current-revision private route-table lookup. The port result is delivery only.
On missing/rejecting/duplicate/stale/disabled/invalid input, KUC discards the
transient state and redraws the last accepted projection. Only a strictly newer
same-root host lease changes accepted presentation.

### Horizontal Overflow And Active-Reveal Contract

The fixed KatanA tab bar has no overflow menu. Its overflow behavior is a
horizontal, scrollbar-hidden scroll region, and `TabItem` requests
`scroll_to_me(Center)` only when `TabBar` has an active-tab scroll request.
KUC therefore models this as generic retained presentation, not as
`OpenOverflow`, a menu, or a host effect:

1. `TabStripScrollPresentation` carries only the boolean
   `request_active_reveal`; it contains no document identity, width, index,
   coordinate, or scroll offset and is not readable by KLE.
2. A consumed `TabStripProjectionLease` copies that flag into private
   `active_reveal_pending` state. When the current accepted active tab is laid
   out within the KUC horizontal `ScrollArea`, KUC requests centered reveal and
   clears the transient pending flag. It does not change active selection or
   emit a proposal.
3. Pointer wheel/trackpad scroll, resize clipping, and horizontal offset remain
   local to that scroll area. KLE receives no offset, widget ID, bounds, or
   event. A newer projection may request reveal again; a plain replacement
   removes the entire retained strip as already specified.
4. Tests must use a constrained root width and prove: overflow clips rather
   than overlaps the navigation controls; reveal changes the visible artifact
   without forwarding a host proposal; manual horizontal scroll changes only
   KUC retained artifact state; resize/replacement keeps active focus and route
   targets valid; pointer, keyboard, and AccessKit paths share current route
   entries where an interaction exists.

### Context Menu, Group Popup, And Palette Boundary

The fixed source uses a tab secondary-click menu for close scopes, pin/unpin,
restore, create/add/remove group, and a non-demo group-header secondary-click
popup for rename, palette, ungroup, and close group
(`tab_context_menu.rs:18-147`, `group_header.rs:22-119`, and
`group_header_popup.rs:23-110`). The KUC design must preserve each visible
branch while excluding every KatanA identity or persistence detail:

1. `TabStripContextMenuPresentation` and `TabStripGroupPopupPresentation` are
   generic KUC-owned presentation-only values. Every item supplies localized
   visible/tooltip/accessibility text and an explicit enabled/visible state;
   no renderer infers a menu branch from tab order, filename, virtual path,
   group membership, or recently-closed state.
2. A current projection compiles each visible item into the private route table.
   Tab routes retain only copied opaque tab targets, group routes only copied
   opaque group targets, and add-to-group entries retain both. Choosing an item
   creates exactly one existing sealed proposal. Menu open/close/focus/hover
   are KUC transient state only and never confirm a host mutation.
3. Group-name editing initializes a KUC-local preedit from the projected group
   presentation. The input value leaves the KUC root only as a non-Clone,
   non-Debug, non-Serialize one-shot submission consumed by the proposal port;
   KLE must not receive a `String`, command, group identifier, cursor, or IME
   state. Enter, source-defined outside close, loss of focus, stale/rejected
   delivery, and a newer projection each have independent test leaves. Escape
   remains a KUC extension unless a fixed-source route is found; it must not be
   reported as a KatanA parity leaf.
4. `TabStripSwatchDescriptor` requires generic host-projected display color and
   accessibility presentation while the host-issued swatch target stays opaque;
   there is no KUC fallback palette color.
   KLE never receives hex, RGB, palette index, selected color, or palette
   geometry. A swatch click is one `RecolorGroup` proposal; selection display
   remains pending only until a newer host projection.
5. The fixed source omits tab and group menus for virtual/demo restrictions.
   KUC receives those conditions exclusively as host-projected item visibility
   and capability state. Disabled, missing, stale, duplicate, rejected, and
   outside-click paths close the local overlay without replay or local document
   mutation.

### Keyboard And AccessKit Boundary

The fixed KatanA `views/top_bar/tab_bar/**` source has no tab-strip key handler;
its only local keyboard path is the group-name field's Enter behavior in
`group_header_popup.rs:34-51`. Consequently, focused tab activation and tab
arrow/Home/End navigation are KUC accessibility extensions, not KatanA-parity
claims, until a source-derived KatanA route exists. They remain release-required
generic behavior but must be recorded under that separate origin.

`TabStripRouteTable` therefore gains a second, current-frame resolved entry
only inside KUC: structural route ID, `egui::Response` ID, actual bounds,
enabled/visible/focusable facts, and host-projected localized accessibility
presentation. The same entry is the sole input to pointer activation, focused
keyboard activation, AccessKit click action, AccessKit node/evidence, and the
opaque proposal builder. KLE may not receive route IDs, `egui` IDs, bounds,
labels, focus, accessibility node IDs, or keyboard state.

Pointer activation requests focus. Enter/Space can activate only a focused,
visible, enabled route, while pointer plus key activation in one frame must
remain one proposal. Arrow/Home/End navigation is KUC-local focus movement and
must skip disabled/hidden routes; it cannot select or reorder a host tab until
an explicit activation produces a proposal. AccessKit actions bind to the same
response ID and current root revision/correlation. Missing, stale, ambiguous,
disabled, duplicate, or nonvisible action requests fail closed without a
proposal. KUC must verify dependency-supported role semantics before declaring
a `TabList`/`Tab` node; no unverified role is assumed.

The implementation and test order is fixed:

1. previous/next controls, disabled state, tooltip/AccessKit metadata, horizontal
   clipping and active reveal;
2. tab close/pin affordances and context-menu close/restore/pin branches;
3. group menu, add/remove/create/collapse/ungroup/close flows;
4. rename IME/Enter/Escape and opaque swatch palette;
5. drag reorder/group move/cancel/invalid drop;
6. every physical pointer, keyboard and AccessKit path, plus artifact, no-leak,
   stale/rejection and fixed-host effect evidence.

No KLE or Storybook implementation can create an intermediate visual control,
derive a route target, hold a group label, or simulate the host confirmation
for any of these stages.

### Tab And Group Overlay Extraction Decision

The existing KUC `ContextMenuPresentation` cannot be mounted for a tab or
group menu as-is: its serializable string item IDs and public item frame record
are valid for the existing generic text context-menu contract, but would make a
TabStrip route, structural order, or action mapping observable outside the
non-wire lease. Reusing it directly would violate the KLE boundary even if the
visible pixels looked correct.

KUC must instead extract a generic, crate-private retained overlay surface that
reuses the existing menu's layout, text-raster, icon, focus-return, Escape,
outside-click, pointer, keyboard, and AccessKit mechanisms. Its input is a
KUC-private `TabStripOverlayRouteTable`, populated while consuming the
`TabStripProjectionLease`; it has no public/string command ID, serialization,
root-frame item record, or KLE callback. The overlay entry carries only:

1. localized visible and accessibility presentation;
2. explicit visible/enabled/check/submenu facts projected by the host;
3. an opaque tab/group/swatch target or a targetless generic root operation;
4. a KUC-private operation family selected before render; and
5. an ephemeral response ID, bounds, focusability, and AccessKit node for the
   current frame.

The renderer never derives an operation from label text, position, capability
boolean, color, group nesting, or an item index. Host presentation determines
whether virtual/demo restrictions hide or disable a menu branch. Selecting an
enabled item closes the retained overlay and forwards exactly one opaque
proposal; disabled, stale, unknown, duplicate, or rejected input closes or
redraws fail-closed without host mutation. A newer projection is the sole
confirmation of a visible tab/group change.

The private menu tree has three explicit presentation node kinds: action,
submenu, and separator. A separator is not synthesized from adjacent labels or
operation families. A submenu has no implicit action; it opens only its
projected child tree. The renderer records and rasterizes every visible action
through the same current-frame opaque route lookup, physical bounds, and
AccessKit path. This is required to preserve the fixed source's close-menu and
group-popup ordering without leaking either ordering or route identities across
the KLE boundary.

Inline group rename is a separate overlay sub-state. Its draft is a
non-Clone/non-Debug/non-Serialize, one-shot KUC-private submission associated
with an opaque group route; KLE cannot observe draft bytes, IME preedit, or
Enter/Escape/outside-close state. Swatches use a KUC-generic display color plus
localized accessibility presentation and an opaque swatch target. The color is
never inferred from a host action, and neither KLE nor a public frame receives
RGB/hex/palette index. This boundary is required before implementing any
KatanA menu branch.

The non-wire projection shape is fixed before implementation:

1. `TabStripTabDescriptor` optionally owns a `TabStripContextMenuPresentation`.
   It contains an ordered tree of visible entries, each with presentation,
   explicit enabled/check state, and a KUC-private generic tab-strip operation.
   A submenu is structural only; entry labels or paths are never operation keys.
2. `TabStripGroupDescriptor` optionally owns a `TabStripGroupPopupPresentation`.
   It contains explicit open/rename presentation, action entries, and swatches.
   Demo/virtual/pinned restrictions are represented by absent entries or a
   disabled entry, not a KUC string or group-name comparison.
3. `TabStripMenuOperation` holds only the opaque target(s) required by its
   generic tab/group operation. `TabStripRouteTable` copies this operation only
   after lease consumption, then binds it to a current-frame response ID.

#### Fixed-Source Overlay Branch Matrix (2026-08-23)

The following conditions are requirements for the host-projected presentation,
not facts KUC may infer. They are independently tested in the fixed-source
catalogue before a host bridge may claim parity:

| Fixed KatanA branch | Required projected order/condition | KUC-only responsibility |
| --- | --- | --- |
| ordinary tab context menu | `Close`, `Close Others`, `Close All`, `Close Tabs to Right`, `Close Tabs to Left`, without a separator | secondary-click opening, ordered render, exact-once opaque route delivery and close |
| virtual tab | the five close entries only; no pin/unpin, group branch, restore branch, or their separators | render exactly the host-projected entries; do not inspect a path or virtual flag |
| real tab | separator then `Pin`/`Unpin`; group branch only when not pinned; restore after the group branch only when projected | route each projected branch without deriving availability |
| ungrouped tab, no non-demo destination | `Create New Group` as a direct item | action delivery only; KUC does not select a color/name or group member |
| ungrouped tab with destinations | `Add to Group` submenu: `Create New Group`, then separator only when candidates follow, then candidate order as projected | submenu open/focus/route lifecycle only |
| grouped tab | `Remove from Group`, then `Add to Group` submenu | no membership inference; exact visible tree comes from projection |
| group header popup | secondary click only, and only when a popup presentation exists; primary header click remains collapse | toggle/open anchor, outside-primary dismissal, overlay layout and focus handling |
| group popup content | rename field, 4px gap, seven projected swatches, 4px gap, separator, `Ungroup`, `Close Group` | retained draft/palette interaction and opaque output only |
| group rename | issue rename only when draft differs; fixed source closes only when Enter causes lost focus | source behavior is the parity case; Escape is separately marked KUC extension |
| group recolor | one recolor route; popup remains open | retain popup and wait for newer host projection to confirm selected swatch |
| rename and recolor in one frame | fixed source gives the later recolor action precedence over rename | encode a specific same-frame regression test; do not forward two actions |

Source locations: fixed revision
`tab_context_menu.rs:19-161`, `tab_item/mod.rs:96-110`,
`group_header.rs:28-35,106-141`, and `group_header_popup.rs:20-111`.
4. A renderer may consume an operation only from that current-frame route
   table. Overlay open/close/focus is retained KUC state; host mutation is a
   sealed proposal and cannot change presentation until a newer lease arrives.

### Tab Strip Drag Decision (2026-08-26)

Tab/group reorder is a KUC retained interaction, not a KLE event family. The
implementation keeps a crate-private drag state containing only copied opaque
source target, accepted projection revision/correlation, native pointer
capture, current private placement, and ghost/indicator paint facts. It does
not retain a tab index, document name/path, group name, pointer coordinate, or
placement outside the KUC root.

1. A host-projected `draggable` capability enables `egui::Sense::click_and_drag`
   on the corresponding current-frame response. The native drag threshold is
   authoritative; KUC does not introduce a second arbitrary threshold.
2. Before/after candidates come only from visible current-frame tab response
   bounds and copied opaque targets. A group-header candidate is `InGroup` only
   when that group is currently projected as a valid destination. `NewGroup`
   is impossible unless a host-projected generic destination exists; it is
   never inferred from free strip space or a label.
3. On threshold crossing KUC emits one sealed `StartDrag` proposal and owns the
   ghost, drop indicator, pointer capture, and optional horizontal autoscroll.
   Hover updates are local only. On a valid release it emits one sealed
   `FinishDrag { committed: true, destination }`; it must not additionally emit
   `ReorderTab` or mutate presentation. On Escape, pointer cancellation,
   invalid drop, source disappearance, stale revision/correlation, or lease
   replacement it emits one `CancelDrag` after a started drag and restores the
   accepted projection.
4. The KUC root owns all pointer/keyboard/AccessKit drag state. KLE receives
   only its existing closed root receipt and forwards the sealed opaque batch
   once; it cannot inspect a source, destination, geometry, operation family,
   ghost, or drag phase. Fixed-host evidence remains separately required for
   KatanA persistence.
5. The drag test matrix is mandatory: below-threshold no-op; one start;
   before/after/group/end placement; self/disabled/virtual/pinned rejection;
   repeated move no duplicate; release/cancel/Escape/lease replacement/source
   disappearance; autoscroll; pointer, keyboard, and AccessKit paths; ghost
   and indicator artifact order/pixels; stale/duplicate proposal failure; and
   no KLE/public-record leakage.

## Required Automated Gates

- KUC unit and integration tests prove root/child revision agreement,
  retained-state survival, exact-one event consumption, stale/duplicate failure,
  disabled no-mutation, bounds non-overlap, overlay z-order, focus return, and
  AccessKit parent-child/action consistency.
- The KUC artifact gate proves nonempty same-root RGBA, deterministic redraw,
  PNG provenance, GIF/MP4 decode frame count/dimensions/hash correspondence,
  and Japanese/IME/exact `⭐️` U+2B50 U+FE0F versus `☆` crop evidence.
- KLE AST rules reject child rendering, local state, coordinate reconstruction,
  font/emoji fallback, KatanA-specific mappings, and every
  source-address-specific KUC type or submission port. They also reject KLE
  URL/path parsing and `OpenUrl` construction; the opaque host projection
  lease remains the only transit surface.
- Fixed-host E2E proves only actual KatanA effects. Missing public bootstrap or
  editor-target input remains an explicit release blocker, never a fallback
  path.

## Release Condition

The full-editor root is not complete when individual KUC components render or
when Storybook videos exist. It is complete only after every region is mounted
in the same retained root, all generic transitions pass the KUC gates, every
source-derived host effect has fixed-KatanA evidence, and macOS/Windows/Linux
color-emoji profile artifacts are joined to the release manifest.
