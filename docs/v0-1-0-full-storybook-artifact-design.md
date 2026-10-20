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

`katana-ui-core@0.4.1` publishes `ConsumerArtifactPlanIssuer` and the opaque
consumer-artifact execution path. `ConsumerArtifactPlanV2` accepts a
consumer-defined, variable-length set of distinct leaf bindings while KUC keeps
ownership of stage numbering, PNG generation, Unicode/IME/measurement/hit-test
evidence, AccessKit evidence, and one-shot receipts. KLE must use that published
registry API; it must not reproduce a renderer, raw-input builder, glyph
fallback, or stage writer.

KUC Issue #81 closed with the `0.4.1` registry release and installable native
evidence verifier. Its manifest distinguishes verifier output from unavailable
physical-device measurement; KLE must preserve that distinction and must not
inject a font path, hash, fallback, or synthetic native-input claim to conceal a
platform failure.

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
replace, diagnostics, gutter, context-menu, scroll, and IME branches). KUC V2
removes the fixed-ten-stage limitation, but it does not map KatanA leaves or
create canonical KLE records. KLE must still reject unresolved source fields,
duplicate leaf bindings, profile drift, and static fixture reuse. It must not
work around those checks with a local renderer, synthetic stage reuse, or an
unpublished KUC dependency.

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
   published KUC V2 plan can meet this condition only after KLE verifies the
   source mapping. A
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

## Producer Wire Contract

The KLE producer accepts only a versioned `kle.consumer-artifact-plan.v1` JSON
document. Each stage contains a source-derived opaque `leaf_id` and one of the
published KUC generic interaction classes. The document also names exactly one
canonical profile, the workflow run ID, the exact source-closure file SHA-256,
and the source-derived profile fingerprint. All three provenance values are
required and copied unchanged into the producer evidence manifest. Digest
values use `sha256:` followed by 64 lowercase hexadecimal digits. The run ID
must be nonempty and contain no whitespace. It contains no KatanA action, document text, Markdown input,
path, URL, coordinate, font policy, raw input, renderer callback, or host effect.

The producer rejects an empty or duplicate leaf set, an empty profile, an unknown
interaction, an invalid leaf identifier, or an existing output directory. It
issues `ConsumerArtifactPlanV2`, validates KUC Unicode evidence after every
stage, and writes a `kle-consumer-artifact-evidence.v1` manifest beside the
KUC-owned numbered PNG and per-stage manifest files. The evidence manifest
records the unchanged source-plan provenance, KUC-returned identifiers,
hashes, AccessKit hash, Unicode hash, and
the numbered media filename. It is a per-profile producer artifact, not a
canonical source-closure record. `katana-parity-check` remains the owner of the
cross-profile canonical `execution-record.json` and `storybook-artifacts.json`
join and must verify the source leaf manifest before accepting this output.

Profile artifact intake reconstructs the expected plan from canonical source
inputs and the total verified mapping. It compares run, source hash, profile
fingerprint, and the exact leaf/interaction population before reading media.
Each producer stage must agree with the KUC per-stage manifest, including its
receipt fingerprint and Unicode payload hash. Missing, duplicate, unknown,
rebound, modified, or symlinked files are rejected. Intake verification alone
does not produce a canonical execution record or close source completeness.

## Non-Goals

- No KLE-local rasterizer, emoji fallback, video compositor, raw-input builder,
  presentation fixture, or semantic host adapter.
- No path/git KUC dependency, mock host proof, static source claim, skip flag, or
  reuse of one fixture stage as evidence for unrelated source leaves.
- No KatanA or KUC repository modification from KLE. Cross-repository work is
  issue-first and owned by that repository.
