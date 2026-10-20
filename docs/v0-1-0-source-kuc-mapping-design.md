# KLE v0.1.0 Source-to-KUC Mapping Design

## Decision

`katana-parity-check` owns the mapping from an authenticated KatanA source leaf
to a generic KUC consumer-artifact stage.  The mapping is neither a KLE widget
implementation nor a KUC extension.  It is a versioned, checked-in semantic
input which is accepted only when it matches the canonical source candidate
exactly.

KLE does not infer an interaction class from an `AppAction` name, source text,
or a Storybook fixture.  An absent, duplicate, stale, or partially populated
mapping leaves the source leaf unresolved and prevents the producer workflow
from running.

## Mapping Input

The source-closure tool will read a `generic-interaction-mapping-v2` document.
Each entry binds exactly one `source_candidate_fingerprint` and repeats its
immutable source identity:

- requirement origin span;
- exactly one source branch ID;
- exactly one action-origin ID;
- required profile IDs;
- a published KUC generic interaction class;
- KUC component family, public KLE show signature, opaque-transit class;
- visible path, state preconditions, and declared effect kind.
- one or more immutable semantic-evidence references.  At least one reference
  must repeat the candidate's source path, source SHA-256, branch ID, branch
  span, and branch excerpt SHA-256.

The mapping has no KatanA payload, document text, Markdown syntax, URL, path,
coordinates, raw input, renderer callback, glyph policy, or font configuration.
It names no final KUC stage ID: KUC V2 issues that ID at execution time.  The
canonical `storybook_stage_id` is instead derived as
`kuc-stage:<source_candidate_fingerprint>`, then replaced only by the KUC-issued
stage ID during the evidence join.

## Validation

Before a mapping can enrich `LeafStatusRecord`, the tool must prove all of the
following:

1. The mapping schema is exactly `generic-interaction-mapping-v2`.
2. The mapping's source candidate, requirement span, branch, action origin, and
   profile set exactly match one canonical structured candidate.
3. Every canonical candidate has one mapping and no mapping targets an unknown
   candidate.
4. A candidate identity and its declared Storybook placeholder occur only
   once. Several requirement bindings or source construction/dispatch routes
   may legitimately refer to the same branch/action pair; they retain their
   distinct authenticated candidate identities and must each be covered.
5. Every mapping has a source-hash and branch-span-matched semantic-evidence
   reference.  A prose rationale, action name, or fixture name alone is not
   evidence.
6. All semantic fields are nonempty and do not use `unresolved:` or placeholder
   values.  Interaction classes are accepted only from the published KUC V2
   allowlist that the KLE producer already validates.
7. `declared_effect_kind` is one of `kuc_retained_ui_effect`,
   `in_process_host_effect`, or `native_external_host_effect`; a host effect
   remains `downstream_required` for KatanA #336.

Failure of any condition is a source-closure failure.  There is no partial
mapping, default interaction, or fixture-stage reuse mode.

## Producer and Join

`source-closure emit-consumer-plan` reads the immutable leaf manifest, source
closure, branch catalog, and mapping together. It revalidates canonical binding
and the complete mapping before selecting one canonical profile. It writes
only the producer wire schema, run ID, exact source-closure file SHA-256,
source-derived profile fingerprint, profile ID, source leaf IDs, and their
verified interaction classes, and refuses to overwrite an existing plan. An enriched
leaf manifest alone is not an authority for an interaction: the verified
mapping remains required at both plan generation and the later evidence join.

After source materialization succeeds, the tool derives one profile-local
`kle.consumer-artifact-plan.v1` from verified mappings.  It passes the exact
leaf IDs and mapped generic interaction classes to the KLE producer.  KUC alone
creates numbered PNGs, frame/accessibility/unicode evidence, receipts, and
stage IDs.

The final join accepts a producer artifact only when its root/run provenance,
profile, leaf ID, interaction class, stage ID, frame ID, media hash, and
receipt are unique and match the enriched leaf manifest.  It then writes
`execution-record.json` and `storybook-artifacts.json` with an explicit
`execution_kind = kuc_consumer_artifact`.  This is KLE/KUC UI evidence only;
it cannot satisfy a KatanA host effect.  Host effects remain separately gated
by the downstream KatanA #336 record.

## Execution Record Boundary

Each canonical execution entry declares `execution_kind` as `katana_host` or
`kuc_consumer_artifact`. Existing records without the field are interpreted
as `katana_host`, preserving their stricter pre-publication rejection. Unknown
kinds are rejected during deserialization. RC artifacts continue to contain no
execution entries. KLE release artifacts may contain only consumer executions
with complete leaf/profile coverage; downstream full parity counts only host
executions and must never accept consumer stages as host-effect proof. The
separate source completeness and consumer evidence integrity gates still apply.

## Consumer Release Bundle

The new `kle.consumer-release-evidence.v1` bundle stores the run ID, the exact
source-closure SHA-256, the exact mapping SHA-256, and one record per canonical
profile with its fingerprint, reconstructed plan SHA-256, and producer evidence
file SHA-256. Mapping bytes are stored as `generic-interaction-mapping.json`;
profile artifacts are stored under `consumer/<canonical-profile-id>/`. These
paths are derived by the verifier rather than supplied by the bundle.

The release gate reconstructs every plan from the immutable canonical binding
and stored mapping, reruns profile intake, and compares all final leaf,
execution and Storybook fields with the reconstruction. A passing leaf retains
`downstream_required` host state and a null host execution ID. KUC stage IDs
must agree across the required profiles of a leaf. Media hashes in canonical
Storybook records use the existing `sha256:` prefix; producer hashes remain
the KUC lowercase hexadecimal form. Legacy before/after receipt records never
satisfy this consumer bundle contract.

Join output is built in a newly reserved staging directory beside the intended
destination. Source and mapping bytes are copied unchanged; only files accepted
by intake are copied from each profile. Intake is rerun against the copies and
the complete artifact validator must pass before publication. Existing output
is never replaced, and a failed join never publishes a partial destination.
Publication uses `renamore::rename_exclusive`, not a check followed by
`std::fs::rename`: the OS must enforce no replacement even if another writer
creates the destination after inspection. An unsupported atomic operation is
an error; the non-atomic fallback API is not used.
Unresolved source population, branch classification, native-input requirements,
or downstream host work are not satisfied by bundle assembly.

## Annotation Catalog

`source-closure emit-generic-annotation-catalog` derives a
`generic-interaction-annotation-catalog.v1` from the immutable leaf manifest,
source closure, and branch catalog.  Each catalog entry carries the candidate
identity plus source path/SHA, branch span, and excerpt hash required by the
mapping evidence.  It is a review input only: it contains no interaction class
or passing status and cannot be supplied to the KLE producer.  The command
reconstructs and validates canonical binding before it writes a new output, so
an artifact from another source closure cannot seed a mapping.

## Workflow Order

1. Capture the existing three profile source facts.
2. Materialize and validate the source closure plus total mapping.
3. Run the KLE/KUC producer matrix on macOS, Windows, and Linux.
4. Join and validate all profile artifacts, including copied media hashes.
5. Run optional post-publication KatanA native host E2E only after the join.

This order deliberately prevents a producer run from turning an unresolved
source candidate into an apparent passing editor feature.
