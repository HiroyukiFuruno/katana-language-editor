# KUC Preview Viewport Design

## Status

This is a pre-implementation boundary design for KLE v0.1.0. It is derived
from the read-only KatanA revision
`4f6a6287c650a38633c7baeb544a92e739c68567`; it is not implementation or
parity evidence. Spark may implement only after this document is reflected in
the source-derived closure, OpenSpec tasks, and KUC contract tests.

The KatanA lockfile resolves `katana-document-viewer 0.5.2` from crates.io
with checksum `d5e6e3cb1791b7a6ceb836f5bd932fcce5600b7e5c163b9637e503260b455eb5`
and `katana-render-runtime 0.4.15`. The closure's KDV API audit MUST use that
resolved registry source and record the package checksum plus the public API
file hashes. The local KDV checkout is currently workspace version `0.2.8` at
revision `5a970da619a4368fec1109b7f98fe22ed2444219`; it is a separate design and
artifact-discipline reference, not proof of the API that this KatanA executes.

## Measured Reference Boundary

KatanA has separate preview routes. They must not be reduced to a static
Markdown image or a single `preview` leaf.

| KatanA source | Observed role | Ownership consequence |
| --- | --- | --- |
| `views/app_frame/central_content.rs` | selects `CodeOnly`, `PreviewOnly`, or `SplitMode`; virtual documents have separate routes | The host projects an opaque generic view descriptor through KLE once. KUC owns reusable retained workspace composition; KLE does not map KatanA view values or requests. |
| `views/layout/{split,split_horizontal,split_vertical}.rs` | horizontal/vertical split, pane order, resizable preview pane and editor surface | KUC owns generic split geometry, hit testing, retained ratio, focus and AccessKit. KLE must not own an egui panel, ratio math or resize hit area. |
| `views/panels/preview/content.rs` | routes document surface, HTML browser, or Markdown sections; performs scroll/hover/task/floating action coordination | every route becomes an individual closure family. A KLE-local preview widget is prohibited. |
| `preview_pane/{ui,section,section_show/**}.rs` | Markdown section paint, anchor mapping, hover line spans, search highlighting and task actions | KatanA implementation is an oracle only. KLE does not copy its CommonMark/egui renderer. KUC consumes an already typed generic preview frame. |
| `preview_pane/document_surface/**` | bridges KDV `DocumentSession`/`DocumentFrame` to a KatanA egui painter | KDV remains the document-domain renderer. The host projects a generic opaque frame descriptor to the KUC viewport through KLE; KLE has no KDV DTO projection, geometry or raster code. |
| `preview_pane/image_html_surface*.rs` | bridges KDV browser session output and input to egui | KDV/KRR remains the browser producer. The host projects a generic opaque external-viewport descriptor to KUC through KLE; KLE has no typed frame/input/navigation adaptation. |
| `preview_pane/{fullscreen,slideshow/**}.rs` | fullscreen/slideshow lifecycle, OS fullscreen request and controls | generic overlay/focus/input belongs in KUC; KatanA layout/native viewport effects are classed individually and never simulated. |

The resolved KDV is UI-independent at the generic UI boundary. Its public
document/session/viewer and browser-session DTOs are rendered by a host UI;
KDV already depends on KUC. Therefore KUC MUST NOT depend on KDV: that would
create a circular dependency and place a document-domain renderer in the
generic UI crate.

## Required Ownership

```text
KDV/KRR: document, browser and renderer-domain source -> public typed frame/plan/event DTO
KatanA:  read-only reference source; unchanged host resolves KDV/KRR domain data
          into opaque generic host descriptors and performs host-owned effects
KLE:     transparent one-time descriptor/event transit only
KUC:     opaque editor-workspace root, split/preview geometry, retained UI state,
         focus, input, Accessibility tree, generic frame rasterization and artifacts
```

KLE is mechanical and non-rendering: it forwards one current opaque descriptor
or event correlation without reading its domain payload. It MUST NOT project
KDV DTOs, map stable IDs/text/semantic roles/source spans/commands, calculate
coordinates, clip regions, line anchors, glyphs, colors, scroll positions,
pixels, browser raster data, or Markdown semantics. An AST rule must reject
those operations outside KUC and the unchanged host.

## KUC Generic Contract

The existing opaque text-command root expands into an opaque workspace root.
Exact Rust names may vary, but its public contract must be equivalent to:

```rust
pub struct WorkspaceViewportInput {
    pub view_mode: WorkspaceViewMode,
    pub split: Option<SplitViewportInput>,
    pub editor: TextCommandSurfaceInput,
    pub preview: PreviewViewportInput,
}

pub struct SplitViewportInput {
    pub axis: SplitAxis,
    pub pane_order: PaneOrder,
    pub persisted_layout_key: UiStateId,
    pub min_preview_extent: LogicalPixels,
    pub preview_extent: LogicalPixels,
}

pub struct PreviewViewportInput {
    pub identity: UiStateId,
    pub revision: u64,
    pub content: PreviewFrameInput,
    pub search: PreviewSearchInput,
    pub scroll: PreviewScrollInput,
    pub capabilities: PreviewCapabilities,
}

pub enum PreviewFrameInput {
    Structured(StructuredPreviewFrame),
    Document(DocumentFrameInput),
    ExternalBrowser(ExternalBrowserFrameInput),
    Failure(PreviewFailureInput),
    Empty(PreviewEmptyInput),
}
```

These are KUC generic input types. They contain no KDV, KatanA, Markdown
parser, filesystem, linter, browser-runtime, or OS-clipboard type. KUC owns
all generic coordinates, retained scroll/resize/focus/overlay state, input
routing, visual layout, platform text rasterization, AccessKit output and the
same-frame artifact. `PreviewFrameInput` must be fully explicit: absent,
mixed-revision, unsupported, duplicate target, non-finite geometry, stale
command correlation, or capability-incompatible input is a typed KUC error,
not a consumer fallback.

`StructuredPreviewFrame` contains semantic node kind, text/span style, source
target, opaque media artifact reference and capability data only. KUC computes
all text layout, glyph selection, geometry, hover/anchor hit testing and
scrolling from that input through `PlatformFontCatalog`; it does not consume a
KDV `ViewerNode.rect` as a layout authority. KLE's mechanical KDV projection
must drop KDV geometry and preserve only semantic/source facts.

`PreviewSurfaceImage` is not a substitute for `StructuredPreviewFrame`. At the
resolved KDV `0.5.2`, `PreviewRenderEngine::attach_surface` calls
`KdvPreviewSurfaceFactory` and `SurfacePainter` to produce pre-rendered
Markdown RGBA. That route bypasses KUC's platform font/emoji/grapheme/hit-test
contract and is prohibited for Markdown editor preview text. A generic opaque
raster input is permitted only for an external-medium payload that KUC cannot
semantically render itself, such as a KDV PDF/document page, a diagram asset,
or a KRR browser frame. Its source/render-runtime identity, dimensions, pixel
hash, input correlation and accessibility alternative are mandatory; it never
counts as KUC text/emoji proof.

The root output remains one opaque frame. Its semantic record adds preview
viewport bounds, current view mode, split axis/order/extent, preview scroll and
anchor facts, source target IDs, current hover/focus/overlay state, typed
events, and AccessKit nodes. It must not expose KUC child paint plans or an
egui state object.

## Input And Effect Classification

| Family | KUC event/state | KLE binding | Required declared effect |
| --- | --- | --- | --- |
| CodeOnly/PreviewOnly/Split selection | generic view command and current root state | maps typed KatanA view request | `in_process_host_effect` when KatanA state changes |
| split axis/order | generic command and root state | maps typed KatanA request | `in_process_host_effect` |
| split resize/pane retained extent | KUC root state and persisted layout key | no KatanA action or direct state mutation | `kuc_retained_ui_effect` plus unchanged bootstrapped host observation |
| preview scroll, editor-preview sync, select-and-jump | typed generic scroll/target request | maps only host target/document request | source-specific class after closure; no KLE coordinate conversion |
| Markdown hover and task activation | generic target/hover/activation event | maps source target or `ToggleTaskList` request | host effect when KatanA data changes; retained effect for pure hover/focus |
| document-frame controls | generic capabilities and typed command event | projects KDV frame/command DTO only | KDV session transition plus class-appropriate host observation |
| HTML browser input/navigation | generic browser viewport event | forwards opaque correlated KDV browser request/result | `native_external_host_effect` when browser/native boundary is used |
| fullscreen/slideshow | KUC overlay/focus/navigation state | maps KatanA viewport/layout request only | `native_external_host_effect` for OS fullscreen; retained effect for pure overlay state |
| preview side-rail pinned/overlay panel, hover/cooldown/focus/empty content | generic `PreviewSideRail` root state | no host action for panel mechanics | `kuc_retained_ui_effect` plus unchanged bootstrapped host observation |
| preview side-rail refresh/search/export/story/tools/meta item | opaque target emitted once | forwards only opaque target/revision/correlation | source-specific host effect; search uses the actual KatanA UI input relay |
| TOC capability and selection | generic capability/opaque source target | no config/path/line conversion | capability/panel mechanics retained; selection follows the actual host scroll route |
| nested outline/accordion/active target | generic `OutlineNavigator` state and revisioned opaque items | no index/line/anchor/source-priority conversion | disclosure/expand/collapse/active scroll are retained; selected item and active target semantics follow source-derived host routes |

KLE MUST NOT maintain a parallel preview state, renderer cache, anchor map,
scroll cache, fullscreen state, browser frame, or slideshow control state. KDV
and KatanA domain state remain authoritative; KUC retained state is limited to
generic UI behavior and is keyed by stable host-provided identities/revisions.

## KDV Projection Gate

Before Spark writes a preview renderer, it must add a read-only audit of the
KDV `0.5.2` registry source resolved by the KatanA lockfile and compile-time
projection probes for all of these current public routes:

1. KDV `DocumentSession` / `DocumentFrame` and every surface/command/event
   variant KatanA currently uses.
2. KDV browser-session source, viewport, input, update, navigation and raster
   frame variants.
3. KDV Markdown preview/viewer plan and its source-span, search, task,
   anchor, image, diagram, HTML, table and error coverage.

The probe must prove that each KatanA-visible branch can be represented in a
KUC generic frame without KLE-local rendering. If a KDV public DTO lacks a
needed fact, the canonical source closure records a fail-closed blocker. KLE
may not use KatanA's egui preview code, a fixture image, or a smaller MVP
preview as a substitute. Editing KDV is outside this KLE/KUC implementation
scope unless the user explicitly opens a separate KDV change. A successful
projection against the local `0.2.8` checkout cannot substitute for the
KatanA-resolved `0.5.2` projection.

The Markdown projection probe must additionally prove that every textual node
is supplied to KUC as semantic data rather than KDV `PreviewSurfaceImage` or
`ViewerNode.rect`. It must compare source spans, commands, search targets,
task controls and all KatanA-visible Markdown variants against the structured
KUC input. Until this passes, Markdown preview is an open release blocker;
KLE/KUC must not silently use the KDV export surface.

## Required Source-Derived Leaves

The closure must generate at least separate leaves for:

- `view.code-only`, `view.preview-only`, `view.split-toggle`,
  `view.split-horizontal`, `view.split-vertical`, `view.pane-editor-first`,
  and `view.pane-preview-first`;
- `split.resize-horizontal`, `split.resize-vertical`,
  `split.layout-persistence`, `split.layout-ratchet-rejection`, and
  `split.no-active-document`;
- `preview.markdown-scroll`, `preview.editor-scroll-sync`,
  `preview.hover-source-range`, `preview.select-and-jump`,
  `preview.search-highlight`, `preview.task-toggle`, and
  `preview.back-to-top`;
- `preview.document-frame-open`, every KDV document command/capability branch,
  frame failure and worker failure;
- `preview.html-browser-open`, viewport resize, input, navigation, loading,
  failure and stale-frame branches;
- `preview.fullscreen-open`, `preview.fullscreen-close`,
  `preview.slideshow-open`, navigation, settings, close and OS-fullscreen
  restoration.

This is a minimum catalogue. The AST closure adds each source guard, match arm,
early return, dynamic callback, external boundary and test-oracle route as an
individual leaf. No aggregate `preview`, `split`, `document-frame`, `browser`,
or `slideshow` stage can pass the release gate.

## Automated Proof And Storybook

Every direct-root preview leaf uses the public
`RawInput -> EguiLanguageEditor::show` path and KUC opaque workspace frame.
KatanA-owned preview shortcut leaves instead record the physical unchanged
KatanA router together with same-run KUC focus/retained-input evidence and
must not claim KLE mounted causality. Every scenario records the exact KUC
root/AccessKit transition and a class-appropriate effect. The
full Storybook acceptance sequence includes Markdown, KDV document-frame and
KDV browser fixtures only when each fixture drives the real KDV public API;
pre-rendered bitmap fixtures are prohibited. KUC-generated numbered PNGs,
contact sheet, SHA-256 manifest, GIF and MP4 include each stage, but remain
review artifacts rather than correctness proof.

The strict gate rejects KLE/KUC local Markdown parsing, KLE local preview
rasterization, KUC-to-KDV dependency, direct KatanA `AppState` mutation,
host-state simulators, manual coordinates, fixed waits, omitted KDV variants,
or a preview stage without the joined source/KUC/AccessKit/effect evidence.
