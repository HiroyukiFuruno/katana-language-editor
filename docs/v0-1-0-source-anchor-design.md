# Source Anchor Contract

## 目的

要件の参照先が宣言又は分岐を持たない処理であることと、その要件の動作が
実装・検証されていることを分離する。source解析の不足をKUC待ちにせず、
宣言だけを動作の証拠に昇格させない。

## 一次情報

固定KatanA revisionは`4f6a6287c650a38633c7baeb544a92e739c68567`。
requirement ledger、source universe、canonical source SHAを引き続き使用する。
branchlessな参照も要件の母集団から除外しない。

既存の12件の`no branch found in source file`には次が含まれる。

- `state/command_inventory/view_commands.rs`のコマンド宣言。
- `state/command_inventory/file_commands.rs`のコマンド宣言。
- `app_action_types.rs`のaction型宣言。
- `tests/integration/editor/layout_persistence.rs`のテスト関数。

これらはsource存在の根拠になるが、実入力・dispatch・文書効果を証明しない。

## 境界

解析・証拠結合はKLEのparity toolが所有する。UI、入力処理、描画、font、
emojiを解析toolに実装しない。KUC/KatanAのsourceは読み取り専用とする。

## Source Anchor

分岐以外のsource事実には架空の`branch_id`を与えず、別のtyped recordを使う。

- `declaration`: enum variant、個別のcommand inventory item等のAST宣言。
- `straightline_test`: ASTで特定できるテスト関数。実行結果とは区別する。
- `behavior_branch`: 既存のbranch catalogと厳密に一致する分岐。
- `unmet_extension`: 要求された動作に対応する既存source routeがない状態。

各recordはrequirement ID、source path、source SHA、AST owner、行範囲、
原文excerpt SHA、semantic roleを持つ。source SHAだけの一致、名前検索だけ、
件数だけ、説明文だけではアンカーを受理しない。source bytesを固定revisionから
認証し、ASTから同じrecordを再生成して検証する。

canonical artifactへ組み込む際は、生成側と検証側が同じschemaとfingerprintを
使用する。古いartifactにdefaultで空の証拠を与えて合格させない。

## 要件と動作

宣言アンカーは要件の補助根拠であり、動作leafとは別の集合に保持する。
コマンドには宣言とdispatch先の分岐・効果を結ぶ明示routeが必要である。
各コマンド、各状態、各profileの不足は個別に残す。テスト関数のアンカーは
そのテストの登録・実行・合格やKLE hostでの動作を代替しない。

`replace.current`と`replace.all`は`AppAction::ReplaceText`の存在だけでは
実現済みにしない。検索UI、対象range/revision、単体・全件置換、no-matchの
効果が実装・検証されるまでuser-mandated extensionとして未達を保持する。

## 実装順と検証

1. 固定source bytesからASTアンカーを生成し、改変・曖昧・欠落を拒否する。
2. 要件ごとの宣言／テスト／動作routeを結合する。未知のbranchless参照は拒否する。
3. producerとpublic validatorに同じ契約を接続し、fingerprintと改変回帰を検証する。
4. 必要な動作leaf、KUC入力・効果、三OSの実行証跡を結合する。

sourceへの参照が解決しても、unbound branch、unmatched route、未実装extension、
実入力・効果・媒体の証跡不足が残る限りrelease completenessは不合格とする。
283 candidatesや診断件数を全機能の母数・達成率に読み替えない。

## Macro Token Recovery

`view_commands.rs`は宣言だけのファイルではない。固定sourceの`vec!`内に
`is_available` callbackの`if`があるが、通常のsyn visitorはmacroのtokensを
式として巡回しない。したがって現在の`no branch found`には解析漏れが含まれる。

限定recoveryは`vec![a, b,]`と`vec![value; count]`の二形式をsyn parserで
解析する。元TokenStreamを使い、文字列化・再構築によってspanを失わせない。
repeat形式も構文を一度だけ採取し、countの評価や実行回数の推測をしない。
`$(...)*`等のmacro定義用repetitionを通常のvec式として解釈しない。

bare `vec`のlocal macro定義、import alias、glob等の名前解決の曖昧さを
検出した場合はrecoveryしない。未知macroのbodyは解釈しない。安全に解析できた
場合もmacro boundaryとcallback boundaryの未解決根拠を保持する。
構文採取だけでstandard macroへの解決・入力由来・host effectを認定しない。

本番visitorの既存branch/call/action収集へ接続し、元sourceのcondition/body
spanとexcerpt SHAを検証する。macro parse failureは元macro spanと理由を
明示的に記録する。list/repeat、空list、nested macro、shadowing、malformed
tokens、日本語/VS16を含むspan、決定性、実固定sourceの利用可否分岐を回帰にする。
producerとvalidatorが同じ採取規則を使い、旧catalogをそのまま合格させない。

## Structural Edge Recording

現在の`ScanState::add_edge`は`to_path = None`を一律に未解決へ記録するため、
パスを必要としない`if`、`return`、`cfg`等も未解決source依存と混在する。
run 39229018204にはこの構造edgeが11,032件あり、callの26,235件とは区別する。

producer内で構造edgeと解決を要する依存edgeを型付きで区別する。
branch/CFG専用の記録経路は既知のcontrol-flow kindだけを受け取り、通常の
source/semantic依存は従来どおり未解決として保持する。未知のkindを構造edgeへ
変換しない。共通のedge ID、元span、kind、symbol、全edgeの採取は維持する。

validatorで既存の未解決edgeを名前によって除外する修正は行わない。
正しいproducerで再生成されたartifactだけが構造edgeの誤分類を解消できる。
branch catalogの全件採取とunclassified gateは維持し、構造が採取できたことを
behavior分類や実行成功に置き換えない。unknown call、macro boundary、module、
import、external UI semantic evidenceの未解決は削除・分類しない。

構造edgeはpath unresolvedを増やさないこと、未解決callは残ること、全branchは
引き続きunclassifiedであること、同一入力でbranch IDとspanが変わらないことを
回帰にする。この修正をsource completenessや全動作踏襲の完了とは扱わない。

## Analysis Revision

今回の採取規則の変更はcanonical bindingの必須`source_analysis_revision`で識別する。
値は`kle.source-analysis.v2`とし、producerが設定し、public validatorが厳密一致を
要求する。欠落・未知revisionは拒否し、旧artifactにdefaultを付けて受理しない。
共有ManifestRootのschemaとは別の解析契約のversionであり、全recordの既存root
整合・source/hash・未解決population検査は維持する。

この識別子は旧catalogの誤った再利用を防ぐためのもので、version文字列だけで
解析完全性・実行・署名やprovenanceを証明したとは扱わない。各実証gateを別途通す。

## Authenticated Anchor Catalog

source anchorはbranch IDを捏造せず、別の`kle.source-anchor-catalog.v1`として
生成する。入力はcanonical source-closure、checked-in requirement alias ledger、
固定revisionのGit blobである。working treeの内容や名前検索だけを採用しない。
各参照fileの元UTF-8 bytesをGitから読み、canonical file SHA-256と照合する。

catalogはsource-closureの実bytes hash、ManifestRoot、alias ledger hash、各fileの
path/hash/元source、AST由来anchorと未解決構文を保持する。公開検証側は既に認証された
canonical source-closureをauthorityとして受け取り、embedded sourceをfile hashで
結び直し、ASTを再解析してanchor全件・span・excerpt hashを再生成し、一致を要求する。
artifactから期待source hashを読み戻すだけでは認証にしない。

AST factはenum variant、function/test declaration、struct constructionを区別する。
aliasの参照先fileに存在するfactは、その要件を実装するとの証明ではなく、意味を
確認するための候補である。同じfileを参照する複数要件へ候補を関連づけても、
その全要件の動作routeが成立したとは扱わない。候補catalogだけでT2/T3の
未結合状態を解除せず、個別nodeと入力・効果の意味的な対応を次段で証明する。
test declarationはtestの実行・合格やbranchlessの証明ではない。struct constructionも
型解決・callback実行の証明ではない。owner、node kind、byte/line/column spanと元nodeの
SHA-256を保持し、同一ownerの複数nodeは元spanで区別する。

macro内の構文採取は既存vec recovery parserを共用する。独自の再tokenizeや
第二のmacro parserは追加しない。shadowing、未知macro、parse失敗、callbackの
未確定性は記録に残し、macroの不存在や実行可否を推測しない。

CLIを`emit-source-anchors`/`validate-source-anchors`として追加し、旧source解析の
問題を再現できる固定sourceと改変拒否を検証する。この段階ではcanonical leafや
release gateのunresolvedを減らさない。次段でproducer/public validatorへ正式結合し、
宣言から入力・dispatch・効果へのrouteがない限り行動要件は未達のままにする。

受入は、固定Git blobとの不一致、file追加/欠落/重複、未知field、別root、alias差替え、
anchor/node/span/excerpt改変を拒否すること、Unicode/CRLFの元bytesとspanを保つこと、
同一入力の決定性、branchlessな宣言/testを偽branchへ変換しないことである。

## 次段のcfg scope検証

現行`branch_catalog/record.rs`は`kind == cfg`だけにinactive profileを記録する。
そのため、無効なcfg itemの内側にある通常の分岐へ条件が継承されない。
さらに`leaf_validation.rs`はnon-cfg branchのinactive predicateを拒否する。
この不整合はKUCの公開待ちではなく、source解析・検証モデルの課題として修正する。

修正はvalidatorの拒否条件を削除するだけでは行わない。元sourceを再解析して
分岐のlexical ancestorと各cfg attributeの所有nodeを確定し、実profileが採取した
inactive cfg edgeのidentity/predicateへ結ぶ。入れ子のguardは積み重ね、いずれかの
ancestorが無効ならそのprofileでinactiveとする。未知predicate・未確定の所有nodeは
active/未解決のままにして、除外へ推測変換しない。

同じ行に複数nodeがある場合も一意に照合できるbyte/column付きspanが必要である。
現在のline-onlyなBranchRecordへ名前や近い行だけで結ばず、必要なschema/revisionの
変更と旧artifact拒否を先に確定する。生成側と検証側は同じ認証sourceからguardの
祖先関係を復元し、誤ったancestor、span、profile、predicate、重複・欠落を拒否する。
inactiveの修正をunclassified/T2/T3の全件処理や動作証拠の省略には使わない。
