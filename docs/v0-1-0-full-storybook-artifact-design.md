# KLE v0.1.0 Full Storybook Artifact Boundary

## Decision

KLE does not render editor widgets, synthesize raw input, compose video, or retain
editor semantics for Storybook. A full-editor artifact is a consumer of a generic
KUC artifact root. KLE only joins the source-derived leaf identity, its generic
interaction class, and KLE's one-shot opaque-transit receipt to the KUC-generated
frame and accessibility evidence.

KatanA-owned effects remain `downstream_required` in the RC artifact. They cannot
be represented as KLE success and require the separate KatanA #336 host record
before the final release gate passes.

## Current Gap

`katana-ui-core@0.3.5` publishes a generic root factory and projection encoder,
but its public Storybook replay path is closed over the eight
`FullTextCommandSurfaceScenarioId` values. The public
`FullTextCommandSurfaceScenarioFactory::issue` accepts only that enum, and the
opaque raw-input stage constructor is private. KLE's current `StorybookHost` is
therefore limited to those fixture scenarios; it cannot request a KUC-owned stage
for every source-derived generic interaction class without either manufacturing
input in KLE or reusing an unrelated fixture. Both would be invalid evidence.

This is a KUC capability gap, not a KLE renderer gap. The required generic API is
tracked in [KUC Issue #40](https://github.com/HiroyukiFuruno/katana-ui-core/issues/40).
KLE must not implement a local substitute.

## Required KUC Consumer Contract

The KUC API must provide a versioned, opaque, consumer-defined artifact plan that:

1. accepts only generic interaction/effect categories and host-projected opaque
   tokens; it accepts no KatanA action, Markdown syntax, document content, path,
   URL, coordinate, glyph fallback, or renderer callback;
2. issues KUC-owned physical stages and retains the same generic full-editor root
   across stages;
3. emits one numbered PNG per stage, decode/hash proof, current-frame AccessKit
   proof, exact `U+2B50 U+FE0F` color-glyph/measurement/hit-test proof, and a
   one-shot opaque forwarding receipt;
4. rejects an incomplete plan, duplicate leaf/stage binding, stale revision,
   reused receipt, media overwrite, and any attempt to serialize an opaque host
   target; and
5. is available through the published registry crate on macOS, Windows, and Linux.

## KLE Integration Plan After Published KUC Support

1. Generate the canonical source closure, then run a separate deterministic leaf
   join stage. Its input is only the fixed-source branch/action-origin record and
   a versioned KLE mapping table. Each mapping names a generic KUC interaction
   class, source-span-based visible path and preconditions, KUC component family,
   KLE public show signature, and one-shot opaque transit class. It has no
   KatanA semantic action, payload, coordinate, content, URL, or mutable editor
   state. Duplicate, stale, unmapped, or `unresolved:*` leaves fail before any
   artifact request.
2. Ask KUC for the full generic plan, execute it using only KUC-owned stages, and
   copy only the resulting KUC receipt/frame identifiers and media hashes into
   `storybook-artifacts.json`.
3. Require three independent OS profile runs. Every KLE/KUC-owned leaf must have
   one complete artifact per required profile. KatanA-owned leaves must state
   `downstream_required`, not `passing`.
4. The RC validator accepts only this complete pre-adoption evidence. The final
   validator additionally requires KatanA #336 physical host execution records
   for every host-effect leaf and the final registry version.

## Non-Goals

- No KLE-local rasterizer, emoji fallback, video compositor, raw-input builder,
  presentation fixture, or semantic host adapter.
- No path/git KUC dependency, mock host proof, static source claim, skip flag, or
  reuse of one fixture stage as evidence for unrelated source leaves.
- No KatanA or KUC repository modification from KLE. Cross-repository work is
  issue-first and owned by that repository.
