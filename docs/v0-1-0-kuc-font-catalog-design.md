# KUC Platform Font Catalog Design

## Status

This design is required for KLE v0.1.0 but is not implemented. It belongs in
`katana-ui-core`, not KLE or KLE Storybook.

## Measured Basis

KUC currently uses `cosmic-text 0.19.0`. Its `FontSystem::new()` performs
system font discovery; its own source states that it should be called once and
the resulting system shared. KUC currently calls it in
`katana-ui-core-text-raster/src/rasterizer.rs:26-48` for every
`PlatformTextRasterizer`. The root creates several such rasterizers through
TextSurface, CommandChrome, and ContextMenu constructors. Candidate font files
are then loaded independently by `load_candidates`.

The required result is one system discovery and one candidate-file load per
catalog key, without sharing text content, cursor/selection, shaping output,
glyph raster output, texture, or IME state between consumer surfaces.

## Ownership and API Shape

KUC text-raster exposes a generic `PlatformFontCatalog` and a catalog-aware
text runtime. The names may vary, but the following ownership is mandatory:

| Value | Owner | May be shared | Must remain isolated |
| --- | --- | --- | --- |
| system `FontSystem`, platform locale, font database, configured candidate fonts, resolved emoji-family fact | `PlatformFontCatalog` | process-wide per catalog key | no document or UI state |
| font system internal resource cache | catalog lock | yes, as a font resource cache only | no TextSurface/KLE/KatanA state |
| `SwashCache`, raster output cache, cache order, texture cache | each `PlatformTextRasterizer` / adapter | no | all entries and eviction |
| text buffer, grapheme bounds, cursor/selection, IME preedit, focus, scroll, hit-test and AccessKit state | each KUC retained surface | no | all state and events |

`PlatformFontCatalog::new` performs the only `FontSystem::new()` and candidate
load for its immutable catalog key. `PlatformTextRasterizer::with_catalog`
retains an `Arc<PlatformFontCatalog>` plus its own `SwashCache` and raster cache.
The existing convenience constructor may obtain a shared catalog, but it must
not call `FontSystem::new()` directly.

The catalog uses controlled mutable access to `FontSystem` only while shaping
or resolving a font. The lock scope must not include KUC event dispatch, KLE
callbacks, texture upload, or consumer code. It is a generic resource boundary,
not a KLE synchronization primitive.

## Catalog Identity

The catalog key includes the ordered proportional, monospace, and emoji
candidate-path lists and the platform/locale inputs that affect the font
database. It excludes per-surface raster cache capacity. Candidate list order
is preserved because it can affect font resolution. A rasterizer whose candidate
configuration does not match its injected catalog returns a typed KUC
configuration error rather than silently using another catalog.

An explicit catalog injection path is required for tests and applications. The
default root factory obtains the process-shared catalog by key so independent
TextSurface, CommandChrome, and ContextMenu roots use the same discovery. The
registry must retain no KLE or document state and must support deterministic
test isolation through a test-only scoped registry, not a production reset API.

## Root Construction Rule

`EguiTextCommandSurfaceAdapter` creates or receives one catalog-aware runtime
for its retained root, then passes cloned catalog handles to its TextSurface,
CommandChrome, and ContextMenu adapters. `EguiCommandChromeAdapter` must not
construct a second TextSurface rasterizer with a different catalog. A standalone
generic KUC component follows the same factory rule.

KLE receives none of these types. It calls the retained KUC root and consumes
the opaque root frame. A KLE font cache, an egui font reset, a Storybook cache,
or a test time threshold is not an acceptable substitute.

## Emoji and IME Contract

The catalog resolves the platform color-emoji face once after candidate loading
and exposes only immutable rasterization facts: resolved family, actual face
identity, loaded-file SHA-256, and catalog fingerprint. The exact input
sequence `Japanese IME <commit> \u2b50\ufe0f <ZWJ sequence>` must retain code
points, grapheme ranges, selection/caret behavior, and color pixels on every
release profile. The required profile families are `Apple Color Emoji` on
macOS, `Segoe UI Emoji` on Windows, and a verified installed color-emoji face
on Linux (the KUC CI setup must provision the pinned face when the runner lacks
one, normally `Noto Color Emoji`). The final KUC root RGBA evidence for `⭐️`
must be non-monochrome and must not equal a `☆` or text fallback rendering on
each profile.

No layer may replace the glyph, append a fallback font in KLE, substitute
`SansSerif`, use a platform-specific assertion skip, or accept an unresolved
color face. A missing family, missing face bytes, changed face hash, or failed
font provisioning is a KUC release-profile failure, not a downgraded
code-point-only test.

### Exact `⭐️` proof

The existing macOS raster unit that draws `🔥⭐️` and counts chromatic pixels is
not sufficient for the requested star. `🔥` can provide all chromatic pixels
while `⭐️` falls back to `☆`, a monochrome text glyph, or a missing-glyph box.
Spark must replace that aggregate acceptance with the following three linked
tests; the old test may remain as a broad regression test but cannot be a
release leaf.

1. A raster-unit case on each release profile draws the exact isolated scalar
   sequence `\u{2b50}\u{fe0f}` with no other emoji. It proves one grapheme
   range, the required resolved color face and immutable file identity,
   non-zero pixels in that grapheme crop, and chromatic pixels in that crop
   rather than in the whole request.
2. A same-config control draws `☆` (`\u{2606}`) in the same foreground,
   scale, wrap, and raster context. The `⭐️` crop must have a different
   deterministic pixel hash and a non-zero pixel delta from the control; the
   color assertion applies only to the `⭐️` crop. Different crop dimensions are
   allowed, but both crops must be non-empty and the comparison must use their
   own authoritative grapheme bounds rather than guessed coordinates.
3. A retained TextSurface/CommandChrome/ContextMenu root on each release
   profile receives Japanese IME commit, the exact `⭐️` sequence, and a ZWJ
   sequence through one public RawInput trace. The final opaque root RGBA frame
   exposes the authoritative text grapheme bounds/record required to crop the
   star. The same isolated star/control assertions apply to the final frame,
   with catalog identity, face SHA-256, AccessKit text, caret/hit range, and no
   duplicate FontSystem discovery.

The KLE consumer, Storybook encoder, and actual KatanA host E2E repeat only
the root-frame identity and recorded crop hashes. They do not rasterize a
second reference glyph or use labels, UTF-8 strings, font-family reports, or
egui shapes as a surrogate for the KUC proof.

## Strict Tests

1. Construct TextSurface, CommandChrome, and ContextMenu roots concurrently
   with the same explicit catalog. Verify exactly one system discovery and one
   candidate-load sequence in catalog instrumentation.
2. Construct two surfaces from one catalog, mutate/rasterize/evict one, then
   prove the other's raster, texture, selection, IME, focus, and output cache
   are unchanged.
3. Dispose one root and construct another; prove catalog reuse is valid and no
   stale text/selection/texture/AccessKit state is visible.
4. Run repeated identical RawInput frames and require equal opaque root pixel
   hashes, records, and AccessKit evidence.
5. On every release profile, run Japanese IME preedit and commit, `⭐️` VS16,
   and a ZWJ sequence through the full retained root. Assert code points,
   grapheme bounds, hit/caret range, resolved face identity, face SHA-256, and
   final RGBA. The isolated `⭐️` crop must have non-monochrome color pixels and
   differ from the same-config `☆` control crop on every profile.
6. Reject mismatched catalog/raster configuration, missing catalog runtime,
   mixed catalog frame records, and constructor paths that directly create a
   `FontSystem` outside catalog creation.

The tests use counters and invariant assertions, never elapsed-time thresholds.
They must run with ordinary strict coverage and cannot skip the KLE/KUC root
combination.
