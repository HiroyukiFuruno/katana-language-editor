# リリース手順

## 方針

`release/vX.Y.Z` ブランチから `master` へ取り込み依頼（Pull Request）を作る。
その取り込み依頼（Pull Request）では通常の品質ゲート（quality gate）とリリース前検査を必須にする。
取り込み（merge）後は自動実行基盤（GitHub Actions）がタグ（tag）、GitHub リリース（GitHub Release）、crates.io 公開を実行する。

## 必須検査

GitHub のブランチ保護（branch protection）では、KML と同じ形で次を必須検査（required check）にする。

- `Test and Build (macos-latest)`
- `Test and Build (ubuntu-latest)`
- `Test and Build (windows-latest)`
- `preflight`

## リリース前検査

`release-preflight` は通常の取り込み依頼（Pull Request）で `just check` を実行する。
これは `v0.1.0-rc.N` のための KLE/KUC gate であり、KatanA host E2E を成功扱いにしない。
`release/v...` の final branch では追加で `just VERSION=vX.Y.Z release-check` を実行する。
`release-check` は公開前の品質ゲートであり、公開完了の証明ではない。
公開後は必ず `just VERSION=vX.Y.Z release-completion-audit` を実行し、tag / GitHub Release / crates.io を別に確認する。
公開前後の残 blocker を一覧したい場合は、fail-fast ではない read-only 診断として `just VERSION=vX.Y.Z release-blocker-audit` を使う。
内容は次の通り。

- 整形確認（format）、静的検査（lint）、単体テスト（unit test）、抽象構文木検査（AST lint）
- KUC contract、full KLE Storybook manifest、KLE 自身の source-derived public-input 検証
- final では `kle-release-parity-check` による source-derived 全 leaf、三 OS profile、KUC opaque receipt の結合検証
- カバレッジ（coverage）。現状の下限は行カバレッジ（line coverage）64%
- `Cargo.toml` の版番号（version）とブランチ版番号（branch version）の一致
- 作業領域（workspace）内部依存の版番号（version）一致
- 対象版番号（version）が公開済み release line から自然な次版であること
- 対象版番号（version）が crates.io に未公開であること
- `katana-language-editor` の梱包（package）と公開の事前実行（publish dry-run）
- `katana-language-editor-egui` の梱包（package）収録対象確認

`katana-language-editor-egui` は `katana-language-editor` を先に公開しないと crates.io 上で依存解決できない。
そのため取り込み依頼（Pull Request）時点では `katana-language-editor` を事前実行（dry-run）し、`katana-language-editor-egui` は収録対象確認までに留める。

## 公開順序

`release/vX.Y.Z` の取り込み（merge）後に `Release` ワークフロー（workflow）が動く。
手動実行（workflow_dispatch）でも `publish_crates=false` の部分 release は許可しない。
手動実行で `publish_crates=false` が指定された場合、tag / GitHub Release の作成前に失敗させる。
順序は次の通り。

1. KUC 側で `katana-ui-core@0.3.17` を公開し、KLE 外部 package resolution が同 crate の `egui` / `text-raster` / `storybook-artifacts` feature を選ぶことを確認する。この KUC release は KLE tag / GitHub Release / KLE crate publish より先でなければならない。
2. 同一の Release run から reusable `source-closure` を三 OS で実行し、assemble 済み artifact を Release job へ download する。release job はこの run の artifact 以外を参照しない
3. `just VERSION=vX.Y.Z release-check`。KLE の公開前 evidence を fail-closed で検証する
4. リリースタグ（release tag）作成。既存 tag がある場合は current HEAD と一致する場合だけ続行
5. GitHub リリース（GitHub Release）作成
6. `katana-language-editor` を crates.io に公開
7. crates.io で `katana-language-editor` が見えるまで待機
8. `katana-language-editor-egui` を crates.io に公開
9. crates.io で `katana-language-editor-egui` が見えるまで待機
10. `just VERSION=vX.Y.Z release-completion-audit` で tag / GitHub Release / crates.io を read-only 監査する。tag は存在だけでなく、既定では audit 実行時の `HEAD`、必要なら `EXPECTED_RELEASE_COMMIT` で指定した release 対象 commit と一致しなければならない

KatanA #336 の採用は final `v0.1.0` 公開後の下流作業である。公開済みの exact registry version を採用して行う physical host E2E は `full-parity-check` で別に検証し、KLE の公開前 gate には含めない。KDV follow-up は final 公開後に Issue 経由で行う。
`source-closure` の native host E2E は protected `workflow_dispatch` で `run_native_host_e2e=true` を明示した場合だけ実行する。Release からの reusable 呼び出しは常にこの input の既定値 `false` を使う。
native host E2E は canonical source-closure artifact と同じ run が生成した専用 layout artifact を `target/source-closure` へ展開する。host test の source-derived target、context-menu manifest、macOS probe を別 run や任意の環境変数から受け取らない。

`release-completion-audit` は最終完了 gate として最初の未達で失敗する。
複数の未達を同時に確認したい場合は `release-blocker-audit` を使い、remote tag、tag commit、GitHub Release、crates.io 2 crate、KUC registry dependency、dependency compatibility を一度に列挙する。

v0.1.0 の現在の完了判定は `docs/v0-1-0-release-readiness.md` に記録する。

## 必要な秘匿値

自動実行基盤（GitHub Actions）には次の秘匿値（secret）が必要。
値は crates.io の API トークン（API token）を使う。

```bash
cd /Users/hiroyuki_furuno/works/private/katana-language-editor
gh secret set CARGO_REGISTRY_TOKEN
```

トークン（token）は秘匿値として扱い、リポジトリ（repository）に保存しない。

なお、`release-preflight.yml` / `release.yml` は KUC v0.3.17 の Unicode と variable-viewport motion evidence contract を再実行するため、`KLE` ワークスペースと上流 KUC checkout を含む CI 作業ディレクトリを前提にしています。KLE の Cargo 依存は crates.io から解決し、checkout を path dependency として使用しません。

- `$GITHUB_WORKSPACE/katana-language-editor`
- `$GITHUB_WORKSPACE/katana-ui-core`

このため、KUC contract の再実行時だけ `../katana-ui-core` を参照します。

`katana-ui-core` は現時点で public リポジトリのため、追加のトークンは通常不要です。
private リポジトリ化した場合は、`actions/checkout` でのクロスリポジトリ取得に必要な read 権限トークンを別途用意してください。
