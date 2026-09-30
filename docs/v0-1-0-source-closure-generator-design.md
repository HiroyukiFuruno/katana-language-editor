# KatanA Source Closure Generator Design

## Status

This is the implementation design for the `katana-parity-check` source closure.
It is incomplete until Spark implements it and every required test passes. The
fixed KatanA repository is read-only throughout this work.

## Current Scaffold Is Rejected

The current `tools/katana-parity-check` source is not a partially acceptable
implementation of this design. It is a migration residue and must be replaced
in a Spark-owned checker batch.

- `src/main.rs:34-44` imports `FEATURES` and `REQUIRED_FEATURE_COUNT`, then
  treats their equality as a parity precondition. A remembered count is not a
  source closure and must not survive in the acceptance route.
- `src/leaf_inventory_types.rs:14-62` gives all leaves one of two shared RawInput
  harnesses; `:102-203` supplies a default `always` condition, common KLE source
  path, common owners and default host effects. These are declarations that can
  falsely make unrelated source branches appear covered.
- `src/leaf_inventory_types.rs:64-72` permits only `ActualKleInput`,
  `SourceOnly`, `StorybookOnly`, and `SimulatorOnly`. It has no
  `katana_shortcut_router` execution mode, while
  `leaf_inventory_validation.rs:123-129` requires every leaf to use a
  `public_show_*` selector. The current checker therefore cannot represent a
  KatanA-owned shortcut without falsely claiming KLE mounts or drives the
  KatanA shortcut router.
- `src/leaf_inventory_tests.rs:20-21` includes the rejection module below the
  test module, while `leaf_inventory_tests_rejections.rs:5-10` resolves sibling
  production modules through the wrong `super` scope. The current test build
  therefore fails with `E0432`; repairing only that import would retain the
  invalid static inventory model.

The replacement entrypoint consumes canonical generator output and validates
its fingerprints, empty unresolved/unclassified sets, and one concrete joined
record per generated leaf. It must not import a hand-maintained feature matrix,
required leaf count, expected ID list, default state condition, shared selector,
shared host-effect template, or one input-origin shape for all leaves. Unit fixtures may create small source trees for
resolver behavior, but their passing condition is the generated closure of that
fixture, never a product leaf inventory.

The implementation must first delete the static acceptance dependency from the
compiled route, then reintroduce each rejection test against the generated
artifact schema. No one-line `E0432` fix, fixture marker assertion, or count
update is a valid intermediate acceptance result.

## Inputs and Outputs

Inputs are the fixed revision and root sets in
`v0-1-0-katana-editor-source-universe.md`. Outputs are the six joined
artifacts in `v0-1-0-parity-manifest-schema.md`.

The generator is deterministic. It sorts paths, symbols, spans, edges, branch
IDs, leaf IDs, and JSON object fields before hashing or writing. It reads raw
file bytes for each SHA-256. The tree fingerprint is the SHA-256 of sorted
`path NUL file-sha256 LF` records. The generator fingerprint is the SHA-256 of
the compiled generator binary and its schema version. No current time,
filesystem iteration order, display locale, or test fixture may affect a
fingerprint.

### Release-Profile Closure

The fixed KatanA and KUC release/CI matrix contains `macos-latest`,
`windows-latest`, and `ubuntu-latest`. The generator treats each actual runner
as a distinct source profile; the profile record includes its runner label,
`rustc -vV` host triple, complete `rustc --print cfg` output, resolved Cargo
feature/dependency graph, lockfile SHA-256, and source-tree fingerprint. A
local macOS run is not evidence for Windows or Linux.

For every profile, the generator expands the active `cfg` graph. It also
records every inactive `cfg` candidate as a classified edge with the profile
predicate and source span; an inactive candidate is not silently discarded.
The release closure is the deterministic union of all three profile graphs.
An added, removed, unclassified, ambiguous, or profile-only edge fails the
gate. Macro, trait, callback, and external-boundary resolution are evaluated
inside each profile and must not inherit a result from a different target.

Every branch catalog, leaf manifest, execution record, and Storybook artifact
records the profile fingerprint and target-specific result. A missing runner,
feature graph, font prerequisite, artifact, or exact profile proof is a
release failure; no target may be skipped, delegated to a default host, or
reported through an aggregate "cross-platform" case.

## Phase 1: Module and Item Index

1. Start from every direct editor, editor-frame sibling, and integration root.
   The fixed editor-frame roots include `TabToolbar`, document `TabBar`,
   `Breadcrumbs`, and `UrlSourceBar`; `WorkspaceToolbar` is included only to
   record its source-spanned host-only sibling rationale. Resolve `mod` items to
   their Rust source file using Rust module rules.
2. Parse `docs/v0-1-0-user-mandated-leaves.json` as a separate immutable input.
   Its only accepted root is `schema_version: "1"` plus an `extensions` array,
   in canonical `replace.current`, then `replace.all` order. It is the only
   non-source leaf input and must not accept a `leaves` alias, category,
   wildcard, count, or inferred-standard-feature entry. Every extension must
   resolve its requirement anchor, KUC component, and KatanA
   definition/dispatch/handler route; the raw file SHA-256 is copied unchanged
   to every generated artifact as `user_mandated_extensions_fingerprint`.
3. Parse every included Rust source with `syn`; syntax parse failure is a
   source-closure failure.
4. Index module paths, `use` aliases, free functions, trait methods, impl
   methods, enum variants, structs, test functions, and macro invocations with
   source path and span.
5. Build a lexical import table for `crate`, `self`, `super`, renamed imports,
   and glob imports. An ambiguous glob is recorded as unresolved, never guessed.

The parser must not depend on line-token matching, a fixed `INVENTORY_FILES`,
or a list of expected leaf counts.

## Phase 2: Edge Discovery

The AST visitor records every behavior-bearing edge from each indexed item:

| AST form | Required destination resolution |
| --- | --- |
| `ItemMod` | child module file or inline module item |
| `ExprCall` and path expression | free function, associated function, constructor, or unresolved external boundary |
| `ExprMethodCall` | concrete impl method or trait method when receiver type is known; otherwise unresolved receiver/type edge |
| `ExprMatch` on `AppAction` or typed editor action | every variant arm, handler invocation, and final state/file/external effect |
| `ExprClosure` / callback registration | registered callback target and caller; unresolved callback is not ignored |
| macro invocation | expanded local source target, explicit external boundary, or unresolved macro edge |
| test attribute / integration registration | test oracle edge with registration path, never an execution substitute |
| external UI API call | lock-pinned package/source/symbol plus KatanA invoking span when the API directly affects editor behavior; unresolved dependency source is a failure |

For every reference test candidate, the generator records a complete path from
one current Cargo integration-test root (for example a file directly compiled
under `crates/katana-ui/tests/`) through `mod` or `#[path]` edges to the test
item. A local declaration inside an unreferenced `mod.rs` is **not** a runnable
registration. The record is `registered` only when that root path resolves at
the fixed reference revision; otherwise it is `unregistered` with the missing
edge and remains a source-oracle candidate only. This catches the current
`integration/editor/mod.rs` situation, where local module declarations do not
reach a root test target, while separately `#[path]`-registered files are
recorded with their actual root target. Neither state can replace a KLE-owned
execution record.

Receiver resolution must at least recognize `self` inside an `impl`, explicit
type names, trait implementations, local variables whose annotated or
constructor-derived types are known, and dispatch helpers. `KatanaApp` action
routes require a four-symbol chain when distinct: enum action definition,
dispatch arm, handler, and final document/state/file/external operation.

### Action-Origin Classification

An `AppAction` definition or handler is not, by itself, a KLE feature. For every
reachable action variant, the generator must search backwards from each
construction site to a physical editor input source and emit
`action-origins.json`. Each origin is classified exactly once:

| Classification | Meaning | KLE root consequence |
| --- | --- | --- |
| `editor_direct_ui` | Current editor, editor-frame, or generic KUC-owned control constructs the action from pointer/keyboard/IME/AccessKit input. | Generate a KUC RawInput/AX leaf and only an opaque one-time KLE transit carrying host target, revision, correlation, and source. |
| `editor_shortcut` | KatanA shortcut arbitration accepts an editor-context shortcut and constructs the action. | Generate the KUC focus/input reservation leaf and actual KatanA shortcut/action effect; do not synthesize an independent control. |
| `editor_host_continuation` | A preceding editor host action internally schedules or transforms into this action. | Join it to the originating leaf; do not fabricate a second KLE event. |
| `user_mandated_extension` | An explicit v0.1.0 requirement adds a generic KUC control although the fixed KatanA editor has no direct UI construction site; an exact KatanA handler/effect exists. | Generate only the named requirement leaf with KUC RawInput/AX and actual host effect. It must never be reported as a KatanA-origin UI feature. |
| `unmounted_editor_state_continuation` | The action/state route is editor-related but no fixed-revision renderer or shortcut constructs the continuation. | Record source span and host state effect; reject a new KLE/KUC control until a source revision supplies an origin. |
| `host_only` | The source belongs to shell/workspace/application behavior rather than the document-editor surface. | Record a source-spanned exclusion rationale; no KLE root feature. |
| `unresolved` | The constructor, callback, macro, or shortcut origin cannot be resolved. | Fail the release gate. |

The mandatory regression case is dirty document close: `CloseDocument` has a
document TabBar origin; at this revision it may set
`pending_close_confirm`; `ForceCloseDocument` consumes that state but has no
renderer or shortcut construction site. The former is `editor_direct_ui`; the
latter is `unmounted_editor_state_continuation`. The generator must reject both
a false "complete close dialog" claim and a locally added KLE confirmation
widget. This classification is also mandatory for document-tab groups,
preview/slideshow actions, source address, diagnostics, command inventory, and
all external native actions.

The only initial `user_mandated_extension` registrations are
`replace.current` and `replace.all` in
`docs/v0-1-0-user-mandated-leaves.json`. The fixed KatanA revision has
`AppAction::ReplaceText` and its handler but no replace-control construction
site under the editor views. The generator records the user requirement span,
KUC `SearchStrip` event, KLE single-consumption forwarding span, and exact host
definition/dispatch/handler/effect spans. It must reject an unregistered user
extension, a generic extension count, a source-derived KatanA UI claim, a KLE
match/range/byte-conversion implementation, or an action without the actual
host effect.

### Opaque Authoring-Target Closure

KatanA `MarkdownAuthoringOp` and `CodeBlockKind` are source-domain types. The
generator must enumerate their current variants and every editor construction
site, then emit one action-origin and one leaf per reachable operation/code
kind. The generated manifest binds that source entry to the host-projected
opaque authoring target, KUC RawInput/AccessKit activation, KLE one-time
forwarding span, `AppAction::AuthorMarkdown` dispatch/handler, and the final
document/cursor observation.

KLE must not represent this coverage with `EditorAuthoringCommand`,
`EditorCodeBlockKind`, a command `String`, a code-kind/info-string conversion,
or a duplicate Markdown enum. The checker rejects a target fabricated outside
the current host projection, a KLE command-ID lookup, a target with no exact
source-origin binding, or a visible operation that has no distinct host effect.
An inventory count is not sufficient: every source variant and physical route
must be materialized separately.

The resolver may add a conservative candidate set. It must not choose one
candidate and discard the rest. Every unresolved or ambiguous candidate remains
in `source-closure.json.unresolved_edges` and fails the release gate until an
exact continuation or a source-spanned host-boundary rationale is recorded.

## Phase 3: Closure Expansion

For each discovered edge, recursively include the destination source file and
item. The following edge classes force expansion even when their destination
is outside `views/panels/editor`:

- UI host/input, focus, pointer, keyboard, scroll, accessibility, and visible
  state;
- action definition, dispatch, handler, document mutation, state mutation,
  save, refresh, undo, preview, diagnostics, image ingest, and browser/file
  boundary;
- command inventory, shortcut arbitration, search, tab/document navigation,
  editor-frame breadcrumb/source-address navigation, split/view layout,
  localization/presentation source, and test oracle;
- KatanA host external operations such as native dialog, clipboard, filesystem,
  or browser launch.

An external OS operation terminates only at the first KatanA source symbol that
invokes it. That termination records the external effect class but does not
close the related leaf. The leaf remains pending until the matching native or
in-process host execution record exists.

An external UI API that directly changes editor text, selection, history,
focus, input consumption, platform output, accessibility or visible layout is
not an OS-effect terminus. The generator resolves it against the fixed
`Cargo.lock` package/version/source/checksum and records the exact crate-source
file hashes and symbol root as an `external_ui_semantic_dependency`. The current
mandatory seeds are KatanA's `egui 0.36.1` `TextEdit::multiline`,
`TextEdit::{load_state,store_state}`, `Event::Paste`, and
`InputState::consume_shortcut` calls inside typed `Context::input_mut` closures.
Absence of the exact dependency source
or executable reference oracle is unresolved; a guessed library behavior or a
KLE egui fallback is invalid.

### Editor-Frame Sibling Classification

The generator begins from the editor-frame routes enumerated in the source
universe. A sibling may be classified as `editor_frame_behavior` only when it
is joined to a generated KLE/KUC/host leaf, or `host_only_sibling` only when a
source span and rationale prove it operates an application shell concern that
does not alter the document-editor surface. `WorkspaceToolbar` is the current
required `host_only_sibling` example. The document `TabBar`, `Breadcrumbs`,
`UrlSourceBar`, document search and status/problems routes are not eligible for
that classification. Reclassifying one as host-only, omitting its action
continuation, or treating the entire tab toolbar as a single leaf is a
fail-closed error.

### Breadcrumb And Source-Address Closure Rule

For `Breadcrumbs`, the generator must traverse both the visible presentation
branches and the nested `BreadcrumbMenu` recursion. It emits distinct retained
leaves for virtual final labels, slash/backslash and empty-segment handling,
no-workspace labels, final labels, menu open/empty/disclosure, and candidate
focus. Each reachable file candidate becomes an independent host-effect leaf:
the trace must include `SelectDocument`, dispatch,
`handle_action_select_document`, `DocumentOps::handle_select_document`, and the
resulting active-document/preview/search/diagnostic continuation. A directory
is a disclosure path, never an inferred document-selection path. KLE path
splitting, workspace-tree traversal, candidate filtering, `PathBuf`, and
coordinate fixtures are rejection conditions.

For `UrlSourceBar`, the generator emits independent leaves for button submit,
lost-focus Enter submit, blank/whitespace rejection, history absent/open/select
without submit, and KUC text/preedit/focus retention. The submit continuation
must begin at `OpenUrl` and enumerate exactly: `file://` recognition; invalid
non-file URL; local validation, canonicalization, openability, and canonical
URL failures; local-document and local-HTML admission/read failure; HTTP
admission; pending request; HTML/browser-source success or failure; binary
document-surface success or failure; collector error; timeout; and disconnected
request. A source-address text value is permitted only as the one-time,
non-Clone/non-Serialize input transport between KUC and the unchanged host; the
generator rejects KLE inspection, persistence, normalization, hashing, logging,
URL parsing, state mutation, direct `UrlTabState` setup, response injection,
and HTTP/KRR/browser mocks. Host evidence uses a real temporary `file://` path
or a real loopback HTTP listener with request/response hashes.

## Phase 4: Decisive Branch Catalog

For every reachable item, catalog all decisions that can alter visible state,
typed action, document/state/file effect, or accessibility result:

- `if` / `else if` / `else`, including boolean guard operands;
- every `match` arm and match guard;
- early `return`, `break`, `continue`, and `?` propagation that changes an
  observable path;
- loop entry/empty/non-empty branches where iteration changes UI or effects;
- `let ... else`, short-circuit control flow, and cfg-selected platform branch.

Each branch has a stable ID from source path, enclosing symbol, AST span, and
ordinal. It is classified as one or more observable leafs or one helper. A
helper classification must give its callers and prove it has no independent
visible/action/effect outcome. The catalog contains the source excerpt hash so
edited conditions cannot retain stale classification.

### Command Inventory Expansion

The generator must expand data-driven command paths from their source values,
not validate a remembered count. At the recorded revision, these are important
seeds:

- `views/panels/editor/toolbar.rs:EditorToolbar::{inline_group,heading_group,
  list_group,block_group,image_group}` supplies separate direct actions and
  selection-enabled guards;
- `views/panels/editor/context_menu.rs:EditorContextMenu::{render,
  render_inline,render_structure,render_headings,render_blocks,render_insert}`
  supplies root order, extension/read-only format guard, nested command path,
  enablement and close effects;
- `views/panels/editor/code_block_menu.rs:CodeBlockMenuOps::show_items` calls
  `CodeBlockKind::all()`. Resolve that definition and produce one branch/leaf
  candidate for every current enum value, plus the no-item and unresolvable
  inventory paths;
- `context_menu_image_ingest.rs:EditorContextMenuImageIngestOps::render`
  supplies file and clipboard-image paths with the host payload enablement
  guard.

It must also parse every `CommandInventoryItem` in the current
`state/command_inventory/{file,edit,view}_commands.rs`, trace its availability
closure and each declared shortcut through `shell_ui_shortcuts`, and emit one
physical-origin record per command/shortcut/context route. `file.save`,
`file.close_document`, `file.restore_closed`, `view.doc_search`,
`view.refresh_document`, `edit.ingest_clipboard_image`, and each authoring
shortcut are current editor-surface seeds. A command that instead operates only
the workspace/application shell is not silently dropped: it is classified
`host_only` with its source span and rationale. The same action can have both a
host-only menu origin and an editor shortcut/direct origin; each origin is
independent and neither classification suppresses the other.

`PreviewSidePanels` is a direct editor root, not an application-shell sibling.
The generator expands the TOC capability guard; pinned leading/trailing,
hover-open/dismiss/cooldown and empty-source branches; hierarchy leaf/parent
selection, accordion disclosure, expand/collapse-all, active anchor source
priority/fallback, `state/toc.rs` candidate stability/suppression/generation/
reset, active auto-scroll, vertical-guide and empty-outline
branches; all seven rail controls; Export/Story/Tools hover-delay and sibling
replacement branches; each four Export format item; story toggles/slideshow
request; tools commands; metadata absent/present guard; and every TOC
selection/view-mode scroll guard. Generic rail/outline mechanics map to
`kuc_retained_ui_effect`; each opaque host target maps to the unchanged KatanA
action/handler or actual KatanA UI-input relay. A direct KLE panel, outline,
hover clock, coordinate input, `TocPosition`, anchor, TOC line/scroll mutation,
or action-only fixture is rejected by the generated boundary scan.

The branch catalog records presentation order as a source statement sequence,
not as a test's expected label list. A newly added command, code kind, guard,
or nested level changes the catalog and leaves an unclassified branch until a
KUC presentation, class-appropriate neutral KLE relay or retained-UI boundary,
declared effect, execution record, and media stage are added. Pointer, keyboard,
and AccessKit selection are separate execution records for each interactable
leaf; one route cannot certify another.

## Phase 5: Parity Evidence Join

The generator joins each leaf to exactly one current KUC root frame and one
same-run execution record. For `editor_direct_ui`, `editor_host_continuation`,
and `user_mandated_extension`, the execution starts with public KLE `RawInput`,
invokes `EguiLanguageEditor::show` once for that stage, has the current opaque
KUC frame, record, and AccessKit node, and reaches the declared effect class.
`kuc_retained_ui_effect` requires the exact current KUC state transition and an
already-bootstrapped, read-only KatanA no-mutation observation; it must not
manufacture a KatanA action or mutate `AppState`. `in_process_host_effect` and
`native_external_host_effect` require their exact KatanA action/handler/effect
paths, naturally emitted after a source-derived physical KatanA UI input route.
The host bridge records only opaque correlation; `trigger_action`, direct
`AppAction`, `pending_action`, fixed coordinates, or action-only fixtures are
rejection conditions.

An `editor_shortcut` constructed by KatanA's own `shell_ui_shortcuts` is a
separate source-owned execution mode. KatanA does not mount KLE at this
revision, so no evidence may claim that the KLE `RawInput` directly drove that
router. The same-run record instead contains: (1) a physical key `RawInput`
into unchanged KatanA and its actual context/availability/reservation/ordering
decision plus action/handler/effect, and (2) the current KUC root/AccessKit
focus or retained-input record for the same bootstrapped document/revision.
It records their correlation as an evidence join, not as a KLE request or a
synthetic causal relay. A KLE/KUC shortcut parser, direct KatanA action
injection, shared keyboard fixture, or a claim of mounted KLE integration is
rejected. This does not reduce the required physical KatanA shortcut proof.

For ordinary content, the only accepted host route is KatanA `UpdateBuffer`.
For replace current/all it is `ReplaceText`. Host-originated external changes
flow only KatanA to KLE. Clipboard uses a correlated one-time request and
resolution transaction. The execution validator rejects dropped, duplicated,
locally completed, cross-document, stale, or simulator-only events.

`native_external_host_effect` records the separate native-driver trace, KatanA
action/frame trace, and same-run document/asset/no-mutation assertion. Pending
dialog state, ignored tests, fixed sleeps, coordinate clicks, manual steps, or
payload injection cannot produce a passing record.

## Phase 6: Fail-Closed Validation

The checker fails for any one of the following:

- source revision, file SHA-256, tree fingerprint, or generator mismatch;
- missing/duplicate release profile, runner/cfg/Cargo-resolution fingerprint
  mismatch, default-host graph reuse, unclassified inactive `cfg` candidate, or
  profile-only branch/edge omitted from the union;
- locked external UI package/version/source/checksum, source hash, or semantic
  symbol mismatch for a behavior-bearing dependency;
- parse failure, missing module file, unresolved/ambiguous edge, or unexpanded
  behavior-bearing call;
- decisive branch with no leaf/helper classification, or helper with an
  insufficient rationale;
- aggregate leaf, shared selector, fixed file/leaf count, absent deferred file,
  or source-marker-only row;
- missing KUC opaque root, current record, AccessKit node, class-appropriate
  mapping/effect, unchanged-host observation for retained UI, or numbered
  Storybook stage;
- `Unsupported`, pending native dialog, ignored live test, fallback artifact,
  or mixed/stale proof fingerprint for a required leaf.

The only passing result has no unresolved edges, unclassified branches,
pending/failing leafs, or evidence mismatches.

## Required Generator Tests

- discovery of nested and out-of-directory modules from each initial root;
- aliases, `crate`/`self`/`super`, concrete `self` methods, trait methods, and
  action enum dispatch/handler/effect chains;
- unresolved receiver, glob ambiguity, dynamic callback, macro expansion, cfg
  branch across macOS/Windows/Linux, missing profile, inactive-candidate
  omission, missing file, stale source hash, nonexistent deferred file, and
  missing/mismatched locked external UI dependency source;
- branch enumeration for guard, early return, `let else`, short circuit, and
  platform branch;
- rejection of fixed counts, shared selectors, aggregate leafs, source-only
  evidence, old KLE aggregate artifacts, and Storybook fallback pixels;
- join failures for wrong tree/generator fingerprint, missing current KUC
  record/AccessKit, duplicated/dropped event, wrong host action, and stale media
  hash;
- a mutation test for every KatanA direct source seed and every action-route
  seed, plus every external UI semantic dependency seed: deleting or changing
  that route must make the closure or leaf join fail.

These tests are part of the release gate. No exclusion, allowlist, lint
suppression, or reduced fixture set can convert a failure into acceptance.
