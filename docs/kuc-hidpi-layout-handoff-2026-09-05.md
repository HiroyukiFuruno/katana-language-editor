# KUC native HiDPI text layout handoff

## 観測

公開 `katana-ui-core 0.3.6` を使用する KLE の `cargo run -p
kle-storybook --locked -- --interactive` を macOS の実ウィンドウで撮影した。
行番号が行間に対して大きく、Diagnostics の見出し・filter の文字が重なる。
撮影時の pixels_per_point は未記録のため、特定倍率での実測完了とはしない。

ローカル画像: `/var/folders/ql/4640yx8s22zg367pjjld7yc00000gn/T/codex-shot-2026-09-05_19-49-44.png`
このパスは公開添付ではない。画像のみを原因や修正完了の根拠にしない。

KLE は通常の eframe viewport を作り、KUC 公開 root を呼び出す。
KLE に font、gutter、DPI の独自補正を追加しない。

## 公開ソースで確認した不整合

1. `src/egui/text_surface/measurement.rs` の `controlled_gutter_width` は
   同倍率の raster.width を `logical_extent(..., scale_factor)` で論理幅へ戻す。
   一方、`src/egui/text_surface/paint.rs` は同倍率で生成した行番号 raster の
   width/height を直接 `UiRect` に使う。物理寸法を論理寸法として描くため、
   scale=2 では計測した行番号に対し描画寸法が約2倍になる経路がある。
2. `src/egui/diagnostics_list/paint.rs` の `text_width` は
   `PlatformTextRasterRequest::from_text` の既定 scale=1 の raster.width を
   現在の scale で割る。`paint_text` は現在の scale で rasterize してから
   scale で割るため、予約幅と描画幅の倍率が一致しない。
3. 同じ `paint_text` は `max_width_px` に `bounds.width() * scale` を渡すが、
   `src/text_raster/layout.rs` は request の max_width をさらに scale 倍する。
   この二重適用と clipping/折返しの関係も確認が必要。

これらは公開ソースの不整合であり、画面の全不具合を説明したとの主張ではない。
KUC 内の同じ変換経路を用いる部品にも影響調査が必要。

## KUC側の対応条件

- [ ] KUC 所有の計測・描画で logical/physical の変換を一致させる。
- [ ] scale=1/1.5/2 の実 raster を使い、行番号の論理画像寸法、予約幅、行高、
  Diagnostics label の予約/描画幅と非交差を数値で検証する。固定画像の更新だけにしない。
- [ ] 日本語/IME と正確な U+2B50 U+FE0F の色付き glyph を保持し、計測・hit test・
  selection と描画の整合を macOS/Windows/Linux の実入力で証跡化する。
- [ ] 修正時に直接/推移的依存・lockfile を調査更新し、既存全ゲートを維持する。
- [ ] 修正版を registry に公開し、下流が公開依存で再検証できるようにする。

KLEからKUCソース/registryキャッシュを直接変更しない。generic artifact plan の
KUC #40 とは別の既存描画不具合であり、#40 の完了を修正検証の代用にしない。
