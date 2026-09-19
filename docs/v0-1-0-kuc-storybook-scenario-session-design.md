# KUC Stateful Storybook Scenario Session Design

## Purpose

The KLE Storybook must be a live harness for the generic KUC editor root, not a
counter-only display. The reusable retained UI remains in KUC. KLE receives
only a fresh opaque host-projection lease and forwards the KUC root event batch
once per frame.

This design does not create a KatanA host substitute. It improves only the
generic Storybook interaction path and cannot be used as actual KatanA editor,
buffer, dirty, undo, file, search, or replace-effect evidence.

## Ownership

| Layer | Owns | Must not own |
| --- | --- | --- |
| KUC `FullTextCommandSurfaceScenarioSession` | generic fixture presentation, retained text value/selection, search and replacement input values, current generic root event processing, fresh opaque leases | KatanA actions, document identity, file/path, query result semantics, Markdown transforms, replacement effect, dirty/undo/preview state |
| KLE Storybook provider | session lifecycle, opaque lease retain/synchronize, opaque root receipt | child presentation, fixture text, coordinates, text/search payload inspection, local renderer or KatanA state |
| KatanA host | all source-derived buffer/search/replace effects | KUC retained component implementation |

## Session Contract

1. `retain_lease` and `synchronize_lease` issue a current KUC opaque lease.
   The caller cannot decode, clone, serialize, or reconstruct child
   presentation.
2. The lease's KUC-owned router creates its effect batch from the current root
   event context. It mutates session state only when the event transport has
   completed every child dispatch and the opaque effect batch is consumed once.
3. The session accepts only generic retained-presentation updates:
   `TextAreaEvent::Change`, selection change, search query input, and
   replacement input. It preserves UTF-8 input exactly, including Japanese,
   `⭐️` U+2B50 U+FE0F, and ZWJ sequences.
4. Search navigation and option controls remain generic UI requests. The
   session neither computes KatanA document ranges nor scrolls a host editor.
5. Replace and Replace All remain visible but disabled while the fixed KatanA
   source has no general replace route. A replacement input value may be shown
   only where the host projection enables it; no generic session may alter text
   in response to a replace request.
6. A fresh lease on the next frame reflects accepted retained UI input. A stale,
   duplicate, rejected, or unconsumed transport changes neither session state
   nor the next projection.

## Required Tests

- KUC unit tests: generic state changes only after one consumed opaque effect;
  duplicate/stale/no-event cases are inert; text, Japanese IME commit, VS16,
  and ZWJ retain exact bytes.
- KUC actual-egui tests: physical Text/IME input produces a root event,
  forwarding consumes it once, the synchronized next frame shows the KUC-owned
  update, and the same-frame artifact/AccessKit record remains valid.
- KLE Storybook tests: the provider imports only the session and opaque lease,
  does not contain fixture/presentation/text geometry, and its generic
  dispatcher is no longer counter-only.
- Existing fixed-KatanA host-E2E and three-native-OS gates remain mandatory
  release evidence; this session test cannot satisfy either gate.
