# KLE v0.1.0 Spark Execution Plan

## Status

This is the implementation handoff plan, not an implementation-complete
report.  The release remains blocked by incomplete KatanA source coverage,
KUC root-artifact ownership, KLE host E2E, Storybook, and strict verification.
All code changes in the batches below are delegated to Spark.  The orchestrator
owns this plan, source audit, review, and integration decision.

## Invariants

1. KatanA at `4f6a6287c650a38633c7baeb544a92e739c68567` is reference-only.
   No KatanA file, test, branch, or worktree is changed.
2. KUC owns all generic text, IME, grapheme/glyph raster, `⭐️` VS16 behavior,
   cursor/selection/hit testing, gutter, generic TabStrip, CommandChrome,
   ContextMenu, generic StatusStrip/DiagnosticsPanel/diff preview, root
   geometry, root artifact order, final RGBA composition, and AccessKit output.
3. KLE neutral remains free of KUC/egui types. The KLE egui crate is a
   transparent consumer of the retained KUC root: it forwards only opaque
   host-issued envelopes and single-consumption transports. It creates no
   editor-domain presentation/action mapping, command catalogue, document
   state, layout, rendering, or host-resolution type.
4. KLE and Storybook never compose child paint plans, convert RGBA into window
   pixels, raster text/icons, infer commands from labels, calculate popup/line
   geometry, or retain a fallback surface.
5. A screenshot, GIF, MP4, KatanA source test, fixture, simulator, or shared
   selector is never leaf-completion evidence by itself.

## Batch 1: KUC Generic Retained Root

**Repository:** `/Users/hiroyuki_furuno/works/private/katana-ui-core`  
**Permitted scope:** generic KUC contracts and actual-egui tests only.  No
KatanA/KLE names, Markdown semantics, host filesystem/clipboard acquisition, or
KLE fixture behavior.

### Required implementation

1. Extend the retained root in
   `crates/katana-ui-core-egui-adapter/src/text_command_surface/` so one
   successful `EguiTextCommandSurfaceAdapter::show` publishes one immutable,
   opaque composited frame.
2. Build that frame inside KUC from the actual root allocated by `show`, using
   the KUC-owned child order.  KUC must validate the required child artifact
   before calling its own `ArtifactCompositor`; no `expect`, partial result, or
   consumer repair is allowed.
3. The public frame must contain these same-frame facts together: root identity
   and bounds; validated ordered layer provenance; final `ArtifactCompositeFrame`
   (RGBA8, dimensions, pixel hash, plan hash); `TextSurfaceFrameRecord` and
   command/context records; current AccessKit evidence; and typed error when
   any fact cannot be made consistent.
4. `EguiTextCommandSurfaceOutput::artifact_order()` and
   `artifact_paint_plans()` become KUC-internal composition details.  Consumers
   receive the opaque composited frame, not child plans.
5. KUC provides the generic validation needed for non-zero root, duplicate or
   out-of-order layer, layer without artifact, dimension/hash mismatch, alpha
   mismatch, and stale/mixed frame record.  The root adapter must return a typed
   error before publishing a frame in each failure case.
6. KUC must introduce a generic process-shared `PlatformFontCatalog` (or an
   equivalent explicitly owned catalog) for platform font discovery. The
   current `PlatformTextRasterizer::new` calls `FontSystem::new`, and every
   TextSurface, CommandChrome, and ContextMenu construction repeats system-font
   discovery. Per-surface text layout, glyph, raster, texture, and selection
   caches remain isolated; only immutable platform discovery may be shared.
   KLE and Storybook must never own this cache.
   The catalog must resolve and evidence an actual color-emoji face on every
   release profile: `Apple Color Emoji` on macOS, `Segoe UI Emoji` on Windows,
   and a verified KUC-provisioned Linux face. Family, face identity, and loaded
   file SHA-256 are KUC-only immutable evidence. `SansSerif`, a missing face,
   a target skip, or any KLE/KDV fallback fails the batch.
7. KUC must extract/generalize the reusable substrate in its existing
   structured `WorkspaceTabBar` as the generic retained `TabStrip` within the
   same opaque full-editor root. No current `CloseableTabStrip` alias may be
   assumed. Its public root contract must not leak the current
   `Workspace*` naming or any KatanA/filesystem type. It accepts non-KatanA
   descriptors and produces typed tab/menu/drag/AccessKit events. KLE must not
   render, order, hit-test, or retain a local document-tab strip. Current
   document actions remain host-owned.
8. KUC must integrate and extend its existing generic `StatusBar` and
   `DiagnosticsList` as retained `StatusStrip` and `DiagnosticsPanel` root
   children, rather than creating duplicates. They accept generic status,
   activity, diagnostic, scope, fix-preview and capability descriptors and emit
   typed status/diagnostic events. KLE must not render, filter, group, scope,
   preview, batch, lay out or retain a local Problems/status surface. Linter
   semantics, document selection, fix payloads, browser URLs and all host
   effects remain KatanA-owned.
9. KUC must provide generic retained `BreadcrumbNavigator` and
   `SourceAddressBar` children for the editor-frame routes in
   `docs/v0-1-0-katana-editor-source-universe.md`. They accept only opaque
   revisioned segment/candidate/history/status descriptors and emit typed
   generic events. KUC owns text focus/preedit/menu/overflow/layout/AccessKit;
   it must not accept paths, URLs, filesystem trees, response payloads, KatanA,
   KDV, KRR or browser types. KLE forwards an opaque selected target once and
   transports a submitted value once without inspecting it;
   all validation, filesystem/network acquisition, history, document and
   browser effects remain host-owned. KUC must extend/reuse its current generic
   `Breadcrumb` molecule rather than create a parallel breadcrumb, and create
   `SourceAddressBar` inside KUC from its generic TextArea/choice primitives;
   neither implementation may be composed in KLE.
10. KUC must provide generic retained `WorkspaceViewport` / `SplitViewport` /
   `PreviewViewport` within the same opaque root. It owns generic split and
   preview layout, resize, scroll, focus, overlays, AccessKit, artifact and
   typed events, but has no KDV/KatanA/Markdown/parser/browser dependency.
   KUC must accept explicit generic structured/document/browser/failure/empty
   frame inputs and fail typed on stale/mixed/unsupported inputs. KUC-to-KDV is
   forbidden because KDV already depends on KUC. The exact boundary is
   `docs/v0-1-0-kuc-preview-viewport-design.md`.
   KDV `PreviewSurfaceImage` is not allowed for Markdown text: Spark must
   project semantic KDV viewer nodes/spans/source targets and let KUC perform
   all text layout/raster/hit testing. An opaque raster is limited to an
   external document/PDF/media/diagram/browser surface with source/runtime
   fingerprint and accessibility alternative.
11. KUC `CommandChrome` must accept host-projected opaque item/group/authoring
    targets with revision and correlation, and emit the same opaque target on
    physical activation. It owns all floating-toolbar/dropdown/anchor/focus/
    dismissal state. It must not accept or derive Markdown operation, code-kind,
    command-ID, shortcut or KatanA data. This replaces, rather than wraps, KLE
    `EditorAuthoringCommand`, `EditorCodeBlockKind`, and
    `EditorAuthoringMenuState`.
    Its search/replace child must extend/reuse KUC `SearchControlStrip` and the
    current command-chrome search adapter within the opaque root, not introduce
    a second search UI in KLE or retain raw query/replacement values there.
12. KUC must provide generic retained `PreviewSideRail` within the opaque root.
    It accepts only revisioned opaque item/panel/source-target descriptors and
    presentation/capability data, while it owns rail/panel geometry, hover
    delay, sibling exclusion, focus, dismissal, AccessKit and artifact. KLE
    must not implement a rail, popup, timer, format/path/line conversion or
    host-state mutation. Rail mechanics are KUC retained proof; opaque targets
    reach the unchanged KatanA host route or actual UI input relay.
    Spark must extend/compose KUC's existing `CollapsiblePanel`, `HoverCard` /
    `Popover`, and generic `TreeView`/disclosure substrate rather than add a
    parallel panel, hover or hierarchy engine. The file-specific `FileTree`
    public API is not the root contract.

### Current KUC root gap to replace

At the current KUC worktree state,
`text_command_surface/types.rs::EguiTextCommandSurfaceOutput` exposes
`root_bounds`, optional child outputs, and `artifact_order`, while
`text_command_surface/artifact.rs::artifact_paint_plans()` publicly reconstructs
borrowed child plans with `expect`. `composition.rs` computes the order but does
not publish a final root frame. This is not a consumer-safe artifact contract.

Spark must make the output contain one named opaque frame field produced by
the root adapter on the same `show` call. The frame must use the actual
`root_bounds` canvas and KUC's generic `ArtifactCompositor` internally. Missing
child artifacts, a duplicate child, or a child not present in the computed
order must produce an `EguiTextCommandSurfaceError` variant rather than panic.
`artifact_order` and child paint-plan access become private implementation
details. KUC test support may inspect them only through a KUC-internal test API;
neither KLE nor a Storybook consumer may import them.

The current KUC performance defect is independently measured: every
`PlatformTextRasterizer::new` makes a new `cosmic_text::FontSystem`
(`katana-ui-core-text-raster/src/rasterizer.rs:24-48`), while command chrome
constructs both a TextSurface adapter and another direct rasterizer
(`katana-ui-core-egui-adapter/src/command_chrome.rs:72-79`) and ContextMenu
constructs another (`context_menu/adapter.rs:31-38`). A sampled real KLE
context-menu case spent its active work in `fontdb::Database::load_system_fonts`
from this construction path. Treat that as a KUC shared-runtime defect; do not
move a font cache into KLE or lower test coverage to mask it.

### Required automated proof

- Actual RawInput tests under
  `crates/katana-ui-core-egui-adapter/tests/text_command_surface/` exercise
  text only; text plus toolbar/search; selection floating chrome; every context
  route; and all layers together.  Every result proves KUC-owned order and
  final RGBA, not only a child output.
- Japanese IME preedit/commit, exact `⭐️` + VS16, and a ZWJ sequence must be
  present in the KUC frame record and final texture/pixel evidence on macOS,
  Windows, and Linux. `⭐️` must be proven as an isolated non-monochrome
  grapheme crop using the profile's resolved color face, and its deterministic
  crop hash/pixels must differ from a same-config `☆` control. A mixed-emoji
  aggregate chromatic count is not evidence; `☆`, `SansSerif`, a missing color
  face, or a text substitution is a rejection.
- Pointer, keyboard, and AccessKit routes for text, tab selection/close/
  reorder/pin/group, command chrome, context menu, gutter marker, focus return,
  and scroll acknowledgement run through the same retained root.
- The same root runs status severity/dirty/activity, Problems toggle/close,
  stable scope selection, expand/collapse, file disclosure, empty state,
  location jump, docs, single/file/visible-scope fix request and preview
  branches. Every stale, missing-key or disabled capability case returns a
  typed KUC error or no-op event as specified by current input; it never falls
  back to a KLE fixture.
- KUC tests include deterministic repeated-frame hash equality and failures for
  every invalid root-artifact state.  Test configuration may not skip a platform
  case or weaken an assertion.
- KUC exposes test-only catalog instrumentation that proves a process creates
  one system-font discovery for a shared catalog across independent TextSurface,
  CommandChrome, and ContextMenu roots, including concurrent construction. The
  test must also prove cache isolation, deterministic pixels, Japanese IME,
  exact colored `⭐️` VS16, and no stale font state after root disposal. A wall
  clock threshold is not an acceptance oracle; repeated system enumeration is.
The catalog identity, root injection, cache-isolation, error, and no-direct-
`FontSystem::new` rules are specified in
`docs/v0-1-0-kuc-font-catalog-design.md`.
The opaque output shape, private child-plan boundary, same-frame composition
algorithm, typed errors, and KLE/Storybook prohibitions are specified in
`docs/v0-1-0-kuc-text-command-root-design.md`.
The generic StatusStrip/DiagnosticsPanel input, event, root-record and test
contract is specified in `docs/v0-1-0-kuc-diagnostics-status-design.md`.
The generic preview/split input, KDV projection boundary, effect classes and
strict test contract are specified in
`docs/v0-1-0-kuc-preview-viewport-design.md`.

## Batch 2: KLE Consumer-Only Migration

**Repository:** `/Users/hiroyuki_furuno/works/private/katana-language-editor`  
**Precondition:** Batch 1 passes its KUC tests and exposes the composited-frame
API.

### Required implementation

1. Change `EguiLanguageEditor::show` and its retained state to store/forward
   the opaque KUC composited frame. It forwards a KUC event only as its opaque
   target/revision/correlation or single-consumption transport; it does not map
   an event to an editor-domain action or presentation value, inspect child
   plans, or make a layout or rendering choice.
2. Remove KLE root-composition residue in
   `crates/katana-language-editor-egui/src/kuc_artifact_aggregate.rs`, its API,
   its test support, and all KLE uses of `artifact_order()` / `artifact_paint_plans()`.
   Removal happens only in the same Spark batch after the public KUC replacement
   has actual RawInput coverage; it is not replaced by a compatibility wrapper.
3. Remove the disconnected local text/gutter/authoring/fallback renderer paths
   only after the public KUC path has equivalent actual-input coverage.  Do not
   hide them via compilation exclusion, `allow`, test-only code, or source move.
4. Keep host-owned save, format, raw clipboard/image acquisition, filesystem,
   preview, linter, and shortcut routing outside KLE. KLE forwards only the
   current opaque correlation or permitted single-consumption transport; it
   must not introduce a host request/action type or implementation for any of
   them.
5. Remove every release-path `EditorError::Unsupported` that stands in for a
   KatanA source-derived or user-mandated editor capability.  This includes the
   current MVP wording and unsupported-control expectations; their replacement
   must be a real KUC/KLE/host path with strict proof, never a passing test that
   accepts `Unsupported`.
6. Replace the local `search_control.rs` replace/replace-all content mutation
   with KUC generic search presentation, an opaque one-time KLE transit, and
   the host `AppAction::ReplaceText` route. The host-E2E runner must drain the complete
   KLE public event stream, not only `EditorActionRequest`: normal user content
   goes through KatanA `UpdateBuffer`; host-external changes flow only back to
   KLE; every clipboard/action/cursor/selection/diagnostic/save event has an
   explicit leaf rule. Storybook counters and the downstream simulator cannot
   supply this effect evidence.
7. Forward host-issued opaque descriptors and only KUC opaque diagnostic
   events once. KUC
   retained Problems open/close, scope and disclosure events stay inside the
   current opaque KUC root and require an unchanged-host observation, not a
   KatanA action or direct `AppState` mutation. Remove every KLE/Storybook-local
   Problems/status/panel/list/scope/preview/fix-batch implementation after KUC
   root coverage exists. KLE does not compare scope labels, calculate preview
   rows, acquire a linter payload, choose a document path or invoke a browser.
8. Remove KLE `EditorAuthoringCommand`, `EditorCodeBlockKind`,
   `EditorAuthoringMenuState`, `EditorAuthoringMenuLifecycle`, their string
   request controls, and every KLE code-kind/command-ID derivation. The only
   authoring route is `KUC RawInput -> opaque target/revision/correlation ->
   KLE one-time forward -> unchanged KatanA host projection lookup ->
   AppAction::AuthorMarkdown`. KLE does not mirror `MarkdownAuthoringOp`.

### Required automated proof

- Public `RawInput -> EguiLanguageEditor::show` tests for direct-root leaves
  prove the KUC frame is replaced each current frame, unchanged in
  order/pixels/provenance, and carries Japanese/IME/`⭐️` VS16/ZWJ, selection,
  read-only, gutter, toolbar, search, context, status and Problems interactions
  to closed neutral relay envelopes. KatanA-owned shortcut leaves use the
  separate physical-router execution mode and same-run KUC focus evidence.
- AST rules reject `EguiKucArtifactAggregate`, `artifact_paint_plans`, direct
  `ArtifactCompositor` use, local RGBA/window conversion, local glyph/gutter
  renderer, fallback names, and direct egui form/popup construction in KLE.
- Tests validate actual KUC record and AccessKit facts, not a fixture count or
  source string.
- Event-bridge tests cover ordinary typing, Japanese IME, `⭐️` VS16, authoring,
  host external change, text/image paste, replace current/all, read-only,
  stale/document-switch, and duplicate-event rejection. They prove each KatanA
  effect once and reject a silently dropped event or locally completed replace.
- Authoring tests generate the current KatanA 14-operation/17-code-kind target
  inventory from source closure, prove KUC RawInput/AccessKit activation for
  each item, reject unknown/stale targets, and prove the exact host action and
  transformed document/cursor result. A KLE command string, enum duplication,
  or local menu lifecycle is an AST failure.

### Mandatory AST migration scope

The current AST failure is architectural evidence. Spark must not mechanically
split its files to satisfy line limits:

- Replace `platform_text_surface.rs` rather than preserving it in smaller
  modules. Remove `kuc_artifact_aggregate*.rs` only after Batch 1's opaque KUC
  frame is consumed. Break `direct_text_surface_actual_input_context_menu.rs`
  and `direct_text_surface_integration_tests.rs` into leaf-scoped KUC-root
  scenarios, without coordinate/action synthesis.
- Move Storybook's `window_renderer.rs`, `window_renderer_fallback.rs`,
  `window.rs`, `tests_motion.rs`, and all shape-count/minifb/fallback paths to
  Batch 3's KUC-frame encoder. `storybook_command_chrome_presentation.rs` must
  lose its local palette and geometry in favor of KUC presentation data.
- Rebuild `leaf_inventory_*` around the generated closure and explicit leaf
  schema, then repair its imports and cohesive test modules. No static
  authoring-only manifest remains as a compatibility layer.
- Only after the ownership migration, apply ordinary focused refactors for
  remaining independent AST items such as `diagnostics/visual.rs`,
  `localization.rs`, and `clipboard_paste_control.rs`; use named constants and
  approved comment forms where those modules continue to exist.

## Batch 3: Full-Spec Storybook And Review Media

**Repository:** `/Users/hiroyuki_furuno/works/private/katana-language-editor`  
**Precondition:** Batch 2 passes.

### Required implementation

1. `tools/kle-storybook` runs the same retained public KLE/KUC root for
   interactive, headless, and motion modes.  Its renderer reads only the KUC
   composited-frame RGBA; it does not call `ArtifactCompositor`, use `minifb`,
   derive window pixels, inspect `egui::FullOutput::shapes`, or render fallback
   glyphs/layout.
2. Implement one source-derived full editor scenario manifest.  Each stage
   declares unique leaf/step id, exact RawInput, KUC current record/hit/action/
   AccessKit fact, composited-frame hash, KLE typed callback, and actual KatanA
   host effect locator.
3. Generate numbered PNGs, contact sheet, JSON manifest, SHA-256 checksums,
   deterministic GIF, and MP4 from the same KUC RGBA sequence.  Decode MP4 and
   compare every canonical RGB24 frame hash to its PNG counterpart.

### Current Storybook failure to replace

The current test command runs 22 tests: 12 pass and 10 fail because the
contract reads an active/hovered diagnostic gutter fact without first producing
the required current public `show` frame. Its window renderer then directly
composes child plans, converts RGBA to window pixels, and retains fallback and
shape-count paths. Spark must replace this entire route with a source-derived
scenario that drives one public KLE/KUC root frame per step and asserts that
same frame's KUC record, AccessKit, typed callback, and opaque pixels. The
missing diagnostic state must be generated by actual RawInput and KUC output,
not patched into a fixture.

### Required coverage

- Actual type/edit, Japanese IME, `⭐️` VS16, ZWJ, caret/selection/delete,
  read-only, line numbers, diagnostic marker/popup/fix paths.
- Floating toolbar: all 12 triggers, selection/focus/outside/clamp/dropdown
  lifecycle; context menu: Save, conditional Format, all 14 authoring actions,
  all 17 code kinds, file and clipboard image actions, secondary/keyboard/
  AccessKit routes.
- Status and Problems: status severity/dirty/activity, count toggle and close,
  OpenTabs/ActiveTab scope, expand/collapse, file disclosure, empty state,
  location jump, docs, entry/file/visible-scope fix requests, preview and
  disabled/stale branches through the same opaque KUC root.
- Preview and split: CodeOnly/PreviewOnly/Split, axis/order, resize/ratchet,
  scroll/hover/source target/task activation, KDV document-frame command and
  failure variants, KDV/KRR browser input/navigation/failure variants, and
  fullscreen/slideshow lifecycle. Every stage must use a real KDV public API
  projection, never an egui copy or a pre-rendered fixture bitmap.
  Markdown stages additionally prove KUC `PlatformFontCatalog` owns Japanese,
  exact `⭐️` VS16 and ZWJ text layout/raster/hit testing; a KDV export-surface
  RGBA image cannot satisfy these stages.
- Preview side rail: TOC capability/toggle/pinned-leading/pinned-trailing,
  hover-open/dismiss/cooldown/empty-source, hierarchy leaf/parent selection,
  accordion disclosure, expand/collapse-all, active-anchor priority/fallback,
  auto-scroll/vertical-guide/empty-outline, refresh/search, panel
  open/close/hover/sibling/outside/Escape, HTML/PDF/PNG/JPG export, story and
  tools controls, metadata absent/present. Every stage uses the same opaque KUC
  root and class-appropriate retained or actual-host evidence.
- Search/navigation and visible user-mandated replace/replace-all; content
  refresh, dirty/save/format/undo; document/view/split/scroll; shortcut
  arbitration; clipboard/image host request outcomes.

## Batch 4: Source-Derived Parity And Actual KatanA Host E2E

**Repository:** `/Users/hiroyuki_furuno/works/private/katana-language-editor`  
**Precondition:** Batches 1-3 compile and have their direct tests.

### Required implementation

1. Replace `tools/katana-parity-check/src/source_inventory_repo.rs` static
   `INVENTORY_FILES` and the nonexistent deferred
   `kle_downstream_adapter.rs` exception with the generated closure in
   `docs/v0-1-0-katana-editor-source-universe.md`.  Use Rust AST parsing, not
   line-token scanning, and record file SHA-256, incoming edge, `path:symbol`,
   branch span, leaf/helper rationale, and reference revision.
   Implement the six joined manifests in
   `docs/v0-1-0-parity-manifest-schema.md`; every consumer must validate the
   identical source, KLE, KUC, and generator fingerprints before it accepts an
   artifact.  Do not add static count, shared-selector, or aggregate-leaf
   compatibility paths.
2. Traverse UI-host, action-dispatch/handler, data-refresh, generic-presentation,
   and test-oracle edges.  An unresolved edge is a release failure until its
   continuation or host-owned boundary is explicitly recorded.
   Implement the six-phase module/item index, conservative call resolution,
   branch catalog, and proof join specified in
   `docs/v0-1-0-source-closure-generator-design.md`. Dynamic, macro, callback,
   trait, cfg, and ambiguous method routes must remain fail-closed rather than
   being skipped by the parser.
3. Replace the 48 authoring-only leaf manifest and its shared selector/template
   fields with unique source-derived leaves.  Every leaf requires exact KatanA
   branch, KLE public RawInput locator, KUC current record plus AccessKit fact,
   and executed KLE-owned actual KatanA host E2E effect.
4. The KLE-owned host E2E path-depends on read-only local KatanA. It must
   bootstrap each physical workspace/document/view through unchanged public
   KatanA `OpenWorkspace`, document-selection and required view/search actions
   before the public KLE RawInput trace; direct `AppState` document/index/view
   mutation and `app_state_mut()` setup are forbidden. It must exercise real
   KatanA document/state/file behavior without adding a KatanA adapter or test
   source. A simulator remains supplemental only.
5. Replace the current partial action bridge before accepting any host-E2E
   result. `UnsupportedAction` and `ExternalHostIntentUnavailable` cannot be
   expected outcomes for source-derived or user-mandated leaves. Every bridge
   call consumes an `EditorActionRequest` captured from the exact public KLE
   RawInput leaf, rather than a test-constructed request.
6. Implement the isolated macOS native external driver specified in
   `docs/v0-1-0-external-host-e2e-design.md` for file-picker, raw-image,
   file-list, and percent-encoded file-URL clipboard leaves. It operates real
   KatanA host facilities through accessibility/pasteboard APIs but introduces
   no product code in KLE, KUC, or KatanA. Missing authorization, clipboard
   restore failure, a skipped native case, or missing same-run asset/link/dirty
   effect fails the release gate.

### Mandatory checker replacement details

The current checker is known-invalid input, not a starting point to preserve:

- `source_inventory_repo.rs` hard-codes `INVENTORY_FILES`, deliberately
  defers the absent `kle_downstream_adapter.rs`, and extracts function names by
  splitting source lines.  Replace all three behaviors. Use `syn`'s parsed
  file/item/statement/expression tree to build the branch and call catalog;
  line-based token parsing is prohibited.
- Add a generated closure record with reference revision, canonical relative
  path, SHA-256, incoming edge kind/source/target symbols, source spans,
  resolved target classification, and a leaf or helper/host-boundary rationale.
  Recompute it on every parity invocation. A record for a file that is missing,
  unresolvable, stale, or does not agree with the reference revision fails.
- Start traversal from the two direct directories plus the documented seeds.
  Traverse editor mounting; action definition -> dispatch arm -> handler ->
  final observable state/file/external boundary; data refresh; generic
  presentation; and KatanA test-oracle registrations. At a trait, macro, or
  external boundary, emit a named unresolved item; only an explicit
  host-boundary rationale may close it. Never treat `syn` parse success or a
  type declaration as a resolved behavior route.
- Incorporate the audit-discovered continuations: `state/scroll.rs`,
  `state/command_inventory/view_commands.rs`, `views/panels/preview/logic.rs`,
  `app/action/process_linter.rs`, `app/document_contract.rs`, and the
  documented search/document/diff-review test-oracle files. This is a floor,
  not a replacement for generated traversal.
- Incorporate the KDV/KRR preview continuation from
  `docs/v0-1-0-kuc-preview-viewport-design.md`: central-content mount,
  split/PreviewOnly, preview content early routes, Markdown sections,
  document-frame worker/commands/failures, browser session/input/navigation,
  fullscreen and slideshow. Record the exact KatanA `Cargo.lock` resolved KDV
  package/API symbol/version/checksum/file hash and fail when a KatanA-visible
  branch has no non-rendering KLE-to-KUC projection. The current local KDV
  `0.2.8` checkout is artifact-discipline reference only and cannot replace
  this `0.5.2` API probe. KUC must never add a KDV dependency and KLE must
  never copy KatanA's egui preview renderer. The Markdown branch must reject
  KDV `PreviewSurfaceImage` and `ViewerNode.rect` as a layout source, preserve
  only semantic node/span/source target data, and prove KUC platform
  text-raster/emoji/hit-test ownership.
- Remove every KatanA adapter patch/evidence path from the compiled checker
  graph (`actual_repo_adapter*.rs`, `katana_adapter_patch.rs`, and release-gate
  paths that accept them). Do not delete retained legacy source before the
  deletion checkpoint; it must be unreferenced and rejected by release checks.
- Repair the immediate E0432 test-module failure before semantic work. The
  rejection tests must import from the correct `leaf_inventory` module scope,
  then add regression tests that fail on static directories/files, an absent
  source, a deferred source, a token-parser false positive, a macro/dynamic
  unresolved edge, and a stale hash/revision.
- Replace `REQUIRED_LEAF_COUNT = 48`, shared selectors, template fields such
  as `state_condition = "always"`, and a single category harness. Each
  generated leaf must have a unique source branch, state predicate, visible or
  non-visual route, typed KLE request/state, public RawInput locator, KUC frame
  record and AccessKit node assertion, and executed KLE-owned host-E2E locator
   with exact effect assertion. A leaf may not be green merely because its
   feature group is green.
- Make host-E2E artifacts record their class (`in_process_host_effect` or
  `native_external_host_effect`) and reject a class mismatch. Native records
  include KatanA revision/source hashes, KLE input trace hash, KUC frame hash,
  AccessKit node ID, native accessibility/pasteboard trace, pre/post host
  record, and exact effect locator. The current ignored KatanA live clipboard
  test and headless pending-dialog state are diagnostic references only.

### Required release condition

`cargo fmt`, clippy with warnings denied, full workspace tests, current AST
lint, KUC contracts, source-closure/leaf parity checker, actual KatanA host E2E,
and Storybook artifact/media verification all pass with zero open leaf,
fallback, source-closure, or host-effect gap.  No test exclusion, coverage
reduction, static selector reuse, or manual validation may satisfy a failure.

## Spark Handoff Discipline

- Each Spark invocation receives exactly one batch or independently reviewable
  sub-batch, allowed files, prohibited files, and the checks above.
- The KUC worktree is already dirty.  Spark first records its target-file diff,
  changes only assigned files, and never reverts unrelated changes.
- The orchestrator reviews the resulting diff against this plan before the next
  batch.  No commit, push, release, or KatanA modification is part of this plan.
