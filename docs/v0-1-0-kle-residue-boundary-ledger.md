# KLE v0.1.0 Consumer Residue Boundary Ledger

## Status

This is a source-observed migration ledger, not a completion claim. The listed
KLE implementation remains non-conforming until the required KUC root exists,
every listed legacy route is absent from compiled acceptance paths, and the
leaf evidence contract passes.

**Revision context:** KatanA is read-only at
`4f6a6287c650a38633c7baeb544a92e739c68567`; KLE is on `release/v0.1.0`.
No row authorizes a KatanA edit or makes KatanA application adoption a KLE
release prerequisite.

## Observed Violations

| KLE path and source observation | Why it violates the boundary or parity | Required destination | KLE end state |
| --- | --- | --- | --- |
| `crates/katana-language-editor-egui/src/platform_text_surface.rs:83-190` | Constructs `PlatformTextRasterizer`, owns texture allocation, root geometry, focus, selection and pointer routing. | KUC generic `TextSurface` inside the opaque retained root. | No local text rasterizer, texture, caret, selection geometry, or pointer routing. |
| `platform_text_surface.rs:149-159` | Sets `EventFilter.tab: true`. KatanA's plain `egui::TextEdit::multiline` does not set this filter; the locked egui default is `tab: false`. | KUC `TextSurface` focus policy parameter, instantiated for KatanA with non-capturing Tab behavior. | KLE passes a typed focus policy only; it cannot intercept Tab or implement indent/unindent. |
| `platform_text_surface.rs:930-1038` | Locally implements copy, cut, paste, ordinary text, IME commit, deletion, Enter, navigation, select-all and clipboard writes. This is a second generic input engine. | KUC generic retained text input and clipboard intent layer. KatanA-specific image ingest remains a typed host request. | KLE maps opaque KUC text/host events to its public event stream; it does not mutate `TextContent` from raw UI input. |
| `crates/katana-language-editor-egui/src/line_gutter.rs:41-93` together with `platform_text_surface.rs` paint/hit code | Computes and paints generic line rows from text, cursor, hover and diagnostics in KLE. The data and geometry are inseparable in the current path. | KUC retained gutter component. KLE provides typed diagnostic descriptors and accepts typed gutter events. | No KLE gutter layout, hit target, hover row, color or pixel calculation. |
| `crates/katana-language-editor-egui/src/kuc_artifact_aggregate.rs:16-155` | Publicly exposes each child artifact, root order and paint-plan borrowing, then rebuilds KUC layering in the consumer. | KUC only: internal child composition and one opaque composited frame record. | Delete the aggregate API and all consumer imports only after the KUC root replacement has equivalent actual-input coverage. |
| `crates/katana-language-editor-egui/src/widget.rs:57-76` | Destructures KUC child outputs and retains `latest_kuc_*` child output / artifact aggregate state. | KUC root returns one frame plus typed events. | Retain only immutable opaque-frame identity required for evidence and typed KLE domain-event mapping. |
| `crates/katana-language-editor/src/authoring.rs`, `authoring/lifecycle.rs`, `controls.rs::EditorAuthoringControl`, and `crates/katana-language-editor-egui/src/authoring_control.rs` | Stores string command/code-kind data, derives command IDs, retains toolbar/dropdown/focus state and cursor anchors, then emits a stringly authoring action. This duplicates host command semantics and KUC retained popup behavior. | KUC generic `CommandChrome` in the opaque root, driven by host-projected opaque item/authoring-target descriptors. | No KLE command string/catalogue, code-kind/info-string conversion, toolbar/dropdown lifecycle, cursor anchor, popup geometry or local authoring event mapping; only one opaque target/revision/correlation forward. |
| `crates/katana-language-editor-egui/src/widget_text.rs:1-34` | Retains the former direct local `PlatformTextSurface` path. | No replacement in KLE; the KUC root is the sole generic text path. | Remove after KUC actual-input tests cover its leaves. Do not hide it with module exclusion. |
| `kuc_command_chrome_*`, `kuc_context_menu_*`, `gutter_control_*`, any current/future preview side-rail module, and Storybook presentation/renderer modules | Names and current migration ledger show consumer-side composition, coordinate, artifact, fixture, rail/panel or hover responsibilities. Exact submodule treatment must be based on exported symbol usage, not filename alone. | KUC owns generic chrome/menu/geometry/artifact/PreviewSideRail work; KLE maps only opaque host targets. | Spark must remove every KLE `egui` form/popup/rail, child-plan, palette, coordinate, hit-test, hover clock, format/path/line conversion and fallback acceptance use after KUC replacement coverage exists. |

## Non-Negotiable Target API Boundary

The public KUC root input is generic and carries document text, selections,
diagnostic descriptors, generic tab/status/viewport descriptors, theme and
locale. It returns only:

1. one opaque composited frame identity and final raster/evidence record;
2. current KUC retained-state / AccessKit facts; and
3. typed generic events plus typed host-request events.

It must not expose child frames, paint plans, child ordering, text glyph
bounds, `egui` geometry, platform font handles, KDV types, Markdown parsing,
or KatanA concepts.

KLE may perform only the following work:

1. project KatanA-domain immutable state to generic KUC descriptors;
2. map KUC host-request events to typed `EditorActionRequest` values; and
3. join root facts to the source-derived leaf execution record.

KLE must not render, rasterize, lay out, hit-test, manage IME/clipboard, own
retained generic panel state, manipulate a KUC child frame, or locally emulate
a KatanA host side effect.

## Required Migration Order

1. **Spark Batch 1, KUC:** implement the opaque retained root, shared
   `PlatformFontCatalog`, generic TextSurface, gutter, host-target-driven CommandChrome, context
   menu, tab strip, status/diagnostics panel, workspace/preview viewport, and
   preview side rail.
   Each component must have KUC RawInput, retained-record, AccessKit and final
   pixel evidence. The exact isolated `STAR + VS16` test distinguishes `⭐️`
   from `☆` through this root.
2. **Spark Batch 2, KLE:** switch `EguiLanguageEditor::show` to the KUC root,
   map its typed events, and remove the aggregate and direct local text path in
   the same batch only after equivalent KUC coverage is green. KLE must pass
   the non-capturing Tab policy unchanged.
3. **Spark Batch 3, Storybook:** drive only that public KLE/KUC root in
   interactive, headless and motion modes. The encoder receives the opaque KUC
   frame; it cannot use a fallback renderer, minifb, `egui` shapes, child plans,
   coordinate reconstruction or fixture glyphs.
4. **Spark Batch 4, proof:** generate the canonical KatanA source closure and
   join every concrete leaf to KLE RawInput, KUC root facts and the correct
   effect class. Generic retained leaves require an unchanged bootstrapped
   KatanA observation; host-effect leaves require a real unchanged-source
   KatanA execution.

## Batch Acceptance and Deletion Gate

The following must all be true before Spark removes a legacy KLE path:

- The KUC public root exposes a typed replacement and its own strict tests
  cover every legacy leaf.
- A public `RawInput -> EguiLanguageEditor::show` test observes the KUC root,
  not a child output or local substitute.
- The canonical parity checker compiles and identifies the leaf by source span,
  KLE locator, KUC record/AccessKit locator and class-appropriate host proof.
- `just ast-lint` rejects the removed-path symbols from all compiled production
  and test acceptance paths without exclusions.
- Storybook's PNG, GIF and MP4 derive from the same final KUC frame sequence;
  decoded media hashes and retained records match each stage.

Until then, legacy source is retained only as an explicitly failing migration
residue. It is neither a release path nor evidence of KatanA parity.
