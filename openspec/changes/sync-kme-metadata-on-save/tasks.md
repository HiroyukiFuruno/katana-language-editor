# Tasks: sync-kme-metadata-on-save

## 1. Metadata Sync Contract

### Definition of Ready

- [ ] KMM metadata schemaとtarget resolution APIが定義済みである
- [ ] P0 `katana-ast-lint` の共通品質ゲート方針が利用可能である
- [ ] Floem editor実装が保存処理へ接続できる

### Tasks

- [ ] 1.1 保存時metadata同期request/result DTOを定義する
- [ ] 1.2 old source、new source、metadata contentをKMMへ渡す境界を定義する
- [ ] 1.3 resolved、moved、conflicted、unresolvedの結果を扱う
- [ ] 1.4 共通AST lintをeditor repository adapterで実行する方針を決める

### Definition of Done

- [ ] editor neutral interfaceにegui型が入っていない
- [ ] metadata schemaはKMMのpublic contractを使い、editorで独自定義しない
- [ ] unresolved targetを削除しないcontractになっている
- [ ] viewer、export、editor-viewer同期制御をKLEの責務に含めていない
- [ ] editor固有のlint driftを品質ゲートにしていない

## 2. Save Flow Integration

### Definition of Ready

- [ ] Task 1のcontractが確定している

### Tasks

- [ ] 2.1 保存直後にmetadata syncを呼ぶ
- [ ] 2.2 metadata更新結果を保存対象に含める
- [ ] 2.3 sync失敗時のrecoverable errorを定義する
- [ ] 2.4 KatanAがeditorへ命令できる中立surfaceを定義し、KLEからviewerを呼ばないことを確認する

### Definition of Done

- [ ] Markdown本文の保存とmetadata更新の順序が明確である
- [ ] metadata更新失敗が本文保存失敗と混同されない

## 3. v0.1.0 Neutral Interface との整合

### Definition of Ready

- [ ] v0.1.0 の `language-editor-component` / `language-editor-theming` / `language-editor-i18n` / `language-editor-host-control` / `language-editor-settings` 契約が確定している

### Tasks

- [ ] 3.1 save / autosave / unresolved 通知に使う全 UI 文字列を `Strings` の追加キーとして定義し、KDV en preset 側で実値を提供する
- [ ] 3.2 save flow が使う色（warning / accent）を `Theme::colors` のセマンティック alias から引き、色リテラルが impl crate に残らないことを確認する
- [ ] 3.3 unresolved target を `EditorDiagnosticsSink` / `EditorDecorationsSink` で host に push する経路を実装する
- [ ] 3.4 自動保存トリガは `EditorSettings::autosave` に従い、KLE 独自の interval / on-off フラグを持たないことを確認する
- [ ] 3.5 host が `EditorWriteAccess::with_origin("kme-sync")` で post-resolve 書き込みを行えるよう API 整合を確認する

### Definition of Done

- [ ] save flow が theme / strings / settings / host-control の DI を尊重する
- [ ] unresolved の UI 表示は host 側に委譲され、KLE 内で完結しない

## 4. Final Verification

- [ ] 4.1 保存時metadata移動、衝突、unresolvedのテストを追加する
- [ ] 4.2 共通AST lintのeditor adapterで検査できることを確認する
- [ ] 4.3 `kle-linter` の `prohibited-color-literal` ルールに save flow 関連コードが違反しないことを確認する
- [ ] 4.4 `npx -y @fission-ai/openspec validate "sync-kme-metadata-on-save" --strict` を実行する
