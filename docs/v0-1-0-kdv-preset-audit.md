# KLE v0.1.0 KDV Preset Audit

## Scope

This note records the KDV-side preset audit for OpenSpec task 13.6. KDV was inspected read-only; no sibling repository edits were made.

This audit does not complete task 13.6 by itself. It records why the task remains pending until the KLE v0.1.0 release exists and the KDV follow-up is tracked by a single actionable issue or implemented in KDV.

## Findings

- Theme preset exists in KDV as `KdvThemeSnapshot::katana_light()` and `KdvThemeSnapshot::katana_dark()`.
  Evidence: `/Users/hiroyuki_furuno/works/private/katana-document-viewer/crates/katana-document-viewer/src/theme.rs:49` and `/Users/hiroyuki_furuno/works/private/katana-document-viewer/crates/katana-document-viewer/src/theme_presets.rs:154`.
- KDV Storybook bridges KDV theme snapshots into KUC `ThemeSnapshot` through `KucThemeBridge::from_kdv` and uses that bridge in the live preview scene path.
  Evidence: `/Users/hiroyuki_furuno/works/private/katana-document-viewer/tools/kdv-storybook/src/preview_theme_bridge.rs:7` and `/Users/hiroyuki_furuno/works/private/katana-document-viewer/tools/kdv-storybook/src/preview_scene_from_output.rs:41`.
- Typography preset/state exists as `ViewerTypographyConfig` with a default preview font size and KDV applies preview-font-size updates through typed settings state.
  Evidence: `/Users/hiroyuki_furuno/works/private/katana-document-viewer/crates/katana-document-viewer/src/viewer/settings_update.rs:41` and `/Users/hiroyuki_furuno/works/private/katana-document-viewer/crates/katana-document-viewer/src/viewer/settings_update_apply.rs:19`.
- Settings state exists as `ViewerSettingsState`, `ViewerSettingsField`, and `ViewerSettingsUpdate`, including theme, mode, preview font size, and interaction toggles.
  Evidence: `/Users/hiroyuki_furuno/works/private/katana-document-viewer/crates/katana-document-viewer/src/viewer/settings_update.rs:6`.
- KDV Storybook has KUC `SettingsList` action and hit-test contracts around the sidebar settings panel.
  Evidence: `/Users/hiroyuki_furuno/works/private/katana-document-viewer/tools/kdv-storybook/src/sidebar.rs:36`, `/Users/hiroyuki_furuno/works/private/katana-document-viewer/tools/kdv-storybook/src/sidebar_hit.rs:103`, and `/Users/hiroyuki_furuno/works/private/katana-document-viewer/tools/kdv-storybook/src/settings_action.rs:30`.
- KLE-specific editor `Strings` / `Locale` / `EditorSettings` preset for `katana-language-editor::EditorConfig` was not found in KDV during this audit.
  Evidence: read-only searches on 2026-07-06 found no hits for `katana-language-editor`, `katana_language_editor`, `EditorConfig`, `Strings`, `Locale`, or `EditorSettings` under KDV `crates/` and `tools/`, and no matching dependency in KDV crate manifests.
- Live GitHub issue search for `HiroyukiFuruno/katana-document-viewer` on 2026-07-06 later returned duplicate open issues with the exact title `KDV preset follow-up for katana-language-editor v0.1.0`: #21, #22, and #23.
  Evidence: `gh issue list --repo HiroyukiFuruno/katana-document-viewer --state open --search 'KDV preset follow-up for katana-language-editor v0.1.0 in:title' --json number,title,url --limit 20`.
- Those duplicate issues currently have empty bodies and therefore are not strong completion evidence for task 13.6.
  Evidence: `gh issue view 21`, `gh issue view 22`, and `gh issue view 23` returned `body: ""` on 2026-07-06.
- KLE now provides `just VERSION=v0.1.0 kdv-preset-followup-audit` as a release-independent read-only gate for task 13.6. On 2026-07-06 it fails because the KDV follow-up issue is duplicated across #21, #22, and #23.

## Post-release action

After the KLE v0.1.0 release is published, KDV must have a single actionable follow-up issue or an implementation for adding KLE editor presets:

- `strings::en()` returning `katana_language_editor::Strings`
- `locale::en_ltr()` returning `katana_language_editor::Locale`
- `settings::default_editor()` returning `katana_language_editor::EditorSettings`
- Optional adapter helpers to map KDV theme/typography/spacing into KLE `EditorConfig`

This keeps KLE free of built-in default presets while giving KDV the host-owned preset surface required by the v0.1.0 contract.
