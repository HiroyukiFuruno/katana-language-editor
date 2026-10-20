# KLE v0.1.0 KUC Root Cutover 実行計画

## 前提

この計画は `docs/v0-1-0-kuc-text-emoji-root-migration-design.md` の境界を実装順に
分解したものである。KatanA は read-only とし、KUC は generic UI の owner、KLE は
opaque root の thin consumer、KatanA host は semantic effect の owner とする。各 gate を
通るまで次段階へ進めず、旧経路を fallback、compatibility wrapper、Storybook evidence として
使用しない。

## Phase 0: Source Closure と受入台帳

1. KatanA の editor UI、shortcut、search、Markdown authoring、clipboard/image、tab、diagnostics、
   preview の source branch を definitionから最終 effectまで leaf 化する。
2. KatanA sourceに無い user-mandated replace/replace-all は明示的に extension leaf として分離する。
3. 各 leaf に owner、physical input、KUC record、KLE transport、host effect、negative case を
   結合する manifest schema を固定する。

**Gate:** source count、UI表示、action定義だけでは合格にせず、未分類 branch、shared aggregate
selector、static-only evidenceを fail-closed にする。

## Phase 1: KUC Search/Replace Transport

1. raw `CommandChromeSearchEvent` を root batch から detachし、canonical sanitized transportだけを
   forwardする。
2. target-scoped one-shot capabilityとhost-projected current boolean stateを query、replacement、
   next、previous、match-case、whole-word、regex、replace-one、replace-all、closeに導入する。
3. current revision/correlation以外、duplicate、callback rejection、outer forwarding failureは
   no-replay/no-mutationとする。
4. Japanese、`⭐️` VS16、ZWJを含む query/replacementを actual RawInput、keyboard、AccessKitから
   操作し、transport/receipt/Debug/manifestの非漏洩を検証する。
5. command/dropdown/context activationも同じ target-scoped one-shot transportへ正規化し、KUC-private
   widget IDやraw command eventをKLE/outer transportに渡さない。

**Gate:** 各 enabled operation の capability一回、receipt一件、raw event不在、stale/replay/failureの
no-effectを個別にKUC strict testで証明する。callback countだけをhost effectの証拠にしない。

### Command surface coverage gate (2026-08-21)

初期の command capability 実装は retained top command toolbar だけを対象にする。これは editor root の
完成を意味しない。current root surface は floating command toolbar を `None` として構築しており、
retained context-menu projection には同等の opaque one-shot activation transport がない。三つの generic
KUC surface が同じ transport 保証を持つまで production cutover を禁止する。

| KUC surface | Current state | Required completion evidence |
| --- | --- | --- |
| Top command toolbar | In progress | dropdown を持つ command は real split control とする。physical primary input は direct opaque capability だけを一回実行し、physical secondary input は dropdown を開き physical child activation が child capability だけを一回実行する。 |
| Floating command toolbar | `SanitizedDocumentRootSurface` が未投影 | root input が visibility、anchor、generic command presentation を投影する。physical pointer、keyboard、focus-return、close、全 activation route を public batch から detach し、一回だけ consume する。 |
| Context menu | presentation-only projection | root input が nested generic menu item と opaque capability を投影する。right-click/open、submenu、pointer/keyboard/AccessKit activation、dismissal、disabled/rejected/stale を同じ private transport で扱い、raw target/event を KLE に渡さない。 |

direct action と dropdown の双方を持つ command には `SplitSecondary` を必須とする。`Primary` は primary
interaction が menu を開く command 専用であり、direct command behavior を置換する test convenience として
使用してはならない。

#### Context-menu activation design

context-menu activation は KLE action map 又は second callback transport を作らず、command activation と同じ
ownership model を使う。generic context item は opaque target と optional private `FnOnce` capability を持つ。
KUC は current retained frame だけで structural render ID を target から導出し、physical menu event を original
target に戻して outer root forwarder より前に capability を実行する。capability、target bytes、structural ID、
label、submenu index は KLE-facing record、receipt、Debug、outer event batch から readback 不能でなければならない。

nested submenu も parent capability を継承せず、同じ transport で leaf ごとに解決する。open、dismissal、
hover/focus、submenu state は retained generic KUC UI state に限定する。enabled leaf activation だけが一件の
transport を作る。missing capability、hidden/disabled leaf、unknown structural ID、stale revision、callback
rejection、outer-forwarder failure、duplicate forwarding、outside click、Escape は transport を作らないか、既存の
transport を replay 不能に consume する。

KUC physical test matrix は right-click/open、pointer leaf activation、nested submenu pointer activation、keyboard
activation、AccessKit activation を個別に持つ。各 case は raw context event が root batch に残らず、opaque
capability 一件と receipt 一件だけが起き、Japanese、`⭐️` VS16、ZWJ の label/target bytes が Debug/manifest から
漏れないことを確認する。KatanA の Save/Format/Edit/Ingest leaf の host effect は後続 KLE/KatanA layer だけが
証明する。

physical test は推定座標を使用してはならない。KUC adapter crate の `#[cfg(test)]` 専用 root output は current
`EguiContextMenuFrameRecord` の item bounds を test harness にだけ渡してよい。production public
`EguiTextCommandSurfaceRootFrame`、KLE-facing sanitized record、receipt、artifact manifest へ child geometry、
item ID、label を追加してはならない。test-only accessor は current frame から取得した KUC-owned bounds だけを
使い、right-click/open 後の record refresh と submenu 遷移後の refreshed child bounds を毎回読み直す。

## Phase 2: KUC Full Retained Root

1. KUCが text surface、emoji catalog/raster、gutter/diagnostics、floating Markdown chrome、
   SearchStrip、context menu、TabStrip、status/diagnostics、viewportを同一 retained rootで保持する。
2. public consumer contractは sanitized projection、closed root frame、one-shot event forwarderだけにし、
   child output、paint plan、texture、geometry、ranges、semantic eventを隠す。
3. host-projected localized strings/icons/opaque targetsを使い、KUCにKatanA enum、path、range、
   command stringを導入しない。

**Gate:** 各 component family を physical RawInput、keyboard、AccessKit、disabled/read-only、
outside/Escape/focus、stale/duplicateで検証する。KLEの座標補正、二つ目のwidget、local rendererは
AST/dependency gateで拒否する。

## Phase 3: KUC Storybook と三OS Emoji 証拠

1. KUC Storybookが同じ full rootを実起動し、full Markdown/search/replace/tab/context操作を提供する。
2. KUC encoderがPNG、contact sheet、GIF、MP4、manifestを出力し、MP4 decode frame hashを元 frameと
   照合する。
3. macOS、Windows、Linuxで実解決color emoji face、file SHA、`⭐️`/`☆` isolated crop、chromatic pixels、
   Japanese IME/ZWJ root traceを採取する。

**Gate:** shape count、手描きglyph、Minifb、fallback、非空pixel、fixture canvasを受入証跡に使用しない。

## Phase 4: KLE Atomic Thin-Consumer Cutover

1. release pathにhost-projection inputとopaque forwarderを導入する。
2. `EguiLanguageEditor`の唯一のretained rendering objectを`KucRootBinding`にする。
3. KLEのcontent/selection/search/authoring/diagnostic/pixel/child-artifact stateと、
   `KucTextSurfaceBinding`、output reconstruction、direct egui input/context workaroundを同一batchで
   release graphから削除する。
4. 旧APIは必要なら非release compatibility adapterに隔離し、production、Storybook、host-E2Eからの
   compiled reachabilityをゼロにする。

**Gate:** actual RawInputでone root show/one forward、legacy dependency zero、transport非観測、
KLE local semantic/range/geometry mutation zeroをsource and runtimeで検証する。

### Phase 4a: Opaque Projection Entry Contract (2026-08-21)

現行 KUC には `SanitizedDocumentRootFactory` と `EguiTextCommandSurfaceHostRoot` があるが、KLE
release path が host-projected snapshot を受け取って retained root を生成する公開入口が未接続である。
この欠落を KLE Storybook fixture、`EguiLanguageEditor::show` の direct call、child artifact aggregate、
又は KLE 側の presentation 再構成で埋めてはならない。

1. KUC は generic `EguiTextCommandSurfaceHostProjectionEncoder` を public に提供する。これは host が
   generic root presentation/style と opaque target bytes を渡して revisioned
   `EguiTextCommandSurfacePresentationToken` を一回生成する API である。KatanA/KLE enum、command string、
   path、selection range、coordinate、font/emoji fallback、child paint plan を受け取らない。
2. KUC の `EguiTextCommandSurfaceHostRoot` は token の retain/synchronize、one `show`、closed-frame
   `forward_events_once` だけを公開する。KLE/Storybook は root record の不可逆 hash と forwarding receipt
   以外の child output/RGBA/geometry/accessibility node を読めない。artifact bytes は KUC Storybook feature
   内だけで encoder に接続する。
3. KLE の release-side input は host が生成済み token と opaque forwarder を注入する境界だけにする。
   KLE は token を decode、serialize、clone、inspect、再構成せず、同一 retained `KucRootBinding` に渡す。
   token を作るための KatanA source mapping は KatanA host integration stage の責務であり、KLE Storybook
   fixture を source-of-truth にしてはならない。
4. `EguiLanguageEditor` の旧 direct path をこの入口へ atomic に置換するまでは、KLE Storybook は
   `EguiLanguageEditor::show`、`latest_kuc_artifact_aggregate`、`SanitizedDocumentRootInput` 自前組立を
   release/acceptance path に使用できない。不足 API は typed fail-closed error とし、fallback を設けない。

5. KUC は `storybook-artifacts` feature 内だけで `EguiTextCommandSurfaceHostRootFrame` を受け取る
   `FullRootArtifactWriter` を公開する。writer は KUC 内部で既に合成済みの root RGBA を PNG と
   root-level manifest に出力し、width/height/root record hash/pixel hash/PNG SHA-256 だけを返す。
   raw RGBA、child artifact、paint plan、palette、child geometry、AccessKit node list は KLE に返さない。
   GIF/MP4/contact sheet/decoded-frame hash はこの root PNG sequence だけを入力として KUC Storybook
   artifact layer が生成する。KLE は stage ID と出力先を渡すだけで、pixel conversion/composition をしない。

**Gate:** KUC では opaque bytes が public Debug/record/event に漏れないこと、stale/duplicate/identity
mismatch が typed error になること、physical RawInput/IME/AccessKit で one show/one forward になることを
検証する。KLE では source/AST により direct `EguiLanguageEditor::show`、artifact aggregate、child
compositor、local token builder を release/Storybook/host-E2E から拒否する。artifact writer は KUC
same-root PNG と manifest の hash/provenance を検証し、KLE local pixel conversion と empty-pixel acceptance を
拒否する。これは KatanA host effect の
証明ではなく、その前提となる ownership gate である。

### Phase 4a Blocker: Host Projection Source (2026-08-21)

KUC projection encoder、closed root、root artifact writer、および KLE `KucRootBinding` の artifact-aware
one-shot operation は存在する。しかし既存 `tools/kle-storybook` と `EguiLanguageEditor` には、KatanA host が
発行する初回 retain token と各 frame synchronize token を受け取る API がない。したがって現行の
`EguiLanguageEditor::show`、artifact aggregate、KLE compositor を削除した時点で Storybook は実入力を描画できない。

この欠落は fixture から `EguiTextCommandSurfacePresentationToken` を再構成して埋めない。KLE は
`HostProjectionProvider` という one-shot provider contract だけを public に持つ。provider は opaque token を
返すか `MissingHostProjection` を返す。token は Clone/Serialize/Debug readback 不能とし、KLE は retain token と
synchronize token を binding に直送するだけである。KatanA host が同一 projection revision/correlation を発行して
provider を実装するまで、KLE Storybook の interactive/smoke/motion/acceptance は `MissingHostProjection` で
fail-closed とする。direct `EguiLanguageEditor::show` を残して成功を装うこと、KUC generic fixture を
KatanA/KLE host projection と表示すること、static artifact/callback を代替証拠にすることを禁止する。

## Phase 5: KLE Storybook と KatanA Host E2E

1. KLE StorybookはKUC root artifact encoderを使い、KatanA由来のopaque host projectionだけを注入する。
2. StorybookでMarkdown controls、search/replace、context、tab、IME、emoji、disabled/read-onlyを
   同一full-root screenとして実操作し、動画とmanifestを生成する。
3. KLE-owned read-only KatanA host harnessで、source-derived physical inputからdocument/dirty/undo/
   scroll/asset/persistence等の最終effectを確認する。

**Gate:** `pending_action`、direct core trigger、fixture mutation、Storybook callbackはhost effectの
代替にしない。KatanAがKLEをmountしていない事実もmanifestに明記し、rendered KLE host frameを
虚偽に主張しない。

## Phase 6: Release Validation

1. source closure manifest、KUC/KLE/host execution record、Storyboard media manifest、三OS emoji evidenceを
   leaf単位でjoinする。
2. `cargo fmt`、workspace clippy、workspace test、strict AST lint、coverage、negative mutation testを
   実行する。
3. unreached legacy path、fallback、source exclusion、unproven OS、missing host effect、media hash mismatchを
   release blockerとする。

**Gate:** 全 leaf、全OS、全strict checkが揃うまでv0.1.0は未完了である。
