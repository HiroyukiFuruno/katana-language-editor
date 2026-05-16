## Why

外部メタデータ（metadata）でPDFページングやLLM注釈を管理する場合、Markdown保存後にtargetがずれる。KMEだけでは編集差分を知らず、editorだけでは文書構造を知らない。

`katana-language-editor` は保存時に編集前後の本文を把握できるため、KMEの位置解決APIを使ってmetadataを更新する責務を持つ。

KLEはviewerやexportを知らない。editor-viewer同期制御はKatanAが持ち、KatanAがeditorまたはviewerへ命令する。

このchangeはP3として扱う。P0 `katana-ast-lint` とP1 KMEのmetadata schemaが先に利用可能であることを前提にする。

## What Changes

- 保存時metadata同期のcontractを定義する
- 編集前本文、編集後本文、metadataをKMEへ渡す
- 解決済みtargetを更新し、復元できないtargetをunresolvedとして保持する
- viewer、export、editor-viewer同期制御をKLEの責務から外す
- KatanAがeditorへ命令するための中立surfaceだけを許可する
- Floem editorを前提にし、egui TextEdit継続を前提にしない
- 共通AST lintをeditor側の品質ゲートとして使う
- v0.1.0 で確定する neutral interface（theme / i18n / host-control API / settings）と整合する形で、保存フローが host (KDV) preset と各種 DI を尊重する
- save 完了 / 自動保存 / unresolved 通知は editor 内部の文字列リテラルや色を持たず、`Strings` / `Theme` 経由のみで表現する
- unresolved target は `EditorDecorationsSink` / `EditorDiagnosticsSink` で host に push し、host が viewer 表示・UI 決定を担う
- 自動保存タイミングは `EditorSettings::autosave` に従い、KLE は独自に間隔を決めない

## Capabilities

### New Capabilities

- `kme-metadata-sync-on-save`: 保存時にmetadata targetを更新する

## Impact

- `katana-language-editor` neutral interface: metadata sync request/result DTO
- `katana-language-editor-floem`: 保存時同期の呼び出し
- `katana-ast-lint`: P0品質ゲート
- KatanA save flow: metadata更新結果の保存とunresolved表示へ接続
