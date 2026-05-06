## ADDED Requirements

### Requirement: Editor synchronizes KME metadata on save

The editor SHALL synchronize external KME metadata when a Markdown document is saved.

#### Scenario: Metadata target moves after edit

- **WHEN** a Markdown document with metadata is edited and saved
- **THEN** the editor sends old source, new source, and metadata to KME target resolution
- **THEN** moved targets are updated in metadata

#### Scenario: Metadata target cannot be resolved

- **WHEN** KME cannot safely resolve a metadata target after edit
- **THEN** the editor preserves that target as unresolved
- **THEN** the editor does not silently delete it

### Requirement: Editor metadata sync depends on P0 and P1 contracts

The editor SHALL treat metadata sync as downstream work after shared AST lint and KME metadata contracts are available.

#### Scenario: Start metadata sync implementation

- **WHEN** editor metadata sync implementation begins
- **THEN** P0 `katana-ast-lint` governance is available
- **THEN** P1 KME metadata schema and target resolution API are available

### Requirement: Editor contract remains UI-framework neutral

The editor neutral interface SHALL NOT expose egui or Floem implementation types.

#### Scenario: Save metadata from Floem implementation

- **WHEN** the Floem editor implementation saves a Markdown document
- **THEN** it uses neutral metadata sync DTOs
- **THEN** downstream users are not required to import Floem types
