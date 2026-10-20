# KLE v0.1.0 Release Integration Design

## Decision

KLE v0.1.0 is released only after a public KUC release that provides the
generic retained editor root required by KLE. The fixed KatanA source is the
frozen source of editor requirements; it is not a KLE runtime dependency and
does not define a pre-release host ABI gate. KLE must not bridge, wrap, or
duplicate either UI runtime.

## Ownership

| Layer | Owns | Must not own |
| --- | --- | --- |
| KUC | retained text surface, platform font catalog and color-emoji shaping, gutter layout, command chrome, context-menu geometry, accessibility projection, frame/artifact record | Markdown, KatanA actions, document state, filesystem, clipboard acquisition, semantic ranges, host coordinates |
| KLE | one retained opaque KUC-root binding and one-time event forwarding | text, selection, cursor, search/replace state, Markdown action enums, rendering, glyph fallback, geometry, event replay |
| KatanA | post-release consumer projection, opaque token-to-`AppAction` routing, buffer/document/search/clipboard/filesystem/history effects | KUC child geometry, rasterization, font fallback, synthetic KLE events |

## Compatibility And Publication Order

1. Complete KUC's generic API and test it on macOS, Linux, and Windows with
   Japanese IME input, exact `⭐️` VS16 color-glyph rendering, ZWJ input,
   accessibility, command chrome, context-menu overflow, and deterministic
   artifact contracts.
2. Commit, tag, publish, and externally resolve that KUC release. KLE may use
   no path dependency in its release package.
3. Bind KLE only to the public KUC opaque-root API; delete and reject all
   legacy KLE-local renderer/state paths. Resolve KLE in a clean external
   package build against the published KUC version.
4. Execute every source-derived KLE leaf through its public input surface. Each
   leaf needs the real KUC frame/AccessKit record and the KLE-owned observable
   result or declared no-mutation outcome. Full Storybook is one required
   artifact, not a substitute for input and accessibility evidence.
5. Publish KLE v0.1.0 only after all previous gates pass.
6. After the KLE registry release is visible, KatanA #336 may adopt that exact
   published version and prove KatanA-specific action, document, filesystem,
   history, and workspace effects. This downstream work is not a KLE release
   prerequisite.

## Downstream Change Rule

An upstream repository does not directly edit its downstream consumer. Each
consumer migration starts from that consumer's Issue after the upstream release
records its published versions, API migration notes, and validation evidence.
KLE tracks the KUC consumer migration in GitHub issue #8. KatanA adoption must
start from a separate KatanA Issue after KLE v0.1.0 is public.

## Non-Negotiable Release Gates

- `⭐️` is verified as the exact VS16 input and distinct from `☆`; code-point
  preservation or a star-shaped fallback alone fails.
- KLE public sources may not contain a `TextEdit`, `ScrollArea`, `Area`, text
  state, semantic range conversion, local glyph/font/raster cache, or action
  semantic decoder.
- KUC public sources may not contain KatanA, KLE, Markdown, document, path,
  clipboard, or filesystem semantics.
- Every source-derived leaf must have KLE-owned public-input, KUC frame,
  AccessKit, and observable-result evidence. KatanA-specific host effects are
  explicitly deferred to KatanA #336 after the KLE registry release.
- No release tag, GitHub Release, or registry publish is performed from a
  dirty source tree.
