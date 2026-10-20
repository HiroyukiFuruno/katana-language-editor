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
