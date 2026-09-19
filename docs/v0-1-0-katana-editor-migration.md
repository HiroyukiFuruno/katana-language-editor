# KLE v0.1.0 KatanA Editor Migration Map

## Scope

This note maps KatanA `katana-ui/src/views/panels/editor` behavior to KLE v0.1.0 implementation boundaries. The first release targets full KatanA editor parity; host-owned execution details are still release blockers until they have KLE contracts and downstream KatanA integration evidence.

## Historical Snapshot Warning

The following migration lists describe a rejected legacy implementation plan,
not the current v0.1.0 ownership contract. They must not be used to justify a
KLE text surface, semantic editor state/action API, egui fallback, or partial
MVP acceptance. The authoritative boundary is
`docs/v0-1-0-editor-requirements.md`: KUC owns generic retained UI and platform
text; KLE performs only opaque one-time transit; unchanged KatanA resolves all
editor semantics and host effects.

The current source audit finds no `EguiLanguageEditor::setup_fonts` or
`ctx.set_fonts` implementation path. Any older entry naming that method is an
obsolete hypothesis, not an explanation or acceptance proof for input jitter.
Jitter can be accepted only after the KUC opaque-root continuous-RawInput
stability contract in the OpenSpec passes.

## Rejected Legacy `katana-language-editor` Ownership Snapshot

- Neutral editor DTOs and traits: content, cursor, selection, search, diagnostics, decorations, scroll, clipboard, accessibility, settings, theme, locale, typography, spacing.
- Character-indexed edit operations: `set_text`, `insert_at`, `replace_range`, `apply_batch`, and write origin events.
- Pending cursor restore after host-owned authoring transforms, represented as char-index `TextRange` selection restore.
- Text clipboard contract through host-injected `ClipboardBackend`.
- Search query/options/matches and next/previous navigation.
- Neutral editor action request/event contract for save, format, image ingest, authoring command, diagnostic fix, view mode, split toggle, and split direction.
- Neutral authoring command inventory, code block kind inventory, toolbar/code-block menu state, and suppression reason contract.
- Diagnostic popup item contract and typed fix / fix-all / docs actions.
- Problems panel state/action contract for open state, scope, status count, and visible fix-all diagnostics.
- Gutter line DTO and typed line-number click action contract.
- Navigation DTOs for preview/editor scroll source ownership, select-and-jump PreviewOnly fallback, and document-search navigation fallback metadata.
- Document state DTOs for dirty/save-clean transitions, reference read-only guard, refresh side-effect reporting, external-change undo pending, and workspace/document scoped identity.

## Rejected Legacy `katana-language-editor-egui` Ownership Snapshot

- egui `TextEdit` MVP adapter with injected theme, typography, settings, syntax highlighter, clipboard, and search state.
- Line number gutter model/rendering with active-line and diagnostic marker color from injected theme tokens.
- Gutter-line state extraction and line click request emission.
- Search match background rendering using injected search color tokens.
- Injected `SyntaxHighlighter` layout path with invalid span fallback.
- Stable TextEdit ID and external-change undo recording through `EguiEditorUndo`.
- OS emoji/custom font fallback through `EguiEditorFontConfig` and `EguiLanguageEditor::setup_fonts`.
- Pending cursor restore application to internal selection/cursor state and egui TextEdit memory.
- Diagnostic popup lookup for gutter lines.
- Problems panel state storage and typed action emission for toggle, scope change, and visible fix-all.
- Pending navigation request storage and navigation action emission for scroll sync, select-and-jump, and document-search navigation.
- Document-state control for dirty reports, save clean reports, reference/read-only synchronization, and pending external undo records.
- Explicit `Unsupported` returns for egui MVP gaps that require Floem or host UI ownership.

## Rejected Legacy Host-Ownership Framing

- Workspace/document identity, file IO, virtual/reference document policy, save/autosave orchestration.
- Markdown/KMM/KCF-specific highlighter implementation, Mermaid/Draw.io rendering, and preview source anchors.
- Image clipboard ingest and workspace file generation.
- Global shortcut priority, context menu composition, and toolbar command UI.
- Preview scroll-sync coordinator and preview-side viewport ownership.

Host-owned means KatanA remains responsible for executing file, Markdown, preview, and workspace-specific behavior. It does not mean those features are out of KLE v0.1.0 scope. KLE must still expose the neutral action/state/event contracts that allow KatanA to preserve the existing editor behavior.

## Rejected Legacy KLE Host-E2E Framing

- Authoring toolbar, context menu action execution, code block menu lifecycle, and viewport-clamped cursor-anchored toolbar behavior remain composed by KatanA host UI, while KLE owns the neutral state/action contracts and KLE-owned actual `KatanaApp` host E2E evidence.
- File image ingest, clipboard image ingest, and clipboard file URL ingest remain KatanA workspace/file operations, while KLE emits typed ingest requests and KLE-owned actual `KatanaApp` host E2E verifies deterministic asset ingest through KLE actions.
- Diagnostic popup lifecycle, hover suppression, Problems panel actions, and lint-fix review launch remain connected to KatanA host review UI, while KLE owns diagnostic/problem action contracts and KLE-owned actual `KatanaApp` host E2E execution evidence.
- Preview/editor scroll sync, view-mode toggle cycle, split direction, select-and-jump fallback, save/file IO, and document-state refresh remain KatanA host coordination concerns, while KLE exposes the neutral reports/actions and KLE-owned actual `KatanaApp` host E2E proves the integration path.

## Verification

- `just check`
- `just katana-interface-check`
- `just katana-downstream-check`
- `just storybook-window-smoke`
- `just storybook-interaction-check`
- `just storybook-emoji-check`
- `just storybook-contract-check`
- `just storybook-motion-artifact` (partial KLE/KUC motion only; not full-editor release evidence)

## Remaining Non-Migration Release Work

- Public release/tag creation and downstream KatanA tag dependency build check.
- KDV post-release preset follow-up when theme / i18n / typography / settings preset gaps remain.
