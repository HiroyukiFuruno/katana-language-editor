use std::{collections::BTreeMap, fs, path::Path};

use super::super::{
    artifact_model::SourceClosureArtifact,
    fingerprint::sha256_hex,
    operational_cli::CliOptions,
    operational_input::FIXED_KATANA_REVISION,
    source_requirement_alias_ledger::{ALIAS_LEDGER, RequirementSourceAliasLedger},
};
use super::model::{AnchoredFile, SCHEMA, SourceAnchorCatalog};
use super::scan::scan_file;

pub(super) fn validate(options: &CliOptions) -> Result<(), String> {
    let catalog_path = Path::new(options.one("catalog")?);
    let catalog_bytes = fs::read(catalog_path)
        .map_err(|error| format!("failed to read source anchor catalog: {error}"))?;
    let catalog: SourceAnchorCatalog = serde_json::from_slice(&catalog_bytes)
        .map_err(|error| format!("invalid source anchor catalog: {error}"))?;
    let mut canonical = serde_json::to_vec_pretty(&catalog)
        .map_err(|error| format!("failed to canonicalize source anchor catalog: {error}"))?;
    canonical.push(b'\n');
    if canonical != catalog_bytes {
        return Err("source anchor catalog is not canonical JSON".into());
    }
    let source_path = Path::new(options.one("source-closure")?);
    let source_bytes = fs::read(source_path)
        .map_err(|error| format!("failed to read canonical source closure: {error}"))?;
    let source: SourceClosureArtifact = serde_json::from_slice(&source_bytes)
        .map_err(|error| format!("invalid canonical source closure: {error}"))?;
    validate_authority(&source, &source_bytes)?;
    verified_aliases(&source)?;
    let expected = expected_from_embedded_source(&source, &source_bytes, &catalog.files)?;
    if serde_json::to_value(expected).map_err(|error| error.to_string())?
        != serde_json::to_value(catalog).map_err(|error| error.to_string())?
    {
        return Err("source anchor catalog does not match authenticated AST reconstruction".into());
    }
    Ok(())
}

pub(super) fn validate_authority(
    source: &SourceClosureArtifact,
    bytes: &[u8],
) -> Result<(), String> {
    if source.root.katana_revision != FIXED_KATANA_REVISION {
        return Err("canonical source closure does not use the fixed KatanA revision".into());
    }
    if source.root.requirement_source_aliases_fingerprint != sha256_hex(ALIAS_LEDGER.as_bytes()) {
        return Err("canonical source closure does not bind the checked-in alias ledger".into());
    }
    if bytes.is_empty() {
        return Err("canonical source closure is empty".into());
    }
    Ok(())
}

pub(super) fn verified_aliases(
    source: &SourceClosureArtifact,
) -> Result<Vec<super::super::source_requirement_alias_ledger::AliasEntry>, String> {
    let aliases = RequirementSourceAliasLedger::verified_alias_entries()?;
    let mut seen = std::collections::BTreeSet::new();
    for alias in &aliases {
        if !seen.insert(alias.source_path.clone()) {
            continue;
        }
        super::super::operational_evidence::validate_rust_source_path(&alias.source_path)?;
        let matches = source
            .files
            .iter()
            .filter(|file| file.path == alias.source_path)
            .count();
        if matches != 1 {
            return Err(format!(
                "canonical source closure must contain exactly one {} entry for {}",
                matches, alias.source_path
            ));
        }
    }
    Ok(aliases)
}

fn expected_from_embedded_source(
    source: &SourceClosureArtifact,
    source_bytes: &[u8],
    files: &[AnchoredFile],
) -> Result<SourceAnchorCatalog, String> {
    let mut aliases_by_path = BTreeMap::<String, Vec<_>>::new();
    for alias in verified_aliases(source)? {
        aliases_by_path
            .entry(alias.source_path.clone())
            .or_default()
            .push(alias);
    }
    if files.len() != aliases_by_path.len() {
        return Err("source anchor catalog has added or missing file records".into());
    }
    let mut expected_files = Vec::new();
    for (path, aliases) in aliases_by_path {
        let file = files
            .iter()
            .find(|file| file.path == path)
            .ok_or_else(|| format!("source anchor catalog is missing source file {path}"))?;
        let source_bytes = file.source_utf8.as_bytes();
        let source_hash = sha256_hex(source_bytes);
        if source_hash != file.sha256 {
            return Err(format!("embedded source bytes hash mismatch: {path}"));
        }
        let closure_file = source
            .files
            .iter()
            .find(|source_file| source_file.path == path)
            .ok_or_else(|| format!("source closure does not contain alias file: {path}"))?;
        if source_hash != closure_file.sha256 {
            return Err(format!(
                "embedded source differs from canonical source closure: {path}"
            ));
        }
        let text = std::str::from_utf8(source_bytes)
            .map_err(|error| format!("embedded source is not UTF-8 ({path}): {error}"))?;
        let syntax = syn::parse_file(text)
            .map_err(|error| format!("embedded source cannot be parsed ({path}): {error}"))?;
        let (_, mut unresolved) = scan_file(text, &syntax)?;
        let anchors = super::generation::scan_aliases(text, &syntax, &aliases)?;
        unresolved.sort();
        expected_files.push(AnchoredFile {
            path,
            sha256: source_hash,
            source_utf8: text.to_owned(),
            anchors,
            unresolved_syntax: unresolved,
        });
    }
    expected_files.sort_by(|left, right| left.path.cmp(&right.path));
    Ok(SourceAnchorCatalog {
        schema: SCHEMA.into(),
        source_closure_sha256: sha256_hex(source_bytes),
        root: source.root.clone(),
        alias_ledger_sha256: sha256_hex(ALIAS_LEDGER.as_bytes()),
        files: expected_files,
    })
}

#[cfg(test)]
#[path = "validation_tests.rs"]
mod tests;
