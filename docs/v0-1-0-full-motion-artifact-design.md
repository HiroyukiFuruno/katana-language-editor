# KLE Full-Motion Artifact Design

## Required User Outcome

KLE v0.1.0 Storybook must automatically produce one reviewable continuous
video for the complete Katana-equivalent editor interaction sequence. The
sequence includes resize, Japanese IME, exact `⭐️` (`U+2B50 U+FE0F`), search,
and Markdown controls. Manual operation, fixed-viewport substitution, or
multiple disconnected clips is not acceptable evidence.

## Ownership Boundary

KUC owns the generic text raster, color emoji glyphs, geometry, hit testing,
AccessKit projection, opaque root artifact receipt, and artifact composition.
KLE owns only the public input loop, opaque root relay, and the release
evidence consumer.

KLE must apply every KUC motion stage unchanged. In particular it must not
replace a KUC-issued resize `RawInput::screen_rect` with a fixed viewport.
KLE must not decode, crop, scale, pad, compose, or otherwise inspect opaque
KUC raster artifacts in order to make a video.

## Required KUC Public Capability

KUC needs a generic opaque full-motion artifact writer that accepts the exact
variable-viewport receipt sequence and emits one fixed-export-canvas GIF/MP4.
KUC chooses and applies normalisation internally while preserving each source
frame's actual viewport. The public output must retain opaque source receipt
provenance without exposing pixels, paint plans, font data, texture state, or
hit-test internals to KLE.

This requirement is tracked by [KUC #34](https://github.com/HiroyukiFuruno/katana-ui-core/issues/34).

## KLE Integration Contract

Once the capability is available from a published registry KUC version, KLE
will:

1. issue the complete KUC motion plan and apply every frame unchanged;
2. render each frame through the one retained opaque KUC root;
3. forward each opaque receipt once to the KUC full-motion artifact writer;
4. write a KLE manifest that references the single KUC artifact and records
   plan order, KUC provenance, root record hashes, and declared effects;
5. reject incomplete plans, idle frames, input rewriting, duplicate receipts,
   changed source/decode hashes, missing resize evidence, or missing video.

## Required Verification

The KUC/KLE contract must pass on macOS, Windows, and Linux with a single
artifact containing the full exact plan. The artifact manifest must verify:

- source and decoded frame count;
- export dimensions and per-source viewport dimensions;
- source/decode frame hashes and root provenance;
- Japanese IME preedit/commit;
- exact colored `⭐️`, distinct from `☆`;
- glyph measurement, hit test, and AccessKit evidence.

The KLE release gate consumes that public registry capability. It does not
replace it with KLE-local rendering or a partial Storybook approximation.
