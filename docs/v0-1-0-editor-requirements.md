# katana-language-editor v0.1.0 Editor Requirements

## 位置付け

この文書は `katana-language-editor` v0.1.0 の要件定義書である。目的は、KatanA が現在持つ editor 体験を KLE へ移管しつつ、egui 依存で壊れやすい UI 契約を KUC の汎用 UI contract へ寄せることにある。

一次情報は以下とする。

- KLE OpenSpec: `openspec/changes/v0-1-0-language-editor-extraction`
- KatanA editor 実装: `/Users/hiroyuki_furuno/works/private/katana/crates/katana-ui/src/views/panels/editor`
- KatanA editor full parity audit: `docs/v0-1-0-katana-editor-full-parity-audit.md`
- KatanA source inventory mapping: `docs/v0-1-0-katana-editor-full-parity-audit.md` の `## Source Inventory`
- KatanA source-universe baseline: `docs/v0-1-0-katana-editor-source-universe.md`
- Spark implementation handoff and batch gates: `docs/v0-1-0-spark-execution-plan.md`
- KUC opaque TextSurface/CommandChrome root design: `docs/v0-1-0-kuc-text-command-root-design.md`
- KatanA editor migration map: `docs/v0-1-0-katana-editor-migration.md`
- KatanA editor 中立契約: `/Users/hiroyuki_furuno/works/private/katana/crates/katana-core/src/editor/mod.rs`
- KDV Storybook 実装: `/Users/hiroyuki_furuno/works/private/katana-document-viewer/tools/kdv-storybook`
- KUC 汎用 UI contract: `/Users/hiroyuki_furuno/works/private/katana-ui-core/crates/katana-ui-core`

## v0.1.0 Done Definition

v0.1.0 は次を満たす。初版 release の対象は KatanA の editor 部分の全機能であり、KLE の release 前 gate が通っただけでは完了としない。

- KatanA の editor が持つ編集、表示、検索、診断、スクロール、選択、ショートカット接続、authoring toolbar/context menu、image ingest entry point、view-mode/split integration、lint-fix review 起動に必要な contract を KLE で定義し、KLE-owned actual `KatanaApp` host E2E で検証する。
- `katana-language-editor` neutral crate は egui、Floem、KUC Storybook runtime などの具象 UI 型を public API に漏らさない。
- v0.1.0 に egui MVP または partial-backend の例外はない。KatanA editor の source-derived feature は、色、文字列、font、spacing、emoji/IME を含めて KUC contract と host injection により完全に実装する。KLE 内の ad hoc fallback は許可しない。
- KUC platform text runtime は system font discovery を generic process-shared
  catalog として所有する。TextSurface、CommandChrome、ContextMenu を同時に
  作っても OS font database の全走査は catalog ごとに一回だけとし、surface
  ごとの text/glyph/texture/selection cache は混在させない。この条件は日本語、
  IME、exact `⭐️` VS16 color glyph の同一性を崩さず、KLE/Storybook の cache
  追加や test coverage 低下で代替してはならない。
- KUC は KatanA/KUC の release matrix である macOS、Windows、Linux の各 profile
  で、実際に解決した color-emoji face とその bytes SHA-256 を catalog evidence
  に記録する。各 profile の `⭐️` (`U+2B50 U+FE0F`) は単独 crop の chromatic RGBA
  pixels と same-config `☆` control との差分で証明する。color face 未解決、
  `SansSerif` substitute、profile skip、KLE/KDV fallback は release failure とする。
- Storybook は KDV と同様に live harness として実装し、`--interactive` 起動時に実際の editor surface を開く。`--motion-artifact` はその同一 surface の出力から生成し、別の fallback renderer、fixture-only canvas、shape count だけを acceptance 根拠にしてはならない。静的 mock や screenshot-only のデモを合格条件にしない。
- KatanA full parity はこの Storybook 視覚起動自体では完了扱いせず、別の release blocker と連動して管理する。
- すべての editor 要件は自動検証に紐付く。KatanA source-derived feature または user-mandated replace capability に対する `EditorError::Unsupported` は release failure であり、検出テストは成功条件ではなく未実装を示す failing gate とする。

## 責務境界

## Public DTO Compatibility

KLE の public DTO は原則 `#[non_exhaustive]` を付ける。host が直接 struct literal に依存しないよう、構築が必要な型には constructor または input struct を提供する。将来フィールドを増やす場合は、constructor の互換性を保つか、意図的な SemVer break として OpenSpec と release notes に明記する。

### KLE neutral crate が持つ

- host-projected opaque identity/revision/capability/presentation descriptor と、closed typed host request の互換 DTO。
- KUC opaque full-editor root を KatanA host descriptor と結ぶ `LanguageEditor` facade。KLE は root child、widget、render state を持たない。
- source-derived request origin/correlation/effect evidence の join contract。
- single-consumption input transport の機械的 forwarding contract。raw document/search/replacement/source/group text、IME origin、clipboard payload、range、line、anchor、pixel、match、undo state を KLE に保持しない。
- generic UI 意味を再計算しない opaque diagnostics/decoration/accessibility descriptor forwarding。文字/grapheme/range conversion、search match、undo/batch edit、clipboard backend、theme/font/string/layout は KUC 又は KatanA host の所有である。

### `katana-language-editor-egui` が持つ

- KUC-owned `TextSurface` / `CommandChrome` shared egui adapter を呼ぶ thin binding。KLE は generic egui component を描画しない。
- KLE neutral contract を KUC typed props/event と host callback へ機械的に bridge する。KLE が value、query、selection、line、scroll、overlay、history、tooltip state を再構築してはならない。
- KUC が供給する platform text-raster / TextSurface / SVG raster runtime の consumer。local texture cache、font lookup、hit-test、gutter、generic overlay を持たない。
- KatanA が既に提供する機能を `Unsupported` に置き換えない。backend limitation は requirement row、KUC implementation task、runnable blocker を同時に持つ場合だけ明示する。

### KUC 側へ寄せる

- emoji / IME / grapheme caret / text area interaction、text/selection/undo visual state、search/replace input、TabStrip、BreadcrumbNavigator、SourceAddressBar、diagnostics/status、viewport/split/preview などの汎用 retained UI contract。
- text span、font、theme token、spacing、motion、icon、host action / hit target / coordinate normalization、range/row/scroll hit fact。
- OS font selection、emoji color glyph raster、grapheme-aware text measurement、caret/hit-test 座標を持つ platform text-raster runtime。その runtime は KLE/KDV の双方が同じ generic API で使う。
- existing `TextArea`、text selection、text span、ScrollArea、ContextMenu、DiagnosticsList を compose する `TextSurface`、generic gutter/annotation/frame record、shared egui/AccessKit adapter。KLE/KDV は実 renderer を持たず、この adapter を利用する。
- Storybook の live component / visual contract / interaction contract の汎用基盤。
- KatanA 固有でない UI 部品や描画 contract。KUC へ追加する場合も Katana 名前空間や Katana 固有 enum を入れない。

### KatanA host 側に残す

- workspace、document path、virtual path、reference document、本文、cursor/undo authoritative state、保存、file IO。
- Markdown formatter、KMM / markdown linter、search match/range、replace byte-range conversion、image ingest、clipboard image file 生成。
- KatanA の command inventory、global shortcut context、preview 同期 coordinator、scroll/anchor/source mapping、tab/group persistence。
- Markdown 固有 highlighter / source anchor adapter / diagnostics source の実装、URL/path/history/fetch/browser/document state。

これらは実行責務を KatanA host に残すという意味であり、初版 parity の対象外にするという意味ではない。KLE v0.1.0 は、KatanA host がこれらを既存 editor と同等に接続できる action/state/event contract と downstream integration evidence を持たなければならない。

### Neutral relay terminology

本書でいう KLE の「typed request」「mapping」「DTO」は、opaque host target、
descriptor revision、correlation、source だけを持つ closed neutral envelope を指す。
KLE が `AppAction`、document identity、path、URL、Markdown operation、range、line、
scroll offset、diagnostic payload、search match、TOC semantic を復号・生成・保持する
意味ではない。KLE は envelope を一度だけ relay し、現在の host projection を照合して
既存 KatanA action へ解決する責務は unchanged host のみが持つ。この定義は本書の旧い
typed-mapping 表現より優先する。

## Editor Parity Requirements

### Authoring inventory clarification (2026-08-13)

The KatanA editor has three distinct authoring surfaces. They must not be
collapsed into a single Storybook counter or a fixed KLE button catalogue.
Their concrete source-of-truth is the read-only KatanA checkout:

| Surface | KatanA source | Required v0.1.0 capability | Required automated evidence |
| --- | --- | --- | --- |
| Floating editor toolbar | `views/panels/editor/toolbar.rs`, `toolbar_popup.rs` | 4 inline actions, heading 1--3, bullet/numbered/quote, code-block trigger and image-file entry point; selection enablement, selection-caret anchor, outside/Escape close, focus return and viewport clamp | Real KUC FloatingCommandToolbar RawInput/AccessKit/frame/artifact test through KLE; no KLE coordinate or button renderer |
| Markdown command inventory and context menu | `state/command_inventory/edit_commands.rs`, `views/panels/editor/context_menu.rs`, `markdown_authoring_op.rs` | KatanA root order is Save, conditional Format, Edit submenu, and Ingest submenu. Edit contains the 14 Markdown operations: bold, italic, strikethrough, inline code, heading 1--3, bullet/numbered/quote, nested code block, horizontal rule, link and table. Ingest contains file and clipboard-image leaves. | Data-driven opaque injected command presentation; every operation emits one current opaque authoring target/revision/correlation event and is resolved only by the read-only KatanA host to `AppAction::AuthorMarkdown`. KLE keeps neither `RunAuthoringCommand`, a command string, nor a duplicated Markdown enum. A KUC frame record must never expose an item outside its interactable bounds; overflow requires KUC-owned scrolling and refreshed public item records before a physical selection. |
| Code and image variants | `code_block_menu.rs`, `context_menu_image_ingest.rs`, `app/action/image_ingest.rs` | all 17 `CodeBlockKind` values through the KUC dropdown; file image ingest and KatanA's clipboard image/file URL host entry points | Pointer/keyboard dropdown and menu tests, disabled/no-event cases, one host action event per selection, KLE-owned actual `KatanaApp` file/clipboard host-E2E evidence |

The floating toolbar uses the KUC CommandChrome visible command subset; the
context menu and KatanA command inventory are separate host command surfaces.
KLE forwards an opaque host-issued target once but neither invents
labels/icons/groups nor implements Markdown text transformation. Host presentation is required and
data-driven; unknown injected IDs must remain visible and accessible when their
corresponding neutral command is injected.

#### Context-menu enablement and clipboard priority (2026-08-13)

KatanA host presentation MUST expose `FormatDocument` only for an editable
`.md` or `.markdown` document. Bold, italic, strikethrough, and inline-code
require a non-empty selection; heading, block, and insertion operations do
not. Clipboard-image is disabled when the host has no eligible payload.

When a paste event has an image payload, the KatanA host consumes it for image
ingest before ordinary text insertion. A supported `file://` image URL follows
the same ingest route. Payload absence or acquisition failure MUST leave the
buffer unchanged; KLE must not turn those cases into `UpdateBuffer`. KUC owns
the generic input and disabled-control behavior; KLE only forwards the current
opaque host projection and one-time intent transit; KatanA owns payload inspection,
file IO, and buffer mutation. The visible context menu has no file-URL item:
file URLs are a host clipboard acquisition outcome, never a KLE menu leaf.

Verification requires separate actual-input cases for every enablement branch,
image-payload precedence, `file://` ingestion, payload-none/error no-mutation,
and read-only rejection. A visible context-menu item is not proof of these
host transitions.

#### Requirements-completeness and evidence model (2026-08-14)

The 26-row capability matrix is only an index. It MUST NOT be used as a
substitute for a source-derived leaf inventory. KatanA is a read-only reference
for this release. Before any row can be closed, the parity gate must prove, for
every KatanA editor source branch, all of the following: the exact source path
and decisive symbol, the visible root order and submenu path where applicable,
state-dependent enablement, the neutral relay or shortcut execution mode, its
KUC/KLE/KatanA owner, and three separately runnable proofs: (1) that KatanA
source marker, (2) an input-origin record, and (3) a declared effect proof.
`editor_direct_ui`, `editor_host_continuation`, and user-mandated extension
leaves require KLE public `RawInput` -> `EguiLanguageEditor::show`. A KatanA-
owned `editor_shortcut` instead requires the physical KatanA RawInput/router
trace plus a same-run KUC root/AccessKit focus and retained-input record for
the same document and revision; it has no KLE mounted-causality claim.
`kuc_retained_ui_effect` proves a same-root KUC/AccessKit state
transition plus a bootstrapped read-only KatanA no-mutation observation;
`in_process_host_effect` and `native_external_host_effect` drive the typed
request through a real KatanA UI frame and assert the concrete document, state,
or file effect. Neither a KatanA source edit nor an in-repository KatanA adapter is required
or permitted as a substitute for proof (3). Context menu evidence is
specifically incomplete unless it distinguishes
`Save -> conditional Format -> Edit -> Ingest`, all fourteen `Edit` leaves,
the seventeen nested code kinds, both visible ingest leaves, and secondary
click, keyboard and AccessKit selection after KUC-owned scrolling.

The source inventory MUST include the decisive clipboard acquisition paths
(`clipboard_image.rs`, `clipboard_file_url.rs`, platform-specific clipboard
handling where compiled), shortcut arbitration (`edit_commands.rs`,
`shortcut_context.rs`, `shell_ui_shortcuts.rs`), and dirty/format refresh
(`process_markdown_formatting.rs`, `refresh_content.rs`). A source-only match,
a Storybook control, a KLE simulator, or a screenshot is not runnable parity
evidence. Missing source coverage remains a release blocker, even when a
top-level feature count looks complete.

KatanA supplies find/navigation/highlight behavior. Visible replace and
replace-all are retained as a **user-mandated v0.1.0 addition**, not presented
as KatanA behavior. They require the same KUC component, actual-input,
document-effect, accessibility, artifact, and KLE-owned actual `KatanaApp`
host-E2E evidence as a
KatanA-derived requirement.

#### Leaf manifest schema and audited branches (2026-08-14)

The parity manifest MUST represent one source-derived state branch per row. A
row is invalid unless it carries all of: exact KatanA `path:symbol` marker,
visible/menu path, state condition, declared effect class, typed KLE request or
retained-UI state, KUC/KLE/KatanA owner, and the expected class-appropriate
effect. `editor_direct_ui`, `editor_host_continuation`, and user-mandated
extension rows additionally require public KLE `RawInput ->
EguiLanguageEditor::show` evidence. A KatanA-owned `editor_shortcut` row instead
requires physical KatanA shortcut-router RawInput plus a same-run correlated
KUC root/AccessKit focus or retained-input record; it has no typed KLE request
and must not claim mounted KLE integration. The release gate must reject a row
missing its origin-appropriate field; a top-level feature, a shared selector,
or an aggregate test result does not supply omitted leaf fields. `find` is KatanA parity; visible
`replace-current` and `replace-all` are user-mandated additions and must stay
separate.

#### External host effect execution (2026-08-14)

The existing KLE host E2E is a partial bridge, not a completion proof: its
current `UnsupportedAction` and `ExternalHostIntentUnavailable` branches, and
the headless image-picker request that never supplies a selected file, leave
required leaves unexecuted. No required leaf may accept either result. The
binding and release protocol is specified in
`docs/v0-1-0-external-host-e2e-design.md`.

An `in_process_host_effect` leaf with a KUC/KLE origin must start with the same
public KLE RawInput case that generated its opaque one-time transit, then execute real
frames on unchanged `KatanaApp` until its exact document/state/file effect is
observed. A KatanA-owned `editor_shortcut` instead starts from physical key
RawInput on KatanA's router and joins its result to the same-run KUC focus/input
record without a typed KLE request. A
`native_external_host_effect` leaf additionally drives KatanA's actual native
dialog or macOS pasteboard from isolated test infrastructure and asserts the
asset/link/dirty/explorer or no-mutation outcome in that same run. KLE and KUC
must never receive an injected image payload, inspect a clipboard, parse a
`file://` URL, or write the asset. A KatanA source test remains an oracle only;
it cannot join a separate request test to close the end-to-end leaf.

The following audit is a source-grounded required minimum. `C/M` means the
KatanA branch is confirmed and its leaf manifest/three proofs are missing.
`I/M` means the behavior also depends on egui or OS input semantics, so source
inspection alone cannot close its contract. All rows are release blockers until
their complete manifest row and executable three proofs exist.

`text.cursor-navigation-family`の固定ソース分岐と未検証範囲は
[カーソル移動の固定ソース参照](v0-1-0-cursor-navigation-reference.md)に記録する。
限定LSP診断やegui単体の入力参照を、KUC比較・三OS実入力・全leafの合格へ昇格しない。

| ID | Status | KatanA decisive branch | Visible path and state condition | Typed state/request and owner | Required actual proof |
| --- | --- | --- | --- | --- | --- |
| `text.editable` | C/M | `crates/katana-ui/src/views/panels/editor/text_edit.rs:TextEdit::multiline` | editor, non-reference document | `ContentChanged`; KUC input, KLE bridge, KatanA buffer | RawInput edit changes real buffer |
| `text.ime-preedit` | I/M | locked `egui::TextEdit` semantic dependency | Japanese, exact `⭐️` VS16, ZWJ preedit and active range | typed IME preedit/range; KUC/KLE/KatanA | root pixels/AX record preedit without duplicate host mutation |
| `text.ime-commit` | I/M | `text_edit.rs:TextEdit::multiline`, locked `egui::TextEdit` semantic dependency | Japanese, exact `⭐️` VS16, ZWJ commit | typed IME commit/range; KUC/KLE/KatanA | one committed mutation reaches the actual KatanA buffer unchanged |
| `text.ime-empty-or-newline-reject` | I/M | locked `egui::TextEdit` semantic dependency | empty/no-active or CR/LF preedit/commit | typed IME no-mutation result; KUC/KLE/KatanA | composition state and buffer remain correct without a host update |
| `text.ime-delete-surrounding` | I/M | locked `egui::TextEdit` semantic dependency | active composition with before/after character counts | typed IME delete-surrounding; KUC/KLE/KatanA | exact Unicode-safe buffer/caret mutation reaches KatanA once |
| `text.enter-newline` | I/M | locked `egui::TextEdit` semantic dependency | focused editable multiline editor | `ContentMutation(enter)`; KUC/KLE/KatanA | selection replacement, newline and caret reach KatanA once |
| `text.tab-not-editor-indent` | I/M | KatanA `text_edit.rs` uses plain `TextEdit::multiline`; locked `egui 0.36.1` default `EventFilter { tab: false }` | focused editor, Tab | generic KUC focus-routing outcome; no editor content event | the editor itself neither inserts `\\t` nor changes the buffer/history/caret; generated closure proves the surrounding focus route rather than assuming an indent feature |
| `text.shift-tab-not-editor-unindent` | I/M | `crates/katana-ui/src/views/panels/editor/text_edit.rs`; locked `egui::TextEdit` default `EventFilter { tab: false }` and `EventFilter::matches` | focused editor, Shift+Tab | generic KUC reverse focus-routing outcome; no editor content event | the editor itself neither removes indentation nor changes buffer/history/caret; generated closure proves the surrounding focus route |
| `text.delete-branch-family` | generated I/M leaf family | locked `egui::TextEdit` semantic dependency | every generated Backspace/Delete/Ctrl-H/Ctrl-K/Ctrl-U/Ctrl-W and modifier branch | generic KUC text operation; KLE/KatanA | each fixed-source branch has a unique leaf, root record and host buffer/caret proof; no aggregate pass |
| `text.cursor-navigation-family` | generated I/M leaf family | locked `egui::TextEdit` transitive `cursor_range.on_event` closure | every fixed-source cursor/selection navigation branch | generic KUC selection state; KUC/KLE/KatanA | each branch has a unique root/AX/host cursor proof; no static family count |
| `text.cursor-restore` | C/M | `text_edit.rs`, `app/action/process_authoring.rs` | after authoring, selection and no-selection | `PendingCursorRestore`; KLE/KatanA | both actual cursor outcomes |
| `history.document-isolation` | C/M | `crates/katana-ui/src/editor_undo.rs` | external format/fix, two documents or workspaces | opaque KUC surface identity plus `HostExternalChange { document_id }`; KUC/KLE/KatanA | retained history and cursor never cross document/workspace identity |
| `document.dirty-refresh` | C/M | `crates/katana-ui/src/app/document.rs` | normal input | content report; KLE/KatanA | dirty, preview, search, diagnostics, timestamp |
| `toolbar.open` | C/M | `crates/katana-ui/src/views/panels/editor/toolbar_popup.rs` | caret/selection, editable, editor focused | floating toolbar open; KUC/KLE | RawInput plus artifact anchor |
| `toolbar.focus-retain` | C/M | `toolbar_popup.rs` | focus moves to toolbar | retained toolbar state; KUC/KLE | focus transfer and AccessKit |
| `toolbar.diagnostic-suppress` | C/M | `text_edit.rs`, `toolbar_popup.rs` | diagnostic hover | suppressed toolbar state; KUC/KLE | hover hides toolbar |
| `toolbar.outside-close` | C/M | `toolbar_popup.rs` | pointer leaves editor/toolbar | close request; KUC/KLE | outside-pointer frame closes it |
| `toolbar.viewport-clamp` | C/M | `toolbar_popup.rs` | caret near each viewport edge | clamped toolbar anchor; KUC/KLE | four-edge bounds contract |
| `context.root-order` | C/M | `crates/katana-ui/src/views/panels/editor/context_menu.rs` | secondary/keyboard/AccessKit open | context presentation; KUC/KLE/KatanA | exact `Save -> Format? -> Edit -> Ingest` path |
| `context.format-enable` | C/M | `context_menu.rs` | editable `.md`/`.markdown` only | `FormatDocument`; KLE/KatanA | extension and read-only matrix |
| `context.inline-enable` | C/M | `context_menu.rs` | non-empty selection for bold/italic/strike/inline code | KUC opaque authoring-target event; KLE/KatanA | disabled state and no emitted host request |
| `context.reference-reject` | C/M | `context_menu.rs`, `app/action/process_authoring.rs` | heading/block visible on reference document | host rejection; KLE/KatanA | visibility distinct from buffer immutability |
| `context.clipboard-image-enable` | C/M | `context_menu_image_ingest.rs` | eligible clipboard image absent/present | ingest availability; KatanA/KLE/KUC | disabled no-payload action cannot mutate |
| `paste.image-priority` | C/M | `crates/katana-ui/src/views/panels/editor/paste.rs` | focused editor receives image plus text | clipboard resolution image; KUC/KLE/KatanA | text is consumed, exactly one image intent |
| `paste.file-url-kind` | C/M | `paste.rs` | image `file://` versus non-image URL | typed clipboard resolution; KLE/KatanA | separate ingest and text-buffer cases |
| `paste.raw-image-selection-preservation` | C/M | `text_edit.rs`, `paste.rs` | focused editable editor with a non-empty selection and raw image payload | opaque host image intent with original selection snapshot; KUC/KLE/KatanA | no `UpdateBuffer`; image Markdown replaces/inserts at the original selection exactly once |
| `paste.file-url-selection-preservation` | C/M | `ui.rs`, `text_edit.rs`, `paste.rs`, `process_authoring.rs` | focused editable editor with a non-empty selection and supported image `file://` text | opaque host image intent with restored original selection; KUC/KLE/KatanA | no URL text commits; image Markdown replaces/inserts at the original selection exactly once |
| `clipboard.file-list` | C/M | `crates/katana-ui/src/app/action/clipboard_image.rs` | first supported file-list image | host acquisition result; KatanA | asset/link/dirty effect |
| `clipboard.file-url-decode` | C/M | `crates/katana-ui/src/app/action/clipboard_file_url.rs` | percent-encoded image URL, unsupported/read-failure | host acquisition result; KatanA | decoded, rejected, and failed cases |
| `clipboard.macos-native` | C/M | `crates/katana-ui/src/app/action/clipboard_image_macos.rs` | macOS PNG/JPEG/TIFF fallback | platform acquisition result; KatanA | deterministic boundary test; live OS only supplemental |
| `ingest.saved-document` | C/M | `crates/katana-ui/src/app/action/image_ingest.rs` | saved Markdown document | `IngestClipboardImage`; KatanA | asset file, Markdown link, dirty, explorer refresh |
| `ingest.unsaved-reject` | C/M | `image_ingest.rs` | unsaved document | typed failed result; KatanA | file and buffer remain unchanged |
| `search.query-filter` | C/M | `crates/katana-ui/src/app/doc_search.rs` | visible text/code; URL and HTML attributes excluded | query changed; KLE/KatanA | multibyte filtered ranges |
| `search.next-wrap` | C/M | `app/doc_search.rs` | Enter/Down next | next request; KLE/KatanA | active index and real scroll target |
| `search.previous-wrap` | C/M | `app/doc_search.rs` | Shift+Enter/Up previous | previous request; KLE/KatanA | active index and real scroll target |
| `search.zero-result` | C/M | `crates/katana-ui/src/views/top_bar/search.rs` | no results | search availability; KUC/KLE | disabled controls in RawInput and AccessKit |
| `search.escape-close` | C/M | `crates/katana-ui/src/views/top_bar/search_logic.rs` | focused search receives Escape | close request; KUC/KLE | focus and close lifecycle |
| `workspace-search.modal-tabs` | C/M | `crates/katana-ui/src/views/modals/search.rs:SearchModal::show` | workspace search open; File Name and Markdown Content tab selection | generic KUC search-modal retained state; KLE forwards only opaque host descriptors | pointer, keyboard, and AccessKit tab changes retain focus policy and no host mutation |
| `workspace-search.filename-filter-family` | generated C/M leaf family | `crates/katana-ui/src/views/modals/search_tabs/filename_tab.rs:FilenameTabOps::show_filename_tab` | file-name query, include/exclude patterns, match-case, match-word, regex, empty, invalid regex, and no workspace | generic KUC query/filter controls; KLE holds no query, regex, path, or result list; KatanA owns workspace traversal | each input branch has a KUC root/AccessKit record and KatanA-owned result/no-result outcome without KLE filesystem or regex logic |
| `workspace-search.markdown-history-family` | generated C/M leaf family | `crates/katana-ui/src/views/modals/search_tabs/md_tab.rs:MdTabOps::show_md_tab` | Markdown-content query; query edit, Enter/lost-focus refresh, history select/remove/clear, empty history, no workspace, and no result | generic KUC modal/search/history UI; KLE forwards only a nonpersistent opaque request; KatanA owns search/history data and traversal | each branch has physical RawInput and root/AccessKit evidence; query/history text is absent from KLE records and manifests |
| `workspace-search.filename-result-select` | generated C/M leaf family | `crates/katana-ui/src/views/modals/search_tabs/filename_tab.rs:104-110` | each current file result exists/missing | opaque KUC result target; KLE one-time transit; KatanA `SelectDocument` | actual result selection reaches KatanA document selection, while missing targets cause no mutation |
| `workspace-search.markdown-result-jump` | generated C/M leaf family | `crates/katana-ui/src/views/modals/search_tabs/md_tab.rs:91-116` | each current Markdown result | opaque KUC result target; KLE one-time transit; KatanA `SelectDocumentAndJump` | actual result selection reaches the KatanA document and source-location effect; KLE has no path, line, range, or coordinate calculation |
| `diagnostic.start-marker` | C/M | `crates/katana-ui/src/views/panels/editor/row_diagnostics.rs` | official diagnostic start line only | gutter marker request; KUC/KLE/KatanA | official/non-official and start/non-start |
| `diagnostic.aggregate-popup` | C/M | `crates/katana-ui/src/views/panels/editor/diagnostics_hover.rs`, `diagnostics_popup.rs` | same-line diagnostics, hover/click/outside/action | diagnostic action; KLE/KatanA | aggregated popup lifecycle and effects |
| `shortcut.priority` | C/M | `crates/katana-ui/src/state/shortcut_context.rs` | Recording > Modal > Editor > Preview > Global | shortcut dispatch; KatanA host | dispatch/rejection for every active context |
| `gutter.number-active-hover-click` | C/M | `line_numbers.rs`, `decorations.rs`, `ui.rs` | every logical row; active cursor row; pointer hover/click | KUC row fact/opaque target; KLE one-time transit; KatanA scroll | numbers, active/hover state, click effect, AccessKit semantics |
| `gutter.marker-priority` | C/M | `row_diagnostics.rs`, `diagnostics_ui.rs` | multiple severities and same row | KUC marker descriptor; KLE has no diagnostic interpretation | Error > Warning > Info resolution, hit target, no local geometry |
| `diagnostic.fix-and-docs` | C/M | `diagnostics_hover.rs`, `app/action/{dispatch,dispatch_tertiary,process_linter}.rs`, `document_edit.rs`, `diff_review.rs` | fix one, fix all, open documentation | opaque diagnostic target transit; KatanA review/browser effect | each action's real buffer/review/browser effect or explicit host boundary |
| `search.open-focus` | C/M | `views/app_frame/tab_toolbar.rs`, `views/top_bar/{search,search_logic}.rs` | search opened, empty/non-empty query | KUC retained search focus; KLE opaque current-root transit only | first focus, count, visible route, AccessKit |
| `replace.current` | user-mandated extension | no fixed-revision KatanA editor-side control; `crates/katana-ui/src/app_action_types.rs`, `crates/katana-ui/src/app/action/dispatch.rs`, `crates/katana-ui/src/app/document_edit.rs:DocumentEditOps::handle_replace_text` define only the existing `ReplaceText` host route | generic KUC search strip with active match | single-consumption replacement/current-result transport; host derives range; KUC/KLE/KatanA | exactly active-match mutation, refreshed range/count/scroll |
| `replace.all` | user-mandated extension | no fixed-revision KatanA editor-side control; `crates/katana-ui/src/app_action_types.rs`, `crates/katana-ui/src/app/action/dispatch.rs`, `crates/katana-ui/src/app/document_edit.rs:DocumentEditOps::handle_replace_text` define only the existing `ReplaceText` host route | generic KUC search strip with one or more matches | single-consumption replacement/all intent; host iterates current matches and derives each range; KUC/KLE/KatanA | all-match mutation, refreshed range/count/scroll, no-match no-op |
| `clipboard.text-copy-selection` | I/M | `text_edit.rs`, locked `egui::TextEdit` semantic dependency | non-empty selection | opaque KUC platform text-clipboard write; KUC/KLE/KatanA | exact selected UTF-8 reaches the platform output once; no content mutation |
| `clipboard.text-copy-empty-selection` | I/M | `text_edit.rs`, locked `egui::TextEdit` semantic dependency | empty selection | opaque KUC text-clipboard outcome; KUC/KLE/KatanA | no clipboard write and no content mutation |
| `clipboard.text-cut-selection` | I/M | `text_edit.rs`, locked `egui::TextEdit` semantic dependency | non-empty selection, editable document | opaque clipboard write plus `ContentMutation(cut)`; KUC/KLE/KatanA | one write and one `UpdateBuffer` host mutation with selection/caret result |
| `clipboard.text-cut-readonly` | I/M | `text_edit.rs`, locked `egui::TextEdit` semantic dependency | non-empty selection, reference document | read-only KUC outcome; KUC/KLE/KatanA | no buffer/history mutation and no forbidden host action |
| `clipboard.text-paste` | I/M | `text_edit.rs`, `paste.rs`, locked `egui::Event::Paste` semantic dependency | focused editable editor, host resolves ordinary UTF-8 text | correlated KUC paste intent/resolution and `ContentMutation(paste_text)`; KUC/KLE/KatanA | one content mutation and one `UpdateBuffer`; Japanese/VS16/ZWJ are preserved |
| `clipboard.text-paste-readonly` | I/M | `text_edit.rs`, locked `egui::Event::Paste` semantic dependency | focused reference editor | read-only paste resolution; KUC/KLE/KatanA | no text mutation, image action, history change, or host buffer update |
| `clipboard.text-paste-empty-or-error` | I/M | `text_edit.rs`, `paste.rs` | focused editor, empty/failed/stale host text resolution | correlated typed outcome; KUC/KLE/KatanA | no text mutation and no duplicate/replayed resolution |
| `history.undo` | C/M | `editor_undo.rs`, locked `egui::TextEdit` semantic dependency | editable document with user or host-origin change | `ContentMutation(undo)`; KUC/KLE/KatanA | exact prior buffer/caret, one host update, per-document identity |
| `history.redo` | C/M | `editor_undo.rs`, locked `egui::TextEdit` semantic dependency | editable document after undo | `ContentMutation(redo)`; KUC/KLE/KatanA | exact next buffer/caret, one host update, per-document identity |
| `history.host-external-sync` | C/M | `editor_undo.rs`, `document_edit.rs`, `process_markdown_formatting.rs` | authoring, replace, format, lint-fix, or refresh acknowledgement | host snapshot/revision acknowledgement; KUC/KLE/KatanA | exactly one retained history transition without user-event replay or duplicate host mutation |
| `shortcut.text-entry-reservation` | C/M | `shell_ui/shortcut_keys.rs`, `shell_ui_shortcuts.rs` | Editor context with navigation/editing/primary A/B/C/I/K/U/V/X/Y/Z | KatanA host arbitration; KUC consumes retained editor input | reserved input reaches one editor path with no command-inventory double dispatch |
| `shortcut.image-paste-exception` | C/M | `shell_ui/shortcut_keys.rs`, `shell_ui_shortcuts.rs` | Editor context primary Shift+V without Alt | KatanA `IngestClipboardImage` route | host consumes the exception once; KUC does not treat it as ordinary text paste |
| `shortcut.command-inventory-family` | generated C/M leaf family | `state/command_inventory/{file,edit,view}_commands.rs`, `shell_ui_shortcuts.rs`, action closure | every fixed-revision command item, availability branch, declared shortcut and winning active context | KatanA shortcut router; KUC focus/input facts; KLE opaque target forwarding only | generator creates one leaf per physical shortcut origin, including save/close/restore/document-search/refresh/image-ingest and every authoring shortcut. Shell/workspace commands require source-spanned `host_only` rationale; a command count or shared keyboard trace is rejected. |
| `syntax.span-decoration` | static non-invocation candidate | `syntax.rs`, `layout.rs`, `types.rs`, `text_edit.rs` | fixed-source search finds no production `EditorLayouter`/`MarkdownSyntaxHighlighter` invocation; `TextEdit::multiline` has no custom layouter and the only `syntax_highlighter` use is a test asserting empty default spans | source-closure generator must prove `unreachable_helper` or resolve a macro/trait/external call | no syntax-rendered parity claim or KLE/KUC local highlighter. A generated resolved invocation reopens its exact branch; a source definition or test alone is never a leaf. |
| `scroll.editor-preview-sync` | C/M | `logic.rs`, `logic_scroll.rs`, `state/{scroll.rs,scroll_sync/mod.rs}`, `views/panels/preview/logic.rs` | editor vs preview origin, heading/EOF | KUC editor scroll fact; KLE opaque transit; KatanA coordination | origin suppression, map/heading behavior, deterministic convergence |
| `scroll.search-and-jump` | C/M | `app/action/process_helpers.rs`, `app/action/process_document.rs`, `views/layout/split.rs`, `state/scroll.rs` | next/previous result; `SelectDocumentAndJump`; `PreviewOnly` fallback | KUC logical-row scroll; KLE opaque transit; KatanA view state | one-shot target consumption and CodeOnly fallback only from PreviewOnly |
| `view.command-toggle-split` | C/M | `state/command_inventory/view_commands.rs`, `app/action/dispatch_secondary.rs` | global `primary+Shift+\\` from every documented view mode | opaque host target transit; KatanA state | one command route changes the active document to `Split`; no editor text shortcut double dispatch |
| `view.command-toggle-code-preview.from-split` | C/M | `state/command_inventory/view_commands.rs`, `app/action/dispatch_secondary.rs` | global `primary+\\`, active mode `Split` | typed `ToggleCodePreview`; KatanA state | exact `Split -> PreviewOnly` transition and persisted active-document mode |
| `view.command-toggle-code-preview.from-preview` | C/M | `state/command_inventory/view_commands.rs`, `app/action/dispatch_secondary.rs` | global `primary+\\`, active mode `PreviewOnly` | typed `ToggleCodePreview`; KatanA state | exact `PreviewOnly -> CodeOnly` transition and persisted active-document mode |
| `view.command-toggle-code-preview.from-code` | C/M | `state/command_inventory/view_commands.rs`, `app/action/dispatch_secondary.rs` | global `primary+\\`, active mode `CodeOnly` | typed `ToggleCodePreview`; KatanA state | exact `CodeOnly -> PreviewOnly` transition and persisted active-document mode |
| `view.set-mode.split` | C/M | `views/panels/preview/{side_panel_tools_inner,tangochou}.rs`, `app/action/dispatch_secondary.rs` | visible split toggle / code-preview switch requests split | typed `SetViewMode(Split)`; KatanA state | exact pointer, keyboard and AccessKit route changes only the active document mode |
| `view.set-mode.preview-only` | C/M | `views/panels/preview/{side_panel_tools_inner,tangochou}.rs`, `app/action/dispatch_secondary.rs` | split toggle off or code-preview switch | typed `SetViewMode(PreviewOnly)`; KatanA state | exact pointer, keyboard and AccessKit route changes only the active document mode |
| `view.set-mode.code-only` | C/M | `views/panels/preview/tangochou.rs`, `app/action/dispatch_secondary.rs` | code-preview switch from preview | typed `SetViewMode(CodeOnly)`; KatanA state | exact pointer, keyboard and AccessKit route changes only the active document mode |
| `view.split-direction.horizontal-to-vertical` | C/M | `views/panels/preview/side_panel_tools_inner.rs`, `app/action/dispatch_secondary.rs` | split tools, current direction `Horizontal` | typed `SetSplitDirection(Vertical)`; KatanA state | active-document `Horizontal -> Vertical`; KUC has no KatanA layout state |
| `view.split-direction.vertical-to-horizontal` | C/M | `views/panels/preview/side_panel_tools_inner.rs`, `app/action/dispatch_secondary.rs` | split tools, current direction `Vertical` | typed `SetSplitDirection(Horizontal)`; KatanA state | active-document `Vertical -> Horizontal`; KUC has no KatanA layout state |
| `view.pane-order.editor-to-preview-first` | C/M | `views/panels/preview/side_panel_tools_inner.rs`, `app/action/dispatch_secondary.rs` | split tools, current order `EditorFirst` | typed `SetPaneOrder(PreviewFirst)`; KatanA state | exact order transition and KUC WorkspaceViewport reflects the returned descriptor |
| `view.pane-order.preview-to-editor-first` | C/M | `views/panels/preview/side_panel_tools_inner.rs`, `app/action/dispatch_secondary.rs` | split tools, current order `PreviewFirst` | typed `SetPaneOrder(EditorFirst)`; KatanA state | exact order transition and KUC WorkspaceViewport reflects the returned descriptor |
| `view.scroll-sync.enable` | C/M | `views/panels/preview/side_panel_tools_inner.rs`, `app/action/dispatch_secondary.rs` | split tools, resolved setting false | typed `ToggleScrollSync(true)`; KatanA state | one `sync_override = Some(true)` host transition and no KUC source-position calculation |
| `view.scroll-sync.disable` | C/M | `views/panels/preview/side_panel_tools_inner.rs`, `app/action/dispatch_secondary.rs` | split tools, resolved setting true | typed `ToggleScrollSync(false)`; KatanA state | one `sync_override = Some(false)` host transition and no KUC source-position calculation |
| `view.layout-persistence` | C/M | `tests/integration/editor/layout_persistence.rs`, state/layout sources reached by closure | document switch/reopen | KatanA layout state; KLE host E2E | document-scoped persisted mode/direction/order |
| `preview.diagram-cache-refresh.theme-idle` | C/M | `shell_ui/shell_ui_update.rs`, `app/action/refresh_content.rs` | theme update and no pending KatanA action | existing host `RefreshDiagrams` continuation | image/viewer caches reset and active preview refreshes once; no KLE control, cache, image, or renderer. |
| `preview.diagram-cache-refresh.theme-pending` | C/M | `crates/katana-ui/src/shell_ui/shell_ui_update.rs`, `crates/katana-ui/src/app/action/refresh_content.rs` | theme update and a non-`None` pending action | existing KatanA pending-action preservation | no `RefreshDiagrams` overwrite, cache reset, or second action; source action remains intact. |
| `preview.diagram-cache-refresh.no-active-document` | C/M | `app/action/refresh_content.rs` | `RefreshDiagrams` with no active document | existing cache-reset host continuation | cache reset occurs; no preview refresh/document mutation follows. |
| `preview.diagram-cache-refresh.virtual-manual` | C/M | `app/action/refresh_content.rs` | active `Katana://` document and manual refresh | `RefreshDocument { is_manual: true }` -> `RefreshDiagrams` host continuation | one cache-reset and preview retry; automatic refresh leaves the virtual path unchanged. |
| `preview.task-toggle-valid` | C/M | `views/panels/preview/{content,content_html_browser}.rs`, `app/action/{dispatch,mod}.rs` | one actionable projected task with a current host semantic target | opaque host task request; KatanA markdown transform | exactly one host-owned task mutation, dirty/preview refresh; KLE/KUC never parse or calculate KatanA global task indices |
| `preview.task-toggle-stale-or-missing` | C/M | `crates/katana-ui/src/views/panels/preview/content.rs`, `crates/katana-ui/src/views/panels/preview/content_html_browser.rs`, `crates/katana-ui/src/app/action/dispatch.rs`, `crates/katana-ui/src/app_action_types.rs` | stale, absent or non-actionable projected task | opaque rejected host task outcome | no buffer/dirty/history/preview duplicate mutation |
| `preview.back-to-top` | C/M | `views/panels/preview/{logic,content}.rs` | visible back-to-top action, current preview offset nonzero | KUC retained `PreviewViewport` scroll state | same-root scroll-to-origin/AccessKit transition and unchanged bootstrapped KatanA host observation |
| `preview.slideshow.open.windowed` | C/M | `state/command_inventory/view_commands.rs`, `app/action/{dispatch,process_helpers}.rs` | `F5` or `primary+alt+Enter`, OS window initially not fullscreen | typed slideshow request; native external host effect | reset page, record prior fullscreen false, exactly one fullscreen-on native observation |
| `preview.slideshow.open-already-fullscreen` | C/M | `crates/katana-ui/src/state/command_inventory/view_commands.rs`, `crates/katana-ui/src/app/action/dispatch.rs`, `crates/katana-ui/src/app/action/process_helpers.rs`, `crates/katana-ui/src/preview_pane/slideshow/modal.rs` | slideshow request, OS window already fullscreen | typed slideshow request; native external host effect | reset page, preserve fullscreen and issue no duplicate fullscreen-on command |
| `preview.slideshow.close-restore-windowed` | C/M | `app/action/process_helpers.rs`, `preview_pane/slideshow/{modal,controls}.rs` | close/Escape, slideshow opened from windowed state | typed close request; native external host effect | close overlay and exactly one fullscreen-off observation |
| `preview.slideshow.close-preserve-fullscreen` | C/M | `crates/katana-ui/src/app/action/process_helpers.rs`, `crates/katana-ui/src/preview_pane/slideshow/modal.rs`, `crates/katana-ui/src/preview_pane/slideshow/controls.rs` | close/Escape, slideshow opened while fullscreen | typed close request; native external host effect | close overlay and no fullscreen-off observation |
| `preview.slideshow-hover-highlight` | C/M | `views/panels/preview/side_panel_story.rs`, `app/action/dispatch.rs` | visible setting off/on | typed presentation setting request; KatanA state | individual false-to-true and true-to-false transitions; source closure may not merge both directions |
| `preview.slideshow-diagram-controls` | C/M | `views/panels/preview/side_panel_story.rs`, `app/action/dispatch.rs` | visible setting off/on | typed presentation setting request; KatanA state | individual false-to-true and true-to-false transitions; source closure may not merge both directions |
| `preview.side-rail.toc-capability` | C/M | `views/panels/preview/side_panels.rs` | `toc_visible` false/true | generic KUC `PreviewSideRail` capability projection | false omits the control and AX node; true exposes exactly one opaque target. KLE has no settings lookup or rail layout. |
| `preview.side-rail.toc-toggle` | C/M | `views/panels/preview/{side_panels,side_panel_toc}.rs`, `app/action/dispatch.rs` | visible TOC closed/open, pointer/keyboard/AccessKit activation | generic KUC `PreviewSideRail` retained panel state | same-root toggle/AX transition, sibling-panel exclusion and unchanged bootstrapped KatanA host observation. KLE does not write `show_toc`. |
| `preview.toc.pinned-left` | C/M | `crates/katana-ui/src/views/panels/preview/side_panel_toc.rs` | TOC visible, pinned, host-projected leading placement | generic KUC pinned side-panel layout | same-root leading dock/AX transition and no local KLE `TocPosition`/panel geometry. |
| `preview.toc.pinned-right` | C/M | `crates/katana-ui/src/views/panels/preview/side_panel_toc.rs` | TOC visible, pinned, host-projected trailing placement | generic KUC pinned side-panel layout | same-root trailing dock/AX transition and no local KLE `TocPosition`/panel geometry. |
| `preview.toc.hover-open` | C/M | `crates/katana-ui/src/views/panels/preview/side_panel_toc.rs` | TOC visible, unpinned, pointer enters button outside cooldown | generic KUC `PreviewSideRail` overlay state | one root/AX overlay-open transition without host mutation. |
| `preview.toc.hover-dismiss` | C/M | `crates/katana-ui/src/views/panels/preview/side_panel_toc.rs` | hover overlay open, pointer leaves button and overlay | generic KUC `PreviewSideRail` overlay state | one root/AX overlay-dismiss transition without host mutation. |
| `preview.toc.hover-cooldown` | C/M | `views/panels/preview/{side_panels,side_panel_toc}.rs` | pin toggle click then remaining pointer hover | generic KUC `PreviewSideRail` transient interaction state | click does not immediately re-open the hover overlay; root/AX record proves the suppression without KLE timer/state. |
| `preview.toc.hover-source-missing` | C/M | `crates/katana-ui/src/views/panels/preview/side_panel_toc.rs` | hover overlay, no active document or no matching preview pane | generic KUC retained empty panel state | visible empty/disabled state, no source target and no host mutation. |
| `preview.toc.pinned-source-missing` | C/M | `crates/katana-ui/src/views/panels/preview/side_panel_toc.rs` | pinned panel, no active document or no matching preview pane | generic KUC retained empty panel state | visible empty/disabled state, no source target and no host mutation. |
| `preview.toc.expand-all` | C/M | `crates/katana-ui/src/views/panels/toc/mod.rs` | TOC header Expand All | generic KUC `OutlineNavigator` retained disclosure state | all currently visible hierarchy nodes become open for one rendered root transition; no KLE force-open flag or host mutation. |
| `preview.toc.collapse-all` | C/M | `crates/katana-ui/src/views/panels/toc/mod.rs` | TOC header Collapse All | generic KUC `OutlineNavigator` retained disclosure state | all currently visible hierarchy nodes become closed for one rendered root transition; no KLE force-open flag or host mutation. |
| `preview.toc.item-leaf-select` | generated C/M leaf family | `crates/katana-ui/src/views/panels/toc/render.rs` | each visible leaf item | generic KUC `OutlineNavigator` opaque item target; actual KatanA TOC UI relay | pointer/keyboard/AccessKit selection and host scroll/current-anchor effect are separate for every item. KLE has no item index or line mapping. |
| `preview.toc.item-parent-select` | generated C/M leaf family | `crates/katana-ui/src/views/panels/toc/render.rs` | each visible parent header | generic KUC `OutlineNavigator` opaque item target; actual KatanA TOC UI relay | parent header selection is distinct from disclosure transition and reaches the actual host route. |
| `preview.toc.item-disclosure` | generated C/M leaf family | `crates/katana-ui/src/views/panels/toc/render.rs` | every expandable parent open/closed | generic KUC `OutlineNavigator` retained disclosure state | pointer/keyboard/AccessKit accordion state plus root/AX record; no host mutation or KLE accordion state. |
| `preview.toc.active-anchor-family` | generated C/M leaf family | `views/panels/toc/{anchor_ops,anchor_lookup_ops}.rs` | CodeOnly editor viewport; PreviewOnly hover then viewport fallback; Split editor/preview/neither; candidate/missing/first-item fallback | host-projected opaque active target; KUC `OutlineNavigator` presentation | every source-priority and fallback branch is generated separately. KatanA retains semantic anchor/line/source selection; KUC owns generic active styling/scroll-into-view; KLE owns neither. |
| `preview.toc.active-stability-family` | generated C/M leaf family | `state/toc.rs` | first/repeated/replaced candidate; below/at stability threshold; click-origin suppression; first/second editor-or-preview observation; preview-hover exception; same/different anchor; document reset | actual KatanA TOC state; KUC receives only resulting opaque active target | every debounce/suppression/generation branch is executed with deterministic monotonic input-frame timestamps, never fixed wait/sleep. KLE/KUC own no candidate, timer, origin or anchor state. |
| `preview.toc.active-auto-scroll` | C/M | `views/panels/toc/{mod,render}.rs`, Toc state closure | active target changed/unchanged, visible/nonvisible | generic KUC `OutlineNavigator` retained scroll state | changed active target scrolls once into view; unchanged target does not; KLE has no outline scroll state. |
| `preview.toc.vertical-guide` | C/M | `views/panels/toc/{mod,render}.rs` | host-projected guide capability false/true | generic KUC `OutlineNavigator` presentation | both guide states produce correct current root/AX artifact without KLE settings access. |
| `preview.toc.empty-outline` | C/M | `views/panels/toc/mod.rs` | document preview has zero outline items | generic KUC `OutlineNavigator` empty state | no selectable target, no host scroll/current mutation and explicit root/AX empty presentation. |
| `preview.side-rail.refresh-manual` | C/M | `views/panels/preview/side_panels.rs`, `app/action/dispatch.rs` | active document | opaque host target; KatanA `RefreshDocument { is_manual: true }` | one physical rail activation reaches the unchanged host handler and records exactly one document refresh path. |
| `preview.side-rail.search-toggle` | C/M | `views/panels/preview/side_panels.rs`, `app/action/dispatch_secondary.rs` | document-search closed/open | KUC opaque target plus actual KatanA search-UI input relay | obey the document-search open/close relay gate; no direct search-state/action fixture or KLE query storage. |
| `preview.side-rail.export-panel` | C/M | `views/panels/preview/{side_panels,side_panel_hover,side_panel_export}.rs` | button toggle, hover open/close, pointer transit between rail and panel | generic KUC `PreviewSideRail` retained panel/hover state | each open, close, hover-delay and sibling replacement is a separate root/AX transition; no KLE popup/geometry/timer/state. |
| `preview.side-rail.export-html` | C/M | `crates/katana-ui/src/views/panels/preview/side_panel_export.rs`, `crates/katana-ui/src/app/action/dispatch_tertiary.rs`, `crates/katana-ui/src/app/export.rs` | Export panel open, HTML item | opaque host target; KatanA `ExportDocument(Html)` | actual export handler/file result or declared host failure, without KLE format/path/payload logic. |
| `preview.side-rail.export-pdf` | C/M | `crates/katana-ui/src/views/panels/preview/side_panel_export.rs`, `crates/katana-ui/src/app/action/dispatch_tertiary.rs`, `crates/katana-ui/src/app/export.rs` | Export panel open, PDF item | opaque host target; KatanA `ExportDocument(Pdf)` | independent physical item and host effect; no aggregate export pass. |
| `preview.side-rail.export-png` | C/M | `crates/katana-ui/src/views/panels/preview/side_panel_export.rs`, `crates/katana-ui/src/app/action/dispatch_tertiary.rs`, `crates/katana-ui/src/app/export.rs` | Export panel open, PNG item | opaque host target; KatanA `ExportDocument(Png)` | independent physical item and host effect; no aggregate export pass. |
| `preview.side-rail.export-jpg` | C/M | `crates/katana-ui/src/views/panels/preview/side_panel_export.rs`, `crates/katana-ui/src/app/action/dispatch_tertiary.rs`, `crates/katana-ui/src/app/export.rs` | Export panel open, JPG item | opaque host target; KatanA `ExportDocument(Jpg)` | independent physical item and host effect; no aggregate export pass. |
| `preview.side-rail.story-panel` | C/M | `crates/katana-ui/src/views/panels/preview/side_panels.rs`, `crates/katana-ui/src/views/panels/preview/side_panel_hover.rs`, `crates/katana-ui/src/views/panels/preview/side_panel_story.rs` | button toggle, hover open/close, sibling replacement | generic KUC `PreviewSideRail` retained panel/hover state | each retained transition has independent root/AX evidence and unchanged host observation. Host-owned slideshow settings/actions remain separate leaves. |
| `preview.side-rail.tools-panel` | C/M | `crates/katana-ui/src/views/panels/preview/side_panels.rs`, `crates/katana-ui/src/views/panels/preview/side_panel_hover.rs`, `crates/katana-ui/src/views/panels/preview/side_panel_tools_inner.rs` | button toggle, hover open/close, sibling replacement | generic KUC `PreviewSideRail` retained panel/hover state | each retained transition has independent root/AX evidence and unchanged host observation. View/split commands remain separate leaves. |
| `preview.side-rail.meta-info` | C/M | `crates/katana-ui/src/views/panels/preview/side_panels.rs`, `crates/katana-ui/src/app/action/dispatch_tertiary.rs` | active document absent/present, Info activation | opaque document-meta host target; KatanA `ShowMetaInfo` | no target/no mutation when absent; present target reaches unchanged host metadata presentation without KLE path ownership. |
| `preview.toc.select-line` | generated C/M leaf family | `crates/katana-ui/src/views/panels/preview/side_panel_toc.rs`, `crates/katana-ui/src/views/panels/preview/side_panel_toc_ops.rs` | each current TOC row and current view mode | opaque source target; actual KatanA UI input relay | each row selection follows the host's current scroll-clear/PreviewOnly guard. KLE has no logical line/scroll state or direct `AppState` mutation. |
| `document.multi-tab-state` | C/M | `app/action/process_document.rs`, `editor_undo.rs`, tab/navigation sources reached by closure | edit/switch/edit/undo across documents | opaque document surface identity; KatanA state | per-document buffer/cursor/history/scroll isolation |
| `document.tab-operation-family` | generated C/M leaf family | `app_frame/{ui,tab_toolbar}.rs`, `top_bar/tab_bar` closure, `process_tabs.rs`, document action handlers | each current direct select, previous/next, close/restore, close-scope, pin, reorder and group operation | generic KUC `TabStrip` event; KLE opaque transit; KatanA state | each document-changing source branch has its own visible path, root/AX record and actual KatanA document/state/persistence effect; no static operation count |
| `frame.breadcrumb-navigation-family` | generated C/M leaf family | `app_frame/{tab_toolbar,breadcrumbs}.rs`, tree candidate/action closure | virtual label, no-workspace labels, workspace prefix menu open/close, every eligible child selection | generic KUC `BreadcrumbNavigator`; KLE opaque descriptor/event transit; KatanA document/tree state | every visible segment/menu branch has root/AX evidence; selection reaches the unchanged KatanA document action. KLE has no path split, tree lookup, candidate filter or geometry. |
| `frame.source-address-family` | generated C/M/N leaf family | `app_frame/tab_toolbar.rs`, `top_bar/url_source.rs`, `app/action/{dispatch_secondary,url_source}.rs`, `state/url_tab.rs` | input change, button/Enter submit, blank rejection, history hidden/open/select, URL validation, local document/HTML, HTTP loading/success/error/timeout/disconnect | generic KUC `SourceAddressBar`; KUC retains edit/preedit/history UI, KLE performs only a non-persistent one-shot typed submit forwarding; KatanA source/document/browser host | every source branch is generated from current source. KLE/KUC retain no URL validation, network, source payload, history, document-tab or browser state. This family permits only a non-Clone/non-Serialize submit consumed by the host bridge; generic TabStrip group-name editing has its separately specified one-shot path. Host E2E is real transport/filesystem and rejects direct `UrlTabState` setup. |
| `diagnostic.problems-panel-toggle` | C/M | `crates/katana-ui/src/views/app_frame/ui.rs`, `crates/katana-ui/src/views/top_bar/status_bar.rs`, `dispatch_secondary.rs` | Problems count button, panel closed/open | generic KUC retained `StatusStrip`/`DiagnosticsPanel` state; KLE host presentation only | count, pointer/keyboard/AccessKit activation, open/close root/AX state and unchanged KatanA host observation |
| `status.message-severity` | generated C/M leaf family | `top_bar/status_bar.rs`, `app_frame/ui.rs` | ready/no-status, error, warning, success, info | generic KUC `StatusStrip`; KLE typed status DTO; KatanA state | localized text/icon/severity, root/AX record and no host mutation |
| `status.dirty-indicator` | C/M | `top_bar/status_bar.rs`, `state` dirty source reached by closure | active document clean/dirty | generic KUC `StatusStrip`; KLE typed presentation | visible/accessibility state follows host revision with no local dirty calculation |
| `status.activity` | generated C/M leaf family | `top_bar/status_bar.rs`, source-spanned activity producer closure | none, one, multiple host activity descriptors | generic KUC `StatusStrip`; opaque host presentation transits KLE unchanged | deterministic activity/empty presentation; non-editor source siblings retain an explicit rationale, not an omitted path |
| `diagnostic.problems-panel-close` | C/M | `crates/katana-ui/src/views/panels/problems/ui.rs` | panel open -> close button | generic KUC retained `DiagnosticsPanel` state; KLE has no event mapping | pointer/keyboard/AccessKit close updates same-root state and leaves KatanA host unchanged |
| `diagnostic.problems-scope-open-tabs` | C/M | `crates/katana-ui/src/views/panels/problems/scope.rs`, `ui.rs` | select open-documents scope | generic KUC stable scope key; KLE supplies scope membership descriptors only | visible generic file set and current root/AX record update without label comparison or KatanA mutation |
| `diagnostic.problems-scope-active-tab` | C/M | `crates/katana-ui/src/views/panels/problems/scope.rs`, `ui.rs` | select active-document scope | generic KUC stable scope key; KLE supplies scope membership descriptors only | visible generic file set and current root/AX record update without label comparison or KatanA mutation |
| `diagnostic.expand-all` | C/M | `crates/katana-ui/src/views/panels/problems/ui.rs`, `diagnostics_renderer.rs` | panel open, visible files | generic KUC retained disclosure state; KLE has no event mapping | every visible file expands once in same root; no host mutation or stale disclosure state |
| `diagnostic.collapse-all` | C/M | `crates/katana-ui/src/views/panels/problems/ui.rs`, `diagnostics_renderer.rs` | panel open, visible files | generic KUC retained disclosure state; KLE has no event mapping | every visible file collapses once in same root; no host mutation or stale disclosure state |
| `diagnostic.file-collapse` | generated C/M leaf family | `diagnostics_renderer.rs` | each visible file group, open/closed | generic KUC retained disclosure; KLE typed file descriptor | per-file stable key, pointer/keyboard/AccessKit transition, no document mutation |
| `diagnostic.empty-state` | C/M | `crates/katana-ui/src/views/panels/problems/ui.rs` | selected scope has zero official diagnostics | generic KUC `DiagnosticsPanel`; KLE typed diagnostics DTO | zero-state visibility/AX and no unavailable fix action |
| `diagnostic.row-select-jump` | generated C/M leaf family | `diagnostics_renderer.rs`, `process_document.rs` | each official diagnostic row | generic KUC diagnostic list event; KLE typed location request; KatanA state | exact selected document/line, search-modal/view transition and scroll effect through actual host E2E |
| `diagnostic.docs-request` | generated C/M leaf family | `diagnostics_renderer.rs`, `process_linter.rs` | diagnostic with/without documentation capability | generic KUC link event; KLE opaque typed request; KatanA external host | enabled/hidden predicate and native browser/no-mutation result without URL ownership in KLE/KUC |
| `diagnostic.fix-all-visible` | C/M | `crates/katana-ui/src/views/panels/problems/ui.rs`, `crates/katana-ui/src/views/panels/problems/bulk_fixes.rs`, `document_edit.rs` | selected visible paths have/no applicable batches | generic KUC event; KLE opaque typed request; KatanA host | exact visible-scope batch, review/buffer/undo/refresh effect or no-op |
| `diagnostic.fix-file` | generated C/M leaf family | `diagnostics_renderer.rs`, `bulk_fixes.rs`, `document_edit.rs` | each visible file with/without applicable fix | generic KUC event; KLE opaque typed request; KatanA host | current file capability, host effect/no-op and no KLE lint batch construction |
| `diagnostic.fix-entry` | generated C/M leaf family | `diagnostics_renderer.rs`, `document_edit.rs` | each official diagnostic with/without applicable fix | generic KUC event; KLE opaque typed request; KatanA host | exact single-fix effect/no-op, refresh and undo semantics |
| `diagnostic.fix-preview` | generated C/M leaf family | `crates/katana-ui/src/views/panels/problems/fix_preview_renderer.rs`, `crates/katana-ui/src/views/panels/problems/fix_preview_model.rs` | hover/focus preview; source present/missing/invalid; truncation | generic KUC diff preview; host-projected opaque presentation transits KLE unchanged | preview opens/closes through root/AX, fixed source line/replacement/truncation facts and no content mutation |

### Non-Aggregate Authoring Leaf Catalog

The source-derived manifest must contain the following individual action IDs;
none may be represented by `toolbar`, `context`, `code`, `authoring`, or an
equivalent group-level pass.

| Surface | Required IDs |
| --- | --- |
| toolbar | `toolbar.bold`, `toolbar.italic`, `toolbar.strikethrough`, `toolbar.inline-code`, `toolbar.heading-1`, `toolbar.heading-2`, `toolbar.heading-3`, `toolbar.bullet-list`, `toolbar.numbered-list`, `toolbar.blockquote`, `toolbar.code-dropdown`, `toolbar.image-file` |
| context root/actions | `context.save`, `context.format-editable-markdown`, `context.bold`, `context.italic`, `context.strikethrough`, `context.inline-code`, `context.heading-1`, `context.heading-2`, `context.heading-3`, `context.bullet-list`, `context.numbered-list`, `context.blockquote`, `context.code-submenu`, `context.horizontal-rule`, `context.insert-link`, `context.insert-table`, `context.ingest-file`, `context.ingest-clipboard-image` |
| code submenu | `code.text`, `code.markdown`, `code.bash`, `code.zsh`, `code.mermaid`, `code.drawio`, `code.plantuml`, `code.json`, `code.yaml`, `code.toml`, `code.rust`, `code.typescript`, `code.javascript`, `code.python`, `code.html`, `code.css`, `code.sql` |
| clipboard ingest route | `image.file`, `image.clipboard-image`, `image.clipboard-file-url` |
| image paste selection preservation | `paste.raw-image-selection-preservation`, `paste.file-url-selection-preservation` |
| locked TextEdit input semantics | `text.editable`, `text.ime-preedit`, `text.ime-commit`, `text.ime-empty-or-newline-reject`, `text.ime-delete-surrounding`, `text.enter-newline`, `text.tab-not-editor-indent`, `text.shift-tab-not-editor-unindent`, `text.delete-branch-family`, `text.cursor-navigation-family`, `text.cursor-restore`; the generated closure expands every fixed `egui 0.36.1` event/mutation branch rather than accepting these family labels |
| text clipboard/history | `clipboard.text-copy-selection`, `clipboard.text-copy-empty-selection`, `clipboard.text-cut-selection`, `clipboard.text-cut-readonly`, `clipboard.text-paste`, `clipboard.text-paste-readonly`, `clipboard.text-paste-empty-or-error`, `history.undo`, `history.redo`, `history.host-external-sync`, `history.document-isolation` |
| shortcut input arbitration | `shortcut.priority`, `shortcut.text-entry-reservation`, `shortcut.image-paste-exception`, `shortcut.command-inventory-family` |
| document tab operations | `document.multi-tab-state`, `document.tab-operation-family` |
| editor-frame navigation | `frame.breadcrumb-navigation-family`, `frame.source-address-family` |
| document search | `search.open-command`, `search.toggle-open`, `search.toggle-close`, `search.open-focus`, `search.query-empty`, `search.query-filter`, `search.query-nonempty-no-result`, `search.next-button`, `search.previous-button`, `search.next-enter`, `search.previous-shift-enter`, `search.next-arrow-down-focused`, `search.previous-arrow-up-focused`, `search.navigation-no-match`, `search.next-wrap`, `search.previous-wrap`, `search.escape-lost-focus-close`, `search.escape-focused-no-close`, `search.editor-scroll`, `search.preview-scroll`; query/close use the required actual-KatanA UI host-input relay, not direct state mutation or an action-only fixture |
| Problems status action | `diagnostic.problems-panel-toggle` |
| status and Problems panel | `status.message-severity`, `status.dirty-indicator`, `status.activity`, `diagnostic.problems-panel-close`, `diagnostic.problems-scope-open-tabs`, `diagnostic.problems-scope-active-tab`, `diagnostic.expand-all`, `diagnostic.collapse-all`, `diagnostic.file-collapse`, `diagnostic.empty-state`, `diagnostic.row-select-jump`, `diagnostic.docs-request`, `diagnostic.fix-all-visible`, `diagnostic.fix-file`, `diagnostic.fix-entry`, `diagnostic.fix-preview` |
| document tab-strip seeds | `document.tab.select`, `document.tab.previous`, `document.tab.next`, `document.tab.close`, `document.tab.close-dirty-pending-unmounted`, `document.tab.close-other`, `document.tab.close-all`, `document.tab.close-right`, `document.tab.close-left`, `document.tab.restore-closed`, `document.tab.pin-toggle`, `document.tab.reorder`, `document.tab.group-create`, `document.tab.group-add`, `document.tab.group-remove`, `document.tab.group-rename`, `document.tab.group-recolor`, `document.tab.group-ungroup`, `document.tab.group-close`, `document.tab.group-collapse`, `document.tab.group-inline-rename-cancel` |
| preview and split | `view.command-toggle-split`, `view.command-toggle-code-preview.from-split`, `view.command-toggle-code-preview.from-preview`, `view.command-toggle-code-preview.from-code`, `view.set-mode.split`, `view.set-mode.preview-only`, `view.set-mode.code-only`, `view.split-direction.horizontal-to-vertical`, `view.split-direction.vertical-to-horizontal`, `view.pane-order.editor-to-preview-first`, `view.pane-order.preview-to-editor-first`, `view.scroll-sync.enable`, `view.scroll-sync.disable`, `split.resize-horizontal`, `split.resize-vertical`, `split.layout-persistence`, `split.layout-ratchet-rejection`, `preview.markdown-scroll`, `preview.editor-scroll-sync`, `preview.hover-source-range`, `preview.select-and-jump`, `preview.search-highlight`, `preview.task-toggle-valid`, `preview.task-toggle-stale-or-missing`, `preview.back-to-top`, `preview.document-frame-open`, `preview.html-browser-open`, `preview.fullscreen-open`, `preview.fullscreen-close`, `preview.slideshow.open.windowed`, `preview.slideshow.open-already-fullscreen`, `preview.slideshow.close-restore-windowed`, `preview.slideshow.close-preserve-fullscreen`, `preview.slideshow-hover-highlight`, `preview.slideshow-diagram-controls`, `preview.side-rail.toc-capability`, `preview.side-rail.toc-toggle`, `preview.toc.pinned-left`, `preview.toc.pinned-right`, `preview.toc.hover-open`, `preview.toc.hover-dismiss`, `preview.toc.hover-cooldown`, `preview.toc.hover-source-missing`, `preview.toc.pinned-source-missing`, `preview.side-rail.refresh-manual`, `preview.side-rail.search-toggle`, `preview.side-rail.export-panel`, `preview.side-rail.export-html`, `preview.side-rail.export-pdf`, `preview.side-rail.export-png`, `preview.side-rail.export-jpg`, `preview.side-rail.story-panel`, `preview.side-rail.tools-panel`, `preview.side-rail.meta-info`, `preview.toc.select-line` |
| preview TOC detail | `preview.toc.expand-all`, `preview.toc.collapse-all`, `preview.toc.item-leaf-select`, `preview.toc.item-parent-select`, `preview.toc.item-disclosure`, `preview.toc.active-anchor-family`, `preview.toc.active-stability-family`, `preview.toc.active-auto-scroll`, `preview.toc.vertical-guide`, `preview.toc.empty-outline`; each generated family is expanded to every fixed-revision branch before acceptance |

Each ID has independent state predicates and an independent physical input
route. The generated closure may add further leaf IDs; it may never collapse
or remove these IDs. Every entry still requires the source/class-correct
input-origin/KUC record+AccessKit/class-appropriate declared-effect proof
quartet. Direct-root leaves use KLE RawInput; KatanA-owned shortcut leaves use
the unchanged physical KatanA router plus same-run KUC focus evidence.

#### Source-Derived TabStrip Operation Catalog (2026-08-23)

The following catalog is the fixed-source basis for the generated
`document.tab-operation-family`; a visible generic tab or a proposal enum value
is not a substitute for any row. KatanA remains read-only at
`4f6a6287c650a38633c7baeb544a92e739c68567`.

| Leaf family | Fixed KatanA source and physical trigger | Required KUC presentation and KLE boundary | Required unchanged-host effect proof |
| --- | --- | --- | --- |
| select | `views/top_bar/tab_bar/tab_item/mod.rs:67-113`, tab body primary click | active/dirty/title truncation/tooltip; KUC sends one opaque target proposal and KLE cannot derive document identity | `SelectDocument`, active document and reveal/scroll state |
| previous / next | `views/top_bar/tab_bar/nav.rs:12-79`, navigation button click; disabled below two documents | KUC-owned disabled button, tooltip and AccessKit alternative; no KLE document order | cyclic `SelectDocument` plus `process_tabs.rs:170-189` reveal request |
| overflow and active reveal | `views/top_bar/tab_bar/mod.rs:44-83`, `tab_item/mod.rs:90-94`, excess-width and post-select state | KUC-owned horizontal clip/scroll/overflow UI; no KLE geometry or scroll cache | unchanged host selection produces the current active reveal outcome |
| close and dirty pending | `tab_item/mod.rs:77-88`, hover close or pinned-tab affordance | KUC close affordance and accessible label; proposal never force-closes locally | `CloseDocument`, including `process_document.rs:179-198` dirty confirmation/no-document-mutation branch |
| close scopes | `tab_context_menu.rs:37-63`, tab context menu in exact order `Close`, `Close Others`, `Close All`, `Close Tabs to Right`, `Close Tabs to Left` without a separator | KUC menu/disabled branches and closed opaque target transit | close-other/all/right/left each updates recently-closed state through `process_tabs.rs:5-105` |
| restore closed | `tab_context_menu.rs:28-34`, conditional menu item | KUC conditional item and no-op when unavailable | `process_tabs.rs:138-168` reopening and pinned-state restoration |
| pin / unpin | `tab_context_menu.rs:65-79`, context action and pinned affordance | KUC-owned pinned visual state and opaque proposal; no KLE group/order state | `TogglePinDocument`, ordered/group-exclusion persistence via `process_tabs.rs:107-136` |
| create / add / remove group | `tab_context_menu.rs:82-147`, tab context menu/submenu | KUC group popup/menu and opaque tab/group capability targets | `CreateTabGroup`, `AddTabToGroup`, `RemoveTabFromGroup` through `process_groups.rs:27-121` |
| collapse and group popup | `group_header.rs:22-35`, primary/secondary group-header input | KUC retained collapse/popup state; KLE has no group identity or popup geometry | `ToggleCollapseTabGroup` persistence through `process_group_lifecycle.rs:52-88` |
| rename group | `group_header_popup.rs:34-51,84-96`, inline text input and Enter | KUC IME/input/focus lifecycle; non-Clone/non-Serialize name submission only | `RenameTabGroup` persistence; changed-draft plus `lost_focus && Enter`, cancel, and stale branches independently proven. Escape is not a fixed-source parity claim. |
| recolor / ungroup / close group | `group_header_popup.rs:54-110`, palette or group popup buttons | KUC palette uses opaque host-issued swatch targets; no KLE color/name/member data | `RecolorTabGroup`, `UngroupTabGroup`, `CloseTabGroup`; close moves members to recently closed via `process_group_lifecycle.rs:5-50` |
| drag reorder / group move / cancel | `tab_drag.rs:6-30`, `drag.rs:13-107`, press-move-release | KUC drag capture/ghost/drop/invalid-drop state and opaque placement target; no KLE coordinates or membership | `ReorderDocument { new_group_id }` or source-defined no-op, with persistence and stale/invalid rejection |

Current KUC root work covers only physical select, group-collapse,
previous/next navigation, ordinary close request, pinned-tab unpin, horizontal
scroll/active reveal, and generic focused Enter/Space plus AccessKit Click
routes. The latter keyboard/AccessKit support is a KUC accessibility extension,
not fixed-KatanA tab-keyboard parity. Every menu branch, group popup/rename/
palette branch, rejection rollback, KLE opaque transit, and fixed-host proof
remains incomplete and is a release blocker.

##### Exact Tab And Group Branch Preconditions (2026-08-23)

The following preconditions are separate leaf predicates. KUC receives the
result only as explicit non-wire visibility/enabled presentation and opaque
routes; it must not infer the predicates from a label, tab index, group name,
or a local KatanA model. KLE receives none of them.

| Leaf | Fixed-source condition | KUC retained behavior | Fixed-host effect still required |
| --- | --- | --- | --- |
| tab context menu visibility | virtual tab hides pin/group/restore operations; pinned tab hides group operations (`tab_context_menu.rs:19-35`) | host projects every visible/enabled item; hidden items get no route or AX node | KatanA virtual/pinned no-op and visible branch effects |
| close / close scopes | right-click menu closes after selection; close ignores pinned tab, dirty close enters confirmation (`tab_context_menu.rs:37-63`, `process_document.rs:179-199`) | one opaque proposal and retained menu close; KUC never confirms or removes a tab | pending confirmation, recently-closed update, active recomputation |
| restore | non-virtual tab and recently-closed availability (`tab_context_menu.rs:28-34`) | explicit menu visibility; unavailable route is absent | prior pinned state and restored active document |
| pin / unpin | non-virtual tab only (`tab_context_menu.rs:65-79`) | explicit toggle presentation, no local reorder/group removal | KatanA ordered pinned state and group exclusion |
| add to group | non-virtual, unpinned tab; existing candidates are non-demo groups that do not contain the tab (`tab_context_menu.rs:82-118`) | private submenu routes only for host-projected candidates | group-existence/demo rejection and persisted membership |
| create group | direct item only when no non-demo group exists and target is ungrouped (`tab_context_menu.rs:99-108`) | opaque create proposal, then wait for host projection to start rename | group creation, default host color, host inline-rename state |
| remove from group | only for a tab that host projects as grouped (`tab_context_menu.rs:82-147`) | one opaque removal proposal; no local empty-group cleanup | all-group removal and empty-group deletion |
| group popup | secondary header click only when group is not system demo (`group_header.rs:28-35`) | private retained popup toggles; demo header has no popup route | no-op/visibility conditions remain host-proven |
| group rename | rename state is host-projected; commit only after changed draft with `lost_focus && Enter` (`group_header_popup.rs:34-51`) | KUC-private IME draft/focus/one-shot submission; Enter alone must not commit/close | persisted name or unchanged state on cancel/stale/reject |
| palette | seven host-projected visual swatches; selected swatch stroke; selecting does not close popup (`group_header_popup.rs:54-102`) | generic KUC display color/accessibility + opaque swatch target; popup remains open | recolor persistence and same-frame rename/recolor precedence |
| ungroup / close group | popup buttons (`group_header_popup.rs:104-110`) | close popup then forward one opaque proposal | members retained for ungroup; close group moves members to recently closed |
| popup outside close | primary press outside popup and header closes it (`group_header.rs:106-141`) | retained local close/focus return, no host proposal | unchanged host state |

The fixed source has no explicit Escape close handler in this tab-strip area.
KUC may provide Escape as a generic accessibility extension, but it is tested
and reported separately from source-defined KatanA parity.

#### Tab-Strip Overflow Source Correction (2026-08-23)

The fixed KatanA source does not expose an overflow popup or an overflow host
action for document tabs. `views/top_bar/tab_bar/mod.rs:44-74` chooses a
horizontal `egui::ScrollArea` only when estimated tab/group width exceeds the
strip width, and `mod.rs:102-121` consumes the active-tab scroll request.
`tab_item/mod.rs:82-84` performs `scroll_to_me(Center)` for the active item.
Therefore the parity leaf is **KUC-retained horizontal clipping/scroll and
active-tab reveal**, with no KLE/KatanA proposal. Any `OpenOverflow` generic
capability remains outside the KatanA v0.1.0 parity matrix unless a separate
non-parity requirement is approved. The retained scroll leaf still requires
pointer, keyboard, AccessKit, artifact, resize, replacement, and no-host-effect
evidence.

#### Explicit User-Mandated Extensions

The following two leaves are explicit v0.1.0 user requirements. They are not
direct KatanA editor UI claims: the fixed revision has the `ReplaceText` host
handler but no source-visible replace control. They must therefore be reported
as `user_mandated_extension`, separately from source-origin parity, while still
requiring the exact KUC/KLE/KatanA proof quartet.

| Requirement ID | Required generic KUC UI | Exact KatanA host route | Prohibitions |
| --- | --- | --- | --- |
| `replace.current` | `SearchStrip` replacement field and replace-current control | `AppAction::ReplaceText` -> `dispatch.rs` -> `DocumentEditOps::handle_replace_text` | No KLE query/match/range/byte conversion/content mutation; no source-derived KatanA replace-control claim. |
| `replace.all` | `SearchStrip` replacement field and replace-all control | One host-derived `ReplaceText` transaction per current matched occurrence, with exact host snapshot/revision enforcement | No KLE iteration, regex/match engine, range conversion, aggregate-only proof, or fabricated KatanA UI source. |

#### KDV-informed full-surface Storybook acceptance contract (2026-08-14)

KDV is the implementation reference for *artifact discipline*, not an excuse
to reuse its viewer-specific scenarios.  Its `tools/kdv-storybook/src/args.rs`
separates interactive launch, window smoke stages, and headless live
acceptance; `window_loop.rs` writes before/after live-acceptance PNGs after an
actual KUC action; and its release scripts regenerate stage PNGs, an acceptance
contact sheet, checksum manifests, and performance evidence. KLE MUST apply the same
discipline to the editor's source-derived leaves, using editor-specific input
and effects.

KLE's single source of truth is a `FullEditorScenarioManifest`. It contains
one or more ordered class-correct input steps for every leaf in the parity
manifest, including the user-mandated `replace-current` and `replace-all`
leaves. A step is invalid unless it declares all of the following:

| Field | Required fact | Owner |
| --- | --- | --- |
| `leaf_id` / `step_id` | exactly one source-derived leaf and deterministic stage order | KLE scenario binding |
| input origin | `editor_direct_ui`/host-continuation/user-extension: exact public KLE `RawInput` and focused surface supplied to `EguiLanguageEditor::show`; `editor_shortcut`: physical KatanA RawInput/router trace plus same-run KUC focus/retained-input record | KLE -> KUC adapter or unchanged KatanA router |
| KUC frame proof | expected TextSurface/CommandChrome/ContextMenu record, typed event, current hit target and AccessKit node | KUC |
| visible proof | named PNG encoded directly from the same opaque KUC composited root frame, plus semantic pixel/text assertions | KUC compositor |
| declared-effect proof | `kuc_retained_ui_effect` root transition plus unchanged host observation, or typed callback and concrete KatanA document/state/file effect for a host-effect class | KUC root / KLE harness / KatanA host |

The manifest MUST cover actual typing, Japanese IME commit, exact `⭐️` VS16
color-glyph text, grapheme caret/delete, selection/read-only behavior, line
numbers/diagnostic gutter/popup, floating Markdown toolbar, secondary-click
and keyboard/AccessKit context-menu traversal, all authoring and code-kind
leaves, save/format/three image-ingest routes, find/replace navigation,
view/split navigation, dirty/undo/external refresh, and shortcut arbitration.
One aggregated screen or a counter of controls does not cover a leaf.

`--interactive`, headless acceptance, and `--motion-artifact` MUST drive the
same retained public KUC component-tree contract. Direct-root stages use
`EguiLanguageEditor::show`; KatanA-owned shortcut stages additionally record
the unchanged KatanA physical router and must not claim that Storybook mounted
KLE in KatanA. The interactive window is used to produce review media;
headless execution is the deterministic automated oracle. KLE may bind editor-domain fixtures,
host callbacks, and scenario state, but it MUST NOT raster text, calculate
glyphs/lines/hit rectangles, synthesize actions from labels, or compose a
fallback surface.  KUC owns the generic RawInput processing, IME/emoji
platform raster, geometry, hit/action records, AccessKit tree, and artifact
composition.  A KLE-specific generic UI feature found while implementing this
contract is a KUC change first, followed by a KLE consumer-only binding.

KUC MUST expose the retained text-command root's complete artifact frame as a
single generic output, including root bounds, ordered TextSurface / toolbar /
search / floating / context-menu layer provenance and the final composited
pixel frame.
KLE may retain that opaque KUC output for inspection but MUST NOT reconstruct
its layer order, join child paint plans, convert RGBA into window pixels, or
manufacture a missing child.  `EguiKucArtifactAggregate` and Storybook-local
pixel conversion are therefore migration targets, not permitted long-term
abstractions.  This restriction applies equally to headless artifact writing
and interactive presentation: all generic composition belongs in KUC.

For every manifest run, the artifact gate MUST generate numbered PNGs, a
contact sheet, a machine-readable manifest, SHA-256 checksums, a deterministic
GIF, and an MP4 review video.  The gate must decode the MP4 and prove each
decoded frame hash against its corresponding compositor PNG.  It must reject a
missing, empty, duplicate, wrong-dimension, stale, or non-KUC-composited frame.
It must also reject `egui` shape counts, minifb/fallback pixels, fixture-only
canvases, KLE-local glyph/text rendering, and static text parsing as proof of
a rendered interaction.  Screenshot/GIF/MP4 review artifacts make progress
inspectable; correctness still requires the KUC record/AccessKit and
class-appropriate declared-effect assertions for the same `step_id`.

The release gate MUST fail closed when any source-derived leaf lacks a stage,
any stage lacks a current KUC record or AccessKit node, a host-effect typed
action lacks its KatanA effect, a retained-UI leaf lacks its unchanged-host
observation, an artifact checksum is stale, or the interactive and headless
runners do not declare the same surface contract/version. This is a fully
automated release condition; no manual test is used to fill a missing proof.

#### KDV-informed artifact discipline and required KLE extension

KDV is the reference for release-artifact discipline. Its argument model
separates `--interactive`, headless smoke/acceptance, and live-acceptance
artifact execution; its release scripts validate generated PNGs and write a
checksum manifest. KLE adopts those properties with the following deliberate
boundary difference:

| KDV practice | KLE v0.1.0 application | KLE prohibition |
| --- | --- | --- |
| separate interactive and deterministic headless runs | all three KLE runners declare and use one retained public KLE/KUC surface contract version | treating an interactive screenshot as a test oracle |
| stage-specific acceptance PNGs/contact sheet/checksums | one numbered PNG per source-derived editor leaf stage, with contact sheet and SHA-256 manifest | an unnumbered representative editor screenshot or static fixture |
| PNG dimensions/non-empty/color/difference validation | parse each final KUC-frame PNG; validate dimensions, alpha, non-empty pixels, duplicate/stale hashes, and expected semantic pixel changes | checking egui shape count, a window texture, or a local canvas instead of KUC RGBA |
| KLE user-required review extension beyond KDV's verified PNG artifact flow | GIF and MP4 are generated from the exact numbered KUC-frame sequence and decoded back to canonical frame hashes | KDV canvas, viewer node factory, or any fallback renderer copied into KLE |

The KLE manifest binds every stage to a KUC record/AccessKit assertion and, for
host-effect leaves, a KLE-owned actual `KatanaApp` host-E2E locator. Thus KDV's
artifact rigor makes the result inspectable without relaxing the independent
source and behavior proofs.

### 1. Text editing

- KUC `TextSurface` が multiline input、read-only interaction policy、IME、caret、selection、copy/cut/paste UI、grapheme measurement と AccessKit を所有する。
- KatanA host が本文、dirty、save、preview/search/diagnostics refresh、external undo、document/workspace isolation を authoritative に所有する。
- current KUC root の physical input は document-edit single-consumption transport として一度だけ host へ渡り、unchanged KatanA `UpdateBuffer` 又は source-derived no-mutation result を得る。KLE は buffer、origin、undo unit、edit transform、clipboard text を state/API に保持しない。
- read-only/reference の reject、focus/selection/copy、normal input、IME、delete、cut/paste、authoring の document update は個別 leaf であり、byte/character/grapheme boundary conversion は KUC 又は KatanA host が所有する。

検証:

- KUC actual RawInput/AccessKit の edit/read-only/IME/grapheme tests と KatanA actual buffer/dirty/undo/document-isolation host E2E。
- KLE AST で local buffer/write/origin/undo/clipboard implementation と payload serialization を拒否する。
- Storybook は同じ root record と execution record を review artifact に join するだけで、buffer oracle を代替しない。

### 2. Cursor and selection

- KUC `TextSurface` が cursor/selection の retained interaction、pointer/keyboard/IME selection、AccessKit を所有する。KatanA host が programmatic selection と authoring 後 pending cursor restore の authoritative state を所有する。
- KLE は host-projected opaque selection revision と KUC current-frame fact を forwarding するだけで、cursor/selection/range を計算、保持、復元しない。
- authoring restore、selection/no-selection、reference read-only、document switch は host effect leaf として KatanA の next editor cycle を検証する。toolbar/context menu は KUC root の child であり KLE core に持たない。

検証:

- KUC actual RawInput/AccessKit cursor-selection tests、KatanA authoring cursor restore host E2E、Japanese/`⭐️` VS16/ZWJ regression。
- KLE AST が range conversion、selection persistence、KUC child widget を拒否すること。

### 3. Line numbers and gutter

- KUC generic gutter が行番号、active/hover/marker priority、row bounds、hit target、scroll alignment、theme/AccessKit を所有する。
- KatanA host が diagnostic/source mapping と document/scroll outcome を所有し、KLE は host-projected opaque gutter target/revision を一度 forwarding するだけである。`line_index`/表示番号/row geometry は KLE request/state に存在してはならない。
- raw `egui::Visuals` fallback、KLE local gutter model、hard-coded color は禁止する。

検証:

- KUC row/marker RawInput/AccessKit tests、actual KatanA jump/scroll/no-mutation E2E、KLE AST opaque-target provenance checks。

### 4. Search

- KUC generic `SearchStrip` が query/replacement input、focus/IME/key lifecycle、open/close、result presentation、disabled state、AccessKit を所有する。
- KatanA host が Markdown-aware query filtering、match/range/index、wrap navigation、PreviewOnly fallback、scroll target、replace-current/replace-all snapshot/range conversion、dirty/undo/refresh を所有する。KLE は search engine、regex parser、query/match state、range/line/scroll calculation、content mutationを持たない。
- query/replacement/current-match は one-shot transport、next/previous は closed direction request、active result/gutter/preview target は opaque host descriptors として KLE を通過する。KatanA の表示テキスト/code inclusion、URL/HTML attribute exclusion、character boundary behavior は host E2E で閉じる。

#### KatanA reference distinction (2026-08-21)

固定した read-only KatanA source の document search bar には query、previous/next、件数、close、
case/whole-word/regex toggle がある。`OpenDocSearch`/`ToggleDocSearch`、初回 focus、
Enter/Shift+Enter/Up/Down/Escape、zero-result の navigation disabled、wrap navigation、
character-offset match conversion、editor/preview highlight、document switch と external buffer refresh
は reference requirement である。index 対象は Markdown `Event::Text` と `Event::Code` だけであり、
link/image URL と HTML content/attribute は除外される。

ただし reference search calculator は現在 case-insensitive literal matching を常に使い、三つの
option value を読まない。従って v0.1.0 は visible control と host-projected current state を再現するが、
source が証明しない option semantic を KatanA parity と主張してはならない。KUC は各 option を generic
opaque one-shot capability として出し、actual host が no-mutation projection 又は将来の behavior change
を決める。

visible document `replace` / `replace-all` は KatanA reference UI には**存在しない**。これは v0.1.0
に対する user-mandated addition であり、reference parity と偽装してはならない。KatanA 既存の
`ReplaceText` route は document search replacement ではなく diagnostic/lint span replacement 用である。
追加 control は KUC に置き、host だけが match selection、byte/character conversion、read-only reject、
mutation、undo、dirty state、preview/search/diagnostics refresh、all/no-match behavior を所有する。
generic capability callback 又は `ReplaceText` direct call は host-E2E evidence ではない。

検証:

- empty/case/multibyte/wrap/zero-result、open/focus/Enter/Shift+Enter/Up/Down/Escape、current/all/no-match/read-only/stale/document-switch/dirty/undo/scroll を KUC RawInput/AccessKit と actual KatanA E2E で leaf ごとに検証する。
- KLE AST が `SearchQuery` persistence、regex/match/range conversion、local replace を拒否する。Storybook は同じ execution record の review artifact のみを出力する。

### 5. Syntax highlight

- KatanA host が document extension/theme/syntax/Markdown/KMM semantics を決定し、KUC が host-projected semantic span/source target を PlatformFontCatalog で layout/raster/hit-test する。KLE は span/range/theme/highlighter cache を実装しない。
- fixed revision の `TextEdit::multiline` source reachability hold が解決するまで、syntax-visible parity を推測してはならない。reachable branch だけを typed host descriptor として KUC root に forward する。
- Mermaid / Draw.io block の構文認識は KatanA/KCF/KMM 側の責務である。

検証:

- source-closure の extension/theme/span reachability、KUC actual span render/AX、reachable branch の actual host effect を join する。KLE local highlighter/cache と Markdown/KMM import は AST で拒否する。

### 6. Diagnostics and decorations

- KUC generic gutter/popup/`DiagnosticsPanel` が hover/click/outside close、focus、overlay suppression、display/preview/layout/AccessKit を所有する。KatanA host が official 判定、diagnostics/fix/review/docs/browser data と effect を所有する。
- KLE は opaque diagnostic/file/fix target と revision を forward するだけで、linter、batch、line mapping、popup lifecycle、URL、review payload、preview line calculation を持たない。
- KUC retained panel/scope/disclosure/hover leaves は host action を発行しない。fix/fix-all/docs/select-and-jump は host-projected opaque target の closed requestであり、`Unsupported` は release failure である。

検証:

- official/non-official、開始行/非開始行、同一行複数、fix 有無、panel scope/disclosure/preview を KUC RawInput/AccessKit で、fix/review/docs/select-and-jump を actual KatanA host E2E で個別に検証する。KLE payload/URL/batch/local popup は AST で拒否する。

### 7. Scroll and viewport

- KUC generic text/preview/split viewport が line/offset/range/pixel/resize/visible rows、scroll-past-end、animation、hit target、AccessKit を所有する。
- KatanA host が preview synchronization、logical source/anchor mapping、pending jump、dead-zone、PreviewOnly fallback、document switch を所有する。KLE は opaque viewport/source target と revision/correlation を forwarding するだけで、scroll control trait、line/pixel conversion、visible-range calculation、anchor/cache を持たない。

検証:

- KUC RawInput/AccessKit viewport tests と actual KatanA scroll coordinator E2E を branch ごとに結合する。KLE local scroll state、row/offset/pixel math、fixture-only smoke を AST で拒否する。

### 8. Shortcuts and commands

- KatanA host が command inventory、key binding parse/conflict、Recording > Modal > Editor > Preview > Global priority、availability、modifier specificity、pending action を所有する。
- KUC が physical key/focus/text-entry reservation と command control UI を所有し、KLE は source-derived closed opaque command target を一度 forward するだけである。KLE は shortcut map、conflict resolution、priority、availability、command string/inventoryを実装しない。

検証:

- KUC text-entry reservation/AccessKit と actual KatanA active context/availability/first-match host E2E を leaf ごとに検証する。KLE shortcut parser/serialization/string fallback は AST で拒否する。

### 9. Clipboard and paste

- KUC generic text surface が copy/cut/paste input and platform output UI を所有し、payload-free clipboard correlation を一度だけ KLE に渡す。KLE は document identity、selection、content snapshot、payload、token resolution を保存又は計算しない。
- KatanA host が raw text/image/file-list/file-URL/macOS pasteboard acquisition priority、`file://`/extension parsing、asset/file/Markdown insertion、dirty/explorer/undo/error/no-mutation を所有する。image transport は KLE/KUC payload-free request であり、host external update のみが次の KUC descriptor を与える。
- copy/cut/paste は KUC retained selection/read-only UI と KatanA authoritative document state を尊重する。KLE/KUC は clipboard backend、`arboard`、pasteboard、URL parser、image bytes/list/path を参照してはならない。

検証:

- Japanese/`⭐️` VS16/ZWJ text paste、raw-image/file-list/file-URL priority、non-image URL text route、no payload/error/read-only/stale/document switch/duplicate を KUC RawInput/AccessKit と actual KatanA native host E2E で個別に検証する。
- asset/Markdown/dirty/explorer、selection preservation、no text mutation、clipboard restore を同じ run で証明する。Storybook は raw payload を保存せず hash-only evidence を参照する。ignored live test は完了根拠にしない。

### 10. Theme, i18n, typography, spacing

- KUC root が generic theme/string/locale/typography/spacing/token/render state を所有し、KatanA host が current semantic presentation descriptor を供給する。KLE は descriptor を forwarding するだけで、config/default preset/visible string/color/font/spacing/autosave state を持たない。
- colors は KUC semantic token、visible strings は host-projected localized presentation、typography は KUC `PlatformFontCatalog` を通す。KLE の en/dark/light preset、font lookup、autosave timer は禁止する。

検証:

- KUC root theme/localization/font RawInput/AccessKit tests、host descriptor revision/stale test、KLE default preset/color/string/font/autosave AST rejection。

### 11. Emoji and IME

- OS 依存 emoji / IME 問題は KLE の egui fallback で隠さない。
- 汎用入力 contract は KUC `TextArea` / text entry contract に寄せる。
- KUC の platform text-raster runtime が OS font family を解決し、emoji span を color glyph pixel として出力する。KLE/KDV はその runtime の adapter であり、独自の emoji segmentation、font-family lookup、raster cache を持たない。
- macOS、Windows、Linux の各 release profile は KUC catalog が解決した color
  emoji face/fingerprint を持ち、exact `⭐️` VS16 の単独 chromatic crop が
  same-config `☆` control と異なることを必須とする。KLE/KDV は face 選択、
  substitute、fallback、profile-specific test exemption を持たない。
- emoji は explicit text span/style contract として扱い、grapheme 単位で caret/delete が動く。
- egui compatibility adapter で KUC runtime の表示を接続できない機能は、release target から外すのではなく、実装または明示的な blocker として残す。

検証:

- KUC `TextArea` の IME commit / emoji grapheme tests と同等の KLE-facing tests。
- `just kuc-contract-check` と KUC runtime の `⭐️` / ZWJ / variation selector
  color-pixel、resolved face SHA-256、measure/hit-test contract tests を
  macOS/Windows/Linux の各 release profile で実行する。
- KLE/KDV が同じ KUC runtime を使う downstream compile and behavior tests。
- 実 editor surface 由来の Storybook emoji artifact。fixture renderer は補助診断に限る。

#### KDV/KUC emoji responsibility revalidation (2026-08-21)

KDV を source audit した結果、emoji segmentation、VS16/ZWJ/grapheme preservation、OS font
catalog、color glyph raster、text raster cache は KDV 固有実装ではない。KDV
`emoji_text.rs` は KUC `UiEmojiTextSegments` の adapter であり、Storybook の
`node_factory_text.rs` も KDV `ViewerTextSpan` を KUC `UiTextSpan` と
`UiTextSpanStyle.emoji` へ投影するだけである。KLE も同じ generic KUC text-raster contract
だけを利用し、KDV の Markdown/KMM 型、KDV renderer、emoji segmentation、font lookup、raster
cache を再利用又は複製してはならない。

| responsibility | owner | decisive source | v0.1.0 consequence |
| --- | --- | --- | --- |
| emoji run segmentation and exact VS16 preservation | KUC | `katana-ui-core-text-raster/src/tests/emoji.rs`, KDV `crates/katana-document-viewer/src/emoji_text.rs` | `⭐️` is one authoritative grapheme/run; KLE must not convert it to `☆` or a text fallback |
| neutral emoji span presentation | KUC | `katana-ui-core/src/render_model/typed_text.rs` | KLE/KDV only project their own domain spans to neutral `UiTextSpan` |
| macOS/Windows/Linux face candidate, family, SHA-256, fingerprint resolution | KUC | `katana-ui-core-text-raster/src/catalog_types/{types,resolver}.rs`, `font_candidates.rs`, `catalog.rs` | KLE must consume resolved KUC evidence, not choose a family or add an OS conditional fallback |
| shaping, color raster, cache, grapheme bounds and hit test | KUC | `katana-ui-core-text-raster/src/rasterizer.rs` | KLE Storybook artifact must originate from the same KUC raster path |
| KMM/Markdown interpretation, viewer-only metadata and diagram/image/PDF cache | KDV | KDV `viewer/node_plan/**`, `preview_runtime/**` | outside KLE/KUC text-surface ownership |

KUC has candidate/resolver code for `Apple Color Emoji`, `Segoe UI Emoji`, and `Noto Color
Emoji`, but current source evidence does not prove that all three release environments provision
the required face. KDV CI provisions Graphviz, not emoji fonts. Therefore three-OS completion
still requires each release profile to provision and resolve the actual face, record its SHA-256,
and prove a chromatic isolated `⭐️` crop differs from the same-config `☆` control. Existing KDV
pixel tests and KUC macOS tests are useful component evidence only; they do not establish a KLE
editor call path or a three-OS release result.

### 12. Storybook

- workspace に `tools/kle-storybook` を追加する。
- `just storybook` は interactive window を起動する。
- `just storybook-window-smoke` は headless/smoke で起動と描画を検査する。
- `just storybook-interaction-check` は KUC current root/AccessKit/typed transport/class-appropriate host effect の同一 `step_id` join を検査する。
- `just storybook-emoji-check` は emoji/IME/text area contract を検査する。
- `just storybook-contract-check` は manual hit-test reconstruction、string parse action synthesis、static-only mock を禁止する。
- release readiness では Storybook の live acceptance artifact を生成できる。artifact は interactive window と同じ editor surface/render path 由来である。
- full-spec scenario は日本語、exact `⭐️` VS16、text/IME、toolbar/context menu、find/replace、gutter/diagnostics、tabs/groups、breadcrumb/source address、preview/split を一つの source-derived manifest に持つ。motion media は KUC opaque root final frame 由来であり、host effect を代替しない。

検証:

- source-derived full scenario の KUC RawInput/AccessKit/frame/hash と actual host effect join、acceptance/motion PNG/GIF/MP4 decoded-frame hash、Japanese/emoji grapheme/color-glyph check。
- static callback/counter、shape count、fallback renderer、fixture-only state、manual hit-test、raw input serialization を AST/runtime gate で拒否する。

### 13. KatanA editor full parity

- `docs/v0-1-0-katana-editor-full-parity-audit.md` の全 feature inventory を release blocker として扱う。
- Markdown-specific transformation、file IO、search/range/undo/scroll mapping は KLE core に hard dependency として入れない。KatanA editor feature に必要な action、selection、cursor restore、context menu、image ingest、diagnostic、view/split、tab/group、breadcrumb/source address、preview/split は KUC generic component と opaque KLE forwarding、unchanged KatanA host effect の三者で表現する。
- format/external refresh lifecycle は KatanA host behavior として source closure から individual leaf を生成する。dirty buffer の disk refresh 抑止、active/non-active document、HTML preview refresh delay を actual KatanA host-E2E で区別し、KLE が content/undo/range/refresh state を持たない。
- downstream KatanA integration は tag release 前の必須 evidence とする。

検証:

- KLE unit / integration tests。
- KLE Storybook live contract tests。
- KUC targeted contract tests。
- KLE-owned actual `KatanaApp` host E2E that asserts effects equivalent to the read-only KatanA editor integration tests。

### 14. Preview, split, and KDV/KRR boundary

- `preview` は単一の表示機能ではない。KatanA の `CodeOnly`、`PreviewOnly`、
  horizontal/vertical Split、pane order/resize/persist、Markdown section、document
  frame、HTML browser、scroll sync、hover-to-source、select-and-jump、task toggle、
  fullscreen、slideshow を個別の source-derived leaf とする。
- KDV/KRR は document/browser/render-domain DTO の source of truth であり、KUC は
  KDV に依存しない。KUC は generic `WorkspaceViewport` / `SplitViewport` /
  `PreviewViewport` を所有し、KLE は KatanA-specific descriptor と KDV public DTO
  の非描画 projection、typed host request/result binding のみを持つ。
- KLE は Markdown parser、preview painter、KDV document painter、browser raster、
  anchor/line/scroll/resize geometry、fullscreen/slideshow UI を持たない。KatanA
  egui preview 実装のコピー、fixture bitmap、MVP preview は禁止する。
- KDV DTO で KatanA の可視 branch を表現できない場合は source closure が blocker
  として fail-closed にする。本 v0.1.0 の KLE/KUC scope で KDV を編集して穴埋め
  しない。
- generic split resize/retained layout は `kuc_retained_ui_effect` と current
  KUC root/AccessKit transition と bootstrapped KatanA unchanged observation で
  証明する。KatanA view/document/native browser/fullscreen effect は branch ごとに
  `in_process_host_effect` または `native_external_host_effect` で証明する。

検証:

- `docs/v0-1-0-kuc-preview-viewport-design.md` の projection probe と KUC actual
  RawInput/AccessKit/root-frame tests。
- KLE public input の opaque one-time transit と、unchanged KatanA の
  source-derived physical UI bootstrap/input route を相関させる view/split/
  scroll/task/document/browser/fullscreen host-E2E。`AppAction`/`pending_action`
  の直接注入は evidence ではない。
- KDV public API を通した full Storybook stages。KUC opaque root から生成された
  PNG/GIF/MP4 は review artifact であり、joined effect evidence を代替しない。

## Automation Matrix

| Requirement | Required automated check |
| --- | --- |
| neutral crate has no egui/Floem/KUC runtime leak | `cargo tree -p katana-language-editor` check + AST lint |
| KatanA editor coordinate utilities | source-closure plus KUC actual root/AccessKit and actual KatanA host-E2E; KLE coordinate/range/anchor implementation AST rejection |
| host descriptor DI | host-projected descriptor revision/stale tests and no-default/no-local-preset lint |
| color/theme discipline | `prohibited-color-literal` ast-lint |
| strings discipline | user-visible string lint or explicit review gate until lint is added |
| search and replace | KUC SearchStrip RawInput/AccessKit + actual KatanA search/replace host-E2E; KLE query/match/range/replace AST rejection |
| editor-frame breadcrumb/source navigation | KUC actual `BreadcrumbNavigator` / `SourceAddressBar` RawInput and AccessKit tests + source-derived KLE root stage + actual KatanA `SelectDocument` / `OpenUrl` host E2E; no URL/tree/payload simulator |
| preview/split/KDV projection | KDV public-DTO projection probes + KUC actual root/AccessKit tests + class-appropriate KatanA host E2E |
| scroll sync / navigation | KUC viewport RawInput/AccessKit + source-derived actual KatanA coordinator E2E; no KLE coordinate/simulator oracle |
| dirty/save/external change | actual KatanA buffer/file/undo E2E plus KUC root records; no KLE document-state simulator oracle |
| diagnostics/decorations | KUC actual marker/popup/panel RawInput and AccessKit tests + actual KatanA fix/docs/review/jump E2E; no `Unsupported` |
| problems panel integration | KUC retained panel/scope/preview evidence + actual KatanA visible-fix E2E |
| read-only/editing | KUC input-policy RawInput/AccessKit + actual KatanA no-mutation E2E |
| clipboard | KUC payload-free RawInput correlation + actual KatanA acquisition E2E + stale/no-mutation contract |
| clipboard image payload acquisition | KatanA host backend contract + real host E2E; ignored OS clipboard test is diagnostic only |
| shortcut routing/arbitration | KUC focus/text-entry reservation + KatanA context/availability/modifier-priority actual host E2E; KLE shortcut parser rejection |
| emoji/IME | KUC platform text-raster/root contract + exact `⭐️` VS16 versus `☆` crop + actual KLE/KatanA join + same-surface Storybook artifact |
| Storybook live harness | `just storybook-*` gates and Storybook contract ast-lint |
| Storybook motion evidence | `just storybook-motion-artifact-gate` with deterministic GIF/MP4 decoded hash, KUC opaque-root provenance and matching leaf execution record; no fixture state/counter oracle |
| KatanA editor full parity | `just katana-parity-check` enforces every source-derived leaf has a KatanA source marker, class-correct input-origin evidence (KLE public `RawInput` or KatanA shortcut-router trace), current KUC root/AccessKit evidence, and an executed class-appropriate declared effect; it fails closed on an absent leaf, simulated state, false mounted-causality claim, or release blocker |
| KatanA source inventory completeness | `just katana-parity-check` emits source closure, action origins, branch catalog and per-leaf joins; every fixed-revision editor source branch/action origin is classified fail-closed |
| KLE host-simulator probe | `just katana-downstream-check` is diagnostic only and cannot supply release evidence, counters, state reflection, or source-closure coverage |
| Declared effect harness | KLE-owned `tools/katana-host-e2e` path-depends on the read-only local `katana-ui`; direct-ui host-effect classes join KLE public RawInput/opaque transit to a source-derived physical KatanA UI input route that naturally emits its action/handler/frame, shortcut classes drive the physical router with same-run KUC focus evidence, and retained-UI classes prove KUC state plus an unchanged bootstrapped host observation. `AppAction`/`pending_action` injection, fixed coordinates, or a KatanA adapter module are invalid. |
| release readiness | `just check` + coverage + Storybook acceptance artifacts |

## Current Verification State

### 2026-08-14 verified release blockers

The following commands were rerun after the full-surface acceptance contract
was recorded.  They are failure evidence, not completion evidence.

- `cargo test -p katana-parity-check --locked` does not compile: its partial
  leaf-inventory rejection module imports `leaf_inventory_entries` and
  `leaf_inventory_types` from the wrong module level (`E0432`).  The partial
  schema also uses shared placeholder values for menu path, state condition,
  typed request, and expected effect, which is invalid under the leaf contract
  above even after the import error is repaired.
- The checker's current `REQUIRED_LEAF_COUNT = 48` covers only toolbar,
  context-menu, code-kind, and image-trigger entries.  It omits the separately
  required text/IME, document/history, search, gutter/diagnostics, view/split,
  dirty-refresh, and shortcut leaves.  It also configures the absent KatanA
  `kle_downstream_adapter.rs` as deferred evidence.  A missing reference file,
  a shared selector/harness, or an aggregate authoring case must fail the
  source-universe and leaf-completeness gates; none may be deferred or counted
  as an exact editor leaf.
- `cargo test -p kle-storybook --locked -- --test-threads=1` produced 22 tests:
  12 passed and 10 failed.  Every failure is blocked by
  `storybook contract expected active hovered diagnostic gutter line`; the
  contract configures the gutter but does not run a current public `show` frame
  before reading KUC facts.  Therefore no live-acceptance or motion artifact is
  valid.
- `just ast-lint` failed.  It detects the fallback renderer and `egui` shape
  count acceptance patterns in Storybook, KLE-local context-menu artifact
  composition, and the legacy `platform_text_surface.rs` local text/gutter
  renderer, as well as unresolved file/function-length and literal violations.
  No lint rule, test, coverage exclusion, or suppression may be weakened to
  make this pass.
- The KatanA reference checkout remains clean on `master`; KLE must obtain
  actual host effects through its own harness and must not modify KatanA.

2026-07-11 の設計再分類: 以下の既存 verification は neutral action/host adapter の到達度を示すだけであり、KUC TextSurface / CommandChrome を用いる actual UI parity の完了証拠ではない。`EguiLanguageEditor::show` は KUC `TextSurface` binding を直接呼び、legacy `PlatformTextSurface` は compiled module graph から外れている。しかし legacy source、`LineGutterModel`、`authoring_helper.rs`、Storybook fallback が物理的に残るため AST gate は失敗し、全 editor UI row は release-ready ではない。syntax highlighting、normal text clipboard、shortcut/undo/redo、native IME/ZWJ selection、AccessKit、find/replace UI、unregistered rendering tests は個別 blocker として扱う。

- Neutral contract と KLE-only checks には save/format/ingest/authoring/diagnostic/view/search/document の typed action、cursor restore、gutter、AccessKit、IME/ZWJ の部分的な到達度がある。ただし、これらは KatanA 実 host effect の証拠ではない。
- KUC ContextMenu overflow は 480x220 の KUC actual RawInput と KLE host E2E で、Save / conditional Format / Edit / Ingest、14 authoring leaves、17 nested code kinds、visible ingest leaves を三経路で通過した。この結果は ContextMenu 汎用 runtime の証拠に限定する。
- `katana-parity-check` は source-derived leaf inventory と、KLE public RawInput、KLE-owned actual KatanA host-E2E の三点証拠の欠落を blocker として検出する。KLE-only test、source marker、Storybook、simulated host state はいずれも単独では success へ格上げしない。
- KatanA repository は本作業の read-only reference である。host-effect evidence は KLE の test harness から取得し、KatanA source を変更して帳尻を合わせない。
- release blocker は、KUC TextSurface/CommandChrome の KLE thin binding 完成、legacy renderer の compiled graph からの離脱、KUC compositor だけを使う full-spec Storybook/motion artifact、`⭐️` VS16 の same-surface color glyph evidence、全 leaf の KLE actual input と KatanA actual host effect、AST/lint/coverage/strict gates の全通過である。
