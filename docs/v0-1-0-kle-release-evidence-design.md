# KLE v0.1.0 Release Evidence Design

## R7 統合範囲（2026-09-05）

`kle-release` は canonical source-closure artifact directory 内の
`kle-release-evidence.json` を必須とする。`source_closure_root_sha256` は
`source-closure.json` の実バイト列全体の SHA-256（`ManifestRoot` 単体の直列化値ではない）である。
候補はこの hash、root の `release_profile_matrix_fingerprint`、source-closure に記録された
profile ID ごとの fingerprint、および source-derived leaf ごとの
`leaf_id` / `storybook_stage_id` / required profile の全組へ結合する。欠落、余剰、三 OS
profile 不足、別 leaf による stage 再利用、相対パス逸脱、symlink、ファイル hash 不一致は
fail-closed とする。

この候補の opaque trace、番号付き PNG、decode/hash、AccessKit snapshot と one-shot
receipt は KUC #40 writer が出力・検証する不透明な入力である。KLE は RawInput を生成せず、
KUC の媒体や内部 layout を再直列化しない。現段階では KUC #40 の published
consumer-defined artifact plan が未提供なので、候補が形式・結合検証を通過しても
`kle-release` はその不足を明示して失敗する。

`downstream-full`（互換 alias: `full`）だけが legacy 5-field `ExecutionRecord` と KatanA
#336 の host E2E を要求する公開後の下流ゲートである。`rc` は advisory precheck のままとし、
公開 RC または final 公開の必須条件とは表示しない。

## Status

R7 wires this boundary into `katana-parity-check --mode kle-release`, the
default `katana-parity-check` recipe, and `release-check`. It preserves the
existing five-field host `ExecutionRecord` and source-closure schema.
`downstream-full` (and compatibility alias `full`) remains the separate,
post-publication KatanA host gate. Remaining evidence work includes the KUC #40
consumer-defined artifact capability, its actual public-show integration in KLE,
and complete native captures. Wiring an integrity validator does not complete
any of those steps or the source-derived requirement classification.

## Decision

KLE v0.1.0 requires source-derived leaf evidence for every canonical release
profile, but it does not require KatanA host E2E. KatanA #336 remains a
post-publication downstream gate. The pre-publication claim is strictly:

`actual KUC RawInput -> current retained KUC root -> exactly one opaque forward
consumption or a verified no-mutation transition -> KLE observable opaque
transit receipt`.

It does not claim an AppAction, buffer, document, filesystem, clipboard,
history, workspace, or any other KatanA effect.

The existing five-field `ExecutionRecord` is downstream-host evidence and is
not changed by this work. KLE release evidence is a separate candidate artifact
with a separate validator.

## Actual Runtime Boundary

The only accepted production capture path is the existing public KLE binding:

1. `tools/kle-storybook/src/host_types_runtime.rs` calls
   `StorybookProjection::show_write_artifact_and_forward_once`.
2. `tools/kle-storybook/src/root_runtime.rs` forwards that call to
   `katana_language_editor_egui::HostProjectionBinding`.
3. `crates/katana-language-editor-egui/src/host_projection_provider.rs` obtains
   a public `EguiTextCommandSurfaceHostProjectionLease` and calls
   `KucRootBinding::show_write_artifact_and_forward_once`.
4. `crates/katana-language-editor-egui/src/kuc_root_binding/artifact.rs`
   renders the retained KUC root, invokes
   `katana_ui_core::egui::OpaqueRootArtifactReceiptWriter`, and calls
   `forward_events_once` exactly once.
5. The call returns the public
   `katana_ui_core::egui::OpaqueRootArtifactReceipt` plus KLE's
   `KucRootBindingReceipt`. `tools/kle-storybook/src/root_runtime_frame.rs`
   combines them in `StorybookRootFrame`.

The accepted receipt fields originate only from
`KucRootBindingReceipt`:

- `root_identity`, `presentation_revision`, and `state_revision`
- `record_hash`, `paint_plan_hash`, and `accessibility_snapshot_hash`
- `correlation_fingerprint`, `event_batch_fingerprint`,
  `event_cardinality`, and `consumed_once`

`consumed_once == true` proves a single invocation of the opaque forwarding
operation. It does not mean one event, one callback, or a KatanA effect.
`StorybookGenericRootForwarder` intentionally has no-op callbacks and copies
KUC dispatch class counts; neither its callbacks nor its counts are downstream
observations. In particular, the existing `dispatched_events <= event_cardinality`
check is not strengthened to equality.

## Candidate Artifact

The implemented parser accepts `kle-release-evidence.json` as the diagnostic
candidate input to the wired gate. It carries:

- one run id and one source-closure root SHA-256;
- exactly `macos-latest`, `windows-latest`, and `ubuntu-latest`, each with a
  nonempty native profile fingerprint;
- one record per `(leaf_id, profile_id, stage_id)`;
- KUC-writer-produced opaque before/after trace references and KUC stage media;
- SHA-256 values recomputed from regular in-root files;
- a public KUC receipt snapshot;
- one of two opaque outcomes:
  - `single_forwarded_batch`: `consumed_once`, nonempty correlation/event batch
    fingerprints, and nonzero KUC event cardinality;
  - `no_mutation`: two public receipt snapshots with the same root identity and
    state revision, both zero-event, and both consumed once.

There is no `passing` boolean, free-form status, event-class count, AppAction,
or host-effect assertion. A source-derived leaf and profile bind to its input,
receipt, and stage through typed fields rather than a shared aggregate marker.

The opaque trace is required before and after the operation. It identifies
KUC-produced stages and receipts, not serialized `RawInput` bytes. KLE neither
serializes nor synthesizes `FullTextCommandSurfaceRawInputStage`. A single fixed
Storybook scenario, including the current 46-frame motion output, cannot stand
in for all source-derived leaves or the three native profiles.

## Integrity Versus Producer Trust

The new parser/validator has two deliberately separate results.

`validate_integrity` parses the candidate, checks the complete profile matrix,
same-run/root/profile/leaf/stage keys, rejects aggregate status fields, rejects
absolute, parent-traversal, symlink, and missing paths, and recomputes all
declared file hashes. This proves only artifact-file integrity.

`validate_kle_release_gate` fails with the explicit KUC #40 blocker at this
stage. Current KUC already owns `OpaqueRootArtifactReceiptWriter` and
`MotionArtifactWriter::write_opaque`; their manifests and writer validation are
the correct basis for numbered PNG, decode, and hash checks. This design does
not require KUC to reveal private receipt fields individually, and does not
claim that current receipt accessors make media unverifiable. The missing
capability is that the public writer cannot yet receive every consumer-defined
generic leaf plan and return the corresponding complete three-OS stage set.

KLE may serialize its own snapshot of public receipt accessors for diagnostic
join checks. That is not a boundary violation. It must not reconstruct KUC
media/layout, serialize opaque host targets, or manufacture RawInput stages.
Until a KUC writer output ties the requested opaque trace, numbered media,
AccessKit, and one-shot receipt to a consumer-defined plan, an external JSON
file can be copied or assembled independently and is diagnostic only.

The missing KUC #40 capability is the published
[`generic full-editor artifact plan` contract](https://github.com/HiroyukiFuruno/katana-ui-core/issues/40):
a versioned opaque consumer-defined plan, KUC-issued physical stages on one
retained root, numbered PNG/decode/hash/current-frame AccessKit/one-shot receipt
per stage, and fail-closed duplicate/stale/reuse/overwrite checks across three
OS. It must not expose KUC geometry, RawInput payload bytes, opaque host targets,
child paint plans, or event semantics.

Until that KUC-owned capability exists and a KLE producer consumes it directly
from the actual call path above, no candidate may pass the KLE release gate.
No local serializer, fixed scenario replay, copied callback count, or manually
authored JSON substitutes for it.

## Producer Integration After KUC #40

After KUC #40 supplies the published registry contract, a KLE-owned Storybook
command may join source-derived leaf IDs to KUC artifact IDs and write its
diagnostic snapshot from `StorybookRootFrame` immediately after the actual show
call. It must consume KUC writer output rather than create a RawInput serializer.
The producer integration may happen only after review proves this producer calls
the existing path exactly once and cannot substitute fixed fixtures or a no-op
callback.

`kle-release` already calls `validate_kle_release_gate`: before #40 is
published it fails with the named KUC blocker. After the producer review, the
same wired invocation will validate KUC writer output for every generated
source-closure leaf across all three profiles. The downstream host command
remains separate and retains the existing host `ExecutionRecord` model.

## Non-Goals

- No KatanA checkout, path dependency, host bridge, or host E2E.
- No change to the downstream host evidence model, including legacy
  `ExecutionRecord`, or to KatanA #336 ownership.
- No changes under `source_closure/requirement_binding*.rs`.
- No new dependency and no KUC sibling edit in this patch.
