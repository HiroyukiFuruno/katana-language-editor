# KLE v0.1.0 KUC UI Evaluation

## Scope

This note records the current KUC generic UI review for KLE v0.1.0. The release goal is to avoid KLE-specific fallback behavior for UI concerns that already belong in KUC.

## Findings

- KUC already provides a generic `TextArea` atom with type, IME composition, newline, tab, auto-grow, wrap, and resize contracts. KatanA's concrete projection must select the non-mutating Tab-focus policy because its fixed `TextEdit::multiline` caller does not use `code_editor()` or `lock_focus(true)`; a generic KUC indent capability must not be enabled merely because it exists.
- KUC Storybook already exercises an emoji + IME TextArea story through real `TextAreaAction` and `TextAreaKeyChord` events.
- KLE Storybook must consume the KUC retained `EguiTextCommandSurface` root. A KLE-owned `TextArea`, fallback renderer, or independently composed child artifact is not an acceptable path.
- The KUC output required by this release is one opaque composited frame containing root bounds, ordered layer provenance, final RGBA pixels, current text/command frame record, and AccessKit evidence. KLE and Storybook may record or encode this output but may not reconstruct it.
- Platform emoji text, including exact `⭐️` with VS16, remains KUC-owned glyph selection/raster behavior. KLE has no font fallback, glyph substitution, or RGBA-to-window conversion responsibility.
- KUC-side contract coverage must exercise text/IME, command chrome, context menu, gutter/annotation, focus/scroll, AccessKit, and final composited-frame provenance through actual RawInput. KLE validates only opaque one-time transit and unchanged frame consumption.
- System font discovery is also a KUC generic runtime concern. The current
  `PlatformTextRasterizer::new` creates a fresh `FontSystem` for each
  TextSurface, CommandChrome, and ContextMenu construction. A real KLE
  context-menu run sampled this path inside `fontdb::Database::load_system_fonts`.
  A process-shared KUC `PlatformFontCatalog` is therefore required; a KLE cache,
  Storybook cache, or test reduction would violate the ownership boundary.
- The current KUC `EguiTextCommandSurfaceOutput` publicly exposes the child
  text/toolbar/floating/search/context outputs and `artifact_order`. This is a
  measured opaque-root violation: child output and layer order must become
  private implementation detail behind one final root frame.
- KUC already has generic `StatusBar`, `DiagnosticsList`, and a structured
  `WorkspaceTabBar`, but no `CloseableTabStrip` identifier was found in the
  current KUC source. Spark must extract/reuse its generic retained tab-strip
  substrate under a domain-neutral `TabStrip` contract instead of claiming an
  alias exists or introducing a second KLE tab widget. Any remaining
  `Workspace*` nomenclature must not leak into the new generic public root
  contract.
- KUC already has generic `structured::SearchControlStrip` and egui command
  chrome search interaction/rendering paths. It already models query, result
  position, optional replacement input and current/all controls. Spark must
  extend/reuse it inside the opaque root with host-projected capability and
  revision boundaries; KLE must not create a second SearchStrip or retain
  query/replacement/match/range/replace state. KatanA-specific regex/match
  semantics and byte ranges remain outside this generic UI.
- KUC already has a generic `Breadcrumb` molecule
  (`molecule/selection/choice.rs:145`, `accessors.rs:80-85`), but its observed
  public state is only a string `crumb_action`. It is not yet the required
  retained editor-frame navigator: it lacks revisioned opaque descriptors,
  candidate menu lifecycle, typed events and same-frame AccessKit/root records.
  Spark must extend/reuse this molecule or its internal primitives; KLE must
  not create a second breadcrumb widget.
- No `AddressBar`, `UrlBar`, or `SourceBar` implementation was found under the
  current KUC core or egui-adapter source. KUC must add a generic
  `SourceAddressBar` that composes existing generic TextArea/choice primitives
  internally. Its API is opaque/revisioned text, history and host-status
  descriptors only. It must not acquire a KatanA/URL/filesystem/browser type or
  push that composition into KLE.
- No current generic KUC `WorkspaceViewport`/`PreviewViewport` root was found
  in the audited source. It must be introduced in KUC as a generic surface;
  KLE must not fill that absence with a local split, preview, scroll, raster or
  hit-test implementation.
- No current generic KUC `PreviewSideRail` was found. KatanA's visible preview
  rail and its Export/Story/Tools hover panels are reusable retained UI, so KUC
  must introduce it inside the same opaque root. Its public contract may carry
  only revisioned opaque item/panel/source-target descriptors plus generic
  presentation/capability facts; it must not name TOC, export, format, path,
  Markdown, KatanA, KDV, KRR, browser or slideshow domain types. KLE must not
  fill this absence with a local rail, panel, hover timer, overlay, geometry,
  TOC line conversion or host-state mutation.
- This does not justify a parallel KUC primitive. The current KUC
  `structured::CollapsiblePanel` already supplies generic leading/trailing,
  pinned/floating-overlay and hover expansion state; `disclosure::HoverCard` /
  `Popover` supply delayed overlay behavior; and generic `TreeView` /
  disclosure foundations supply hierarchy behavior. Spark must extend and
  compose these in KUC behind the opaque root. The file-named `FileTree` public
  API is not an admissible KLE surface, but its generic `TreeView` substrate may
  be generalized internally. No KatanA outline, TOC, path or document type may
  leak into the resulting KUC contract.

## Measured Current KUC Evidence (2026-08-14)

The following is a source audit of the current KUC working tree, not an
implementation-complete claim.

| Current source evidence | Measured fact | Required migration consequence |
| --- | --- | --- |
| `katana-ui-core-egui-adapter/src/text_command_surface/types.rs:110-145` | `EguiTextCommandSurfaceOutput` publicly exposes text, toolbar, floating, search and context-menu child outputs, and publishes `artifact_order`. | Replace the public child-output API with one opaque root-frame record. KLE and Storybook may not read child outputs or layer ordering. |
| `katana-ui-core-egui-adapter/src/text_command_surface/composition.rs:26-157` | The current root allocates only toolbar, text, and search rectangles, then separately invokes floating/context children. It has no retained tab/status/diagnostics/viewport/breadcrumb/source-address/preview-rail root model. | Extend the one KUC-owned root before consumer migration. KLE must not append a second layout tree, reserve child height, or use sequential `egui::Ui` composition to fill the missing editor frame. |
| `katana-ui-core-egui-adapter/src/text_command_surface/artifact.rs:30-76` | `artifact_paint_plans` reconstructs public child paint plans and uses `expect` for every present layer. | Keep compositor internals private and return typed root-level failures before a frame is emitted. No KLE artifact aggregation, shape counting or fallback renderer is admissible. |
| `katana-ui-core-text-raster/src/rasterizer.rs:13-48` | Each `PlatformTextRasterizer::new` allocates `FontSystem`, loads all candidates and derives emoji availability independently. | Introduce one KUC-owned, immutable `PlatformFontCatalog` per deterministic key; rasterizers retain only per-surface shaping/raster caches. KLE cannot cache fonts or select a fallback font. |
| `katana-ui-core/src/render_model/typed_text.rs:114-124` | `UiPlatformEmojiFontFamily::platform` resolves `Apple Color Emoji` only on macOS and returns `SansSerif` for every non-macOS target. | Replace this target-default with KUC-owned color-face discovery: `Apple Color Emoji` on macOS, `Segoe UI Emoji` on Windows, and a verified provisioned color face on Linux. Persist only immutable family/face/SHA evidence in the catalog; absent color face fails that release profile. |
| `katana-ui-core-text-raster/src/tests.rs:23-46` | The exact `⭐️` test establishes one grapheme and hit range only. | Retain this test and add an isolated VS16 color-glyph crop/hash test through the opaque root and AccessKit path. |
| `katana-ui-core-text-raster/src/tests.rs:270-285` | The only macOS color assertion rasterizes aggregate `🔥⭐️` and counts chromatic pixels. It cannot prove that the `⭐️` crop, rather than `🔥`, is colored or differs from `☆`. | Replace the aggregate acceptance criterion with independent `⭐️` versus `☆` crops, exact source bytes (`U+2B50 U+FE0F`), non-monochrome pixels in the `⭐️` crop, and an inequality assertion against the text-star control. |
| `katana-ui-core/src/molecule/structured/search_control_strip/actions.rs:17-105` | Search and replacement already have reusable retained actions/events, including query, next/previous and one/all replacement. | Reuse it internally in the opaque root. Query and replacement text are root-local user input; KLE forwards one closed host transport and never holds search options, matches, ranges or byte conversions. |
| `katana-ui-core/src/molecule/structured/tree_view_hit_test_types.rs:1-31` | Generic tree hit testing currently publishes coordinate-bearing input and string node IDs. | Use it only behind `OutlineNavigator`; the root must own geometry and expose revisioned opaque targets/events. KLE must not receive coordinates, outline indexes, anchors or source lines. |
| `katana-ui-core/src/molecule/selection/choice.rs:139-145` | `Breadcrumb` exists only as a choice molecule. | Generalize its internal visual primitive into the required root-owned `BreadcrumbNavigator`; its existing string action is insufficient as the editor-frame contract. |

The evidence establishes reusable KUC substrate, not a release-ready generic
root. Every required editor-frame child remains incomplete until the opaque
root, shared font catalog and generated leaf-level tests exist in KUC.

## Current Decision

- A local KUC edit is required: the retained text-command root must publish the opaque composited frame described above. The existing KLE `EguiKucArtifactAggregate` and Storybook `rgba_to_window_pixels` approach violates this boundary and is migration residue.
- KLE must keep KUC integration only at the egui binding and tool/host consumption boundary. It must not retain generic composition state, child-layer ordering, pixel conversion, or fallback text rendering.
- KatanA-specific names or markdown-specific behavior must not be added to KUC.
- KUC must generalize its existing `Breadcrumb` before exposing
  `BreadcrumbNavigator`; it must construct the new `SourceAddressBar` inside
  KUC from existing generic text/choice primitives. Both are root-owned,
  revisioned retained components with typed events and AccessKit evidence, not
  standalone KLE forms.
- KUC must introduce `PreviewSideRail` as another root-owned revisioned
  component. Its generic open/close/hover/focus mechanics have retained
  root/AccessKit evidence; an opaque target is forwarded only when the current
  host descriptor declares a host effect.
- The shared KUC font catalog may share only immutable platform discovery.
  Per-surface shaping, raster, texture, selection, and state caches must remain
  isolated, and the catalog contract must preserve Japanese IME, ZWJ, and exact
  non-monochrome `⭐️` VS16 behavior.

## Verification

- `rtk just kuc-contract-check`
- `rtk just storybook-contract-check`
- `rtk just ast-lint`
- `rtk cargo test -p kle-storybook --locked`
- `rtk cargo test -p katana-ui-core theme::tests::theme_uses_katana_colors_and_font_roles --locked`
- `rtk cargo test -p katana-ui-core window::coordinate_tests::scaled_surface_point_maps_back_to_canvas_point --locked`
- `rtk cargo test -p katana-ui-core molecule::structured::file_tree_hit_tests::hit_target_returns_file_row_contract --locked`
- `rtk cargo test -p katana-ui-core-storybook visual_interaction_text_area_state_tests --locked`
- KUC catalog instrumentation: independent/concurrent TextSurface,
  CommandChrome, and ContextMenu roots must prove one system-font discovery per
  catalog while retaining independent cache state and deterministic glyph pixels.
- KUC `BreadcrumbNavigator` tests: virtual/non-navigable, no-workspace,
  prefix menu open/close, candidate select, stale revision, overflow and
  pointer/keyboard/AccessKit paths using opaque IDs only.
- KUC `SourceAddressBar` tests: controlled text/preedit, button/Enter submit,
  blank no-host-event, empty/nonempty history, history select, stale revision,
  Japanese/`⭐️` VS16 raster/AccessKit and independent-root retained state. No
  fixture may validate an URL, build a path, make a network request or render a
  browser frame.
