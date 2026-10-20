# Locked External UI Semantic Reference Audit

## Status

This is a source audit for KLE v0.1.0 and not implementation evidence. KatanA
remains read-only at `4f6a6287c650a38633c7baeb544a92e739c68567`. Every listed
behavior is release-blocking until a source-derived leaf has a current KUC
opaque-root record, AccessKit evidence, public KLE input, and its KLE-owned
observable outcome. Actual KatanA host effects are verified after public KLE
v0.1.0 adoption under KatanA #336, not as a KLE publication prerequisite.

## Locked Reference Identity

KatanA's `Cargo.lock` pins the behavior-bearing UI packages below. Their
registry source is a reference oracle only; KLE must never retain their text
renderer, glyph fallback, state cache, shape output, or clipboard code.

| Package | Version | Cargo checksum | Locally audited source file | SHA-256 |
| --- | --- | --- | --- | --- |
| `eframe` | `0.36.1` | `2afc0cbcdb6896b7bfb1dbbebaf7b9af9635ff38fd01e89bdd0174c1717b1857` | integration boundary selected by KatanA | lock-pinned package record required at generation time |
| `egui` | `0.36.1` | `c977ac91dfaa651633fd9722e4ce9ccb32cda4c748b89a5cb57e504036e37c13` | `src/widgets/text_edit/builder.rs` | `1c2198aa1c397d40e0e384cb6e69252822a6bdf870b8b3d9ddafb2dfd5f98cbd` |
| `egui` | `0.36.1` | same package | `src/widgets/text_edit/state.rs` | `b94badf39aaa4f7628b4b24695466a9b368c76caf0d9821df87bc90faab701b1` |
| `egui` | `0.36.1` | same package | `src/context.rs` | `3bd6ae9c54157353012d0f978b1a4cc1c41de8fbce333d7596443583d82c9e03` |
| `egui` | `0.36.1` | same package | `src/input_state/mod.rs` | `17b17cab696f45a6044a891e8fe2ef2767da126fb04272142b7de626c6ae34c0` |
| `egui` | `0.36.1` | same package | `src/data/input/event.rs` | `f2d802df6cf308d457c888742152848e3394e6d86df462c06caeec1d18f3d147` |

The earlier generator required an expanded `.cargo-checksum.json`, but the
actual registry source does not contain this vendor-directory metadata. The
2026-09-05 r18/r19 diagnostic explicitly rejected that layout and recorded no
source symbols. This was a KLE verifier assumption error, not a missing KUC
capability. The correction uses the authenticated archive as the primary source
of file identity; any optional vendor checksum manifest adds constraints rather
than providing the origin of trust. See the [Cargo source replacement
documentation](https://doc.rust-lang.org/cargo/reference/source-replacement.html).

On 2026-09-05, a separate read-only check hashed the cached `egui-0.36.1.crate`
archive and the three named members streamed directly from that archive. The
archive hash matched the lock checksum above, and all three member hashes
matched this table. This is bounded audit evidence, not implementation of the
generator's archive-to-source identity gate or proof of transitive completeness.
A local cache path is not an identity field in generated evidence.

The corrected r20 diagnostic subsequently authenticated the real registry
archive and expanded Rust source set. It emitted the five exact entry symbols
from builder, input_state, and event source files, bound their report fingerprint
into the diagnostic root, and retained only the explicit diagnostic limitation
on invocation/transitive closure. This proves the input-identity correction on
that run, not editor parity, platform execution, or final gate completion.

## KatanA Invocation Contract

The diagnostic now feeds the scanner's fixed-blob-verified source bytes into
the bounded invocation visitor. The r23 run exposed a missing production edge:
`editor/paste.rs` uses `egui::Event::Paste` in tuple match patterns, not only
expressions. The correction records those as `event-pattern` with their exact
spans. Macro expansion, arbitrary Rust name resolution, and transitive semantic
closure remain unresolved; these records are not runtime execution receipts.

`TextEditRenderer::render` builds `TextEdit::multiline` with a
workspace/document-scoped ID, `interactive(!doc.is_reference)`, monospace font,
infinite desired width, 40 desired rows, the documented gutter margin, and no
frame. It performs KatanA image-paste interception before the widget and emits
`UpdateBuffer` after a normal text change. It does not install a custom
layouter. `EditorUndoOps` separately reads/writes the locked `TextEdit` state
for host-originated external changes.

Therefore the external semantic closure begins at these KatanA invocations:

| KatanA call | Locked external root | Required generated dependency edge |
| --- | --- | --- |
| `text_edit.rs:TextEditRenderer::render` | `TextEdit::multiline` and input handling | standard editing, selection, clipboard, history, IME, platform output |
| `editor_undo.rs:EditorUndoOps::{record_external_change,cursor_for_text}` | `TextEdit::{load_state,store_state}`, `TextEditState::undoer` | host-origin history, character cursor clamp, document/workspace identity |
| `editor/logic.rs:EditorLogicOps::apply_pending_cursor` | `TextEdit::{load_state,store_state}` | programmatic character-range cursor restore |
| `shell_ui/shell_ui_shortcuts.rs:command_shortcut_consumed` | `Context::input_mut` closure to `InputState::consume_shortcut` | post-arbitration event consumption with typed closure binding provenance |

## Measured TextEdit Branch Families

Every branch below becomes an independently fingerprinted source-derived leaf
using the fixed external source span. The table is not a static count and is
expanded again by the generator when the locked source changes.

| External branch | Observable KatanA reference behavior | Required KUC replacement and proof |
| --- | --- | --- |
| selection-only event | cursor/selection events update the character-safe cursor without text mutation | KUC owns grapheme-safe selection/caret and AX selection state; no `UpdateBuffer` is emitted. |
| copy selected / empty | non-empty selection issues one platform text copy; empty selection emits no copy | KUC emits its internal generic platform effect, KLE cannot inspect text, and root record proves zero or one effect. |
| cut selected / empty | selected text is copied then deleted; empty selection is a no-op | KUC emits one clipboard effect and one content transaction only for the selected editable case. |
| paste non-empty / empty | non-empty text replaces selection and, in multiline mode, preserves pasted line breaks; empty paste is a no-op | KUC waits for correlated KatanA host resolution so image precedence remains host-owned; text resolution creates exactly one mutation. |
| ordinary text / CR-LF suppression | non-empty `Event::Text` inserts text except bare CR/LF, which are handled as key input | KUC preserves Unicode scalar/grapheme behavior and emits no duplicate newline mutation. |
| Tab / Shift-Tab | KatanA uses plain `TextEdit::multiline` without enabling Tab indentation; the locked default `EventFilter { tab: false }` reserves Tab for focus navigation | forward/reverse focus-routing evidence and no editor indentation mutation; do not require an unreachable indentation branch as the reference behavior. |
| Enter | multiline Enter replaces selection and inserts one newline | KUC action, caret, history and KatanA `UpdateBuffer` result are joined. |
| redo | primary-Y and primary-Shift-Z apply the retained next state when it exists | KUC owns retained history and emits one `redo` content mutation; KLE cannot route it through a global command. |
| undo | primary-Z applies the retained previous state when it exists | KUC owns retained history and emits one `undo` content mutation with exact character-safe caret. |
| Backspace/Delete | selected text, scalar, word, paragraph, platform/Alt/Ctrl variants, and Windows Shift-Delete guard use distinct external branches | KUC replaces behavior through generic text operations; every branch receives a generated leaf instead of a single delete aggregate. |
| Ctrl-H/Ctrl-K/Ctrl-U/Ctrl-W | alternate delete keys have individual preceding-character, following-paragraph, preceding-paragraph, and previous-word effects | generated leafs require exact buffer/caret outcome and no shortcut-router double dispatch. |
| IME Enabled / Disabled notification | the owned-event match returns no mutation for both deprecated notifications; the notification itself does not reset composition | distinguish notification handling from loss of composition ownership; no fabricated commit or reset. |
| IME ownership lost | before event handling, lack of ownership changes cursor purpose to selection and collapses a non-empty cursor range to its primary cursor; unowned IME events do not enter the composition branch | KUC records ownership transition and selection/caret outcome without inventing a content commit. |
| IME empty/no active, newline | empty preedit/commit without active composition and newline preedit/commit do not mutate text | exact no-mutation record, including Japanese/VS16/ZWJ surrounding selection. |
| IME preedit | replaces prior preedit range, records active character range, and allows empty preedit to clear it | KUC owns preedit/active-range rendering, glyph shaping, caret and AX semantics through `PlatformFontCatalog`. |
| IME commit | clears the prior preedit/selection, inserts a non-empty commit, and changes cursor purpose to selection; empty commit during active composition clears that preedit, unlike empty commit without active composition | separate active-empty clearing from ignored inactive-empty input; verify the exact content/caret outcome and preserved `⭐️` VS16/ZWJ code points without duplicate forwarding. |
| IME delete surrounding | deletes requested character counts around the composition cursor | KUC uses Unicode-safe semantic positions and proves exact host buffer/caret result. |
| history feed | undo state is fed before and after input using frame time and cursor/text state | KUC records the equivalent retained transition once per accepted transaction; a host acknowledgement must not replay user input. |
| focus/interactivity gate | input handling runs only while the widget is interactive and focused; KatanA reference documents non-interactive as neither editable nor selectable | KUC root exposes read-only/focus semantic state and suppresses all mutation/clipboard/history paths accordingly. |

`cursor_range.on_event`, `check_for_mutating_key_press`, `TextBuffer` mutation
methods, and every transitive external symbol reached from the table are part
of the same closure. The generator must recurse through them rather than accept
this table as complete.

## Fixed Platform Bridge

The fixed KatanA Cargo manifest patches `egui-winit` to `vendor/egui-winit`.
Its lock entry is version `0.36.1` with no registry source/checksum, so upstream
registry bytes are not a valid replacement oracle. The explicit source root
is `vendor/egui-winit/src/lib.rs`, authenticated through the fixed Git blob.

`State::on_window_event` routes to `on_ime` and `on_keyboard_input`; the former
converts composition byte ranges to character ranges, including an explicit
Windows branch, and the latter constructs the clipboard text paste event.
The r24 diagnostic follows the declared modules into clipboard, dropped_file,
safe_area, and window_settings and records their fixed source hashes. It also
records the actual paste construction at `lib.rs:1028` and three production
paste match-pattern spans in KatanA `editor/paste.rs`.

This fills a source-inventory omission, not an OS test gap. The next semantic
graph must connect this bridge to `TextEdit::show`, `events`, `CCursorRange`,
`TextBuffer`, and `Undoer`, while independently retaining shortcut arbitration
and state load/store roots. Registry winit, clipboard, and accessibility
dependencies still require their own origin and semantic evidence. Generic
replacement behavior remains KUC-owned; native proof and full leaf enrichment
are not supplied by this audit.

## KUC/KLE/Host Boundary

KUC implements all generic behavior in the table through its retained
`TextSurface` and shared platform catalog. It may issue a generic framework
text-copy effect inside the KUC adapter, but it does not inspect native image
or file clipboard payloads. KLE provides an opaque surface identity and typed
host mapping only; it owns no text buffer mutation algorithm, history stack,
clipboard payload, cursor geometry, font fallback, or raw egui state.

KatanA retains document mutation (`UpdateBuffer`), image/file clipboard
acquisition, document/workspace identity, Markdown transforms, filesystem,
and global shortcut arbitration. The KatanA host sees one acknowledged KUC
content transaction or one host-owned image request, never both for one paste.

## Strict Evidence Rule

For every generated external dependency leaf, the gate requires all of:

1. exact KatanA lock/package/source/symbol/invoking-span fingerprint;
2. one public KLE RawInput scenario, with no direct state injection;
3. one current opaque KUC root record, RGBA frame and AccessKit assertion;
4. an ordered KUC-to-KLE opaque forwarding trace with no dropped,
   duplicated, stale, cross-document, or locally completed event; and
5. the named KLE-owned observable or no-mutation outcome, correlated with
   the same root and input receipt.

After public KLE adoption, the separate `downstream-full` gate under KatanA
#336 additionally requires the KatanA transaction trace and named actual
buffer/cursor/history/clipboard/no-mutation effect. The host-effect column
above defines that subsequent verification scope; it does not reverse the
publication order.

An external crate unit test, a KLE event injection, an egui shape count, a
string search, a fallback render, or a screenshot/video alone cannot close any
leaf.
