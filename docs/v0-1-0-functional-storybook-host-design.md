# Functional Storybook Host Boundary

## 判断対象

汎用UIは全てKUCに置く。現在のStorybookはKUC scenario sessionの表示と
opaque transportの受領を行うが、文書編集・検索・置換・undoの効果を持つhostではない。
この表示用sessionやdispatch件数を、機能完成の証拠として使わない。

本書は独立demo host案の設計である。文書操作までKUCへ置く案との所有判断を
非同期で確認中であり、本書をもって当該境界の承認済みとは扱わない。
既存の解析・依存更新・検証・公開準備はこの判断待ちに依存させない。

## 公開APIで確認した接続

KUC 0.4.2の`EguiTextCommandSurfaceHostProjectionEncoder`は、hostが所有する
presentation/styleをopaque tokenへ変換する公開APIである。KLEがそのtokenを
復号・再構築する必要はない。

`EguiTextCommandSurfaceHostProjectionLease::new`はtokenと
`KucRootEffectRouter`を受け取る。routerの`KucRootEventBatchContext`は現在rootの
revision/correlationとtyped text/toolbar/floating/search/context eventsを公開する。
hostは`KucOpaqueHostEffectBatch::from_handler`へone-shot効果を渡せる。
このAPIが存在するため、現在のno-op dispatcherから直ちにKUC機能不足と断定しない。

## 独立Host案の所有

| 所有者 | 責務 | 持たせないもの |
| --- | --- | --- |
| KUC | text area、行番号、toolbar/popup、検索・置換control、menu、scroll、font/emoji、計測、hit test、AccessKit | KatanA固有action名、ファイル運用 |
| KLE neutral/egui | provider契約、opaque lease retain/synchronize、root show、one-shot forwarding、型付き失敗 | 文書内容のコピー、UI部品、独自描画・座標計算 |
| 独立demo host | 文書・revision・selection・history、command binding、検索結果と置換transaction、Markdown操作の意味、host projection | widget、paint、font policy、glyph fallback、汎用hit test |
| Storybook launcher | 実hostの起動、操作シナリオ、媒体・数値観測の保存 | 固定fixtureで成功を返すhost、dispatch数からの効果推定 |

demo hostを採る場合は`publish=false`の独立packageに置き、KLE公開crateへ
混入させない。Storybook自身もraw文書stateを保持しない。KUCに汎用UIを置く境界は
維持し、既存AST規約の検査対象を新packageにも明示的に追加する。
ディレクトリ名で全検査を免除する実装は行わない。

## 実装計画と停止点

1. 所有判断後、OpenSpecのraw state禁止範囲とhost許可範囲を同時に確定する。
2. 公開APIだけの小さな結合テストで、KUC入力、one-shot host効果、次revisionの
   lease同期までを通す。routerのcallback件数ではなく文書の変化を検査する。
3. 固定KatanAの要件ごとに、入力・前提state・command・文書効果・undo/redo・
   read-only拒否を実装する。検索、単体/全件置換、全Markdown補助を省略しない。
4. source leafとhostの操作・結果を結合し、全profileの現行runへ接続する。
   seed画像、古い媒体、別runのreceiptで補完しない。

## 自動受入

実UI入力から最終文書・selection・dirty・history・検索結果を検証する。
単体/全件置換はUnicode境界、空query、no-match、read-only、stale revision、
複数一致、undo/redoを含める。Markdown補助は選択あり/なし、複数行、IME確定との
前後関係を含め、固定KatanAの意味と比較する。

日本語・正確な`⭐️`・色glyph・計測・hit test・AccessKitはKUCの記録を用い、
三OS実入力とsynthetic契約テストを区別する。動画/SSは判断用の補助資料であり、
自動受入を代替しない。KatanA #336は公開後の実host採用検証として別に扱う。
