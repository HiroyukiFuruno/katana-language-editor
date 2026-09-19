# v0.1.0 Full-Spec Editor Surface Design

## Status

This document is the design gate for the KatanA-compatible editor surface. No
KLE-specific authoring toolbar, search panel, icon rasterizer, or Storybook
fallback renderer is an acceptable final implementation. The current
`authoring_helper.rs` is a temporary action-wiring probe and must be removed
before the feature is considered implemented.

### Superseding Authoring Boundary (2026-08-14)

Any later reference in this historical migration design to
`EditorAuthoringCommand`, `EditorCodeBlockKind`, `EditorAuthoringMenuState`,
`EditorAuthoringMenuLifecycle`, `RunAuthoringCommand { command: String }`, a
KLE code-kind map, or KLE cursor/popup state describes current residue only. It
is not an admissible target API. The required contract is:

```text
KatanA host projection (opaque authoring target/revision/correlation)
  -> KLE mechanical forwarding
  -> KUC generic CommandChrome retained interaction
  -> same opaque target/revision/correlation
  -> unchanged KatanA host lookup -> AppAction::AuthorMarkdown
```

KUC knows no Markdown operation or code kind. KLE neither stores nor derives a
command semantic. The generated KatanA source closure enumerates every
`MarkdownAuthoringOp`, `CodeBlockKind`, physical origin, host handler, and
individual leaf proof. This section overrides all older wording in this file.

### Superseding Root, Preview Rail, And Outline Boundary (2026-08-14)

All generic editor-frame interaction is internal KUC composition. This includes
the text surface, command chrome, search/replace strip, context menu, document
tabs, breadcrumb navigation, source address, workspace/split/preview viewport,
status/diagnostics, preview side rail, and hierarchical outline. KLE has only
host-projected opaque descriptors, a closed host request surface, and one-time
event forwarding. It has no render tree, font catalog, text/query/content
state, panel state, path/URL, export format, outline index/level/line/anchor,
hover clock, geometry, TOC state, or scroll calculation.

`PreviewSideRail` and `OutlineNavigator` must extend/combine existing generic
KUC `CollapsiblePanel`, `HoverCard`/`Popover`, and `TreeView`/disclosure
substrate. KUC retains rail/overlay/accordion/focus/tooltip/empty-state
behavior. KatanA alone derives the semantic active outline target and resolves
an opaque selection to its existing action/UI relay. KLE must not construct a
TOC enum, source priority, export command, `AppAction`, or direct host-state
fixture. This section overrides any older KLE/KDV/egui rendering or typed
Markdown/search action language in this historical design.

### Superseding Neutral Relay Terminology (2026-08-14)

Any later reference in this document to KLE “mapping”, a “typed KLE action”,
an editor/domain DTO, or `EditorEvent` is valid only when it denotes a closed
neutral envelope of opaque host target, descriptor revision, correlation and
source. It does not carry or construct a KatanA action, document identity,
path, range, line, URL, Markdown operation, diagnostic payload, TOC semantic,
or source/browser result. KLE forwards that envelope once and the unchanged
host resolves it. This rule supersedes all older action-mapping wording below.

### Historical Migration Text Handling

This file contains earlier migration investigation written before the opaque
root boundary was established. Every later sentence that gives KLE a
`TextRange`/`TextOffset`, byte/character conversion, document identity,
selection/caret, query/match/result, clipboard/history backend, command
semantic, anchor/viewport/scroll value, source path/URL, rendered child, or
KUC component lifecycle is **rejected historical residue**, not implementation
guidance. Spark must implement only the superseding sections above together
with `docs/v0-1-0-kuc-text-command-root-design.md` and
`docs/v0-1-0-neutral-host-request-design.md`; AST checks must reject the
listed KLE responsibilities. The rejected text remains temporarily only to
trace migration residue and must be removed in the KUC-root cutover batch.

## Source Of Truth

- KatanA authoring UI (reference-only checkout in this scope):
  `../katana/crates/katana-ui/src/views/panels/editor/toolbar.rs`,
  `toolbar_popup.rs`, and `code_block_menu.rs`.
- KatanA editor input, gutter, selection, diagnostics, and highlights:
  `katana-ui/src/views/panels/editor/text_edit.rs` and its collaborators.
- KatanA document-search behavior: `katana-ui/src/views/top_bar/search.rs`,
  `app/doc_search.rs`, and editor/preview highlight integration.
- KUC reusable contracts: `molecule::toolbar`,
  `molecule::structured::search_control_strip`, `UiIconProps`, and placement
  contracts.

The audit must keep a runnable test command next to every source row. Reading
source code, a screenshot, or a Storybook fixture alone is not evidence of
parity.

## Required Ownership

| Concern | Owner | KLE responsibility | Explicitly prohibited |
| --- | --- | --- | --- |
| Command toolbar groups, enabled state, focus lifecycle, keyboard navigation, overflow, typed command events | KUC `CommandChrome` component that composes existing Toolbar contracts | one-time forwarding of a host-issued opaque target/revision/correlation only | KLE-owned toolbar state machine, button inventory, command semantic or event mapper |
| Floating anchor and viewport clamping | KUC placement/component contract | none; the KUC root consumes its own frame facts and host capability | KLE anchor/viewport forwarding, popup-coordinate or panel-size algorithm |
| Search/replace input state, option toggles, result position, replace events | KUC `CommandChromeSearchStrip` that composes existing `SearchControlStrip` | one-time forwarding of a non-persistent text transport or opaque target only | KLE `SearchQuery`/result/capability state, form state or string-derived actions |
| SVG icon props, SVG rasterization, pixel cache, paint policy | KUC | none; the opaque KUC root owns adapter paint submission | KLE/KDV SVG parsing, `resvg`, `tiny-skia`, texture upload, or duplicate icon cache |
| Text/emoji raster, grapheme bounds, caret/hit-test, selection, IME, generic line gutter, range annotation, accessibility tree | KUC `TextSurface` + shared adapter, composed from platform text-raster / TextArea / selection / scroll substrate | invoke the opaque root once and forward its one returned closed transport | editor-domain DTO/range conversion, egui font fallback, KLE/KDV font lookup, local texture cache, local hit-test, local gutter renderer, local text-surface state machine |
| Markdown transformation, image file ingest, save/format/file IO | unchanged KatanA host | forward one opaque host request only | KLE dependency on KatanA/Markdown/file system or `EditorActionRequest` semantic construction |
| KatanA icon assets | Host-provided opaque presentation consumed by KUC | none | KatanA icon enum/assets in KUC or KLE |
| Storybook | KUC opaque root with KLE's transit-only binding | run generated same-root scenarios and emit review artifacts from the final KUC frame | fixture glyph renderer, label parsing, fallback pixels, a KLE render tree, or evidence substitution |

`katana-language-editor` remains neutral and MUST NOT expose KUC or egui types.
KUC types may appear only in the egui adapter and host/Storybook configuration
boundary.

## Migration Readiness Gate (2026-07-14)

KLE の現実装と local KUC runtime を source level で突合した。KLE の
`PlatformTextSurface`、`LineGutterModel`、`authoring_helper` は generic renderer
を重複しており、このままの置換は認めない。KLE の次の実装は、下表の KUC
contract が実装・real-egui test・guard まで閉じてからに限定する。

| Generic concern | Required KUC contract | KLE may do | KLE must not do | Gate |
| --- | --- | --- | --- | --- |
| editor text viewport | KUC opaque root `TextSurface` surface/viewport/content bounds、one scroll state、clipped paint/hit/IME/AccessKit record | borrow the host projection for the single `show` call and forward the one returned neutral transport | text value/selection/caret/range conversion、`ScrollArea`、scroll coordinate、hit test、caret/IME rectangle | KUC actual wheel/pointer/IME frame test |
| text/gutter/annotation | KUC platform text raster span、generic gutter visual role、annotation paint/layer record | forward host-issued opaque presentation/target descriptors without inspection | syntax/diagnostic/search conversion, line-number renderer、row color rule、range geometry、text texture | KUC glyph/row/annotation pixel and AccessKit test |
| clipboard/history/context | KUC payload-minimized one-time transport | forward the closed transport once to the unchanged host | `ClipboardBackend`、undo/action control、clipboard/history store、context coordinate calculation | KUC request test + class-correct host effect test |
| authoring toolbar/dropdown | KUC `FloatingCommandToolbar`/placement/dropdown/tooltip/typed event | forward one host-issued opaque target/revision/correlation | host action ID、selection/read-only derivation、`EditorActionRequest` semantic mapping、`egui::Area`、button inventory、popup placement、icon fallback | KUC actual floating/keyboard/outside-click test |
| find/replace | KUC `CommandChromeSearchStrip` + shared single-line `TextSurface` | forward one non-persistent query/replacement transport or opaque target | `SearchQuery`/result count/capability state, local form state、`TextEdit`、label-derived action | KUC IME/key/disabled/focus test + class-correct host effect test |

### Controlled-consumer prerequisites (2026-08-13)

`TextSurface` and `CommandChrome` have renderer and interaction contracts, but
their current construction-oriented APIs are not sufficient for a real KLE
consumer. Reconstructing a KUC component on every editor update would discard
KUC-owned focus, drag, IME, dropdown, scroll, and texture state. Conversely,
letting KLE preserve that state or calculate the missing geometry would recreate
the prohibited local renderer. The following additive KUC contracts are a hard
precondition for the KLE migration batch.

| Required KUC capability | Consumer input | State that remains KUC-owned | Prohibited KLE substitute | Required automated proof |
| --- | --- | --- | --- | --- |
| controlled TextSurface presentation synchronization | a host-projected, single-use content/presentation handle is passed directly into the KUC root during `show`; KLE cannot retain or inspect selection/caret/range state | focus, drag, scroll, IME/preedit composition, raster/texture cache, layout and hit test | recreating `TextSurface`, private selection action dispatch, local text/glyph layout, byte/character conversion | real-egui external value/selection synchronization during IME, Japanese/`⭐️`/ZWJ, pointer/scroll/AccessKit preservation |
| automatic numbered gutter with sparse presentation overrides | generic enabled mode and opaque marker/target/presentation descriptors only | line enumeration, display labels, row bounds, gutter raster, pointer/accessibility hit targets | `LineGutterModel`, KLE line/range/priority/anchor conversion, row bounds/number paint | multi-line edit/scroll shows correctly aligned numbers and markers in one KUC frame record and AccessKit tree |
| controlled CommandChrome and search-strip synchronization | opaque action/group/item configuration, localized presentation data, enablement and visibility; query/replacement cross only as non-persistent one-time transports | focus, dropdown/open state, tooltip lifecycle, input identity, platform raster and all generic form state | KLE button inventory, label switch, form store, `egui::TextEdit`, synthetic core `apply_action`, query/count/capability state | actual `RawInput` updates controlled state without focus loss; disabled controls emit no action; Japanese/`⭐️` query and replace retain color-texture evidence |
| self-measured floating placement | KUC uses its own current-frame facts and host-projected visibility capability; KLE forwards neither anchor nor viewport data | panel measurement, viewport clamping, placement, outside/Escape dismissal, focus return | KLE panel dimensions, coordinate arithmetic, selection anchor/viewport forwarding, popup placement | actual selection anchor with resize/scroll/keyboard/outside-click verifies one KUC-owned bounds record and focus-return event |
| data-driven command presentation | host-provided opaque command/group/item ids, order, localized label/tooltip/accessibility data, and adapter-bound icon props | command control inventory rendering, icon/SVG raster, disabled/focus/dropdown behavior | `command_label` id switch, fixed authoring label fields as rendering inventory, local icon fallback | all KatanA command groups and 17 code kinds are injected without a KLE id-to-label branch; unknown injected id remains renderable/accessibly named |

### TextSurface gutter and scroll gap (2026-08-13)

KatanA reference `row_diagnostics.rs` renders one host-provided LightBulb SVG
action icon on the diagnostic start line, chooses Error > Warning > Info, and
opens the diagnostic action popup from that icon. Its `logic_scroll.rs` resolves
requested logical lines to the rendered text position without a host-side pixel
calculation. The current KUC automatic gutter emits labels, bounds and opaque
row IDs, but accepts overrides only by a previously issued row ID and paints no
marker icon. Direct KLE migration would therefore either reintroduce a local
line enumerator/icon renderer or drop the KatanA behavior; neither is allowed.

Before the KLE widget migration, KUC must add these generic contracts without
KatanA/KLE types:

| Capability | KLE input | KUC responsibility | Required proof |
| --- | --- | --- | --- |
| range-anchored gutter override | opaque marker target/revision and visual/accessibility data; source range/anchor/priority selection is resolved by the host before projection | resolve the KUC text-layout row, select the highest-priority stable override, preserve automatic labels/bounds/row IDs | multi-line Japanese/`⭐️` source case selects only the host-projected start row; priority and update stability are tested in KUC |
| gutter marker raster and action hit target | host-provided `UiIconProps` and semantic paint role | SVG raster cache, marker size/position, clipping, AccessKit node and `GutterMarkerActivated` event | real-egui pointer and AccessKit test proves the icon texture is painted and callback target is KUC-owned |
| controlled logical-row scroll request | opaque host-issued scroll target/revision/correlation | resolve KUC logical presentation/scroll once, report acknowledgement, then allow user scrolling | real-egui request/ack/user-wheel sequence proves no KLE pixel, line, offset, anchor, or line-to-rect calculation |

KLE does not access `TextRange`, byte offsets, source anchors, diagnostic
severity or marker priority in this path. The host projects opaque descriptors,
and KUC owns generic role, layout and hit behavior. KLE must not enumerate
lines, calculate icon/row geometry, parse SVG, cache SVG pixels, or resolve a
scroll rectangle. The KLE widget must wait for these KUC contracts and their
real-egui evidence before replacing the old surface; otherwise the migration
would be visually and behaviorally partial.

Anchor and viewport facts never cross into KLE. KUC alone computes panel size
and final placement. Presentation labels, icons, tooltips, and accessible names
remain host-provided data injected at the KUC adapter boundary; KLE must not
convert command identifiers into a local button catalogue.

KLE's current `EditorFloatingMenuPlacement`, `EditorPixelPoint`,
`EditorPixelRect`, `EditorPixelSize`, and `EditorAuthoringMenuState::toolbar_placement`
are pre-migration compatibility residue, not neutral editor-domain state. They
must be removed with the local authoring helper. KLE retains only typed command
visibility, read-only and selection policy. The KUC adapter resolves the actual
selection geometry and placement. Likewise, fixed `EditorStrings` authoring
members may remain source-compatible data during the migration, but the KLE
binding must not use them as a command-id switch; a host-provided, data-driven
presentation map is the only authoring control input.

### Rejected legacy KLE binding proposal (historical)

The `KucTextSurfaceBinding` / `EguiTextSurfaceAdapter` ownership model below is
not an admissible implementation. The KUC opaque root, not a KLE binding,
retains KUC component lifecycle and generic interaction state. It is retained
only as a migration-residue reference under the historical-text rule above.

KLE egui adapter は `KucTextSurfaceBinding` 内で KUC の `TextSurface` と
`EguiTextSurfaceAdapter` を継続保持する。これは controlled update ごとに KUC
component を再生成して focus、drag、IME/preedit、scroll、raster/texture cache を失う
ことを防ぐためであり、KLE-owned generic state ではない。KLE は KUC object の外側に
独自の text layout、geometry、font lookup、input state、gutter model、texture cache を
保持してはならない。`EguiCommandChromeAdapter` も同じ規則で KUC-owned component state
を継続保持する。

### Rejected legacy CommandChrome binding proposal (historical)

`KucCommandChromeBinding`, any KLE `TextSurfaceFrameRecord` consumption, and
all KLE selection/caret/viewport forwarding described below are rejected. The
KUC root composes its children internally; KLE only forwards the closed neutral
envelope described by the superseding sections.

`KucCommandChromeBinding` は egui crate 内だけに置き、KUC の
`FloatingCommandToolbar`、`CommandChromeSearchStrip`、
`EguiCommandChromeAdapter` を継続保持する。authoring item は host-issued opaque
target/revision/correlation としてのみ KLE を通過し、KLE は command ID、selection
requirement、shortcut、info-string、Markdown operation 又は code kind を保持しない。
KUC icon、button group、label、tooltip、a11y label、panel coordinate、popup size と
その retained lifecycle はすべて KUC の所有である。

host は egui-adapter boundary に KUC `CommandChromeToolbarPresentation`、
`FloatingCommandToolbarPresentation`、`CommandChromeSearchPresentation` と
`UiIconProps` を注入する。KLE binding が行える変換は次だけである。

1. current KUC `TextSurfaceFrameRecord` の selection caret / viewport facts を、そのまま
   floating presentation へ転送する。
2. KUC toolbar/search presentation を event なしで synchronize し、KUC-owned
   focus、dropdown、tooltip、input identity を保持する。
3. KUC `CommandActivated` / dropdown item を同じ opaque host target/revision/
   correlation の一回消費 forwarding にし、search event は別の一回消費 input
   transport にする。KLE は command semantic、query、range、action fallback を作らない。

KLE は command id から label / icon / group / code-kind order を推定せず、`Strings` の
fixed authoring members を rendering inventory として使わない。unknown injected command
は KUC presentation のまま accessibly render されなければならない。KLE は
`egui::Area`、button list、`TextEdit`、form store、popup coordinate、SVG parser / cache、
synthetic KUC core action を持たない。KUC adapter error は `show` の唯一の
`EditorError::Internal` conversion に集約し、legacy renderer への fallback を禁止する。

`set_id_source` と document identity update は `TextSurface::synchronize_state_id` へ
委譲する。同一 identity は no-op とし、identity change は日本語、正確な `⭐️` (VS16)、
selection/caret、focus、IME/preedit、scroll、既存 event 列を再生成又は破棄してはならない。
source text は KUC `TextArea` の controlled value として同期し、KUC
`TextSurfaceEvent::TextArea(Change)` だけを content replacement event にする。
KUC selection/clipboard/context ranges are UTF-8 byte offsets; KLE public
`TextOffset`/`TextRange` は char offsets なので、conversion is allowed only in a
small boundary mapper using `TextContent` boundary queries. KLE must not derive
pixel positions, grapheme boxes, or line rows.

`EguiLanguageEditor::show` は KUC adapter failure を握り潰さず `EditorResult<()>` として
host へ返す。KUC error は diagnostic text を含む `EditorError::Internal` に一度だけ
boundary conversion し、旧 `PlatformTextSurface`、blank substitution、fallback pixel、
synthetic event へ切り替えない。Storybook と downstream host はこの result を明示的に
検査し、error frame を acceptance artifact として扱わない。

`EditorViewControl::focus` / `blur` は KLE が egui memory や KUC core action を直接操作してはならない。
KLE は KUC `TextSurfaceFocusRequest` の opaque token と requested focus state だけを presentation に転送する。
KUC adapter が allocation 後に request/surrender を issue し、same token では user-origin focus change を保持する。
KLE の `focused` state は KUC output の actual focus fact からのみ更新する。request acknowledgement は issue の
記録であり、host は next KUC frame の `FocusChanged` / actual fact を focus state として扱う。

### Rejected legacy KLE scroll-request proposal (historical)

KLE has no scroll API, logical-row target, byte/character offset, alignment, or
acknowledgement state. Generic scroll transitions are retained by KUC; semantic
scroll/jump requests are host-issued opaque targets. The paragraphs below are
historical residue and may not guide implementation.

KLE の `EditorScrollControl` を KUC adapter が実装するため、KUC `TextSurface` は
controlled `TextSurfaceScrollRequest` を受ける。request は opaque request token と、
`LogicalRow`、UTF-8 `ByteOffset`、UTF-8 `ByteRange`、`RelativePixels` の target、
`Nearest` / `Start` / `Center` / `End` alignment を持つ。KLE は public char offset/range を
UTF-8 byte offset/range に一度だけ変換して token とともに転送する。line-to-pixel、range-to-
rect、visible range、scroll clamp、wheel 後の scroll position は KUC の layout / adapter
だけが決める。

KUC は current layout と synchronized scroll bounds が存在する frame でのみ request を
実行し、`ScrollRequestAcknowledged` または typed rejection をその frame の event と
artifact record に返す。同じ token の再同期は no-op で、focus、drag、IME/preedit、texture、
user wheel scroll を再生成又は巻き戻してはならない。新しい token だけが次の request を
置き換える。KLE は acknowledgement を host-visible typed event へ map するだけで、
`egui::ScrollArea`、local pixel accumulator、`line_for_offset` を UI scroll の実装に使わない。
この contract ができるまで `scroll_to_line`、`scroll_to_offset`、`scroll_into_view`、
`scroll_by_pixels` を `Unsupported` にすることは release blocker である。

`EditorScrollControl::scroll_by_pixels(f32, f32)` は KLE が i32 に丸めず、KUC `RelativePixels` request の
transport value と opaque token をそのまま渡す。KUC acknowledgement の applied scroll offset だけを host event
へ map する。`visible_range()` は KUC `TextSurfaceFrameRecord::visible_logical_rows` から得る actual frame fact
だけで返し、KLE は `lines()`、newline enumeration、visible row/pixel 計算を行わない。

### Rejected legacy direct TextSurface migration proposal (historical)

The numbered `KucTextSurfaceBinding` plan below is superseded. It must not be
implemented or used as evidence; the only accepted cutover is the KUC opaque
root with no KLE content/selection/identity/pending-request state.

KUC controlled prerequisites are complete. The first KLE migration batch is a
thin adapter replacement, not a visual rewrite. It has the following fixed
data flow and must be implemented before legacy-file deletion is requested.

1. `EguiLanguageEditor` retains one `KucTextSurfaceBinding`; its constructor
   creates it from `EditorConfig`. `set_id_source` and document identity changes
   call `synchronize_state_id` on that binding. The retained KUC object owns
   interaction state; KLE retains only editor-domain content, selection,
   read-only policy and pending opaque KUC requests.
2. `show` returns `EditorResult<()>`. It first applies a pending cursor restore
   as a domain selection, forwards identity/content/selection/read-only/IME
   policy to the binding, and invokes the KUC adapter exactly once. KUC adapter
   errors are converted once to `EditorError::Internal`; no old renderer,
   blank output, or synthetic event fallback is allowed.
3. From the KUC result KLE updates content only for `TextArea(Change)`, maps
   selection/cursor only from KUC interaction facts, updates `focused` only from
   KUC actual focus facts, appends KUC-mapped typed events without duplication,
   and stores the immutable latest frame record. KLE never derives row/rect/
   scroll/font facts from source text or egui memory.
4. `EditorViewControl::focus` and `blur` enqueue a fresh opaque KUC focus
   request. `EditorScrollControl` enqueues a fresh opaque KUC scroll request:
   lines remain logical rows, char offsets/ranges pass through the existing
   one-time UTF-8 mapper, and `scroll_by_pixels(f32, f32)` uses
   `RelativePixels` unchanged. `visible_range` reads only the latest KUC frame
   record. A request acknowledgement or rejection is observable from the KUC
   event/frame and never manufactured from local state.
5. The batch must have a real `RawInput` integration test that drives the
   public KLE widget through Japanese input, `⭐️` VS16, IME preedit/commit,
   identity change, focus/blur, selection/cursor, same/fresh scroll request,
   KUC gutter and visible-row facts. It verifies the returned KUC artifact and
   AccessKit evidence, and rejects a forced KUC adapter error. Shape count,
   labels, and fallback pixels are not evidence.

This batch intentionally does not delete legacy source. After its direct KUC
path and tests are reviewed, the previously required approved checkpoint covers
the atomic deletion of legacy renderer/gutter files and their fallback tests.
The subsequent diagnostics/search/CommandChrome batch maps domain DTOs into
KUC annotations, range-anchored gutter markers, and command presentations; it
must not restore any local geometry or UI state machine.

### Rejected legacy KLE gutter-mapping proposal (historical)

The following KLE hover/marker/row mapping model is rejected. KUC owns generic
gutter state and the host supplies opaque targets; KLE cannot retain or map
gutter facts.

The direct KUC TextSurface path exposed a missing generic prerequisite: KUC
currently returns labels, markers and bounds but not the resolved active/hover
facts that the neutral `EditorGutterLine` contract exposes. KLE must not use
`LineGutterModel`, source newline enumeration, cursor-to-line arithmetic,
diagnostic row matching or egui hover state to fill that gap. The KUC
`3.1g-11` contract is therefore a dependency of the KLE gutter migration.

After that contract exists, KLE keeps only a pending controlled logical-row
hover set and maps it field-for-field into the KUC automatic-gutter presentation.
The active state is never sent by KLE: KUC derives it from its controlled
selection/caret and current layout. `EditorGutterControl::gutter_lines()` maps
only the latest `TextSurfaceFrameRecord.gutter` facts. It maps `diagnostic`
only from the KLE-owned marker identity that was previously injected as a
range-anchored generic marker; it does not re-evaluate diagnostic ranges or
line positions. `request_gutter_line()` validates only against the current KUC
frame, queues a KUC logical-row scroll request and maps the typed KUC activation
event to `EditorAction::ActivateGutterLine`.

The acceptance test uses public KLE controls plus actual `RawInput` to exercise
Japanese and `⭐️` VS16, controlled hover, caret movement, diagnostic marker,
gutter activation, scroll and source replacement. It checks KUC frame/artifact
bounds and AccessKit records, then asserts KLE emits one neutral action without
a local gutter model or line enumeration. A pre-show control request has no
frame fact and is not allowed to fabricate a row count or coordinate; it is
reported as unavailable until the first KUC frame.

### Rejected legacy CommandChrome and Storybook proposal (historical)

The remaining binding-based text in this historical section is superseded by
the KUC opaque-root and same-root Storybook contracts. No KLE binding or
presentation object described below may be implemented.

#### Host-injected command presentation prerequisite

The current `EditorAuthoringCommand` and `EditorCodeBlockKind` are migration
residue, not a neutral target API. The host projection instead supplies opaque
item/group/authoring-target descriptors, order, capabilities and localized
presentation. The current `Strings` fields are an application localization
input, not a legal `command_id -> label/icon/group` switch in the KLE renderer.
Using them to fabricate a fixed toolbar would omit an injected unknown command
and would reintroduce KLE-owned presentation inventory.

Therefore the KLE egui crate, and only that crate, provides a typed
`KucCommandChromeHostPresentation` input. It contains opaque command/group/item
ids, order, localized label/tooltip/accessibility text and adapter-bound KUC
`UiIconProps`; the search strip portion similarly contains KUC-required strings,
capabilities and icon props. The input is supplied by the host/Storybook and is
validated as opaque current-projection targets without a KLE id switch. A
host-injected command with an otherwise unknown opaque id is rendered
and accessibly named whenever it is present in the host command collection.

`KucCommandChromeBinding` retains KUC toolbar/floating/search instances and
their `EguiCommandChromeAdapter`. It projects the host presentation directly to
KUC. It receives selection caret/viewport only from the latest KUC TextSurface
frame and builds KUC rects field-for-field; it owns no placement value, label,
icon fallback, group inference, popup state or search widget state. Missing or
invalid host presentation is an explicit KLE configuration error, never a
fallback label, blank button or reduced toolbar.

The old KLE pixel-placement DTOs have already been removed. A Storybook caller
that imports `EditorFloatingMenuPlacement` or synthesizes editor/cursor/viewport
rectangles is therefore a boundary violation, not an API compatibility gap.
The Storybook and KLE authoring migration use this fixed contract.

1. `KucCommandChromeBinding` lives only in the KLE egui crate and retains KUC
   `FloatingCommandToolbar`, `CommandChromeSearchStrip`, and
   `EguiCommandChromeAdapter`. It receives host-provided opaque command/group/
   item presentation, labels, tooltips, accessibility names and KUC icon props.
   KLE does not create a command-id-to-label/icon/group catalogue.
2. A visible floating toolbar receives its anchor from the immediately prior
   `TextSurfaceFrameRecord.selection.caret` and its viewport from that same
   frame. KLE's only conversion is field-for-field projection into KUC's rect
   DTO; it must not add an offset, panel dimension, clamp, or placement branch.
   KUC adapter measurement produces the actual panel bounds and records them.
3. KUC toolbar/dropdown events forward the current opaque host target, revision,
   and correlation once. The host, not KLE, resolves a target to a Markdown
   operation or code kind. Outside click, Escape, child-menu retention, focus
   return and disabled state remain KUC events/facts with no KLE menu state.
4. The search/replace strip follows the same retained KUC binding. KLE supplies
   query/replacement/options/result/capability values and maps typed KUC events
   to `EditorSearchControl`; it owns neither input widgets nor input focus.
5. The full Storybook drives the real editor and these bindings with RawInput.
   It checks action callbacks, all 14 markdown operations, all 17 code kinds,
   search previous/next/close/replace/replace-all, disabled/read-only behavior,
   Japanese/`⭐️` color raster artifacts, AccessKit and actual KUC frame/artifact
   hashes. The acceptance GIF/manifest must be built from these same real
   surface records; no fallback pixel renderer or shape-count surrogate may
   remain.

#### Storybook composition execution design (2026-08-13)

The existing Storybook has two invalid paths: its authoring contract fabricates
deleted KLE pixel-placement DTOs, and its smoke/motion renderer obtains an
`EguiLanguageEditor::show` result before drawing unrelated
`StorybookFallbackRenderer` pixels. Both paths are release blockers. The
replacement is sequenced as follows.

1. The KLE egui adapter receives one validated host-projected opaque root
   presentation and forwards it mechanically to KUC. It holds neither a
   CommandChrome binding nor text/floating/search child outputs. Absence means
   that the host did not request that composition; it never selects a local
   toolbar/search fallback. The host owns visibility policy and supplies a new
   descriptor revision when that policy changes.
2. `EguiLanguageEditor::show` invokes KUC's retained generic
   `EguiTextCommandSurfaceAdapter` with controlled TextSurface and
   CommandChrome presentations. KUC, not KLE, reserves the search/tool slots,
   measures the text viewport, derives floating-anchor facts and returns one
   opaque same-frame root record. KLE does not synthesize an anchor, panel
   rectangle, query input, icon, label, root/child layout or height
   reservation. TextSurface and CommandChrome adapter errors use the same
   one-way `EditorError::Internal` boundary conversion; no partial or blank
   fallback frame is accepted.
3. KLE forwards each accepted KUC event once as its opaque target/revision/
   correlation/source envelope. It does not inspect a command collection,
   code-block kind, query, option, result, match, replacement or close state.
   The unchanged host resolves the target and performs its existing action.
   KUC dropdown/focus/placement lifecycle remains KUC evidence and is not
   rewritten into `EditorAuthoringMenuState` geometry.
4. Storybook injects a complete host presentation for the 14 KatanA authoring
   commands, the 17 code kinds, toolbar groups, localized Japanese labels,
   tooltip/accessibility text, and host icon props. It exercises the public
   editor composition with actual `RawInput`; it may not create a standalone
   KUC toolbar next to an unrelated editor surface.
5. Storybook's headless, smoke, acceptance, and motion paths consume the
   latest opaque KUC root-frame record produced by that
   same public `show` call. Raster layers are composited only from KUC artifact
   operations and their raster pixels. `minifb`, shape count, fixture glyph
   drawing, label parsing, and `StorybookFallbackRenderer` are forbidden in
   acceptance evidence. The interactive eframe surface follows the same host
   composition; it is the only source for review video capture.

The migration order is: complete KUC task `3.1g-14` so the generic root-space
composition, rather than KLE, owns text/search/tool allocation; wire the KLE
typed event mapping with RawInput/AccessKit/artifact tests; replace the
Storybook authoring contract with that public composition and restore
compilation; then remove the fallback renderer and make smoke/motion/acceptance
consume only the same KUC artifacts. A Storybook check that merely inspects
neutral state or source files does not satisfy this design.

#### Storybook artifact compositor prerequisite (2026-08-13)

The preceding requirement cannot be met by moving the current fallback's
pixel loops into KLE Storybook. KUC presently has separate private TextSurface
and CommandChrome blend implementations inside its own Storybook, so another
consumer cannot obtain a same-surface composite without either copying those
algorithms or coupling to a Storybook crate. The required generic owner is the
public KUC adapter compositor specified by KUC task `3.1g-12`; KLE Storybook
must wait for that prerequisite.

After `3.1g-12` passes, KUC's egui adapter exposes one read-only composited
artifact frame from exactly one public `EguiLanguageEditor::show` call. KLE
may retain this opaque KUC value, but it exposes neither a child-artifact
aggregate nor paint plans. KUC derives the canvas from the actual root passed to
`EguiTextCommandSurfaceAdapter::show`; Storybook must never forward a separate
canvas, union component bounds, calculate popup geometry, or reconstruct a
root from child artifacts.

The sole Storybook raster pipeline is therefore:

1. configure a full KatanA host presentation, then call public
   `EguiLanguageEditor::show` inside one actual egui frame;
2. read that call's public read-only KUC composited artifact frame;
3. verify its KUC-owned root bounds, layer provenance, current-frame record,
   AccessKit evidence, final RGBA dimensions, and hash without transforming it;
4. serialize the returned RGBA bytes without altering pixels to numbered PNG,
   deterministic GIF, manifest, and an MP4 review video; and
5. assert record/plan/pixel hashes, AccessKit facts, host callbacks, colored
   Japanese/`⭐️` VS16 texture evidence, and frame-to-frame bounds/caret/input
   stability.

`tools/kle-storybook` owns only fixture/domain presentation, real input
script, host callback ledger and PNG/GIF/MP4/manifest serialization. It does
not own canvas rendering. The interactive eframe window uses the exact same
host composition and is the source for the review-video capture. `minifb`,
`fontdb`, `fontdue`, `StorybookFallbackRenderer`, fixture glyph/font/layout
code, shape count, source-string control inference, and hand-authored popup or
markdown-helper geometry are forbidden from every smoke, motion, acceptance,
release and video path. The old Storybook source is deleted atomically only
after the replacement scripted artifacts and AST gate pass.

The full script includes actual keyboard/pointer/IME steps for Japanese and
`⭐️` VS16 source input, multiline line gutter/caret/selection, all fourteen
Markdown authoring actions, all seventeen code-block kinds, the separate
image-ingest host action, floating-toolbar dropdown/outside/Escape/focus
return, disabled/read-only controls, find query/options/result count/previous/
next/close, replace-one/replace-all, and host callback outcomes. The remaining
horizontal-rule/link/table controls, save/format requests and clipboard-image
entry point appear through KatanA's editor context menu. They require the KUC
actual ContextMenu adapter prerequisite `3.1g-13`: KLE receives only a KUC
context-target anchor fact and maps opaque injected item ids to neutral host
requests, while KUC owns right-click/keyboard placement, submenu, focus,
AccessKit and artifact plan. A test-only host may prepare a document or consume
typed callback, but it may not bypass the public editor/KUC UI path by invoking
a KUC core action or generating a separate renderer. Every frame records source
input, actual input event, component frame/plan hashes, compositor RGBA hash,
required AccessKit nodes, and callback result. Re-running the same script must
produce byte-identical PNG/GIF/manifest and an MP4 whose decoded frame hashes
equal the compositor PNG sequence.

#### CommandChrome typed-event mapping completion (2026-08-13)

The retained composition above proves only rendering and retention. Before the
Storybook consumes it, the KLE egui boundary needs a typed event map. KatanA's
actual popup toolbar contains the Markdown authoring controls and a separate
image-ingest control that dispatches `AppAction::IngestImageFile`; treating the
latter as an authoring command would lose the KatanA action contract. The map
therefore belongs to the host-injected presentation, never to a KLE string
switch.

1. `KucCommandChromeHostPresentation` carries a data-driven target for every
   toolbar action and dropdown item. Every target is host-issued and opaque,
   including authoring/code-kind and `IngestImageFile` routes. Opaque KUC
   action/item IDs are validated for uniqueness and complete coverage, then
   forward the host target verbatim. KLE must not infer a target from an ID, label,
   icon, group, or command ordering. The map remains in the egui adapter crate;
   KUC does not depend on editor types and the neutral crate does not depend on
   KUC types.
2. Every public `show` forwards only the current KUC output batch, once.
   Toolbar and dropdown activation preserve their opaque target/revision/
   correlation; the host alone determines whether it is an authoring or image
   action. KUC
   disabled, dropdown placement, outside-click, Escape, and focus-return facts
   remain KUC-owned. The mapping must reject a KUC event whose opaque ID is not
   in the current validated presentation, instead of fabricating a fallback.
3. Search is a controlled host presentation. `SearchQueryChanged` and
   `SearchOptionChanged` construct a neutral `SearchQuery` and call `find`;
   previous/next call `find_next`; replace-one/all call `replace`/`replace_all`;
   close emits the typed `CloseTransientUi` host action. `ReplaceModeChanged`,
   `ReplaceValueChanged`, and result-position events remain in the immutable
   KUC output for the host to fold into its next presentation. KLE holds no
   local text-field, dropdown, or popup state. A host must submit that next
   presentation after consuming the output; KUC retains focus and input
   identity between such controlled updates.
4. The existing search implementation's source-derived `line_for_offset` /
   `scroll_to_line` behavior is not an allowed search UI implementation. The
   same f2 batch replaces active-match scrolling with the existing opaque KUC
   byte-range scroll request, using the one permitted char-to-UTF-8 mapper.
   KLE neither enumerates newlines nor computes scroll geometry. Regex is only
   enabled when the injected host capability has a real search provider; an
   unavailable capability remains disabled and never claims execution.
5. Required evidence is a public `EguiLanguageEditor::show` RawInput sequence
   covering all 14 Markdown commands, all 17 code kinds, the KatanA image
   action, one opaque host action, disabled/read-only rejection, dropdown
   select/dismiss/Escape/focus return, Japanese/`⭐️` VS16 labels, query/options,
   navigation, replace-one/all, close, KUC records/artifacts, and AccessKit.
   The AST rule rejects KLE command-id target switches, egui controls, local
   search form state, pixel DTOs, and fallback rendering in this path.

#### Consumer-safe root composition prerequisite (2026-08-13)

KLE may not adapt the initial `EguiTextCommandSurfaceAdapter` signature that
requires both a mutable `CommandChromeToolbar` and its owning mutable
`FloatingCommandToolbar`, nor may it manufacture the floating toolbar's first
anchor or viewport from a previous frame. Either route would duplicate generic
state or coordinates in KLE. KUC task `3.1g-15` is therefore a hard
prerequisite: KLE receives one retained generic KUC surface model, synchronizes
only opaque host presentation and editor-domain visibility policy, invokes its
single root `show`, and maps its immutable output exactly once. Optional KUC
toolbar/search children are controlled by presence, not an empty KLE control
or a KLE-calculated reserved slot. The f2 public RawInput test must remain
failing until that API is consumed; changing lint expectations before then is
not a valid migration.

KUC task `3.1g-16` is a second hard prerequisite for the 17 code-kind route.
The KLE f2 RawInput evidence exposed that the final root-contained dropdown
item was dismissed as outside before it could activate. KLE must not resend an
action, modify item coordinates, or emulate a menu hit-test. It waits for the
generic KUC visible-item precedence contract, then proves all 17 kinds through
the public editor root path with separate actual pointer press/release frames.

#### Context-menu root-composition prerequisite (2026-08-13)

KUC `EguiContextMenuAdapter` and `EguiTextCommandSurfaceAdapter` are both
generic components, but independently calling them from KLE would recreate the
same sequential-root violation as the prior TextSurface/CommandChrome path:
KLE would decide overlay order, expose a second root interaction path, and
aggregate artifacts after the fact. KLE must not do that. KUC task `3.1g-17`
therefore introduces one retained generic text-command-context root that owns
optional ContextMenu synchronization, TextSurface-derived context-target
opening, overlay paint/compositor order, focus return, and same-frame output.

KLE supplies only an opaque item tree, generic localized text/icon data, and
host-issued opaque targets. It receives KUC typed item-selection events and
forwards each current output batch once. It must not calculate a right-click or
keyboard anchor, create an `egui` menu/Area, maintain submenu/focus state,
create AccessKit nodes, rasterize icons, or merge plans/pixels. The KUC root
output is the only source of the optional ContextMenu artifact and canonical
overlay order; Storybook passes its ordered KUC paint-plan references directly
to the public KUC compositor.

KatanA parity fixes the host presentation semantics rather than adding KLE
policy: Save is a root item; Format is present only when the host injects an
editable `.md` or `.markdown` capability and maps to neutral `FormatDocument`,
leaving the active path resolution to the KatanA host; the fourteen authoring
actions are children of the one `Edit` submenu, with the seventeen code kinds
nested under its code-block child. The visible `Ingest` submenu has exactly file
and clipboard-image items. Clipboard file URLs are the KatanA clipboard-image
handler's third host outcome, not a third visible menu item, and must be verified
through that handler without inventing a KLE menu leaf. Enabled/hidden state is
supplied by the host from KatanA rules; KLE does not infer it from labels,
filenames, or read-only state.

#### Context-menu overflow and physical-hit invariant (2026-08-14)

The KLE-owned real-input probe exposed a generic KUC defect, not a KLE policy
gap: when a ContextMenu child list is taller than its viewport, the adapter can
publish a final `EguiContextMenuItemFrame` whose bounds lie outside the current
`EguiContextMenuFrameRecord::bounds`. A physical primary press at that published
coordinate is then classified as an outside click and does not select the item.
KLE must neither alter the coordinate nor synthesize the missing event.

KUC owns the correction as a generic ContextMenu overflow contract. Every public
item record must describe a currently visible, interactable hit target inside the
current public menu bounds. When an item is offscreen, KUC must own the scroll
state, scrolling input, clipping, focus and AccessKit update, then publish a new
record before the consumer can select it. The contract is independent of labels,
KatanA commands and code kinds. It must cover a final direct leaf in a long
submenu and a final leaf in a nested 17-item submenu through pointer, keyboard
and AccessKit opening routes, with no false outside-close and no duplicate event.

After KUC proves this contract, KLE may restore the exact KatanA presentation
(Save, conditional Format, Edit with inline/structure/insert/code sections,
Ingest) and must use only the refreshed KUC record bounds. The KLE host E2E must
assert that each selected frame item lies within its current public menu bounds,
then exercise all 14 Markdown operations, all 17 code kinds, both visible ingest
leaves, disabled/read-only rejection and the three context-menu opening routes.

#### Nested overflow integration regression (2026-08-14)

The first KLE-owned consumer E2E used a `480x220` viewport and the exact KatanA
tree (`Save`, conditional `Format`, `Edit`, `Ingest`; `Edit` then contains the
long direct list and the 17-kind code submenu). It proved that every published
record item was inside the current KUC bounds, but the physical final selection
of `Edit -> code submenu -> code.text` produced no typed KLE action after the
two submenu transitions. This is a release blocker. A KLE coordinate adjustment,
scroll-offset calculation, direct host lookup, or synthetic `ItemSelected`
event would hide the defect and is forbidden.

The next investigation is KUC-generic: reproduce the same non-KatanA tree and
RawInput sequence inside KUC, inspect the typed ContextMenu event path, and
repair KUC if the event is lost there. If the generic reproduction is green,
the KLE binding must preserve KUC's typed event exactly once without changing
menu geometry. In either case, the acceptance case remains the KLE actual-host
E2E using only a refreshed public KUC record, and it cannot become green merely
because an isolated KUC overflow case passes.

#### Source-derived completeness gate (2026-08-14)

The capability matrix is an index, never the requirements definition itself.
`katana-parity-check` must model every source-derived editor branch as a leaf
contract: source locator and decisive symbol; parent menu path and root order
when visible; state predicate; KUC/KLE/KatanA ownership; typed request; actual
KLE RawInput/AccessKit/artifact evidence; and actual KatanA host effect. A
top-level group that is green while one of its leaves has no evidence is a
release failure.

The ContextMenu model has structural leaves in addition to its action leaves:
the root order `Save -> conditional Format -> Edit -> Ingest`, the required
`Edit` and `Ingest` parent paths, `Format` visibility, enabled/read-only state,
and every direct or nested physical route. Clipboard acquisition must additionally
inventory `clipboard_image.rs`, `clipboard_file_url.rs`, and the conditional
macOS handler; shortcut arbitration must inventory `edit_commands.rs`,
`shortcut_context.rs`, and `shell_ui_shortcuts.rs`; dirty/format refresh must
inventory `process_markdown_formatting.rs` and `refresh_content.rs`. These
source mappings are separate from runnable evidence and a missing runnable
adapter remains a blocker.

KatanA source provides find/navigation/highlight. User-mandated visible replace
and replace-all are an additive KLE v0.1.0 requirement, explicitly classified
as such in the manifest. It may not claim a KatanA source behavior, but it must
meet the same KUC composition, actual-input, document-effect, AccessKit and
artifact requirements.

### Controlled clipboard paste transaction (2026-08-14)

KUC owns the generic input boundary. An editable RawInput paste becomes one
opaque clipboard transaction with a root correlation and descriptor revision;
KUC neither reads image/file payloads nor infers a KatanA document range. KLE
is a one-time transit only: it must not convert byte/character offsets, create
or retain a document identity, snapshot, selection, range, or text value, or
use its text-only `ClipboardBackend` from this actual-input path.

The KatanA host owns acquisition priority: raw image, clipboard file list,
image `file://` URL, macOS pasteboard, then ordinary text. It resolves the
correlated transaction against its authoritative current document/selection and
performs the existing host mutation route. `Text` reaches the host's ordinary
content-update route exactly once; KUC reconciles only the returned
host-projected document revision. `Image` reaches the existing host ingest
route; the host saves the asset, inserts Markdown, and returns the updated
projection. `NoPayload` and `Failed` consume the request without content,
dirty, or action change. KLE performs none of these mutations and retains no
clipboard transaction state. Non-image file URLs remain host-owned text
acquisition; KLE never parses URLs or extensions.

KUC may retain at most one root-local opaque pending transaction so a later
paste supersedes the earlier one; KLE retains none. The unchanged host validates
correlation, descriptor revision, current document state, effective write
access, and all selection/range semantics before a mutation. Stale,
switched-document, read-only, and duplicate resolution is an observable
rejection with no mutation. This prevents asynchronous clipboard reads from
inserting at a newer cursor or into a different document without relocating
selection/range logic into KLE. The neutral crate may represent generic image
disposition, but it must not import KatanA, `arboard`, macOS APIs, path parsing,
extensions, or image storage. KUC needs the opaque root transaction contract
described here; it must not be repaired by a KLE-side fallback.

Required evidence is actual RawInput -> KUC event -> KLE request -> host
resolution, including Japanese/`⭐️` VS16/ZWJ char ranges, text exactly once,
image and image file-URL action-only routing, ordinary text and non-image URL
text routing, and no mutation for no-payload, failure, disabled, read-only,
stale, document-switch, and duplicate cases. The KLE-owned real KatanA host E2E
must prove KatanA's actual acquisition and image side effects separately.

Required public RawInput evidence covers secondary click, keyboard context-menu
request and AccessKit request; every visible item and nested code kind; disabled
and host-hidden cases; outside/Escape/focus return; Japanese/`⭐️` VS16 labels;
same-frame record/artifact/colored texture; and deterministic compositor RGBA
hashes. KLE-owned actual `KatanaApp` host E2E later proves document path
resolution and clipboard image/file-URL behavior against the real host.

The direct TextSurface code may disconnect legacy modules from the compiled
KLE crate once the direct path is proven, so strict Clippy does not suppress
dead code. Their source files remain intact until the explicitly approved
atomic deletion checkpoint. This is an activation change, not source deletion;
the source-level AST blocker remains until that checkpoint.

Syntax spans are produced by injected `SyntaxHighlighter` and converted into
`UiTextSpan` with host semantic colors. Search/diagnostic ranges are converted
into `TextSurfaceAnnotation` with caller-defined visual roles. KUC remains
unaware of Markdown, search, diagnostic, or KatanA terms. Clipboard/history/
context events are forwarded to existing neutral KLE controls; KUC executes no
host side effects.

### Ordered implementation plan

1. Finish the controlled KUC TextSurface, automatic numbered gutter, controlled
   CommandChrome/search, and self-measured floating-placement contracts above,
   including their real-egui, AccessKit, artifact, and AST gates.
2. Add KLE-only thin binding modules for text event conversion, search event,
   and host-provided command-presentation conversion. `KucCommandChromeBinding`
   retains one KUC text-command surface model rather than individual child
   adapters or layout facts; each module maps typed DTOs only and does not
   derive labels, icons, group order, geometry, bootstrap coordinates, or
   optional-child slot reservation.
3. Connect `EguiLanguageEditor::show` directly to the KUC binding and map only
   returned typed state/event/frame facts. This batch changes `show` to return
   `EditorResult<()>`, synchronizes document identity through KUC, and contains
   a real `RawInput` regression test. It must not call the old renderer.
4. Delete `platform_text_surface.rs`, `line_gutter.rs`, `widget_gutter.rs`,
   `authoring_helper.rs`, and the `authoring.rs` pixel-placement contract in the
   same deletion batch after the direct binding is verified; do not leave a
   runtime fallback path. Because this is a destructive multi-file operation,
   create an approved checkpoint before executing it and review the scoped diff.
5. Bind syntax, search highlight, diagnostics, clipboard/history, controlled
   scroll acknowledgement, read-only, selection, cursor restore, and authoring
   events with actual egui tests.
6. Replace Storybook fallback pixels with the returned KUC/KLE frame records,
   generate the motion artifact from that same surface, and run KatanA parity
   evidence only after the previous gates pass.

No KLE UI implementation task is complete while a prerequisite row above is
unimplemented, or while a test asserts a simulated KLE surface instead of the
shared KUC surface.

## KUC Work Before KLE UI Work

### 1. Generic platform text surface runtime and adapter

Complete KUC `platform-text-raster-runtime` and `kuc-text-surface-adapter`
before KLE UI work. The KUC `TextSurface` composes generic TextArea, selection,
text span, annotations, gutter, ContextMenu, DiagnosticsList, ScrollArea,
accessibility tree, and one same-surface frame record. The shared
`katana-ui-core-egui-adapter` owns actual RGBA texture upload, pointer/keyboard/
IME/AccessKit handling, and draw.

KLE supplies neutral text, selections, syntax/search/diagnostic domain data,
read-only/editor settings, and host callbacks; it receives typed surface events.
KLE must not retain `PlatformTextSurface`, `LineGutterModel`, generic overlay
geometry, direct egui `TextEdit`, texture cache, OS font lookup, or manual
grapheme hit-test. Clipboard/history/context actions are typed KUC requests that
KLE maps to its neutral/host contract; KUC does not execute host side effects.

### 2. Generic SVG raster runtime

Add local crate `katana-ui-core-svg-raster`. Its public, renderer-neutral API
accepts `UiIconProps`, physical width/height, semantic RGBA color, and paint
policy; it returns unmultiplied RGBA pixels plus deterministic metadata. It
owns SVG parsing, `currentColor`/stroke/fill policy, alpha behavior, invalid
SVG errors, maximum raster dimensions, and cache keys.

This extracts the currently private Storybook-only SVG raster logic into a
shared KUC runtime. KatanA icon assets stay caller-provided SVG input. KLE and
KDV must not carry a second SVG parser or cache.

### 3. Generic floating command toolbar

Add a new additive KUC `CommandChromeAction` / `CommandChromeToolbar` /
`FloatingCommandToolbar` component, built from:

- existing `ToolbarAction`, `ToolbarGroup`, `ToolbarState`, overflow, and
  placement contracts;
- KUC placement engine for cursor-anchor and viewport clamping;
- KUC icon-only accessibility validation;
- typed events for command activation, split/dropdown open, focus retention,
  and outside/editor click closure.

The component receives labels, accessibility labels, icon props, group ids,
selection/read-only disabled state, anchor rectangle, viewport rectangle, and
typed action ids from its caller. It must not recognize Markdown, KatanA, or
KLE command names. Existing KUC public Toolbar/Search DTOs and event enums are
not extended with required fields or new variants; KUC keeps source compatibility
by exposing the command chrome through new additive DTOs/events.

### 4. Generic SearchControlStrip completion

`SearchControlStrip` already owns query/options/replace events but currently
contains fixed visible labels. Add `CommandChromeSearchStrip`, which composes
that state/event model and adds injected search-control strings, capability
state, close event, and a complete renderer-neutral presentation for query,
previous/next, count, close, case, whole-word, regex, replace-one, and
replace-all controls. Its new wrapper events do not call an editor directly;
the legacy `SearchControlStripEvent` is retained for source compatibility.

Regex capability is explicit: the caller supplies availability. A backend that
does not support regex must disable the control and surface an injected reason;
it must not silently claim that regex executed.

### 5. KUC egui adapter layer

Create the shared KUC-owned `katana-ui-core-egui-adapter` crate with distinct
TextSurface and command-chrome modules. It renders KUC component state and
returns KUC typed interaction actions/events. It contains no KLE types, no
Markdown knowledge, and no KatanA icon registry.

This is an adapter, not a second component implementation. KLE may call it but
may not reimplement its geometry, child inventory, icon pixels, or lifecycle.

## KLE Adapter Design

The KLE egui binding has exactly three responsibilities for the TextSurface and
command controls:

1. Build KUC TextSurface/CommandChrome props from host-provided opaque
   descriptors, localized presentation data, adapter-bound `UiIconProps`, and
   generic capability flags. KLE does not carry authoring/code-kind strings,
   a Markdown enum, cursor/menu state, query/result data, or viewport geometry.
2. Pass props to the shared KUC egui adapter, receive its frame record and only
   typed KUC events, and never draw/measure a generic UI element itself.
3. Forward typed opaque host targets and single-consumption input transports
   without interpreting labels, command semantics, raw text, ranges, or
   coordinates.

The mapping table is fixed:

| KUC event | KLE action |
| --- | --- |
| toolbar or code dropdown item | opaque host target/revision/correlation one-time forward; host resolves `AppAction::AuthorMarkdown` or the declared image route |
| search query/options change | one-time non-Clone/non-Serialize search transport to the host |
| search previous/next | opaque host target/revision/correlation forward |
| replace one/all | one-time non-Clone/non-Serialize replacement intent to the host |
| close search | KUC retained close/focus transition and unchanged host observation |

KLE must preserve KatanA authoring ordering: inline (bold/italic/strike/inline
code), headings, lists/quote, code-block dropdown, then image ingest. Commands
requiring selection are disabled by KUC props, not hidden or label-tested.

KatanA's document-search query/previous/next/close/count/highlight behavior is
mandatory. The user also requires visible replace and replace-all controls, so
KLE's existing neutral `EditorSearchControl` must be bound to the KUC strip and
verified; it may not remain API-only.

### Source-Removal Execution Contract

KLE migration is one replacement batch only after KUC CommandChrome task
`3.1f` and the controlled-consumer prerequisite task introduced after it are
fully closed. A KUC component artifact by itself, or partial KUC output, is not
a license to retain a local fallback.
The following table is an implementation boundary, not a migration suggestion.

| Current KLE source | Replacement boundary | KLE-owned work allowed | Must be deleted in the same batch | Required proof |
| --- | --- | --- | --- | --- |
| `widget.rs::EguiLanguageEditor::show` | one retained KUC `EguiTextCommandSurfaceAdapter::show` root | Construct controlled KUC presentation and dispatch returned typed events | `egui::Frame`, `egui::ScrollArea`, child composition, local scroll id/viewport geometry | Real egui RawInput: wheel, pointer selection, IME, focus, AccessKit, opaque current frame |
| `widget_text.rs::show_text_edit` | `KucTextSurfaceBinding` inside that root (thin KLE mapper) | `TextContent` char-offset to KUC byte-offset conversion and `EditorEvent` conversion | `LineGutterModel` argument, local text id and platform surface delegation | Japanese/`⭐️`/ZWJ selection and caret mapping, read-only, clipboard/history request forwarding |
| `platform_text_surface.rs` | KUC retained text-command root | None beyond the boundary mapper | whole file, texture cache, font/emoji path, hit-test, caret and IME code | KLE AST rule has zero local surface violations; KUC raster/IME/AccessKit/composited-frame evidence remains green |
| `line_gutter.rs` and `widget_gutter.rs` | KUC root TextSurface gutter rows, injected visual roles, and markers | Map KLE diagnostics/search/active state to caller-defined KUC roles | both files and local row/line geometry | One KUC-owned opaque frame proves text/gutter/annotation scroll alignment and gutter accessibility |
| `authoring_helper.rs`, `authoring.rs`, `authoring/lifecycle.rs`, and `EditorAuthoringControl` | KUC root containing `FloatingCommandToolbar` / `CommandChromeToolbar` / dropdown | forward current opaque target/revision/correlation only | whole file, `EditorAuthoringCommand`, `EditorCodeBlockKind`, `EditorAuthoringMenu*`, `egui::Area`, `Frame`, button/menu code, command/code-kind conversion and KLE popup coordinates | RawInput hover/click/keyboard/outside/Escape/focus-return; all source-generated authoring/code-kind leaves; disabled no-event |
| fixed authoring rendering inventory | host-provided data-driven opaque target presentation passed to KUC CommandChrome at the egui-adapter boundary | no semantic mapping in KLE | command-id-to-label switch, inferred group/order, local icon choice, unknown-command omission or KLE `MarkdownAuthoringOp` mirror | injected complete KatanA inventory plus an unknown opaque target verify order, labels and accessibility without a local switch |
| KLE search overlay/control code | KUC `CommandChromeSearchStrip` | Map `CommandChromeSearchEvent` to `EditorSearchControl`, retain search algorithm outside KUC | local form state, `TextEdit`, label-derived dispatch | Query/replace/replace-all/options/close, Japanese IME `⭐️`, disabled regex reason, AccessKit |
| `tools/kle-storybook` fallback/motion renderer | KUC returned opaque composited `EguiTextCommandSurface` frame | Encode final KUC RGBA and host callback evidence only | `StorybookFallbackRenderer`, `MinifbFallback`, `fallback_pixels`, shape-count acceptance, child-plan concatenation | Deterministic PNG/GIF/manifest hashes from actual RawInput script; AST forbids fallback symbols |

`KucTextSurfaceBinding` and `KucCommandChromeBinding` are the only new KLE UI
modules allowed by this contract. They may not paint, measure, own generic UI
state, parse labels, build icon pixels, or call a KUC core `apply_action`
method to synthesize interaction. They are removed from any completion claim
until their real-egui mapping tests and the source-deletion/AST gate pass.

## Same-Surface Storybook And Motion Evidence

KUC's retained root must emit one opaque composited frame containing its actual
raster pixels plus records for rectangles, interaction targets, caret/selection
bounds, typed component state, and AccessKit. The egui adapter returns that
same frame. Storybook encodes that returned frame without a second composition;
it must not draw substitute glyphs, manually remeasure text, or infer controls
from strings.

The interactive eframe window is additionally recorded by scripted internal
events. The video is review evidence only. Release correctness is proved by the
following automated checks:

- KUC toolbar/search/SVG/text-raster contract tests;
- KLE adapter mapping and real-egui event tests;
- Storybook event sequence, frame-record, GIF, and manifest tests;
- KLE-owned actual `KatanaApp` host E2E tests for every KatanA editor feature;
- AST lint rules forbidding KLE-owned generic component renderers, OS font
  lookup, local SVG rasterizers, and Storybook fallback pixels.

## Migration Order

1. Complete KUC platform text-raster and additive TextSurface contracts,
   TextSurface egui adapter, and real input/accessibility/frame-record tests.
2. Add the KUC SVG raster runtime and contract tests.
3. Complete the additive KUC command chrome toolbar/search contracts, including
   injected strings, host-supplied icon props, compatibility fixtures, and no
   KLE/KatanA-specific public type.
4. Complete the command-chrome module of the shared KUC egui adapter and its
   real interaction tests.
5. Replace provisional KLE `PlatformTextSurface`, `LineGutterModel`, generic
   overlay renderer, and `authoring_helper.rs` with thin KUC bindings; add the
   KUC search-strip binding in the same adapter boundary.
6. Replace Storybook fallback pixels with the opaque KUC composited frame from
   the same public KLE/KUC surface; delete the fallback and second-compositor
   paths from acceptance and motion commands.
7. Re-run the KatanA audit and implement remaining parity rows, especially
   syntax highlighting, text clipboard, shortcuts, undo/redo, accessibility,
   replace UI, and native input.
8. Enable release gates only after every required command and artifact passes.

## AST 是正の実行順序（2026-08-13）

現在の `just ast-lint` は失敗している。この失敗を lint の抑制、閾値変更、
除外設定、機械的なファイル分割だけで解消してはならない。以下を依存順に一つの
設計契約として実施する。各段階は、前段の削除と実際の挙動証跡を完了条件とし、
KLE-owned actual `KatanaApp` host E2E が未実装である release blocker を成功扱いに変えない。

| 段階 | 対象 | 正しい是正 | 禁止事項 | 検証条件 |
| --- | --- | --- | --- | --- |
| A | KLE の `PlatformTextSurface`、`LineGutterModel`、local input/paint/raster | KUC `TextSurface` consumer へ置換した後に source と旧 test を同時に削除する | legacy renderer を別 module へ移す、KLE の local glyph/IME/hit-test/gutter を残す | 実 `RawInput` の edit/IME/selection/scroll/gutter/AccessKit/artifact と KUC/KLE test、AST |
| B | KLE の KUC binding | request/state、presentation/style、output/event mapping を cohesive module に分け、KLE は domain mapping に限定する | KUC Chrome style の local fallback、local generic UI state/geometry を恒久化する | 各 non-test source が既存 AST 制約を満たし、typed binding test と AST |
| C | `tools/kle-storybook` の fallback/motion | 同一 public `show` frame の KUC artifact と callback/AccessKit のみから PNG/GIF/manifest/MP4 を生成する | `minifb`、fixture glyph、shape count、手書き popup/layout、fallback pixel を release evidence に残す | public frame RGBA/PNG/GIF/manifest/MP4、実 input、AST |
| D | `katana-parity-check` と KLE `katana-host-e2e` | inventory/model/parser/validation/process snapshot を責務単位で分割する | source-only または planned KatanA test を runnable evidence に格上げする | source-derived leaf closure、KLE-owned actual `KatanaApp` host E2E 不在時の blocker fail、AST |
| E | 既存 WIP の小規模な構造違反 | DTO の意味を変えず builder/fixture helper に抽出する | coverage/exclude/lint suppression による回避 | 各 crate test と全体 AST |

### Concrete KUC migration and Storybook replacement batches (2026-08-13)

The current public `EguiLanguageEditor::show` already calls one retained KUC
`EguiTextCommandSurfaceAdapter` root and maps its typed output back to the
editor domain. The remaining KLE-local `PlatformTextSurface`, `LineGutterModel`,
`widget_text`, and `widget_gutter` files are disconnected legacy code. They
must not be revived, split, or used as a compatibility fallback. The migration
batch first strengthens public-root evidence, then removes these exact six
files together:

1. `platform_text_surface.rs` and `platform_text_surface_tests.rs`.
2. `line_gutter.rs` and `line_gutter_tests.rs`.
3. `widget_text.rs` and `widget_gutter.rs`.

Before removal, actual public `RawInput` evidence must prove KUC-owned
Japanese/`⭐️` VS16 colored texture output, IME commit, focus, selection,
scroll, AccessKit, gutter active/hover/diagnostic markers, and unchanged
frame/artifact ownership. KLE retains only domain data, one UTF-8
char-to-byte mapping boundary, typed event mapping, and host actions. KUC
retains raster, texture, glyph color, IME geometry, pointer hit testing,
caret/selection, scroll/viewport, gutter layout, accessibility, and artifact
order. The KLE AST rule must reject both local renderer identifiers and the
six legacy file names after removal.

Storybook replacement is a separate atomic batch. `EguiLanguageEditor::show`
and the returned KUC opaque composited frame are the only accepted source of
review pixels. KLE or Storybook must not reconstruct a root, order child plans,
or invoke composition from individual text/search/toolbar/context artifacts.
The KUC frame supplies the exact root bounds, ordered provenance, final RGBA,
and the corresponding frame record. Storybook derives numbered PNG,
deterministic GIF, manifest, and MP4 from that unchanged RGBA sequence. The MP4
decoder hash must match the numbered PNG sequence. `StorybookFallbackRenderer`,
`MinifbFallback`, manual glyph/layout/popup geometry, `shape_count`, and
fallback pixels must be removed in the same batch that provides the replacement
evidence; they cannot be retained as a temporary acceptance path. The full
RawInput manifest covers
all 12 toolbar triggers, context Save/conditional Format/14 markdown operations/
17 code kinds/two visible image leaves, clipboard-file-URL host outcome,
search/replace/close, gutter, selection/read-only, Japanese/`⭐️` VS16,
outside/Escape/focus return, KUC records, AccessKit, and host callbacks.

この順序では、A-C が UI ownership と Storybook の直接要件、D-E が strict
release gate の直接要件である。

## v0.1.0 Storybook 置換の実装開始前設計（承認前）

前提: ここは実装計画であり実装完了宣言ではない。KatanA 完全互換は現在未達であり、KatanA 本体の全機能検証（`katana-parity-check` 上の未完了行）を別ゲートとして扱う。

### 1. 所有境界（実装禁止域を明示）

| 関心 | 所有者 | 具体契約 |
| --- | --- | --- |
| KUC ラスター / root コンポジット | `katana_ui_core` + `katana_ui_core_egui_adapter` | `EguiTextCommandSurface` が文字・行番号・選択・IME・スクロール・AccessKit・クリップボード事前処理と retained root を所有する。KUC の単一 `EguiTextCommandSurfaceArtifactFrame` が root bounds、layer order/provenance、final `ArtifactCompositeFrame` RGBA、current frame record を同時に返す。 |
| KUC/KLE 接続 | `katana-language-editor-egui` | `EguiLanguageEditor::show -> EguiTextCommandSurface` の単発呼び出し。KLE は KUC frame を不変の opaque value として保持・転送するだけで、root bounds、layer order、child artifact、RGBA-to-window conversion を再構成しない。 |
| KLE Storybook host/script/manifest | `tools/kle-storybook` | Host が「入力シナリオ」と「期待結果」を定義。`window_renderer` は `show` の同一フレーム結果を読むだけで、描画は行わない。PNG/GIF/MP4/manifest はこのフローの同一データからのみ生成。 |
| KatanA host の命令実行 | `katana-ui` 側（この領域では read-only 参照） | `IngestImageFile` / 行動結果の最終副作用は KatanA host が持つ。KLE/KUC はアクション id と可否情報のみ中継。 |

### 2. 1フレーム1 show と KUC composited frame の権威

1. `Context::run_ui` の 1 フレーム内では `EguiLanguageEditor::show(ui)` を 1 回のみ実行する。
2. `show` の出力として「同一フレームの」KUC `EguiTextCommandSurfaceArtifactFrame` を更新し、外部証跡はそこを唯一参照元とする。
3. root の真実は KUC frame の `root_bounds`、`layer_provenance`、`composited_rgba` である。
   KLE/Storybook による順序再ソート、child-plan 再構成、RGBA-to-window conversion は禁止する。
4. 同フレームで `shape_count` の二次評価や `shape_count` 署名を導入しない。`shape_count` はレガシー証跡扱いとして `ast-lint` か実行時拒否へ。

### 2a. KUC public composited-artifact contract

KUC task `3.1g-14` must add an additive generic output owned by
`EguiTextCommandSurfaceAdapter`; the exact Rust identifier may differ, but the
contract is fixed as follows.

| Field | KUC requirement | Forbidden consumer behavior |
| --- | --- | --- |
| root identity | the actual root `UiRect` and a monotonically scoped frame identity from the one `show` call | passing an alternate canvas, using the previous frame, or synthesizing root bounds |
| layer provenance | ordered `Text`, optional `Toolbar`, optional `Search`, optional `Floating`, optional `ContextMenu` records, validated before output construction | reordering, omitting, inserting, or recovering a child in KLE/Storybook |
| final pixels | `ArtifactCompositeFrame` created within KUC from the retained root; dimensions, RGBA8 bytes, alpha invariant, and deterministic hash are stored together | exposing child paint plans as the normal consumer API, a second `ArtifactCompositor::compose`, or KLE RGBA-to-window conversion |
| semantic evidence | current `TextSurfaceFrameRecord`, command/context hit-action records, and the current AccessKit update associated with the pixels | mixing a record or AccessKit node from another frame, or accepting pixels without them |
| failures | missing/duplicate/out-of-order child, missing child artifact, invalid root, composition error, or record/pixel mismatch returns a typed KUC error before a frame is published | `expect`, fallback pixels, partial aggregate, warning-only recovery, or a KLE-generated substitute |

The current `EguiTextCommandSurfaceOutput::artifact_order()` and
`artifact_paint_plans()` are an intermediate KUC implementation detail. They
must become private to the KUC root-composition path once this contract is
available. `EguiKucArtifactAggregate`, `artifact_paint_plans()` in KLE, and
`StorybookRenderer::rgba_to_window_pixels` are explicitly migration residue.

### 3. RawInput 総合シナリオ要件（同一スクリプト）

KUC 公開パスの `RawInput` からのみ以下を検証する。少なくとも 1 トランザクション内で以下の最小セットを実行する。

1. テキスト系: 通常入力 `text("かな")` + `Ime(Preedit("⭐️"))` + `Ime(Commit("⭐️"))` + `text("⭐️")` + `text("\u{200d}")`（ZWJ）
2. カーソル/選択/フォーカス: クリック、矢印移動、`focus()`/`blur()`、外部クリック、Esc。
3. フローティングツールバー: KUC 注入のトリガ数は固定で **12**（inline 4 + heading 3 + list/quote 3 + code dropdown 1 + image-file 1）であることを確認。
4. Markdown command inventory/context menu: source closure が生成した **14** 種の opaque authoring target を 1 種ずつ押下し、同じ target/revision/correlation が host の `AppAction::AuthorMarkdown` effect へ到達することを確認。
5. コード種別ドロップダウン: source closure が生成した **17** 種類の opaque code-kind target を 1 種ずつ開いて選択し、KUC が同一フレーム内で同順 target event を返し、host が実際の code-kind effect を観測すること。
6. 画像系アクション: host 注入 `IngestImageFile` を含むイメージ系アクションを実行し、テキスト直挿入ではなく host リクエストへ変換されること。
7. Search strip: クエリ入力（`⭐️` 含む）→ previous/next ナビ→match option 変更→replace-one/replace-all→close。
8. 行番号/ガター: hovered 行・診断行活性化、行ジャンプ/スクロール要求、scroll token ack。
9. 読み取り専用: read-only true 時に編集系イベントが発火しないこと。
10. コンテキストメニュー: 右クリック/Shift+F10/アクセシビリティ action request で同一ルートからイベント変換されること。
11. 連続フレーム: 以上を 120〜240 frame の 1 脚本で行い、`motion` と `acceptance` で同一入力列を再生できること。

以上は「floating toolbar **12 トリガー** + **Markdown operation 14** + **code-kind 17** + 検索 + ガター + read-only + IME/emoji」を同一 RawInput 駆動証跡として扱う。

### 4. PNG / GIF / MP4 / manifest 生成とハッシュ整合

1. 1フレームごとに:
   - `show` 実行
   - 直近 KUC `EguiTextCommandSurfaceArtifactFrame` を取得
   - KUC frame の `root_bounds`、layer provenance、final `composited_rgba`、frame record の整合を検証
   - KUC が合成済みで返した RGBA の全ピクセル alpha を検証（全点 `alpha == 255` であることを要求）。満たされない場合は rejection とする。
   - KUC `composited_rgba` を `RGB24`（A は無視）に正規化して `canonical_rgb24` とし、フレーム番号付き PNG を保存する。
2. フレーム群を同順で GIF 化。
3. `Frame` 群は `canonical_rgb24` を共通入力として固定する。
4. GIF は同一順で再エンコード・再デコードして `frame_hash` を検証する。
5. MP4 は `rgb24` を保持できる可逆エンコード設定（可逆コンテナ設定を含む）で生成し、decode 後の `canonical_rgb24` byte 列を逐フレームで PNG の `canonical_rgb24_hash` と完全一致比較する。
   - エンコーダ/デコーダ初期化・設定取得・フレーム生成・シーケンス decode のいずれかに失敗した場合は acceptance 失敗。
   - `rgba/ARGB` の完全一致、または lossy トランスコード前提は不可。
6. manifest は必須キーを持つ:
   `output_png_path` 系列、`output_gif_path`、`output_mp4_path`、`width` `height` `frames`、
   `frame_checksums` / `frame_non_background_pixels` / `frame_pixel_hash` / `paint_plan_hash` / `compositor_frame_hash` / `motion_changed`、
   インタラクションイベントサマリ（起点/source/selection/action）、AccessKit ノード名サンプル。
7. MP4 生成・検証は `storybook-motion-artifact-gate` の同一制約として実装前提に組み込む。
8. PNG/GIF/MP4 の比較は `canonical_rgb24_hash` でのみ評価し、`canonical_rgb24_hash != PNG hash` または `manifest/hash` に不足がある場合は全件失敗。

### 5. 受け入れ時の拒否条件（合否に直接効く）

- fallback 参照: `StorybookFallbackRenderer`, `fallback_pixels`, `fallback_pixel` 系モジュールの実行パスが成果物生成に使われたら失敗。
- `StorybookContractRule` の `fallback-renderer` / `shape-count-acceptance` を同時に満たすこと。
- `minifb` は storybook の最終置換後、interactive 表示含めていかなる最終生成フローでも使用しない。`Window::update_with_buffer` が証跡や表示で採用されていれば失敗。
- 手動カーソル矩形、手動ガター行番号描画、`shape_count` を真の合格条件にすることは不可。
- `font_emoji_loaded` 等の既存 OS font fallback ランタイム証跡は、KUC composite 実行と一致しない場合は警告ではなく拒否。
- KUC `EguiTextCommandSurfaceArtifactFrame` 取得なし、または同一 `show` frame の final RGBA / root provenance / frame record が未更新での PNG 出力は失敗。

### 6. エラー/部分フレーム、determinism、AccessKit、callback の扱い

1. `EguiLanguageEditor::show` が失敗した場合、`EditorError::Internal` へ 1 回だけ変換し、ローカル fallback に逃がさない。失敗フレームは artifact 側では `frame_error` として明示記録し、成功フレームと混在しない行列を作らない。
2. `show` 成功でも `partial frame` は次のうちいずれかを欠いた/不整合の場合と定義する（KUC公開契約外の状態判定は参照しない）。
   - KUC composited frame の `root_bounds` が未収集または無効。
   - KUC composited frame の layer provenance が必須 child を欠く。
   - KUC final RGBA の dimensions / hash / root bounds が不整合。
   - AccessKit 記録または host callback（`EditorActionRequest` / `EditorEvent`）が不足。
   - 同一フレームで上記項目を欠損した場合は `partial_frame=true` とし、次フレームで継承しない。
3. 1 回の入力列で複数回実行した場合、`frame_checksum` と `compositor pixel/hash` は完全一致（決定性）であること。
4. 各フレームで AccessKit update を取得し、`MultilineTextInput`、ボタン、`TextInput`、検索入力、メニュー項目のラベル/ロールを検証。
5. コールバック（`EditorEvent::ActionRequested`、`EditorEvent::ContentChanged` など）はフレームごとに台帳化し、同じ入力で再生した場合に同一列を再現すること。

### 7. 削除チェックポイント

本設計では以下を「未使用化」対象とし、KUC/KUC-公開経路が検証されるまで削除しない。現行リポジトリが mixed worktree のため、明示承認が出る前の自動削除は禁止。

- `tools/kle-storybook/src/window_renderer_fallback*.rs` 系列
- `tools/kle-storybook/src/tests_motion.rs` 内の fallback 形状参照ケース（該当行）
- KLE 側互換 residue の `platform_text_surface.rs`, `line_gutter.rs`, `widget_gutter.rs`, `widget_text.rs`, `authoring_helper.rs` と同種のローカル再描画/測定ロジック
- `minifb` 依存（`tools/kle-storybook` の `Cargo.toml`）および `Minifb` renderer 実装（storybook の最終生成および interactive 表示の両方対象）

この節は実装ガードとして採用し、明示承認を受けるまでは「未使用であること」の証跡のみ確認する。

### KLE-owned actual KatanA host E2E (2026-08-13)

KatanA remains read-only in this scope. Therefore a gate requiring a new
`katana-ui` test file or a KLE dependency already committed into KatanA proves
adoption status, not KLE compatibility, and is invalid for this release scope.
The required replacement is a KLE-owned integration test that path-depends on
the local read-only `katana-ui` crate and drives two independent UI contexts:

1. KLE receives real `RawInput` and emits `EditorEvent` / `EditorActionRequest`.
2. A test-only adapter maps those typed values to public KatanA `AppAction`s;
   no egui type, widget, or renderer crosses this boundary.
3. The test instantiates real `KatanaApp`, invokes public `trigger_action`, and
   uses only KatanA's existing test hooks to assert actual buffer, dirty/save,
   authoring, search, diagnostics, and document state.
4. A pre/post read-only repository snapshot proves the test made no KatanA
   source or manifest changes. KatanA adoption tests remain a separate future
   gate that is enabled only after a KatanA-side change is explicitly approved.

The KLE test must run the real KatanA state path, not a simulator. Since KLE and
KatanA currently use different egui versions, each owns its own `egui::Context`;
the typed event/action stream is the sole cross-version boundary. The parity
checker must require the KLE E2E selector, pinned KatanA revision and manifest,
executed command, and snapshot result. It must not require a nonexistent KatanA
adapter file. This replaces an impossible adoption assertion with stronger,
runnable compatibility evidence without relaxing a feature requirement.

## Non-Negotiable Gates

- No KLE-specific authoring/search/text-surface/gutter/annotation component
  module survives.
- No KLE/KDV SVG rasterizer, SVG cache, OS font path, emoji segmentation,
  font lookup, generic text texture cache, generic pointer hit-test, or
  fallback renderer survives.
- `UiIconProps` originates from the host, not a KatanA enum in KUC/KLE.
- KUC component behavior is tested in KUC; KLE only tests binding/mapping; the
  Storybook only tests real composition and host callbacks.
- `StorybookFallbackRenderer` cannot be called by smoke, acceptance, motion,
  release, or screenshot evidence paths.
- The KatanA parity check fails for every source feature without a runnable
  KatanA evidence command.
