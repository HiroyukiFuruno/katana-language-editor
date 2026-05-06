## Why

外部メタデータ（metadata）でPDFページングやLLM注釈を管理する場合、Markdown保存後にtargetがずれる。KMEだけでは編集差分を知らず、editorだけでは文書構造を知らない。

`katana-language-editor` は保存時に編集前後の本文を把握できるため、KMEの位置解決APIを使ってmetadataを更新する責務を持つ。

このchangeはP3として扱う。P0 `katana-ast-lint` とP1 KMEのmetadata schemaが先に利用可能であることを前提にする。

## What Changes

- 保存時metadata同期のcontractを定義する
- 編集前本文、編集後本文、metadataをKMEへ渡す
- 解決済みtargetを更新し、復元できないtargetをunresolvedとして保持する
- Floem editorを前提にし、egui TextEdit継続を前提にしない
- 共通AST lintをeditor側の品質ゲートとして使う

## Capabilities

### New Capabilities

- `kme-metadata-sync-on-save`: 保存時にmetadata targetを更新する

## Impact

- `katana-language-editor` neutral interface: metadata sync request/result DTO
- `katana-language-editor-floem`: 保存時同期の呼び出し
- `katana-ast-lint`: P0品質ゲート
- KatanA save flow: metadata更新結果の保存とunresolved表示へ接続
