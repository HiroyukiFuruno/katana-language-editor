## ADDED Requirements

### Requirement: 全UI文字列はhost presentationとKUCの文字列契約を通さなければならない

全UI文字列（メニュー、ボタン、ツールチップ、エラー、accessibility label、context menu、find bar等）はhost presentationとKUCのgeneric文字列契約を通さなければならない（MUST）。KLEはopaque projectionを中継し、自然言語literal、キー解決、default文字列を持ってはならない（MUST NOT）。

#### Scenario: hostが翻訳済みpresentationを渡す

- **WHEN** hostが公開KUC契約へ翻訳済みpresentationを供給する
- **THEN** KUC rootは全UI文字列とAccessKit labelを同じpresentationから描画・出力する
- **THEN** `egui::Label::new("Save")` のような直接リテラルが impl crate に存在しない

#### Scenario: KLE configに文字列を再導入しない

- **WHEN** consumerがEditorConfig/Inputのstrings又はlocale fieldに依存する
- **THEN** 各fieldの独立compile-failとAST検査で拒否する
- **THEN** Option/defaultや別名storeで互換性を装わない

### Requirement: locale は LTR/RTL direction を含み、editor は direction に応じて配置を切り替えなければならない

host presentationは言語とLTR/RTL directionを含み、KUCが行先頭・行末・gutter配置と入力を処理しなければならない（MUST）。KLEはLocale DTOの保持・方向計算を行わない。旧Locale/TextDirection公開型の除去も別工程として残る。

#### Scenario: RTL locale で配置が反転する

- **WHEN** hostが言語ar・direction RTLのpresentationを供給する
- **THEN** gutter は行の右側に配置される
- **THEN** caret 移動キーは方向に応じて意味付けされ、KUC frame record と AccessKit に同じ direction が反映される

### Requirement: KLEにdefault文字列やKDV固有preset依存を持たない

KLEはdefault文字列を持ってはならない（MUST NOT）。英語を含む必要な翻訳はhost/KUC契約で供給する。KDV固有en presetの提供・採用をKLE公開の前提にしない。

#### Scenario: KDV固有presetなしで英語表示する

- **WHEN** hostが公開KUC契約へ英語presentationを供給する
- **THEN** KDV固有crateなしでKUC rootが英語表示する
- **THEN** KLE crate 内で `Strings::default()` / 英語フォールバックを grep しても見つからない

#### Scenario: 未訳キー検出

- **WHEN** hostが独自言語を供給するが必須キーが欠落している
- **THEN** KUC/hostの型で閉じた必須フィールド契約がコンパイル時に検知する
- **THEN** 実行時の untranslated key を許容しない（key の集合は型レベルで閉じている）

### Requirement: 文字列キー集合はeditor機能と対応し拡張方針を明示しなければならない

KUC/hostの文字列契約はUIのsemantic単位ごとに必須キーを持ち、公開文字列構造体の`#[non_exhaustive]`による拡張と将来の互換性方針を明示しなければならない（MUST）。KLEに同じStrings DTOを複製しない。旧Strings公開型の除去と翻訳網羅性の証明は未完了として別々に検査する。

#### Scenario: 新規 UI 機能追加時の互換性

- **WHEN** find bar等の新機能に必要な文字列キーを追加する
- **THEN** KUC/hostが型で閉じた契約と明示した互換性方針に従って更新する
- **THEN** 既存 host のコンパイルが壊れない（または意図的に壊して上書きを促す）
