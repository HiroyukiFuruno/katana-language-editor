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
