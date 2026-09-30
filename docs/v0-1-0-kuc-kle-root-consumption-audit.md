# KUC/KLE TextCommandSurface root 消費監査

## 目的と判定方法

KUC の generic `TextCommandSurface` root が提供する primitive と、KLE がそれを消費するために必要な最小 integration 契約を、2026-08-22 時点の source から確認した。型やファイルの存在だけでは接続済みと判定せず、実際の `show` 呼び出し、token の同期、event forwarding の経路を別々に確認する。

対象は次の source である。

- KUC: `/Users/hiroyuki_furuno/works/private/katana-ui-core`
- KLE: `/Users/hiroyuki_furuno/works/private/katana-language-editor`

## Source-backed primitive

| 責務 | source location | 確認できる内容 | 判定 |
|---|---|---|---|
| generic surface の子要素保持 | KUC `crates/katana-ui-core-egui-adapter/src/text_command_surface/types.rs:26-39` | `EguiTextCommandSurface` が `TextSurface`、toolbar、floating toolbar、search strip、context menu を同一 surface の optional child として保持する。 | 提供済み |
| generic presentation 契約 | KUC `.../text_command_surface/types.rs:56-65` | text、toolbar、floating、search、context menu の controlled presentation を一つの generic presentation にまとめる。 | 提供済み |
| child adapter の統合 | KUC `.../text_command_surface/types.rs:67-93` | text adapter と command-chrome adapter を同一 root adapter に保持し、同じ `PlatformTextRasterConfig` を渡す。 | 提供済み |
| root の retained identity / revision | KUC `.../text_command_surface/root.rs:29-35`, `61-80` | root が identity と state revision を保持する。 | 提供済み |
| root frame の描画と event batch 生成 | KUC `.../text_command_surface/root.rs:98-141` | `show` が adapter 描画、event batch、root metadata、paint composition、closed frame を一回で生成する。child の内部 model は返さない。 | 提供済み |
| opaque token の retain | KUC `.../text_command_surface/host_root_facade.rs:25-67`, `116-135` | consumer は opaque target/presentation token を生成し、`EguiTextCommandSurfaceRootFactory::retain` に渡す。token payload は consumer から読めない。 | 提供済み |
| retained root の公開操作 | KUC `.../text_command_surface/host_root_facade.rs:143-155` および KUC `.../text_command_surface/root.rs:110-115` | consumer が保持する root は token 同期後に一回の `show` を行う設計である。 | 提供済み |
| text/font/emoji の raster 責務 | KUC `crates/katana-ui-core-text-raster/src/rasterizer.rs:15-28`, `69-125` | `PlatformTextRasterizer` が platform font catalog を保持し、text を rasterize する。emoji span の font availability を検査する。 | KUC 所有 |
| emoji font の選択 | KUC `.../katana-ui-core-text-raster/src/layout.rs:82-125` | emoji span は `PlatformColorEmojiFaceRecord` の resolved family を使い、解決できなければ error とする。通常の文字と同じ fallback に黙って置換しない。 | KUC 所有 |

## KLE 側の integration 契約

| 契約 | source location | source で確認できる内容 | 接続判定 |
|---|---|---|---|
| provider が token を供給 | KLE `crates/katana-language-editor-egui/src/host_projection_provider.rs:11-22` | `HostProjectionProvider` が retain 用と synchronize 用の `EguiTextCommandSurfacePresentationToken` を返す。missing/stale/duplicate を error として表現する。 | 契約あり |
| KLE が opaque root を retain | KLE `crates/katana-language-editor-egui/src/host_projection_provider.rs:48-65` | `HostProjectionBinding::new` が provider から token を一度取得し、`KucRootBinding::new` に渡す。 | binding 内では接続 |
| KLE が frame を描画し event batch を一度だけ forward | KLE `.../host_projection_provider.rs:68-83`、`.../kuc_root_binding.rs:111-137` | synchronize、KUC root `show`、`forward_events_once` の順を binding 内で実行する。 | binding 内では接続 |
| Storybook の artifact 経路 | KLE `tools/kle-storybook/src/root_runtime.rs:54-87` | `InjectedProjection` が `HostProjectionBinding` を呼び出し、artifact と receipt を受け取る。 | Storybook 経路のみ |
| Storybook の event transport | KLE `tools/kle-storybook/src/root_runtime.rs:40-43`, `91-100` | `StorybookOpaqueForwarder` は `forwarded_batches` を increment して `Ok(())` を返すだけである。 | host effect の証拠ではない |
| Storybook の起動経路 | KLE `tools/kle-storybook/src/main.rs:35-39` | `require_injected_host_projection` を呼ぶが、同じ起動経路には `with_host_projection_provider` の呼出しがない。 | 現状は `MissingHostProjection` で停止し、live harness ではない |
| editor main `show` からの消費 | KLE `crates/katana-language-editor-egui/src/lib.rs:1-17` | この crate の公開 source は provider/binding の module と再 export のみで、editor main の `show` 実装や、実 host state への event 適用呼び出しを確認できない。 | 未接続/接続不明 |

## 最小 integration 契約の設計

実際の KLE host が root を消費するには、次の契約が必要である。

1. host は KUC の generic `EguiTextCommandSurfacePresentation` と style を生成し、opaque `EguiTextCommandSurfacePresentationToken` を provider 経由で供給する。
2. KLE は `EguiTextCommandSurfaceRootFactory` や child model を再実装せず、`HostProjectionBinding` を editor の実際の frame loop から呼び出す。
3. 一回の frame について、KLE は `synchronize_token`、KUC root `show`、`forward_events_once` を一回ずつ実行する。
4. forwarding 先は counter、artifact receipt、または no-op ではなく、KUC の opaque event transport を host の実際の state transition に適用する実装でなければならない。
5. KLE は emoji、font catalog、glyph rasterization、child toolbar/search/context-menu の個別描画を所有しない。これらは KUC の root/adapter/text-raster の責務とする。
6. Storybook は同じ KUC root と実 host adapter を描画する harness とし、counter-only forwarder の成功を host interaction の成功とは判定しない。

KLE unit test が `EguiTextCommandSurfacePresentation` や
`TextCommandSurfaceStyle` を生成して valid token を作ることは、この契約の
証拠ではない。これらは host が KUC generic model から作る projection であり、
KLE に KUC child model の constructor や opaque payload の組み立てを持たせない。
valid token を使う live integration test は Storybook host 又は actual KatanA
host harness に置く。

同様に、Storybook host が KUC root の style を独自に複製してはならない。現行の
default style builder は KUC adapter の sanitized root private module にあるため、
KUC は同じ token/style builder を generic `TextCommandSurfaceStyle` factory として
公開し、sanitized root と Storybook host がそれを共有する。この factory はKUCの
theme/font/raster ownershipに留め、KatanA/KLE固有の表示名や意味論を持たない。

## KUC 側で先に必要な generic dispatch primitive

現行の `EguiTextCommandSurfaceRootEventTransport` は opaque なまま
`KucRootEventBatchForwarder` へ一回渡せる。しかし payload は private で、
forwarder が generic text/search/command/context-menu event を host 側で
一回消費する公開 API は source 上で確認できない。この状態では、KLE は
transport を転送できても real host effect へ接続できない。

最初の実装は KUC にのみ置く。KUC は transport を一回だけ generic
dispatcher へ展開する public contract を提供し、dispatcher が受け取るのは
KUC 定義の generic input/command/search/context-menu event と host-issued
opaque target/revision/correlation に限る。KatanA action、document、path、URL、
range、replace semantics、font/emoji policy はその contract に含めない。
KLE は dispatcher を実装・解釈せず、host supplied forwarder に transport を
一回渡すだけとする。duplicate、stale、unhandled、forwarder error は KUC の
typed error として fail-closed にする。

## 監査結論

KUC の generic root primitive と KLE binding 内の最小 token/show/forward 契約は source 上で確認できる。しかし、KLE の editor main `show` から `HostProjectionBinding` を実際に呼び出し、forwarded event を host state に適用する接続は、この監査対象 source からは確認できない。したがって、現時点で「KLE が KUC root を editor として実消費している」とは判定しない。

Storybook の artifact 生成と `forwarded_batches` の増加は、KUC root が描画されたことの補助的な記録に留まり、editor の host effect、入力反映、検索/コマンド状態遷移、emoji/font の実 host 表示を証明しない。

さらに通常の Storybook 起動経路は provider を注入していないため、描画前に
`MissingHostProjection` で失敗する。起動可能にする修正は、fixture token や
counter forwarder を追加することではなく、KUC generic projection と real host
forwarder を同じ public frame loop に注入することである。

## 変更ファイルと未解決事項

変更ファイル:

- `docs/v0-1-0-kuc-kle-root-consumption-audit.md`

未解決事項:

- KLE editor main の実 `show` 呼び出し元と frame loop が source 上で特定できていない。
- KUC event transport を KLE host state に適用する実装と、適用結果を検証する real-host evidence がない。
- Storybook が実 host effect を検証する forwarder/host harness になっていない。
- KUC の emoji/font 責務を KLE/KDV が重複実装していないことは確認対象だが、KLE の実 host 接続がないため統合表示までは判定できない。
