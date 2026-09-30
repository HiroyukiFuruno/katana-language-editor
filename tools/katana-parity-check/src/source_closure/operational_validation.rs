use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

use super::operational_input::SourceClosureInput;

pub(super) struct InputVerifier<'a> {
    pub(super) evidence_dir: &'a Path,
    pub(super) input_json: &'a Path,
    pub(super) seen_capture_ids: BTreeMap<String, String>,
    pub(super) seen_paths: BTreeMap<String, String>,
    pub(super) capture_run_id: Option<String>,
}

impl<'a> InputVerifier<'a> {
    pub(super) fn new(evidence_dir: &'a Path, input_json: &'a Path) -> Self {
        Self {
            evidence_dir,
            input_json,
            seen_capture_ids: BTreeMap::new(),
            seen_paths: BTreeMap::new(),
            capture_run_id: None,
        }
    }

    pub(super) fn verify(mut self, input: &SourceClosureInput) -> Result<(), String> {
        if input.input_schema_version != super::operational_input::INPUT_SCHEMA_VERSION {
            return Err("input_schema_version must be exactly \"1\"".into());
        }
        self.verify_seeds(input)?;
        self.verify_root(&input.root)?;
        self.verify_profiles(input)?;
        if input.root.release_profile_matrix_fingerprint != input.matrix_fingerprint() {
            return Err("release profile matrix fingerprint mismatch".into());
        }
        Ok(())
    }

    fn verify_seeds(&self, input: &SourceClosureInput) -> Result<(), String> {
        if input.katana_seed_paths.is_empty() {
            return Err("katana_seed_paths must not be empty".into());
        }
        let mut paths = BTreeSet::new();
        for path in &input.katana_seed_paths {
            super::operational_evidence::validate_relative_path(path)?;
            if !paths.insert(path) {
                return Err(format!("duplicate KatanA seed path: {path}"));
            }
        }
        if !input
            .katana_seed_paths
            .windows(2)
            .all(|pair| pair[0] < pair[1])
        {
            return Err("KatanA seed paths must be in canonical order".into());
        }
        Ok(())
    }

    fn verify_profiles(&mut self, input: &SourceClosureInput) -> Result<(), String> {
        let expected = ["macos-latest", "windows-latest", "ubuntu-latest"];
        if input.profile_probes.len() != expected.len() {
            return Err("exactly three profile probes are required".into());
        }
        let mut ids = BTreeSet::new();
        for (probe, expected_id) in input.profile_probes.iter().zip(expected) {
            if probe.id != expected_id || probe.runner_label != expected_id {
                return Err("profile probes must use canonical order and runner labels".into());
            }
            if !ids.insert(probe.id.as_str()) {
                return Err(format!("duplicate profile probe: {}", probe.id));
            }
            self.verify_profile(probe)?;
            if probe.katana_revision != input.root.katana_revision {
                return Err(format!(
                    "{expected_id} profile KatanA revision does not match root provenance"
                ));
            }
        }

        self.verify_profile_tree_alignment(input)?;
        Ok(())
    }

    fn verify_profile_tree_alignment(&self, input: &SourceClosureInput) -> Result<(), String> {
        let expected = input
            .root
            .evidence
            .katana_tree
            .iter()
            .map(|entry| (entry.source_path.as_str(), entry.evidence.sha256.as_str()))
            .collect::<BTreeMap<_, _>>();
        if expected.is_empty() {
            return Err("KatanA tree evidence must not be empty".into());
        }

        for probe in &input.profile_probes {
            let prefix = format!("profiles/{}/raw/source-tree/", probe.id);
            let published_prefix = format!("evidence/{prefix}");
            let mut captured = BTreeMap::new();
            for evidence in &probe.source_tree {
                let Some(source_path) = evidence
                    .path
                    .strip_prefix(&prefix)
                    .or_else(|| evidence.path.strip_prefix(&published_prefix))
                else {
                    return Err(format!(
                        "{} source-tree evidence is not profile-scoped: {}",
                        probe.id, evidence.path
                    ));
                };
                if source_path.is_empty() {
                    return Err(format!(
                        "{} source-tree evidence has an empty source path",
                        probe.id
                    ));
                }
                if captured
                    .insert(source_path, evidence.sha256.as_str())
                    .is_some()
                {
                    return Err(format!(
                        "{} source-tree evidence contains a duplicate source path: {}",
                        probe.id, source_path
                    ));
                }
            }
            if captured != expected {
                return Err(format!(
                    "{} source tree does not match the assembled KatanA provenance",
                    probe.id
                ));
            }
        }
        Ok(())
    }
}
