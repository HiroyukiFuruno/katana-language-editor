# katana-language-editor — UI 分離計画 抜粋

作成日: 2026-05-17  
canonical: [`katana/docs/architecture/ui-separation/detailed-design-and-tasks.md`](../../katana/docs/architecture/ui-separation/detailed-design-and-tasks.md)

## このファイルの位置付け

本ファイルは KatanA ecosystem の **UI 分離構想 master** から `katana-language-editor` (KLE) 担当部分を抜粋したもの。task ID は master と同一。**master が単一情報源**であり、本ファイル単独で task を追加・修正してはならない。

## Repository の役割

`katana-language-editor` (KLE) は **language-neutral な編集ドメイン**を所有する。

- markdown / Rust / TypeScript など特定言語に縛られない。
- `katana-markdown-model` (KMM) に hard dependency を **持たない**。
- syntax / jump / hover / completion / formatter / diagnostics / source anchor を **port (trait)** として宣言する。default 言語実装は持たない。
- 利用側 (KatanA など) が言語固有の port implementation を inject する。markdown 用 implementation は `katana-language-editor-md` 別 crate として配置する。
- UI framework を知らない。`katana-ui-core` の Component model に依存しない。

詳細: master [`5.3 katana-language-editor` 詳細設計](../../katana/docs/architecture/ui-separation/detailed-design-and-tasks.md#53-katana-language-editor-詳細設計)

## 担当 Phase

- **Phase 3**: KLE domain 強化 + ports + md adapter (本 repo のメイン作業)
- **P6-A-005**: KLE は KMM に hard dep を持たない方針確定 (ADR 化)
- **横断**: P0 (governance)

## Phase 3 設計原則

- Editor owns editing domain.
- Editor is **language-neutral**。
- Editor does not implement syntax / jump / hover / completion / formatter. それらは **port (trait)** として宣言し、利用側が inject する。
- Editor は default language implementation を持たない。
- Editor does not depend on KMM など特定言語 model。
- Editor exposes source anchors and events via neutral DTO。

## Task list (master 抜粋)

### P3-A. Buffer model

- [ ] P3-A-001: `BufferId` を定義する。
- [ ] P3-A-002: `BufferRevision` を定義する。
- [ ] P3-A-003: `BufferSnapshot` を定義する。
- [ ] P3-A-004: `TextRange` を定義する。
- [ ] P3-A-005: `TextEdit` を定義する。
- [ ] P3-A-006: `EditBatch` を定義する。
- [ ] P3-A-007: `TextContent` を BufferSnapshot に接続する。
- [ ] P3-A-008: buffer serialization test を作る。
- [ ] P3-A-009: edit application test を作る。

### P3-B. Cursor / selection

- [ ] P3-B-001: `CursorAffinity` を定義する。
- [ ] P3-B-002: `CursorRange` を定義する。
- [ ] P3-B-003: `SelectionMode` を定義する。
- [ ] P3-B-004: `MultiSelection` を定義する。
- [ ] P3-B-005: cursor movement command を定義する。
- [ ] P3-B-006: selection changed event を拡張する。
- [ ] P3-B-007: cursor restore model を作る。
- [ ] P3-B-008: cursor fixture test を作る。

### P3-C. Commands and events

- [ ] P3-C-001: `EditorCommand` enum を作る。
- [ ] P3-C-002: `InsertText` command を作る。
- [ ] P3-C-003: `DeleteRange` command を作る。
- [ ] P3-C-004: `ReplaceRange` command を作る。
- [ ] P3-C-005: `MoveCursor` command を作る。
- [ ] P3-C-006: `ApplyFormat` command を作る。
- [ ] P3-C-007: `OpenDocument` command を作る。
- [ ] P3-C-008: `EditorEvent` に revision を含める。
- [ ] P3-C-009: event ordering test を作る。

### P3-D. Diagnostics

- [ ] P3-D-001: `Diagnostic` を定義する。
- [ ] P3-D-002: `DiagnosticSeverity` を定義する。
- [ ] P3-D-003: `DiagnosticSource` を定義する。
- [ ] P3-D-004: `DiagnosticRange` を定義する。
- [ ] P3-D-005: markdown linter diagnostics を受ける DTO を作る。
- [ ] P3-D-006: AST lint diagnostics を受ける DTO を作る。
- [ ] P3-D-007: diagnostics changed event を作る。
- [ ] P3-D-008: diagnostics snapshot test を作る。

### P3-E. Source anchor (language-neutral)

KLE core 側は markdown / KMM を含む特定言語に依存しない。source anchor は opaque DTO + adapter trait に閉じる。

- [ ] P3-E-001: `SourceAnchor` (opaque ID + position 情報を持つ言語非依存 DTO) を定義する。
- [ ] P3-E-002: `LineColumnRange` DTO を定義する。
- [ ] P3-E-003: text fingerprint DTO を定義する。
- [ ] P3-E-004: `SourceAnchorAdapter` trait を定義する (snapshot + position から SourceAnchor を返す)。
- [ ] P3-E-005: source anchor changed event を定義する (anchor は opaque)。
- [ ] P3-E-006: `SourceAnchorAdapter` 用の `NoopSourceAnchorAdapter` を明示提供する。
- [ ] P3-E-007: KLE crate が `katana-markdown-model` を含む markdown 系 crate を import していないことを script で検査する。
- [ ] P3-E-008: source anchor fixture test を作る (markdown 概念を含まないこと)。

### P3-G. External ports (language-neutral trait group)

KLE は syntax / jump / hover / completion / formatter / diagnostics / source anchor の各機能を **port (trait)** として宣言する。default 言語実装は提供しない。利用側 (KatanA / katana-language-editor-md など) が必ず inject する。

- [ ] P3-G-001: `SyntaxHighlightProvider` trait と `SyntaxToken` DTO (kind / range / scope) を定義する。
- [ ] P3-G-002: `JumpProvider` trait と `JumpTarget` DTO (uri / range) を定義する。
- [ ] P3-G-003: `HoverProvider` trait と `HoverContent` DTO (framework-neutral rich text) を定義する。
- [ ] P3-G-004: `CompletionProvider` trait と `CompletionItem` DTO を定義する。
- [ ] P3-G-005: `FormatterProvider` trait を定義する (snapshot + range から TextEdit のリストを返す)。
- [ ] P3-G-006: `DiagnosticsSource` trait を定義する (BufferId に対する diagnostics ストリームを購読する)。
- [ ] P3-G-007: editor builder API を作り、上記 port を必須引数として受け取る。port なしでは editor を construct できないようにする。
- [ ] P3-G-008: `NoopSyntaxHighlightProvider` / `NoopJumpProvider` 等 default を明示提供する (`Default::default()` には依存させず明示渡しを強制)。
- [ ] P3-G-009: port trait が KLE crate 内に閉じ、markdown / KMM 概念を含まないことを script で検査する。
- [ ] P3-G-010: port fixture test を作る (Noop / Mock implementation で editor が動作することを検証)。
- [ ] P3-G-011: port wiring 方針を README に明文化する (`docs/usage/ports.md`)。

### P3-H. Markdown port adapter (`katana-language-editor-md`)

KLE 自体に markdown 知識を入れない代わりに、markdown 用の port implementation を別 crate として提供する。配置は新規独立 crate または KatanA repo 内 crate のどちらでもよい (ADR で決める)。

- [ ] P3-H-001: `katana-language-editor-md` crate を新設する。配置場所 (独立 repo / KatanA repo 内) を ADR `docs/adr/kle-md-adapter-placement.md` に記録する。
- [ ] P3-H-002: KMM document model を入力とする `SyntaxHighlightProvider` implementation を提供する。
- [ ] P3-H-003: KMM source span を返す `SourceAnchorAdapter` implementation を提供する。
- [ ] P3-H-004: markdown 用 `JumpProvider` implementation (heading / link target) を提供する。
- [ ] P3-H-005: markdown 用 `CompletionProvider` stub (heading / link suggestions) を提供する。
- [ ] P3-H-006: markdown 用 `FormatterProvider` を提供する (KMM 既存 formatter があれば委譲)。
- [ ] P3-H-007: `katana-language-editor-md` から KLE への dependency 方向が正しい (KLE が md adapter を知らない) ことを script で検査する。
- [ ] P3-H-008: KatanA から `katana-language-editor-md` を inject する flow を `docs/usage/katana-markdown-editor.md` に記載する。
- [ ] P3-H-009: md adapter の fixture test (KMM input -> port output) を作る。

### P3-F. Adapter containment

- [ ] P3-F-001: `katana-language-editor-egui` を compatibility adapter として README に記録する。
- [ ] P3-F-002: future `katana-language-editor-floem` の crate boundary を設計する。
- [ ] P3-F-003: core crate から egui import がないことを script で検査する。
- [ ] P3-F-004: adapter crate の only dependency policy を作る。
- [ ] P3-F-005: KatanA integration で adapter 型を直接保持しないようにする。

## P6-A-005 (KMM 依存ポリシー)

KLE は KMM に **hard dependency を持たない**。markdown source mapping は `SourceAnchorAdapter` trait 経由で受け取り、KMM 用 implementation は `katana-language-editor-md` crate に閉じる。

- [ ] P6-A-005: この方針を ADR `docs/adr/kle-kmm-dependency.md` に記録する。検査: P3-E-007 の script で KLE crate が KMM / markdown 系 crate を import していないことを確認する。

## 前提 (depends on) / 出力 (provides)

- **前提 (P0 完了)**:
  - P0 governance (`KME` 表記の `KMM` 読み替え方針確定など)
  - dependency leak guard (P0-C-005)

- **出力**:
  - language-neutral editor domain crate (`katana-language-editor`)
  - port trait 7 種 (Syntax / Jump / Hover / Completion / Formatter / Diagnostics / SourceAnchor)
  - opaque `SourceAnchor` DTO + neutral `LineColumnRange` / fingerprint
  - markdown 用 port implementations (`katana-language-editor-md`)
  - editor builder API (port 必須)

## Done criteria

本 repo に関する master 9 章 Done criteria のうち、該当項目:

- [ ] `katana-language-editor` core が UI なしで compile できる
- [ ] KLE が `katana-markdown-model` を含む markdown 系 crate を import していない (P3-E-007 / P3-G-009 script 通過)
- [ ] port trait が KLE crate 内に閉じている
- [ ] markdown 用 implementation が `katana-language-editor-md` に閉じている
- [ ] egui implementation が compatibility adapter として README に記録されている

## drift 検出

- 本ファイルの task ID は master と完全一致する。
- P8-A-001 の CI script が master と本ファイルの task ID 一致を検査する。

## 参照リンク

- [master detailed-design-and-tasks.md](../../katana/docs/architecture/ui-separation/detailed-design-and-tasks.md)
- [master principles.md](../../katana/docs/architecture/ui-separation/principles.md)
- [overview README](../../katana/docs/architecture/ui-separation/README.md)
- [既存 docs/release.md](release.md)
