## ADDED Requirements

### Requirement: 設定更新の責務はKUCとhostに分離しなければならない

KLE MUST NOT 公開traitに `apply_settings` 又は同等の設定更新storeを提供する。
generic presentation/input policy はKUC、autosave/file lifecycleとアプリ固有の
shortcut arbitrationはhostが所有する。KLEは既存opaque root接続で渡される
revisioned projectionと一度だけ消費するeventを扱い、設定値を解釈・保持しない。

#### Scenario: neutral trait実装が旧設定メソッドを追加する

- **WHEN** consumer が `LanguageEditor` のtrait実装に `apply_settings` を定義する
- **THEN** コンパイラはtraitに存在しないメソッドとして拒否する
- **THEN** 設定を無視して成功を返す互換shimを追加しない

#### Scenario: hostが設定を変更する

- **WHEN** hostがgeneric UI policy又はhost document policyを変更する
- **THEN** 各所有者の既存routeで変更を処理し、KLEにautosave timerやshortcut mapを作らない
- **THEN** source-derived設定leafとKUC実入力の証跡が不足する場合は未検証として公開を拒否する

### Requirement: 設定DTOの除去と設定動作の互換性を別々に証明しなければならない

KLE MUST NOT `EditorSettings`、`AutosavePolicy`、`ShortcutMap`、`ShortcutBinding`、
`KeyBinding`、`KeyModifier`、`SemanticAction`、typography/spacingの
所有を公開APIに残す。autosaveの有効/無効と間隔、shortcutの上書き/競合、word-wrap、
tab、line-number、font policyの動作要件は削除せず、固定KatanA sourceの各分岐と
所有者の実行証跡へ結び付ける。旧DTOの既定値や仮の30秒fixtureをKatanAの実仕様の
代わりにしてはならない。config fieldや型が消えたことだけでは設定動作の完成ではない。

#### Scenario: 旧設定型の公開を拒否する

- **WHEN** consumerが旧設定7型をroot又はtypes moduleからimportする
- **THEN** 各型・各公開パスの独立compile-fail検査で拒否する
- **AND** neutral内で同名struct/enum/type aliasを再定義した場合もASTゲートで拒否する
- **AND** apply_settings拒否検査は削除した型のimport失敗ではなくtrait非所属を検査する

#### Scenario: sourceで定義された設定動作を検証する

- **WHEN** source-derived leafにautosave又はshortcut又はgeneric表示設定の分岐がある
- **THEN** host policyとKUC policyの所有者を明示し、設定前後の効果とno-opを検査する
- **THEN** KLEの設定型やメソッドが消えたことだけを動作互換の証明にしない

#### Scenario: KDV固有presetなしでKLEを検証する

- **WHEN** hostが既存KUC公開契約へ必要なprojection/policyを供給する
- **THEN** KLEはKDV固有presetを要求せず、一度だけのopaque relayとして動作する
- **THEN** KLEにdefault timer、shortcut arbitration、font fallbackを作らない
