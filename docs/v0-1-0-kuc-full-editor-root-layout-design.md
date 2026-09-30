# KUC 完全エディタルート配置設計

状態: 設計・実装計画を記録済み。KUCの計測viewport伝播とtext-raster実測metricsをpublic text-command rootの全text slotへ接続した。adapter 145件、KUC core 226件、text-raster 27件、対象crate strict clippyは通過した。ルート全体の視覚品質、Storybook、固定KatanA互換は未達であり、本書はその証拠ではない。

## 1. 目的と現在の問題

本書は、KLE v0.1.0 のエディタルートについて、責務境界と配置契約を固定するための設計・計画書である。計測viewport伝播の限定実装は開始したが、以後の実装は本書の所有境界と検証条件に従う。本書そのものは、固定KatanAとの互換性を示す証拠ではない。

現在のKLE Storybookでは、同じコマンド群が通常ツールバーと選択時のフローティングツールバーへ二重に取り込まれている。`host_types_presentation.rs:21-49` が `toolbar` と `floating` の両方を供給し、`toolbar_projection.rs:44-...` が12個のMarkdown操作を定義し、`floating_toolbar_projection.rs:10-27` が同じpresentationから別のツールバーを生成している。その結果、通常ツールバーと選択ツールバーが同時に表示される。

また、現在のStorybookは `host_types_presentation.rs:29-36` で1280 x 720の固定 `TextSurfaceViewport` を渡している。KUCは `text_command_surface/composition.rs:20-58` でルートの実際の矩形と、ツールバー、文書、検索の矩形を計算している。このため、providerが渡す古いルートサイズのviewportをレイアウトの基準にしてはならない。文書面は、ルートがそのフレームで計測して割り当てた文書矩形を使う必要がある。

固定KatanAのソースは読み取り専用で、`/tmp/katana-fixed-source-closure-20260821` にある。固定エディタは `crates/katana-ui/src/views/panels/editor/text_edit.rs:51-162` で複数行入力、コンテキストメニュー、検索装飾、行番号、選択ツールバーを描画する。エディタルートは `crates/katana-ui/src/views/panels/editor/ui.rs:78-123` でスクロール領域を構成し、文書検索は `crates/katana-ui/src/views/app_frame/tab_toolbar.rs:66-109` から取り込まれる。

KUCには `katana-ui-core-egui-adapter/src/text_command_surface/types.rs:28-67` に、テキスト、通常ツールバー、フローティングツールバー、検索、コンテキストの汎用型がすでにある。KUCは現在、`composition.rs:29-118` で通常ツールバー、テキスト、検索を配置し、その後にフローティングとコンテキストのオーバーレイを描画する。この汎用ルート契約をKUC側で発展させる。KLEが座標補正、操作の二重投入、個別描画で帳尻を合わせてはならない。

## 2. 固定ソースから引き継ぐ機能

実装計画では、固定ソースに存在する次の機能を、個別の要件として保持する。

- 選択、キャレット、IME、コピー、貼り付け、元に戻す、やり直し、読み取り専用を含む複数行Markdown入力
- 行番号ガターと診断ガター
- 検索一致の装飾と現在の一致状態
- 選択範囲を起点とするMarkdown記法の作成ツールバー
- エディタのコンテキストメニュー
- 文書検索のクエリ、次へ、前へ、終了、件数、ハイライト状態
- スクロール位置の保持と同期
- Markdown構文およびコードブロックに関係する操作
- ネイティブホストでの操作解決と、その結果として生じる文書・状態の変化
- プラットフォームごとの文字ラスタライズ。`⭐️` はU+2B50 U+FE0Fとして表示し、`☆`に置き換えない

これはソース監査の基準であり、実行可能な証拠の代わりにはならない。固定KatanAのホスト動作とOSごとの絵文字表示は、別途必須の外部証拠とする。

## 3. KUC汎用ルート契約

KUCは、テキストとコマンド群を持つ画面のために、ひとつの汎用ルートコンポジタを提供する。ルートは次のスロットを管理する。

1. `primary command chrome`: 任意。ルートの主操作群であり、選択オーバーレイが存在する場合に暗黙に必須とはしない。
2. `document surface`: 必須。テキスト、ガター、選択、キャレット、注釈、スクロール、IME、アクセシビリティ状態を含む。
3. `auxiliary search`: 任意。配置は `top`、`bottom`、`external` のいずれかを選ぶ汎用ポリシーとする。`external` では、ルートはスロットとフォーカス契約を報告するが、文書矩形の内側には描画しない。
4. `selection overlay`: 任意。有効な選択範囲とアンカーがあり、汎用表示ポリシーが許可した場合だけ描画する。
5. `context overlay`: 任意。ルートのオーバーレイ層に描画し、文書矩形を変更しない。

ルートは次の責務を持つ。

- 実際に割り当てられたルート矩形を計測し、重ならないスロット矩形を割り当てる
- 計測した文書矩形をテキストサーフェスアダプタへ渡す
- 文書矩形が変わっても保持中の論理スクロール位置を維持する
- z-order、クリッピング、オーバーレイの衝突処理を定義する
- providerの座標を使わず、選択オーバーレイとコンテキストオーバーレイをルート内に収める
- 検索、文書、オーバーレイの間でフォーカス、IME、フレーム内の入力所有権を切り替える
- 決定的なアーティファクト層順とアクセシビリティ巡回順を公開する
- 描画した同じフレームから、数値ジオメトリと入力ルーティングの記録を作る

providerが渡せるのは、汎用presentationと不透明なイベント値だけである。providerは固定のルートサイズviewport、ポップアップ座標、ガター座標を強制できず、ルートコンポジタの外側で子要素を個別描画できない。

### 3.1 コマンド群の取り付け規則

KUCは、同じコマンド群を通常スロットと選択オーバーレイスロットの両方へ取り付けることを禁止する。例外は、ホストが互いに異なる識別子を持つ2つの不透明なコマンド群を供給し、両スロットへの取り付けを明示的に要求した場合だけである。

KUCは、同一の群識別子、意図しない複製、一方から他方を暗黙に導出したコマンド群を拒否する。ラベルや見た目からpresentationの同一性を推測してはならず、識別子と取り付けポリシーを契約の基準にする。

### 3.2 計測viewport規則

ルートは、KUCの `AdapterMeasured` sizing、または同等の計測ルート契約を使って文書面を配置する。権威となるviewportは、そのフレームでルートが割り当てた実際の文書矩形である。consumerが渡す古いルートサイズviewportをレイアウトの入力にしてはならない。

テキストサーフェスアダプタは計測された文書矩形を受け取り、レイアウト変更後も論理スクロール位置を保持する。ルートの計測では、通常コマンド群の実測高さと、検索の配置を考慮する。`external` の検索は文書矩形から領域を差し引かない。選択とコンテキストのオーバーレイは文書サイズの計算から除外し、ルートのオーバーレイポリシーでクリップまたは収める。

この規則を実装するために、KLEの座標型、egui矩形、バイト範囲変換、ポップアップアンカー変換、ガター幅計算を追加してはならない。

### 3.3 ルート順序と衝突契約

ルートの順序は次のとおり固定する。

1. 取り付けられている場合の通常コマンド群
2. 計測された文書面
3. ルート内に配置された場合の補助検索
4. 選択オーバーレイ
5. コンテキストオーバーレイ

この順序は、単なる描画順ではなく、アーティファクトとアクセシビリティの契約である。ルートは各層について、取り付け回数、計測矩形、クリップ矩形、フォーカス対象、アクセシビリティ順序を記録する。文書面とルート内検索の重なりは禁止する。オーバーレイの重なりはproviderの座標ではなく、ルートが持つ衝突ポリシーで解決する。

### 3.4 実測text metrics契約

themeはfont roleと希望サイズを宣言するだけであり、`TextCommandSurfaceStyle` が希望サイズから固定のline-heightやgutter幅を推測してはならない。KUC text-rasterは、同一frameのfont catalog、選択されたface、pixels-per-point、font roleから汎用の実測metricsを解決する。metricsは少なくともascent、descent、baseline、line height、glyph advance、gutter label幅を含む。

KUC rootは同一のmetricsを本文、行番号/diagnostic gutter、検索入力、command chrome、floating overlayの計測に使う。scale factor、font catalog、font roleが変わったframeではmetricsと全slot矩形を再計算する。KLEはfont size、line height、gutter幅、baseline、glyph advance、DPI、座標補正を供給または再計算してはならない。

既存の `TextCommandSurfaceStyle::standard()` と `from_theme()`、既存style structのfieldは後方互換を維持する。実測metricsはKUC内部のruntime結果または加算builderとして導入し、既存consumerに必須fieldを追加しない。standard themeの13px値をKatanA特例として変更するのではなく、実rootが同一rendering contextのmetricsを用いることを標準経路にする。

KUCの数値・pixel検証は、同一catalog/scaleでの決定性、DPI変更時の再計測、本文/gutter/search/chromeのmetrics共有、日本語/漢字/ASCII混在のadvance・折返し・caret、正確な `⭐️`（U+2B50 U+FE0F）と `☆` の別crop、色付きglyph、混在行のbaseline/line-height、frame間jitterを必須とする。3 OSのface identityとraster証拠は固定KatanA host E2Eとは別のKUC release blockerとして保持する。

2026-08-23時点で `katana-ui-core-text-raster` に `PlatformTextMetricsRequest`、`PlatformTextMetrics`、grapheme advance、`PlatformTextRasterizer::measure_text` を追加し、cosmic-textのlayoutからmetricsを取得する基盤を実装した。さらにKUC public text-command rootはframe開始時に `PlatformTextMetricsFrame` を初期化し、本文、automatic gutter、search input、primary/floating command chrome、context menuの実raster requestへ同一frameの実測line heightを渡す。実egui root testはRawInputでcontext menuを開き、本文・gutter・search・toolbar・floating・menuの日本語/`⭐️`を記録し、DPI変更で全recordが再計測されることを確認する。adapter 145件、KUC core 226件、text-raster 27件、format、対象crate strict clippyは通過した。ただし、KatanAとの目視一致、artifactの最小文字高・行番号本文開始位置・gutter占有率・frame間pixel jitterの数値gate、3 OS color glyph artifact、KLE Storybook統合は未実装であり、この接続だけで文字高・ガター・jitterの合格を主張してはならない。

### 3.5 Full-surface Storybook scenario契約

KLE StorybookはKUC rootの表示確認であり、KLEがeditor fixture、選択range、viewport、font、行番号幅、floating anchor、menu/search/chromeの状態を組み立てる場所ではない。KUC adapterは次のgeneric `FullTextCommandSurfaceScenario` を所有し、scenario IDからopaque host projection leaseとdeterministic RawInput stage列を発行する。

1. `resting`: 長文・複数行・CJK/ASCII/VS16を持つ未選択の本文、automatic gutter、primary command chrome、検索strip。
2. `selection`: pointer selection後にだけfloating chromeを表示し、Escape/outside click/focus returnまでを含む。
3. `find`: query text、previous/next/close、およびfixed host contract不在を示すdisabled generic replacement capabilityを持つ。queryの意味、match/range/replace mutationはscenarioにもKLEにも持たせない。KUC fixture自身のmatch/current annotationは§3.5.4のvisual contractに限る。
4. `context`: actual RawInputでcontext menuを開閉する。項目はopaque targetを持つgeneric command presentationだけとする。
5. `readonly`: text mutationを拒否しつつ選択、検索、アクセシビリティを検証する。
6. `resize-scroll-ime`: resize/scroll/IME/CJK/`⭐️` VS16、scale変更、frame間jitterを同一root artifactで検証する。

KUCはscenarioのdocument text、generic localized display text、selection生成、gutter、style、layout/raster、artifact/AccessKit/RawInputを所有する。KLE Storybookはsource-derived leaf IDとKUC scenario IDの対応、opaque leaseのone-shot relay、actual host effectのreceiptだけを持つ。KLEはraw fixture text、selection byte offset、font/spacing、pixel/anchor、KatanA command/action ID、検索・Markdown・replaceの意味状態を保持しない。固定KatanA routeがないReplace / Replace AllはKUC generic capabilityとしてdisabled表示できても、KLE/KUC local mutationやsource parity claimを作らずrelease blockerのままとする。

各scenarioはroot-composited PNG、GIF、MP4、decoded-frame hash、KUC root record、AccessKit record、opaque dispatch receiptを同一manifestに記録する。visual gateはminimum text pixel height、gutter/content start、occupied text rows、floating visibility/offset、CJK/VS16 chromatic crop、same-state frame hash、resize/scale deltaを数値で検査する。これらはfixed KatanA host effectとは別のKUC/KLE Storybook証拠である。

#### 3.5.1 KUC-owned motion plan

GIF/MP4のframe順序、反復、およびprovenanceはKUC adapterの`FullTextCommandSurfaceMotionPlan`が所有する。planは全six scenarioと各scenarioのKUC-defined RawInput stageを含むcatalogueを一巡するまでを最小要求数とし、それ未満の要求を補完、切捨て、又はKLE側のidle frameで偽装してはならない。要求数はKUC catalogueの完結した周期数でなければならず、周期途中で終わる要求はfail-closedとする。要求数がcatalogueを上回る場合はKUCが完結したcatalogueを同じ順序で反復して、正確な要求frame数を発行する。各catalogue stageはKUC provenanceとroot receiptを必須とし、KLE生成の無操作frameとは区別する。

各motion frameはKUC-issued scenario ID、opaque input stage、KUC-issued provenance IDを持つ。KLEはscenario IDの意味・stage index・event payloadを解釈又は組立せず、scenario切替時にopaque leaseを差し替え、stageを一度だけpublic rootへ適用する。manifestのstage列はKUC-issued provenanceをそのまま記録し、`idle`、欠損stage、KLE合成label、未分類frameが一つでもあればartifact生成を失敗させる。KUC contract testは最小catalogueの完全被覆、要求数未満のfail-closed、反復後にも未分類frameがないことを検証し、KLE Storybook testはrequested/encoded/manifest frame数とKUC-issued provenanceの全件一致を検証する。

#### 3.5.2 Current-frame physical selection driver

`selection` scenarioのnon-empty range、floating visibility、anchor、pointer座標をpresentation又はKLEに固定してはならない。KUCの最初のselection frameはcollapsed text surfaceをpublic rootで描画し、そのsame-frame `KucInteractionLocator`だけが実際の`TextSurfaceFrameRecord`から安全な本文hit areaを導く。locatorはprivate geometryを保持した`KucOpaqueTextSelectionDriver`を発行し、driverがpress、move、releaseを別々のsubsequent frameに一度ずつ適用する。release後のcurrent-frame KUC recordがnon-empty selectionとfloating surfaceを持たなければfail-closedとする。

KLE StorybookはKUC-issued motion frameが指定するopaque lifecycle hookだけを呼び、driverを読まずに保持して次入力へ適用する。KLEは`UiRect`、pointer位置、selection range、frame state、driver phaseを持たず、callbackを捨てたりraw inputを追加したりしない。driverのroot identity不一致、二重適用、stage順序不正、最終selection/floating欠落はKUC typed errorとしてartifact生成を停止する。outside/Escape/focus returnは同じKUC driver familyの後続契約であり、fixed KatanA host E2Eとは別に検証する。

現行のselection scenarioはpresentationから非空rangeを与える暫定表示であり、この設計の実装ではない。したがって、静的selection artifact、floating表示、KUC内部unit testだけをphysical selection完了の根拠にしてはならない。`11.0k7c` と `11.0k7c1` が完了するまで、release blockerとして扱う。

#### 3.5.3 Opaque selection continuation contract

KUCは`FullTextCommandSurfaceMotionFrame::capture_continuation`だけを公開し、KLE callbackへ渡されたcurrent-frame `KucInteractionLocator`を消費して、次frameへ適用可能な`KucOpaqueMotionContinuation`を返す。motion frame自身が開始、press、move、release、最終確認のどれを要求するかをKUC-privateに保持する。KLEは`Option<KucOpaqueMotionContinuation>`を非観測で保持し、各frameの直前に一度だけ`apply_to`し、描画直後に同じmotion frameへ渡す以外の操作をしてはならない。

continuationはroot identity、state revision、correlation fingerprint、one-shot consumed state、次phaseのopaque pointer inputをKUC内に保持する。current rootとの不一致、replay、phase飛越、pointer press/move/releaseの欠落、release後のcollapsed range、floating不在、outside/Escape/focus returnの不整合は、KUC typed errorでartifact生成を停止する。public Debug、manifest、KLE receiptはphase、座標、range、rect、raw eventを公開しない。

実装は次の順序に限定する。まずKUC adapterにpublic-root経路そのものを使うdriver contract testを追加し、次にscenario factoryをcollapsed selection startとKUC-owned motion catalogueへ変更する。その後にKLE egui bindingの既存opaque callbackをcurrent leaseを再同期しないartifact pathへ通し、Storybookはopaque continuationをrelayするだけにする。最後にKUC/KLE AST、real-egui/AccessKit/RGBA、GIF/MP4 decode、物理selection/floatingのtransition assertionを同時に通す。KLEにtemporary coordinates、static range、fallback event、test-only semantic switchを置く実装は禁止する。

#### 3.5.4 KUC-owned find trace contract

`find` scenarioは検索stripを静的に描くだけでは不合格とする。KUCはquery text inputを、document text inputとは区別されたgeneric AccessKit targetとして同一frameのevidence ledgerに登録する。`KucInteractionLocator`はそのtargetからのみopaque focus requestを発行し、KLEはstate ID、bounds、pointer座標、入力文字列を観測しない。KUC motion frameはfocus後にIME preedit/commit、next、previous、closeをKUC-issuedの順序で進め、各stepのevent batch、focus、search result summary、active result、close event後のsearch不在をsame retained rootのrecordとcompositeへ記録する。

検索queryの意味、match/range、document mutation、host action mappingはKUCにもKLEにも置かない。scenarioが用いる入力文字列はKUC fixture内部のgeneric presentationであり、KLE Storybookが合成又は補正してはならない。`find` の初期frameだけは、KUC fixture自身が宣言した汎用 `TextSurfaceAnnotation` を `generic-search-match` と `generic-search-current` の二つの視覚roleで本文に描画する。これはannotationの範囲、優先度、RGBA paint、compositeを検証するためのfixture visual contractであり、入力queryから検索結果を導出したり、host検索のmatch/range/current stateを代替又は証明するものではない。annotationの文字列、範囲、role、色、paint planはKUC内部に閉じ、KLEのtoken、receipt、manifest、公開APIへ出してはならない。Replace / Replace Allはfixed KatanAに一般文書routeがないため、可視でもdisabled capabilityのままとし、locatorはdisabled requestをtyped rejectionし、local replacement event又はmutationを絶対に生成しない。

KUCはfind trace用のopaque continuationをselection continuationと同じgeneric continuation familyで保持する。KLEはcontinuationの種類、phase、raw event、query、control identityを解釈せず、frame直前に一度適用し、frame直後にKUC callbackへ返すだけにする。root mismatch、stale frame、重複apply、focus未成立、IME commit欠落、disabled Replace requestの成功、next/previous/closeのevent欠落、close event後のsearch残存はKUC typed errorとしてartifact生成を停止する。

## 4. KLEの責務境界

KLEは薄いbindingおよび不透明値の中継層とする。KLEに許されるのは次の処理だけである。

- KUCルートのleaseを生成または保持する
- 不透明な汎用presentation tokenをルートへ渡す
- 1フレームにつき公開ルートのshow操作を1回だけ呼ぶ
- 不透明なルートイベントバッチを1回だけ転送する
- 不透明なreceipt、hash、artifact、ホストコールバック値を保持して転送する

KLEに次のものを持たせてはならない。

- authoring ID、コマンド名、Markdown操作名、KatanA固有のメニューID
- 文書パス、検索クエリ、置換、バイト範囲、文字範囲、テキスト変更ロジック
- egui座標、ポップアップ座標、ガター座標、ローカルviewport補正
- ホスト操作enum、`AppState`、`KatanaApp`、ホスト効果の実行
- KatanA固有のフォーカス、選択、IME、スクロール、ライフサイクル意味論
- KLE独自のテキスト描画、絵文字描画、フォントフォールバック、SVGラスタライザ、レイアウト回避策

KLEの受け入れ検証では、これらの禁止事項をASTで検査する。公開ルートが動いたという結果だけでは、境界を満たした証拠として不十分である。

## 5. Storybook fixture契約

Storybookは、KLEの不透明bindingを経由して、実際の公開KUCルートを実行する。fixtureは、次の独立した状態を供給する。

- 選択なし。キャレットとIMEフォーカスがある状態
- 有効なアンカーを持つ選択あり。選択オーバーレイを表示する状態
- 選択あり。汎用表示ポリシーによってオーバーレイを隠す状態
- 選択オーバーレイなしで通常コマンド群だけがある状態
- 通常コマンド群なしで選択オーバーレイだけがある状態
- 補助検索を上、下、外部へ配置する状態
- コンテキストオーバーレイの開、閉、フォーカス復帰
- 読み取り専用文書面と無効化されたコマンド状態
- 日本語、正確な `⭐️`、Markdown構文、コードブロックのドロップダウン、複数行
- 入力、IME確定、選択、検索、コンテキスト、フォーカス、スクロール、ツールバー操作を含む連続RawInputフレーム

fixtureはホスト操作名、文書payload、検索や範囲の意味論、偽のKatanA効果を作ってはならない。Storybookのartifact、画像、GIF、動画は探索用およびKUC公開ルートの証拠に限られ、固定KatanAホストE2Eの証拠にはならない。

## 6. 実装計画

次の8段階を順番に進める。段階1のうち、実rootが計測した文書矩形を `TextSurface` へ伝播し、resize後にも論理スクロールを保持する限定変更だけは着手済みである。これはルート全体の配置、視覚品質、KatanA互換の完了を意味しない。

1. `katana-ui-core-egui-adapter/src/text_command_surface/` に、汎用ルートのスロット、計測viewport、z-order、クリッピング、衝突、フォーカス、IME、artifactの契約を定義する。
2. `katana-ui-core/src/molecule/command_chrome/` に、製品固有の名前やAPIを含まない、汎用コマンド群の識別子と取り付けポリシーを定義する。
3. `katana-ui-core-egui-adapter/src/text_surface/` で、計測された文書矩形、ガター、選択、注釈、IME、スクロール保持、ラスタライズ、アクセシビリティ出力を接続する。
4. `crates/katana-language-editor-egui/src/` のKLE bindingを、不透明なlease、show、転送、receipt、artifactの中継だけに縮小する。
5. `tools/kle-storybook/src/host_types_presentation.rs` の固定ルートサイズviewportと製品固有の組み立てを、汎用の不透明fixture入力へ置き換える。
6. `tools/kle-storybook/src/toolbar_projection.rs` と `tools/kle-storybook/src/floating_toolbar_projection.rs` で、取り付けポリシーを明示し、同じコマンド群が両スロットへ投影されないようにする。
7. `tools/kle-storybook/src/host_context.rs` と `tools/kle-storybook/src/storybook_document_search_*.rs` から、意味を持つホストpayloadの生成を除き、汎用状態とpresentationのfixtureだけを公開する。
8. `tools/kle-storybook/src/tests_*.rs`、`crates/kle-linter/`、`Justfile` に厳格なテストとゲートを追加し、公開ルートとsource-closure受け入れ経路を検証する。

## 7. 厳格な検証マトリクス

| 層 | 必須の証拠 | 失敗条件 |
| --- | --- | --- |
| KUC unit/contract | スロット一意性、計測矩形、重なりなし、衝突処理、フォーカス/IME routing、スクロール保持、コマンド群取り付け規則、アクセシビリティ順序、artifact順序 | 層の重複、古いviewport、重なり、または順序の非決定性 |
| KLE AST lint | KLE、egui adapter、Storybookに意味を持つ操作、検索、置換、パス、範囲、座標、ホスト効果APIがないこと | 禁止シンボル、対応付け、またはローカル描画が1つでも存在する |
| 公開ルートRawInput | 実egui context、連続フレーム、入力、CJK、IME、選択、検索、ツールバー、コンテキスト、フォーカス、スクロール | simulatorだけのcallback、またはフレーム間receiptの欠落 |
| 数値画像とmotion | 固定canvas、非空画素、日本語glyph、正確な `⭐️` の色付き表示、prefix/caret/gutter/helperの安定した矩形、一意なツールバー、宣言したスロット矩形 | 文字化け、`☆`に似た輪郭、ガタつき、古いgeometry、ツールバー重複 |
| KUC 3 OS | macOS、Windows、Linuxでの色付き表示、フォントファイルhash、`⭐️` と `☆` の分離、日本語とIMEの証拠 | profile欠落、fallbackだけの証拠、またはフォント/ラスタライズの同一性未解決 |
| 固定KatanA E2E | 読み取り専用の固定ソース、実際のnative入力、実際の文書・状態・ファイル・ホスト効果、OSごとのエディタ証拠 | Storybook、ソースだけ、simulator、またはKLE/KUCの偽ホストbridge |

固定KatanA E2Eは必須であり、Storybook、画像、動画、KUCテスト、source inventory、downstream診断で置き換えてはならない。

## 8. 完了条件

- 汎用KUCルートが、すべてのスロット割り当て、計測viewportの伝播、オーバーレイ方針、衝突、クリッピング、フォーカス/IME、アクセシビリティ順序、artifact順序を所有する。
- 明示的に異なる不透明コマンド群が供給されない限り、同じコマンド群を通常スロットと選択スロットの両方へ取り付けられない。
- KLEが、上記の不透明bindingと中継契約だけを含む。
- Storybookが、意味を持つホストfixtureなしで、独立したエディタ状態を使う実公開ルートを実行する。
- すべての検証行が再現可能なartifact付きで通過し、未解決のblockerがない。
- 固定KatanAのnative E2Eが、検索と置換を含む必要な機能について、機能ごとのホスト効果を証明する。
- macOS、Windows、Linuxが、正確な `⭐️` の色付きglyphと日本語/IME動作を証明する。

## 9. 非対象とリリースblocker

非対象は、固定KatanAの変更、KLE固有の座標補正、KLEとKDVへの二重の絵文字実装、fallback rendererの許容、Storybook artifactをnative host証拠として扱うことである。

source-closure artifactの欠落、execution recordの未完成、固定ホストbridgeの欠落、OS profileの欠落、絵文字表示の未解決、コマンド群の重複、古いviewport、レイアウトの重なり、入力のガタつき、文字化け、KLEの意味を持つAPI、strict AST/test/lintゲートの失敗のいずれかがあれば、リリースを停止する。現在の二重ツールバーと固定viewport artifactは、既知の視覚契約blockerである。本書は、それらを解決済みとは扱わない。

## 10. 2026-08-23 視覚監査による追加設計

2026-08-23の視覚監査では、KUC rootのforeground dropdownがartifact compositorの本文レイヤーに隠れる欠陥を発見した。KUCがforeground `Area`の物理inputと同じz-orderでtoolbar artifactを本文後に合成するよう修正し、open、item selection、closeを含む38-frame catalogueを2周する76-frame artifactを再生成した。最新artifactはGIF SHA-256 `ffdf967a7707399d8cac8e9758d22c95f15146b9efba08e41ff0e0c91722e67b`、MP4 SHA-256 `2e5b2fba97d07071f42656fa98d92adadbe42367b06477d180ecb7eab27d54c2`、KUC canonical manifest SHA-256 `7bb5b53c1be49cb83b7bf261624cc4b29d88fe1d6ec1f082a17477223d5c76a1`、manifest file SHA-256 `b07312749a5aed4fbcb7a125923f7c7c0fe35a58c35f43420b66c297311a4149`、76 frame/1280x720である。MP4は76 decoded framesを報告し、frame 031で17候補のgeneric language-choice dropdownが本文前景に表示されることを確認した。`⭐️`と`☆`の区別を含むこのartifactはKUC/KLE公開rootの限定証拠であり、完全な視覚監査、固定KatanAとの視覚互換、3 OS glyph proof、fixed-host互換の合格証跡ではない。

次の実装単位を、この順で分離する。

1. KUC: generic rootのslot mount identityを実装し、primary command chromeとselection overlayが同一command familyを二重mountできないことをroot contractで拒否する。KatanA操作名、ラベル、action IDは受け取らない。2026-08-23: `CommandChromeFamilyId`、additive versioned token、root pre-render rejection、retained rootのin-place family更新を実装した。legacy tokenでprimary/floatingが同居する場合は既定の同一familyとしてfail-closedにし、両slotを使うconsumerは明示的に別familyを渡す。family値は公開readback/error/debugに露出しない。adapter全144件、KUC core 226件、format、adapter strict clippyを通過した。これはgeneric KUC契約の完了事実であり、KLE binding、視覚品質、fixed KatanA host互換の証拠ではない。
2. KUC: text raster/layoutの可読性契約を追加する。実rootの全text slotが同一frameの実測metricsを使う接続までは完了した。実root矩形、gutter幅、行番号本文開始位置、最小文字高、line height、CJK幅、`⭐️`の色付きraster、scroll/resize後のanchorをreal-eguiとpixel artifactで検証する。KLEのfont値、座標補正、fallbackを追加しない。
3. KUC: generic selection overlayの表示条件、anchor、viewport clamp、outside/Escape、focus return、diagnostic suppressionをroot内に完成させる。初期選択を強制せず、同一frameのRawInputで検証する。
4. KUC: generic gutter/current-line/hover-line/diagnostic/search-match decorationを入力・raster・AccessKit・artifact契約に接続する。KatanAの診断型、検索range、handlerを導入しない。
5. KLE Storybook: KUCへopaque presentationを一度だけ渡すfixtureへ縮小する。未選択、選択、検索、診断、読み取り専用、context menu、scroll/resizeを別frame状態として供給する。短文の常時選択fixture、KLE座標、KatanA action ID、KatanA固有文言・変換・検索状態を持ち込まない。
6. KLE Storybook gate: layout occupancy、gutter、文字高、unique mount、floating距離、CJK/VS16 raster、frame間jitterを数値化し、旧frameを成功として受理しない。固定KatanA side effectはこのgateと分離したfail-closed host E2Eでのみ扱う。

現行の `tools/kle-storybook/src/config.rs` と `tools/kle-storybook/src/kuc_contract_checks.rs` には、旧editor configurationや座標検査をKLE側で構築する残存経路がある。公開rootの実行経路とKUC contractで代替できることを確認してから、KLEの既存テストを失わせずに削除し、ASTで再導入を拒否する。これらの削除前にKLE側のfont、gutter、surface座標を修正して視覚問題を覆い隠してはならない。

固定KatanA sourceに一般文書Replace/Replace AllのUI/action routeは確認できない。そのため、この設計はReplace/Replace AllをKLE/KUCに実装または有効化する根拠にせず、既存のrelease blockerを維持する。host仕様が別途確定しない限り、表示fixtureもgeneric unavailable capability以上を主張しない。

## 11. 2026-08-23 フルサーフェス再設計

### 11.1 監査結果と判定

固定Katanaは40行の初期表示要求、52pxの行番号・診断ガター、編集可能/参照専用の本文、現在行/hover行/検索一致/診断の装飾、選択位置に追従して開閉するauthoring toolbar、文書検索、階層context menu、スクロール、IMEを一体として持つ。根拠は固定sourceの`crates/katana-ui/src/shell/constants.rs:9`、`views/panels/editor/text_edit.rs:31-151`、`toolbar.rs:23-148`、`toolbar_popup.rs:14-110`、`context_menu.rs:9-193`、`views/top_bar/search.rs:13-136`である。

現行KUC scenarioは5行のfixture、6個の`Primary N`操作、固定selection、単発wheel/text eventに留まる。このため`frame-000.png`の本文占有率、gutter、authoring chrome、検索、入力遷移は固定Katanaの観測要件を満たさない。色付き`⭐️`の表示だけを合格根拠にしてはならない。

### 11.2 所有境界

KUCは次を所有する。

- generic document buffer、caret/selection、readonly、clipboard/history許可、IME preedit、viewport/scroll/resize、automatic numbered gutter、active/hover row、opaque marker/annotation、検索strip、primary/floating/context chrome、階層menu、focus return、root artifact/AccessKit/RawInput trace。
- opaque command target、localized label/icon/tooltip、capability、hostへのclosed event batch。KUCはtargetの意味、Markdown変換、保存、ファイル、画像payload、診断生成、preview同期を知ってはならない。
- full-surface scenarioのfixture文書、selection生成、入力stage、rasterと数値検証。KLEはこれらの文字列、range、座標、font、viewport、行番号幅を複製しない。

KLEはKUC leaseを一度だけ取得し、公開rootのshow、closed event batch、opaque receiptを一度だけ転送する。KLEはMarkdown authoring/search/diagnostic/document/host actionを解釈、保存、再現、テストfixture化しない。Katana固有のcommand targetから実効果への解決は固定host E2Eだけで証明する。

### 11.3 KUC full-surface scenario状態機械

`FullTextCommandSurfaceScenarioFactory`を、単なる静的presentationではなくKUC内で完結する連続RawInput state machineへ拡張する。すべてのstageはKUCが作り、KLEはstage payloadを読むことなく適用する。

1. `resting`: 40行以上のCJK/ASCII/Markdown構文/code block/正確な`⭐️` VS16を持つ未選択document、automatic numbered gutter、可視行/本文開始/metricsの記録、primary command chromeを描画する。
2. `selection`: actual pointer/keyboard inputで複数行selectionを生成し、その結果だけからfloating chromeを表示する。outside click、Escape、focus return、viewport edge clamp、readonly/annotation hover抑制まで同一root state machineで検証する。
3. `find`: query入力、no-result/result/active-result、previous/next/close、本文のgeneric match decorationとsummaryの同期を検証する。Replace / Replace Allはfixed Katana routeがない限り可視だがdisabledのgeneric capabilityに留め、mutationを起こさない。
4. `context`: actual secondary pointer、階層menu open/close、submenu、enabled/disabled、opaque click event、viewport clampを検証する。具体的なSave/Markdown/Image意味はfixtureにもKUC APIにも導入しない。
5. `readonly`: edit/cut/paste/history mutation拒否、selection/search/Accessibility許可、command capabilityのdisabled規則を連続stageで検証する。
6. `resize-scroll-ime`: 実screen rect変更、scroll後のvisible rows/anchor保持、scale変更後の再metrics、composition start/update/commit/cancel、CJK/`⭐️`混在caret/preedit、same-state jitterを検証する。

primary/floating/contextの各command inventoryは、KUCがgeneric group/opaque target/label/icon/capabilityとして保持する。Katanaの14 authoring triggerの有無とtarget-to-host効果は、KUC scenarioではなくsource-derived fixed-host E2Eの別の必須証跡とする。

#### 11.3.1 Generic language-choice dropdown

KUC の `CommandChromeDropdown` は、generic command に付随する opaque item
target、visible label、accessibility label、enabled state を保持し、primary
activation、pointer item selection、Arrow/Home/End/Enter、Escape、outside click、focus
return、viewport clamp、AccessKit record と one-shot closed event を同じ retained root
で所有する。KLE は item count、label、menu bounds、pointer、highlight、close reason、
selected value を read、retain、serialize、又は再構成してはならない。

fixed KatanA の code-block popup は `CodeBlockKind::all()` の17候補
(`text`から`sql`)を表示する。KUC full-surface fixtureは、この固定 source と同じ可視
17 label を **generic language-choice visual profile** として KUC 内だけに持つ。これは
KatanA enum、Markdown fence、host action、又は成功効果をKUCへ導入するものではない。
各 visible item はKUC内部でopaque targetに対応付け、consumerは閉じた一回限りのevent
transportだけを受ける。実hostではhost projectionが同じ形式のopaque targetとlocalized
labelを供給し、固定KatanAだけがtargetを`CodeBlockKind`とbuffer変換へ解決する。

scenarioはKUC-owned RawInput traceで、closed toolbar、primary open、17 itemの
AccessKit/record、Arrow navigation、Enter selectionによるmenu close、Escape close、
outside close、focus returnを個別frameとして検証する。選択後のfixture text mutation、
KLE callback count、KatanA action直呼びは不合格とする。full-surface GIF/MP4にはopenと
closeを含めるが、固定KatanAの17 code-kind効果、undo、dirty、preview、cursorはnative
host E2Eまで未達blockerのままとする。

2026-08-23のKUC実装では、menuをtoolbar child clipではなくKUC-owned foreground layerに
配置する。current-frame locatorのopaque click continuationはaim/press/releaseを別frameで
処理し、各frameでKUC内部のcurrent AccessKit targetを再解決する。これによりmenu配置の
確定後にboundsが変化しても、KLEがpointer又はgeometryを保持せずに実入力を継続できる。
KUC external-consumer contractはactual rootで、17候補のclosed/open、先頭
候補のpointer選択、ArrowDownで最終候補への到達とEnter選択、Escape後のtrigger focus
return/reopenを検証する。KUC retained rootはAccessKitを自律的に有効化し、full-surface
catalogueはtriggerとitemのaim/press/releaseをcurrent-frame target再解決で実行する。opened
foreground dropdownとclosed後のroot composite RGBA hashが異なることをKUC testで検証し、
GIF/MP4へこのtraceを追加済みである。fixed KatanA host E2Eへの接続は別の未達項目である。

### 11.4 数値・artifact受入契約

KUC artifact writerは連番`frame-NNN`のopaque root receiptのみを受け、scenario名はKLE独自stage名としてreceiptへ混入させない。scenario coverageはKUCが発行したopaque catalog/recordで検証し、KLEは名前、座標、内容、画像を生成しない。

full-surface gateは少なくとも次をfail-closedで検証する。

- resting artifactが40行以上のsource fixtureとautomatic gutterを持ち、表示viewport内の本文・gutter・primary chromeの非空領域とtext pixel heightをKUC record/RGBAから検証する。
- selection/find/context/readonly/resize-scroll-imeの各KUC stageが一度以上artifact、AccessKit record、opaque dispatch receiptに現れ、未選択restingでfloatingが不在である。
- selection transition、find active result、context submenu、readonly mutation拒否、resize/scroll/IME commit/cancelが、前後frame recordとevent batchで観測できる。
- `⭐️`と`☆`のraster cropが異なり、`⭐️` cropにchromatic pixelがある。日本語のglyph、行番号、caret、search decoration、floating anchorは同一metrics frameで検証する。
- GIF/MP4はKUCで符号化し、decodeしたframe数、canonical canvas、hash、stage provenanceを検証する。resize stageはroot artifact本来のviewport寸法とRGBA/hashを保持したまま、KUC writerが最大canonical canvasへ透明paddingして符号化する。KLEが画像をresize、letterbox、合成、またはstageを除外してはならない。PNGのみ、静的1枚、KLE生成画像、固定待機による動画は証拠にならない。

これらはKUC/KLE Storybookの受入契約であり、fixed Katana host parityの代替ではない。fixed sourceにある14 authoring trigger、context action、document search、reference readonly、scroll synchronization、host effectは、読み取り専用fixed sourceから導出したnative host E2Eで機能ごとに検証するまでrelease blockerのままとする。

### 11.5 実装順序

1. KUC scenario factory/state recordを11.3に合わせて拡張し、KUC real-egui/AccessKit/RGBA testsを先に追加する。
2. KUC motion writerの動画生成性能、resize artifactのcanonical canvas正規化、scenario catalog/provenance検証を修正する。
3. KLE Storybookはopaque KUC scenario relayだけを保ち、KUC stage catalogを全数実行する。KLE側のfixture/coordinates/semantic mappingをASTで拒否する。
4. KUC数値artifact gate、KLE contract/AST gate、動画decode gateを通してから、fixed Katana host E2Eの不足featureを一つずつ接続・証明する。

この順序のいずれかを飛ばしてKLE側に見た目・入力・Markdownの補正を入れることは境界違反である。

### 11.6 行番号ガターとauthoring chromeの可読性契約

KUC automatic gutterは、行番号rasterの自然幅だけをviewport offsetにしてはならない。numeric labelと本文が接触すると、2桁以上の行で本文先頭と行番号が視覚的に連結する。KUCはgeneric minimum hit columnと両側paddingを持ち、実測最大label幅にpaddingを加えた値とminimum columnの大きい方をgutter幅にする。40行scenarioではこのKUC contractが固定Katanaの52px column requirementを下回らないことを数値recordで検証する。KLEが52px、padding、gutter X、本文Xを渡すことは禁止する。

KUC generic command chromeは既存の`CommandChromeIcon` catalogを使い、icon-only display、tooltip/accessibility label、opaque target、selectionによるdisabled capability、group layoutを同一componentで所有する。scenarioはKatana固有のaction enumや実変換を持たずに、inline 4、heading 3、list 3、block 1、media 1の12個のgeneric rich-authoring affordanceを描画する。固定Katanaのbutton inventoryとhost effectの完全一致は、別途fixed-host E2Eでだけ合格にできる。

KUC real-egui/RGBA gateは、gutter texture boundsと本文texture boundsが交差しないこと、本文開始位置がminimum gutterより右であること、40行fixtureに2桁labelが存在すること、primary/floating chromeがopaque command inventoryを一度ずつ描画すること、VS16/CJK cropを保持することを検証する。2026-08-23に外部consumer形式のKUC contractで12操作すべてをAccessKitからKUC-owned opaque pointer requestへ変換し、対応するgeneric `CommandActivated`だけが一度dispatchされることを追加検証した。KLE Storybookはroot artifactを消費するだけで、これらのgeometryまたはアイコンを補正しない。

### 11.7 Selection transitionとfloating panel契約

KUC scenarioの`selection`はpresentation作成時の固定rangeだけでfloatingを可視化してはならない。resting frameから同じretained rootへKUC-owned physical pointer press/move/releaseを送り、selection record、floating open、anchor、viewport clamp、toolbar panelを順に観測する。各stage後にもdocument surface、gutter、primary chromeが維持されなければならず、pointer stageで本文が消える、surface sizeが縮む、floatingだけが残る場合はfail-closedとする。

floating command chromeはKUCがpanel background、border、corner radius、inner padding、action group spacing、tooltip、z-orderを描画する。panel boundsはtoolbar action boundsを余白付きで完全に含み、selection anchorの下方優先、viewport edge clamp、outside click/Escape/focus returnを同一record/RGBA/AccessKitで検証する。KLEはfloating frame、padding、corner radius、anchor、pointer座標を持たない。

fixed Katanaのauthoring toolbarに見た目と操作が一致するかは、KUC generic panel証拠の後にsource-derived native host E2Eで評価する。KUC scenarioのgeneric icon catalogやpanelは、そのE2Eを置き換えない。

### 11.8 Frame Stability Evidence Contract

入力中の文字のガタつきはKLE、Storybook動画、又は目視で判定しない。KUC retained root
だけが同一 frame の text layout、paint plan、final composite を対応付けられるため、数値
evidence は KUC 内部の test-only collector に閉じる。KLE が glyph、caret、gutter、search、
floating panel の座標や crop を readback する API は追加しない。

collector は各 public-root frame から次の opaque fact を取得する。

1. text content texture、automatic gutter label texture、primary/floating/search texture の
   KUC-private bounds と同一 metrics-frame fingerprint。
2. KUC authoritative caret/selection/preedit rect、viewport/surface bounds、current visible rows。
3. final root composite 上の Japanese、exact `⭐️` VS16、`☆` control の KUC-private crop hash
   と chromatic-pixel result。crop position/face path/pixel bytesは consumerへ公開しない。
4. root record、AccessKit、paint-plan、composite の同一 frame correlation。

同じ viewport/scale/catalog/retained state の frame を二回描画した場合、collector は本文 prefix、
line number、primary chrome、search decoration、caret の model bounds と texture bounds が完全一致
することを要求する。append、IME preedit update、IME commit、selection drag、scroll、resize は、
変更対象以外の同一 logical row/slot の bounds が不変であることを検証する。resize/scrollは
viewport又は可視 rowsの変化を許容するが、同一 retained state の再描画との差分を「jitter」として
隠してはならない。時間閾値、shape count、KLE callback、動画の目視、座標丸めの許容は不合格とする。

test matrix は resting 再描画、text append、Japanese IME preedit/commit、VS16/ZWJ selection、
selection floating open/close、find open/query/next/previous/close、readonly rejection、resize/scroll
の各 trace を持つ。各 trace は KUC public RawInput、same-root record、AccessKit、final composite
を同時に採取し、fixture又は stage 名だけの比較を禁止する。3 OS color glyph profile はこの local
stability gate と別に fail-closedであり、macOSの一件のraster testで代替しない。
