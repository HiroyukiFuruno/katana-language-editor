## Context

metadata targetは行番号だけでは壊れやすい。editorは編集差分を知っているため、保存時にKMMへ再対応を依頼するのが最も自然である。

ただし、editor-viewer同期制御はKatanAが担う。KLEは保存時metadata同期とeditor surfaceを提供するだけで、viewer state、scroll state、highlight stateを知らない。

## Goals

- 保存時にmetadata targetを更新する。
- 自動復元できないtargetをunresolvedとして保持する。
- KMM schemaを使い、editor独自schemaを作らない。
- Floem editor実装を前提にする。
- P0 `katana-ast-lint` を品質ゲートにする。

## Non-Goals

- KMM文書モデルをeditor内部で再実装すること。
- metadataをMarkdown本文へ埋め込むこと。
- unresolved targetを自動削除すること。
- egui TextEdit実装を前提にしたAPIを追加すること。
- viewerやexportをこのchangeで実装すること。
- KLEがeditor-viewer同期coordinatorになること。

## Decisions

### Save-time Sync

metadata同期は保存直後に行う。editorはold source、new source、metadata path、metadata contentをKMMへ渡し、resolution resultを受け取る。

### P3 Consumer Order

`katana-language-editor` のmetadata同期はP3作業とする。P0 `katana-ast-lint` とP1 KMM metadata schema / target resolution APIが揃った後に進める。

### Unresolved Preservation

復元できないtargetはunresolvedとしてmetadataへ残す。UI表示はKatanA/KDV側の責務であり、editorは削除しない。

### Neutral Interface

`katana-language-editor` のpublic contractはKMM public DTOまたはeditor-owned neutral DTOだけを扱う。Floem実装型をneutral interfaceへ漏らさない。

KatanAがeditorへscroll、selection、highlightなどの命令を送る場合、KLEはeditor側の命令surfaceだけを提供する。KLEからKDVを呼ばない。

### v0.1.0 Neutral Interface との整合

v0.1.0 の `language-editor-component` / `language-editor-theming` / `language-editor-i18n` / `language-editor-host-control` / `language-editor-settings` で確定する DI 契約を前提にする。

- 保存ボタン文言・自動保存通知・unresolved 警告・conflict 表示など save flow が生む全 UI 文字列は `Strings` 経由（KDV en preset を必須）で解決する。KLE 内に literal 文字列を持たない。
- save / autosave 関連のインジケータ色（diagnostic_warn / decoration_accent 等）は `Theme::colors` のセマンティック alias を引く。色リテラルは `kle-linter` の `prohibited-color-literal` で禁止する。
- unresolved target は `EditorDiagnosticsSink::push(Diagnostic { severity: Warning, .. })` または `EditorDecorationsSink::push_gutter_marker(..)` で host に通知する。KLE は viewer へ通知せず、host が viewer / dialog を決める。
- 保存トリガと自動保存タイマは `EditorSettings::autosave` に従う。KLE 独自の interval を持たない。
- host が `EditorWriteAccess` で metadata sync 結果に基づく後処理書き込みを行う場合、`with_origin("kme-sync")` 等で origin タグを付け、通常編集と区別する。
