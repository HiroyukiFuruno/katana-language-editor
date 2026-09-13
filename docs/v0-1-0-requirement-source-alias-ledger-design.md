# v0.1.0 Requirement Source Alias Ledger Design

## Purpose

`v0-1-0-editor-requirements.md` contains both fully qualified KatanA source
paths and short references such as `toolbar_popup.rs`. A filename is not a
source identity: `ui.rs`, `types.rs`, `mod.rs`, and several other names occur
in multiple KatanA modules. The release gate must not infer an editor source
from a filename search or silently leave it outside source closure.

The existing `SourceRequirementsLedger` validates every machine-resolvable
KatanA UI source in the `Editor Parity Requirements` section against the
source-universe. This document specifies the next strict contract for the
remaining short references.

## Ownership

The ledger is a KLE release-spec artifact only. It identifies fixed KatanA
sources and does not copy their behavior into KLE. It must not contain KUC
rendering, KLE widgets, document text, ranges, paths, actions, host state, or
test fixtures.

KUC remains the owner of generic UI and artifact rendering. KatanA remains the
owner of semantic behavior and all host effects.

## Canonical Artifact

Create `docs/v0-1-0-editor-requirement-source-aliases.json` with this schema:

```json
{
  "schema_version": "1",
  "katana_revision": "4f6a6287c650a38633c7baeb544a92e739c68567",
  "entries": [
    {
      "requirement_id": "toolbar.focus-retain",
      "short_reference": "toolbar_popup.rs",
      "source_path": "crates/katana-ui/src/views/panels/editor/toolbar_popup.rs"
    }
  ],
  "non_katana_references": [
    {
      "requirement_id": "example.external-reference",
      "short_reference": "platform_text_surface.rs",
      "owner": "KUC-or-KLE",
      "reason": "not a KatanA source-closure input"
    }
  ]
}
```

Every short reference in a requirement row must have exactly one entry keyed by
its requirement ID, or one explicit non-KatanA classification keyed by the
same requirement ID. A shared
filename used by more than one requirement gets one entry per requirement; a
single global basename mapping is prohibited.

## Validation Contract

The release gate will validate all of the following:

1. The alias artifact schema and fixed KatanA SHA are exact.
2. Every source path is a normalized relative path under the fixed KatanA tree.
3. The source path is present in the source-universe directly or below a
   declared directory root, then resolves through the root manifest.
4. Every requirement short reference has exactly one classification. Missing,
   duplicate, stale, ambiguous, and unused aliases fail closed.
5. A `non_katana_references` entry cannot point under `crates/katana-ui` and
   must name its owning boundary and rationale.
6. Changing the requirements, source-universe, root manifest, alias ledger, or
   fixed KatanA source invalidates the canonical source-closure fingerprint.

No runtime basename scan, fuzzy match, path suffix match, wildcard exception,
or test-only alias is allowed. The fixed KatanA checkout is only used to prove
the declared path exists at the pinned revision.

## Delivery Plan

1. Enumerate every short source reference with its requirement ID and classify
   it as KatanA behavior source or an explicit external boundary.
2. Replace ambiguous prose-only references with fully qualified paths where the
   requirement document can express them directly. Use the alias artifact only
   when retaining a short reference improves the table readability.
3. Add the alias ledger parser and unit regressions for missing, duplicate,
   wrong-SHA, wrong-owner, ambiguous, stale, and manifest-omitted entries.
4. Run the checker against the fixed KatanA revision. It must emit a specific
   failure for every unresolved reference before materialization.
5. Make the alias ledger a required source-closure provenance input and rerun
   the full three-profile artifact pipeline once KUC #40 provides the generic
   full-editor artifact plan.

## Done Definition

This design is complete only when every requirement source reference is either
an exact source-closure input or an explicit external-boundary record. It does
not produce terminal leaves, three-OS Storybook media, or KatanA host E2E by
itself; those remain separate release gates.
