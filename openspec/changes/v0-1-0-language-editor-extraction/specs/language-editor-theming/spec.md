## ADDED Requirements

### Requirement: 全色表現はhost presentationとKUC theme契約を通さなければならない

editor内の全色（前景・背景・選択・キャレット・ガター・行ハイライト・diagnostics underline・decoration overlay等）はhostが供給するpresentationとKUCのgeneric theme契約を通さなければならない（MUST）。KLEはopaque projectionを中継し、色の解決、palette変換、hardcode、fallbackを持ってはならない（MUST NOT）。

#### Scenario: hostがthemeを供給して描画する

- **WHEN** hostが公開KUC契約へthemeを含むpresentationを供給する
- **THEN** KUC rootの全描画はそのsemantic colorを使い、同一frameの証跡で確認する
- **THEN** `egui::Color32::WHITE` や `Color::rgb(...)` 直書きが impl crate に存在しない

#### Scenario: KLE configにthemeを再導入しない

- **WHEN** consumerがEditorConfig/Inputのtheme fieldに依存する
- **THEN** 独立compile-failとAST検査で拒否し、Optionやdefaultによる互換shimを作らない
- **THEN** 必要な色の完全性はhost/KUC契約で検査し、field除去だけで合格としない

### Requirement: KLEにdefault themeやKDV固有preset依存を持たない

KLEはdefaultテーマ（dark/light preset）を持ってはならない（MUST NOT）。preset選択はhost、generic themeの処理はKUCが所有する。KDV固有presetの提供・採用をKLE公開の前提としてはならない。

#### Scenario: KDV固有presetなしで利用する

- **WHEN** hostが公開KUC契約を使ってthemeを供給する
- **THEN** KDV固有crateへの依存なしに指定色をKUC rootが描画する
- **THEN** KLE crate 内で `Theme::default()` 相当の関数を grep しても見つからない

#### Scenario: ダーク/ライト切替は host が制御する

- **WHEN** host が UI 操作で dark/light を切り替える
- **THEN** hostは更新presentationをKUCへ供給し、KLEはopaque leaseの同期だけを行う
- **THEN** KLEはtheme切替の意思決定やapply_theme storeを持たない

### Requirement: 色トークンは意味付き alias を含まなければならない

色契約はraw paletteだけでなく、本文・補助文字・選択背景・caret・gutter・現在行・diagnostic severity・decoration等のsemantic roleを網羅しなければならない（MUST）。所有者はKUC/hostであり、KLEに同じtoken DTOを複製しない。旧型の除去だけで機能検証を完了としてはならない。

#### Scenario: 旧KLE theme型の再導入を拒否する

- **WHEN** consumerがRgba/ColorTokens/ColorTokensInput/Theme/EditorThemeをKLEのroot又はtypesからimportする
- **THEN** 各型・各パスの10独立compile-failで拒否する
- **AND** neutral内で同名struct/enum/type aliasを定義してもAST検査で拒否する
- **AND** literal color検査は削除したKLE Themeの再導入を修正案として勧めない
- **AND** この検査をsemantic color/切替/表示の動作証明にしない

#### Scenario: 実装は alias を使って描画する

- **WHEN** editor 実装が diagnostics underline を描画する
- **THEN** KUCが対応するseverityのsemantic roleを使って描画する
- **THEN** raw palette index への直接アクセスは禁止される
