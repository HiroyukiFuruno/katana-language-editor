# Fixed KatanA Host Observability Design

## Decision

Fixed KatanA parity evidence is produced only by `katana-host-e2e` against the
unchanged fixed revision. KLE and KUC remain out of the bootstrap, locator,
input, and host-effect observation path except as the independently verified
generic UI implementation under test.

The harness creates a disposable Markdown workspace and an isolated
`KATANA_CONFIG_DIR`. It then uses the fixed application's own workspace
restoration contract, or the native Open Workspace control if restoration
cannot be established, to make the document visible. It does not inject a
document buffer, action, range, event batch, or KLE/KUC projection into the
host.

## Required Evidence Chain

For every source-derived leaf, one execution correlation must bind:

1. fixed-source revision and disposable workspace identity;
2. native process/window identity and a current AccessKit generation;
3. a source-derived editor target and the physical input route;
4. the selected current-frame button/menu target and one activation;
5. a final externally observable host effect; and
6. replay/no-mutation or typed failure evidence where the route is disabled.

The native target locator must derive only accessibility role, ancestry,
enabled state, locale digest, and current frame generation from the fixed
source manifest. Coordinates, fabricated widget ids, `egui::RawInput`, and
host state injection are prohibited.

## Host-Only Effects

The harness observes rather than recreates KatanA semantics:

* save/format through filesystem bytes and timestamps;
* image ingest through asset files, Markdown link, dirty state, and Explorer
  refresh where externally observable;
* document search through current-frame accessibility/focus/navigation state;
* authoring through a persisted document save/reopen and source-text effect;
* clipboard through the real platform clipboard payload boundary.

Buffer ranges, Markdown transformation, cursor restoration, undo internals,
pending actions, document mutation, clipboard decoding, and file dialog
payload resolution remain KatanA private host behavior. They must never be
implemented in KLE/KUC or assumed from a pending action.

## Failure Policy

No editor target, unavailable native permission, absent workspace restoration,
ambiguous accessibility node, unsupported external observation, or missing
Windows/Linux profile artifact is a typed failure and a parity blocker. The
harness may not substitute an in-process `KatanaApp`, synthesized `RawInput`,
fixed waits, or a Storybook/KLE/KUC artifact.

## Current Editor Locator Constraint

At the fixed revision, the editor body is an unnamed
`MultilineTextInput`. Its enclosing `ScrollArea` and `Frame` are also
unnamed, and the editor's `egui::Id` is not a native accessibility identity.
Role alone is not a unique editor locator. Until a native current-frame
observation can prove a distinct source-derived correlation, the harness must
record `TargetMissing` or `TargetAmbiguous` and block the affected leaf. It
must not select by geometry, document value, guessed label, raw input, or an
internal widget identifier.

The next safe harness increment is therefore an observation-only editor
candidate audit: it captures role/name/description/ancestor/state facts from
the native current frame, binds them to the restored-workspace correlation,
and fails closed for zero or multiple candidates. It does not activate or
interpret an editor candidate.

## Initial Implementation Order

1. Add a disposable workspace/config bootstrap that proves the fixed app has
   restored the target workspace before any editor assertion.
2. Add current-frame editor target discovery from the fixed source manifest.
3. Bind native pointer, keyboard, and AccessKit activation to that target and
   record generation/replay behavior.
4. Implement individual external effect observers, beginning with save and
   format, then authoring, image ingest, search, clipboard, cursor/undo, and
   diagnostics.
5. Capture independent macOS, Windows, and Linux artifacts and make their
   absence fail the release gate.

## Native AX Workspace-Correlation Observation

### Purpose

The next host-harness increment is a macOS-native observation adapter. Its
only responsibility is to prove, or fail to prove, that the current native AX
frame belongs to the disposable workspace restored by the fixed application.
It then records editor candidates for the existing fail-closed audit. It does
not select, focus, type into, click, or otherwise activate an editor.

### Correlation Contract

The disposable workspace directory must have a per-run, non-secret basename.
The harness writes that absolute path into the fixed application's
`workspace.json` using the existing restoration contract. The expected AX
workspace label is derived only from that basename and from the fixed-source
`WorkspaceTabBarDetail` path-display rule. The observation succeeds only when
the current native AX window contains exactly one enabled workspace-tab or
workspace-display candidate whose title/description digest equals that
source-derived expected label digest.

The evidence record contains only:

* fixed revision and source-span digest;
* process/window identity digest and monotonic observation generation;
* workspace basename digest and correlation candidate count;
* AX role, title/description presence digests, enabled/focused/read-only state,
  and ancestor-role digests for editor candidates; and
* a canonical SHA-256 of the redacted record.

It must never persist a document value, selected text, full path, raw AX
element pointer, geometry, node identifier, or fixture contents. The expected
workspace basename itself is not used as an editor selector: it establishes
only that the observed frame is the restored workspace frame.

### Native Snapshot Boundary

The adapter lives exclusively in `tools/katana-host-e2e`. It traverses current
AX windows with bounded depth and cycle detection, reads only role, title,
description, enabled, focused, read-only, and child/ancestor relationships,
then converts them to the redacted observation DTO. It must reject malformed
or unavailable AX attributes with a typed failure. KLE, KUC, the fixed source,
and in-process `FixedHostSession` remain outside this path.

AX notifications and current-frame snapshots are condition based. Fixed sleeps,
coordinate fallback, `egui::Id`, `RawInput`, document-marker lookup, and
guessed locale labels are prohibited. Missing workspace correlation, zero
editor candidates, multiple editor candidates, or an unnamed editor identity
are expected typed blockers, not conditions to relax.

The editor observation contract is separate from the native `Open Workspace`
button locator. Reusing that button record's source span or `AXButton` role for
an editor candidate is invalid. The contract must bind the fixed editor's
`MultilineTextInput` source role/span to a tested generic
AccessKit-to-native-role translation, or fail typed `Unsupported` when no such
translation is available. It must not create a KatanA-specific guessed AX role
mapping.

### Completion Criteria for This Increment

1. A native run produces either one redacted workspace-correlated candidate
   observation or a typed fail-closed artifact.
2. The test proves that a mismatched workspace label, duplicate correlation
   label, missing AX attribute, zero editor candidate, and multiple editor
   candidate cannot pass.
3. The source-derived basename display rule and fixed revision/span are part
   of the artifact contract.
4. No interaction or host effect is claimed by this increment. Successful
   editor location remains insufficient for any parity row until a later
   source-derived input route and external effect observer are joined.

## Rejected Alternatives

* Editing fixed KatanA to add a test port.
* Treating `FixedHostSession`, `RawInput`, a pending action, a source marker,
  Storybook, video, or a screenshot as final KatanA host-effect evidence.
* Reimplementing workspace/document/Markdown/file/clipboard semantics in KLE
  or KUC.
