use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use super::fingerprint::sha256_hex;
use super::operational_evidence::validate_rust_source_path;
use super::operational_input::FIXED_KATANA_REVISION;
use super::operational_loader::VerifiedSourceClosureInput;
use crate::system::ProcessService;

pub struct ActualReferenceVerifier;

pub struct ActualReferenceBinding<'a> {
    verified: &'a VerifiedSourceClosureInput,
    katana_root: PathBuf,
    captured_sha256: BTreeMap<String, String>,
}

impl ActualReferenceVerifier {
    pub fn bind<'a>(
        verified: &'a VerifiedSourceClosureInput,
        katana_root: &Path,
    ) -> Result<ActualReferenceBinding<'a>, String> {
        let root = resolve_git_root(katana_root)?;
        verify_revision(&root)?;
        let captured_sha256 = verify_tree_capture(verified, &root)?;
        for seed in &verified.input().katana_seed_paths {
            if !captured_sha256.contains_key(seed) {
                return Err(format!("KatanA seed is not captured in the tree: {seed}"));
            }
        }
        Ok(ActualReferenceBinding {
            verified,
            katana_root: root,
            captured_sha256,
        })
    }
}

impl ActualReferenceBinding<'_> {
    pub(crate) fn input(&self) -> &super::operational_input::SourceClosureInput {
        self.verified.input()
    }

    pub(crate) fn katana_root(&self) -> &Path {
        &self.katana_root
    }

    pub(crate) fn verify_scanned_file(
        &self,
        relative_path: &str,
        scanned_sha256: &str,
    ) -> Result<(), String> {
        let expected = self
            .captured_sha256
            .get(relative_path)
            .ok_or_else(|| format!("discovered KatanA source is not captured: {relative_path}"))?;
        let actual = read_actual_sha256(&self.katana_root, relative_path)?;
        if actual != *expected || scanned_sha256 != expected {
            return Err(format!(
                "discovered KatanA source SHA-256 mismatch: {relative_path}"
            ));
        }
        Ok(())
    }
}

fn resolve_git_root(path: &Path) -> Result<PathBuf, String> {
    let root = path
        .canonicalize()
        .map_err(|error| format!("KatanA root is missing or unreadable: {error}"))?;
    if !root.is_dir() {
        return Err("KatanA root must be a directory".into());
    }
    let reported = git_text(&root, &["rev-parse", "--show-toplevel"])?;
    let reported = PathBuf::from(reported.trim())
        .canonicalize()
        .map_err(|error| format!("git top-level path is unreadable: {error}"))?;
    if reported != root {
        return Err("supplied KatanA root is not the git repository top-level".into());
    }
    Ok(root)
}

fn verify_revision(root: &Path) -> Result<(), String> {
    let revision = git_text(root, &["rev-parse", "HEAD"])?;
    if revision.trim() != FIXED_KATANA_REVISION {
        return Err("supplied KatanA root revision does not match the fixed revision".into());
    }
    Ok(())
}

fn verify_tree_capture(
    verified: &VerifiedSourceClosureInput,
    root: &Path,
) -> Result<BTreeMap<String, String>, String> {
    let tree = &verified.input().root.evidence.katana_tree;
    let mut captured = BTreeMap::new();
    let mut paths = BTreeSet::new();
    let mut actual_records = Vec::new();
    for entry in tree {
        validate_rust_source_path(&entry.source_path)?;
        if !paths.insert(&entry.source_path) {
            return Err(format!(
                "duplicate KatanA tree source path: {}",
                entry.source_path
            ));
        }
        let actual = read_actual_sha256(root, &entry.source_path)?;
        if actual != entry.evidence.sha256 {
            return Err(format!(
                "KatanA tree capture differs from actual source: {}",
                entry.source_path
            ));
        }
        actual_records.push(format!("{}\0{}\n", entry.source_path, actual));
        captured.insert(entry.source_path.clone(), entry.evidence.sha256.clone());
    }
    actual_records.sort_unstable();
    if sha256_hex(actual_records.concat().as_bytes())
        != verified.input().root.katana_tree_fingerprint
    {
        return Err("actual KatanA tree fingerprint mismatch".into());
    }
    Ok(captured)
}

fn read_actual_sha256(root: &Path, relative_path: &str) -> Result<String, String> {
    let path = root.join(relative_path);
    let canonical = path
        .canonicalize()
        .map_err(|error| format!("captured KatanA source is missing: {relative_path}: {error}"))?;
    if !canonical.starts_with(root) || !canonical.is_file() {
        return Err(format!(
            "captured KatanA source is foreign: {relative_path}"
        ));
    }
    let bytes = std::fs::read(&canonical).map_err(|error| {
        format!("captured KatanA source is unreadable: {relative_path}: {error}")
    })?;
    Ok(sha256_hex(&bytes))
}

fn git_text(root: &Path, arguments: &[&str]) -> Result<String, String> {
    let output = ProcessService::create_command("git")
        .arg("-C")
        .arg(root)
        .args(arguments)
        .output()
        .map_err(|error| format!("failed to execute read-only git provenance command: {error}"))?;
    if !output.status.success() {
        return Err(format!(
            "read-only git provenance command failed: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        ));
    }
    String::from_utf8(output.stdout)
        .map_err(|error| format!("git provenance output is not UTF-8: {error}"))
}
