# KLE Storybook full-spec KUC-root live harness 設計

## 目的と判定

KLE Storybookを、MVP/legacy editor/counter/fallbackの確認器から、KUC generic rootを実際に消費するlive harnessへ置き換える。Storybook単体の表示、source marker、callback、counter、fixture、画像・動画は完了証拠ではない。各source-derived leafについて、KUC root、AccessKit、KLE transit、必要なKatanA host effectを同一execution correlationで結合できた場合だけ受入とする。

参照基準は固定KatanA source audit（`docs/v0-1-0-editor-interaction-source-audit.md:3-9`、revision `4f6a6287c650a38633c7baeb544a92e739c68567`）、KUC/KLE root監査（`docs/v0-1-0-kuc-kle-root-consumption-audit.md:12-82`）、host-E2E設計（`docs/v0-1-0-external-host-e2e-design.md:34-49,521-548`）、OpenSpec tasks `10.4-10.9a, 12.4ao-12.4ar` とする。

## 現状の失敗事実

| 事実 | 根拠 | 設計上の判定 |
|---|---|---|
| `main` は `require_injected_host_projection()` を呼ぶが、同じ起動経路でproviderを注入しない | `tools/kle-storybook/src/main.rs:35-39`; `docs/v0-1-0-kuc-kle-root-consumption-audit.md:33-36` | provider注入前は常に `MissingHostProjection`。live harnessではない |
| provider付きrootのbinding自体は `synchronize -> KUC show -> forward once` を持つ | `tools/kle-storybook/src/root_runtime.rs:54-88`; `docs/v0-1-0-kuc-kle-root-consumption-audit.md:30-32` | binding存在をmain editor/host接続済みとは扱わない |
| Storybook forwarderは `forwarded_batches` を増やして `Ok(())` を返すだけ | `tools/kle-storybook/src/root_runtime.rs:40-43,91-100`; 同監査 `:34,58-71` | counter-only。host effectの証拠にならない |
| Storybook hostは `EguiLanguageEditor`を保持し、fixtureをseedするが、projectionは `None` で初期化する。legacy `EguiLanguageEditor`をKUC rootとしてmainが実表示する経路はcurrent worktreeにない | `tools/kle-storybook/src/host_types.rs:3-4,52-56`; `tools/kle-storybook/src/host_runtime_fixtures.rs:15-29`; `tools/kle-storybook/src/main.rs:35-58` | 旧editor fixture/legacy stateは最終rootのsourceにならない |
| interactiveは `show_root_write_artifact_and_forward_once`、headlessは `render_headless_frame`、motionは `motion_artifact.rs:79-119` のartifact専用経路を使う | `tools/kle-storybook/src/window.rs:23-37,130-145`; `tools/kle-storybook/src/window_renderer.rs:31-80`; `tools/kle-storybook/src/motion_artifact.rs:79-119` | editor/fallback/artifact専用経路を廃止し、同一public KUC root frame loopへ統一 |
| 一般Replace/Replace Allの固定source routeはない。`ReplaceText`は単一spanのlint/review route | `docs/v0-1-0-editor-interaction-source-audit.md:32-37,58-64`; `docs/v0-1-0-external-host-e2e-design.md:550-574` | 仕様決定まで明示的release blocker。KLE/KUCで偽装しない |

## Ownershipとports

| 層/port | 所有するもの | 所有しないもの |
|---|---|---|
| KUC generic presentation/root | text surface、toolbar/floating/search/context-menuのpresentation、retained identity/revision、root frame | KatanA action、文書/path/URL、Markdown意味論 |
| KUC platform runtime | text/raster、font catalog、CJK、IME、VS16/ZWJ、color glyph、selection/cursor、AccessKit、popup/menu、generic dispatcher | KatanA固有検索、authoring transform、file IO、replace semantics |
| KUC dispatcher port | opaque transportを一回だけgeneric text/search/command/context-menu eventへ展開。stale/duplicate/unhandled/errorはtyped fail-closed | `AppAction`、`DocumentState`、KatanA targetの解釈 |
| Storybook host port | hostからgeneric KUC projection/token/styleを生成しproviderへ供給。generic dispatcherを実装する | token payloadのdecode/clone/serialize/inspect、KatanA固有action列挙 |
| KLE `EguiTextCommandSurfaceEditor` transit | opaque tokenを受け、actual `show`へ渡し、one-shot eventをreal host forwarderへ渡す。receipt/correlationを記録 | child render、font/emoji fallback、local search/replace、host state mutation |
| KatanA host effect port | search/authoring/document/file/clipboard/shortcutの既存source route、state/buffer/dirty/undo/preview/file effect | KUC generic componentの再実装 |
| Storybook artifact port | KUC compositorが生成したframe PNG/record/AccessKit/manifestを保存。GIF/MP4は検証済みframeから派生 | artifactをcorrectness proofにすること、別rendererのpixelを混ぜること |

KUCはgenericであり、Katana固有の意味論をKUC/KLEへ持ち込まない。valid projectionはhostがKUC generic modelから生成し、KLE test/runtimeでchild modelを組み立てない（`docs/v0-1-0-kuc-kle-root-consumption-audit.md:42-54`）。

### Search host transport の設計ゲート

KUC の現行 public root transport は generic `CommandChromeSearchEvent` を class として一回だけ forward するが、host-projected opaque search target/correlation を actual host effect へ伝える port は持たない。したがって、sanitized target を token 内の process-local state に保持するだけでは KLE/KatanA host effect に到達せず、不正な completion root になる。この経路を Storybook または KLE に追加してはならない。

document/workspace search を実装する前に、KUC は target を readback/serialize/debug 公開せず、current-frame event と host-issued correlation を一回だけ結ぶ generic opaque transport を設計する。KLE はその transport を観測・decode せず一回だけ relay し、host だけが correlation を解決する。physical RawInput/AccessKit、stale revision、disabled target-absent、duplicate consumption、wire payload no-leak、actual host effect を同じ execution record で検証できるまで、SearchStrip の表示や Storybook callback を受入証拠にしない。

## 最終アーキテクチャ

```text
StorybookHost
  ├─ ScenarioManifestSource (source-derived leafのみ)
  ├─ GenericProjectionProvider (host-owned KUC presentation/token)
  ├─ GenericDispatcher (KUC event -> typed host port)
  ├─ RealHostEffectObserver (KatanA effect / no-mutation assertion)
  └─ CorrelationLedger (input -> root -> AX -> transit -> effect)
        |
KLE EguiTextCommandSurfaceEditor
  └─ opaque token -> actual show -> one-shot transit receipt
        |
KUC RetainedRoot
  ├─ TextSurface / CommandChrome / SearchStrip / ContextMenu
  ├─ PlatformTextRasterizer + shared PlatformFontCatalog
  ├─ AccessKit tree and current-frame hit/action record
  └─ compositor PNG + root record/hash
```

旧 `StorybookEditorFixture`の文字列・cursor・selection・diagnosticsをpresentation sourceにしない。host観測からprojectionを作り、同一public `show`をinteractive、headless acceptance、motion runnerが使う。KLEにlocal renderer、fallback renderer、minifb、shape-count、coordinate/glyph/line hit-test、fixture callback、legacy text stateを残さない。KUC dispatcher primitiveが先行し、KLEは解釈せず一回だけforwardする（OpenSpec `12.4ap-12.4aq`、監査 `:56-71`）。

## Full scenario inventory

`FullEditorScenarioManifest`を唯一のstage sourceとし、各`step_id`を一つのleafにする（OpenSpec `10.7a-10.7c`）。各leafはtrigger、precondition、close/focus、KUC/AX assertion、class、host effectを持つ。

| 群 | 個別leafの範囲 |
|---|---|
| Document find | open、query、empty/no-result、next、prev、close、focus/IME/AccessKit |
| Workspace search | File Name / Markdown Content tab、filename query、include/exclude/match、invalid regex、history select/remove/clear、no-workspace/no-result、`SelectDocument`、`SelectDocumentAndJump`。document searchのaggregate代替は禁止 |
| Editor geometry/state | line gutter target/jump/scroll、diagnostics popup open/action/close/outside/hover suppression、read-only、selection/no-selection、cursor restore、focus handoff |
| Authoring | 常設toolbar14操作（inline 4、structure 8、reference 2）、floating toolbar lifecycle、outside/Escape close、viewport clamp、diagnostic suppression、dirty/undo/preview。candidate列挙だけは不可 |
| Code | code-block menuの`CodeBlockKind`全17種、open/select/close/outside/Escape/focus/read-only |
| Context menu | save、format、horizontal-rule、link、table、authoring subgroup、clipboard-image、disabled/close/focus |
| Image ingest | file picker、clipboard raw image、clipboard file list、percent-encoded `file://`、ordinary text paste、payload-none/error、read-only、unsaved。成功時のasset/Markdown link/dirty/Explorer refresh、cancel/error/no-mutationを分離 |
| Navigation chrome | document tabs（select/traverse/close/pin/restore/reorder/group）、breadcrumb virtual/no-workspace/menu/candidate、source address input/button/Enter/blank/history |
| Text/raster | 日本語、IME composition/commit、exact `⭐️`（U+2B50 U+FE0F）、variation selector、ZWJ、CJKのcode point、selection/cursor、same-surface raster/AccessKit |
| Replace blocker | Replace/Replace Allは固定sourceでroute不在。visible support、counter、callback、`ReplaceText`単一span、`Unsupported`期待を成功扱いせず、host仕様決定までmanifestで未達blockerを記録 |

根拠: 固定sourceの境界と主要leafは `docs/v0-1-0-editor-interaction-source-audit.md:13-19,21-64`、OpenSpec `tasks.md:142-152,323-330`。特に14操作、17種、3 image ingest、read-only/selection、Unicode/IMEは各々実KatanA buffer/file/state effectまで検証する。

### Navigation Input Scenario Boundary

Source Address の Storybook 表現は KLE の widget、URL input、又は callback として追加しない。
KUC の `FullTextCommandSurfaceScenarioFactory` が generic
`NavigationInput` scenario を発行し、同一 opaque host-projection lease の中で
`SourceAddressProjectionLease` を消費する。KLE と `tools/kle-storybook` が知るのは
generic scenario ID、opaque lease、既存の root forwarding receipt だけである。

KUC は scenario 内部に localized presentation、physical pointer/focus、raw text、Enter、
KUC-private consume-only submission port を保持する。stage の public API/debug、KLE
receipt、frame record、AccessKit、artifact、router context から raw text、URL/path、
source-address target を readback できてはならない。scenario port は KUC test でのみ
submission を受け取ったことを検査し、Storybook host の effect callback 又は KLE state
へ値を渡さない。これは generic retained-UI/input evidence であり、fixed KatanA の
`OpenUrl` effect の代替にはならない。

実装順序は次で固定する。

1. KUC scenario factory に `NavigationInput` を追加し、opaque lease 内で source-input
   strip と private one-shot port を mount する。
2. KUC test は空の retained root に KUC-owned physical pointer、text、Enter を順に
   適用し、submission 一回、duplicate/stale/port rejection、public raw-value non-leak、
   root/AccessKit/artifact coherence を確認する。
3. KLE Storybook は generic `NavigationInput` ID を既存の full scenario inventory に
   加え、opaque transit receipt の cardinality と KUC artifact/AccessKit の一致だけを
   検査する。Source Address 固有型、入力値、座標、URL/OpenUrl、port の import は
   AST gate で拒否する。
4. fixed KatanA host が実際に bootstrap/input/effect observation を公開した後だけ、
   同じ KUC-private submission を host が `OpenUrl` として解決する end-to-end leaf を
   追加する。それ以前は scenario green や動画を host parity と報告しない。

### Workspace Tabs Scenario Boundary

`WorkspaceTabs` は、KUC の同一 retained root に本文、既存の generic command
surface、generic `TabStripProjectionLease` を結合して描画する Storybook scenario
である。これは KatanA の document/workspace state を再実装する scenario ではない。
KUC が localized tab/group presentation、host-issued opaque capability target、drag
pointer capture、context/group overlay、current-frame AccessKit、最終composite artifact
を保持する。scenario 内部の target、label、色、座標、drag stage は KUC-private とし、
KLE/Storybook は scenario ID、opaque root lease、stage数、closed receipt、artifact
hash 以外を読むことはできない。

この scenario は次の UI proof を必須とする。

1. tab-strip は同一rootの本文より前の領域にあり、bounds が重ならない。
2. physical RawInput の drag は native threshold 後に `StartDrag` を一度だけ送り、
   hostが明示した tab/group/end destination だけに `FinishDrag` を一度だけ送る。
   Escape、disabled/stale/rejected/unknown destination は `CancelDrag` のみとする。
3. drag ghost と drop indicator は root-composited artifact に載り、KLE は別renderer、
   geometry、fixture canvas を持たない。
4. Storybook の motion/contract runner は同一scenarioを実行し、各stageでKUC root
   record、AccessKit、KLE opaque transit receipt、artifact provenanceを検証する。

この generic scenario の成功は KUC retained UI の証拠に限定する。select/close/pin/
restore/reorder/group persistence の KatanA parity は、固定KatanA hostの同一 `step_id`
における physical input、handler、final state/persistence effect が別途結合するまで
未完了である。

### Default Workbench Composition

interactive Storybook の初期 surface は `WorkspaceTabs` を KUC が合成する full-workbench
state とする。同一 root artifact には、generic tab strip、Markdown command chrome、automatic
line gutter、document-find query/result count/current-match highlight、および platform text
raster を必ず含める。検索の next/previous/close は KUC の generic one-shot transport として
描画し、`Replace` / `Replace all` は fixed KatanA に一般 route がない間、visible disabled
control のままにする。KLE はこの構成を条件分岐、text fixture、query、range、tab target、
geometry として保持せず、opaque `WorkspaceTabs` lease を一度だけ relay する。

selection floating toolbar、context menu、code-block dropdown、read-only、IME、tab drag は、
同じ KUC root contract の state-dependent overlay である。初期 surface に常時表示して
selection/focus lifecycle を偽装せず、各 overlay は physical RawInput/AccessKit trace と
KUC artifact の個別 scenario で検証する。generic Storybook root の成功は fixed KatanA
host effect の証拠ではない。

## Strict automated verification

1. 各実行にsource revision/hash、`step_id`、input fixture hash、immutable `execution_correlation`を付与する。
2. `RawInput`（pointer/key/IME/AccessKit）を物理またはsource-derived host inputとして記録し、KUC current frameのroot record/hash、hit/action、AccessKit node/focus/valueを同じframeで記録する。
3. KLEはopaque transit receipt（one-shot、target/revision/correlation、cardinality）だけを記録し、dispatcher receiptはgeneric event、consumer、stale/duplicate/unhandled/error classを記録する。`pending_action`、direct `AppAction`、state hook、simulatorは不可。
4. effect classを `kuc_retained_ui_effect`、`in_process_host_effect`、`native_external_host_effect`、KatanA-owned `editor_shortcut`に分ける。後二者はKatanA自身のinput/action/handler/frameと、文書・buffer・dirty・undo・preview・file・Explorer効果をassertし、KUC-only表示で代替しない（host-E2E `:41-69`）。
5. fail条件はmissing/duplicate/stale/cross-document/wrong-direction/unhandled、counter-only、fixture injection、fallback、manual/fixed coordinate/fixed wait、ignored external test、未解決Replace。no-mutation leafもpre/post host state hashで確認する。
6. artifactは番号付きKUC-composited PNG、manifest、SHA-256、record/AX/effect receiptを生成し、header/dimension/non-empty/duplicate/staleness/provenanceを検証する。GIF/MP4は全検証済みPNG列からのみ生成し、decode frame hashとPNG hash列を一致検査する。スクリーンショット・動画はレビュー用で、正しさの証明にしない（OpenSpec `10.8a`、host-E2E `:593-599`）。
7. `⭐️`は単独grapheme cropのchromatic pixel/hashと同一configの`☆` controlとの差分で検証する。aggregate emoji色数、code-point-only、SansSerif/OS fallback、profile skipはfail（OpenSpec `10.9a`）。

## 実装フェーズと受入条件

| phase | 実装範囲 | 受入条件 |
|---|---|---|
| 0. gate固定 | manifest schema、owner/class、replace blocker、禁止語/AST/runtime gate | `FullEditorScenarioManifest`が全leafを列挙し、Replace/legacy/fallbackを成功扱いしない。source hashを記録 |
| 1. KUC dispatch | generic dispatcher、one-shot receipt、correlation、stale/duplicate/error | KUC unit/contract testでgeneric eventのみを一回消費。Katana-specific型・semanticsなし |
| 2. KLE root cutover | actual main `show`、provider受領、opaque transitのみ。interactive/headless/motionを同じroot loopへ統合 | provider注入時はKUC root/AX/receiptが生成され、non-counter forwarderへ一回到達。provider不在は明示fail。legacy editor/fallback pathなし |
| 3. Storybook host | host-owned generic projection、dispatcher、effect observer、correlation ledger | valid tokenをKLEが組み立てず、同一correlationにRoot/AX/transit/effectが結合。fixture text stateやcallback counterは受入証拠にならない |
| 4. scenario coverage | search/gutter/diagnostics/toolbar14/floating/context/code17/image3/tabs/breadcrumb/source/read-only/selection/JP-IME-raster | 全leafを実RawInput、KUC/AX、class-appropriate KatanA effectで個別green。workspace searchをdocument searchで代替しない |
| 5. external/effect gate | clipboard/file/native、shortcut router、buffer/file/dirty/undo/preview、no-mutation | direct action/pending action/host simulatorなし。KatanA実hostの入力・handler・effectとKUC frameを相関結合 |
| 6. artifact/release gate | checksum/freshness、PNG列、GIF/MP4 hash、font profile | decoded MP4 hash一致、KUC compositor provenance、macOS/Windows/Linuxのcolor face証拠。Replace blockerが解消されるまでfull-spec完了を宣言しない |

「MVP done」「Storybook表示済み」「artifact生成済み」は完了状態ではない。全phaseの受入条件、全source leaf、Replace blockerのhost仕様決定、実KatanA効果が揃うまで、状態は未完了のままとする。
