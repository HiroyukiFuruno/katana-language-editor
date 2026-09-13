# KLE v0.1.0 source-closure operational input design

## Status

これは `SourceClosureInput` の運用入力契約を固定する設計書である。現在
の入力型/materializer がこの契約を満たしていること、六つの生成 artifact
が存在すること、また KLE v0.1.0 の parity が完成したことを意味しない。

表記は次の三種類に分ける。

- **現状**: 現在の repository/docs から確認できる事実。
- **計画**: 次の実装 batch で実現する契約。
- **release blocker**: 欠けた場合に入力生成・merge・release gate を失敗させる条件。

## 1. Canonical input

### 1.1 所有と非生成性

**計画**: 非生成の入力ファイルを次の一つに固定する。

```text
artifacts/v0-1-0/source-closure-input/<katana-revision>/source-closure-input.json
```

このファイルは CI の三 profile probe と read-only provenance capture を
merge して初めて作られる。`source-closure.json` など六つの parity artifact、
静的 leaf 数、Storybook の manifest、テスト fixture から逆算して作っては
ならない。生成物を入力として再利用することも禁止する。

`SourceClosureInput` の wire format は `input_schema_version = "1"` の
versioned JSON とし、canonical JSON の field order、配列順、改行、UTF-8、
sha256 表記 (`sha256:<64 lowercase hex>`) を固定する。未知の root key、欠落
field、重複配列要素、非 canonical 順序は reject する。

### 1.2 型と各 field の evidence

入力の論理型は次のとおりである。`evidence` は説明文ではなく、同じ run で
保存された raw capture の相対 path と raw-byte SHA-256 を必須にする。

```text
SourceClosureInput {
  input_schema_version: "1",
  katana_seed_paths: canonical relative paths[],
  root: RootProvenance,
  profile_probes: [ProfileProbe(macOS), ProfileProbe(Windows), ProfileProbe(Linux)]
}

EvidenceRef {
  capture_id: registered capture operation ID,
  path: canonical relative path,
  sha256: sha256(raw bytes),
  command_or_source: exact command text or immutable source path,
  runner_label: exact runner label or "read-only-local-source",
  exit_status: 0
}

RootProvenance {
  schema_version: "1",
  katana_revision: full git SHA,
  katana_tree_fingerprint: sha256,
  katana_external_ui_fingerprint: sha256,
  user_mandated_extensions_fingerprint: sha256,
  kle_tree_fingerprint: sha256,
  kuc_tree_fingerprint: sha256,
  release_profile_matrix_fingerprint: sha256,
  generator_fingerprint: sha256,
  generated_at_utc: RFC3339 UTC, non-identity metadata only,
  evidence: {
    katana_revision: EvidenceRef,
    katana_tree: EvidenceRef[],
    katana_external_ui: EvidenceRef[],
    user_mandated_extensions: EvidenceRef,
    kle_tree: EvidenceRef[],
    kuc_tree: EvidenceRef[],
    release_profile_matrix: EvidenceRef[],
    generator_binary: EvidenceRef,
    generator_schema: EvidenceRef
  }
}

ProfileProbe {
  id: one of the three canonical IDs,
  runner_label: exact GitHub matrix label,
  rustc_host_triple: value from rustc -vV,
  rustc_vv_raw: EvidenceRef(command_or_source = "rustc -vV"),
  rustc_cfg_raw: EvidenceRef(command_or_source = "rustc --print cfg"),
  cargo_resolution_raw: EvidenceRef(registered locked-resolution capture),
  cargo_lock_raw: EvidenceRef(immutable Cargo.lock path),
  source_tree_fingerprint: sha256,
  source_tree: EvidenceRef[],
  active_edge_ids: canonical edge IDs[],
  inactive_cfg_edges: [{ edge_id, predicate, source_span }],
  cfg_edge_probe: EvidenceRef,
  fingerprint: sha256(canonical profile record)
}
```

`EvidenceRef` の `path` は input file からの relative path、`sha256` はその raw
bytes に対する SHA-256、`exit_status` は capture command の実終了値である。
`command_or_source` は説明文・推測・短縮表示ではなく、登録済み command の exact
text または immutable source path でなければならない。runner-owned evidence に
`read-only-local-source` を指定してはならない。

root の各 field は次の evidence へ一対一に写像する。

| root field | capture/evidence source | 受入れ条件 |
| --- | --- | --- |
| `katana_revision` | KatanA read-only checkout の exact command `git rev-parse HEAD` | 固定 revision `4f6a6287c650a38633c7baeb544a92e739c68567` と完全一致 |
| `katana_tree_fingerprint` | 同じ checkout の source-universe 対象 file bytes と sorted path/hash record | KatanA の未追跡・未保存差分を含めず、対象集合と raw bytes を保存 |
| `katana_external_ui_fingerprint` | KatanA `Cargo.lock` の locked package/source/checksum と、behavior-bearing UI crate source/symbol capture | `egui 0.36.1` 等の lock/source/symbol evidence が欠ければ reject |
| `user_mandated_extensions_fingerprint` | `docs/v0-1-0-user-mandated-leaves.json` の raw bytes | `replace.current` と `replace.all` の immutable input の raw sha256 |
| `kle_tree_fingerprint` | KLE checkout の対象 source/config/lock bytes | capture run の revision/tree と一致し、生成 artifact から逆算しない |
| `kuc_tree_fingerprint` | local KUC checkout の対象 source/config/lock bytes | KUC を編集せず read-only captureし、KLE inputには hash/evidence だけを渡す |
| `release_profile_matrix_fingerprint` | canonical順の三つの `ProfileProbe` の identity record と三 runner の raw probe refs | profile record の再計算値と完全一致。入力値を信頼して省略しない |
| `generator_fingerprint` | generator binary bytes と schema/contract bytes の capture | binary/schema の raw sha256 を同じ run で固定 |

KatanA、KUC、KLE の tree fingerprint は、各 repository の source tree を
同じ path normalization と sorted `path NUL file-sha256 LF` 規則で計算する。
local macOS の tree や Cargo 解決を他 profile の値として流用してはならない。

## 2. 三 profile の実測入力

### 2.1 GitHub matrix

**計画**: GitHub Actions の `macos-latest`、`windows-latest`、
`ubuntu-latest` 各 runner が独立した `ProfileProbe` を生成する。runner
label と `id` は固定順に次のとおりである。

```text
macos-latest, windows-latest, ubuntu-latest
```

各 runner は少なくとも次を同じ capture run に保存する。

1. `rustc -vV` の全 raw output と、その output に対応する host triple。
2. `rustc --print cfg` の全 raw output と raw-byte SHA-256。行の削除、並べ替え、
   手入力の target_os 追加を認めない。
3. locked Cargo resolution の feature/dependency graph raw output と SHA-256。
   graph は resolver の実測結果であり、Cargo.toml の静的列挙ではない。
4. その runner が読む KatanA `Cargo.lock` の raw-byte SHA-256。
5. source-universe 対象 tree の raw file capture と tree fingerprint。
6. AST/module/cfg scan が確定した active edge と、inactive candidate の
   predicate/source span。

`cargo resolution` の具体的な command line/flags は、この設計書では新規に
発明しない。次の実装 task が既存の CI entry point に対して command を確定し、
その command ID、raw output path、exit status、toolchain identity を契約に
登録する。未登録の手動出力、短縮 graph、別 command の代替出力は受理しない。

各 profile field と raw evidence の対応は次で固定する。

| profile field | evidence source | identityへの取り込み |
| --- | --- | --- |
| `id`, `runner_label` | GitHub matrix context と runner の実行 metadata | 両方を canonical record に含める |
| `rustc_host_triple` | `rustc -vV` raw capture の `host` 行 | triple と raw SHA を含める |
| `rustc_vv_raw` | exact `rustc -vV` raw bytes | raw SHA を含める |
| `rustc_cfg_raw` | exact `rustc --print cfg` raw bytes | raw SHA を含める |
| `cargo_resolution_raw` | 登録済み locked Cargo resolution capture | raw SHA を含める。静的 manifest は不可 |
| `cargo_lock_raw` | runner が使用した KatanA `Cargo.lock` raw bytes | raw SHA を含める |
| `source_tree_fingerprint`, `source_tree` | source-universe 対象 file bytes と sorted path/hash record | tree fingerprint と file evidence SHA を含める |
| `active_edge_ids`, `inactive_cfg_edges` | `cfg_edge_probe` の raw output と source span | edge ID、predicate、span、probe SHA を含める |
| `fingerprint` | 上記 identity fields の canonical serialization | assembly 時に再計算し、入力値を信頼しない |

**release blocker**: Windows または Linux の runner が欠ける、macOS の
結果を代用する、host triple/cfg/graph/lock/tree のいずれかが別 run のもの、
profile ID が重複または順序違い、inactive edge が欠ける場合は merge 失敗とする。

### 2.2 `⭐️` と日本語の位置づけ

`⭐️` (`U+2B50 U+FE0F`)、日本語、ZWJ、IME の表示・入力はこの入力契約だけで
UI 実装完了とは判定しない。各 profile の source/cfg/runtime provenance と
KUC color-face/host evidence を後続の execution/artifact record に結合する
ための profile identity を供給するだけである。ここで `⭐️` を `☆` に置換
したり、font fallback の成功を宣言したりしてはならない。UI proof が無い状態は
後続 gate の release blocker のままである。

## 3. Provenance と deterministic identity

### 3.1 固定 revision と外部依存

**現状**: source-universe は KatanA revision
`4f6a6287c650a38633c7baeb544a92e739c68567` を read-only reference とし、
KatanA の editor/UI/action/data-refresh/test-oracle edge と lock-resolved
external UI dependency を closure 対象としている。

**計画**: capture は次を検証してから入力を組み立てる。

- KatanA HEAD、実際の source bytes、対象 `Cargo.lock` が固定 revision と一致する。
- KLE/KUC の source/config/lock tree fingerprint は実 checkout から算出する。
- user-mandated JSON は raw bytes を hash し、意味を再シリアライズして hash しない。
- external UI evidence は lock package/version/source/checksum、source file bytes、
  invoking symbol/span を一組で保存する。
- generator binary と schema/contract bytes の fingerprint は profile matrix と
  独立に保存する。

### 3.2 `generated_at_utc` の扱い

`generated_at_utc` は既存六 artifact の共通 root に残るが、**identity field
ではない**。次の規則で既存 schema と deterministic identity の不一致を解消する。

1. `generated_at_utc` は capture/生成時刻を示す非同一性 metadata としてのみ保持する。
2. root fingerprint、profile fingerprint、generator fingerprint、artifact
   freshness/hash、canonical JSON の比較 bytes からは常に除外する。
3. canonical identity serialization はこの field を省略した root/payload を使用する。
   physical report に時刻を含める場合も、受入れ側は同じ omission rule で比較する。
4. 時刻だけが異なる physical JSON を同一 identity とみなし、時刻を変更して
   fingerprint を合わせることは失敗とする。

この規則を実装できない consumer は schema mismatch として release blocker に
する。現在の docs にある「全体の byte-stable output」と root の時刻 field の
同居は、この omission rule を実装するまで未解決の設計差分である。

## 4. Assemble / validate / merge

**計画**: assembly は profile raw evidence を profile ID で結合し、次の順で
fail-closed 検証する。

### 4.1 一時 staging と canonical input の分離

`target/source-closure` は CI runner の一時 staging root であり、canonical
`SourceClosureInput` の保存先ではない。各 workflow run はこの root を空の
run-scoped directory として開始し、既存の内容を読み込まず、run の終了時に
破棄する。ローカル作業ディレクトリ、checkout に残った `target`、前回の
workflow artifact、default profile、macOS の代替値を staging source として
扱ってはならない。

staging の期待 tree は次のとおりである。`<run-id>` は
`SOURCE_CLOSURE_RUN_ID` と完全一致し、profile directory 名と artifact 内の
profile/run receipt は相互に一致しなければならない。

```text
target/source-closure/<run-id>/
  profiles/
    macos-latest/{profile.json,raw/...,checksums.json}
    windows-latest/{profile.json,raw/...,checksums.json}
    ubuntu-latest/{profile.json,raw/...,checksums.json}
  provenance/...
  assembled/source-closure-input.json
  validation/input-validation-receipt.json
  materialized/...
```

profile artifact の展開直後に staging validator は次を検査する。

1. workflow run から取得した artifact 集合は、正確に
   `source-closure-profile-macos-latest`、
   `source-closure-profile-windows-latest`、
   `source-closure-profile-ubuntu-latest` の三つだけである。未知の artifact、
   欠落、同名の複数取得、余分な profile は reject する。
2. 各 artifact の root は一つの profile directory だけを含み、directory 名、
   `profile.json.id`、embedded run ID、artifact metadata の profile/run identity
   が一致する。`profiles/` 直下の未知 directory、profile file の重複、別
   profile の混在、nested duplicate path、空 directory は reject する。
3. 各 profile directory は `profile.json`、`checksums.json`、必要な raw evidence
   を全て持ち、checksums の対象と実ファイル集合が完全一致する。partial
   directory、空 raw file、path traversal、symlink、既存ファイルへの上書きは
   reject する。展開先の既存 path と衝突する場合も merge せず reject する。
4. profile の run ID、fixed KatanA revision、runner label、profile ID は
   staging 全体で一つの値に固定する。別 run の evidence は SHA-256 が一致して
   いても同一 run とみなさず reject する。

この structural validation が成功した後だけ、`assemble-input` は
`assembled/source-closure-input.json` を一時生成できる。`target/source-closure`
直下の `source-closure-input.json` を canonical と解釈してはならない。

### 4.2 canonical publication transaction

検証済み input の公開 tree は次の一つに固定する。

```text
artifacts/v0-1-0/source-closure-input/<katana-revision>/
  source-closure-input.json
  evidence/
    profiles/macos-latest/...
    profiles/windows-latest/...
    profiles/ubuntu-latest/...
    provenance/...
  validation/input-validation-receipt.json
```

ここで `source-closure-input.json` が canonical immutable
`SourceClosureInput` の唯一の path であり、入力内の `EvidenceRef.path` はこの
publication tree root からの canonical relative path とする。staging の
`profiles/` や `provenance/` を input から直接参照してはならない。

publication は次の transaction とする。

1. `assemble-input` は staging 内の一意な一時 assembly path にのみ書き込む。
2. `validate-input` は assembly JSON、全 raw evidence、三 profile の完全性、
   run/revision/tree/hash/canonical-order を再計算し、成功時に validation receipt
   を同じ一時 staging に書く。validation 前に publication tree を作成したり、
   既存 canonical file を更新したりしてはならない。
3. materializer は validation receipt を信頼せず、入力を自身の loader/schema/hash/
   canonical-order 検証で再検証する。この独立検証が成功しない限り、source
   closure artifact の生成・公開を開始してはならない。
4. loader verification と materialization が全て成功した後、同一 run の staging
   evidence を canonical publication tree の新規 temporary sibling に copy し、
   copy 後に path set と raw SHA-256 を再検算する。検算が成功した temporary tree
   だけを atomic rename で未使用の canonical revision directory として公開する。
5. 同じ `<katana-revision>` の canonical directory が既に存在する場合は、内容が
   同一でも上書きせず、canonical bytes/evidence/hash が完全一致するかを比較して
   既存公開を再利用する。差異があれば reject する。partial な公開 directory、
   temporary sibling、validation 前の copy は公開物として扱わず、次回 run の入力
   にも再利用しない。

CI artifact upload は staging の raw evidence と、最後に生成された validation
済み publication package の監査用保存であって、canonical path の代替ではない。
upload/download の成功だけでは `SourceClosureInput` の公開成功と判定しない。

**現状との差分**: 現行 workflow の `actions/download-artifact` の pattern 展開と
`target/source-closure` への merge は、この設計の exact-set、run-scoped directory、
collision rejection をまだ証明しない。次の実装では、download 結果を上記の
staging validator に渡し、validator の成功を `assemble-input` の前提にする必要が
ある。workflow が未実行であること、artifact が存在すること、既存の transient
JSON が読めることは、canonical input の存在証明にならない。

1. 三つの profile が各一件だけ存在し、canonical order と exact ID を確認する。
2. 各 raw capture の path、SHA-256、exit status、runner label、host triple を
   current run と照合する。
3. KatanA revision、KatanA/KLE/KUC tree、external UI、user input、generator
   fingerprint を再計算し、`RootProvenance` の主張を信頼せず検証する。
4. profile record を canonical serialization し、三件の順序付き matrix hash を
  再計算する。入力の `release_profile_matrix_fingerprint` と一致しない場合は reject。
5. active edge と inactive cfg edge の union が source closure の profile applicability
   を全て覆うことを確認する。profile-only edge、unclassified edge、unknown edge、
   foreign revision の edge は reject。
6. 全ての source path が current KatanA tree 内にあり、SHA-256、module resolution、
   lock/source evidence が current capture と一致することを確認する。
7. 検証済みの一つの canonical `SourceClosureInput` だけを materializer に渡す。
   materializer は渡された input を loader 側でも独立検証する。assembly 側の
   validation receipt、CI job の成功、artifact 名、staging path はこの loader
   verification を省略する根拠にならない。
   一部 profile のまま `source-closure.json` を生成する経路は作らない。

**release blocker**: 欠落、重複、stale/foreign revision、tree 不一致、partial
cross-run evidence、local static JSON、手動 profile override、default-host reuse、
空の unresolved edge を「成功」と扱う fallback は全て拒否する。

## 5. 次の実装 batch の lifecycle

### 5.1 操作順序

**計画**: 次の順序を固定する。ここで示す名前は lifecycle の operation ID
であり、未指定の最終 CLI flag を発明するものではない。

1. `capture-profile`: 各 GitHub runner が自身の raw probe と profile-scoped evidence
   を保存する。
2. `capture-provenance`: KatanA read-only revision、KLE/KUC tree、user input、
   external UI、generator/schema を同じ release revision に結び付ける。
3. `assemble-input`: 三 profile と root provenance を canonical order で merge する。
4. `validate-input`: raw SHA、revision、tree、matrix、edge closure を再計算し、
   不備なら materialization 前に終了する。
5. `materialize-closure`: validate 済み input だけから six-artifact pipeline の
   source closure stage を生成する。

各 operation の入出力と所有権を次のように固定する。

| operation | 入力 | 出力 | 検証・失敗責務 |
| --- | --- | --- | --- |
| `capture-profile` | 当該 runner の checkout と matrix identity | 自 profile の artifact directory | runner は raw command、profile ID、run ID、exit status、checksums を記録し、自己 profile 以外を書けない |
| artifact staging | 同一 workflow run の三 artifact | `target/source-closure/<run-id>/profiles` | staging validator が exact set、重複、未知、partial、path collision、run identity を拒否する |
| `capture-provenance` | 固定 revision と read-only KLE/KUC/KatanA/user input | staging の `provenance` | raw bytes と source identity を記録する。既存 provenance や local/default input を読み込まない |
| `assemble-input` | 構造検証済み三 profile と provenance | staging の `assembled/source-closure-input.json` | KLE parity checker が canonical順・schema・参照 path を組み立てる。未検証 profile は入力にしない |
| `validate-input` | assembled input と staging raw evidence | `validation/input-validation-receipt.json` | 全 hash、同一 run、revision、tree、edge closure を再計算する。失敗時は publication を作らない |
| `materialize-closure` | validation 済み input と固定 KatanA checkout | staging の materialized output | materializer が独立 loader verification を行い、成功後だけ artifact を生成する |
| canonical publication | loader 検証済み input、raw evidence、validation receipt | `artifacts/v0-1-0/source-closure-input/<katana-revision>/` | temporary sibling を再検算して atomic publish する。既存 revision は immutable、差異は reject |

各段階は、前段の成功を示すファイルが存在するだけでは成功扱いにしない。
各段階が直前の入力を parse し、identity と SHA-256 を再検算する。これにより、
staging path の差し替え、stale receipt、CI artifact の再利用、validation 済みに
見える local JSON の注入を拒否できる。

具体的な subcommand、flag、CI job wiring は tasks `12.1n2` と `12.1n13` の
実装対象として確定し、この設計書から推測して追加しない。

### 5.2 path、ownership、immutability

```text
artifacts/v0-1-0/source-closure-input/<katana-revision>/
  source-closure-input.json
  evidence/
    profiles/macos-latest/{probe.json,raw/...,checksums.json}
    profiles/windows-latest/{probe.json,raw/...,checksums.json}
    profiles/ubuntu-latest/{probe.json,raw/...,checksums.json}
    provenance/{katana,kuc,kle,external-ui,user-input,generator}/...
  validation/input-validation-receipt.json
```

この tree は canonical publication package の expected tree であり、CI の
`target/source-closure` tree と同一視しない。GitHub runner は自分の
`profiles/<id>` artifact だけを所有し、staging validator と KLE parity checker
は artifact set、assembly、validation を所有する。materializer は loader
verification と materialized output を所有し、canonical publication owner は
検証済み package の copy、再検算、atomic publish と immutable revision check を
所有する。KatanA/KUC はこの作業から編集せず、provenance capture の read-only
source である。capture directory と raw file は assembly 後に immutable とし、
上書きではなく新しい run/revision directory を作る。profile ID、revision、raw
SHA、run ID が一致しない既存 directory は再利用しない。

全 operation は欠落・不一致・未登録 command・非 UTF-8 source・parse failure・
unresolved edge・partial evidence のいずれかで non-zero failure とし、空 artifact、
placeholder、旧 input の持ち越し、static JSON の補完を出力しない。

## 6. Test plan

### Positive

- 三 runner の exact profile order、実測 host triple、raw cfg/graph/lock/tree SHA が
 一致し、canonical matrix fingerprint が再計算値と一致する。
- 固定 KatanA revision、KLE/KUC tree、user input、external UI、generator/schema の
  provenance が同じ run の raw evidence から再現できる。
- active/inactive cfg edge の classified union と module/source hash が一致する。
- `generated_at_utc` だけを変更しても identity comparison が同一になる。

### Rejection

- Windows probe 欠如、Linux probe 欠如、profile 重複、順序違い、macOS 値の代用。
- 未知の profile artifact、artifact 内の未知 directory、profile directory の
  `profile.json`/`checksums.json` 欠落、空 raw directory、duplicate path、symlink、
  展開先 collision、run ID のない profile、別 run の同一 SHA evidence。
- rustc cfg raw の一行削除/追加、host triple mismatch、Cargo graph mismatch、
  lockfile SHA mismatch、異なる run の profile 混在。
- KatanA revision の変更、foreign source path、KLE/KUC tree mismatch、user input の
  raw-byte変更、external UI package/source/symbol mismatch、generator/schema mismatch。
- matrix fingerprint の改変、active/inactive edge omission、unresolved/unknown edge、
  stale source hash、placeholder/default 値、static JSON の手動投入。
- 時刻を fingerprint に混ぜた入力、または時刻を除外できない consumer。
- validation 前の canonical directory 作成、既存 canonical revision の上書き、
  staging の `source-closure-input.json` の直接公開、materializer の loader
  verification を receipt/job 成功で省略する経路。

`⭐️`、日本語、IME、ZWJ の専用 UI test はこの入力契約の pass 条件ではない。
それらは三 profile の source/runtime identity と後続 KUC host/artifact evidence を
結合する linkage test として登録し、KUC の color glyph 実装や KLE の表示結果を
この設計書だけで主張しない。従って UI proof 未実施は明確な release blocker である。

## 7. 完了条件

この設計作業の完了条件は本書と tasks ledger の入力契約 subtask が存在し、
`git diff --check` が通ることだけである。入力 materializer、三 OS capture、
六 artifact、KLE/KUC/KatanA 実動作、Storybook、v0.1.0 release の完了は本書の
完了条件に含めず、未達のまま release blocker として残す。
