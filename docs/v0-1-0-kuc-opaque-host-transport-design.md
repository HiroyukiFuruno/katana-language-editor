# KUC Opaque Host Transport 設計

## 目的

KUC の TextCommandSurface で発生した generic UI input を、KLE が target、query、range、document state、callback payload を観測せずに actual host effect へ一回だけ届ける。document/workspace search、authoring、image ingest、navigation は同じ transport 契約を使い、KUC に Katana 固有の action を追加しない。

## 現行の不足

`EguiTextCommandSurfaceRootEventTransport` は Text/Toolbar/Floating/Search/ContextMenu の raw generic event class を dispatcher に渡すだけである。`SanitizedSearchProjection` の opaque target を token payload や process-local `Rc` に置いても、KLE の public root relay から host effect へ消費する port がない。この状態で SearchStrip を Storybook に表示しても、actual effect の証拠にならない。

2026-08-22 の source audit では、さらに `SanitizedDocumentRootFactory` が Search / Command / ContextMenu の raw event を root payload から detach し、`RootEventForwarderBridge` が各 `Sanitized*EventTransport::invoke_once()` を **outer forwarder の前に直接実行**していることを確認した。後から渡される `SanitizedDocumentRootEventTransport` は private な root transport と sanitized event vector を抱えるだけで、actual host が root transport を dispatch する public port を持たない。これは KLE が opaque relay だけを担い、actual host だけが effect を起動する境界に反する。SearchStrip だけを足したり、既存 bridge に callback counter を置いたりしても、この欠陥を解消しない。

## 所有境界

| 層 | 所有するもの | 禁止するもの |
|---|---|---|
| KUC | generic control input、revision/current-frame validation、opaque host-effect batch、single consumption | Katana action、path、query、document/buffer mutation |
| host projection provider | opaque target と host-owned one-shot callback の登録、localized presentation | KUC component tree の直接操作 |
| KLE | outer event transport の一回 relay、receipt/correlation/cardinality | callback target の decode/readback/clone/serialize、query/range/state の保持 |
| actual host | registered callback の実行、source-derived state/file/native effect assertion | KUC layout/renderer の再実装 |

## Transport 契約

1. host は KUC generic projection builder に opaque callback registration を渡す。registration は host-only callback と correlation を持つが、payload accessor、`Clone`、`Serialize`、`Debug` payload 出力を持たない。
2. KUC は RawInput/AccessKit を current revision の generic control event に変換し、capability/revision/correlation を検証する。disabled、missing target、stale、duplicate、callback rejection は fail-closed とする。
3. valid event は `KucOpaqueHostEffectBatch` に格納する。batch は KLE から内容を読めず、`consume_once(dispatcher)` だけを持つ。
4. root outer transport は既存の generic class dispatch と effect batch を同一 consumption state に保持する。dispatcher は generic event class を受けた後、opaque effect batch を一回だけ host dispatcher へ渡す。途中 error は残りを実行しない。
5. KLE は outer transport を一回だけ host forwarder に渡す。KLE receipt は class count、effect count、correlation fingerprint、consumed-once だけを記録する。
6. host dispatcher が batch を消費した時だけ登録 callback が呼ばれる。host は callback の closure を通じて自身の action/state/file effect を実行し、KUC/KLE は target 内容を観測しない。

## Sanitized bridge の置換設計

1. `SanitizedDocumentRootFactory` は capability / revision / correlation を検証して sanitized event を生成してよいが、生成 frame 中に `invoke_once` してはならない。
2. `RootEventForwarderBridge` は command、context-menu、search の validated sanitized event を一つの `KucOpaqueHostEffectBatch` に収容し、root transport へ一回だけ attach する。event vector は outer sanitized transport の payload として持ち出さない。
3. `KucOpaqueHostEffectBatch` は登録 effect の個数だけを receipt 用 metadata として扱える。target、query、replacement、range、callback、capability は accessor / token wire / `Debug` / error / artifact に現れない。
4. outer forwarder が transport を受理した後、actual host が `dispatch_once` を一回実行する。KLE の forwarder は transport を再構成、dispatch、clone、decode しない。raw child dispatch が全て成功して初めて batch が host dispatcher に渡る。
5. outer forwarder error、child dispatcher error、opaque effect rejection では current batch の未実行 effect は mutation なしで廃棄する。再 forward、再 dispatch、同一 effect の再起動は不可とする。
6. `SanitizedDocumentRootEventTransport` と `SanitizedDocumentRootEventForwarder` がこの旧 direct-invoke ownership を public に示す限り、それらは KLE release path から除外する。互換 wrapper を残す場合も、root transport を host dispatcher へ一回だけ relay するだけに限定し、sanitized event vector / invoke API を公開しない。

この置換は search 専用の修正ではない。同じ bridge にある command / context-menu を direct invoke のまま残すと、「generic KUC effect batch」が UI class ごとに異なる ownership を持つことになり、KLE の opaque relay という v0.1.0 境界を再び破る。

## KLE 接続前の lease 境界

現行 KLE の `HostProjectionProvider` は `EguiTextCommandSurfacePresentationToken` だけを返し、`KucRootBinding` はその wire token を retained KUC root へ渡す。token は serialize 可能な presentation であるため、callback、target、correlation registration を追加してはならない。また、KLE の `KucRootBinding` は raw `EguiTextCommandSurfaceRootEventTransport` を outer forwarder へ渡すだけであり、`dispatch_once` を呼んではならない。

このため KLE の SearchStrip / command / context-menu を接続する前に、以下の二層契約を導入する。

1. `EguiTextCommandSurfacePresentationToken` は従来どおり wire-only presentation とし、callback、opaque target、runtime registry を持たない。
2. KUC は非 serialize・non-clone の `EguiTextCommandSurfaceHostProjectionLease` を提供する。lease は private token と private `KucOpaqueHostEffectRegistry` を保持し、host projection provider だけが builder で生成する。KLE は lease の内部へ access せず、KUC binding に一回渡すだけとする。
3. registry の key は KUC generic control identity（root identity / revision / generic event class / KUC-local action correlation）だけである。Katana action、path、query、range、document state は key や KLE DTO に入れない。
4. KUC retained root は lease の current revision と control identity が一致した RawInput / AccessKit event だけを registry に照合し、対応する opaque callback を `KucOpaqueHostEffectBatch` に移す。stale、disabled、missing registration、duplicate は batch を作らず fail-closed とする。
5. KLE `HostProjectionProvider` / `HostProjectionBinding` は token を受け取る API から lease を受け取る API へ置換する。KLE は lease の `Debug`、wire encode、semantic access、callback invocation、`dispatch_once` を持たない。AST lint でこの禁止を固定する。
6. `KucRootEventBatchForwarder` の実装は actual host が提供する。Storybook は actual host adapter を通じて root transport を dispatch し、KLE public editor は outer relay のみを行う。KLE/Storybook の callback counter は host effect の代替証拠にしない。

この lease を先に実装・検証しない限り、KUC の sanitized bridge だけを KLE に import したり、wire token に `Rc` を詰めたり、Storybook の `KucRootEventBatchDispatcher` で local state を更新したりしてはならない。

### Lease 内部の generic router

lease は callback を単に保管するだけでは足りない。RawInput / AccessKit の event value（検索文字列、toggle 値、generic action id）を current-frame の登録 callback へ対応付ける必要がある。そのため KUC には次の private-payload / public-generic 契約を置く。

1. `KucRootEventEffectRouter` は host が実装する generic trait で、Text / Toolbar / Floating / Search / ContextMenu の **KUC event payload** と root identity / revision / correlation を受け取る。event cardinality だけでは SearchStrip の query / replacement / option value を actual host へ渡せないため不合格とする。Katana type、KLE type、document type は trait に出さない。
2. router は current frame の generic event から `Option<KucOpaqueHostEffectBatch>` を返す。batch は callback と host-owned target を private に保持する。router は raw event を KLE へ返さず、KLE は router の instance や result に access しない。
3. `EguiTextCommandSurfaceHostProjectionLease` は wire token と router を private field として所有し、factory / retained root だけが consume する。`Serialize`、`Clone`、payload getter、router getter、payload を出す `Debug` は持たない。
4. retained root は `show` 後かつ outer forward 前に router を一回呼び、current root event batch へ opaque effect batch を attach する。この router failure は generic opaque routing error とし、outer forward は行わず callback も起動しない。
5. outer forwarder error は attached batch を未実行のまま廃棄する。actual host `dispatch_once` は raw class dispatch 成功後だけ batch を consume する。raw class failure / router failure / effect failure は相互に mutation を持ち越さない。

router の input は KUC generic event 型を使うため host は effect を選択できる。一方で lease 自体の public API は opaque なので、KLE が query、range、selection、target、callback を再構成する余地はない。KLE source は router context の accessor、router construction、lease `into_parts`、raw event variant への match を AST lint で禁止する。

## 必須検証

- public host-token -> retained KUC root -> physical RawInput と AccessKit -> one generic Search class event と一 opaque host effect の相関を確認する。
- physical RawInput / AccessKit から Search、Command、ContextMenu の各 one-shot effect を生成し、outer relay 前には effect が 0 回、actual host dispatch 後には各 1 回であることを確認する。
- callback は同一 frame で一度だけ実行され、二度目、stale revision、disabled/missing target、forwarder error、callback rejection は no-mutation である。
- `SanitizedDocumentRootFactory` と `RootEventForwarderBridge` の source / contract test は `invoke_once` が frame construction 又は outer forwarder の内部で呼ばれないことを拒否する。
- token wire、Debug、KLE receipt、artifact、dispatcher error から opaque target/callback/query/range が漏れない。
- lease の wire encode / clone / public payload accessor が compile / source contract で拒否され、KLE source の `dispatch_once`、runtime registry readback、sanitized transport import は AST lint で拒否される。
- same revision の projection/correlation 相違は reject し、new revision だけが current-frame state を更新する。
- Storybook は KUC root/AccessKit/transit/effect を同一 execution record に残す。callback counter、local search/replace、固定座標 hit-test、fixture completion は不合格とする。

## Storybook generic interaction locator

KatanA の toolbar、floating toolbar、code dropdown、context menu を Storybook の
physical `RawInput` / AccessKit で検証する際、KLE が action の座標、egui `Id`、
AccessKit node を推測・再構成してはならない。これを避けるため、KUC は
current-frame の component record からだけ得られる **generic interaction locator** を
提供する。

1. locator の selector は KUC presentation の opaque/generic action identity と child
   class だけであり、Katana action、document path、selection range、query、callback は
   含めない。
2. locator は current revision / correlation に束縛され、stale revision、disabled、
   hidden、dropdown 未展開、missing action を typed no-mutation で拒否する。
3. locator は KUC 内で actual egui response / AccessKit node / current bounds を解決する。
   KLE は `UiRect`、egui `Id`、coordinate、hit-test、raw event を計算・保存しない。
4. Storybook host は locator が返す one-shot generic input request だけを物理 frame に
   投入し、KLE public editor は既存 root relay だけを実行する。root record、AccessKit、
   event batch、opaque host effect、artifact hash を同じ correlation で join する。
5. KUC API は generic のまま KDV/KLE が再利用できる。KLE 固有 test helper、fixed
   coordinate、action-id hash 再実装、callback counter は API または test fixture の
   代替にならない。

### 実装受入条件

- `KucOpaqueInteractionRequest` は selector の成功時にだけ KUC 内部の入力を保持し、
  host が `egui::RawInput` へ一度だけ適用できる opaque API を持つ。`InputState`、
  `Event`、座標、egui `Id` を KLE/KDV や public caller へ露出しない。
- root は current frame の actual component response から locator を構成する。test-only
  record、artifact bounds、過去 frame の record を production locator の入力にしては
  ならない。locator の構築と request 適用は root identity / revision / correlation を
  検証し、root mismatch、stale、duplicate、already-queued を typed no-mutation で拒否する。
- strict test は toolbar、floating toolbar、dropdown trigger、17 code items、search
  controls、context-menu leaf の physical RawInput/AccessKit route を current-frame locator
  経由で実行する。missing、hidden、disabled、ambiguous、stale、duplicate、別 root、二重適用を
  それぞれ no-mutation で検証する。
- 同じ current frame で二つ以上の enabled target が同一の pointer-interactable bounds を持つ
  場合、pointer `RawInput` は target を識別できない。locator は selector が一致していても
  `Ambiguous` を返し、input/event/effect を変更してはならない。AccessKit action request を
  pointer click の代替として注入してこの衝突を通過させることは禁止する。これは production
  layout の overlap を検出する generic KUC invariant であり、actual root overlay fixture で
  physical root/AccessKit/no-mutation を検証する。
- context menu を物理入力で開く本文領域は command item ではない。この入口を Storybook や
  KLE が AccessKit label、record、座標から復元してはならない。KUC locator は current-frame の
  actual TextSurface response にのみ束縛した、一回限りの opaque context-open request を提供する。
  KLE は request の geometry、TextSurface identity、RawInput event を観測せず transit するだけと
  する。KUC は stale/root mismatch/duplicate/hidden/disabled/ambiguous を no-mutation で拒否し、
  request 適用後の次フレームで初めて context-menu leaf を locator に公開する。これは KUC の
  generic text-surface interaction であり、KatanA/KLE 固有の context action 又は座標 API を
  導入する根拠にはしない。
- 2026-08-22 に置かれた未追跡 `interaction_locator.rs` の骨格は module に接続されておらず、
  現行の adapter test/lint green の対象外である。この状態を実装又は検証済みと数えない。
  上記 API と strict test がそろうまで source code の採用、Storybook host-E2E、parity
  evidence への昇格を禁止する。

この locator が実装・strict test されるまで、code dropdown 17 種、context menu、
floating authoring action の Storybook evidence は表示確認に留まり、KatanA parity / host
effect evidence としては扱わない。

## 非目標

Replace / Replace All の一般文書 route は固定 KatanA source にない。transport はその不足を隠さず、user-mandated extension blocker のまま扱う。KUC/KLE に検索アルゴリズム、置換、KatanA action enum、document state を実装しない。
