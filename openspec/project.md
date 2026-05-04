# katana-language-editor OpenSpec

## Project

`katana-language-editor`（kle）は、language editor の neutral interface と egui MVP 実装を提供する library。KatanA はこれを git dependency として consume する。

## Design Principles

- `katana-language-editor` crate（neutral interface）は `egui` に依存しない。
- `katana-language-editor-egui` crate が egui MVP 実装を持つ。将来の独自 input surface（`x-x-x-native-input-surface`）への差し替えはこの crate のみ変更すれば良い。
- `LanguageEditor` trait の surface に egui 型を含めない。

## Versioning

- `v0.1.x`: KatanA v0.27.0 で分離する editor 実装の移管。egui TextEdit MVP 確立。
- `v0.2.x`: 独自 input surface への差し替え（x-x-x-native-input-surface 対応）

## Consumers

- [KatanA](https://github.com/HiroyukiFuruno/KatanA) — git tag pinned（v0.27.0 で取り込み）

---

## UI フレームワーク移行方針（egui → Floem）

このセクションはエコシステム全体で共通の方針。詳細は [KatanA openspec/project.md](https://github.com/HiroyukiFuruno/KatanA/blob/master/openspec/project.md) を正とする。

### 技術選定（確定）

| 層 | 採用 |
|----|------|
| UI フレームワーク | **Floem**（Rust 純正・クロスプラットフォーム） |
| 文字描画 | **cosmic-text**（IME 完全対応・カラー絵文字 SBIX/CBTF） |
| 2D レンダリング | **vello + wgpu**（compute-shader・Metal/DX12/Vulkan） |
| レイアウト | **taffy**（flexbox + CSS Grid） |
| アーキテクチャ参考 | **GPUI / Zed**（設計の教材として活用） |

React / TypeScript / WebView は使用しない。Rust 純正のみ。

### egui から脱却する理由（要約）

- カラー絵文字：epaint が SBIX/CBTF 非対応 → cosmic-text で解決
- IME 不完全：egui TextEdit の composition が壊れる → cosmic-text + winit で解決
- レイアウト拡張不可：vendor パッチなしに行間・マージンを変えられない → vello Scene への直接描画で解決
- immediate mode の再描画コスト → vello の retained 描画で解決

### この repo の責務

各 `-egui` impl crate を `-floem` impl crate に差し替える。neutral interface crate は変えない。
KatanA の `Cargo.toml` の impl crate 行を変えるだけで移行が完了する。

### katana-language-editor の移行

```
katana-language-editor          neutral interface（変わらない）
katana-language-editor-egui     MVP 実装（Phase 1 で置き換え対象）
katana-language-editor-floem    Floem + cosmic-text 実装（Phase 1 で新規作成）
```

Phase 1 が最優先。editor 入力の IME・絵文字問題はユーザーが最初に触れる痛みであるため。
