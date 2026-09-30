# KatanA Source Closure Implementation Plan

## Status

This plan is an implementation gate for the v0.1.0 parity checker. It does
not make the existing static `matrix::FEATURES`, legacy leaf inventory, a
scratch KLE test, or Storybook evidence acceptable. KatanA remains read-only at
`4f6a6287c650a38633c7baeb544a92e739c68567`.

## Fixed Decisions

- The canonical release input/output is exactly the six artifacts in
  `v0-1-0-parity-manifest-schema.md`. No static feature count, expected-ID
  list, shared selector, default condition, or compatibility manifest is read
  on the acceptance path.
- `docs/v0-1-0-user-mandated-leaves.json` is the only non-source leaf input.
  It may add only its named `replace.current` and `replace.all` leaves; it
  cannot authorize a category or wildcard.
- The generator reports ambiguity, unresolved callbacks, dynamic dispatch,
  macro expansion gaps, unknown `cfg`, missing profile evidence, and missing
  host effects as data and the release validator rejects each one.
- A KUC component test proves generic interaction only. A KLE consumer test
  proves one-time opaque transit only. Neither may satisfy a KatanA host effect
  leaf. The execution record joins all three evidence levels by generated leaf
  fingerprint.
- The current static checker remains a failing migration residue until the
  canonical route is complete. It must be removed atomically; it must not run
  in parallel as a release fallback.

## Operational CI Provenance

The three-profile workflow is an acceptance producer, not a documentation
example. It must fail before publication unless every profile, repository and
CLI contract is reproducible.

- The workflow captures `macos-latest`, `windows-latest`, and `ubuntu-latest`
  independently, then assembles them only when their run ID and fixed KatanA
  revision match.
- The workflow invokes `materialize-closure` with the parser's exact
  `--canonical-root` option. Unknown option spellings are a workflow contract
  failure and must be covered by a checker test.
- Provenance retains the KLE and KUC resolved commit IDs and clean-worktree
  state in addition to their tracked-tree fingerprints. A byte fingerprint
  without its source revision cannot identify a release input.
- KUC is checked out at the provenance-recorded revision. A floating checkout,
  missing revision, dirty worktree, mixed run ID, missing profile, or profile
  substitution is a release failure.
- The workflow file itself is a release input: it must be tracked and its
  capture, upload, download, assemble, validate and materialize invocation
  contract is checked by `katana-parity-check`.

These requirements establish source-closure provenance only. They do not
claim KatanA has adopted KLE and cannot replace actual KatanA host E2E.

### Current Implementation State

As of 2026-08-26, `capture-provenance` records and verifies the full Git SHA
and the empty `git status --porcelain=v1 --untracked-files=all` output for both
KLE and KUC. Those values participate in the canonical root identity. A dirty
tree, including an untracked file, fails before a provenance artifact is
finalized.

The checked-out KUC repository remains at `v0.1.4` with the required generic
Unicode/root work still uncommitted. It must not be pinned as evidence for that
work. The source-closure workflow is therefore not a release workflow yet: it
must be committed only after KUC has a clean released commit, and every KLE
execution job must check out that exact immutable SHA at `../katana-ui-core`.
The current workflow lacks that cross-job KUC pin and is deliberately an
unmet release condition rather than a fallback.

## Implementation Order

### 1. Canonical Index And Closure

Owner: `tools/katana-parity-check/src/source_closure/`.

1. Parse the source-universe roots and user-mandated input with `syn`.
2. Build deterministic module, import-alias, item, test-registration, and
   external-dependency indexes from source bytes and spans.
3. Traverse module/call/method/closure/macro/action/cfg edges into
   `source-closure.json`; preserve every unresolved candidate instead of
   guessing a target.
4. Materialize the three distinct release profile records from actual runner
   evidence. A local macOS result cannot stand in for Windows or Linux.

Exit condition: mutation tests prove that source/file hash, alias, cfg,
registered-test, external-ui symbol, and unresolved-edge changes invalidate
the artifact. There is no fixed product feature count in this phase.

### 2. Action Origins And Branches

Owner: new generated-artifact modules under
`tools/katana-parity-check/src/source_closure/`.

1. For each reachable action construction, trace backwards to one input origin
   and forwards through definition, dispatch, handler, and final host effect.
2. Classify every origin exactly once as direct UI, shortcut, host continuation,
   user-mandated extension, unmounted continuation, host-only, or unresolved.
3. Expand every reachable visible/action/accessibility branch and early return
   into `branch-catalog.json` with source span and profile membership.
4. Generate Markdown authoring operations, all code-block kinds, context-menu
   leaves, text/search/replace leaves, tab/document leaves, and editor-frame
   sibling leaves from their actual source variants and construction sites.

Exit condition: fixture mutations for aliases, trait dispatch, callbacks,
macros, duplicate action origins, dirty-close continuation, user-mandated
replace, and every branch classification fail closed. No KLE command enum or
count-based inventory participates.

#### 2a. Action Construction Index Foundation

Before any origin may be classified, the generator records source spans for
the `AppAction` enum definition/variants and every AST construction site
(`AppAction::Variant(...)` and struct-style variants). The index is derived
from parsed items and expressions, not token scanning. A construction site
whose path is imported, dynamically dispatched, macro-generated, ambiguous, or
otherwise not proven to resolve to one indexed `AppAction` variant is emitted
as an explicit unresolved entry; it cannot silently disappear or be assigned a
default origin. The initial `action-origins.json` slice therefore leaves every
otherwise discovered site unresolved until a later phase proves its backwards
input origin and forwards dispatch/handler/effect route.

This foundation adds no accepted leaf and cannot make the six-artifact
validator pass. Its purpose is to make the absence of a complete action trace
observable and mutation-testable before deriving any editor inventory from it.

#### 2b. Dispatch-Arm Index Foundation

The next source-derived index records every `match` arm whose pattern names an
`AppAction` variant in a reachable dispatch function. For each arm it records
the action variant span, dispatch-function span, arm span, and every direct
handler call syntax in that arm. A direct state assignment, a nested condition,
a fallthrough `other` arm, an indirect/trait/macro callback, zero handlers, or
multiple handlers is not a final effect and remains explicitly unresolved.
The index may append source facts to an already unresolved action origin, but
it must not classify the input origin or claim a handler's semantic effect.

This phase deliberately models the actual `dispatch_action` /
`dispatch_secondary` / `dispatch_tertiary` chain without assuming that every
action reaches a named handler. Full route tracing later proves inter-dispatch
fallthrough, handler definition, host state/file/native effect, and profile
branches before clearing an unresolved action origin.

#### 2c. Handler-Definition Index Foundation

For a direct handler-call syntax recorded by the dispatch-arm index, the
generator indexes parsed inherent/trait method definitions with their exact
implementation, method, and span facts. It may attach a handler definition to
a provisional dispatch route only when there is exactly one syntax-compatible
definition in the reachable source universe. Missing, duplicate, trait-only,
macro-generated, receiver-incompatible, or dynamically selected definitions
remain unresolved. This index does not infer a method's final effect from its
name, body text, test name, or documentation.

The later effect phase must traverse the selected method body and its calls,
assignments, early returns, cfg branches, and external/native boundaries. Until
then every action origin remains unresolved even if its dispatch arm and
handler definition are known.

#### 2d. Handler-Body Fact Index Foundation

For every uniquely linked handler definition, the generator records the parsed
body's direct call syntax, receiver/assignment facts, return/early-return
spans, nested `if`/`match` branches, and cfg attributes. The index may not call
an assignment or a call an effect by name alone: each fact is a source span and
syntax record. Closures, macros, trait/dynamic calls, unresolved imports, and
external/native calls remain explicit boundary facts. A handler with multiple
terminal-looking facts, or a body whose terminal behavior cannot be proven,
keeps its action origin unresolved.

The purpose is an auditable input for later effect classification, not a
substitute for executing KatanA. No body fact may clear a branch classification
or satisfy a host-effect leaf.

#### 2e. Input-Origin Candidate Index Foundation

For every action construction site, the generator records enclosing AST input
candidates such as an egui response activation condition, keyboard/shortcut
consumption condition, accesskit action condition, callback/closure boundary,
or a non-UI host continuation. These are candidate source facts, not an origin
classification. A construction outside a structurally provable candidate, a
construction under multiple candidates, macro-generated input, or an unknown
condition remains explicit unresolved evidence.

Only a later phase may classify one candidate into direct UI, shortcut, host
continuation, user-mandated extension, unmounted continuation, host-only, or
unresolved after it proves the complete backwards route and profile behavior.
The candidate index never infers an actual physical input or KLE/KUC transport.

#### 2f. Terminal-Effect Candidate Index Foundation

For every uniquely linked handler definition, the generator records parsed
terminal-effect *candidates* separately from handler-body facts: direct
`self`-receiver assignment, same-implementation continuation call,
external/native boundary call, return, and every enclosing `if`/`match`/`cfg`
branch. Each candidate carries exact syntax and span provenance. A candidate
is never classified as a KatanA host effect from its name, receiver type,
return value, test name, or documentation.

Multiple candidate terminals, a candidate behind a branch, an unresolved
import, a dynamic/trait call, a macro/closure boundary, or an absent terminal
candidate remains explicit unresolved evidence. Later route expansion must
prove the complete inter-handler path and actual host state/file/native effect
before it can clear an action origin. This index creates no leaf and cannot
satisfy KUC, KLE transit, Storybook, or host-E2E evidence.

The bounded foundation is implemented with deterministic syntax/span mutation
tests. It records candidates in `action-origins.json` only as unresolved route
facts; every `origin_classification` remains `unresolved`.

#### 2g. Same-Implementation Continuation Route Foundation

The generator may connect a terminal candidate of the exact form
`self.method(...)` to a method definition only when one inherent definition on
the same implementation type has a receiver-compatible signature. The route
record includes the call span and both method-definition spans. It is still a
provisional source route, not an effect classification.

Missing, duplicate, recursive, mutually recursive, branch-contained, or
receiver-incompatible continuations remain unresolved. Any continuation that
crosses a trait object, a dynamic receiver, a macro/closure, an unresolved
import, another implementation type, or an external/native boundary remains
an explicit boundary. The route expansion may not clear an action origin,
generate a leaf, or substitute for physical KUC/KLE/KatanA execution.

#### 2h. Branch Outcome Candidate Foundation

For every recorded `if` and `match` branch in a uniquely linked handler, the
generator records each arm/body as an outcome candidate with its condition or
pattern syntax and span, the enclosing cfg provenance, and the terminal
candidate facts reached in that arm. It must distinguish an absent `else`, an
empty arm, an early return, and a fallthrough body. This is a structural
coverage index only; it does not infer boolean truth, profile membership,
visible state, command availability, or a host effect.

Every branch outcome remains unresolved until later profile evaluation and
physical execution prove it. Nested branches, guards, macros, closures,
dynamic/trait calls, and source parse failures are explicit boundaries. The
index may not collapse sibling arms into a shared selector, clear an action
origin, generate a leaf, or classify an effect.

### 3. Generated Leaf Join

Owner: `source_closure` artifact assembly and validator.

1. Join each generated source branch/origin to exactly one KUC generic surface
   capability, KLE opaque transit, and host execution requirement.
2. Emit `leaf-manifest.json` without aggregate selectors, `always` state,
   source markers, or simulator-only execution modes.
3. Require `execution-record.json` to contain physical input, current-frame
   AccessKit discovery where applicable, correlation, handler, and final
   effect for every host-effect leaf.
4. Require `storybook-artifacts.json` to refer to the same public KUC root
   artifact and generated leaf fingerprint, never a fallback renderer or
   callback counter.

Exit condition: duplicate, missing, stale, cross-profile, source-only,
simulator-only, and mismatched artifact/effect records are rejected.

### 4. Atomic Acceptance Cutover

Owner: `tools/katana-parity-check/src/main.rs` and release scripts.

1. Add an explicit generate command that writes all six canonical artifacts
   from verified inputs.
2. Add an explicit validate command that accepts only the canonical artifact
   directory and verifies root fingerprints, closure completeness, leaf joins,
   execution, and Storybook evidence.
3. Delete the compiled acceptance dependencies on `matrix::FEATURES`,
   `REQUIRED_FEATURE_COUNT`, `LegacyLeafCapabilityAudit`, and legacy source
   markers in the same change that makes validate authoritative.
4. Keep the release invocation fail-closed when artifacts are absent or any
   leaf remains open. Do not convert missing host E2E into a warning.

Exit condition: an intentionally incomplete artifact directory fails, a full
fixture directory passes, and production acceptance has no static inventory
imports or compatibility route.

## Current Entry Condition

The existing source-closure unit suite validates only foundation artifacts and
fixed-reference capture. The live checker still reports static/legacy blockers
and is not a completeness proof. Phase 1 must be implemented before KLE/KUC
feature evidence can be accepted as full KatanA parity.
