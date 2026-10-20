use std::{collections::BTreeMap, fs, io::Write, path::Path};

use serde::Serialize;
use syn::File;

use super::super::{
    artifact_model::SourceClosureArtifact, fingerprint::sha256_hex, operational_cli::CliOptions,
    operational_process::git_output, source_requirement_alias_ledger::ALIAS_LEDGER,
};
use super::model::{AnchoredFile, SCHEMA, SourceAnchorCatalog};
use super::scan::scan_file;
use super::validation::{validate_authority, verified_aliases};

pub(super) fn emit(options: &CliOptions) -> Result<(), String> {
    let source_path = Path::new(options.one("source-closure")?);
    let source_bytes = fs::read(source_path)
        .map_err(|error| format!("failed to read canonical source closure: {error}"))?;
    let source: SourceClosureArtifact = serde_json::from_slice(&source_bytes)
        .map_err(|error| format!("invalid canonical source closure: {error}"))?;
    validate_authority(&source, &source_bytes)?;
    let repo = Path::new(options.one("katana-repo")?);
    let mut aliases_by_path = BTreeMap::<String, Vec<_>>::new();
    for alias in verified_aliases(&source)? {
        aliases_by_path
            .entry(alias.source_path.clone())
            .or_default()
            .push(alias);
    }
    let mut files = Vec::new();
    for (path, aliases) in aliases_by_path {
        let source_bytes = git_blob(repo, &path)?;
        let source_text = std::str::from_utf8(&source_bytes)
            .map_err(|error| format!("fixed Git source is not UTF-8 ({path}): {error}"))?;
        let closure_file = source
            .files
            .iter()
            .find(|file| file.path == path)
            .ok_or_else(|| {
                format!("alias source is absent from canonical source closure: {path}")
            })?;
        let source_hash = sha256_hex(&source_bytes);
        if source_hash != closure_file.sha256 {
            return Err(format!(
                "fixed Git blob hash differs from source closure: {path}"
            ));
        }
        let syntax = syn::parse_file(source_text)
            .map_err(|error| format!("fixed Git source cannot be parsed ({path}): {error}"))?;
        let (_, mut unresolved) = scan_file(source_text, &syntax)?;
        let mut anchors = scan_aliases(source_text, &syntax, &aliases)?;
        anchors.sort();
        unresolved.sort();
        files.push(AnchoredFile {
            path,
            sha256: source_hash,
            source_utf8: source_text.to_owned(),
            anchors,
            unresolved_syntax: unresolved,
        });
    }
    files.sort_by(|left, right| left.path.cmp(&right.path));
    let catalog = SourceAnchorCatalog {
        schema: SCHEMA.into(),
        source_closure_sha256: sha256_hex(&source_bytes),
        root: source.root,
        alias_ledger_sha256: sha256_hex(ALIAS_LEDGER.as_bytes()),
        files,
    };
    write_exclusive_json(Path::new(options.one("output")?), &catalog)
}

pub(super) fn scan_alias(
    source: &str,
    syntax: &File,
    requirement_id: &str,
    short_reference: &str,
) -> Result<Vec<super::model::AnchorFact>, String> {
    let (anchors, _) = scan_file(source, syntax)?;
    Ok(anchors
        .into_iter()
        .map(|mut fact| {
            fact.requirement_id = requirement_id.to_owned();
            fact.short_reference = short_reference.to_owned();
            fact
        })
        .collect())
}

pub(super) fn scan_aliases(
    source: &str,
    syntax: &File,
    aliases: &[super::super::source_requirement_alias_ledger::AliasEntry],
) -> Result<Vec<super::model::AnchorFact>, String> {
    let mut anchors = Vec::new();
    for alias in aliases {
        anchors.extend(scan_alias(
            source,
            syntax,
            &alias.requirement_id,
            &alias.short_reference,
        )?);
    }
    anchors.sort();
    Ok(anchors)
}

fn git_blob(repo: &Path, path: &str) -> Result<Vec<u8>, String> {
    git_blob_at(
        repo,
        super::super::operational_input::FIXED_KATANA_REVISION,
        path,
    )
}

fn git_blob_at(repo: &Path, revision: &str, path: &str) -> Result<Vec<u8>, String> {
    super::super::operational_evidence::validate_rust_source_path(path)?;
    let spec = format!("{revision}:{path}");
    git_output(repo, &["show", &spec])
}

fn write_exclusive_json<T: Serialize>(path: &Path, value: &T) -> Result<(), String> {
    let mut bytes = serde_json::to_vec_pretty(value)
        .map_err(|error| format!("failed to serialize source anchor catalog: {error}"))?;
    bytes.push(b'\n');
    let mut output = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
        .map_err(|error| format!("failed to create source anchor catalog: {error}"))?;
    output
        .write_all(&bytes)
        .map_err(|error| format!("failed to write source anchor catalog: {error}"))
}

#[cfg(test)]
#[path = "generation_tests.rs"]
mod tests;
