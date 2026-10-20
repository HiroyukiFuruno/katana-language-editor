# KUC Unicode Evidence Design

## Purpose

KUC owns platform text rasterization and accessibility output. The v0.1.0
evidence for Japanese IME, exact `⭐️` and `☆` must therefore be generated
inside KUC. KLE only consumes the KUC root through its opaque lease and does
not inspect text, glyph geometry, font data, crop pixels or AccessKit nodes.

## Canonical Evidence

One retained `EguiTextCommandSurfaceRoot` frame produces one canonical generic
artifact containing all of the following:

1. The exact scalar sequences for `⭐️` (`U+2B50 U+FE0F`), `☆` (`U+2606`) and
   the required ZWJ fixture.
2. A physical IME preedit and commit trace, caret bounds and grapheme hit
   ranges from that retained root.
3. Isolated RGBA crops for `⭐️` and `☆`, each crop hash and chromatic-pixel
   count, and a non-zero difference between the two.
4. The resolved platform emoji face family, file path, SHA-256 and catalog
   fingerprint.
5. The current frame's actual `MultilineTextInput` AccessKit node role, value
   scalar sequence and bounds, plus the KUC AccessKit snapshot hash.
6. The matching root frame, root record and root RGBA hashes.

The artifact is invalid if a scalar sequence, required node, role, bounds,
font pin, hash, color crop, IME event or root correlation is absent or
inconsistent. Synthetic fixture crops can test validation failures but cannot
serve as release evidence.

## Profile Rules

Each `macos-latest`, `windows-latest` and `ubuntu-latest` run records its own
font identity and artifact. A missing or unavailable platform color emoji face
fails the release profile; it is not a skip. Reusing a profile artifact,
mixing profile fingerprints or accepting a code-point-only assertion fails.

## Ownership

- KUC: font discovery, rasterization, retained root, IME, AccessKit capture,
  crop comparison, canonical artifact and profile validation.
- KLE: opaque root retain/synchronize and one-shot transport only.
- KatanA: final document and host effects after an approved downstream
  adoption. KUC evidence cannot substitute this host gate.
