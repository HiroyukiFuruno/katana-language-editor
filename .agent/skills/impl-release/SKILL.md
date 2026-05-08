---
name: impl-release
description: katana-language-editorで指定バージョンのOpenSpec実装、品質確認、リリースPR、GitHub Release、crates.io publish、事後整理までを一気通貫で進めるときに使う。/impl-release vX.Y.Z と同等のリリース実装ワークフロー。
---

# impl-release

このスキルは、`/impl-release vX.Y.Z` として扱うリリース実装ワークフローの入口です。

## 実行ルール

1. ユーザー指定のバージョン（例: `v0.1.0`）を対象にする。
2. 詳細手順は `.agent/workflows/impl-release.md` を正として読み込む。
3. OpenSpec 実装、検証、リリース準備、PR 作成、自己レビュー、公開、事後整理まで進める。
4. 作業開始前に必ず `git status --short --branch` を確認し、既存差分と関心事を混ぜない。
5. merge 後の自動 release が失敗した場合、別の tag / publish コマンドで迂回しない。原因を直して同じフローへ戻る。

## 注意

- `workflows/impl-release.md` は手順書であり、Agent のスキル呼び出し対象ではない。
- `/` から呼び出したい場合は、この `impl-release` スキルを使う。
