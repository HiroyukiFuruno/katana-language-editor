## ADDED Requirements

### Requirement: 外部から設定を受け取れる EditorSettings IF を提供しなければならない

`katana-language-editor` neutral crate は `EditorSettings` 構造体と、それを `EditorConfig::settings` に **non-nullable** で含める契約を提供しなければならない（MUST）。最低限 `autosave: AutosavePolicy` / `shortcuts: ShortcutMap` / `word_wrap: bool` / `tab_size: u8` / `line_numbers: bool` / `font_scale: f32` を含む。

#### Scenario: host が EditorSettings を渡して editor を構築する

- **WHEN** host が `EditorSettings { autosave: AutosavePolicy::Off, shortcuts: kdv_default_shortcuts(), .. }` を `EditorConfig::settings` に渡す
- **THEN** editor は対応する挙動で初期化される
- **THEN** `Option<EditorSettings>` ではなく必須引数として扱われる

#### Scenario: runtime での settings 反映

- **WHEN** host が `editor.apply_settings(new_settings)` を呼ぶ
- **THEN** editor は内部状態（autosave timer、shortcut bindings、word wrap モード等）を新設定で再構成する
- **THEN** 反映できない設定変更は `Result<_, EditorError::Unsupported>` を返す

### Requirement: AutosavePolicy は enabled と interval を分離して扱わなければならない

`AutosavePolicy { enabled: bool, interval: Option<Duration> }` 形を取り、`enabled = false` の時は autosave を実行せず、`enabled = true` の時は `interval` が指定されていればそれを尊重し、未指定なら editor 既定値で動作する（MUST）。

#### Scenario: 自動保存 OFF

- **WHEN** host が `AutosavePolicy { enabled: false, interval: None }` を渡す
- **THEN** editor は内部 timer を停止する
- **THEN** save は host からの明示的トリガでのみ行う

#### Scenario: 自動保存 ON + 30 秒間隔

- **WHEN** host が `AutosavePolicy { enabled: true, interval: Some(Duration::from_secs(30)) }` を渡す
- **THEN** editor は最後の編集から 30 秒経過時に save イベントを発火する
- **THEN** save 完了通知は `EditorEvent::AutosavedAt(timestamp)` 等で host が受け取る

### Requirement: ショートカット上書きを ShortcutMap で受け取らなければならない

`ShortcutMap` は semantic action key（`SemanticAction::Save` / `Find` / `Replace` / `Undo` / `Redo` / `ToggleComment` / `ToggleReadOnly` / `Focus` 等）と key binding（modifier + key code）の対応を保持する（MUST）。host が `EditorConfig::settings.shortcuts` で上書きできる（MUST）。

#### Scenario: host が Cmd+S を上書きする

- **WHEN** host が `ShortcutMap` に `SemanticAction::Save -> Cmd+Shift+S` を渡す
- **THEN** editor は Cmd+Shift+S で save action を発火する
- **THEN** デフォルトの Cmd+S binding は上書きされる

#### Scenario: 競合検知

- **WHEN** ShortcutMap に同じ key binding が複数 action に割り当てられる
- **THEN** editor は `Err(EditorError::ConflictingShortcut { binding, actions })` を返す
- **THEN** host が解決するまで editor は構築されない

### Requirement: settings の default は KDV 側 preset とし、KLE crate は default を持たない

`katana-language-editor` crate は `EditorSettings::default()` 相当の preset を持ってはならない（MUST NOT）。host (KDV) は `kdv-presets::settings::default()` を提供し、host が必須引数として渡す。

#### Scenario: KDV preset を渡す

- **WHEN** host が `kdv-presets::settings::default()` を取得し editor に渡す
- **THEN** editor は KDV 由来の設定で初期化される
- **THEN** KLE crate 内で `EditorSettings::default()` を grep しても見つからない
