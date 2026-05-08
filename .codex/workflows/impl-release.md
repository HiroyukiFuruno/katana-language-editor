---
description: 指定バージョンのOpenSpec実装、品質確認、リリース準備、PR作成、GitHub Release、crates.io publish、自己レビュー、事後整理までを自律的に遂行する Implementation & Release Autopilot ワークフロー。
---

# /impl-release vX.Y.Z

指定バージョンの OpenSpec change に基づく実装・修正から、release branch、PR、merge 後の自動 release 確認までを一気通貫で進める。

## 前提

- 作業対象 repository は `katana-language-editor`
- default branch は `master`
- release branch は `release/vX.Y.Z`
- 公開は `release/vX.Y.Z` から `master` へ merge された後の GitHub Actions を正とする
- `cargo publish`、tag 作成、GitHub Release 作成をローカルで直叩きして迂回しない

## 停止ルール

次の場合だけ止めてユーザー判断を仰ぐ。

- OpenSpec tasks にない不足や想定外が出た
- 公開 API、互換性、release 成果物、既存差分を壊すリスクがある
- 指定 version が既存 release line から見て不自然に飛んでいる
- branch protection や secret 変更など、GitHub 設定を変更する必要がある

## Phase 0: 状態把握

```bash
git status --short --branch
git fetch origin --prune --tags
just --list --unsorted
```

確認するもの:

- 既存差分を release 作業へ混ぜない
- `Cargo.toml` の現在 version
- 最新 tag / 最新 GitHub Release
- 対象 OpenSpec change と `tasks.md`

## Phase 1: 作業ブランチ

```bash
git switch master
git pull --ff-only origin master
git switch -c release/vX.Y.Z
```

既存の `release/vX.Y.Z` がある場合は、それを継続する。

## Phase 2: 実装

1. 対象 change の `proposal.md`、`design.md`、`tasks.md`、`specs/**/spec.md` を読む。
2. `tasks.md` の順番で実装する。
3. 完了した task だけ `[x]` にする。
4. 判断材料が不足している場合だけ確認する。

## Phase 3: 品質確認

基本 gate:

```bash
just check
just VERSION=vX.Y.Z release-check
git diff --check
```

対象 change の `tasks.md` に追加 verification がある場合は、それも実行する。

## Phase 4: PR

1. commit 前に `git status --short --branch` と `git diff --cached --stat` を確認する。
2. release branch を push する。
3. `master` 向け PR を作る。
4. PR に `@codex review` を投稿する。

推奨 PR body:

```markdown
## Summary
- Prepare vX.Y.Z release
- Complete <change-id>

## Verification
- just VERSION=vX.Y.Z release-check
```

## Phase 5: Merge

1. 必須 check が通っていることを確認する。
2. cloud review の未対応指摘がないことを確認する。
3. merge はユーザー承認後だけ実行する。
4. `--admin` は使わない。

```bash
gh pr merge --merge --delete-branch <PR番号またはURL>
```

## Phase 6: 自動 release 確認

merge 後、Release workflow と crates.io 公開結果を確認する。

```bash
git switch master
git pull --ff-only origin master
just release-status
gh run list --repo HiroyukiFuruno/katana-language-editor --workflow Release --limit 5
```

## 完了条件

- [ ] 対象 OpenSpec change の全 task が完了している
- [ ] `just VERSION=vX.Y.Z release-check` が成功している
- [ ] release PR が `master` に merge されている
- [ ] Release workflow が成功している
- [ ] `katana-language-editor` と `katana-language-editor-egui` の公開状態を確認している
- [ ] branch hygiene が完了している
