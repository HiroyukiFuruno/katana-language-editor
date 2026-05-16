## ADDED Requirements

### Requirement: Editor synchronizes KMM metadata on save

The editor SHALL synchronize external KMM metadata when a Markdown document is saved.

#### Scenario: Metadata target moves after edit

- **WHEN** a Markdown document with metadata is edited and saved
- **THEN** the editor sends old source, new source, and metadata to KMM target resolution
- **THEN** moved targets are updated in metadata

#### Scenario: Metadata target cannot be resolved

- **WHEN** KMM cannot safely resolve a metadata target after edit
- **THEN** the editor preserves that target as unresolved
- **THEN** the editor does not silently delete it

### Requirement: Editor metadata sync depends on P0 and P1 contracts

The editor SHALL treat metadata sync as downstream work after shared AST lint and KMM metadata contracts are available.

#### Scenario: Start metadata sync implementation

- **WHEN** editor metadata sync implementation begins
- **THEN** P0 `katana-ast-lint` governance is available
- **THEN** P1 KMM metadata schema and target resolution API are available

### Requirement: Editor contract remains UI-framework neutral

The editor neutral interface SHALL NOT expose egui or Floem implementation types.

#### Scenario: Save metadata from Floem implementation

- **WHEN** the Floem editor implementation saves a Markdown document
- **THEN** it uses neutral metadata sync DTOs
- **THEN** downstream users are not required to import Floem types

### Requirement: Editor does not own viewer synchronization control

The editor SHALL expose editor-side command surfaces without coordinating viewer state.

#### Scenario: KatanA synchronizes editor and viewer

- **WHEN** KatanA decides that editor or viewer state should change
- **THEN** KatanA sends commands to the editor or viewer
- **THEN** the editor does not call the viewer
- **THEN** the editor does not own scroll, selection, or highlight synchronization policy

### Requirement: Save flow MUST go through v0.1.0 neutral DI (theme / strings / settings / host-control)

The save and metadata sync flow SHALL not embed UI strings, colors, autosave intervals, or shortcuts inside the editor crate. All such values MUST come from `Strings` / `Theme` / `EditorSettings` injected via `EditorConfig`, and unresolved-target indications MUST be pushed through `EditorDiagnosticsSink` or `EditorDecorationsSink`.

#### Scenario: Save UI strings come from injected Strings

- **WHEN** the editor emits a save / autosave / unresolved indicator label
- **THEN** the label text is resolved through `Strings` keys supplied by host (KDV en preset is mandatory)
- **THEN** no English / Japanese string literal exists inside the editor crate for these labels

#### Scenario: Unresolved indicators are pushed to host sinks

- **WHEN** KMM resolution yields unresolved targets
- **THEN** the editor pushes diagnostics or decorations through `EditorDiagnosticsSink` / `EditorDecorationsSink`
- **THEN** the editor itself does not render a modal or call the viewer

#### Scenario: Autosave timing follows EditorSettings

- **WHEN** the editor decides whether and when to auto-save
- **THEN** it consults `EditorSettings::autosave`
- **THEN** it does not maintain its own enabled flag or interval

#### Scenario: Host-driven post-resolve writes are tagged

- **WHEN** the host applies follow-up edits via `EditorWriteAccess` after a sync resolution
- **THEN** the host calls `with_origin("kme-sync")` (or an equivalent tag)
- **THEN** those writes are distinguishable from normal user edits in editor events
