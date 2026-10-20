# v0.1.0 KLE Opaque Host Action Boundary Design

## Decision

KLE has no public or internal KatanA action enum, action request, action
source, action payload, document range, cursor, file URL, diagnostic id,
scroll position, view-mode, or host-effect state. It receives a retained KUC
root projection from the host and forwards one opaque event transport exactly
once. Its observable output is a closed receipt containing only root revision,
correlation, KUC record/accessibility hashes, event cardinality, and consumed
state.

KUC owns generic text input, command chrome, menus, popup lifecycle, generic
selection interaction, accessibility, platform text runtime, and the
one-shot event transport. KUC does not know KatanA actions or document
semantics. The fixed KatanA host alone resolves its own physical controls,
shortcuts, document state, markdown transforms, clipboard payloads, file I/O,
diagnostics, view modes, and navigation.

## Current Violation Inventory

The following KLE-core types recreate fixed-host semantics and must be
removed rather than generalized as KLE APIs:

* `EditorAction`, `EditorActionRequest`, and `EditorActionSource`;
* action variants for save, format, image/clipboard ingest, diagnostics,
  Problems panel, gutter line, scroll/jump, view/split mode, and transient UI;
* `EditorActionControl` and `EditorEvent::ActionRequested`;
* request DTOs carrying scroll coordinates, lines, diagnostic identifiers,
  URL/file payloads, or document selection state; and
* Storybook contracts that create, match, or count KLE semantic actions.

These are not a stable generic editor contract. They duplicate KatanA's
`AppAction` routing or private state using a different type name.

## Replacement Boundary

### KUC Root

The host supplies KUC generic presentation and opaque targets. KUC emits only
generic child-class events (`text`, `toolbar`, `floating`, `search`, and
`context menu`) inside a one-shot transport. Any semantic payload remains
opaque to KLE. KUC's public receipt never exposes event contents.

### KLE

KLE accepts the opaque projection lease and forwards it once. It may reject a
missing, stale, reentrant, or replayed lease with a typed boundary error. It
does not inspect the projection, generate a target, map an event, mutate a
document, or retain a host operation. KLE Storybook uses the same public root
and records only generic class cardinality plus opaque consumption.

### Fixed KatanA

The unchanged host owns every action mapping. Fixed-host E2E discovers the
source-derived native control, performs physical input, and observes the final
external effect. No KLE action, raw input, pending action, fixture mutation,
or in-process `KatanaApp` can substitute for this proof.

## Migration Plan

1. Inventory every `EditorAction*`, `EditorEvent::ActionRequested`, control
   request, and Storybook contract use, then connect each to its fixed-source
   host owner and required host effect.
2. Replace KLE consumers with public KUC root receipts and opaque forwarding;
   remove semantic event/action constructors, match arms, state, and tests.
3. Add an AST action-boundary rule over KLE core, KLE egui, and Storybook that
   rejects all remaining semantic action types, host mappings, payload fields,
   and action-event assertions without excluding source files.
4. Extend KUC only for a genuinely generic missing transport or visual-state
   primitive. KUC must not receive KatanA names, action enums, document
   payloads, or host transformations.
5. Add real fixed-host physical-effect tests feature by feature. The absence
   of a source route, native target, input permission, or observer is a typed
   release blocker.

## Required Evidence

* AST rejection fixtures for every removed action category and KatanA mapping.
* KLE/KUC root tests for opaque one-shot/replay/stale/error behavior with no
  semantic payload access.
* Storybook public-root artifacts demonstrating actual KUC controls and opaque
  receipt behavior, never local action success.
* Fixed KatanA host E2E for save, format, image ingest, clipboard, diagnostics,
  gutter, search/navigation, view mode, and Markdown authoring, each joined to
  the corresponding final host effect and no-mutation/error branch.

Until all four evidence layers exist for each fixed-source leaf, v0.1.0 is not
complete.
