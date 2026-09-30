# v0.1.0 Markdown Authoring Host Boundary Design

## Decision

KUC owns the reusable command-chrome, dropdown, context-menu, floating-menu,
focus, dismissal, capability, and visual-state behavior. KLE only retains an
opaque, single-consumption transit between that KUC root and its host. Fixed
KatanA remains the only owner of `MarkdownAuthoringOp`, `CodeBlockKind`,
selection/cursor ranges, buffer transforms, undo, preview refresh, image file
I/O, and `AppAction` mapping.

Consequently, KLE must not expose or retain `EditorAuthoringCommand`,
`EditorCodeBlockKind`, `EditorAuthoringMenuState`, cursor anchors, selection
requirements, authoring command strings, code-block strings, or
`RunAuthoringCommand`. These types are not a neutral editor API: they recreate
the fixed host's Markdown command semantics and popup lifecycle in KLE.

## Fixed-Source Catalog

The immutable source is
`/tmp/katana-fixed-source-closure-20260821` at
`4f6a6287c650a38633c7baeb544a92e739c68567`.

The visual catalog is derived from, but does not reimplement, these fixed host
routes:

| Surface | Fixed-source route | Host-owned result |
|---|---|---|
| persistent toolbar | `views/panels/editor/toolbar.rs` | `AppAction::AuthorMarkdown` for inline, heading, list, quote, code block, and image controls |
| selected-text floating toolbar | `views/panels/editor/toolbar_popup.rs` | the same action family with fixed-host popup/focus/close behavior |
| code-block menu | `views/panels/editor/code_block_menu.rs` | all 17 fixed `CodeBlockKind` values |
| editor context menu | `views/panels/editor/context_menu.rs` | save, format, authoring, image-file and clipboard-image intents |
| transform | `views/panels/editor/authoring.rs` and `app/action/dispatch_secondary.rs` | buffer, cursor, undo, dirty and preview behavior |

KLE's source closure may describe the existence, order, icon category,
availability class, and fixed-source span of a visual item. It must not derive
an operation from it, construct an `AppAction`, convert a selected range, or
infer the result text. The payload crossing from KUC through KLE is an opaque
event correlation and is consumed once by the host.

## Ownership Contract

### KUC

KUC accepts generic host-projected command and context-menu presentation:

* opaque stable item correlation;
* visible/accessibility text and icon category;
* generic enabled/disabled capability and reason;
* dropdown hierarchy and generic open/close/focus state; and
* generic event batches and root-frame/AccessKit evidence.

KUC must not name KatanA, Markdown authoring operations, code languages,
document ranges, files, or host action enums. It owns all popup placement,
viewport handling, selection-toolbar visibility mechanics, keyboard and
AccessKit interaction mechanics, and emoji/CJK text rendering.

### KLE

KLE combines the public KUC components using host-projected presentation and
forwards one opaque event batch. It may record only the receipt count,
root-generation, and opaque correlation needed to prevent replay. It does not
store command identifiers, selections, cursor coordinates, submenu state,
code kind, action source, or resulting document state.

The Storybook uses the same public KLE/KUC root. It can demonstrate each
source-derived visual leaf and its KUC event/capability state, including all
17 code-menu entries. Its host records only opaque event consumption. It must
not turn an event into a Markdown transform, a KatanA action, a local text
mutation, or a simulated success.

### Fixed KatanA Host

Only the fixed host maps a physical source-derived input to
`AppAction::AuthorMarkdown`, observes the resulting external state, and
records the buffer/undo/preview/file effects. KatanA is read-only. The host
E2E is therefore an external observation harness, not a KLE integration patch
or an in-process action bridge.

## Migration Plan

1. Add an AST boundary rule that rejects authoring command types, code-kind
   types, menu/cursor/selection state, operation strings, and action mapping
   inside KLE core, KLE egui, and Storybook; it must permit KUC generic
   presentation and opaque forwarding.
2. Introduce or use KUC-only generic command-chrome presentation/event types
   for every toolbar, floating-toolbar, dropdown, and context-menu leaf.
   Generic state remains within the retained KUC root.
3. Remove KLE core authoring APIs and their action/event routes. Replace all
   Storybook local command specs with host-projected, non-semantic opaque
   presentation and receipt-only assertions.
4. Make the KLE AST gate reject any reachable local Markdown transform,
   `AppAction` mapping, range/cursor conversion, fixed code-kind list, or
   command-string-to-operation conversion.
5. Extend the fixed-host E2E only after native current-frame correlation is
   available. Each catalog leaf requires one physical input route and one
   final host effect, or a typed blocker where the fixed source lacks a route.

## Required Automated Evidence

* KUC root tests cover toolbar/context/floating/dropdown state, all 17 generic
  dropdown entries, pointer/keyboard/AccessKit interaction, focus, outside
  dismissal, disabled controls, and exact `STAR + VS16` color rendering.
* KLE tests prove public-root rendering, one opaque event consumption, replay
  rejection, no KLE document mutation, and no exposed authoring semantics.
* AST tests reject a KLE authoring operation enum, command/code string mapping,
  cursor/range state, `AppAction`, local Markdown transform, and local
  code-kind catalog on every compiled and test path.
* Fixed KatanA host E2E proves all inline operations, eight structure/reference
  operations, 17 code kinds, context-menu paths, floating-toolbar lifecycle,
  disabled/read-only behavior, Unicode/VS16 cases, undo, dirty/preview, and
  image external effects. Each proof joins fixed revision, source span,
  native input, current frame, and final host observation.

No Storybook image, video, KUC-only test, KLE-only test, or source inventory
is a substitute for the final fixed-host effect evidence.

## Explicit Non-Goals

* Implementing a new general Replace or Replace All behavior.
* Editing KatanA to provide an integration or test-only API.
* Mapping KLE events to private KatanA action types.
* Using `RawInput`, coordinates, fixed waits, document text markers, or an
  in-process `KatanaApp` as host E2E evidence.
