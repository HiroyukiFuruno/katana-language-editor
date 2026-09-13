# KLE v0.1.0 KUC Text / Emoji / Root 移行設計監査

## 監査範囲と結論

本書は実装前の境界監査である。KLE v0.1.0 は、KLE が汎用 UI を実装するのではなく、KUC の retained root を一度呼び、KatanA の意味論的 host との間で opaque な descriptor / one-shot transport だけを運ぶ構成に固定する。KLE の分割や helper 化で既存の汎用実装を温存する案は受け入れない。

現時点では移行開始条件を満たしていない。KLE-local surface と子 artifact 再集約が残り、KUC root の公開出力が child 出力を露出し、`⭐️` の三OS color-glyph 証明も不足している。

## 1. 現状の証拠と削除対象

| 現状の証拠 | 現在の責務 | 移行判断 |
| --- | --- | --- |
| `crates/katana-language-editor-egui/src/platform_text_surface.rs:17-27` | rasterizer、texture、drag anchor、IME preedit、focus/click 状態を KLE struct が保持 | ファイル全体を削除。別ファイルへ分割して残さない |
| 同 `:83-127` | KLE が rasterize、allocate、focus を直接制御 | KUC `TextSurface` root の opaque frame/event 呼出しへ置換 |
| 同 `:281-300`、`:303-389` | egui IME preedit/commit/disabled と cursor rect/output を KLE が処理 | KUC が IME、caret、AccessKit/IME output を所有。KLE は commit/event を一度転送 |
| 同 `:393-445` | pointer selection、drag anchor、raster bounds からの hit offset | KUC が grapheme-safe hit-test、selection、caret を所有 |
| 同 `:448-545` | background、selection/preedit、caret、texture upload の paint | KUC の raster/layout/paint root へ移管 |
| 同 `:667-736` | 行番号、diagnostic 色、gutter 幅・行座標を KLE が描画 | KUC generic line gutter/annotation に opaque marker/presentation を渡す |
| `crates/katana-language-editor-egui/src/kuc_artifact_aggregate.rs:14-24` | KLE が KUC child artifact を保持する aggregate | KUC opaque root frame 到着後にファイル全体を削除 |
| 同 `:53-102` | KLE が child order を検証し paint plan を再構成 | KUC 内部責務。KLE に `artifact_order`、paint plan、child artifact を公開しない |
| 同 `:118-129` | KLE が child output から再度 aggregate を生成 | KUC の single root frame API に置換 |
| `tools/kle-storybook/src/window.rs:16-21,40-43` | smoke の Minifb fallback 分岐を持つ | fallback launcher を削除し、KUC root の実 frame encoder だけにする |
| `tools/kle-storybook/src/window_renderer.rs:41-61,75-94` | KLE Storybook が child paint plan を取得し、shape count と fixture で editor を判定 | KLE Storybook の renderer/stat 判定を削除。KUC root frame record/hash/event を検証 |
| `tools/kle-storybook/src/window_renderer_fallback.rs:28-39`、`window_renderer_fallback_glyph_star.rs:6-23` | Minifb、固定矩形、手書き star pattern で画面と星を生成 | ファイル群を削除。`⭐️` の見た目を手描きで代替しない |

これらは「KLE 内で責務を分割すればよい」問題ではない。raster/cache、font selection、grapheme layout、IME、selection/hit-test、gutter geometry、popup placement は KDV と KLE の双方で再利用される汎用責務であり、KLE に残すと KUC/KDV と別実装・別キャッシュ・別入力状態が再発する。`openspec/changes/v0-1-0-language-editor-extraction/tasks.md:195-198,212,217,304-305` に記録された既存 AST failure は、まさにこの残置と fallback/shape-count 証跡を対象としている。

## 2. 境界の確定

### KUC が提供する generic root

KUC は次の部品を組み合わせた一つの retained root を提供する。KLE が child を直接 compose したり、child geometry を読むことはできない。

| KUC root / component | 必須責務 |
| --- | --- |
| `TextSurface` | UTF-8 text snapshot、grapheme/byte-safe selection、caret、IME preedit/commit、undo/redo、focus、scroll、pointer/keyboard hit-test、AccessKit、text/gutter/annotation frame |
| `PlatformFontCatalog` / color-emoji provisioner | platform font database と候補 font の一回限りの discovery/load、解決した emoji face identity と file SHA-256、catalog fingerprint |
| raster/layout runtime | shaping、Japanese、VS16、ZWJ、grapheme bounds、selection/preedit decoration、texture/RGBA、deterministic cache。surface ごとの text/selection/raster/texture state は共有しない |
| line gutter / diagnostics / decorations | line number、active row、diagnostic marker、range annotation、marker hit target、AccessKit。行番号や byte/range の意味変換は host descriptor から供給する |
| floating Markdown command chrome | KUC generic item/group descriptor、anchor、dropdown、focus、Escape/outside dismissal、viewport clamp、AccessKit、opaque activation target |
| `SearchStrip` | query/replacement input、IME、open/close、next/previous、replace-current/all の opaque event。match range、Markdown parser、document mutation は持たない |
| root frame/transport | child output、paint plan、egui ID、texture handle、child order を隠し、同一 frame の RGBA、record、pixel/plan hash、AccessKit snapshot、closed event batch を返す |

現在のKUCは `text_command_surface/types.rs:108-156` で child output と `artifact_order` を公開し、`composition.rs:20-168` で child adapterを呼び出して child output を返している。これは移行先の内部構造としては使えるが、KLE向け公開契約としてはまだ不十分である。KUC側で child graph を private にし、root frame または typed error のみを public consumer contract とする。

### KLE が保持できるもの

KLE の公開境界は、KatanA host descriptor、root identity/revision、opaque command target、opaque paste/search/content mutation transport、one-shot acknowledgement の受け渡しだけとする。transport は KLE が parse、clone、serialize、coalesce、再編集してはならない。KLE に許可するのは次の薄い adapter である。

1. KatanA の source-derived descriptor を KUC generic presentation descriptor へ変換する。
2. KUC の閉じた event batch を host bridge へ一度転送する。
3. host acknowledgement を次 frame の KUC snapshot として渡す。
4. KUC root frame の identity/hash/record を parity artifact に記録する。

KLE は `SearchQuery`、match list、byte range、`LineGutterModel`、Markdown operation enum、font family、cursor geometry、egui shape/paint plan、KUC child output を持たない。

### KatanA semantic host が保持するもの

KatanA host は Markdown parsing/syntax spans、document revision、byte/range semantics、diagnostics source、file/image clipboard acquisition、search/replace semantics、shortcut arbitration、preview/KDV/KRR effect、save/dirty/history acknowledgement を保持する。KUC はこれらを解釈せず、KLE は再実装しない。KatanA はこの監査の対象外であり、編集しない。

## 3. `⭐️`、日本語、IME、ZWJ の必須契約

入力の正規形を U+2B50 U+FE0F の二 scalar `⭐️` と固定する。U+2606 `☆`、text glyph、missing-glyph box、monochrome substitute への正規化・置換・fallback acceptance は禁止する。

KUC には現在、`katana-ui-core-text-raster` の `PlatformFontCatalog`、profile-specific
emoji candidate/pinned SHA policy、resolved face record と、same root の
`unicode_color_glyph_evidence` trace がある。2026-08-21 にこの macOS 環境で
`unicode_color_glyph_evidence_contract` の4 test（catalog/profile mismatch reject、VS16/
monochrome/indistinguishable crop reject、actual pinned root traceを含む）が通過した。
これにより KDV 固有の emoji lookup を KLE に複製する必要はない。

しかし release evidence としては次の不足が残る。

* macOS 以外の actual provisioned color face、raw file SHA、isolated crop を採取していない。
* Windows の Segoe UI Emoji と Linux の provisioned Noto Color Emoji を同じ runnerで
  fail-closed に解決した実行 record がない。
* evidence trace は現時点で legacy text-command root を対象とし、sanitized full editor
  root と KLE thin binding の同一 frame ではない。
* KLE/KDV の release artifact と三 OS manifest への結合、video decoder verification、
  actual KatanA host E2E がない。

KUC側で維持・拡張すべき契約は次の通りである。

1. `PlatformFontCatalog` は catalog key ごとの discovery/load と resolved face SHA を保持し、TextSurface、CommandChrome、ContextMenu、TabStrip が同じ catalog policy を利用する。surface state/cache は共有しない。
2. macOS は Apple Color Emoji、Windows は Segoe UI Emoji、Linux は pinned/provisioned Noto Color Emoji 等の実在 color face を解決する。family 名だけ、OS 分岐だけ、候補 path の存在だけでは合格にしない。
3. exact isolated `⭐️` raster crop、同条件 `☆` control crop、両者の bounds/hash/pixel delta、crop 内の chromatic pixels、resolved face identity/file SHA を三OSで記録する。
4. 同じ sanitized retained root に日本語 IME preedit/commit、`⭐️`、ZWJ（例 `👩‍💻`）を実 `RawInput` で通し、code point、grapheme range、caret/hit-test、AccessKit text、最終RGBAを検証する。IME preedit は表示用に保持しても host content を先取り変更しない。
5. 候補不足、face不在、face SHA変更、VS16が単色/空、ZWJ分割、grapheme境界不一致は全て typed error とし、KLE fallback、`SansSerif` substitution、platform skip、warning-only recovery を禁止する。

KDV にある `emoji_text`、OS emoji family lookup、text-raster/cache 相当は KDV/KLE に複製せず、この KUC `PlatformFontCatalog` / raster contract へ統合する。KDV固有の document/viewer semantics は移さず、emoji segmentation、font lookup、raster resource cache だけを共通化対象とする。既存の重複禁止と未完了条件は `tasks.md:198,212` に記録済みである。

## 4. KUC full-root Storybook と証明

KUCには `katana-ui-core-storybook/src/main.rs:23-64` の TextSurface/CommandChrome artifact 入口がある。しかしKLEのStorybookがこれをchild artifactとして再構成することは認めない。KUC側に、TextSurface、gutter/diagnostics/decorations、floating Markdown chrome、SearchStrip、context menu、root frame、IME、Japanese、`⭐️`、ZWJ、AccessKit を同じ retained root で表示・操作・記録する full-root page を置く。

KLEの `window_renderer.rs:41-61` のaggregate/compositor経路、`:75-94` のshape count/fixture判定、`window.rs:46-59,72-114` のMinifb smoke/fallback証跡は受入根拠から除外する。スクリーンショット/GIF/動画は表示確認とフィードバック用の補助資産に限定し、正しさは次の deterministic evidence で判定する。

* root frame の identity、dimensions、RGBA SHA-256、plan SHA-256、record hash、AccessKit snapshot が同一 RawInput trace で再現する。
* 各操作は leaf ID、入力列、state revision、event batch、host acknowledgement、frame hash を持つ。shape count、非空ピクセル、固定fixture、callback count は代替証拠にしない。
* Storybookは KUC root を直接起動し、KLE固有 host action、Markdown parser、fake fixture、手描き glyph、fallback renderer を持たない。
* visual asset は同じ root frame の encoder が生成し、manifest は profile fingerprint と exact crop/bounds/face SHA を含む。

## 5. 段階移行、削除、検証、rollback

### 実施順

1. **設計凍結**: KUC root API、catalog identity、opaque transport、frame record、三OS evidence schema を先に固定する。KUC契約未成立の間はKLE/KatanA実装を開始しない。
2. **KUC基盤**: catalog、raster/layout、TextSurface、gutter/diagnostics、CommandChrome、SearchStrip、root compositor、AccessKit、三OS color emoji proof をKUC側で完成させる。
3. **KUC Storybook**: full root page と real RawInput scenario、deterministic frame/artifact manifestをKUC側で完成させる。
4. **KLE adapter**: KLEは descriptor/transport adapterだけに縮小し、KUC rootを一回呼ぶ。KLE-local `platform_text_surface.rs`、artifact aggregate、direct input/context-menu workaround、fallback renderer を削除する。
5. **host E2E**: 実KatanA host の source-derived action/branch leaf ごとに、実入力から host effect までを検証する。action-only fixture、pending action、Storybook callbackだけの成功は不合格とする。
6. **release gate**: source closure、action origins、branch catalog、leaf manifest、execution record、storybook artifact、AST/lint、coverage、三OS実行証跡を結合し、未解決/重複/省略を fail-closed にする。

### 物理削除と禁止される置換

KUC root が受入条件を満たした後、KLEから次を物理削除する。

* `crates/katana-language-editor-egui/src/platform_text_surface.rs`
* `crates/katana-language-editor-egui/src/kuc_artifact_aggregate.rs` と対応 test/support
* `tools/kle-storybook/src/window_renderer_fallback*.rs`
* `tools/kle-storybook/src/window_renderer.rs` の child aggregate、shape-count、fixture判定経路
* direct text-surface input/context-menu workaround、KLE-local authoring helper、KLE-local font/emoji/raster/texture/gutter state

分割保存、`#[allow]`/exclude、test expectation の緩和、KLE内の暫定fallback、KDVからのコピー、KUC child artifact の再公開はrollbackではなく不合格とする。

### rollback / acceptance boundary

KUC root contract、三OS `⭐️` proof、KLE opaque transport、実host E2E のいずれかが欠けた場合は、KLEの旧実装をrelease branchへ戻して完了扱いにするのではなく、移行 batch 全体を未受入として止める。戻してよいのは未接続の設計資料・検証用生成物だけであり、旧KLE surfaceを新APIの横に二重維持してはならない。

受入は、全 source-derived leaf の実入力・最終状態・native effect・frame/AccessKit/hash が揃い、`just ast-lint` の13.2系が違反ゼロ、三OSのKUC証跡が揃い、KLEに禁止責務がゼロになった時だけ成立する。現時点では未達である。

## 6. 未解決の設計ギャップ

### 6.1 KUC host-root factory / KLE Storybook 接続ゲート

KLE Storybook を KUC root へ接続するには、KUC が retained root の生成、保持、
`show`、root frame record、RGBA artifact encoder、閉じた event batch を一つの
consumer-safe API に隠蔽しなければならない。現在の `KucRootBinding` は、既に
生成済みの `EguiTextCommandSurfaceRootOutput` の one-shot event を転送するだけで
あり、root を生成・保持・表示する API ではない。この状態で KLE Storybook が
`EguiTextCommandSurfaceRoot`、child adapter、artifact compositor、frame pixels を
直接組み立てることは禁止する。

次 batch の公開境界は次の形に固定する。

1. **KUC** は opaque な revisioned presentation descriptor から generic retained
   root を生成・更新し、KUC 内部で TextSurface、gutter/diagnostics、floating
   CommandChrome、SearchStrip、ContextMenu、IME、AccessKit、raster/catalog を保持する。
2. **KUC** は root を一回表示する consumer API として、closed root record と
   non-Clone/non-Serialize one-shot event transport だけを返す。child output、paint
   plan、texture、egui ID、geometry、range、search text、command item/event enum は公開しない。
3. **KLE** は KatanA-derived semantic descriptor を KUC の opaque presentation
   descriptor に写像し、KUC root を一度呼び、transport を host bridge へ一度だけ
   forward する。KLE Storybook は KUC が出力した MP4/GIF/PNG/manifest の path/hash/
   frame receipt を記録できるが、pixel を再合成、検査、描画しない。
4. **KUC Storybook** は full-root artifact encoder と MP4 decoder verification を
   所有する。KLE Storybook は別 renderer、Minifb fallback、shape count、handwritten
   glyph、fixture canvas を持たない。実KatanA host resolution は後続 host E2E の責務で
   あり、KUC/KLE Storybook callback を最終 effect の証拠にしない。

#### Input / output の非対称性

ここでいう opaque は **root output と event transport** に適用する。KLE が有効な
generic root を生成できない opaque byte token だけを公開すると、KatanA を編集しない
条件では runtime consumer が成立しない。そのため KUC は次の限定した public input
projection を提供する。

* KLE は一時的な KUC generic projection を渡せる。projection は TextSurface の
  document snapshot、generic command/search/context presentation、theme/style、opaque host
  target token からなり、KatanA action、Markdown enum、search range/match、byte/line、
  geometry、child state を含まない。
* KUC が projection から retained root を生成・同期し、すべての child/state/layoutを
  所有する。KLE は projection を root 間で保持、解析、merge、逆シリアライズしては
  ならない。
* root frame output は closed record と non-Clone/non-Serialize one-shot event transport
  だけである。KLE は record の identity/revision/hashを記録しうるが、RGBA、paint plan、
  texture、AccessKit node、child output、event payload は取得できない。

KUC の opaque byte token は Storybook artifact や host-provided external projection
の input carrier として残してよいが、KLE runtime を成立させる唯一の root construction
route にしてはならない。この projection / closed-output contract を KUC compile contract
と KLE source-boundary test の双方で固定する。

#### Sanitized projection schema

既存の `TextSurfacePresentation`、`CommandChromeSearchPresentation` を public root
projection として再公開してはならない。これらは UTF-8 byte offset、annotation、
gutter、scroll/focus request、search result count/index を含み、KLE が保持してはならない
input state を逆流させる。KUC は root 専用の sanitized projection を新設する。

| projection field | KLE が一時的に渡せる内容 | KUC が保持/解釈する内容 | 禁止 |
| --- | --- | --- | --- |
| document snapshot | revisioned UTF-8 snapshot と opaque document identity | grapheme layout、selection/caret、IME、scroll、undo、gutter/annotation、AccessKit | byte/line/range、selection、scroll/focus request、gutter marker |
| command presentation | opaque host-target、localized text/icon/capabilityだけの group/item projection | toolbar/dropdown/context lifecycle、anchor、focus、placement、activation event | Markdown enum、command id switch、geometry、typed item event |
| search presentation | opaque request/replacement target、localized label、boolean capability、opaque summary text | query/replacement input、focus、next/previous/replace controls、event transport | match array、result count/index、byte conversion、content mutation |
| style/theme | KUC-owned style key/token reference | font catalog、raster/cache、RGBA、texture、pixel/accessibility record | font family/path/SHA selection、color literal、pixel/texture handle |

Factory `retain_projection` / `synchronize_projection` はこの schema だけを受け、root
`show` は **one call per consumer frame** の closed facade frame を返す。retained editor
はフレームをまたいで再描画されるため、root lifetime 全体で一度しか `show` できないという
誤った制約は置かない。KLE binding は各 consumer frame で一度だけ呼ぶことを runtime testで
証明する。

#### Projection API の段階的確定

v0.1.0 の KUC factory は document-only foundation から開始し、同じ
`SanitizedEditorProjection` に command、search、context、tab の各 generic sub-projection
を追加する。各 sub-projection は `revision` と opaque host target を持つが、KLE が
command の意味を列挙又は分岐できる ID は持たない。KUC は KLE に child component を
公開せず、sub-projection を受け取って retained child lifecycle を更新する。

| sub-projection | KLE から渡せる値 | KUC-only retained state | 受入テスト |
| --- | --- | --- | --- |
| document | opaque identity、revision、UTF-8 snapshot、style key | caret/selection、IME、undo、scroll、gutter、AccessKit | identity/stale/conflict、Japanese/IME/ZWJ、one root show/frame |
| command | opaque target、group/item順序、localized label/tooltip、KUC icon props、enabled/visible capability | toolbar/dropdown/floating/context anchor、focus、hit-test、keyboard/AccessKit action | pointer/keyboard/AccessKit activation、single-consumption transport |
| search | opaque action target、localized label、enabled/visible capability、opaque summary | query/replacement TextArea、focus、next/previous/replace state | actual RawInput query/replacement、close、one-shot event、no match leakage |
| tab | opaque tab/group/placement/swatch target、localized label/icon/capability | active/focus/order/overflow/menu/drag/inline rename/palette | pointer/keyboard/AccessKit event、stale/no-op host bridge |
| context | opaque target、localized label/icon/capability、generic ordering | open/close/submenu/overflow/focus/placement | actual context input、nested menu、outside/Escape、single-consumption transport |

KUC 以外の crate はこの表の public input を一フレームの呼出しにだけ使い、projectionを
保持、比較、再送、ログ出力してはならない。KUC record には root-level revision/hash/
dimensions/accessibility hashだけを残し、sub-projectionの選択値、入力値、range、geometry、
RGBAを残さない。`SanitizedEditorProjection` の全 sub-projection が実装されるまでは
document foundation又はKUC Storybookだけを full-editor evidence と呼ばない。

KUC の KLE-facing re-export から `EguiTextCommandSurfaceRootOutput`、child output、
artifact/paint-plan/texture 型を除外する。Storybook feature 内部だけが RGBA encoder を
利用でき、KLE public/runtime dependency は同じ型に到達できない。既存 opaque byte token
route は external host injection と Storybook artifact 用の補助経路として保つ。

#### KLE root cutover interface

KLE の旧 `EguiLanguageEditor` は `TextContent`、selection、cursor、query/match、gutter、
scroll/focus request、authoring/menu、KUC child output、artifact aggregate を同じ retained
state に持つ。この形の field を残したまま `SanitizedDocumentRoot` を横に追加する移行は
禁止する。KLE の cutover 後の egui adapter は、KUC retained root 一個と、現在の host
projection をそのフレームだけで KUC に消費移譲する入口だけを持つ。

```text
host-created SanitizedDocumentRootInput (one frame, consuming)
  -> KLE `synchronize` and exactly one `show`
  -> KUC `SanitizedDocumentRootFrame`
  -> KLE exactly one `forward_events_once`
  -> host bridge
```

この interface における不変条件を次で固定する。

1. `SanitizedDocumentRootInput` と各 sanitized sub-projection は egui adapter crate の
   call parameter としてのみ移動する。KLE は field、getter、clone、serialize、debug log、
   retry queue、artifact/manifest に保存しない。snapshot、label、icon、capability、opaque
   target を KLE が再構成又は比較してはならない。
2. KLE が保持できる runtime object は KUC `SanitizedDocumentRoot` のみである。KLE は
   child adapter、`EguiTextCommandSurfaceRootOutput`、`TextSurfaceFrameRecord`、artifact
   aggregate、egui response、paint plan、texture、pixel geometry を保持又は公開しない。
3. `show` の戻り値は closed record と one-shot forwarding 以外を持たない。KLE は record
   を host に返す際も identity/revision/hash/dimensions/accessibility hash だけを扱い、
   event count、event kind、target、input payload を取り出さない。forwarder は同一 frame
   に一度だけ呼び、二度目は KUC の `AlreadyConsumed` をそのまま失敗として返す。
4. input revision と identity が KUC `synchronize` で reject された場合、KLE は
   `EditorError::Internal` へ一度だけ変換して return する。legacy renderer、blank frame、
   direct `egui` child、synthetic event、旧 event queue への fallback は禁止する。
5. public `LanguageEditor` の旧 content/search/selection/scroll/authoring/gutter control は
   同じ migration batch で opaque host-frame injection と one-shot host bridge に置換する。
   KatanA は read-only であるため、KLE だけの current callers は移行中に compatibility
   completion を名乗れない。KatanA mounted E2E は host adoption 後の release blocker とする。

この cutover は KLE に UI を移す設計ではない。KLE が `SanitizedDocumentRootInput` を
生成する必要がある場合も、host-issued descriptor を KUC generic input へ一方向に移す
だけであり、KLE 自身の enum、検索、range/line/pixel 変換、action mapping、rendering を
導入してはならない。型の公開場所は KUC egui adapter に限定し、neutral
`katana-language-editor` crate は KUC/egui 型を再 export しない。

#### Command batch の実装確定と次段の設計ゲート

2026-08-21 の KUC command projection batch では、次の実装と証拠を確認した。

* `SanitizedDocumentRootInput` は `Option<SanitizedCommandProjection>` を consuming
  builder で受け取り、projection を外部から読み戻せない。
* KUC 内部 mapper が generic `CommandChromeToolbarPresentation` を生成し、同じ
  retained root の toolbar presentation に接続する。KLE/KatanA/Markdown の意味分岐はない。
* 同一 revision の snapshot/style に加え、projection の KUC-private deterministic
  fingerprint を比較し、差分を `RevisionConflict` として拒否する。
* 実 egui root を二回描画し、新しい command projection によって closed root record の
  `record_hash` が変化することを確認した。`sanitized_` filter は 21 tests passed、
  adapter Clippy `-D warnings`、rustfmt、空白検査を通過している。

この証拠は command toolbar の実接続だけを示し、full editor 完了を示さない。次段の
search/replace と context menu は既存 KUC child 経路を再利用する。`CommandChromeSearchStrip`
と `CommandChromeSearchPresentation` は内部だけで構成し、`ContextMenuPresentation` も
内部だけで構成する。sanitized input は各 child の opaque target、localized text/icon、
capability、KUC 内部で表示する UTF-8 value を一時的に受けるが、match count/index/range、
document mutation、child output、geometry は受けない。search query/replacement の値は
KUC 内部でのみ保持し、KLE の projection に読み戻しAPIを設けない。

##### Command activation transport: mandatory ownership model

2026-08-21 の command projection audit では、KUC は group/item/dropdown の表示順、localized
text/icon、enabled/visible、KUC-private structural widget ID を構成できる一方、current root transport
は generic `CommandChromeToolbarEvent` / floating event を残している。public transport 自体は opaque
でも、このままでは host-projected `SanitizedCommandTarget` と current revision/correlation を一回だけ
解決する capability がなく、KLE が structural ID や command event を解釈して host action を決める
余地が残る。この表示-only foundationを Markdown authoring 又は image ingest 完了として扱わない。

採用する契約は search transport と同じ ownership model である。

1. `SanitizedCommandTarget` と `SanitizedCommandDropdownItem` は current host projection 作成時に
   generic non-text `FnOnce` activation capability を所有権付きで受け取る。KUC は target bytes、
   capability、structural widget IDを public accessor/Debug/manifest に出さない。
2. KUC retained root は toolbar、floating toolbar、dropdown、context の physical pointer/keyboard/
   AccessKit inputを KUC-private structural IDから original opaque targetへ戻し、generic
   `SanitizedCommandActivationTransport` だけを生成する。KLE は command id、label、group、icon、
   Markdown/code-kind enum、dropdown indexを readback又はswitchしない。
3. raw `CommandChromeToolbarEvent`、`FloatingCommandToolbarEvent` と command/dropdown child eventは
   sanitized command activation に正規化した後、generic root batch/fingerprint/cardinality/outer
   forwarderに残してはならない。tab/search/non-command eventsはそれぞれの canonical transportに
   残す。
4. root event batch は forwarding 前に消費し、generation check後に command capabilityを frame orderで
   一回だけ実行する。callback rejection、wrong capability kind、outer forwarding failure、stale/duplicate
   は no-replay/no-mutation とし、KLE retry queueを禁止する。
5. KUC generic contract testは localized Japanese/`⭐️`/ZWJ labelsを含む enabled direct actionと
   dropdown action、disabled/hidden/unknown target、outside/Escape/focus return、callback rejection、
   stale/duplicateを physical RawInput/keyboard/AccessKitで検証する。KatanA の14 Markdown commandと
   17 code-kindの全 host effectは、後続の host projection/host E2Eで個別に結合する。

KUC に `AuthorMarkdown`、`CodeBlockKind`、image path、clipboard payload、KatanA actionを導入しては
ならない。KLE-local target-to-action map、closure factory、command string、fallback itemも不適合である。
この command activation contract と context menu の同型 transport が成立するまで、KLE Storybookの
toolbar/dropdown表示を full Markdown editor evidenceと呼ばない。

各段の完了条件は次の通りとする。

1. search/replace: query と replacement の IME/raw input、open/close、next/previous、
   replace-current/all を同じ retained root で実行し、closed root event transport と
   record hash を検証する。match array や byte range の漏洩テストも追加する。
2. context menu: existing `EguiContextMenuAdapter` の open/submenu/outside/Escape/
   type-ahead 経路を sanitized generic projection へ写像し、opaque target の一回限り
   event forwarding と record hash 差分を実 egui で検証する。
3. tab: KUC core の `workspace_tab_bar` は nested `parent_group_id` を持つ generic retained
   modelへ拡張済みで、sanitized adapter は current frame の physical pointer activation を
   KUC-private render id -> original opaque target route table で closed root event に変換する。
   これは flat group / single `group_id` だった旧差分を KLE で補うものではない。現段階の
   pointer activation evidence は foundation に限られ、overflow/menu/drag/rename/recolor/
   close/collapse の全 family、keyboard/AccessKit、host effect は未完である。
   `SanitizedTabGroup` は opaque group target と group capability を必須にし、label/order/
   icon から target を推測しない。tab response、group id、egui response、opaque target を
   tuple 又は public adapter API として外へ返してはならない。既存部品の単なる再 export や
   KLE-local tab 実装は禁止する。

#### TabStrip interaction-family design gate

select/collapse だけの TabStrip は KatanA reference の tab behavior を満たさない。KUC は
close、previous/next、overflow、close variants、pin/restore、context menu、drag reorder/group
move、group create/add/remove/rename/recolor/ungroup/close/collapse の generic interaction
familyを retained root 内に持つ。各 visible operation は次の input/output 条件を満たすまで
KLE Storybook または parity evidence に含めない。

| contract | KUC public sanitized input | KUC-private runtime | closed transport |
| --- | --- | --- | --- |
| tab close/pin/restore/select/traversal | opaque tab target、capability、localized visible/tooltip/accessibility text、descriptor revision | close affordance、keyboard/AccessKit alternative、focus/overflow/hit test | opaque tab target、generic operation kind、revision/correlation |
| group menu/rename/recolor/ungroup/close/collapse | opaque group target、capability、localized action/input/accessibility text、opaque swatch targets | menu、inline IME input、palette、outside/Escape、drag/overflow | opaque group/swatch target、generic operation kind、revision/correlation。group nameだけは non-Clone/non-Serialize one-shot value |
| drag reorder/group move | opaque moving/placement/group target、capability、localized accessibility text | pointer geometry、keyboard/AccessKit alternative、no-op detection、collapsed auto expand | opaque moving/placement/group target、generic operation kind、revision/correlation |

`SanitizedTabCapabilities::close_state(true)` や group capability だけから KUC が English
`Close`、host action、target、color value、order/index、path、group membership を推測するのは
禁止する。visible label、tooltip、accessibility text、empty/error textは generic localized
presentation として host projection から KUC へ渡す。KLE はこれらを読まず、`egui::Button`、
context menu、inline input、palette、drag coordinates、shortcut map、semantic operation switchを
持たない。KUC internal structural render ID は event routing のみに使い、transport/receipt/
Debug/Storybook manifest に出さない。

host effect を伴う close/pin/restore/reorder/group 操作で、KUC が current projection を
`CloseTab` のように先行 mutate してはならない。close affordance は generic non-mutating
intent を一回だけ閉じた transport に入れ、tab は host が次 revision の projection で除去又は
変更するまで保持する。dirty confirmation、force-close、persistence、document selectionは
KatanA host のみが決める。KUC retained transition は menu/focus/hover/inline editなど純粋な
generic UI state に限り、host operation success の偽装に使わない。

各 family の acceptance は pointer、keyboard、AccessKit の physical input と、same-root
closed transport、stale revision/correlation、disabled/hidden、outside/Escape、nested group、
overflow/no-op、opaque leak guard を必要とする。KatanA の close confirmation、persistence、
filesystem/document mutationは host E2Eだけで検証し、KUC core action の直接呼出しは acceptance
evidenceにしない。

#### Search and replace interaction-family design gate

`SanitizedSearchProjection` の label/capability/target だけを root record に反映することは、
検索又は置換の実装ではない。KUC retained root は query/replacement の text area、IME/preedit、
open/close、focus、Escape、next/previous、replace-current、replace-all、disabled/empty state、
current match summary と AccessKit を generic UI state として所有する。KatanA host は query
解釈、regex、match/range/index、wrap、scroll source、replace byte-boundary conversion、dirty、
undo、refresh を唯一所有し、KLE はいずれも保持・計算・readbackしない。

| contract | KUC public sanitized input | KUC-private runtime | closed transport |
| --- | --- | --- | --- |
| open/close/next/previous | operation ごとの opaque target、localized visible/tooltip/accessibility text、enabled、descriptor revision | visibility、focus、button/key/AccessKit routing、current query IME state | opaque target、generic operation kind、revision/correlation |
| query/replacement entry | field ごとの localized label/accessibility text、input capability、descriptor revision | UTF-8/IME/preedit/caret/selection と transient field value | target/revision/correlation と non-Clone/non-Serialize single-consumption value。KLE は value を観測・clone・log・serializeしない |
| replace-current/all | opaque target、localized text、enabled、host-projected current-match summary | empty/disabled validation、focus、physical trigger、pending UI state | opaque target、operation kind、revision/correlation と必要な one-shot input value。range/match/count は含めない |

KUC は `Search`、`Replace`、`Find`、empty/error text をハードコードせず、表示可能な文言を
generic host projection から受ける。KUC が query/replacement、label、tab target、visible order
から KatanA action、document path、range、match index、regex、replacement payload を導出することは
禁止する。KLE は `SearchControl`、regex parser、query/match/replace state、string fallback、
`TextEdit`、座標、host result cache を持たず、consuming request を同一frameで一度だけhost bridgeへ
forwardする。host が受理又は拒否した後の次 descriptor だけが KUC state を更新できる。

##### Search option state synchronization gate

検索 option は activation target だけでは controlled input にならない。host-projected search
descriptor は generic boolean `match_case`、`whole_word`、`use_regex` の current state を、各 option
の opaque target/capability と同じ identity/revision/correlation に結合して渡す。KUC はこの state を
初期表示と accepted host acknowledgement 後の再同期に使い、toggle直後の一時 UI state 以外を
独自に authoritative state として保持しない。KLE は option value、default、toggle cache、option
meaning を持たず、host projection を読む/書く compatibility mapping を実装しない。

`use_regex` が false 固定の presentation、または target 有無だけで checked state を推測する実装は
不適合である。state なしの target は unavailable とし、同 revision で state/capability が変化した
projection は existing revision conflict contractで拒否する。KUC test は host-projected initial state、
physical pointer/keyboard/AccessKit toggle、one-shot transport、next revision acknowledgement、
stale/duplicate/no-target/read-only、Japanese/`⭐️`/ZWJ Debug leakを個別に検証する。KatanA search
calculator が現時点で option を無視する事実は、visible toggleの projection/transport契約を省略する
根拠にしない。

KUC は既存の `CommandChromeSearchEvent` と text-area change をそのままroot transportに
出してはならない。同じ query/replacement を二重搬送すると host の二重適用・二重ログを招く。
root 内部で一度だけ `SanitizedSearchEventTransport` に正規化し、operation別 opaque target、
root revision/correlation、必要なときだけnon-Clone/non-Serialize text valueを閉じ込める。
match range、matched text、regex結果、document offset、selection、host replacement resultは
transportに含めない。transportはforwarder呼出し前に消費状態へ遷移し、forwarderが失敗しても
再送を許可しない。この fail-closed 規則はテストし、host は失敗を観測した次投影で再試行UIを
決める。KLEがtransportをclone、serialize、Debug出力又はreadbackする経路はASTで拒否する。

ただし「opaque」であることは host effect を不能にすることを意味しない。KUC が KatanA の
query/range/action を知ることも、KLE が文字列を解釈することも禁止する一方、host は current
projection 作成時に登録した one-shot resolver capability で、同じ opaque target/correlationを
一度だけ解決できなければならない。この resolver は KUC generic trait として注入され、KUCは
payloadをKLE public APIへreadbackせずに所有権ごと消費してhost callbackへ渡す。KLEは resolver
registry、target-to-action table、query/replacement cache を保持しない。KatanA host だけが登録時の
semantic target と受け取った one-shot text を照合し、検索・replace host effect又はno-mutationを
決定する。opaque transportを受け取るだけで読取り不能な `RecordingForwarder` はtransport境界の
unit testには使えるが、検索/replace feature又はhost E2Eの完了証拠には使えない。

この capability が無い段階で、KUC event transportを「検索対応」と表示してはならない。先に
generic resolver lifetime、stale/replayed correlation rejection、forwarder failure時の消費済み規則、
host callbackの一回性、KLE AST prohibitionを設計・実装し、次にKUC physical input、KLE one-shot
forward、KatanA actual host effectを同じ leaf recordへ結合する。

##### Search resolver transport: mandatory ownership model

2026-08-21 の実装監査で、`CommandChromeSearchEvent` が generic root payload と
`SanitizedSearchEventTransport` の双方に残り、`⭐️` の IME commit 一件が forwarding receipt
で二件として計上されることを確認した。これは opaque 化で隠せる問題ではなく、同一 host
effect を二回実行し得る release-blocking boundary defect である。検索 family の root transport
は次の不変条件を同時に満たさなければならない。

1. `CommandChromeSearchEvent` は root 内で一度だけ sanitized search transport へ正規化する。
   正規化済み search event を generic root payload、root fingerprint、cardinality、下位
   forwarderへ残してはならない。text/toolbar/floating/context-menu の非検索イベントは従来の
   generic root transport に残す。search 一件だけの root frame の receipt cardinality は一件である。
2. host は current projection 作成時に、operation target ごとの generic one-shot capability を
   KUC input として登録する。capability は KatanA action、document path、range、query history、
   regex、match index を KUC に公開しない。KUC は target ごとに閉じた capability を所有し、
   routing 後の operation kind と必要時の one-shot text を所有権ごと一回だけ capability に渡す。
3. KLE は root を `show` し、閉じた transport を一回だけ forward する機械的 consumer である。
   target bytes、operation switch、query/replacement bytes、resolver registry、retry queue、
   action tableを公開 API、state、Debug、log、Storybook fixture に保持又は readback してはならない。
4. capability の実装は host が保持する current projection の revision/correlation を照合し、
   stale、cross-document、duplicate を no-mutation として拒否する。KUC は拒否を成功と偽装して
   retained document/search state を更新しない。host の次 projection のみが visible result を更新する。
5. root event batch は forwarder 呼出し前に消費済みに遷移する。capability/forwarder failure でも
   元 transport、text、target は再取得・再送できない。failure は opaque typed failure として
   上位へ戻し、KLE が payload を復元して retry する経路を作らない。

この capability は search 固有の KatanA command enum ではない。KUC は generic text-input / unit
operation の one-shot consumer contract を提供し、host-projected target が具体的な KatanA search,
replace, close 等の意味を閉じ込める。capability の object-safe Rust API、host error の非漏洩表現、
projection lifetime、text / unit operation の型分離は実装前に KUC existing callback pattern と照合し、
本節の五条件を満たすものだけを採用する。KLE-local resolver、KLE/KDV duplicate handler、public
payload accessor、generic string command は設計不適合である。

この修正の KUC contract test は、Japanese / exact `⭐️` VS16 / ZWJ の IME commit、replacement
commit、next/previous、option、replace-one/all、close をそれぞれ physical RawInput から起動し、
同一 root revision で registered capability がちょうど一度だけ消費されることを検証する。各 case
は generic root transport に search raw event が残らないこと、receipt cardinality が event 数と
一致すること、replay/stale/failure が no-mutation かつ再送不能であること、Debug/manifest に raw
text が出ないことを要求する。`RecordingForwarder` 単独はこの contract の代替にならない。

##### Search resolver transport: selected Rust contract

capability registry を root 又は KLE に置く案は採用しない。registry は target の意味、document
scope、lifetime を root consumer に持ち込み、cross-document の再解決経路を作るためである。KUC
`SanitizedSearchTarget` は opaque target signature に加えて、host が current projection の作成時に
所有権を渡す非公開 one-shot capability slot を保持する。slot は event transport へ shared handle
として移り、KUC 内部の `try_borrow_mut().take()` だけが capability を取得できる。KLE public API は
target bytes、slot、operation、text の accessor を持たない。

capability は KatanA action ではなく、KUC generic search interaction の二種類だけを受ける。

| capability input | generic variants | host-only semantic resolution |
| --- | --- | --- |
| text operation | query changed、replacement changed、replace-one、replace-all と one-shot UTF-8 text | current opaque target/revision/correlation を照合し、search/replace effect、dirty/undo/scroll/refresh を実行又は no-mutation で拒否する |
| unit operation | close、previous、next、match-case/whole-word/regex changed とその boolean value | current opaque target/revision/correlation を照合し、search navigation/option/close effect 又は no-mutation を決める |

両 capability は `FnOnce` 相当の object-safe consumer として target ごとに登録する。KUC は callback
の具体的 error、KatanA type、action、path、range を知らない。host callback は成功又は KUC-defined
opaque `HostEffectRejected` だけを返し、source error text や boxed application error を KUC receipt、
KLE error、Debug、manifest に混入させない。KatanA host が詳細を記録する必要がある場合も、その
record は KatanA host 内部に限定する。`Rc<RefCell<Option<...>>>` を使う場合は reentrant borrow を
typed opaque rejection とし、callback を呼ぶ前に `take()` して必ず at-most-once にする。

`SanitizedDocumentRootFrame` は root identity、revision、root generation を KUC-private shared
generation cell とともに保持する。`synchronize` がより新しい revision を受理した時点で generation
を更新し、古い frame の `forward_events_once` は capability/outer forwarder を一切呼ばず
`StaleFrame` を返す。identity が異なる root は frame を共有しない。same-revision の異なる
projection は既存どおり `RevisionConflict` とし、同じ target signature であっても capability slot
を差し替えない。これにより host projection 以外の target byte 再利用では capability を発火できない。

dispatch order は (1) root event batch と tab/search batch を consume、(2) generation を検証、
(3) sanitized search capability を frame order で一回ずつ実行、(4) 成功時のみ残る非search root
transport を opaque outer forwarder へ一回渡す、(5) receipt を返す、に固定する。一つの frame の
複数 capability の途中拒否は rollback しない。先行 effect の再実行を禁止し、host は次 projection
で整合した UI を返す。capability 成功後の outer forwarder failure、capability rejection、borrow
conflict のいずれでも frame は消費済みで、KLE/KUC は retry queue を作らない。この at-most-once
failure contract と raw search channel detach は同一 KUC migration batch とする。

KLE は callback を構築又は semantic switch してはならず、KatanA host-projected generic KUC
capability を opaque input として retained root へ渡すだけとする。KLE-local closure、query cache、
target-to-action map は AST failure とする。KatanA read-only reference への実接続は後続の
KLE-owned host-E2E harness で行い、capability callback counter だけを host effect evidence にしない。

受入試験は open/close、empty/nonempty、IME/preedit/commit、Enter/Shift+Enter、next/previous、
replace-current/all、disabled/read-only、stale revision、Escape/focus、pointer/keyboard/AccessKit、
one-shot replay rejection、KLE debug/manifest leak guard を個別に持つ。KatanA host E2E は Japanese、
exact `⭐️` VS16、ZWJ を含む buffer で current/all/no-match/read-only/stale/document-switch/dirty/
undo/scroll を個別に検証し、直接 `ReplaceText`、pending action、fixture state は証拠にしない。

search/context の実装が済むまで、Storybook の search/context/tab 表示や動画を
full-editor evidence と呼ばない。tab adapter が成立するまで、tab gap は未達として
明示し、command batch の証拠で代用しない。

この API と KUC contract test が成立するまで、`tools/kle-storybook` の旧経路を
KLE-local root に置換してはならない。KLE の `platform_text_surface.rs`、aggregate、
fallback renderer の物理削除は、KUC host-root factory、KLE thin consumption、actual
KatanA host E2E が揃った同一 migration batch でのみ実施する。

* KUC child output/paint planを隠した sanitized single root frame API は foundation と
  command projection、tab の select/collapse/close-request foundation まで実装済みである。
  search、replace、context menu、tab context menu/drag/group operation、full editor component
  family、tab の全操作、KLE thin consumption は未成立である。
* catalog単位の一回限り discovery/load、resolved face identity/file SHA、Linux/Windowsの実color face provisioningが未成立。
* isolated `⭐️` versus `☆` crop proof と日本語/IME/ZWJの三OS retained-root evidenceが未成立。
* KUC full-root Storybookとroot frame manifestが未成立。KLE Storybookは現在もfallback/shape-countを含む。
* KLE actual host E2E、source-derived leaf全件、AST違反ゼロ、三OS execution recordは未成立。

本書は設計台帳である。KatanA は read-only のまま、KUC/KLE の実装・検証進捗は tasks と
各 repo の実行結果で別途判定する。ここに書かれた foundation 実装は release/parity 完了を
意味しない。

## 7. KLE Production Cutover Gate

### 7.1 現行 call graph の不適合

2026-08-21 の source audit では、本番の `EguiLanguageEditor::show` は
`KucTextSurfaceBinding::show` を呼び、KLE が `content`、selection/cursor、search query/match/
active index、diagnostics/gutter、authoring menu、scroll/focus、child artifact と latest KUC frame
を retained state として保持し、出力を `apply_kuc_show_output` で再び KLE の semantic state に
戻している。`KucRootBinding` は `SanitizedDocumentRoot` を一つだけ保持し
`synchronize -> show -> forward_events_once` を行えるが、production call graph からは未参照である。

この二経路を段階的に併用して release evidence を作ることは禁止する。旧 root を表示しながら
新 root の receipt を補助的に記録する、または KLE の semantic state から sanitized input を
組み立てる compatibility wrapper は、KLE が所有してはならない意味論を残すため不適合である。

### 7.2 採用する公開境界

release path の公開入口は host-projected opaque root input と one-shot opaque event forwarder
だけとする。host が current identity/revision/snapshot/style、opaque presentation、target-scoped
capability を組み立て、KLE は `KucRootBinding` を唯一の retained UI object として同一 frame に
一度だけ呼ぶ。KLE は frame record の identity/revision/hash と one-shot forwarding receipt だけを
保持・記録できる。target 意味、text value、query/replacement、range/match、authoring command、
diagnostic、geometry、child artifact、pixel plan を inspect、clone、cache、reconstruct してはならない。

旧 `LanguageEditor` / `EditorWriteAccess` / `EditorSelectionControl` / `EditorSearchControl` /
`EditorDiagnosticsSink` / `EditorAuthoringControl` と `EguiLanguageEditor::show(ui)` は、同じ
release render path の互換 wrapper にしてはならない。必要な移行互換性は別 crate 又は明示的に
非 release の adapter に隔離し、AST と compiled dependency gate が production/Storybook/host-E2E
からの到達を拒否する。旧 API を保っている事実は v0.1.0 の互換性又は parity の証拠にならない。

### 7.3 Atomic replacement slice

KUC full root と target-scoped capability の全 operation contract が通った後、一つの atomic batch
で次を行う。

1. `EguiLanguageEditor` の retained rendering field を `KucRootBinding` のみにする。
2. public host-projection input と opaque one-shot forwarder を導入し、`show` の唯一の render/event
   entrance とする。
3. `KucTextSurfaceBinding`、`KucTextSurfacePresentation`、`KucTextSurfaceStyle`、
   `KucTextSurfaceMapping`、`EguiKucArtifactAggregate`、`latest_kuc_*`、KLE output-to-semantic
   reconstruction を production graph から同時に削除する。
4. `platform_text_surface.rs`、direct egui input/context-menu helper、KLE local search/range/content
   mutation、local authoring/menu/pixel state を同じ release graph から削除する。
5. Storybook を同じ full KUC root の opaque frame encoder に接続し、fallback/minifb/shape-count/
   manual compositor を同時に削除する。

partial migration、feature flag、source exclude、runtime fallback、KLE-local callback/action switch は
受理しない。atomic batch の前に KUC full-root physical contract が不足している間は、KLE の旧画面を
新 API に繋いだように見せる実装を行わず、KUC owner 側の不足を先に解消する。

### 7.4 Cutover acceptance

cutover は KUC RawInput/AccessKit の全 operation contract、KLE の one-call/one-forward source and
runtime gate、KLE release graph の legacy-reachability zero、full-root Storybook の PNG/GIF/MP4
decoder-hash evidence、KatanA read-only host effect record、AST/lint zero が同時に成立した時だけ
accepted とする。KLE binding unit test、callback count、静的 source marker、非空スクリーンショット
だけでは acceptance にしない。
