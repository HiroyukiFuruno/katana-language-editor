# KLE v0.1.0 Release Readiness

## Current Conclusion

As of 2026-09-02, KLE v0.1.0 is **not release-ready**. No release tag,
GitHub Release, crates.io publication, version bump, staging, commit, or push
is permitted while the local parity gates below remain open.

Historical baseline: As of 2026-08-14, KLE v0.1.0 is **not release-ready**.
The later local-gate progress recorded below does not change that release
conclusion while the fixed-host and three-profile blockers remain open.

Historical entries in this file that claimed a passing `release-check`, a
working Storybook motion artifact, cached font reuse in KLE, or a completed
KatanA downstream adapter are rejected as release evidence. They describe an
older partial/fallback path and do not satisfy the current full KatanA editor
parity, KUC-ownership, same-surface emoji, or actual-host-E2E requirements.

## Current Verified Facts

| Area | Current result | Evidence interpretation |
| --- | --- | --- |
| OpenSpec document consistency | `scripts/openspec validate v0-1-0-language-editor-extraction --strict` passes | document structure is valid only; it proves no implementation leaf |
| whitespace integrity | `git diff --check` passes | no whitespace error in the current KLE worktree; it proves no runtime behavior |
| KatanA reference | required fixed reference is `4f6a6287c650a38633c7baeb544a92e739c68567`; the live checkout is not evidence because it is at a different revision and contains unrelated local changes. A detached source tree produced the `macos-latest` capture on 2026-08-26. | KatanA remains read-only. Windows/Linux native captures, three-profile provenance/assembly, and actual KatanA host evidence are still required; the macOS source capture alone cannot satisfy any of them. |
| KatanA runtime adoption | on 2026-09-04, fixed-source `just katana-host-e2e-context-menu` passed 5 source-inventory tests and failed its host bridge test with `UnavailableBridge`: fixed `KatanaApp` exposes neither document bootstrap nor editor-target input, while editor/context-menu routing remains internal. | This is the expected pre-adoption state. KatanA #336 is a downstream task after final v0.1.0 publication. KLE must not fabricate a bridge or alter the read-only source. |
| source closure / parity checker | `cargo test -p katana-parity-check --locked -- --test-threads=1` passes 230 tests after RC/final mode separation. | `katana-parity-check --mode kle-release` requires source-derived KLE/KUC evidence and keeps every host result `downstream_required`; `--mode downstream-full` rejects that state until KatanA #336 records same-leaf host execution after publication. |
| AST lint | `just ast-lint` passes after splitting the KLE artifact relay and restoring strict Storybook contract boundaries | the rule still rejects KLE-owned text surfaces, presentation, selection fields and fallback codecs; test-only negative fixtures are excluded from production source scanning rather than accepted as production code |
| KUC generic quality gates | `katana-ui-core@0.3.5` is public as a single registry crate. Its Release workflow run `33804772291` completed successfully. | KLE resolves that public crate with the `egui` and `text-raster` features. KUC's cross-platform Unicode contract is dependency evidence, not KLE release evidence. |
| Storybook unit/contract suite | `just check` on 2026-09-04 ran the KUC v0.3.5 46-frame motion test to completion: 1 passed, 31 filtered, 461.55s. Its workspace suite also passed 361 tests. | KLE must not replace the required one continuous full-plan video with segmented clips or a KLE-local compositor. This is KUC-consumer evidence only; it is not full-editor Storybook coverage. |
| KUC root contract | partial: public variable-viewport writer consumed and verified; source-closure full-editor coverage pending | KUC owns the retained text-command root, raster/artifact writer and physical selection continuation. The public v0.3.5 writer created one continuous variable-viewport artifact with 46 source frames, 46 decoded frames, the `900x520` resize stage, `⭐️` scalar sequence `[U+2B50, U+FE0F]`, IME preedit/commit, hit test, and AccessKit evidence. This does not establish KLE full-editor parity. |
| KUC Unicode retained-root artifact | the public v0.3.5 contract covers exact `⭐️` / `☆` distinction, color pixels, IME preedit/commit, hit testing, and `MultilineTextInput` accessibility on macOS, Windows, and Linux CI. | this proves KUC generic-root behavior. KLE still needs current-version consumption evidence and all KLE gates before publication. |
| KLE platform text / emoji | invalid as parity evidence | the current KLE checks only code-point retention, an identifier string containing `⭐️`, `egui::FullOutput.shapes`, and aggregate paint plans. None asserts the final opaque KUC-root pixel glyph, shared catalog identity, or KatanA-host effect for Japanese IME, exact color `⭐️` VS16, or ZWJ behavior |
| actual KatanA host E2E | fixed KatanA cannot execute KLE before it adopts the published final crate. | KatanA #336 must run physical host E2E against the exact final registry dependency after publication. KLE/KUC must not create a fake bridge or edit the read-only reference. |
| source analysis | incomplete | direct editor, preview, document-tab, diagnostics, and editor-frame breadcrumb/source-address seeds are documented, but the required `syn`-derived closure/branch catalog has not been generated |
| current fail-closed parity gate | `just check` keeps the advisory `katana-parity-check --mode rc`; `release-check` invokes `kle-release-parity-check`. Both reject missing canonical source-closure artifacts; `kle-release` requires source-derived three-OS leaf binding and a published KUC multi-leaf producer. KUC #65 tracks that producer contract and KUC #66 tracks the current Linux color-emoji registry failure. `full-parity-check` remains a post-publication downstream host gate. | Local KUC/KLE test success cannot be promoted to release readiness until the three OS closure and leaf-correlated KUC evidence exist. |

## Historical Baseline Evidence

The following rows preserve the 2026-08-14 gate record. They are historical
failure evidence, not the current result of the commands above.

| Area | Historical result |
| --- | --- |
| source closure / parity checker | 2026-08-14 `cargo test -p katana-parity-check --locked` fails: `leaf_inventory_tests_rejections.rs` imports `leaf_inventory_entries` and `leaf_inventory_types` from an invalid sibling scope (`E0432`) |
| AST lint | `just ast-lint` fails in both `ast_linter_*` suites |
| KUC root contract | incomplete |
| KatanA downstream host E2E | required after final registry v0.1.0 publication |

## Invalid Artifacts

The previously named `target/acceptance/kle-storybook-motion-artifact.*`
files, counters such as `editor_egui_frame_rendered`, `font_emoji_loaded`, or
shape-count/jitter reports are not current v0.1.0 evidence. The same rejection
applies to the current KLE `colored_vs16` string predicate,
`egui::FullOutput.shapes` checks, and aggregate paint-plan checks. They may not
be shown as successful review material, included in a release checklist, or
used to justify publication because they can originate from a fallback renderer,
partial frame, local font path, Storybook-only state, or a glyph that is not
the requested VS16 color emoji.

The replacement artifact contract is defined in:

- `docs/v0-1-0-parity-manifest-schema.md`
- `docs/v0-1-0-editor-requirements.md`
- `docs/v0-1-0-kuc-text-command-root-design.md`
- `docs/v0-1-0-kuc-diagnostics-status-design.md`
- `docs/v0-1-0-external-host-e2e-design.md`

Only numbered PNGs, contact sheet, GIF, and MP4 produced from the same opaque
KUC root frames as the matched execution records may be review artifacts. They
remain supplementary: each stage also requires a source branch, public KLE
RawInput, KUC record/AccessKit proof and a class-appropriate declared effect.

## Release Preconditions

Publication work may begin only after all conditions below are passing in the
same source/KLE/KUC/generator fingerprint set:

1. The KUC release that supplies every KLE-resolved direct and transitive
   `katana-ui-core*` dependency is published first, each required crate version
   is visible on crates.io, and a clean external KLE package resolution selects
   exactly that KUC release set. `katana-ui-core@0.3.5` satisfies the registry
   availability condition and exposes the KUC #34 variable-viewport artifact
   API. KLE must not tag, create a GitHub Release, or publish either KLE crate
   before it verifies that exact resolved version.
2. KUC publishes and strictly tests the generic opaque TextSurface/CommandChrome/
   ContextMenu/TabStrip/BreadcrumbNavigator/SourceAddressBar/StatusStrip/
   DiagnosticsPanel/WorkspaceViewport/PreviewSideRail/OutlineNavigator root and
   shared platform font catalog.
3. KLE becomes an opaque consumer with no aggregate, local text/gutter/popup/
   glyph/compositor/fallback path.
4. The source-derived `syn` closure, action-origin catalog, decisive branch catalog and six joined
   manifests compile and fail closed for every mismatch.
5. Every source-derived and user-mandated leaf has a unique RawInput path,
   current KUC record/AccessKit node and a class-appropriate declared effect:
   host-effect leaves have a typed mapping and actual KatanA effect, while
   retained-UI leaves have a same-root transition and unchanged bootstrapped
   host observation. Required `Unsupported` outcomes are absent.
6. Native external image/clipboard leaves run unattended with the isolated
   driver, clipboard restoration and final KatanA asset/Markdown/no-mutation
   evidence.
7. Full-spec Storybook generates only KUC-root PNG/GIF/MP4 artifacts and
   validates their hashes, dimensions, freshness and decoded frame equality.
8. `cargo fmt`, warnings-denied clippy, workspace tests, strict AST lint,
   KUC contracts, parity checker, actual host E2E, Storybook artifact gate and
   coverage all pass without exclusions or relaxed assertions.
External publication checks are intentionally deferred. Passing a tag or
registry check before these local preconditions would not establish KatanA
editor parity.
