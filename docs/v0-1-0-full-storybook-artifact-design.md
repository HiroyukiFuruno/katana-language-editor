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

`katana-ui-core@0.3.11` publishes `ConsumerArtifactPlanIssuer` and the opaque
consumer-artifact execution path. Its ten-stage
`GenericInteractionClass::FULL_EDITOR_SEQUENCE` is KUC-owned and emits physical
PNG, Unicode/IME/measurement/hit-test evidence, and one-shot receipts. KLE must
use that published registry API; it must not reproduce a renderer, raw-input
builder, glyph fallback, or stage writer.

The published `0.3.11` consumer fails closed on the Ubuntu runner because its
KUC-owned Linux `NotoColorEmoji.ttf` expected hash is absent. This is tracked
as [KUC Issue #66](https://github.com/HiroyukiFuruno/katana-ui-core/issues/66).
KLE must not inject a font path, hash, or fallback to conceal it.

KLE's current consumer uses synthetic identifiers
`kle-full-editor-stage-<index>` and the fixed generic action target
`kuc.rich.inline-strong`. That proves the KUC contract only. It does **not**
prove that a KatanA source-derived leaf was mapped to the corresponding generic
interaction, nor can it populate canonical `execution-record.json` and
`storybook-artifacts.json`. Reusing those fixture stages as proof for arbitrary
source leaves is invalid.

The remaining KLE work is therefore a fail-closed source-to-KUC join. It is not
a KUC rendering change. The fixed KatanA inventory contains many independently
observable leaves (including 17 code-block kinds, 12 toolbar commands, search,
replace, diagnostics, gutter, context-menu, scroll, and IME branches). The v1
issuer accepts the ten generic classes exactly once and therefore cannot bind
every source-derived leaf without reusing an unrelated stage. KLE must reject
that mismatch and raise a KUC Issue for a generic plan version that supports a
distinct KUC stage per leaf while retaining KUC ownership of interaction,
rendering, and opaque transport. That requirement is tracked in
[KUC Issue #65](https://github.com/HiroyukiFuruno/katana-ui-core/issues/65).

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

## KLE Integration Plan

1. Materialize the authenticated fixed-KatanA source closure before requesting
   any UI artifact. Build a versioned KLE mapping table from immutable leaf
   provenance only: source candidate fingerprint, action-origin ID, branch ID,
   required profile IDs, generic interaction class, component family, public
   KLE show signature, and opaque transit class. It must contain no KatanA
   payload, Markdown text, document content, coordinate, URL, renderer callback,
   font policy, or raw input.
2. Verify the mapping is total over the source-derived KLE-owned leaf set:
   every source leaf is mapped once to a KUC generic interaction class and a
   distinct KUC-issued stage, source/profile identities are unchanged, and no
   `unresolved:*` field remains. Multiple leaves may share a generic interaction
   class, but they must never share a KUC stage or reuse its media/receipt. The
   current v1 exact-ten-stage sequence cannot meet this condition; implementation
   begins only after the required published KUC plan version is available. A
   host-effect leaf remains `downstream_required` for KatanA #336 but still
   requires the KLE/KUC generic stage evidence.
3. Create `ConsumerArtifactStageBinding` values from the verified leaf IDs, not
   synthetic ordinal names. Obtain each lease exclusively through the KUC
   scenario session, issue the KUC plan, and execute it using only
   `IssuedConsumerArtifactPlan::execute_next`. KLE may read KUC-produced IDs,
   hashes, and receipt metadata; it may not create or alter PNG, accessibility,
   Unicode, IME, measurement, hit-test, or receipt data.
4. Write the canonical execution and Storybook records only by joining KUC output
   to the already verified leaf/profile tuple. Each output entry must include the
   source leaf ID, declared KUC stage ID, KUC frame record ID, relative numbered
   PNG path, KUC media SHA-256, execution status, and profile ID. Reject duplicate
   receipt/frame/stage/leaf bindings, profile drift, stale root fingerprints,
   missing numbered PNGs, or overwrite attempts before publication.
5. Run the same producer independently on macOS, Windows, and Linux. The
   assembler accepts each profile only when every required source leaf has one
   matching execution record and KUC media entry. It copies verified records and
   media into the canonical source-closure root, then runs the existing strict
   artifact validator. No static fixture, prior-run artifact, or local path/git
   KUC dependency is accepted.
6. `kle-release` validates this KLE/KUC evidence and leaves KatanA host effects
   as `downstream_required`. KatanA #336 remains a separate post-publication
   downstream adoption and real-host E2E gate.

## Non-Goals

- No KLE-local rasterizer, emoji fallback, video compositor, raw-input builder,
  presentation fixture, or semantic host adapter.
- No path/git KUC dependency, mock host proof, static source claim, skip flag, or
  reuse of one fixture stage as evidence for unrelated source leaves.
- No KatanA or KUC repository modification from KLE. Cross-repository work is
  issue-first and owned by that repository.
