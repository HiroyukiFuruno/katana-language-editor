# v0.1.0 Context Menu Host E2E Design

## Status

This document fixes the design boundary for the Context Menu host-E2E work.
It is a design and acceptance contract, not execution evidence.  The fixed
KatanA source remains read-only at revision
`4f6a6287c650a38633c7baeb544a92e739c68567`.

## Problem

`just katana-parity-check` currently fails before the parity matrix because
`tools/katana-host-e2e/tests/context_menu_input.rs` does not exist.  Creating
a test that only enumerates actions, injects a KLE callback, or matches source
markers would make that command green without proving the editor.  That is
prohibited.

## Ownership

| Owner | Responsibility | Explicit non-responsibility |
| --- | --- | --- |
| KUC | Generic retained text surface, popup/menu lifecycle, physical-input and AccessKit evidence, current-frame opaque record | KatanA action names, Markdown operation tables, document paths, buffer ranges, host state |
| KLE | Compose KUC surfaces and forward an opaque, one-shot host record unchanged | Coordinates, `egui::Id`, raw input synthesis, Markdown/action enums, document mutation, host-effect injection |
| KatanA host | Resolve its own source-derived context leaves and perform save, format, authoring, and ingest effects | Delegating semantic operation selection to KLE or KUC |
| `katana-host-e2e` | Drive the real host/input routes and preserve immutable per-run evidence | Simulator counters, action-only equivalence, source-marker-only success |

No new KLE-specific menu, text, raster, accessibility, coordinate, or action
implementation is permitted.  Generic behaviour discovered during this work
must be added to KUC, with KLE retaining only opaque composition/transit.

## Required Evidence Plan

Every source-derived leaf has two independent layers.  Neither replaces the
other.

1. **KLE/KUC implementation layer.** Drive the public
   `katana_language_editor_egui::EguiLanguageEditor` through a current-frame
   KUC record.  The opener and leaf must be physical input against visible
   record bounds; request leases are single-use.  Verify retained record,
   AccessKit visibility, lifecycle/close semantics, and opaque receipt.  Do
   not inspect or manufacture a KatanA action in KLE.
2. **Fixed KatanA host layer.** Build the clean fixed source in an isolated
   `FixedSourceHarness` and drive `KatanaApp` through the same actual input
   family.  The host alone resolves the effect.  Assert an observable host
   outcome: persisted save/format result, exact Markdown buffer transform and
   cursor/undo state, or the required file/clipboard dialog/ingest outcome.
   Host state may be observed only through a purpose-built E2E observation
   port outside KLE/KUC; it must never be used to inject the action.

The two layers are joined only by a source-derived leaf identifier, fixed
source hash/revision, input class, and independently captured evidence hash.
They are not joined by `EditorAction`, a semantic callback, coordinates, text
ranges, or a test-only action map.

## Physical Input Families

For each eligible context-menu leaf, run all three open routes against the
real editor text surface:

- secondary pointer press/release;
- `Shift+F10` while the editor owns focus;
- AccessKit invoke on the current-frame editor text target.

After opening, obtain the leaf only from the refreshed visible KUC/host record
and activate it through its bounds or platform accessibility target.  A stale,
offscreen, disabled, replayed, or wrong-surface request must fail closed and
produce no document/host mutation.  No route may be represented by a direct
`AppAction`, `EditorAction`, `RawInput` built in KLE, or a hard-coded menu
coordinate.

## Leaf Matrix and Effects

The matrix is derived from the fixed source audit and is exhaustive for this
context menu:

- Save;
- Format only for editable `.md` / `.markdown`, and absent for ineligible
  sources;
- 13 direct authoring rows: Bold, Italic, Strikethrough, Inline Code, H1, H2,
  H3, Bullet List, Numbered List, Blockquote, Horizontal Rule, Link, Table;
- Code Block for each of the 17 source-defined `CodeBlockKind` values;
- Image File and Clipboard Image, including unsupported/denied clipboard
  behaviour.

For authoring leaves the fixed host assertion includes exact UTF-8 buffer
result, selection/cursor restoration, dirty transition, undo/redo transition,
and a read-only/reference no-mutation case.  The 13 direct authoring rows plus
17 code-kind rows must remain mechanically tied to the fixed-source inventory;
the count is an assertion, not a hand-maintained substitute for that inventory.

Save and Format require filesystem assertions from isolated fixtures.  Image
File requires the actual dialog/selected-file continuation; Clipboard Image
requires the platform clipboard priority and failure cases.  A callback receipt
or a pending `AppAction` is insufficient for these effects.

## Test Structure

`tools/katana-host-e2e/src/context_menu_input.rs` may contain generic test
orchestration and evidence structs.  It may not define KatanA Markdown enums,
coordinate maps, document mutation helpers, or simulator-only host state.
`tools/katana-host-e2e/tests/context_menu_input.rs` owns only source-derived
scenario declarations and assertions.  It must call the public KLE route for
the implementation layer and the clean fixed-host harness for the host layer.

The tests must not be introduced merely to satisfy the marker audit in
`katana-parity-check`.  The audit is strengthened only after real test runtime
exists, and it must reject source-only, callback-only, static-gallery, and
action-injection substitutes.

## Acceptance and Failure Policy

- `just katana-host-e2e-context-menu` runs the real test target under
  `--locked`; it must fail if the target or either evidence layer is absent.
- `just katana-parity-check` runs that E2E target before static parity audit.
- strict test, clippy with `-D warnings`, formatting, AST rules, stale/replay,
  no-mutation, and cross-route equivalence are required.
- macOS AX/input permission failures fail closed and are reported as an
  environment gate.  They are never bypassed or converted to a skipped pass.
- A green Context Menu gate closes only this matrix.  Document find/search,
  line gutter, toolbars/floating controls, selection/IME/undo, image ingest,
  full host integration, emoji cross-platform proof, visual video evidence,
  and release gates remain separately incomplete.
