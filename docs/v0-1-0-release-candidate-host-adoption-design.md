# KLE v0.1.0 RC と KatanA Host Adoption の実行設計

## 結論

`katana-language-editor v0.1.0` の最終公開前に actual KatanA host E2E を
必須にする一方、KatanA が registry から KLE を取得していない状態では、その
E2E を実行できない。この循環は final gate の省略や local path dependency では
解かない。公開済み registry crate だけを使う `v0.1.0-rc.N` を一度だけ介在させ、
KatanA #336 の採用と real host E2E を経た後に `v0.1.0` を公開する。

RC は完了や最終 release を意味しない。final `v0.1.0` は RC と同じ public API
および実装 fingerprint を持ち、許容される差分は package version、release metadata、
KatanA の `=0.1.0` dependency update に限る。

## 不変条件

1. KLE は KatanA、KUC、KDV を直接編集しない。KatanA の採用と実ホスト作業は
   [KatanA #336](https://github.com/HiroyukiFuruno/KatanA/issues/336) だけで行う。
2. RC、final とも KUC は crates.io の `katana-ui-core@0.3.5` を使用する。path/git
   KUC dependency、KLE-local compositor、glyph fallback、KatanA 固有 UI/API は不許可。
3. KatanA host E2E は公開済み exact KLE version を解決し、physical input -> KatanA
   natural handler -> final document/file/view effect を一つの correlation に記録する。
   direct `AppAction` injection、simulator、Storybook callback、固定座標、sleep は証拠に
   ならない。
4. `KATANA_REPO` は checkout location の可変 locator である。source-closure baseline
   は固定 SHA `4f6a6287c650a38633c7baeb544a92e739c68567`、adoption E2E は #336 の
   immutable KatanA commit をそれぞれ revision/provenance として記録する。
5. `⭐️` は U+2B50 U+FE0F の scalar sequence、色付き glyph pixel、IME preedit/commit、
   text measurement、hit test、AccessKit を macOS/Windows/Linux で同じ evidence set に
   持つ。`☆`、code-point retention、shape count は代替にしない。

## 段階

### A. RC 前 KLE/KUC gate

KLE の全 source-derived requirement を source universe に保持する。KLE/KUC-owned leaf
は full Storybook で KUC root frame、hash、decode、AccessKit、opaque one-shot receipt を
検証する。KatanA-owned effect leaf は `downstream_required` として明示し、pass へ昇格
させない。三 OS source capture/provenance は RC 対象 commit の clean worktree から生成する。

この段階で通すものは format、warnings-denied Clippy、workspace tests、AST lint、coverage、
KUC registry resolution、KUC Unicode contract、KLE opaque boundary、full KUC-root Storybook、
source-derived requirement/branch/action-origin completenessである。`block@0.1.6` のような
future-incompatibility は RC でも fail-closed とする。

### B. `v0.1.0-rc.N` の公開

1. RC version は `0.1.0-rc.N`、GitHub Release は prerelease として作成する。
2. core crate を先に publish し、crates.io 可視化後に egui crate を publish する。
3. KUC/KLE package resolution、package contents、RC tag/Release/crates.io visibility を
   read-only audit で固定する。
4. RC artifact は KLE requirement ledger、source closure identity、KUC version、public API
   fingerprint、Storybook artifact hash を handoff する。host E2E 未達を completion として
   表示してはならない。

### C. KatanA #336 採用と full host E2E

KatanA #336 は `katana-language-editor = "=0.1.0-rc.N"` と
`katana-language-editor-egui = "=0.1.0-rc.N"` を registry から解決する。KatanA branch の
adapter diff は Issue と PR に紐付け、baseline source closure と adoption commit の
source/provenance diff を出す。

全 editor leaf は次の joined evidence を要する。

- requirement/source branch/action origin
- physical KUC RawInput と current-frame AccessKit target
- one KLE opaque transit 又は retained-UI no-mutation receipt
- physical KatanA UI route と natural handler
- document/buffer/undo/save/search/diagnostics/asset/view の final effect 又は明示 no-mutation
- source/KUC/KLE/KatanA revision、frame hash、artifact hash、OS profile、correlation

file/image/clipboard、Replace/Replace All、authoring 14 trigger、code kind 17、line numbers、
diagnostics、navigation、workspace search、Markdown search、IME、日本語、`⭐️` を個別 leaf と
して実行する。fixed baseline に route が無い Replace/Replace All は #336 で仕様を確定し、
未確定なら final gate を fail-closed にする。

### D. final `v0.1.0`

RC と final の public API、KUC dependency set、full Storybook manifest、source-closure
requirement set は byte/fingerprint compatible でなければならない。finalization で許される
変更は version/release metadata と #336 の exact `=0.1.0` dependency updateだけである。

KatanA は final registry version を解決して同じ full host E2E を再実行する。RC evidence
だけで final を通さない。final KLE tag、GitHub Release、core/egui crates.io visibility、
KatanA host evidence、KDV follow-up、不要 branch/worktree cleanup が完了して初めて、本 goal
を complete とする。

## Gate 分離

| Gate | 目的 | host E2E の扱い |
| --- | --- | --- |
| `kle-rc-check` | RC publishable な KLE/KUC-owned contract を検証する | downstream-required leaf を宣言し、pass と混同しない |
| `katana-adoption-check` | #336 が exact RC registry dependency を採用したことを検証する | KatanA PR/commit と registry resolution を固定する |
| `full-parity-check` | final release candidate の全 leaf joined evidence を検証する | KatanA physical host E2E を必須にする |
| `release-check` | final `v0.1.0` の公開可否を検証する | `full-parity-check` の final-version evidence を必須にする |

既存の `just check` が pre-adoption KLE gate と final host parity を同時に要求する構成は
この設計に反する。実装時は `katana-parity-check` の mode と artifact schema を分け、
pre-adoption の `downstream_required` を completed leaf と扱わない。host E2E を省略する
ための flag、allowlist、skip、mock、path dependency を追加してはならない。

## 検証と公開後整理

各段階の command exit status、revision、artifact hash、registry resolution を Issue #8/#11/#336
へ記録する。final public release 後に default branch へ切替・pull を行い、merged release/RC
branch、temporary worktree、temporary fixed-source clone を ownership と merge state を確認して
から削除する。remote branch は明示指示がある場合だけ削除する。
