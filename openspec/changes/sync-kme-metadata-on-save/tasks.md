# Tasks: sync-kme-metadata-on-save

## 1. Metadata Sync Contract

### Definition of Ready

- [ ] KME metadata schemaとtarget resolution APIが定義済みである
- [ ] P0 `katana-ast-lint` の共通品質ゲート方針が利用可能である
- [ ] Floem editor実装が保存処理へ接続できる

### Tasks

- [ ] 1.1 保存時metadata同期request/result DTOを定義する
- [ ] 1.2 old source、new source、metadata contentをKMEへ渡す境界を定義する
- [ ] 1.3 resolved、moved、conflicted、unresolvedの結果を扱う
- [ ] 1.4 共通AST lintをeditor repository adapterで実行する方針を決める

### Definition of Done

- [ ] editor neutral interfaceにegui型が入っていない
- [ ] metadata schemaはKMEのpublic contractを使い、editorで独自定義しない
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

## 3. Final Verification

- [ ] 3.1 保存時metadata移動、衝突、unresolvedのテストを追加する
- [ ] 3.2 共通AST lintのeditor adapterで検査できることを確認する
- [ ] 3.3 `npx -y @fission-ai/openspec validate "sync-kme-metadata-on-save" --strict` を実行する
