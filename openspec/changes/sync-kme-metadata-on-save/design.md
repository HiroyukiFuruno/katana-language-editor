## Context

metadata targetは行番号だけでは壊れやすい。editorは編集差分を知っているため、保存時にKMEへ再対応を依頼するのが最も自然である。

ただし、editor-viewer同期制御はKatanAが担う。KLEは保存時metadata同期とeditor surfaceを提供するだけで、viewer state、scroll state、highlight stateを知らない。

## Goals

- 保存時にmetadata targetを更新する。
- 自動復元できないtargetをunresolvedとして保持する。
- KME schemaを使い、editor独自schemaを作らない。
- Floem editor実装を前提にする。
- P0 `katana-ast-lint` を品質ゲートにする。

## Non-Goals

- KME文書モデルをeditor内部で再実装すること。
- metadataをMarkdown本文へ埋め込むこと。
- unresolved targetを自動削除すること。
- egui TextEdit実装を前提にしたAPIを追加すること。
- viewerやexportをこのchangeで実装すること。
- KLEがeditor-viewer同期coordinatorになること。

## Decisions

### Save-time Sync

metadata同期は保存直後に行う。editorはold source、new source、metadata path、metadata contentをKMEへ渡し、resolution resultを受け取る。

### P3 Consumer Order

`katana-language-editor` のmetadata同期はP3作業とする。P0 `katana-ast-lint` とP1 KME metadata schema / target resolution APIが揃った後に進める。

### Unresolved Preservation

復元できないtargetはunresolvedとしてmetadataへ残す。UI表示はKatanA/KDV側の責務であり、editorは削除しない。

### Neutral Interface

`katana-language-editor` のpublic contractはKME public DTOまたはeditor-owned neutral DTOだけを扱う。Floem実装型をneutral interfaceへ漏らさない。

KatanAがeditorへscroll、selection、highlightなどの命令を送る場合、KLEはeditor側の命令surfaceだけを提供する。KLEからKDVを呼ばない。
