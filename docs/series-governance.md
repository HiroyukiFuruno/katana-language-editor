# Series Governance

対象Issue: https://github.com/HiroyukiFuruno/katana-language-editor/issues/9

## 境界

自動作業はIssue起点とし、KatanAシリーズの作業branchは`release/*`のみを使用する。
default branchを直接編集しない。既存のformat・strict品質ゲートを削除、置換、
迂回しない。hookはCIと実装者の検証を代替しない。

## CommitとPush

- 作業commitには対象repositoryの`Issue: #<number>` trailerを一つ付ける。
- commit-msgで形式を検査し、pre-pushで実際のOpen IssueをGitHubから確認する。
- pushの全refを検査する。新規・更新branchは`release/*`に限定する。
- default branchの削除・直接pushは拒否する。古い非release branchの削除は
  branch命名の移行を妨げないが、default branchは保護する。
- sibling依存の追加・変更には`Downstream-Issue: #<number>`も必要とする。
  Issueの`Upstream-Release:`、`Dependency-Update:`、`Validation:`は空値を拒否する。
- 依存変更はworkspace/memberのmanifestとlockfileを対象にする。単一rootの
  `Cargo.toml`だけを検索してmember変更を取りこぼさない。
- GitHub参照の失敗・不明なref・不正な範囲は拒否する。取得失敗をOpen扱いしない。

Gitのtrailer・ref・履歴を使用し、本文中の見かけのIssue文字列をtrailerとして
受理しない。正規のremoteからrepository/default branchを特定する。productionの
GitHub取得先や実行binaryをテスト用環境変数で差し替える迂回口は設けない。

## 公開後の整理

GitHub Releaseが公開済みでdraft/prereleaseではないこと、tagのcommit、default
branchとの関係を確認してから処理する。未公開・別commitのtagを根拠に削除しない。
remote cleanupは公開済みtagとorigin/defaultの双方に含まれる`release/*`だけを
対象とする。default branch、未統合の将来branchは削除しない。

local cleanupはstashゼロ、cleanを確認してからdefault branchへswitchし、
`pull --ff-only`する。別worktreeで使用中のbranch、dirty worktree、判定不能な
対象は削除しない。保持理由と未完了を明示し、整理完了と偽らない。
使用中か不明なworktreeは自動削除せず、実行セッションを確認した上で別途整理する。

release workflowは全crate公開後にremote cleanupを実行する。localのpost-merge
hookは公開済みreleaseを確認してlocal cleanupへ接続する。再入・再帰を防ぎ、
既存hookを無効化しない。CIから利用者のlocal checkoutを操作したとは扱わない。

pre-push/commit-msgはLefthookのscript jobを使用する。v2.2.0の通常commandは
変更ファイルがない場合に省略されるため、空commitや削除refのpolicy確認には使わない。
pre-pushはstdinを一つのguardへ渡し、成功後に従来のstrict gateを必ず実行する。
`just hooks-install`で実際のhookを設定し、script実行を含む検査を行う。

Issue #9 COREのcleanup CLIは、`--tag vX.Y.Z`を必須とし、originから特定した
`github.com`のrepositoryに対する実GitHub APIのReleaseだけを公開根拠とする。
Releaseはtag名一致、`draft=false`、`prerelease=false`、`published_at`、公開URLを
厳密に確認する。両公開crateのexact versionを実`cargo info --registry crates-io`の
終了コードで確認する。Cargoのindex/cache確認であり独立したlive registry attestation
ではないという制約をreportへ残す。人間向けstdoutの`name:`等は解析しない。
確認前のfetch/switch/pull/deleteは禁止し、テストモード、gh差替え、
repository差替えによるbypassも設けない。dry-runはfetchを含む全local Git mutationを
行わず、tag/defaultのgraph factが欠落またはstaleなら同期成功にしない。

remote cleanupは、公開tagのpeeled commitと`origin/default`の双方にtipが祖先である
`origin/release/*`だけを期待tipのlease付き削除pushで削除する。local cleanupはcleanかつstashゼロを
確認し、実defaultへswitchして`pull --ff-only`した後、他worktreeにboundでない merged
`release/*`だけを`git branch -d`する。default/master/main/current、dirty、unknown、
unmerged、boundは理由付きで保持し、残存があれば非成功statusを返す。worktreeは削除せず、
任意lockfileやsourceも削除しない。post-merge hookは公開latestが404のときだけno-opとし、
auth/network errorは失敗扱いにする。cleanupのpullによるhook再入だけを内部guardで抑止し、
cleanup CLI自体の公開Release検証は常に実行する。

## 検証

形式・policyのunit testと、実Git repository/履歴を用いる検査を分離する。
GitHubとの結合検査には実repositoryのIssue/Releaseを読み取り専用で使う。
偽の`gh`、mock server、`GOVERNANCE_TEST_MODE`等で公開確認を省略するIT/E2Eは
受理しない。production cleanupには公開確認を飛ばすオプションを設けない。

正常・trailer欠落/重複・本文偽装・closed Issue・取得失敗・member依存変更・
lockfile欠落・default保護・非release更新・複数ref・削除refを検査する。
cleanupはdirty/stash/使用中/未統合/未公開tagの保持、default同期、実削除対象の
公開tagへの包含を検査する。実remote削除を未公開KLEのテスト目的で行わない。

## 旧Worktreeの扱い

`issue/9-series-governance-hooks`のHEADはmasterへ統合済みだが、四つのtracked
差分と七つの未追跡ファイルが残る。primaryの`release/*`へ必要な変更を正式統合し、
検証してから旧branch/worktreeを整理する。stashやpatchへの退避だけでは完了にしない。
旧案の公開確認bypass、偽ghによるIT、非release作業branch、限定的な依存検出は
上記契約へ改める。原差分は統合・不採用理由が確定するまで保持する。
