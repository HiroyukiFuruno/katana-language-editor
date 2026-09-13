# KUC Root Emoji Evidence Tightening Design

## Decision

`⭐️` (U+2B50 U+FE0F) の受入証跡は、KUC retained root が同一 `RawInput`
trace で生成した最終 RGBA からのみ採取する。KLE、Storybook、KDV、または別の
`PlatformTextRasterizer` が crop を再生成する経路は受入証跡に使わない。

KUC は一つの `PlatformFontCatalog` を root lifetime で共有し、TextSurface、
CommandChrome、SearchStrip、ContextMenu が同じ catalog policy、resolved color face、
file SHA-256、fingerprint を使用する。各 component の text/cache/selection state は共有
しない。KatanA 固有の command、document、range、path、URL、host effect はこの API に
含めない。

## Current Gap

現在の `KucUnicodeColorGlyphEvidenceCapture` は root frame を生成した後、別途
`PlatformTextRasterizer::new(options.config)` で `⭐️` / `☆` の bounds と pixels を取得する。
このため root の `root_rgba_hash` と star/control crop が同じ catalog instance、同じ
texture、同じ composite frame であることを示していない。また root adapter は
TextSurface、CommandChrome、ContextMenu ごとに font catalog を作る。

## KUC-only Change Boundary

1. `PlatformTextRasterizer` に既存の `with_catalog` を用い、KUC adapter 内で root-owned
   `Arc<PlatformFontCatalog>` を一度だけ作る。
2. TextSurface、CommandChrome と必要時の ContextMenu はその catalog を借用して各自の
   rasterizer を作る。KLE は catalog、font path、family、SHA、pixels を取得しない。
3. KUC root 内部に only-for-evidence の observation を置く。これは同一 root frame の
   TextSurface grapheme geometry と final composite RGBA から `⭐️` と `☆` を crop する。
   child paint plan、texture handle、geometry を consumer public API に公開しない。
4. evidence は crop hash、pixel count、chromatic count、resolved face SHA、catalog
   fingerprint、root RGBA/record/AccessKit hashes を一つの canonical artifact に結合する。
   crop の bytes/hash は root composite の対応範囲と一致しなければ fail-closed とする。
5. Japanese、IME preedit/commit、ZWJ、exact VS16 grapheme/hit/caret は同じ trace の
   retained root から検証する。`☆`、monochrome glyph、missing face、profile/fingerprint
   mismatch、separate-raster crop は全て失敗させる。

## Verification

* macOS: Apple Color Emoji の pinned face SHA、final-root `⭐️` crop の chromatic pixels、
  `☆` control crop との差分を通す。
* Windows: Segoe UI Emoji、Linux: provisioned/pinned Noto Color Emoji でも同じ test を
  skip せず実行する。profile artifact が無い場合は release gate を失敗させる。
* KUC unit/integration test は catalog discovery が root ごとに一回であり、child ごとの
  discovery が発生しないことを検証する。
* KLE Storybook は KUC public root artifact の frame/record/AccessKit/hash を参照するだけ
  とし、crop 再計算、font fallback、pixel compositor、手描き emoji を持たない。

## Rejected Alternatives

* KLE で emoji font を指定・上書きする。
* `⭐️` の文字列または code point 一致だけを合格にする。
* root hash と別 raster crop を同一証跡として結合する。
* OS に color face がない時に `SansSerif`、`☆`、missing glyph、skip を受け入れる。
