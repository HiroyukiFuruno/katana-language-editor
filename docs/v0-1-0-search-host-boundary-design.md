# Search Host Boundary Design

## Decision

KLE never computes document matches, character/byte ranges, regular
expressions, navigation positions, replacement text, or document mutation for
document search. Those are KatanA host responsibilities. KUC owns only the
generic search-control UI: host-projected presentation, option state, typed
input/navigation/replace intents, accessibility, focus, and rendering.

KLE forwards a KUC event as one opaque host intent and may retain only an
opaque one-shot receipt. It cannot decode an intent into query/range/replace
semantics, keep a document buffer, or manufacture a result/active index.

## Storybook

The KLE Storybook renders the public KLE/KUC root using a fixed host-projected
search presentation. Its host adapter records that exactly one opaque intent
was forwarded and exposes a closed receipt. It does not search, navigate,
replace, update fixture text, or derive a result count after an event. A
Storybook event round trip proves public UI wiring only, never KatanA document
behavior.

Replace and Replace All remain disabled/blocking presentations until a fixed
KatanA host contract and real host-effect evidence exist. A KLE/KUC local
implementation is prohibited even for Storybook demonstration.

## Migration

1. Delete local Storybook query matching, regex/range calculation, replacement
   mutation, local active-index state, and content-result tests.
2. Replace the local search host with a projection-only fixture and opaque
   intent recorder. Preserve KUC's typed search controls and the public KLE
   root path.
3. Remove public KLE `find`, `find_next`, `replace`, and `replace_all`
   execution paths. Retain only generic document editing utilities that are
   unreachable from search controls; do not rename a local implementation to
   hide it.
4. Require source guards for the removed local algorithms and semantic state.
   KUC control events and KLE opaque one-shot forwarding must still have
   physical RawInput/AccessKit and callback receipt tests.
5. Keep fixed KatanA Markdown-aware matching, navigation, refresh, scroll,
   undo, cursor, save, and all replace effects in the actual-host E2E matrix.

## Acceptance Criteria

* No KLE/Storybook search path contains a regex engine, match/range
  calculation, `find_matches`, replace/splice/range mutation, retained document
  text, result list, or active result index.
* KLE core exposes no executable search/replace API. Its remaining generic
  text editing API is not reachable from any search event.
* KUC owns all reusable search-control widgets and typed intents, without
  document semantics.
* Storybook proves public root input to opaque host receipt, disabled
  replacement no-mutation, one-shot/replay behavior, and exact `⭐️` code
  points. It does not assert mutated document text.
* Fixed KatanA host E2E remains the only completion evidence for Markdown-aware
  search, navigation, replacement, and document effects. Until available,
  affected leaves remain fail-closed release blockers.

## Prohibited Alternatives

* Moving KLE query/range/replace logic into a Storybook helper or test-only
  module.
* Moving KatanA document semantics into KUC.
* Replacing local mutation with a hidden callback/counter and treating it as a
  host effect.
* Removing parity blockers, Replace/Replace All blocker declarations, or host
  E2E requirements to make the suite green.
