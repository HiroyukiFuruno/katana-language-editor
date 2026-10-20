# v0.1.0 Context Menu Native Target Design Audit

## 監査対象と結論

監査対象は、固定KatanA source closure の revision
`4f6a6287c650a38633c7baeb544a92e739c68567`、既存の
`v0-1-0-context-menu-host-e2e-design.md`、
`v0-1-0-external-host-e2e-design.md`、source-derived native target 実装、
および固定KatanAのeditor/context/accessibility関連ソースである。

結論は次のとおり。

1. 固定ソースはeditor本文を`egui::TextEdit`として生成し、固定ソース内の
   `egui_kittest`では`Role::MultilineTextInput`として観測できる。
2. 固定ソースはContext Menuのleafを`egui::Button`として生成する。従って、
   menuが開いたフレームのegui/AccessKit treeにbutton nodeが現れる可能性は
   ソース上支持される。
3. しかし、現行のsource-derived native target artifactはExplorerの
   `Open Workspace`ボタンだけを対象にしており、editor本文やContext Menuの
   leafを表現しない。現行locatorもrole/name digestだけで、親menu、leafの
   source span、frame世代、route、再実行防止を表現しない。
4. 固定KatanAソースから、macOS AXでeditor本文とContext Menu各leafが実行時に
   同一フレームの物理targetとして現れることを、まだ証明できていない。
   `egui_kittest`のsemantic tree検証をmacOS AXの証拠に読み替えてはならない。
5. AccessKit action requestのegui-winit側配送経路は存在するが、固定KatanA側に
   Context Menu用の公開target contract、source-derived leaf identifier、または
   effect evidenceとのjoin点はない。この差をKLE/KUCで補うことは禁止する。

したがって、現時点ではContext Menu native targetingは**未成立**であり、
v0.1.0の実ホストE2Eに対する明示的なrelease blockerである。以下の設計は、
このblockerを解消するための設計であり、実装完了や実行成功の報告ではない。

## 固定ソースから確認できる事実

### Editor本文

固定KatanAは`TextEditRenderer::render`で`egui::TextEdit::multiline(buffer)`を
生成し、`interactive(editable)`、monospace font、無限幅、初期表示行数を設定して
いる（`/tmp/katana-fixed-source-closure-20260821/crates/katana-ui/src/views/panels/editor/text_edit.rs:51-63`）。
同じresponseをContext Menuの起点に渡している（同:66-75）。これは本文とmenuの
起点が同一のUI responseであることを示すが、native AX nodeの安定したIDを示すもの
ではない。

固定ソースのintegration testは、editorを`Role::MultilineTextInput`で検索し、
値を条件に選択している（`/tmp/katana-fixed-source-closure-20260821/crates/katana-ui/tests/integration/editor/ui.rs:7-38`）。
これはin-process `egui_kittest` treeの証拠であり、macOS AX clientから取得した
window treeの証拠ではない。本文のcurrent-frame AX target成立には、実KatanA native
windowからのAX snapshotが必要である。

### Context Menuと各leaf

Context Menu本体は`response.context_menu`で開かれる（`/tmp/katana-fixed-source-closure-20260821/crates/katana-ui/src/views/panels/editor/context_menu.rs:16-18`）。
Saveはbuttonとして生成される（同:19-22）。Formatはeditableかつ拡張子が`.md`/
`.markdown`の場合だけ生成される（同:23-30, 184-192）。Authoring submenuと
ingest submenuは別のmenuとして生成される（同:31-48）。

inline、heading、block、link、tableのleafは`add_enabled`/`Button`で生成される
（同:56-85, 94-170, 172-181）。Code Blockの17種類は`CodeBlockKind::all()`を
走査して各kindのdisplay labelをbuttonにする（`/tmp/katana-fixed-source-closure-20260821/crates/katana-ui/src/views/panels/editor/code_block_menu.rs:20-29`）。
Image FileとClipboard Imageもbuttonで、Clipboard Imageはpayloadの有無でdisabled
になる（`/tmp/katana-fixed-source-closure-20260821/crates/katana-ui/src/views/panels/editor/context_menu_image_ingest.rs:7-29`）。

このソースから確実に導けるのは「該当フレームでbutton widgetを生成する」ことまで
である。各leafにmacOS AXで取得可能な一意のlabel、親menuからの構造path、安定した
node identity、AccessKit invokeを受けてKatanAのaction/effectへ到達することは、
このソース断片だけでは証明されない。

### Accessibilityの配送境界

固定vendorのegui-winitにはAccessKit adapterを初期化するAPIがあり、
`ActionRequest`を`egui::Event::AccessKitActionRequest`へ追加するAPIもある
（`/tmp/katana-fixed-source-closure-20260821/vendor/egui-winit/src/lib.rs:177-191, 772-780`）。
frame処理時にはAccessKit updateをadapterへ渡す（同:1151-1157）。

一方、固定KatanAのmainは`eframe::run_native`を起動し、`KatanaApp::new(state)`を
生成するだけである（`/tmp/katana-fixed-source-closure-20260821/crates/katana-ui/src/main.rs:35-40, 68-85, 102-125`）。
`KatanaApp::new`は`AppState`だけを受け取り、documentやContext Menu targetを
受け取らない（`/tmp/katana-fixed-source-closure-20260821/crates/katana-ui/src/shell/mod.rs:38-110`）。
固定KatanA側にAccessKit action requestをContext Menu leafのsource-derived
contractとして受け取る公開APIは見当たらない。`trigger_action`はpending actionを
直接設定するAPIであり、物理入力の証拠には使えない（同:129-132）。

## 現行artifact/locatorとの乖離

source generatorは現在、Explorerの固定source span
`crates/katana-ui/src/views/panels/explorer/empty.rs:52-60`だけを検査し、
`egui::Button::new`、`menu.open_workspace`、`pick_open_workspace`を要求する
（`tools/katana-parity-check/src/source_closure/native_target.rs:10-20, 39-54`）。
生成recordはschema、generator、revision、profile、source span digest、role digest、
locale name digestだけである（同:69-80、および
`tools/katana-host-e2e/src/source_derived_native_target/types.rs:7-17`）。

loaderはrevision、digest形式、locale digestの並び、source worktreeのclean状態を
検証するが、Context Menuの親子構造やleaf inventoryを検証しない
（`tools/katana-host-e2e/src/source_derived_native_target/loader.rs:13-50, 54-92, 120-169`）。
locatorはcurrent AccessKit updateの全nodeからrole/name digestを検索し、複数件、欠落
label、disabled、bounds欠落を拒否する（`tools/katana-host-e2e/src/host_target_locator.rs:7-72, 85-116`）。
これはgenericな最終node選択の部品としては再利用可能だが、Context Menuの完全な
target artifactではない。

またframe captureはAccessKit update、root、frame hashを取得し、任意のlocatorで
current targetを選ぶ（`tools/katana-host-e2e/src/physical_frame_capture.rs:59-128`）。
しかし、現在の実行sourceには、menuを開く前の本文target、開いた後のleaf target、
同一runのrouteとeffectを結合する世代契約がない。

## 生成可能な値と禁止値

### 固定ソースから生成してよい値

- 固定revision、clean/detached状態、source closure/profile fingerprint。
- source span digest。本文、Context Menu、submenu、各leafの構造的位置を
  source pathとspanから導出する。
- AccessKitのsemantic role期待値。本文は固定sourceの`TextEdit`に対応する
  `MultilineTextInput`、button leafは`Button`という期待値を、実frameで確認する。
- i18n keyと全bundled localeから解決したlabelのSHA-256 digest集合。現行Explorer
  generatorと同じくraw labelをartifactに出さない。
- 構造path digest。例として`editor.text_surface/context_menu/authoring/code_block/rust`
  のような意味を持つ固定sourceの親子pathを、raw action enumではなくsource
  inventoryの識別子としてdigest化する。
- enabled/disabledのsource条件。Formatの拡張子・editable条件、Clipboard Imageの
  payload条件など、固定ソースの分岐をmanifestの条件として記録する。

### artifact/runtimeへ出してはならない値

- document content、selection range、cursor位置、入力文字列、clipboard payload。
- screen/window coordinates、固定クリック座標、egui::Id、内部node idを事前値として
  注入すること。boundsは対象をcurrent frameで再解決した結果だけ一時的に使う。
- KLEのsemantic action map、`EditorAction`/`AppAction`、KatanA action enum、
  `pending_action`、KLE callback名。
- raw locale label。比較はsource-derived digestで行い、実行証跡にもraw textを残さない。
- source recordを使わない手動label、環境変数、test argument、fixture由来のtarget。

## 必要なgeneric host-target artifact

Context Menu用recordは現行schemaを置き換えるのではなく、別schemaとして追加する。
最低限の論理モデルは次のとおり。

```text
ContextMenuTargetManifest {
  schema_version,
  generated_by,
  katana_revision,
  profile_fingerprint,
  source_closure_fingerprint,
  surface: { source_span_digest, role },
  menu: { source_span_digest, role, path_digest },
  leaves: [{ leaf_path_digest, source_span_digest, role,
             locale_label_digests, enabled_condition_digest,
             parent_path_digest }],
  routes: [secondary_pointer, shift_f10, accesskit_invoke]
}
```

record validationはunknown field拒否、fixed revision/profile一致、source spanの
再計算一致、locale digestのsorted/distinct、leaf pathの重複拒否を必須にする。
raw labelやraw document dataはschema上持たせない。

### current-frame locator

1. 前フレームでeditor本文をrole、label digestまたはsource-derived structural
   relationで一意に解決する。候補0件、複数件、disabled、bounds欠落は失敗。
2. そのcurrent frameの対象に対して、指定routeを一度だけ発行する。
3. 次フレームのAccessKit/AX snapshotを取得し、menu rootとleafを親子関係、role、
   locale digest、source-derived pathで再解決する。前フレームのnode idやboundsは
   再利用しない。
4. leafのdynamic boundsはこのsnapshotからのみ取得し、実行直前に対象が同一frame
   generationであることを確認する。frame hash、AX tree digest、target path digest、
   route、one-shot tokenをrecordする。
5. stale generation、replay token、offscreen/zero/非有限bounds、disabled leaf、
   label mismatch、親menu不在、同一pathの複数候補はfail-closedとし、host effectを
   実行しない。

localeはraw labelを比較せず、固定sourceの全bundled localeから生成したdigest集合
   に対して一致させる。実行localeが集合外、locale間で同一digestが同一scopeに複数
   現れる場合、labelだけではtargetを確定できないため、構造pathと親nodeで絞れなければ
   拒否する。sourceから構造pathを得られないleafは、label digestだけで無理に解決しない。

## 3 routeの観測契約

各leafについて、次の3 routeを別executionとして実行する。

- secondary pointer: 前フレームで本文targetを解決し、そのdynamic bounds内へOSの
  secondary press/releaseを配送する。固定座標は禁止。
- Shift+F10: 前フレームで本文targetがfocusされていることを確認し、OS key eventを
  配送する。KLEがRawInputを合成してはならない。
- AccessKit invoke: AX/AccessKit snapshotからcurrent targetを解決し、OS accessibility
  invokeを発行する。固定KatanAにこのinvokeをmenu opening/leaf activationとして
  観測可能な公開経路が存在しない場合、このrouteは成立しない。

各routeの証跡は、pre-frame hash、本文target、input kind、post-open frame hash、
menu/leaf target、activation frame、close transition、one-shot/replay結果を持つ。
route間で一致させるのはsource leaf path、fixed revision、effect class、証跡digestで
あり、KatanA action enumや座標ではない。

## effect evidence join

target evidenceとeffect evidenceは、KLE/KUCのcallback名で結合しない。join keyは
`execution_id`、fixed revision、source leaf path digest、route、pre/post frame digest、
one-shot token digestに限定する。

leaf activation後は、固定KatanA自身が自然に生成したaction/handler/effectを、host側の
観測ポートで検証する。authoringはUTF-8 buffer、selection/cursor、dirty、undo状態、
save/formatはfilesystemとdirty transition、ingestはnative dialog/clipboardからの
最終file/Markdown effectを検証する。pending actionやcallback receiptだけではeffect
evidenceにならない。KLE/KUC側はopaque transportが一度だけforwardされたことと、
current-frame target recordの整合性だけを証明する。

## 明示的release blocker

次の不足は、KLE/KUCの実装で埋めてはならない。

1. 固定KatanA native windowのmacOS AX snapshotで、editor本文が実行時に
   `MultilineTextInput`として取得できることの証拠がない。固定sourceのkittest検証は
   代替にならない。
2. Context Menuを実際に開いた直後のAX treeで、root、submenu、全leafを親子構造と
   locale digestで一意に取得できることの証拠がない。
3. fixed KatanAのAccessKit/AX invokeが、本文Context Menuのopenおよびleaf activation
   に到達することのsource-backed host証拠がない。egui-winitの配送APIだけでは不足。
4. 3 routeそれぞれについて、current-frame targetから実ホストのsave/format/authoring/
   ingest effectまで同一executionでjoinできる実行証拠がない。

これらが未解決の間、`katana-host-e2e-context-menu`をgreen扱いにせず、Context Menu
native targetingを完成扱いにしない。固定sourceにrouteやtarget contractを追加する
必要が判明しても、本監査の範囲でKatanAを変更せず、別途固定source変更として設計・承認
する。

## 実装計画と所有境界

### source-closure generator / parity-check

1. 固定sourceのeditor TextEdit、Context Menu、submenu、17 code-kind、ingest leafを
   AST/source spanから収集する。
2. i18n keyを全localeへ展開し、raw labelを保存せずdigest集合を生成する。
3. structural path、role、enabled condition、source/profile/revision fingerprintを
   ContextMenuTargetManifestへ出力する。
4. fixed source spanの変更、leaf欠落、重複path、locale digestの不整合、未定義routeを
   parity-checkで拒否する。

### generic host E2E driver

1. clean fixed sourceをrevision-pinnedで起動し、macOS AX権限と対象windowをfail-closed
   で確認する。
2. current AX/AccessKit snapshotから本文、menu、leafを毎回再解決し、dynamic boundsと
   frame generationを管理する。
3. secondary、Shift+F10、AccessKit invokeをOS入力として記録し、replay/stale/
   ambiguous/disabledを拒否する。
4. 固定KatanAの自然なeffectを観測し、source leaf digestとexecution evidenceをjoinする。
   action injection、AppState mutation、fixed coordinate、sleep、KLE semantic mapは
   実装しない。

### KUC / KLE

KUCに入れてよいのは、複数hostで再利用できるgenericなcurrent-frame opaque target
   record、AccessKit/AX evidence、freshness、one-shot lease、dynamic bounds、
   ambiguity rejection、generic input-route receiptだけである。KLEはそのrecordを
   不透明にcompose/forwardするだけとし、KatanAのContext Menu名、Markdown operation、
   document state、native dialog/clipboard処理、座標・action mappingを持たない。

KUC/KLEのStorybookやin-process testはgeneric contractの回帰検証には使えるが、固定
KatanAのmacOS AX target成立、3 route、実host effectの代替証拠にはしない。
