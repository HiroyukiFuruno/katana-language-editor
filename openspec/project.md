# katana-language-editor OpenSpec

## Project

`katana-language-editor`（KLE）は、language editor の neutral interface と実装を提供するlibrary。KatanA はこれを git dependency としてconsumeする。

## Design Principles

- `katana-language-editor` crate（neutral interface）は `egui` に依存しない。
- `katana-language-editor-floem` crate が正式実装を持つ。
- `LanguageEditor` trait の surface に egui 型を含めない。
- KLEはMarkdown viewer、export、editor-viewer同期制御を持たない。
- 同期制御はKatanAが持ち、KatanAがeditorまたはviewerへscroll、selection、highlightなどの命令を送る。

## Versioning

- `v0.1.x`: KatanAで分離するeditor実装の移管。KMM metadata同期の接続方針を確立。
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

### Floem dependency の取り扱い（必須ルール）

`floem` は **必ず git dependency** として取り込む。crates.io の `floem = "0.2.0"`（2024-11 頃公開）以降は main 側の進行と乖離しており、theme / styling / editor view API が満たせない。

```toml
# Cargo.toml (workspace.dependencies)
floem = { git = "https://github.com/lapce/floem", rev = "<pinned-sha>" }
```

- `rev`（commit sha）でピン留めする。ブランチ追従はしない。
- `katana-language-editor-floem` crate は v0.1.0 段階では skeleton（コンパイル可能）にとどめ、本実装は v0.2.x で行う。
- `cargo tree -p katana-language-editor-floem` で `floem` の source が `git+https://github.com/lapce/floem` を指していることを CI で確認する。
- 更新時は rev を bump し、PR 内で behavior 差分・依存（vello / wgpu / cosmic-text）の波及を明示する。

### Neutral interface の DI 必須ルール（v0.1.0 以降）

`katana-language-editor` neutral crate は host (KDV) からの DI を **non-nullable** で受け取り、KLE 内に default preset を持たない。`Option<Theme>` / `Option<Strings>` / `Option<EditorSettings>` などは禁止する。

| DI | 何を渡すか | preset 提供元 |
|----|-----------|---------------|
| `Theme` | 色トークン（semantic alias 含む） | `kdv-presets::theme` |
| `Strings` + `Locale` | UI 文字列 + LTR/RTL direction | `kdv-presets::strings`（en 必須） |
| `Typography` / `Spacing` | font / 余白 / radius トークン | `kdv-presets::typography` |
| `EditorSettings` | autosave / shortcuts / wrap / tab | `kdv-presets::settings` |
| `SyntaxHighlighter` | 言語別ハイライタ | KatanA など host が実装 |
| `ClipboardBackend` | clipboard 操作の抽象化 | host |

色のハードコードは `kle-linter` の `prohibited-color-literal` で機械的に禁止する。文字列リテラルも将来同様のルールで補強する方針。

`katana-language-editor` は以下の host 制御 API を public に公開する：

- `EditorScrollControl` / `EditorWriteAccess` / `EditorViewControl`（read-only / focus）
- `EditorHistoryControl` / `EditorSelectionControl` / `EditorSearchControl` / `EditorClipboardControl`
- `EditorDiagnosticsSink` / `EditorDecorationsSink` / `EditorAccessibility`

egui MVP で未対応の API は `EditorError::Unsupported` を返し、Floem 実装で完全対応する。

---

## KMM構想での扱い

KMM構想ではP3として、P0 `katana-ast-lint`、P1 `katana-markdown-model`、P2 `katana-ui-widget` の境界を受けて、保存時の外部メタデータ（metadata）同期を実装する。

- KMM文書モデルやmetadata schemaを再実装しない。
- 保存時のmetadata同期は、KMMの位置解決APIを呼ぶ。
- 自動復元できないtargetは削除せず、unresolvedとして保持する。
- KLEはviewerやexportを知らない。
- KLEはeditor-viewer同期のcoordinatorにならない。
- KatanAからeditor操作命令を受けるsurfaceは持つが、viewerへ命令しない。
- neutral interfaceへeguiやFloemの実装型を漏らさない。
- 共通AST lintを品質ゲートにする。
