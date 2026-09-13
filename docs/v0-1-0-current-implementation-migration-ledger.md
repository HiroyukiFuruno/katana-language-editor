# Current Implementation Migration Ledger

## Status

This is a current-state migration inventory. Every row below is incomplete and
is not release evidence. Spark must implement replacements in the owning
repository before removing the residue. KatanA remains read-only.

The latest measured `just ast-lint` run on 2026-08-14 fails both
`ast_linter_*` suites. This is an intended fail-closed outcome, not a reason
to exclude paths: it currently reports Storybook fallback/minifb/shape-count
bypass, KLE-owned text raster and aggregate composition, consumer-side egui
input, local palettes, hidden-error helpers, and responsibility-boundary
violations. The exact current implementation residue below is the migration
scope for Spark; it is not a list of allowed exceptions.

The failure must be fixed by replacing/removing the wrong owner, not by making
the existing paths cosmetically pass. The same run reports: local neutral
diagnostics/localization and clipboard helpers beyond the file/function limits;
KLE egui `platform_text_surface` and context-menu aggregation; public
aggregate-paint reconstruction; direct egui input; an injected color literal;
hidden-error methods; Storybook `StorybookFallbackRenderer`, `MinifbFallback`,
and `shape_count` acceptance; and parity-check files above the responsibility
limit. Spark may split a replacement by cohesive responsibility, but must not
split, rename, exclude, feature-gate, or suppress a legacy local renderer so
that the old acceptance path remains compiled.

### Storybook Compile Audit (2026-08-21)

`cargo check -p kle-storybook --locked` is currently a failure, not a
temporary runnable fallback: the manifest lacks the `eframe`, `egui`, and
`katana-ui-core-egui-adapter` dependencies used by the source, and `main.rs`
does not declare several modules it imports through the old window/motion
path. No `interactive`, smoke, GIF, screenshot, or motion output from this
tree is admissible evidence until a single KUC-full-root implementation
replaces that graph.

The compiled/source residue also includes `MinifbFallback`,
`StorybookFallbackRenderer`, fixture-derived window rasterization,
Storybook-side `ArtifactCompositor::compose`, shape-count acceptance, and
fixed-ID/coordinate action construction. The existing Storybook lint rule
finds string patterns only; it does not prove module/dependency reachability.
The replacement must be one atomic KUC-frame capture/encoding path with
numbered PNG, contact sheet, GIF, MP4, decoded-MP4-to-PNG frame-hash equality,
per-leaf manifest, KUC record/AccessKit provenance, and a strict dependency
and AST gate that rejects every fallback path.

### Measured AST Gate Snapshot (2026-08-14)

`rtk proxy just ast-lint` exited `101`; both `ast_linter_*` suites failed. The
full unmodified runner log is
`~/Library/Application Support/rtk/tee/1786717236_cargo_test.log`. This is a
release blocker. The following facts refine the migration order and are not
independent cleanup tasks:

| Failure class | Current measured locations | Required resolution |
| --- | --- | --- |
| local generic text/composition | `platform_text_surface.rs` is 1262 lines; `direct_text_surface_actual_input_context_menu.rs` is 837; `kuc_artifact_aggregate_test_support.rs` is 301; `kuc_artifact_aggregate_tests.rs` is 257 | replace the KLE text, gutter, context-menu and aggregate renderer with the KUC opaque root, then remove the compiled legacy route. Splitting the old renderer without changing ownership is insufficient. |
| hidden failures and palette ownership | `clipboard.rs` has `expect`; `platform_text_surface.rs` has `expect`, `unwrap_or_default`, and a prohibited color literal | KUC root returns typed errors and injected generic theme tokens. Do not add fallbacks, `expect` wrappers, color literals, or lint exceptions in KLE. |
| Storybook false evidence | `motion_artifact*.rs`, `motion_manifest*.rs`, `window*.rs`, and `tests_motion*.rs` retain `shape_count`, `StorybookFallbackRenderer`, or `MinifbFallback` | replace every compiled/test route with a KUC opaque-root frame encoder. PNG/GIF/MP4 acceptance joins decoded frame hashes to KUC frame/record/AccessKit evidence. |
| consumer-owned command surface | `kuc_context_menu_tests.rs` has an egui editor-input module; `kuc_artifact_aggregate.rs` reconstructs context-menu plans | move input, geometry, command lifecycle and artifact composition to KUC `EguiTextCommandSurface`; KLE retains no context-menu adapter/plan. |
| residual responsibility length | `diagnostics/visual.rs`, `localization.rs`, `clipboard_paste_control.rs`, direct-input tests, Storybook presentation, and parity-check modules exceed strict limits | split only the replacement implementation by cohesive owner responsibility. Each module must stay under file/function limits while retaining all strict leaves and no disabled rule. |
| comments | `host_runtime_helpers_acceptance.rs:80` and `motion_manifest_lines.rs:61` use prohibited `//` comments | correct only after those files have been replaced or retained for a valid responsibility; comment conversion cannot be reported as an AST-gate fix by itself. |

### Measured Storybook Gutter Gate (2026-08-14)

The unmodified command `rtk proxy env CARGO_TARGET_DIR=/tmp/kle-storybook-verify-20260814-r2 cargo test -p kle-storybook --locked -- --test-threads=1` exited `101`: 22 tests ran, 12 passed, and 10 failed with `storybook contract expected active hovered diagnostic gutter line`. This blocks the Storybook contract, live-acceptance artifact, and all motion artifact/jitter tests.

The immediate measured cause is stale-frame observation: `StorybookGutterContract::setup_gutter_fixture` changes content, cursor, diagnostics and hovered rows directly on `EguiLanguageEditor`, then `verified_gutter_lines` calls `gutter_lines`. That method reads only `latest_kuc_frame`; no public `show` call has produced a frame for the changed state. The assertion therefore reads an earlier frame, not the configured fixture.

Spark must not repair this by adding a local post-mutation `show`, relaxing the assertion, or accepting a callback counter. That would retain the forbidden KLE-owned editor fixture and would still not be an opaque KUC root proof. The replacement is the KUC-root RawInput scenario: it supplies the current host descriptor, produces one current opaque frame/record/AccessKit snapshot, drives the gutter target through the same root, and joins the resulting event to the class-appropriate KatanA effect. Until this replacement passes, no current PNG, GIF, MP4, screenshot, or interactive window from this Storybook is an admissible review artifact.

| Current location | Measured problem | Required owner and replacement | Removal/acceptance condition |
| --- | --- | --- | --- |
| KUC `text_command_surface/types.rs`, `artifact.rs`, `composition.rs` | public child order and `artifact_paint_plans()` expose plans; `expect` reconstructs child data; no final root frame | KUC retained root publishes one opaque composited frame with current records/AccessKit and typed validation errors | all KUC root RawInput, invalid-artifact, exact emoji/IME, and catalog tests pass; child plans become internal |
| KUC `molecule/selection/{choice,accessors}.rs` `Breadcrumb`; no current KUC `SourceAddressBar` | the existing breadcrumb has only a string `crumb_action` and lacks revisioned opaque descriptors, candidate-menu lifecycle, typed events and same-root evidence; no generic address/source component exists | generalize/reuse KUC `Breadcrumb` as retained `BreadcrumbNavigator`; add root-owned generic `SourceAddressBar` by composing KUC TextArea/choice primitives internally | pointer/keyboard/AccessKit/current-frame tests cover virtual/no-workspace/prefix/candidate/stale/overflow and source text/preedit/button/Enter/blank/history paths; KUC/KLE never parse paths/URLs, acquire payloads or make network/browser calls |
| no current KUC `PreviewSideRail` | KatanA's preview rail has generic retained buttons, tooltip/focus, overlays, hover-delay, sibling exclusion and dismiss mechanics, but KUC has no reusable owner | add root-owned generic `PreviewSideRail` with opaque revisioned item/panel/source-target/capability descriptors and typed events | KUC RawInput/AccessKit/opaque-frame tests cover availability, open/close/hover-delay/sibling/outside/Escape and opaque target correlation. KLE has no rail/panel/hover clock/format/path/line/state implementation. |
| KLE `kuc_artifact_aggregate.rs`, `_api.rs`, `_test_support.rs`, `_tests.rs`, `kuc_text_surface_binding.rs`, `widget_state.rs` | KLE retains child artifacts/order and reconstructs paint plans | KLE keeps only the immutable opaque KUC root frame plus one-time opaque host descriptor/event transit; it does not map a KatanA domain value | no production/test use of `EguiKucArtifactAggregate`, `artifact_order`, or `artifact_paint_plans`; AST gate passes |
| KLE `widget.rs:7-28`, `kuc_text_surface_binding.rs:33-63,94-155` | every `show` builds a KLE request from content, selections, cursor, diagnostics and hovered gutter rows, adds KLE chrome/context-menu child bindings, then maps child outputs and `EguiKucArtifactAggregate` | KUC full retained root receives only opaque host descriptors and owns generic text/chrome/menu/frame state; KLE forwards one opaque descriptor/event without semantic content, range, cursor, diagnostic, gutter, style or child artifact access | KUC-root continuous RawInput proof shows unchanged prefix glyph/hit bounds, gutter geometry, root bounds and catalog fingerprint remain identical while typing; caret is the same-frame KUC authoritative bound on every macOS/Windows/Linux trace, with no KLE child output or frame reconstruction |
| KLE `platform_text_surface.rs` and its tests | local `PlatformTextRasterizer`, texture, caret/IME/glyph/gutter renderer remains in source | KUC TextSurface retained root consumes all generic platform text state | KUC equivalent leaf evidence exists; file is removed at approved deletion checkpoint, not hidden by module exclusion |
| KLE `direct_text_surface_actual_input_context_menu.rs`, Storybook `tests_kuc_surface.rs` and `window_renderer.rs` | `⭐️` is accepted by code-point/string predicates, `egui::FullOutput.shapes`, or KLE aggregate plans; none proves final color glyph pixels and several paths breach the opaque-root boundary | KUC retained root, using its shared font catalog, emits final RGBA plus grapheme/record/AccessKit evidence; KLE forwards only that frame and host event | every macOS/Windows/Linux final-root test distinguishes `⭐️` from `☆`, proves an isolated color glyph with KUC-resolved face SHA-256, retains Japanese IME/ZWJ/caret semantics and joins it to KLE and KatanA host evidence; no `shapes`, aggregate, `SansSerif`, or identifier predicate remains |
| KUC `render_model/typed_text.rs:114-124` | the current resolver chooses `Apple Color Emoji` only on macOS and `SansSerif` elsewhere | replace it with KUC `PlatformFontCatalog` color-face discovery and CI provisioning for every release profile; KLE/KDV cannot override it | macOS `Apple Color Emoji`, Windows `Segoe UI Emoji`, and a verified Linux color face each produce the exact isolated `⭐️` VS16 versus `☆` crop proof; absent family/bytes/profile artifact fails release |
| KUC `.github/workflows/test-and-build.yml:62-117` | unit tests use macOS/Ubuntu/Windows, but `storybook-check` and AST lint are currently skipped on Windows | extend the KUC-owned release proof so the opaque-root exact-emoji artifact and strict profile manifest run on all three profiles; generic lint tooling differences do not exempt runtime proof | each profile produces and validates its own KUC-root RGBA/AccessKit/`⭐️` versus `☆` artifact; Windows cannot inherit macOS/Linux evidence |
| KLE direct geometry test/scenario files including `kuc_context_menu_test_geometry.rs` and `gutter_control_raw_input_scenario.rs` | consumer-side coordinate/hit behavior risks duplicating generic KUC geometry | KUC root RawInput scenarios own current records, bounds, scroll, hit target, and AccessKit validation | scenarios become leaf-scoped KUC-root tests without coordinate/action synthesis; AST gate has no violation |
| KLE `search_control.rs` | local `replace`, `replace_all`, `replace_char_range`, content event completion, and regex `Unsupported` remain | extend/reuse KUC `structured::SearchControlStrip` and command-chrome search adapter inside the opaque root; it emits a single-consumption query/replacement/current-match intent. KLE forwards opaque revision/correlation only; the host derives the exact character/byte range against its snapshot, then unchanged KatanA executes `ReplaceText` | current/all/no-match/read-only/stale/document switch/dirty/undo/scroll host E2E passes, host-only byte-boundary conversion is proved with Japanese/VS16/ZWJ fixtures, and `Unsupported` cannot accept a required leaf |
| KLE `actions.rs`, `events.rs`, `navigation.rs`, `search.rs` | public actions/events persist or serialize arbitrary command/authoring strings, documentation/file URLs, diagnostic IDs, content/origin/query strings, line/segment/progress/range/anchor values, and KUC-retained Problems controls | KUC owns generic input/geometry/retained state. KLE has only closed opaque host requests plus single-consumption document/search/replacement/viewport/source-address/group-name transport; the host resolves all paths, ranges, linter data, URLs and state | AST rejects `HostCommand`, URL/file-list payloads, persistent raw input/query/origin/coordinate state, generic string actions and KLE line/range conversion. Every transport joins one current KUC root event to an exact KatanA effect/no-mutation record without raw artifact serialization |
| KLE `authoring.rs`, `authoring/lifecycle.rs`, `controls.rs::EditorAuthoringControl`, and egui `authoring_control.rs` | public command/code-kind strings, shortcut/info-string derivation, cursor-anchored menu state, lifecycle booleans and request-by-string recreate both KatanA command semantics and KUC generic popup behavior in KLE | KUC generic `CommandChrome` receives host-projected opaque command-item/authoring-target descriptors and owns toolbar/dropdown/anchor/focus/dismissal. KLE forwards the current opaque target/revision/correlation once; the unchanged KatanA host resolves it to `AppAction::AuthorMarkdown` | AST rejects all listed KLE authoring symbols, string target derivation and KLE menu/cursor state. Generated source closure proves all 14 Markdown operations and all 17 code kinds individually from KUC RawInput/AccessKit through actual KatanA effects |
| Storybook `window_renderer.rs` | directly calls `ArtifactCompositor`, reads aggregate child plans, and converts RGBA to window pixels | Storybook encodes only the same opaque KUC root RGBA returned by public `show` | no direct compositor/plan/RGBA-window conversion; current record/AccessKit/execution record joins every media stage |
| Storybook `window_renderer_fallback.rs`, `window.rs`, `tests_motion.rs`, motion helper files | fallback/minifb/shape-count paths can create non-editor pixels or acceptance facts | KUC-frame encoder and source-derived `FullEditorScenarioManifest` | no fallback symbol or shape-count acceptance in compiled or test path; PNG/GIF/MP4 decoded hashes match KUC frames |
| Storybook `storybook_command_chrome_presentation.rs` | local palette/geometry presentation can become a second generic UI layer | KUC generic presentation data; KLE only binds opaque host command items | KLE owns no generic palette/layout/button rendering and leaves all current KUC component records intact |
| Storybook `host_runtime_helpers.rs`, `window_renderer_fallback*.rs`, `window.rs`, `motion_artifact*.rs`, `motion_manifest*.rs`, `tests_motion*.rs` | current acceptance accepts `EditorError::Unsupported`, `StorybookFallbackRenderer`, `MinifbFallback`, or `shape_count`; it can therefore report pixels and motion without an opaque KUC editor frame | KUC full-editor root frame encoder, KLE domain-only scenario binding, and source-derived stage manifest | AST has no fallback/shape-count/`Unsupported` acceptance symbol in a compiled/test path; every decoded PNG/GIF/MP4 frame hashes to the corresponding KUC root record and source-derived leaf |
| KLE `platform_text_surface.rs`, `direct_text_surface_actual_input_context_menu.rs`, `kuc_artifact_aggregate*.rs` | current strict lint reports files at 1262, 837 and 301 lines respectively, plus local generic text/composition violations | split responsibility at the KUC-owned TextSurface/CommandChrome root boundary; KLE files contain typed bindings and host-action projection only | no local text raster, palette, coordinate, hit-test, child-plan or fallback symbol; all split modules remain below the strict file/function limits without lint exclusions |
| KLE `tools/katana-parity-check/src/leaf_inventory_{types,validation,tests_rejections}.rs` | current strict lint reports 204/214/235-line files and the checker has an unresolved sibling import (`E0432`) | generated `syn` closure modules split by source-index, graph resolution, manifest schema, validation, and rejection test responsibility | checker compiles; every module meets lint limits and accepts only canonical generated artifacts |
| `tools/katana-host-e2e/src/action_bridge/*`, `tests/command_chrome_input.rs`, `tests/actual_host.rs` | `UnsupportedAction` / `ExternalHostIntentUnavailable` and expected error cases leave required effects unexecuted | source-derived total event/action bridge plus isolated native external driver | every public KLE event/action reaches one actual KatanA effect or explicit no-mutation proof; no required error result is accepted |
| `tools/katana-host-e2e/src/host.rs:185-188` | `ActionBridge::to_app_action` is followed by `app.trigger_action(action)` and one frame; this is direct action injection rather than a source-derived KatanA UI route | replace the action bridge with correlation-only opaque transit plus a semantic dynamic physical KatanA UI input driver; KatanA must naturally emit its action/handler/frame | AST rejects `trigger_action`, `pending_action`, direct `AppAction`, action-map unit tests, fixed coordinates and raw replay without a current semantic target; each leaf records the physical source-input span and naturally emitted effect |
| `tools/katana-host-e2e/src/host.rs:77-86` | the RawInput runner calls KatanA UI but discards `FullOutput` after clearing texture deltas, so no current KatanA AccessKit target, frame hash, role/name hash, or bounds is available | return an evidence-only same-frame snapshot that permits semantic dynamic target discovery before the physical event and records only required hashes | missing/duplicate/stale/disabled/non-AccessKit target is fail-closed; KLE never supplies coordinates, semantic input, or action payload |
| `tools/katana-host-e2e/tests/file_url_paste_actual_host.rs:41-75,84-125` | image/non-image file URL tests start from `with_markdown_editor_document` and assert `pending_action_for_test()` before any final host effect | replace fixture bootstrap with source-derived physical workspace/document routes and assert the final document, asset, Markdown, dirty, explorer and no-mutation effects in the same execution | fixture state setup and pending-action acceptance are AST/E2E rejected as release evidence; retained test coverage must use the physical target record and final effect locator |
| `tools/katana-host-e2e/src/host.rs` | fixture pushes documents into `AppState`, assigns active document and uses `app_state_mut()` to set view state before a KLE trace | physical workspace/document bootstrap through unchanged public `OpenWorkspace`, document-selection, view/split/search actions and observable KatanA frames; read-only hook observation only after the effect | AST rejects direct document list/index or mutable host-state setup; each host-E2E execution record includes its KatanA bootstrap action/frame chain before the KLE input trace |
| KatanA `Cargo.toml`, `views/app_frame/central_content.rs`, `views/layout/{split_horizontal,split_vertical}.rs` | KatanA neither depends on KLE nor mounts it: all three paths directly construct legacy `EditorContent::new` | KatanA remains read-only. KLE/KUC real-root evidence and a separate read-only KatanA host-effect bridge are the v0.1.0 parity proof; retain the mount fact in the canonical manifest | no execution record may claim an actual KatanA rendered-editor frame from KLE until a separately approved KatanA adoption change is made. This does not replace or weaken the required source/KLE-KUC/host-effect proof quartet for the KLE release. |
| `tools/katana-parity-check/src/source_inventory_repo.rs`, `leaf_inventory*` | static/deferred inventory, shared evidence templates, current E0432 compile failure, and no backward action-origin classification prevent reliable parity assessment. `LeafEvidenceKind` accepts only KLE RawInput/source/Storybook/simulator modes, while validation requires every leaf's `public_show_*` selector, so it cannot represent KatanA-owned shortcut routing without a false KLE mounted-causality claim. | `syn`-based source closure, action origins, branch catalog, six joined manifests, and branch-specific proof join with `kle_kuc_root` versus `katana_shortcut_router` input origin | compile passes; no static count/deferred absent path/shared selector/default state/effect; action definition/handler alone cannot create a KLE leaf; direct-root and shortcut execution records are individually validated; mutation and mismatch tests fail closed |
| `tools/katana-parity-check/src/actual_repo_adapter*.rs`, `katana_adapter_patch.rs`, release-gate references | compiled historical KatanA adapter/patch evidence conflicts with KatanA read-only host-E2E design | KLE-owned host E2E uses local read-only `katana-ui` path dependency with no KatanA patch | legacy path is removed only after actual host E2E and source closure pass; it is never used as compatibility evidence |

## Atomic Migration Order

1. KUC opaque root and platform font catalog with strict KUC evidence.
2. KUC retained BreadcrumbNavigator, SourceAddressBar, PreviewSideRail, and
   complete TabStrip with strict generic evidence, followed by KLE's closed opaque host mapping
   and actual KatanA document-tab/SelectDocument/OpenUrl evidence. All raw
   user input crosses only through non-persistent single-consumption transport;
   `HostCommand`, action URL fields, arbitrary command strings, clipboard
   file-URL payloads, and KLE input/query/coordinate state are removal targets.
3. KLE consumer-only frame and event bindings, including removal of aggregate and
   local generic surface paths.
4. Total KatanA host bridge, host-owned replace, and native external effects.
5. Generated KatanA closure/leaf proof join.
6. Full-spec Storybook frame encoding and media evidence.

Each stage rejects its old path before the next stage is accepted. Physical
deletion follows the approved deletion checkpoint only after the replacement
compiles and its strict tests pass. No source exclusion, feature flag, fallback,
or compatibility wrapper may make a residue appear complete.
