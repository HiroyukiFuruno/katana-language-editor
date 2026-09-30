# KLE v0.1.0 Editor Interaction Source Audit

## 監査条件

- 参照対象: `/tmp/katana-fixed-source-closure-20260821`
- 固定revision: `4f6a6287c650a38633c7baeb544a92e739c68567`
- KatanAはread-onlyで参照した。KatanA/KUC/KLE product/tools/workflowはこの監査では変更していない。
- sourceの存在は、対応済みの証拠ではない。各行のrequired evidenceを満たすまで、KLE v0.1.0の完了とは扱わない。
- 行番号は上記固定sourceに対する `nl -ba` の行番号である。

## 境界判定

| 責務 | owner | KLE v0.1.0での扱い |
|---|---|---|
| TextEdit、selection、cursor、focus、line gutter、scroll、popup、context menuの汎用UI | KUC | KUCのgeneric componentを組み合わせる。KLEにegui相当の再実装を置かない。 |
| font/raster、CJK、VS16 emoji、IME、clipboard、AccessKit semantic input | KUC | KUC root/platform runtimeで保証する。KLEは文字列変換・font fallback・emoji置換を実装しない。 |
| KatanA固有のDocument/SearchState/AppAction/Markdown変換/file IO | KatanA host | KLEはopaque targetを一度だけtransitし、hostが既存handlerへ解決する。KLEにKatanA固有actionやbuffer semanticsを持たせない。 |
| toolbar/context menuの配置とKatanA固有の操作一覧 | KLE composition + KUC | KLEはKUC部品をKatanA source-derived catalogに従って構成する。処理本体はhost/KUCのowner境界を越えない。 |
| Storybook | KLE harness + KUC live root | 静的ギャラリー、shape-count、fallback/minifbは不可。実コンポーネントのlive input/effectを表示し、host effectは別証拠として結合する。 |

## Document find

| 機能/分岐 | source file:line | UI trigger (pointer/keyboard/AccessKit) | action/handler route | state precondition | success/error/disabled/close/focus | owner | required real-host effect | required strict automated evidence |
|---|---|---|---|---|---|---|---|---|
| 文書検索を開く | `crates/katana-ui/src/app/action/dispatch_secondary.rs:82-87` | command palette/shortcutからaction。source上のwidget固有pointerは別実装。AccessKit経路はsourceに独立定義なし。 | `AppAction::OpenDocSearch` -> `doc_search_open=true`、`search_newly_opened=true` | hostが起動済み。document search stateが利用可能。 | 次frameで入力へfocus要求。 | KatanA host state、KUC input | 実KatanAのeditor上部に検索barが出て、入力へfocusする。 | fixed KatanAをreal hostで起動し、opaque open request、AccessKit tree、focus owner、visible search barを同一correlationで記録。source-only action testは不可。 |
| 文書検索のtoggle/close | `crates/katana-ui/src/app/action/dispatch_secondary.rs:89-101`、`views/top_bar/search_logic.rs:43-45` | toggle action、Escape。Close iconは`views/top_bar/search.rs:41-52`のpointer。AccessKit semantic buttonはreal hostで確認する。 | toggle -> open/clear matches、Escape -> `doc_search_open=false` | open時はquery/matchesを持ち得る。 | toggle close時にmatchをclear。Escapeはinputがlost focusの分岐だけでclose。 | KatanA host + KUC button/input | close後にbarが消え、editor focusが不意に別widgetへ移らない。 | pointer、Escape、AccessKit activateの3入力をreal hostで個別検証。close state、match clear、focus restoreをassert。 |
| query変更 | `views/top_bar/search.rs:58-65`、`views/top_bar/search_logic.rs:39-41`、`app/action/dispatch_secondary.rs:102` | text input、IME commit、AccessKit value change。 | `response.changed()` -> `DocSearchQueryChanged` -> `handle_action_doc_search_changed`（dispatch route） | active documentがあること。empty queryはmatchなし。 | query変更後にmatchesとactive indexを更新。 | KUC text input + KatanA search state | 実editorの検索hit decorationとcountが同一frame系列で更新。 | 日本語/VS16を含むquery、empty、no-result、IME commitをreal hostで検証。query・ranges・count・paint/semantic outputをjoin。 |
| 検索対象とmatch計算 | `app/doc_search.rs:24-78` | UI query changeが入口。 | `DocSearchOps::compute_matches`。Markdown parserの`Event::Text`/`Event::Code`だけを対象、HTML等は除外。 | query non-empty。regex構築失敗は空matches。 | case-insensitive、sorted/unique character ranges。 | KatanA host logic | markdown text/inline code/fenced codeがhitし、URL/HTML attribute等はhitしない。 | fixture unit testに加えreal buffer/real hostでUTF-8 character rangeとdecoration位置を検証。 |
| next/previous | `views/top_bar/search.rs:68-98`、`views/top_bar/search_logic.rs:27-37` | pointer arrow buttons、Enter、Shift+Enter、focused ArrowUp/ArrowDown、AccessKit button activate。 | `DocSearchNext/Prev` -> `handle_action_doc_search_next/prev(ctx)` | matches non-emptyでbutton enabled。emptyならdisabled。 | wrap direction、active index、editor scroll target。 | KUC controls + KatanA navigation | real editor viewportが対象match行へ移動し、zero-matchでは移動しない。 | pointer/keyboard/AX各々、first/last wrap、zero match、editor/preview scrollを個別E2E。fixed coordinateやsleepは禁止。 |
| 文書findとworkspace Markdown searchの区別 | `state/search.rs:42-47`、`views/modals/search.rs:48-112`、`views/modals/search_tabs/md_tab.rs:13-154` | Command+F系のdocument barとSearch modalのpointer/keyboard/AX tab操作。 | document searchは`doc_search_*`、workspace searchは`WorkspaceSearchOps::search_workspace` -> `SelectDocumentAndJump`。 | workspace searchはworkspaceがない場合results clear。query emptyもresults clear。 | no-result表示、history、result clickからdocument選択/jump。 | KUC search controls + KatanA host search | workspace result clickが実documentを開き、line/byte rangeへjumpする。 | document/workspaceの混同がないことをreal hostで検証。result path/lineはhost-derived opaque targetとして扱い、KLEが検索を再実装しないことをASTで検査。 |

## Document replace / replace-all

| 機能/分岐 | source file:line | UI trigger (pointer/keyboard/AccessKit) | action/handler route | state precondition | success/error/disabled/close/focus | owner | required real-host effect | required strict automated evidence |
|---|---|---|---|---|---|---|---|---|
| 一般文書Replace | 固定sourceの該当editor/search routeにUI/actionは存在しない。`app_action_types.rs:41-44`の`ReplaceText`はaction型として存在するが、`app/document_edit.rs:25-79`はlint/review等から渡された単一byte spanを置換する処理。 | 一般検索bar、検索modal、context menu、keyboard、AccessKitのreplace triggerはsource上で確認できない。 | `AppAction::ReplaceText { span, replacement }` -> `DocumentEditOps::handle_replace_text`。検索UIからのrouteではない。 | active document indexが必要。 | changed時のみundo record、preview refresh、search match refresh、diagnostic timestamp。active docなし/無変更はreturn。 | KatanA hostの既存lint/edit route。KLE/KUCはReplace semanticsを所有しない。 | KLEが一般replace UIを主張するには、KatanA hostでsourceに存在する別routeの証拠、または要件として新規host機能を明示しない限り不可。 | source auditがReplace UI absentを検出し、KLE release gateが未実装をfail。`ReplaceText`を一般replaceの完了証拠に使用しないnegative test。 |
| Replace All | 固定sourceに`ReplaceAll`/`replace_all`のeditor action・UI routeはない。検索結果を全件置換するhandlerも確認できない。 | pointer/keyboard/AccessKit triggerなし。 | routeなし。`DocSearchOps`はmatches計算のみ。 | 該当なし。 | 該当なし。 | 未定義。KLEで独自実装してはならない。必要ならKatanA host仕様の追加判断が先。 | v0.1.0の完全互換対象としては未達。KLE側にlocal replace-allを置かず、host contract/仕様判断が完了するまでrelease blocker。 | fixed source inventoryのabsence assertion、KLE action/APIにreplace-allがないことのAST assertion、real-host requirement未解決をrelease gateでfail。 |

## Markdown authoring toolbar / floating toolbar

| 機能/分岐 | source file:line | UI trigger (pointer/keyboard/AccessKit) | action/handler route | state precondition | success/error/disabled/close/focus | owner | required real-host effect | required strict automated evidence |
|---|---|---|---|---|---|---|---|---|
| 常設toolbarのinline操作 | `views/panels/editor/toolbar.rs:23-39,54-82` | pointer、keyboard navigation、button semantic。Bold/Italic/Strike/InlineCodeはselectionなしでdisabled。 | `AppAction::AuthorMarkdown(Bold/Italic/Strikethrough/InlineCode)` | editable Markdown editor、selection required for inline ops。 | click emits action。実変換後cursor/selectionを保持。 | KUC toolbar/button + KatanA host authoring | real buffer、dirty、preview、cursorが更新。 | 各4操作をselectionあり/なし、Unicode、read-onlyでreal host E2E。KUC RawInput/AX -> opaque transit -> host bufferをjoin。 |
| heading/list/blockquote | `toolbar.rs:84-134` | pointer/keyboard/AX button activate。 | `AuthorMarkdown(Heading1..3/BulletList/NumberedList/Blockquote)` | editable。heading/listはselectionなしでもenabled。 | line prefix、複数行のlist、cursor位置。 | KUC controls + KatanA transform | active line/selectionのMarkdownだけが変わり、undo/dirty/previewが連動。 | 5操作×cursor/selection/blank line/Unicodeのreal buffer assertion。 |
| code block dropdown | `toolbar.rs:136-148`、`code_block_menu.rs:20-29,40-81` | pointer button、keyboard/AX menu navigation。button downでもopen。outside clickでclose。 | `AuthorMarkdown(CodeBlock(kind))`、選択後`ui.close()`。 | editable/cursor。17 `CodeBlockKind`を列挙。 | kind選択でmenu close。outside clickでclose。popup stateはmemoryに保持。 | KUC popup/menu + KatanA host transform | 17言語それぞれのfenceとcursorが実bufferに反映。 | 17 kind全件、open/select/close、outside/Escape、focus、read-onlyをreal hostで検証。候補列挙だけは不可。 |
| horizontal rule/link/table/image | `toolbar.rs:140-148`、`context_menu.rs:154-181`、`markdown_authoring_op.rs:61-75` | toolbar image pointer/AX、context menu pointer/keyboard/AX。 | `HorizontalRule`, `InsertLink`, `InsertTable`、imageは`IngestImageFile`。 | link/tableはcursor/selection。imageはactive saved Markdown等の条件。 | link/tableはsnippetまたはselection wrap。imageはfile dialog route。 | KUC controls; transform/file IOはKatanA host | buffer/dirty/preview、imageならasset fileとExplorer refresh。 | exact buffer/cursor/undo、real file IO、dialog cancel/errorをE2E。 |
| selection floating toolbar | `toolbar_popup.rs:15-81,102-114`、`text_edit.rs:51-85,154-185` | editor focus + selection/cursor、pointer toolbar操作、keyboard/AX button。diagnostic hover時は抑制。 | popup内の常設`EditorToolbar` -> 上記AuthorMarkdown action。 | editable、cursorあり、focusまたはpopup既開、suppress=false。 | editor click/outsideでclose、code menu open時相互排他、viewport edge clamp。 | KUC popup/layout/interaction facade + KatanA host op | selectionが変わる/失われる際のpopup visibility、focus、cursor restoreを実hostで確認。 | selection create/lost focus/outside click/viewport clamp/code dropdown close/diagnostic suppressionをreal hostで個別assert。fixed coordinate/sleep不可。 |

## Context menu / code block menu / image ingest

| 機能/分岐 | source file:line | UI trigger (pointer/keyboard/AccessKit) | action/handler route | state precondition | success/error/disabled/close/focus | owner | required real-host effect | required strict automated evidence |
|---|---|---|---|---|---|---|---|---|
| editor context menu | `context_menu.rs:9-54`、`text_edit.rs:69-75` | TextEdit context-menu pointer/keyboard/AX menu open。 | Save、FormatMarkdownFile、authoring subgroup、ingest subgroup。各選択は`ui.close()`。 | formatはeditableかつ`.md`/`.markdown`。inline opsはselection必須、structure/insertはenabled。 | action後menu close。read-onlyではformat/inline disabled。 | KUC menu + KatanA host actions | real host save/format/authoring/image effect。 | menu open via pointer/keyboard/AX、disabled visibility、close/focus、各leafのhost effectを検証。 |
| image file ingest | `context_menu_image_ingest.rs:7-22`、`app/action/image_ingest.rs:5-35` | menu pointer/keyboard/AX。file pickerはnative host。 | `IngestImageFile` -> file dialog -> read bytes -> `process_image_ingest`。cancelはno-op、dialog unavailableはpending dialog retry。 | editable active saved Markdown、supported image file。 | read/write/create-dir failureはreturn/error logging。成功時markdown image tag挿入、`RefreshExplorer`。 | KUC menu/opaque file intent + KatanA file IO | asset file作成、relative link挿入、dirty、Explorer refresh。 | real temporary file + native dialog/host bridge、success/cancel/unavailable/read-only/unsaved/errorを個別検証。 |
| clipboard image/file URL ingest | `views/panels/editor/paste.rs:14-66,89-143`、`app/action/clipboard_image.rs:14-62` | focused editorでpaste key/Event::Paste。arboard image/file list/file URL/macOS pasteboardを順に読む。 | paste eventをinterceptし、`IngestClipboardImage` -> `read_image_payload` -> `process_image_ingest`。ordinary text pasteは通す。 | editor focus、clipboard payloadがsupported image。 | unsupported/no payloadは通常pasteまたはno-op。payload read errorはstatus error。intercept時はtext eventを除去。 | KUC clipboard/input boundary + KatanA ingest/file IO | image bytes -> asset/link/dirty/Explorer refresh、ordinary textは変化なし。 | real host clipboard contractをbackend別に検証。raw image/file list/file URL/payload-none/error/read-only、Japanese/VS16 text coexistenceをmock-onlyでなくhost E2Eへ分離。 |
| transform semantics | `markdown_authoring_op.rs:76-96`、`views/panels/editor/authoring.rs:14-88`、`authoring_utils.rs:5-195` | toolbar/context/command actionからのみ。 | `handle_action_author_markdown` -> active doc guard -> cursor range -> `MarkdownAuthoringOps::apply` -> `handle_update_buffer` -> pending cursor。 | active doc、not reference。cursor absent時はbuffer末尾。selectionはbyte boundaryへ変換。 | reference/active docなしはno-op。transform後cursorを次frameへ復元。 | KatanA host transform。KLE/KUCはsemantic transformを持たない。 | exact resulting buffer, dirty/preview/undo, cursor range。 | 14 operations（inline 4、block/structure 8、reference 2）と17 code kinds、no-selection/selection/reference/UTF-8をreal hostで全件検証。 |

## Current KLE gap and release interpretation

- KLEの現状は、固定KatanAの上記route全体をreal hostへ接続した証拠になっていない。KUC root/library unit test、source marker、Storybook表示だけでは代替できない。
- Document Replace/Replace Allは固定KatanA sourceに一般UI routeがない。`ReplaceText`の存在だけで実装済みと判定しない。KLEで独自実装するのもowner境界違反である。
- Markdown toolbarの全操作、floating toolbarのfocus/close/clamp、context menu、code block全17種、image ingestのnative/file/clipboard分岐は、実KatanA host effectまでのleaf evidenceが必要である。
- 日本語入力、IME、exact `⭐️`（U+2B50 U+FE0F）、ZWJ、CJKの表示・selection・cursorはKUC platform text/raster責務であり、KLEで補正しない。real hostでcode point sequenceと表示状態を別々に検証する。
- AccessKitはsourceに明示されたsemantic情報とreal hostの実際のtreeを分けて扱う。`widget_info`やbutton labelの存在は、activate後のhost effectの証明ではない。

## 参照したKatanA source一覧

- `crates/katana-ui/src/views/top_bar/search.rs`
- `crates/katana-ui/src/views/top_bar/search_logic.rs`
- `crates/katana-ui/src/widgets/search_bar.rs`
- `crates/katana-ui/src/state/search.rs`
- `crates/katana-ui/src/app/doc_search.rs`
- `crates/katana-ui/src/views/modals/search.rs`
- `crates/katana-ui/src/views/modals/search_tabs/md_tab.rs`
- `crates/katana-ui/src/views/modals/search_tabs/filename_tab.rs`
- `crates/katana-ui/src/app/action/dispatch.rs`
- `crates/katana-ui/src/app/action/dispatch_secondary.rs`
- `crates/katana-ui/src/app/action_types.rs`
- `crates/katana-ui/src/app/document_edit.rs`
- `crates/katana-ui/src/views/panels/editor/ui.rs`
- `crates/katana-ui/src/views/panels/editor/text_edit.rs`
- `crates/katana-ui/src/views/panels/editor/line_numbers.rs`
- `crates/katana-ui/src/views/panels/editor/toolbar.rs`
- `crates/katana-ui/src/views/panels/editor/toolbar_popup.rs`
- `crates/katana-ui/src/views/panels/editor/context_menu.rs`
- `crates/katana-ui/src/views/panels/editor/context_menu_image_ingest.rs`
- `crates/katana-ui/src/views/panels/editor/code_block_menu.rs`
- `crates/katana-ui/src/views/panels/editor/paste.rs`
- `crates/katana-ui/src/views/panels/editor/authoring.rs`
- `crates/katana-ui/src/views/panels/editor/authoring_utils.rs`
- `crates/katana-ui/src/markdown_authoring_op.rs`
- `crates/katana-ui/src/app/action/process_authoring.rs`
- `crates/katana-ui/src/app/action/image_ingest.rs`
- `crates/katana-ui/src/app/action/clipboard_image.rs`

