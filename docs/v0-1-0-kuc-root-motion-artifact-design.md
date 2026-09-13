# KUC Root Motion Artifact Design

## Decision

KUC が generic retained-root motion artifact を所有する。KLE Storybook は、KLE public
`show_write_artifact_with_opaque_frame` から得た closed `FullRootArtifact` receipt の順序だけを
KUC に渡す。KLE は RGBA を読む、PNG を合成する、GIF/MP4 を encode/decode する、font/glyph を
描く、frame hash を再計算する責務を持たない。

## Reuse Boundary

KUC Storybook にある既存の text-command-root MP4 pipeline を、KatanA/KLE 固有 scenario
を含まない generic writer に抽出する。writer の入力は次だけとする。

* ordered, unique stage identifier
* KUC `FullRootArtifact` が出力した PNG path, width, height, pixel hash, PNG SHA-256,
  root record hash
* expected frame count and generic encoding settings

writer は input PNG の存在、RGBA8、dimensions、PNG SHA-256、duplicate/stale stage、non-empty
pixels、root artifact provenance を検証してから GIF/MP4 を生成する。MP4 は KUC が発見した
`ffmpeg` を使い、decode 後の frame count と dimensions を fail-closed に検証する。KLE に
`ffmpeg` path、encoder arguments、decoded RGBA、frame bitmap を公開しない。

## Output

KUC writer は artifact paths and SHA-256、frame/root record hashes、encoder identity/version、
decoded MP4 evidence、canonical manifest hash だけを返す。KLE はそれを source-derived scenario
correlation と KLE opaque transit receipt に結合できるが、video を host effect や KatanA parity
の代替根拠にはしない。

## Verification

1. KUC unit tests: empty/duplicate/stale/missing/mismatched PNG、wrong dimensions、bad SHA、
   missing `ffmpeg`、encoder/decode failure が typed error になる。
2. KUC integration: actual retained root frames from KUC public artifact writer onlyで GIF/MP4を
   生成し、manifest、PNG sequence、decoded frame count/dimension/hash provenanceを検証する。
3. KLE integration: actual KLE public opaque root A/B artifact routeを通過した receiptで KUC
   writer を呼び、KLE source に pixel, compositor, GIF/MP4, ffmpeg, fallback renderer が無いことを
   AST/compile/runtime testで固定する。
4. macOS/Windows/Linux: each profile independently creates and validates its own artifact. A local
   macOS video never proves the other profiles.

## Rejected Alternatives

* KLE Storybook が `gif`/`png`/`ffmpeg` を直接使う。
* KUC generic scenario videoをKLE public root artifactとして表示する。
* video existence, frame count only, or screenshotをcorrectness evidenceとする。
* missing encoder/profileをskip/fallbackで通す。
