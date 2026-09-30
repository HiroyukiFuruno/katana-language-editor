# KLE v0.1.0 Neutral Host-Request Design

## Status

This is an implementation contract for Spark. It records a source-observed
boundary defect in the current KLE action API and is **not** implementation or
release evidence. KatanA at
`4f6a6287c650a38633c7baeb544a92e739c68567` remains read-only.

The required direction is one-way:

```text
KatanA host descriptors
        -> KLE mechanical projection
        -> KUC opaque retained root
        -> typed KUC event
        -> KLE one-time typed forwarding
        -> unchanged KatanA action/host service
```

KLE must not turn host identifiers into paths, URLs, tree nodes, diagnostics,
clipboard payloads, history entries, or browser state. KUC must not receive
KatanA-specific data or acquire host data. The host is the only authority that
interprets an identifier or source-address submission.

## Measured Defects

`crates/katana-language-editor/src/actions.rs:27-68` currently exposes the
following release-blocking API shapes:

| Current API shape | Why it violates the boundary | Required replacement |
| --- | --- | --- |
| `IngestClipboardFileUrls { urls: Vec<String> }` | KLE receives and can retain, inspect, log, or translate OS clipboard file URLs. | The existing payload-free correlated clipboard request/resolution contract; only the unchanged KatanA host reads the clipboard. |
| `OpenDiagnosticDocumentation { diagnostic_id, url }` | A KLE public action carries a host-owned documentation URL. | An opaque diagnostic target only. The host resolves the documentation link and runs `OpenLinterDoc`. |
| `HostCommand { command: String }` | An unbounded string permits an unclosed, unverifiable host-command surface. | One source-derived closed typed variant per supported action, or no exported action. |
| `RunAuthoringCommand { command: String }` | A free-form command name is not a closed source inventory. | One host-issued opaque authoring target with descriptor revision and interaction correlation. The unchanged host resolves it to the source-derived `AppAction::AuthorMarkdown`; KLE does not duplicate `MarkdownAuthoringOp`. |
| `authoring.rs::EditorAuthoringCommand` / `EditorCodeBlockKind` | `id`, `info_string`, and shortcut `String` values let KLE construct, inspect, serialize, or remap host command semantics. These structs are current migration residue, not a closed target contract. | Opaque command-item and authoring-target descriptors supplied only by the current host projection. The generated source closure, rather than a KLE enum or string catalogue, proves all 14 operations and all 17 code kinds. |
| `EditorAuthoringMenuState`, `EditorAuthoringMenuLifecycle`, and their `CursorPosition` anchor | KLE owns generic floating-menu visibility, code submenu lifetime, focus transitions, suppression, and placement input. | KUC `CommandChrome` owns retained toolbar/dropdown/focus/outside/Escape lifecycle and derives its anchor from its own TextSurface layout. KLE has no menu state, lifecycle, cursor anchor, or placement DTO. |
| `ApplyDiagnosticFix` / `ApplyAllDiagnosticFixes` / `ApplyProblemsFixes` with `String` or `Vec<String>` IDs | KLE can create linter batches and select diagnostics outside a current host projection. | One opaque diagnostic/fix-operation target and descriptor revision; the host derives all fixes, files, batches, and documentation. |
| `ActivateGutterLine { line_index }` | A raw line index makes KLE an owner of gutter-to-document mapping. | An opaque host-projected gutter target and revision. |
| `EditorScrollSyncRequest`, `EditorSelectAndJumpRequest`, and `EditorDocumentSearchNavigation` line/segment/progress fields | KLE persists or normalizes viewport coordinates, anchors, preview fallback, and search targets. | A KUC-owned one-shot viewport/search event with opaque host source target; the host owns logical mapping, anchors, and KatanA action details. |
| `EditorEvent` payload variants and `SearchQuery` derive `Clone`/`Serialize` around raw content, origin, and query strings | KLE can retain or serialize typed user input as editor state or test artifacts. | Typed, single-consumption input transport with redacted evidence; KLE has no content/query/origin persistence. |
| Missing document-tab request family | KUC cannot report select/close/pin/restore/reorder/group commands without KLE inventing paths, indices, group membership, colors, or persistence. | Closed tab/group requests carrying only host-projected opaque tab, group, placement, and swatch targets plus revision/correlation. |
| Missing breadcrumb/document-tab selection request | KUC cannot report selection without inventing a KatanA path/tree contract. | A closed request carrying only a host-projected opaque selection target, descriptor revision, source and correlation. |
| Missing source-address submit request | The current API cannot transfer one user-entered source value to the actual KatanA `OpenUrl` route. | A separate non-persistent, non-serializable one-shot submission event; it is not an `EditorAction`. |
| Missing preview-side-rail request boundary | KLE has no closed way to distinguish generic rail mechanics from KatanA refresh/search/export/story/tools/meta/TOC effects without leaking panel names, formats, paths or lines. | KUC `PreviewSideRail` retains generic panel mechanics. It emits either no host event or one opaque host-issued target/revision/correlation; the host resolves each target. |
| Missing outline-selection request boundary | KLE could otherwise turn a KatanA TOC row into a line, anchor, index, source-priority or scroll request. | KUC `OutlineNavigator` receives only opaque revisioned item targets and presentation nesting. KUC retains disclosure/active presentation/scroll; a selected item emits one opaque target/revision/correlation that the host resolves against current KatanA outline state. |

`EditorDocumentIdentity` at
`crates/katana-language-editor/src/document_state.rs:12-40` remains the
neutral document identity. KLE may compare it for equality and pass it back to
the host, but must not split, normalize, or derive a filesystem path from either
field. Its values originate in a host descriptor.

## Public Contract

### Opaque host descriptors

Spark must introduce opaque identifier and correlation types in KLE rather
than public `String` fields for host lookup values. Their exact Rust names may
follow local conventions, but their semantics are fixed here:

| Type role | Required contents | KLE-permitted operation | Prohibited operation |
| --- | --- | --- | --- |
| diagnostic target | Host-issued opaque key and descriptor revision | Equality, forwarding, stale-revision rejection | Reading or constructing URL, rule metadata, batch membership, path, or documentation payload. |
| gutter target | Host-issued opaque target and descriptor revision | Equality, forwarding | Deriving line/document/scroll coordinates from a string. |
| document target | `EditorDocumentIdentity`, descriptor revision, interaction correlation | Equality, forwarding | Path split, tree traversal, workspace lookup, document creation, or selection state mutation. |
| authoring target | Host-issued opaque command target, descriptor revision, interaction correlation | Equality, one-time forwarding | Command-ID lookup, Markdown-operation/code-kind enum, shortcut/info-string decoding, transform or KUC menu-state ownership. |
| rail or outline target | Host-issued opaque item target, descriptor revision, interaction correlation | Equality, one-time forwarding | Panel name, export format, document path, TOC index/level/line/anchor/source priority, `TocState`, timer, scroll or active-target mutation. |
| interaction correlation | Monotonic opaque ID scoped to one rendered root revision | Associate a KUC event with one host response | Reuse across leaf executions or use as user-visible state. |

Opaque identifiers may be created only while projecting current host state into
the KUC root. They may not be deserialized from an untrusted UI payload or
manufactured by Storybook, a parity fixture, or the host-E2E runner.

### Closed action surface

`EditorAction` remains suitable only for closed, replay-safe typed requests.
Spark must replace the current open string and URL-bearing variants with a
source-derived exhaustive set. At minimum, the action surface must contain
separate typed requests for save, format, opaque authoring-target activation,
image-file intent, clipboard-image intent, diagnostic fix/fix-all/documentation by opaque
target, gutter activation by opaque target, document search direction, scroll
sync, select-and-jump, view mode/split direction, document selection, and the
complete closed document-tab operation family defined below.

The following KUC-retained interactions must **not** become KLE actions:

- opening or closing the breadcrumb candidate list;
- source-address focus, text editing, IME preedit, history disclosure and
  history selection before submit;
- Problems panel visibility, scope choice, disclosure, expansion, preview,
  hover, and focus;
- preview-side-rail panel open/close, hover delay, sibling replacement,
  tooltip, focus and outside/Escape dismissal;
- outline accordion disclosure, expand/collapse-all, active presentation,
  generic auto-scroll and empty-outline state;
- close of a KUC-only transient overlay.

Those leaves produce only a KUC root transition plus an unchanged bootstrapped
KatanA host observation. `ToggleProblemsPanel`, `SetProblemsScope`, and
`CloseTransientUi` must therefore be removed from the KLE host-action release
path when the opaque KUC root supersedes the current local controls.

`HostCommand`, any equivalent stringly escape hatch, and any fallback arm that
accepts an unknown request are prohibited. A KatanA capability without a
source-derived KUC event and closed KLE request remains unavailable; it is not
represented by an arbitrary command string.

### Single-consumption input transport

Raw user input may cross KLE only as a typed, source-derived, single-consumption
transport from a current KUC opaque root event to the host bridge. It is never
an arbitrary `EditorAction { String }`. This rule covers the current editor
surface rather than only one control:

| Transport kind | KUC-owned input | Host-only interpretation/effect | KLE prohibition |
| --- | --- | --- | --- |
| document edit | Text/IME/clipboard text change and selection correlation | Exact KatanA `UpdateBuffer` / history / dirty / refresh route | Content state, origin string, edit transform, clipboard text, undo state. |
| search and replacement | Query, replacement text, find/replace control state and current-root correlation | Exact KatanA document search or `ReplaceText` route after the host verifies its current snapshot and character/byte range | Query/match/replace state, string matching, range conversion, text mutation. |
| viewport, gutter, preview select-and-jump | Scroll/resize/hit/semantic source target and current-root correlation | KatanA scroll coordinator or `SelectDocumentAndJump` mapping | Physical/logical geometry, line index, anchor, fallback mode, scroll cache. |
| source address | Address text and KUC button/Enter correlation | Unchanged KatanA `OpenUrl` | URL parsing/history/fetch/source payload/browser state. |
| TabStrip group name | Generic inline text edit and opaque group target | Unchanged KatanA `RenameTabGroup` after host target lookup | Group name, membership, persistence, path, color hex. |

Each payload-bearing transport is a dedicated variant, not a generic command
string. It owns its bytes only until a consuming host-bridge call completes and
contains the KUC root revision and opaque correlation. New transport records
are not `Clone`, `Serialize`, or `Deserialize`. The existing payload-bearing
`EditorEvent` and `SearchQuery` API must be replaced before release so that a
KLE state, action queue, Storybook scenario, manifest, log, or retry queue
cannot retain raw input. Public non-payload status/output descriptors may keep
their normal data traits.

The source-address transport contains no parsed scheme, pathname, URL object,
history entry, document payload, result, or browser field. The group-name
transport contains no membership list, persisted tab state, path, document
payload, color hex string, or filesystem field. Viewport/search/gutter
transports carry opaque host source targets rather than KLE line, segment,
progress, match, or range values.

The host verifies root revision and correlation before consuming any transport.
Stale, duplicate, cancelled, and cross-document transports cannot issue a
KatanA action and must leave relevant host state unchanged. Blank handling is
source-specific: source-address whitespace rejection remains a no-action KUC
leaf, while a changed TabStrip group name, including an empty value if KatanA
currently emits it, follows the exact KatanA route. No generic blank rejection
may alter that distinction.

The host bridge passes source-address text unchanged to `AppAction::OpenUrl`,
group-name text unchanged to `AppAction::RenameTabGroup` after opaque lookup,
and document/search text only to their exact source-derived KatanA routes. The
host responds by projecting the next opaque editor-frame descriptor and an
effect record. KLE never stores a source result, query, group name, document
content, or tab state.

Raw input is forbidden in test snapshots, parity manifests, screenshots, media
filenames, and assertion messages. Evidence records input kind, correlation,
revision, byte length, and SHA-256 only.

### Document selection

KUC's generic `TabStrip` and `BreadcrumbNavigator` receive only
host-projected opaque document descriptors. Selecting a tab or a breadcrumb
candidate emits the same closed document-selection request with a distinct
`EditorActionSource` (`DocumentTabs` or `BreadcrumbNavigator`), the descriptor
revision, correlation, and opaque selection target.

KLE does no candidate lookup. The host validates the revision, looks up the
identity in the host projection it issued, then maps it to unchanged KatanA
`SelectDocument`. Virtual/no-workspace final breadcrumbs and menu-open/close
are retained KUC leaves and do not issue this request.

### Complete document TabStrip request family

The KatanA document TabBar is in KLE scope. A selection-only request is
insufficient. KUC generic `TabStrip` events must map to a closed KLE tab request
family with one source-derived variant for every host-effect operation below:

| KatanA operation family | KLE request contents | KLE must not contain |
| --- | --- | --- |
| select and previous/next traversal | Opaque document target or closed traversal direction, revision/correlation | `PathBuf`, tab index, active-document state. |
| close, close-other/all/left/right, restore | Opaque tab target where required, revision/correlation | Dirty confirmation policy, recently-closed stack, filesystem state. |
| pin/unpin | Opaque document target, revision/correlation | Pinned flag mutation or tab persistence. |
| reorder and add/remove group by drag | Opaque moving tab, opaque placement/group target, revision/correlation | Visual/physical index conversion, geometry, membership computation, virtual-document exception logic. |
| create/add/remove/ungroup/close/collapse group | Opaque tab/group/candidate targets, revision/correlation | Group membership, group persistence, group names/colors, or workspace state. |
| rename group | The dedicated one-shot group-name record above | Retained editable text or group name. |
| recolor group | Opaque current swatch target, group target, revision/correlation | Hex color parsing/storage or palette geometry. |

KUC owns drag hit testing, overflow, context menu, generic inline input,
palette selection, and all retained state. A color palette is projected as
opaque host-issued swatch IDs; KUC returns the selected swatch ID and the host
maps it to its existing color value. KLE must never carry a color hex value.
The host resolves every opaque request against the projection it issued, then
performs unchanged KatanA `Close*`, `TogglePinDocument`, `RestoreClosedDocument`,
`ReorderDocument`, `Create/Add/Remove/Rename/Recolor/Ungroup/Close/ToggleCollapse`
or selection action. The current fixed-revision `ForceCloseDocument` has no
KLE-root UI origin after a dirty close, so it is not a KLE request; KLE/KUC must
not invent a confirmation dialog. KLE must not route any tab operation through
`HostCommand`.

### Diagnostics and clipboard

Diagnostic IDs are opaque host descriptor keys. `OpenDiagnosticDocumentation`
and all fix actions carry only such keys and revisions. Their KatanA handler
resolves documentation URLs, lint batches, files, and review/browser effects.
KLE does not hold `url`, path, batch, rule, or linter data.

Clipboard raw image, file-list image, and file-URL image continue to use the
payload-free `EditorClipboardPasteRequest` correlation. The native host driver
prepares the operating-system clipboard; KatanA alone reads and prioritizes it.
No action or event in KLE/KUC may contain image bytes, file-list entries, or
file URLs.

## Mapping Lifecycle

1. The unchanged KatanA host creates a current editor-frame projection. It
   includes opaque document, breadcrumb, source-address, diagnostic, gutter,
   authoring, clipboard, rail, and outline correlation descriptors, but no
   KLE-derived path, URL, command string, code kind, Markdown operation,
   export format, TOC line, anchor, source priority, or scroll data.
2. KLE mechanically converts that projection to the KUC opaque root input. It
   may carry identity/key/revision values verbatim and must not derive state.
3. The KUC root consumes physical `RawInput`, owns layout, IME, glyphs, focus,
   menus, tabs, history disclosure, and retained transitions, then emits a
   typed event.
4. KLE either records the KUC-retained transition with no host action, forwards
   a closed `EditorAction`, or consumes and immediately forwards one
   source-derived input transport.
5. The unchanged KatanA host validates correlation/revision, executes the
   exact source-derived action/effect, and supplies the next projection.
6. The execution record joins the same KUC record/AccessKit node, KLE request
   fingerprint, KatanA action/handler/effect span, and final host observation.
   A record cannot claim both a retained and host effect for one leaf.

## Required Strict Checks

Spark must implement all of these before any action API replacement is accepted:

| Check | Required rejection or assertion |
| --- | --- |
| API AST check | Reject `HostCommand`, `urls`, public action `url` fields, arbitrary authoring command strings, `EditorAuthoringCommand`, `EditorCodeBlockKind`, `EditorAuthoringMenuState`, `EditorAuthoringMenuLifecycle`, KLE cursor anchors, and action variants for KUC-retained panel/breadcrumb/source-address/outline UI. |
| input transport AST check | Reject generic string commands, `Clone`, `Serialize`, `Deserialize`, persistent state fields, snapshot/manifest serialization, logs, and non-redacted debug output for document/search/replacement/source-address/group-name payloads. Reject KLE line/segment/progress/range/anchor computation for viewport, gutter, select-and-jump, rail, and outline targets. |
| mapping exhaustiveness | The generated KatanA AST closure verifies every currently reachable authoring construction through its opaque host target to one exact `AppAction::AuthorMarkdown` value and handler branch. KLE maps no Markdown enum or command string, and no required leaf may take `UnsupportedAction`. |
| opaque-target provenance | Reject KUC/KLE construction of diagnostic/gutter/document/rail/outline target strings outside the current host projection. Reject direct `UrlTabState`, document-tree, workspace, path, TOC, anchor, line, or scroll mutation in KLE and the harness. |
| stale and duplicate | For every opaque target and input transport, run stale revision, stale correlation, duplicate correlation, document switch, cancellation, and source-specific blank cases; assert the exact KatanA mutation or no-mutation outcome. |
| actual host effects | Drive KUC RawInput through public KLE, then unchanged KatanA document-tab actions, `SelectDocument`, or `OpenUrl`; use a real temporary file and loopback HTTP only. Do not mock KatanA, `ehttp`, KRR, or browser state. |
| observability hygiene | Store only input kind, correlation, revision, input length, and SHA-256 in manifests/artifacts. Assert that raw document/search/replacement/source input, paths, URLs, clipboard data, and documentation URLs are absent. |

The generated source-closure manifest must contain one individual record for
each document-tab selection, breadcrumb candidate selection, source-address
button submit, source-address Enter submit, blank rejection, invalid input,
local-file branch, HTTP success/error/timeout/disconnect branch, stale
revision, duplicate correlation, and cancellation branch. Aggregate counts,
shared simulator callbacks, or a Storybook-only assertion are not acceptance
evidence.

## Implementation Ownership

| Repository | Required work | Explicitly forbidden |
| --- | --- | --- |
| KUC | Generic `BreadcrumbNavigator`, `SourceAddressBar`, `TabStrip`, `PreviewSideRail`, `OutlineNavigator`, opaque root events, retained state, RawInput/AccessKit/artifact tests. | KatanA action types, URL/path/TOC validation, filesystem/network/browser/clipboard acquisition, KLE wrappers. |
| KLE | Transparent opaque host-descriptor and one-shot transport transit; source-derived evidence join. | URL parsing/history, source/browser/TOC state, path/tree/linter/clipboard payload logic, local generic UI or rendering. |
| KatanA host (read-only) | Existing `SelectDocument`, `OpenUrl`, diagnostic, image, document, and browser effects. | Any source, manifest, test hook, adapter, or branch modification for this release. |
| KLE host-E2E | Real public KLE/KatanA execution, host descriptor lookup, native/loopback fixture setup, strict records. | Fabricated requests, direct app-state mutation, KatanA/KRR/ehttp mocks, raw source/payload persistence. |

The work is implementation work for Spark only. The main agent may update this
design, source inventory, OpenSpec tasks, and review evidence; it must not
implement KUC/KLE runtime code.
