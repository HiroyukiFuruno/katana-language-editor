# カーソル移動の固定ソース参照

この文書は`text.cursor-navigation-family`の分岐解析を具体化する作業資料である。
KLE/KUCの実装合格表でも、全操作の網羅性証明でもない。全項目のKUC比較、
三OS実入力、AccessKit、KatanA採用後の最終cursor効果は未検証。

## Source Identity

- KatanA revision: `4f6a6287c650a38633c7baeb544a92e739c68567`
- KatanA `crates/katana-ui/src/views/panels/editor/text_edit.rs:51`の標準multiline
  と`:66`のshow。KLEからこのcheckoutは編集しない。
- egui: `0.36.1`、registry archive SHA256
  `c977ac91dfaa651633fd9722e4ce9ccb32cda4c748b89a5cb57e504036e37c13`
- `src/widgets/text_edit/builder.rs` SHA256
  `1c2198aa1c397d40e0e384cb6e69252822a6bdf870b8b3d9ddafb2dfd5f98cbd`
- `src/text_selection/cursor_range.rs` SHA256
  `ba2d32c55f92c472a58ea51fa2df7798cf8dd95dbf3b9bb414fa3119edba0161`
- 上記memberは実診断`requirement-bindings-20260905-r30.json`の認証済みsource集合。
  `CCursor`はUnicode scalarの位置であり、byte offsetやgrapheme位置ではない。

## 入力分岐

builder `events`はcursorイベントをmutationイベントより先に処理する。
cursor移動で本文が変わらないことと、selectionの変更通知は別の観測値である。

| source | 条件 | 固定ソースが定義する効果 | 残る検証 |
| --- | --- | --- | --- |
| `on_event` | Key pressed=true | `on_key_press`へ移譲 | 全modifier/OSの実入力 |
| `on_event` | Key released又は未対応event | false、cursorを更新しない | 他widgetのevent消費との結合 |
| `on_event` | AccessKit SetTextSelection、同じwidget/root tree、有効focus/anchor | primary/secondaryを対応cursorへ更新、h_pos消去 | native AX、行run/chunk境界、誤targetの拒否 |
| `on_key_press` | A + command | 全文選択 | modifier競合とnative shortcut |
| `on_key_press` | 左右、modifierなし、非空選択 | 左は小さい端、右は大きい端へcollapse | 逆選択、折返し境界のprefer_next_row |
| `on_key_press` | 左右上下HomeEnd | primaryをhelperで移動、shiftなしならsecondaryも更新 | 下記helper全分岐 |
| `on_key_press` | Mac + ctrl + !shift、P/N/B/F/A/E | helperへ移譲しsecondaryをprimaryへ合わせる | 実macOS入力、OS差分 |
| `on_key_press` | その他 | false | 未対応keyの保持 |

## 移動Helper

以下は`move_single_cursor`の分岐であり、Galleyとword境界の内部を解析済みとはしない。
`Mac && ctrl && !shift`が通常key分岐より先に評価されることに注意する。

| 条件 | 効果 | 未解決の下流 |
| --- | --- | --- |
| Mac ctrl !shift A/E | 行頭/行末 | Galleyの視覚行と論理行、折返し |
| Mac ctrl !shift P/N | 上/下へ1行 | h_pos維持、短行/長行、先頭末尾 |
| Mac ctrl !shift B/F | 左/右へ1文字 | scalar位置とgraphemeの差 |
| Mac ctrl !shift その他 | helperは即return | 呼出元によるselection collapseとの合成 |
| 通常Left/Right + alt又はctrl | 前/次の単語境界 | `ccursor_previous_word/next_word`、区切り、Unicode |
| 通常Left/Right + mac_cmd | 行頭/行末 | alt/ctrl優先と折返し |
| 通常Left/Right、上記なし | 左/右へ1文字 | scalar、境界、prefer_next_row |
| 通常Up/Down + command | 本文先頭/末尾 | shift selection、空本文 |
| 通常Up/Down、commandなし | 上/下へ1行 | h_pos、折返し、短行 |
| 通常Home/End + ctrl | 本文先頭/末尾 | OS先行分岐による到達性 |
| 通常Home/End、ctrlなし | 行頭/行末 | shift selection、折返し |

## 単語境界の下流

認証済み`src/text_selection/text_cursor_state.rs`のSHA256は
`0ceda3cb879cc72b363564fafef578ff6ef5d6f05155bb015d8780f7c2d1ccf5`。
以下は固定sourceの意味上の注意点であり、KUCの比較合格ではない。

- `ccursor_next_word`は`next_word_boundary_char_index`へ渡し、戻りcursorの
  prefer_next_rowをfalseにする。
- `ccursor_previous_word`はgrapheme単位で逆順の文字列を作って同じboundary関数へ渡す。
  scalar数との差で位置を戻し、prefer_next_rowはtrueになる。単なるbyte逆順ではない。
- boundary関数はUnicodeの`split_word_bound_indices`を使う。word内のドットも
  区切りとし、cursorより後のドット位置で止まる。
- word先頭へ止まる条件には`cursor_ci < word_ci && !all_word_chars(word)`がある。
  `all_word_chars`は各charがalphanumeric又はunderscoreかを検査する。
- 呼出し先のunicode-segmentation、反復器、grapheme処理、端点での挙動はさらに下流。
  ASCIIの単語移動例だけではUnicode全体の互換性を証明できない。

callHierarchyの直接呼出しはこの下流を発見する入口であり、implicit coercion、
動的dispatch、macro、分岐到達性を含む完全なグラフと同一視しない。

## 実行済みの限定証拠

### 直接Call Hierarchy

`target/acceptance/cursor-calls-epaint-lsp-20260906-r1/report.json`で、実rust-analyzerから
直接outgoing callを採取した。errorsは空、engine終了コードは0、r1/r2の
callHierarchy部分は一致した。prepared rootの識別子/型/URI、caller範囲、正確な
UTF-16抜粋、各fromRangeを検査し、空/null/不正結果を区別する。

| root | 呼出し先レコード | 呼出し位置 | epaint source identity |
| --- | --- | --- | --- |
| on_event | 3 | 4 | 0 |
| on_key_press | 6 | 9 | 0 |
| move_single_cursor | 10 | 20 | 8 |

この19レコード/33位置は機能数でも網羅率でもない。egui member内のtargetは
archiveとbytesのsource identityを認証しただけで、呼出し先の意味を解析済みとは
しない。epaintのbegin/end、行頭末尾、左右1文字、上下1行の8targetは、固定KatanAの
Cargo.lockで指定された`epaint 0.36.1`のcrate archive SHA-256、tar member、展開済み
source bytesを照合して`authenticated_epaint_member_source_identity_only`として記録した。
target名の意味、再帰探索・分岐到達性・KUC比較・native実入力は未実施である。
再帰探索・分岐到達性・KUC比較・native実入力は未実施である。

### Modifier参照

`reference_navigation_modifiers.rs`に6ケースを追加し、固定egui参照は合計38件成功。
Alt左右の単語境界と通常左右との差、Alt+Shift固定端、mac_cmd左右の行境界と通常
左右との差、mac_cmd+Shift固定端、command上下の本文境界、command+Shift固定端を
実Contextで確認した。mac_cmdの2ケースはmacOS上の参照だけである。
Ctrl+B/F/A/E/P/NやUnicode単語境界全体は追加検証していない。

### 前工程の固定定義参照

2026-09-05の実診断は`target/acceptance/cursor-lsp-20260905-r2/report.json`。
rust-analyzer 1.97.1で4呼出し位置を解決し、errorsは空、engine終了コードは0。
cursor_range内の呼出し元メソッド範囲はLSP documentSymbolで検査し、各edgeへ
保存した。定義の識別子range、impl内へのmethod包含、queryのmethod内包含を確認する。
builderの最初の呼出しは認証済みmemberの一意anchorから照会し、method rangeの
追加検査対象はcursor_range内の3照会である。全4辺に同じ証拠があるとはしない。

固定egui参照テストには以下の8ケースを追加し、既存24ケースと合わせ32件成功した。
全体Rustは601成功/1失敗（別経路の検索Close）で、公開合格ではない。

| 参照テスト | 観測した意味 |
| --- | --- |
| `arrow_left_and_right_collapse_forward_selection_to_min_or_max` | 順方向選択の左/右collapse |
| `arrow_collapse_uses_min_or_max_for_reverse_selection` | 逆方向選択でも小さい/大きい端を使う |
| `shift_arrows_extend_primary_and_keep_secondary` | Shift左で伸張、右で縮小、secondary維持 |
| `home_and_end_move_to_current_line_boundaries` | modifierなしの行頭/行末 |
| `shift_home_and_end_extend_from_the_primary_cursor` | Home/Endでprimaryを動かし固定端を維持 |
| `command_a_selects_the_full_scalar_range_without_changing_text` | 全文選択と本文不変 |
| `released_navigation_key_does_not_move_or_report_change` | 未押下eventの位置/本文不変 |
| `unicode_navigation_tracks_vs16_selection_collapse_and_japanese_scalar` | 星後scalar2、VS16後3、選択collapse、日本語後4 |

LSP診断はbuilderからon_event、on_key_press、move_single_cursorへの定義解決を
認証済みmemberと正確な識別子rangeで照合する。macro、Galley内部、word境界、
AccessKit、OS分岐の完全な意味解析を証明しない。

実行参照テストは本物のegui Context/TextEdit/StringへRawInputを渡して結果を
観測する。OSから到達した物理入力ではなく、フォント描画・日本語IME・VS16の
色付きglyph/hit testを検証した証拠としては扱わない。KUCのgrapheme要件を
固定eguiのscalar動作へ引き下げない。
