# KLE v0.1.0 R4 source-derived leaf binding 設計

2026-09-05: T1/T2/T3 と直接参照診断に加え、T4の正式manifest生成・整合性検査を
接続した。未解決候補は保持して公開判定から除外しない。source/requirement binding や runtime evidence
全体が完成したとは判定しない。

## T4 の限定修正境界

### 外部ソース採取と意味解析の完了を分離する

現producerは5つの外部入口を採取するだけで、推移的な呼出し・分岐・動的dispatchの
closureを生成しない。diagnosticだけでなくcanonicalの共通出力でも未解析理由を
必ず保持し、入口とlock情報が揃うことを`complete=true`へ昇格させない。
後続の意味解析が実装されるまで、この制約を成功フラグや手動宣言で解除しない。

#### T4g: TextEdit builderからshowへの明示的な第一辺

固定KatanAのeditor本体は`TextEdit::multiline`でbuilderを生成したあと、同じ局所bindingの
`show(ui)`で実際の入力・layout処理へ入る。`multiline`だけを外部入口として保持すると、
builder生成を実行入口と誤認し、`TextEdit::show`の外部source定義を採取しない。この辺は
KatanAの`views/panels/editor/text_edit.rs`にある明示的なAST形だけを対象にする。

scannerは`let <plain-ident> = <method-chain>(TextEdit::multiline(...))`を検出した時だけ、その
identを`TextEdit` builder bindingとして同一function scopeに登録する。後続の
`<same-ident>.show(...)`だけを`TextEdit::show` invocationとして記録する。別型の`show`、
unqualified builder、field/tuple/pattern binding、closure越境、shadowing、macro、一般的なRust型推論は
解決済みと扱わない。`show`の呼出し又はegui側の厳密なdefinitionが欠ける場合は既存の
fail-closed missing-edge/definitionとして残す。

このpatchは`TextEdit::show`の内部推移、input event、IME、glyph、layout、KUC置換、実行証跡を
推測しない。従ってexternal semantic dependencyは引き続き`complete=false`であり、全semantic
closureやrelease判定へ昇格しない。

#### T4h: 認証済みegui内の直接semantic edge

`TextEdit::show`の定義には`TextEditState::load`と同一fileの`events`という、AST上で直接示される
内部辺がある。`TextEdit::load_state`と`store_state`にも同様に`TextEditState`への明示的な辺がある。
source hashが認証済みの各Rust fileを一旦すべてparseし、seed definitionを起点に直接の`ExprCall`と、
明示型parameter receiverに限る`ExprMethodCall`を採取する。targetは次の場合だけ記録する。

- 同一source fileに唯一存在するfree function。
- source集合全体に唯一存在する`Type::associated_function`。
- function parameterの明示型に一致するplain bindingに対するmethod call。
- `TextEdit::show`の明示`&Ui` parameterに対する、source集合で唯一のinherent
  `Ui::ctx(&self)`。receiverのreference kind、`Ui` struct、inherent methodはいずれも
  一意かつ非genericでなければならない。
- 同一blockで先行する`let <plain-ident> = Type::associated_function(...)`。associated
  functionはsource集合で一意で、明示return typeが`Type`又は`Self`の非chain callに限る。

同一scopeのmethod chain、`Option<Type>`等のwrapper return、trait method、destructuring、
field binding、closure越境は型を推論しない。shadowingするplain bindingは先行する型情報を
消去する。T4hは全ての同一scope型推論又はsemantic closureを完了としない。

唯一のwrapper例外は、source集合で一意なassociated functionの明示returnが`Option<Type>`又は
`Option<Self>`であり、同じsource集合の一意な`struct Type`が`#[derive(Default)]`を持ち、
直後に引数なし`unwrap_or_default()`を呼ぶ形だけである。この結果のplain bindingに対しては、
source集合の一意な`#[derive(Clone)] struct Type`に限り、引数なし`clone()`を一段だけ
type-preserving receiver chainとして扱う。これにより固定eguiの
`TextEditState::load(...).unwrap_or_default()`と`state.clone().store(...)`を構造的に結ぶ。
任意のwrapper、任意のtrait method、二段以上のchainは解決しない。

別の限定規則として、`TextEdit::show`の明示`&Ui` parameterに対する唯一のinherent
`Ui::ctx(&self) -> &Context`と、source集合で唯一のnon-generic inherent
`Context::method(&self, ...)`が確認できる時だけ、`ui.ctx().method(...)`の内側callを
edgeとして採取する。callの外側に続く`unwrap_or_default()`等のchainは追跡も型解決も
しない。したがってこの規則は内側callのsource spanだけを記録し、outer chainの結果、
到達可能性又はeffectを証明しない。`Ui`/`Context`のalias、trait、複数候補、generic、
値receiver、shadow、macro、closure、pattern bindingは受理しない。

edgeはfrom/target symbol、source file、exact span、kindをfingerprint対象として保持する。複数候補、
unqualified cross-file function、`Ui` trait/alias/複数inherent method、untyped又はchain local initializer、field/tuple/pattern binding、shadowing、macro、nested closure、trait/dynamic
dispatch、外部crate targetを解決済みにしない。採取対象外の呼出しを削除せず、既存の
`transitive semantic closure has not been captured`を維持する。したがってこのdirect edge集合は
全call graph、IME、glyph、layout、effect、KUC置換、host executionの証明ではない。

認証済みarchiveのRust source集合は、入口定義を含まないファイルもSHA付きで
保持する。symbol/spanは引き続き正確に採取できる5入口に限定し、全ファイルの
hash採取を全symbol解析と呼ばない。これにより後続の推移解析で必要なsourceを
診断出力から脱落させない。改変・追加・欠落拒否と決定的な並び順は維持する。
入口が全て揃った場合の未完了判定と、入口なしsourceのhash保持を回帰検証する。

出力側だけでなく公開validatorも外部依存レコードを検査する。eguiレコードの欠落、
package重複、未完了flag、未解決理由、replacement leaf不在、空のlock/source/symbol/
span/invocation/fingerprintを拒否する。これらは必要条件であり、手書きの完全flagや
非空文字列だけを推移解析の証明とは扱わない。全semantic graphと実行leafの照合は
後続であり、この限定修正を公開証跡の完成としない。

### 型解決エンジンによる外部呼出し診断

既存ASTの名前末尾推測を一般Rust型解決へ拡張せず、診断専用LSP clientで
固定KatanAの`text_edit.show`からeguiの`events`、`check_for_mutating_key_press`
への実定義解決を試す。固定lockのegui archive SHAと展開先member bytesを照合して
からdidOpenし、単一定義、認証済みURI、UTF-16 symbol rangeの一致を必須とする。
null、複数候補、別版、別ファイル、改変、曖昧anchorを成功に読み替えない。
KLE target配下へ診断JSONのみ生成し、Cargo offline/locked、build scriptsおよび
proc macro無効を維持する。UI実装の移植やsibling編集は行わない。

この固定した2辺は解析方式の検証であり、全call graphや全branchの証明ではない。
canonical manifestへは接続せず、semantic_complete=falseを維持する。
後続の動的dispatch、macro、プラットフォーム分岐と全要件への結合が必要である。

次の限定診断はbuilderの`delete_previous_char`からTextBufferの既定メソッド、
`delete_selected_ccursor_range`、`delete_char_range`宣言までを追う。
各source memberは同じ固定egui archiveで認証する。最後のtrait宣言への解決は
実装の選択ではなく、`unresolved_dynamic_dispatch`として記録する。
KatanA renderの`buffer: &mut String`は具体型の手掛かりだが、これだけで全イベントの
実行やKUCの同等動作を証明しない。UTF-16 query位置とソース内CharIndex、
実バッファのbyte index、UIのgrapheme位置を同一視しない。

### 固定eguiの実行参照テスト

次の診断は実LSPのcallHierarchyを使い、成功済み`follow_cursor_range`の定義結果から
認証済みcursor_rangeのon_event、on_key_press、move_single_cursorの直接outgoing callを
採取する。move_single_cursorの二つの呼出しsiteは同一rootに畳み、rootを重複させない。
prepare結果は単一item、name/URI/selectionRangeの完全一致、UTF-16として有効で識別子を
含むroot.rangeを必須にする。各fromRangeはroot内でUTF-16に正確に抜粋し、全edgeと全
fromRangeを安定順序で保持する。nullはunresolved、空、malformedとは区別し、rootのprepare失敗や
途中の整合性失敗はreport.errorsとなりpassingにしない。同じ固定egui memberへの到達は
archiveとbytesで動的memberのsource identityだけを認証する（target名の意味解決は
主張しない）。`epaint 0.36.1`のRust targetは、固定KatanAのCargo.lock checksumと
crate archive、tar member、展開済みsource bytesが一致する場合に限り
`authenticated_epaint_member_source_identity_only`として記録する。標準ライブラリ等はURIを
保持した`unresolved_external_boundary`として残す。未知targetを消さず、未解析呼出しがあることを
成功flagで隠さない。出力はsemantic_complete=falseかつtransitive traversal未実施を明記する。
これは3関数の直接呼出し採取であり、再帰的closure・分岐到達性・動的dispatch・
macro展開の完全性ではない。canonicalやpassing leafには昇格させない。

modifier参照は既存実Context harnessでAlt単語移動、mac_cmd行境界、command上下の
本文境界とShift固定端を観測する。プラットフォーム固有のCtrl分岐はそのOS上の
参照のみとし、OSを偽装しない。単語境界の全Unicode規則は別途残す。

カーソル移動の限定診断は、認証済みbuilderの`cursor_range.on_event`から
`CCursorRange::on_event`、`on_key_press`、`move_single_cursor`を実LSPで解決する。
同じarchiveで認証したcursor_range sourceの識別子rangeだけを受け入れ、呼出し位置、
別URI、複数結果、曖昧anchorを拒否する。これはGalley/word境界/AccessKit/各OSの
全推移解析ではなく、semantic_complete=falseの診断結果として保持する。

実行参照は既存の実Context/TextEdit/String harnessを使い、左右での選択解除、
Shiftでの選択伸縮、Home/End、command+Aと未押下eventを確認する。フレーム間に
cursor stateを再注入せず、本文・primary/secondary・changedを結果から照合する。
日本語やVS16を含む文字列はscalar indexとbyte offsetを区別するが、描画品質や
grapheme要件の合格証拠にはしない。KatanA実ホスト・KUC比較は別途必要である。

`katana-parity-check`のtest-only依存として固定lockと同版のeguiを利用し、
実Context/RawInput/TextEditとStringで削除処理を実行する。テストは
選択範囲・文字列・変更通知・カーソルを実際のフレーム結果から確認し、
TextBuffer既定メソッドをKLEへ複製しない。通常文字のBackspace/Delete、
境界no-op、選択範囲削除、read-only、Unicode scalar/VS16の差を参照対象とする。
境界no-opは本文不変を指す。固定eguiの実測ではBackspace/Deleteの境界操作でも
response.changed=trueであり、入力なしフレームのfalseと区別する。
VS16付き文字の末尾BackspaceではVS16だけが削除された。この参照結果を
KUCのgrapheme保持要件の代替にせず、既知の差として扱う。
この結果は固定eguiの意味を確認する参照証拠であり、KatanA実ホスト、
KUC比較、OS入力、全source leafの合格証拠ではない。実測で現要件と差があれば
要件を自動で弱めず、差を独立して記録する。既存ゲートは緩めない。

次の実行参照ではAlt/Ctrlの単語削除、mac_cmdの段落削除、Ctrl-H/K/U/W、
選択あり/なしと修飾なしを区別する。既存の実Context harnessを共有し、
イベントのmodifier以外にOSを上書きしない。WindowsのShift-Deleteガードなど
host-dependent分岐はこのmacOS参照実行で検証済みにしない。

入力履歴の参照は同一Context/Id/Stringを保持するtest-only harnessで行う。
フレーム間にTextEditStateを再設定して結果を作らず、Text/Keyイベントで入力、
Undo、Redoを実行する。RawInput.timeは単調な明示値とし、実装のstable_timeに
対応する入力なしフレームを使う。sleepやtimeout延長で履歴を成立させない。
本文・selection・changedを各段階で観測し、Redo後の復元、Undo後の新規入力による
Redo破棄、履歴なしのno-opを区別する。この参照もKUC/実ホストの履歴証明ではない。

### 対話Storybookの実行境界

対話ウィンドウは既存KUC rootの非artifact表示APIを使用し、描画のたびに
固定ディレクトリへ証跡を書かない。artifact生成モードのoverwrite拒否やreceipt
検証は変更しない。既定の対話起動はフレーム数で終了せず、明示した--framesだけを
有限実行とする。表示エラーはrun結果へ伝搬し、空の成功として返さない。
KLEが保持するのはwindowの寿命・frame budget・errorのみで、入力/文字/レイアウトの
実装はKUCに保持する。固定8 scenarioの制約およびKUC #40の未提供機能は残る。
`storybook-window-smoke`は同じnative対話経路を明示2frameで実行する。
headless artifact modeの成功を実window起動の証拠にしない。recipe単位の回帰と
macOSでの実起動終了を確認し、入力/IME/全機能の受入完了とは区別する。

### 旧設定更新メソッドの除去

`LanguageEditor::apply_settings`はKUCのpresentation policyとKatanAのdocument
policyを混在させる旧APIで、現行opaque rootでは使わない。まずこのメソッドを
neutral traitとFloem skeleton、neutral interface診断hostから除去する。
成功だけを返すshim、設定の破棄を隠す別名メソッド、KLE独自のupdate storeを
追加しない。診断hostのconfig生成はDTO構築の確認として区別し、実editorの設定
適用を検証したことにしない。

既存public traitの実装が設定メソッドなしでコンパイルできる正例と、旧メソッドを
traitに実装しようとした場合に拒否されるcompile-fail例を検証する。KUC opaque
rootの全体回帰も維持する。これはsettings/typography/spacing DTO全体の除去や
KatanAの設定routeの実入力証明ではなく、tasks 6.5の旧mutation入口の限定除去。
CIの`just unit-test`はall-targetsに加えworkspaceのdoctestを明示実行し、公開APIの
compile-fail検査を通常ゲートから漏らさない。

### 旧config内のgeneric設定保持の除去

`EditorConfig`と`EditorConfigInput`から`typography`/`spacing`/`settings` fieldを
除去する。新しい別名field、map、optional storeへ移さず、既存KUC opaque rootの
presentation/input policyを唯一のgeneric実行経路とする。neutral interface診断の
DTO構築と、現在未接続の旧Storybook config sourceも同じfield構成へ同期する。
利用していないfont/spacing token読み出し、autosave/shortcut設定生成は残さない。

AST lintはneutral crateのこの2つのstructを構文で識別し、上記3fieldがprivateで
再追加されても拒否する。汎用struct全体を同名fieldだけで禁止せず、無関係な型を
誤検出しない。空に見せる文字列検索ではなく実fieldを調べる。設定型自体のpublic
export残存と、host/KUCでの設定動作の全leaf検証はこの工程では未完了のまま残す。

### Storybookのretained provider同期

公開KUC 0.3.6の`FullTextCommandSurfaceScenarioSession`はdispatch結果を共有stateへ
適用し、`synchronize_lease`で更新presentationと次revisionのleaseを発行する。
初期leaseだけを取得してproviderを破棄すると、この更新経路を消費できない。
Storybook `InjectedProjection`は既存の公開`EguiTextCommandSurfaceEditor<Provider>`を保持し、
対話frameとartifact frameの両方を同じprovider同期付きpublic経路へ接続する。
`show_current_*`を呼ぶ別経路や手動のKUC state更新は残さない。
`HostProjectionBinding`は公開editor内部に保持されるものとし、Storybookが直接
使わない。既存のdirect-kle-binding禁止lintを維持する。

provider retainは構築時に一回、synchronizeは各表示呼出しで一回とし、失敗は
表示成功やartifact成功へ変換しない。実KUC scenario sessionを用いた連続frame、
既存全scenario artifact/receipt回帰、native smokeを再検証する。計数用のtest wrapperは
実KUC sessionへの転送だけを行い、入力結果やKUC返却値のmockを作らない。
この同期修正はneutral trait移動と独立に、現在の公開APIだけで実施する。

### neutral provider接続設計

移行前のegui実装はneutralの`LanguageEditor`を実装せず、egui側定義の
`HostProjectionProvider`を使っていた。neutral契約と実経路を結び付けるため、
provider trait/errorをneutralの唯一の定義へ移し、`type Lease`でvendor型を分離する。
egui adapterは`Lease = EguiTextCommandSurfaceHostProjectionLease`の等価制約で
実KUC leaseだけを受ける。既存egui import pathは同一定義の再exportとし、二重trait、
String/Anyへの消去、payloadのclone/decode/serializeを追加しない。

編集範囲はneutral新module/lib、egui provider/lib/root editorと各provider test、
downstream probe、Storybook provider/runtimeの型制約。全impl/useの呼出しを
確認し、neutral単体の正例、誤Leaseのcompile-fail、既存KUC leaseのmove制約、
実KUC retain/synchronize/forward回帰を必要とする。associated type自体はnon-Cloneを
保証しないため、実KUC型とbindingでの所有権移動を保証点とする。
この接続は実装済み。neutral単体の正例と実KUC leaseの誤型・move・clone拒否を
doctestで確認した。旧neutral DTO/control全体の整理と全機能証跡は別途残る。

境界の再発防止として、workspaceのproduction Rust ASTで
`HostProjectionProvider` traitと`HostProjectionProviderError`型の定義を
neutral crateの`src/host_projection.rs`だけに許可する。adapter側の`pub use`は許可し、
同名trait/enum/struct/type aliasの再定義はprivate、nested、raw identifierでも拒否する。
別名による意味的な複製まで証明する検査ではない。文字列/commentをコードと誤認せず、
型同一性と実KUC lease等価制約のcompile検査は引き続き別途維持する。

実施時のneutral-only検証は`katana-interface-check`も新providerへ切り替える。
これはcompile-only consumerであり、旧KatanA風editorの本文/config/clipboard実装を
用いて機能互換を示すものではない。未接続hostはtyped missing projectionを返すだけとし、
fake KUC leaseや疑似的な編集成功を作らない。実KUC leaseの正例・誤型拒否・一回消費は
egui側の実KUC回帰と公開API doctestで別途検証する。

### neutral style DTOの二重所有を解消する限定設計

`Typography`/`Spacing`はneutralのstyle moduleとroot/types再exportだけに参照が残る。
config fieldは既に除去され、現行egui editorはKUC opaque projectionを使用するため、
未使用の2型とそのexportを除去する。font family、size、weight、line height、
letter spacing、padding、radius、gapをKLEの別名DTO/defaultに移さない。
KUC公開APIやhost projectionの内容をこのパッチで変更しない。フォント設定、描画、
計測、hit test自体の機能要件は残し、型除去をその機能検証として扱わない。

編集はneutral `style.rs`一ファイルの除去と`lib.rs`/`types.rs`の公開接続修正に限定する。
rootとtypesの各パス・各型を独立したcompile-fail doctestで拒否し、neutral consumerと
実KUC adapterの既存検証を維持する。ASTはneutral sourceで同名struct/enum/typeの
再定義を拒否し、文字列や無関係な型、neutral外の定義には適用しない。別名の意味的
複製や全UI設定APIの移行を完了したとはしない。settings/theme/localizationは別工程。

### neutral settings DTOの二重所有を解消する限定設計

settings moduleの7型（EditorSettings、AutosavePolicy、ShortcutMap、ShortcutBinding、
KeyBinding、KeyModifier、SemanticAction）はmodule内部・export・旧拒否doctestだけで
使われている。settings.rs一ファイルとroot/types exportを除去し、KLEのshortcut競合
計算も残さない。host autosave/shortcut arbitrationとKUC input policyの機能要件は
削除せず、設定specのsource-derived proofとして残す。別名storeや互換shimは作らない。

旧apply_settings拒否doctestはEditorSettings importを取り除き、既存の型だけで成立する
trait実装へunit型の旧メソッドを追加する形にする。型の未解決ではなくtrait非所属を
拒否理由とする。各型・root/typesの14独立compile-failも追加して各exportの復活を検査する。
ASTはneutralの同名struct/enum/type aliasを拒否する。既存config/styleの規則は維持し、
private/raw/nested、無関係な型、literal、path scope、解析エラーの回帰を確認する。
旧ShortcutMapの内部unit test一件の除去は不要APIの除去に伴うものであり、hostの競合
機能の検証を完了・不要とはしない。theme/localization/control/error APIは本変更外。

### generic presentation config fieldの除去

EditorConfig/Inputに残るtheme/strings/localeは現在のopaque editor経路で使わず、
neutral内のconstructorと未接続StorybookConfigFactoryだけが参照する。3fieldとその
constructor転送を除去する。Storybookのconfig.rsはmainにmod宣言がなく、他の実呼出し
も存在しないため一ファイルを除去する。KLEローカルpalette/英語preset/clipboard storeを
別の場所へ移さない。現行KUC scenario session/root、入力、描画は変更しない。

neutral configのsyntax_highlighter/clipboard、Theme/Strings/Locale型そのものは別工程。
2structそれぞれの3fieldアクセスを6独立compile-failで拒否し、残るconstructorの型が
コンパイルできる正例とneutral consumerを確認する。ASTの対象fieldを6種類へ拡張し、
private/raw/nested再導入、無関係型/literal、path scopeを検査する。通常ゲート維持。
色・文字列・RTL等の機能要件はhost presentationとKUCの同一frame証跡で要求し続ける。
旧KDV preset必須化やKLE config DIの指定は現在の責務境界へ修正するが、視覚・入力・
翻訳キーの網羅性をconfig除去だけで完了扱いにしない。

### neutral theme DTOの除去

Rgba/ColorTokens/ColorTokensInput/Theme/EditorThemeはtheme moduleとroot/types export
以外に実コード利用がない。theme.rs一ファイルと公開接続を除去し、各型・各公開パスの
10独立compile-failを追加する。KUC theme型のKLE再exportや別名color DTOを代替にしない。
既存style定義所有権ASTの対象へ5名を加え、型の種類・private/raw/nestedと無関係型/
literal/path scopeを回帰する。KUC/hostのsemantic color roleとtheme切替機能の要件は残す。

literal color lintの修正案内は削除するKLE Themeへ誘導せず、KUCがhost presentationの
色を解決する境界を案内する。literalの検出条件・severityは変更しない。非literal引数
のunit fixtureがliteral判定に該当しないことはKLEローカルrendererを許可する証拠ではない。
この工程でもpalette、pixel、native inputは変更せず、見た目の互換性を型除去で証明しない。

追加監査でcolor ruleがfile-level allowとtests/kdv-presetsというpath componentだけで
対象を除外し、attribute ruleも同じallowを許可していることを確認した。この回避経路は
撤去する。色ruleは対象3crateのsrc境界だけでfileを選び、既存の明示的cfg(test) visitor
処理は維持する。checkout祖先やsrcのsubmodule名にtests/presetがあっても免除しない。
global attribute ruleはcolor専用allowも他のallowと同様に拒否する。構造化SourceFileを
使うfixtureで対象path・対象外path・file-level allowを検査し、attribute ASTの回帰も
追加する。新たなallow/除外を追加して既存gateを通すことは禁止する。

### 色診断のliteral/hintを構造化する

既存color specが要求するliteral/hintは現在のViolationに存在せず、案内文だけでは
未達である。ViolationにOption<String>のliteral/hintを追加し、既存newはNoneのまま、
color ruleだけがwith_literal_hintで両方を供給する。既存の他ruleとreport先頭行を維持し、
追加値は改行等をescapeした補足行へ出力する。色判定条件やseverityは変えない。

ColorLiteralVisitorはSourceFileのsourceを借用し、同じsource由来のAST spanで
constant path/full constructor call/string literalの原文を取得する。列をUTF-8のbyteと
取り違えず、日本語/emoji、CRLF、複数行、raw stringを検証する。token再整形や空の
placeholderで代替しない。範囲を取得できない場合はDiagnosticSpanエラーを返し、
lint成功や部分的な違反一覧へ変換しない。既存ColorLiteralPatternsの判定強度は維持する。
追加のspec照合でhsl/hsla文字列の検出漏れを確認したため、RGB系と同じtrim/case規則で
この2prefixを追加し、正例・無関係文字列の回帰で既存明示要件へ合わせる。

検証は診断の既存new/report互換性、追加field/report escape、実sourceでの位置と原文、
範囲外span拒否、既存scope/allow回避拒否と全gate。これはlintの診断要件であり、
editorのUI/全leaf/三OSの完成とは分離する。依存追加・sibling編集は行わない。

### 外部pathを現在moduleへ誤解決しない

`resolve_rust_name_path_candidates`は問い合わせたpathの末尾を短縮して既存のRust
module fileを探すが、問い合わせの先頭まで捨てて現在moduleだけを返してはならない。
固定診断には`std::env::consts::OS`のto_pathが呼出元about_info/mod.rsになる例がある。
探索の下限は、現在moduleやself/superの基底だけでなく、問い合わせ由来のmodule
segmentを少なくとも1つ残す位置とする。基底だけの一致はsymbol定義の証明ではない。
未対応の同一file内symbolは引き続き未解決として残し、定義indexへの接続を別途行う。
実在する子module、crate/self/superのmodule参照、同階層の曖昧候補の保持を回帰で確認する。
ScanStateのpending edgeを解決済みへ変更する作業はこの修正と混ぜない。

### 正式manifestへの構造化接続

次の実装では旧 `forward_route` の文字列joinを正式生成経路から外し、
`leaf-manifest.json` の必須payloadに要件inventory、T2/T3の全入力report、
構造化候補reportを保存する。任意sidecarや候補だけの抜粋は採用しない。
要件本文はコンパイル済み固定入力とbytes一致を確認し、inventoryの本文SHAを
loaderでも固定入力へ照合する。alias/rootの検証も維持する。

loaderは固定要件からinventoryとT2を再生成し、保存値との一致を確認する。
T3のfingerprint/親/root/source factsを検査し、構造化候補を再生成して保存値へ
照合する。これにより未結合binding、zero-reference route、未解決、unbound
branchを削ったreportは、hashだけ合わせても採用しない。ただしT3自体の全source
fact採取の完全性は別途scan証拠が必要であり、hash検査だけでは証明しない。
leafは構造化candidateのidentityと対応し、候補投影の段階ではstatusをcandidateに
固定する。全OSのprofile、実入力、KUC component、opaque outcomeが未証明のまま
passingへ変更する経路を追加しない。正式release validatorの既存拒否条件は維持する。
候補から生成したleafのID、source candidate fingerprint、要件/span、branchとprofileは
不変の対応として検査する。将来の実証拠に基づく意味分類まで恒久的に禁止するために
全fieldを未解決の生成値へ固定してはならない。実証拠を受理する別gateが必要である。
整合性検査とclosure合格を分け、正式releaseでは未分類のinventory/T2/T3、未結合、
要件未参照routeを明示的に拒否する。単に保存しただけで合格条件から外さない。

作業順は必須payloadと再構築検査、materializerへの接続、既存fixtureの移行と
全ゲート再検証とする。複数artifactの中断耐性を持つ公開方式への変更は独立工程で、
この接続だけを原子的公開の完成とは呼ばない。

### 外部ソースの読み取り境界

#### 固定vendorのplatform input root

固定KatanAのCargo.tomlはegui-winitを`vendor/egui-winit`へpatchしている。
Cargo.lockの同packageにはregistry source/checksumがないため、registry archiveへ
置き換えてはならない。`vendor/egui-winit/src/lib.rs`を明示rootに追加し、通常の
固定Git blob照合とmodule追跡で`on_window_event`、`on_ime`、`on_keyboard_input`、
clipboard接続を採取する。source universeとroot fingerprintは同時更新する。
これらのsource採取はOS実入力の実行証拠ではなく、KUC generic platform inputの
置換要件を導く入力である。winit/clipboard/accesskit等のregistry依存の推移解析は
別途必要であり、vendor rootの追加だけで入力経路を閉じない。

#### 固定shortcut呼出しの型境界

固定KatanAの`command_shortcut_consumed`は`ctx: &egui::Context`から
`ctx.input_mut(|i| i.consume_shortcut(&parsed))`を呼ぶ。egui 0.36.1の定義は
`src/input_state/mod.rs`の`InputState::consume_shortcut`であり、
`Context::consume_shortcut`は存在しない。必要入口と採取定義をこの実型へ修正する。
呼出しedgeは型確認済みContextのinput_mutの単一closure引数から渡されるbindingに
限定して採取し、直接Context呼出し、他型の同名method、shadowing、closure外の
同名変数を認証しない。macroや未知型は未解決を保持する。固定ファイルの実診断と
同じ形のpositive/negative fixtureを検証し、一般Rust型推論の完了と扱わない。

#### Lockからarchiveへのidentity接続

次の実装は固定metadataが選んだCargo registry package rootから、同じregistry cacheの
`egui-<version>.crate`を解決する。標準registry layoutとpackage名/versionの一致を
確認し、存在しないarchiveや未知layoutを展開済みchecksum台帳で代替しない。
archiveのSHA-256を固定Cargo.lock checksumと照合してからgzip/tarを解析する。
OSのtarコマンドやfilesystemへの展開は用いず、flate2/tarのRust APIを使う。

archive全体の圧縮bytesは64MiB、展開bytesは256MiBを上限とする。超過・破損は
明示的な未解決であり、上限までの部分集合を成功として返さない。archive内は正しい
package prefix配下の相対POSIX pathを要求し、重複、traversal、symlink/hardlink、
非通常fileを拒否する。directoryはpath検査後に内容解析対象から外す。
認証済みarchiveのRust file集合を一次情報とし、展開先のRust file集合との完全一致と
各fileの実bytes hash一致を確認してからsynへ渡す。
`.cargo-checksum.json`はvendor directory source用であり、通常のregistry cacheには
存在を要求しない。存在する場合は追加の整合性制約としてpackage/hash/path集合を
照合し、不正・symlink・読取失敗を無視しない。欠落時にcache側へ台帳を生成しない。
この修正は2026-09-05の実診断r18が標準registry配置を誤拒否したことに基づく。
根拠: https://doc.rust-lang.org/cargo/reference/source-replacement.html と
https://doc.rust-lang.org/cargo/guide/cargo-home.html 。
hash不一致のfileからsymbols/spanを採取しない。

回帰は正しいregistry layoutの実gzip/tar fixtureをRAII directory内に作り、正常、
archive改変/欠落/破損、sourceとchecksum同時改変、Rust file省略/追加、重複path、
prefix違い、traversal/リンク、サイズ上限を検証する。既存のパス拒否検査は維持する。
これはpackage由来を確定する工程であり、5入口以降の推移的semantic closureや
同時filesystem差替えに対するhandleベースの保護を完成扱いしない。

固定source診断CLIでも同じarchive照合を使用する。checkoutとCargo.lockの固定blobを
照合してからregistry sourceを採取し、そのreportのfingerprintを診断rootへ結合する。
呼出し解析にはsource scannerが固定blobと照合した同じbytesだけを渡す。別のpath集合や
未照合の再読込を足さず、重複走査は呼出し入力へ二重登録しない。固定ソースの全採取済み
呼出しを既存invocation visitorへ接続し、missing seedやmacro/dynamic/type ambiguityを
reportへ保持する。推移的semantic closureは未採取と明示し、外部dependency
reportのcompleteと診断全体のpassing/executionはfalseのままとする。読み取り失敗は
reportの未解決に保持し、実行例を単体fixtureだけで代替しない。

固定`editor/paste.rs`の`egui::Event::Paste(text)`は式でなくmatch patternである。
診断接続ではこのpattern spanもevent入口として採取し、テスト内のEvent構築だけを
本番paste入口の証拠にしない。constructorとtuple patternを区別し、末尾文字列だけが
似た`OtherEvent::Paste`を受理しない。macro内部は展開済みの証拠がない限り未解決。

外部dependencyの任意checksum台帳からRustファイルを読む前に、台帳自身がpackage
root内の通常ファイルであることを検証する。台帳内パスは相対POSIXパスに限定し、
空要素、`.`/`..`、絶対パス、Windows prefix/backslashを拒否する。path各要素の
symlinkを拒否し、canonical pathがpackage root内の通常ファイルであることも確認する。
未知・不正パスは明示的なunresolvedとし、外部ファイルのbytesを読まない。
この検査はキャッシュの出自やarchive checksum一致を証明しない。archiveからの
identity検証と推移的semantic解析は引き続き別の未完了条件である。

### 構造化leaf候補の診断接続

この節は正式manifest接続前の段階的設計記録である。現行の生成範囲は上記
「正式manifestへの構造化接続」に従い、診断限定という当時の制約は更新済みである。

T2のRequirementBindingResultとT3のSourceActionBindingReportを、sourceとbranch
catalogへ再照合する。既存binding IDは分割せず、同じ生成関数による完全一致keyで
参照する。各候補は要件binding、branch、action routeの構造化した根拠を保持する。
root全項目、両reportのfingerprintと親fingerprint、source SHA、branch ID/file/span/
excerpt SHA、生成箇所の行範囲包含を確認し、不一致や重複参照を拒否する。
定義・生成・dispatchの各source factも実source集合と照合する。

出力は診断CLIの`source_leaf_candidates`に限り、canonical leaf manifestには
まだ接続しない。未結合binding、参照のないroute、T2/T3の未解決項目を残す。
候補IDは要件bindingとrouteの構造化bytesから決定し、行単位の精度限界を明記する。
未知項目の除外や、runtime outcome/三OS実行/passingの付与は禁止する。
これが旧文字列joinを置き換える前の検証可能な接続点となる。

### 旧leaf joinの入力整合性

構造化T2/T3接続に先立ち、旧joinもsource/branch/actionのroot全14項目を
比較する。source pathとbranch IDの重複、source集合にないbranch参照は
候補を生成する前に拒否する。全branchが正しいsourceを参照するがactionの
該当routeがない場合は引き続き候補ゼロとなり、passingを意味しない。
回帰では各root項目をbranch/actionの片側だけ変更し、空の候補集合でも拒否を
確認する。既存の未知sourceを黙殺するテストは正しい入力拒否のテストへ訂正する。
未知の意味分類やrouteの推測はこの修正では行わない。

- 要件表の暗黙参照は、固定 KatanA ソースの実在する定義・分岐を読んでから
  明示パスへ置換する。要件内容、実装状態、受入条件は変更しない。
- 同じファイルに直接参照と directory 参照がある場合も、それぞれの参照根拠を
  診断に保持する。参照集合と requirement/file の結合集合は区別する。
- `crate::app_state::AppAction::CopyPathToClipboard` のような修飾パスは
  未解決 construction 候補として記録する。修飾パス末尾の名前が一致するだけで
  indexed definition へ解決したとは判定しない。元のパス、variant、span と
  未解決理由を保持し、既存の direct path の扱いを変えない。
- 回帰テストは実 Rust parser を使い、修飾 path、異なる enum、macro、
  複数参照の共存を検証する。未知候補の増加を失敗とみなして捨てない。

## 結論

以下は初期T1からの設計経緯である。canonical生成を変更しないという初期の範囲は
上段のT4設計で更新した。UI所有境界、未証明項目を合格にしない条件は引き続き有効。

R4 の最初の実装単位は、checked-in の requirement source alias ledger を
`SourceClosureArtifact.files` と `BranchCatalogArtifact.branches` に決定的に結合し、
「要件 ID - KatanA source file - branch span」の source-derived binding 候補だけを
生成する純粋な処理と回帰テストである。`ActionOriginsArtifact` の unresolved な
action、handler、最終 effect をこの段階で補完しない。既存の
`GeneratedLeafJoinArtifact`、`LeafStatusRecord`、canonical artifact の公開経路は
変更しない。

これにより、KUC #40 の汎用 artifact 能力や KatanA #336 の downstream host E2E を
待たずに、固定された KatanA revision に対し、要件の source alias が実際に走査された
source file と branch に結び付くかを検査できる。KLE v0.1.0 の公開前には、全 leaf の
KLE public input、KUC frame、AccessKit、KLE-owned observable result 又は明示された
no-mutation outcome の証拠が別途必要である。KatanA 固有の action/document/filesystem/
history/workspace host effect は、公開済み KLE を #336 が採用した後の責務であり、KLE
release prerequisite ではない。一方で、この binding 自体は runtime 効果、KLE 実装、
または要件全体の完全性を証明しない。

## 1. 現行境界

### 入力

- `docs/v0-1-0-editor-requirement-source-aliases.json` は schema `1`、固定
  KatanA revision、`entries` (`requirement_id`, `short_reference`, `source_path`) と
  `non_katana_references` を持つ。
- `RequirementSourceAliasLedger::validate_checked_in_coverage()` は、要件文書の
  short reference と alias ledger の組を一意に分類し、KatanA UI source path が
  source universe と root manifest に含まれることを検査する。これは手書き台帳の
  自己申告を合格へ変換する処理ではない。
- `SourceClosureMaterializer::materialize` は固定入力を scan して
  `SourceClosureArtifact` を組み立てる。各 `SourceClosureFile` は `path` と
  `sha256` を保持する。
- `materialize_branch_catalog` は `cfg`、`if`、`match` edge を `BranchRecord` に
  1 対 1 で記録する。現在は全 `branch_id` を
  `BranchCatalogArtifact.unclassified_branch_ids` に入れる。
- `materialize_action_origins` は `AppAction` definition / construction / dispatch
  route の source facts を採取するが、`origin_classification` を `unresolved` とし、
  `unresolved_action_origins` を残す。

### 型と現在の出力

- `BranchRecord` には `branch_id`、`file`、`symbol`、`span`、`kind`、`condition`、
  profile IDs、`source_excerpt_sha256` がある。
- `ActionOrigin` には action definition/construction/route と未解決状態を記録する
  fields がある。handler 又は effect の解決済み型ではない。
- `GeneratedLeafJoinArtifact::build_leaf_join_candidates` は file を route が言及する
  branch/action の組から `GeneratedLeafJoinCandidate` を作る。生成 leaf は
  `status = "candidate"` で、visible path、KUC、KLE public-show、opaque transit、
  actual host effect、execution、Storybook stage を `unresolved:*` に固定する。
- `SourceClosureMaterializer` は上記 candidate を `as_leaf_manifest` で
  `LeafManifestArtifact` としてシリアライズする実装を持つ。ただし、現時点で
  全 branch/action が未分類・未解決であり、検証済み canonical closure 出力は生成して
  いない。candidate JSON の存在又はローカル macOS profile は canonical 合格を意味しない。
- 現行 `leaf_host_e2e_validation.rs` は `ArtifactValidationMode::Full` で
  `KatanaHostE2eState::Passing` と execution ID を要求する。この挙動は公開境界の一次
  設計と矛盾するため、KLE release gate と post-release #336 gate を分ける別修正 task の
  対象である。R4 の source binding では validator を緩和又は変更しない。

## 2. join 規則

### 安全な最初の join

入力 alias の各 `entries` 行について、次を全て満たす時だけ binding 候補を作る。

1. `RequirementSourceAliasLedger` の既存 coverage validation が通る。
2. `source_path` と完全一致する `SourceClosureArtifact.files` がちょうど 1 件ある。
3. その file と完全一致する `BranchCatalogArtifact.branches[*].file` の各 branch を
   1 件ずつ候補にする。
4. 候補は `(requirement_id, source_path, branch_id)` の昇順でソートし、同一 key の
   重複をエラーにする。

候補の根拠は少なくとも以下を構造化して保持する。文字列の説明文だけで判定しない。

- `requirement_id`、`short_reference`、`source_path`
- `SourceClosureFile.sha256`
- `BranchRecord.branch_id`、`span.start_line`、`span.end_line`、
  `source_excerpt_sha256`
- 入力 root の `katana_revision`、`katana_tree_fingerprint`、
  `requirement_source_aliases_fingerprint`、`generator_fingerprint`

これは alias が指す source file と branch の対応であり、alias の source path を
branch span より細かい requirement span と読み替えない。1 file に複数 requirement、
1 requirement に複数 source file、1 file に複数 branch があるため、多対多は正常な
結果である。

### 母集団を縮めない条件

alias ledger は short reference の正規化入力にすぎず、全要件や全 source branch の
母集団ではない。現在の 70 entries だけを列挙して網羅性合格と判定してはならない。
要件文書内の直接 path / directory reference は既存 `SourceRequirementsLedger` と
`source_roots_ledger` の規則を用い、後段で requirement ID 付き入力として取り込む。
走査された全 branch を逆方向にも照合し、対応要件がない branch を unbound record に
残す。未知 branch、macro/外部呼出し、未分類候補を alias 集合の外という理由で捨てない。
T1 はこのうち short-reference 入力だけを扱い、T2 以降の逆方向検査を代替しない。

### 未解決と一意性

- alias source path が走査 file にない、又は同一 path の source file が複数ある場合は、
  binding を捏造せず、requirement ID、path、固定 root fingerprint を含む unresolved
  record にする。
- file に branch がない場合は「branch 未発見」を unresolved とする。要件が満たされた、
  又は branch 非依存であるとは判定しない。
- alias ledger の duplicate、未知 source path、source-universe/root 不一致は既存
  validation error のままにする。後段で選択的に無視しない。
- 同じ `(requirement_id, source_path, branch_id)` が複数回生じた時は deterministic に
  1 件へ丸めず、入力又は join の曖昧性として失敗させる。
- root triple が一致しない時は、既存 `ensure_same_root` と同じ fail-closed 原則を
  適用する。別 SHA の alias、source、branch を結合しない。

### action / handler / effect への接続

この最初の join は action を必須入力にしない。`ActionOrigin` を接続する次段階では、
`action_id`、definition span、construction site、dispatch route の各 source fact が同じ
root にあり、branch file との関係を明示できる場合だけ「source route candidate」を
追加する。`origin_classification == "unresolved"`、対応する
`unresolved_action_origins`、ambiguous definition、macro、dynamic callback、未解決 route
はすべて unresolved のまま残す。

handler と effect はさらに別の段階である。source 上の呼出し又は route の検出は
runtime の最終効果を証明しないため、`LeafStatusRecord.declared_effect_kind`、
`execution_id`、`status = "passing"` を設定しない。action との結合後も、根拠 SHA と
span を持つ未解決 candidate でしかない。

## 3. ownership と三 OS proof

| 境界 | R4 source binding の責務 | この段階でしないこと |
| --- | --- | --- |
| KLE | fixed source、alias、branch、action source fact の deterministic な結合と未解決の報告 | KLE UI が動作する、又は公開 API / opaque transit が接続済みと判定すること |
| KUC | KLE が必要とする generic component / artifact capability の owner を後段で参照できるよう、未解決を保持する | KUC #40 の完了待ち、KUC component の推測、KUC 実装の変更 |
| KatanA | fixed revision の source input と、公開後 #336 で必要になる downstream adoption requirement を保持する | KLE release 前に KatanA #336 の host E2E、buffer/state/file 等の最終効果を要求又は証明すること |
| OS proof | profile IDs と cfg source fact を候補の根拠として保存する | macOS の結果を Windows/Linux に継承すること、三 OS 実行を合格と扱うこと |

KLE release の全 leaf は KLE public input、KUC frame、AccessKit、observable result
又は明示された no-mutation outcome の source-derived 証拠を持たなければならない。
release の profile closure には `macos-latest`、`windows-latest`、`ubuntu-latest` の
個別 profile proof が必要であり、本設計の source-derived binding はその input/display
proof の代替ではない。KatanA 固有 host effect の三 OS 証拠は、KLE 公開後の #336 の
別 gate とする。

## 4. 実装タスクと回帰条件

### T1: alias 取得を read-only に公開する

対象は `source_requirement_alias_ledger.rs`、既存の検証実装
`source_requirement_alias_validation.rs`、その unit test に限定する。
`RequirementSourceAliasLedger` に、checked-in ledger を既存の deserialize と coverage
validation を経て `(requirement_id, short_reference, source_path)` の決定的集合として返す
read-only API を加える。手書きの別台帳、allowlist、status field は加えない。
既存の `validate_checked_in_coverage` は検証済み入力取得へ委譲し、同じ検証を二重実装
しない。未使用 API の lint 抑制は追加しない。

回帰条件は、既存 alias coverage が通ること、順序が入力順に依存しないこと、duplicate /
unknown path が既存 validation と同じく失敗すること、外部 reference を KatanA source
binding に混ぜないことである。

### T2: requirement-file-branch candidate を pure に生成する

新規 `requirement_binding.rs` と `requirement_binding_tests.rs` で、T1 の集合、
`SourceClosureArtifact`、`BranchCatalogArtifact` を結合する。候補型には requirement
identity と source/branch の SHA/span を持たせ、resolved と passing の状態を持たせない。
`leaf_join.rs`、`action_origins/materialize.rs`、artifact schema、publication はこの task
では変更しない。

回帰条件は、同一入力の byte/順序安定性、source SHA 又は branch span/excerpt SHA の変更が
根拠 fingerprint を変更すること、存在しない/重複 source、branch 不在、duplicate key、
root mismatch が fail-closed になることである。fixture は小さな Rust source と parser
facts を使い、手書きで passing を作らない。
alias に現れない file の新規 branch も unbound として残る回帰を必須にする。

#### T2 の実行導線

pure な候補生成を未使用のまま残さず、`source-closure audit-requirement-bindings`
から実行する。この診断は固定 revision の読み取り専用ソースに既存の root resolver /
AST visitor / branch catalog を適用し、要件別候補と逆引きの未対応分岐を JSON で出力する。
canonical materializer の代用ではなく、出力 schema は
`requirement-binding-diagnostic-v1` とする。実入力を実施していないため profile は空とし、
三 OS の実行、branch の active 判定、leaf の passing は生成しない。

`--katana-repo` は固定 SHA の clean checkout のみを受け付け、KatanA には書き込まない。
source universe と alias はチェックイン済み入力を検証して使い、探索範囲から外れた
ファイルや解決できない edge を結果から捨てない。出力先の既存ファイルは上書きしない。
この追加は診断用 CLI と新規診断 module に限定し、canonical artifact schema と
publication、`leaf_join` の合格条件は T4 まで変更しない。

### T3: action source route を別 candidate facet として追加する

#### T2 補完: 要件表の直接参照を同じ診断へ取り込む

短縮 alias だけでは `text.editable` のような完全パス参照行を落とすため、要件表を
requirement ID と source cell の組として読み、既存 `SourceRequirementsLedger` の
正規化処理を再利用する。完全パス、既存の相対 prefix、braced `.rs` 参照、`/**` を
source closure の走査済み file 集合へ結合する。ディレクトリ参照は path component
境界で照合し、未走査ファイルを想像して生成しない。

同じ requirement/file に複数の参照がある場合、参照の集合を provenance に保持して
結合は一意化する。要件表の行そのものを inventory に残し、ソース参照なし、外部
ライブラリ依存、未知の参照、走査 file 不在、未展開ディレクトリは未解決として
報告する。short alias の non-KatanA 分類を source binding へ混ぜない。要件の
同義語や `same fixed source` を前行から推測しない。重複 requirement ID と曖昧な
入力は失敗させる。source cell を requirement の意味や branch の決定条件とは扱わない。

診断には全行 inventory と正規化結果を追加し、正規化した source entry を T2/T3
へ渡す。alias-only 診断結果は比較用に保持する。外部依存の解析、実入力、canonical
leaf 分類、passing はこの補完でも未実施である。要件本文 fingerprint を inventory
へ保持し、順序安定、完全パス/相対/brace/directory、同一参照の重複、未知参照、
ソースなし、入力変更、未対応 branch 残存を実 parser fixture で検査する。

対象は `action_origins` の事実型、route resolver、T2 の candidate である。唯一の
definition、明示的 dispatch route、同一 root、明示的 file/span を満たす時だけ action
candidate facet を作る。handler/effect は含めない。

回帰条件は、alias/macro/ambiguous definition/unresolved route が candidate を passing に
しないこと、unresolved fact が artifact に残ること、route mutation が fingerprint を
変更することである。

診断での初期実装は `ScanState` の構造化済み `ActionDefinition` /
`ActionConstruction` / `DispatchArm` を直接結合する。既存の説明用
`forward_route` 文字列を再解析して確定情報へ変換しない。定義が一意で、直接の
`AppAction` 生成と明示 dispatch arm を持つ場合だけ、各 source SHA と正確な span を
保持した route 候補を生成する。alias、macro、定義の曖昧さ、dispatch 不在、
解決できない dispatch reason は個別の未解決記録に残す。

requirement 候補との結合では、生成 site の file と branch span の包含を確認する。
同一 file でも span 外の生成 site は結合しない。これは構文上の候補であり、条件の
真偽、実行可能性、handler の最終効果を証明しない。結合できない requirement /
action 候補も診断から落とさない。三 OS profile 不在は引き続き不在として扱う。

### T4: leaf join と canonical output の統合

#### 統合前の branch 列挙補完

既存 generator design の Phase 4 が要求する制御構文を、既存 AST visitor と
branch catalog へ追加する。`if` の判定式は token からの文字列推測ではなく
`syn::ExprIf::cond.span()` から切り出し、then / else の body は独立した候補 edge とする。
match の pattern / guard / body、`let else`、短絡 `&&` / `||`、loop / while / for、
return / break / continue / `?` を漏らさず未分類候補に残す。

`proc_macro2::LineColumn` の column は UTF-8 文字数なので、byte offset へ変換してから
source excerpt を採取する。行の範囲外や UTF-8 境界違反は拒否する。日本語や絵文字を
同じ行に置く parsed fixture、複数行の条件、条件内の struct pattern / block を必須の
回帰とする。制御構文の列挙は意味上の到達性・observable leaf 分類ではなく、全候補を
unclassified のまま後続 join に渡す。native profile の active 判定も捏造しない。

T1--T3 が安定してから、`GeneratedLeafJoinArtifact` と validator/publication の契約を
変更する。requirement-bound leaf ID、provenance schema、unresolved record を導入し、
既存の `candidate` validator test を更新する。この task 以前に
`LeafStatusRecord.status` を passing へ昇格しない。

回帰条件は、requirement-bound candidate が未分類 branch/unresolved action を含む限り
closed validation に失敗すること、canonical output が source root / alias fingerprint /
profile fingerprints と一致すること、candidate の存在だけでは KLE public input / KUC
frame / AccessKit / observable result の leaf evidence を満たさないことである。KatanA
host E2E は #336 の公開後 gate として別に検査する。現行 validator の Rc/Full 境界は
この task で変更しない。

#### T4a: 構造化済み AppAction origin の限定投影

`StructuredLeafCandidate.route` は、同一固定root内でdefinition、construction、dispatch、
dispatch variant patternのsource SHAとspanを検証済みである。この四つが揃うcandidateだけは
`action_origin_ids` に `katana:AppAction::<variant>` を投影してよい。これはactionの識別子と
source provenanceの結合だけであり、handler、buffer mutation、KUC component、opaque transit、
Storybook stage、host effectを解決済みにしない。statusは必ず`candidate`のままとする。

projectionはrouteのvariantを再解析せず、既存の構造化routeだけを入力にする。空のvariant、
route factの改竄、candidate provenanceの重複、projectionとの差異は失敗にする。既存の
`unresolved:*` presentation/effect fieldsを実証拠なしに置換せず、release validatorの拒否も
変えない。

#### T4b: 未結合branchの構造分類を保持する

`RequirementBindingResult.unbound_branches`は、現在branch ID、file、span、excerpt hashだけを
保持する。このままでは未結合集合を`if`、`match`、`cfg`などの実際の構文種別や条件で
分類できない。既に認証・materialize済みの`BranchRecord`から`symbol`、`kind`、`condition`、
active profile IDs、inactive cfg predicates、incoming edge IDsをそのまま複写する。

この追加はbranchを要件へ結合せず、到達性・effect・KUC component・Storybook stageを推測しない。
新しい全fieldはfingerprintに含め、source branch factsとの差異、重複、順序不安定を回帰で
拒否する。未結合のままcanonical release validatorが失敗する挙動は維持する。

#### T4c: 未解決scan edge種別を構造化する

`SourceActionUnresolved`の`unresolved_scan_edge`は、edge種別をreason文字列へ埋めている。
これは`call`、`if`、`use`等の未解決集合を機械的に区別できず、文字列再解析を誘発する。
scan stateがすでに保持するedge kindを`source_edge_kind: Option<String>`として記録する。
scan edge以外は`None`、scan edgeだけは空でない元のkindを持つ。sortとfingerprintは新fieldを
含め、kindの改竄、欠落、同一既存入力の順序差を拒否する。

この分類はedgeの実行可能性・handler/effect・KUC/Storybook/host evidenceを推測せず、
unresolved statusを減らさない。reasonは人間向け診断として維持し、分類の一次入力には使わない。

#### T4d: 未解決scan edge詳細を構造化する

`ClosureEdge.to_symbol`はscan時のraw detailであり、call target、importの未解決理由、branch label等を
保持する。`unresolved_scan_edge`の表示用reasonは種別だけで、このdetailを落としているため、次段の
source解析はJSONの表示文又は原本を再走査しなければならない。

`SourceActionUnresolved.source_edge_detail: Option<String>`に、scan edgeだけの`to_symbol`をそのまま
保持する。非scan unresolvedは`None`、raw detailがないscan edgeも`None`とする。Someの空又は空白値、
空又は空白のkindはfail-closedで拒否する。sortとfingerprintに両fieldを含め、detailの改竄、欠落、
順序差を回帰検査する。既存reasonは変更しない。

このpatchはdetailの意味をtarget/resolution/到達可能性へ昇格しない。action route、handler/effect、
KUC/Storybook/host evidence、status、release validatorは変更しない。

#### T4e: lexical resolution結果を構造化する

未解決callのdetailには`external boundaries remain unresolved`等の表示文が含まれるが、これは
`LexicalResolution`の`Unresolved`、`Ambiguous`、`AmbiguousAlias`を文字列化した結果である。
表示文から原因を再分類するのではなく、edge生成時にclosedなresolution statusを保存する。

`ClosureEdge`は任意のresolution statusを持ち、`record_resolution`だけが`local`、
`unresolved`、`ambiguous`、`ambiguous_alias`を設定する。branch、module、method call、直接
`record_unresolved`は`None`のままにする。`SourceActionUnresolved`はscan edgeのstatusだけを
転記する。statusはto_pathの存在、外部dependency、実行可能性、handler/effect、KUC/Storybook/
host evidenceを意味しない。既存reason/detailを変更せず、未知・空statusを拒否し、sortと
fingerprintへ含める。

同一file free functionを名前だけでlocal解決することは行わない。引数又はlocal closureによる
shadowingを現scope modelが証明できないためであり、正確なbinding scope解析は後続の独立taskとする。

#### T4f: 既走査targetへのback edgeを未解決にしない

`ScanState::add_edge`はlocal targetへのedgeを常にpendingへ入れる。しかしtarget fileが既に
`record_file`済みの後にback edgeが追加されると、pendingを消費する再走査は行われず、
`finalize_unscanned_targets`がfalse unresolvedとして出力する。target fileのpathとSHAが同じ
scan stateに存在する場合、materializationは既に事実として完了している。

local target pathが`files`に存在するedgeはpendingへ追加しない。未走査targetだけをpendingに
入れ、existing pathでもsource hash、edge kind、symbol、effect、実行到達性を推測しない。
module/import/callの順序を逆転した同一fixtureでもJSON bytesを安定化し、既走査targetへの
back edgeがunresolvedに残らず、未走査又はmaterialization失敗targetが残ることを回帰検査する。

候補pathは`canonicalize`してからstateへ入れる。macOSのcase-insensitive filesystemでは
`crate::Icon`が`Icon/mod.rs`として存在判定されても、inventoryの実体pathは`icon/mod.rs`である。
canonical pathを使うことで既走査判定とinventory照合を同じpath identityにそろえる。これは
moduleの意味、symbol binding、外部boundary、実行到達性を推測する処理ではない。

## 5. 最初に着手する限定パッチ

最初の patch は T1 のみとする。

- `tools/katana-parity-check/src/source_closure/source_requirement_alias_ledger.rs` に、
  checked-in alias ledger を coverage validation 後に決定的な source binding input として
  返す `RequirementSourceAliasLedger` の read-only API を追加する。
- 同 module の既存 test か、責務が独立する場合だけ新規
  `source_requirement_alias_ledger_tests.rs` に、順序安定、duplicate/unknown path の拒否、
  `non_katana_references` の除外を追加する。
- `leaf_join.rs`、`branch_catalog.rs`、`action_origins/materialize.rs`、JSON alias ledger、
  canonical artifact、validator、materializer/publication は変更しない。

この patch は source alias の検証済み入力化だけを行い、branch との結合、action/handler/
effect、KUC ownership、三 OS proof、KLE の全 leaf evidence、公開後 #336 の KatanA host
effect、leaf status の変更を後続 task に残す。したがって、未実行候補を pass に昇格する
経路も、手書き台帳だけを相互確認する経路も増やさない。
