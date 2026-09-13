use super::fingerprint::sha256_hex;
use super::operational_input::{
    EvidenceRef, FIXED_KATANA_REVISION, KatanaTreeEvidence, RootProvenance,
};
use super::operational_repository_validation::validate_repository_metadata;
use super::operational_validation::InputVerifier;

impl<'a> InputVerifier<'a> {
    pub(super) fn verify_root(&mut self, root: &RootProvenance) -> Result<(), String> {
        if root.schema_version != super::operational_input::INPUT_SCHEMA_VERSION {
            return Err("root schema_version must be exactly \"1\"".into());
        }
        if root.katana_revision != FIXED_KATANA_REVISION {
            return Err("KatanA revision does not match the fixed release revision".into());
        }
        validate_repository_metadata(root)?;
        super::operational_evidence::validate_timestamp(&root.generated_at_utc)?;
        for (value, name) in [
            (&root.katana_tree_fingerprint, "katana_tree_fingerprint"),
            (
                &root.katana_external_ui_fingerprint,
                "katana_external_ui_fingerprint",
            ),
            (
                &root.user_mandated_extensions_fingerprint,
                "user_mandated_extensions_fingerprint",
            ),
            (
                &root.source_universe_fingerprint,
                "source_universe_fingerprint",
            ),
            (
                &root.requirement_source_aliases_fingerprint,
                "requirement_source_aliases_fingerprint",
            ),
            (&root.kle_tree_fingerprint, "kle_tree_fingerprint"),
            (&root.kuc_tree_fingerprint, "kuc_tree_fingerprint"),
            (
                &root.release_profile_matrix_fingerprint,
                "release_profile_matrix_fingerprint",
            ),
            (&root.generator_fingerprint, "generator_fingerprint"),
        ] {
            super::operational_evidence::validate_hash(value, name)?;
        }

        let revision =
            self.read_evidence(&root.evidence.katana_revision, "read-only-local-source")?;
        if String::from_utf8_lossy(&revision).trim() != FIXED_KATANA_REVISION {
            return Err("KatanA revision evidence does not match the fixed revision".into());
        }
        self.verify_repository_provenance(
            &root.kle_revision,
            root.kle_worktree_clean,
            &root.evidence.kle_revision,
            &root.evidence.kle_worktree_status,
            "KLE",
        )?;
        self.verify_repository_provenance(
            &root.kuc_revision,
            root.kuc_worktree_clean,
            &root.evidence.kuc_revision,
            &root.evidence.kuc_worktree_status,
            "KUC",
        )?;
        let user = self.read_evidence(
            &root.evidence.user_mandated_extensions,
            "read-only-local-source",
        )?;
        if sha256_hex(&user) != root.user_mandated_extensions_fingerprint {
            return Err("user-mandated extension fingerprint mismatch".into());
        }
        super::user_mandated_extensions::validate(&user)?;
        let source_universe =
            self.read_evidence(&root.evidence.source_universe, "read-only-local-source")?;
        if sha256_hex(&source_universe) != root.source_universe_fingerprint {
            return Err("source-universe fingerprint mismatch".into());
        }
        super::source_roots::validate_source_universe_document(&source_universe)?;
        let aliases = self.read_evidence(
            &root.evidence.requirement_source_aliases,
            "read-only-local-source",
        )?;
        if sha256_hex(&aliases) != root.requirement_source_aliases_fingerprint {
            return Err("requirement source alias ledger fingerprint mismatch".into());
        }
        super::source_requirement_alias_ledger::RequirementSourceAliasLedger::validate_captured_bytes(
            &aliases,
        )?;
        self.verify_katana_tree(
            &root.evidence.katana_tree,
            &root.katana_tree_fingerprint,
            "katana tree",
        )?;
        self.verify_tree(
            &root.evidence.katana_external_ui,
            &root.katana_external_ui_fingerprint,
            "KatanA external UI",
        )?;
        self.verify_tree(
            &root.evidence.kle_tree,
            &root.kle_tree_fingerprint,
            "KLE tree",
        )?;
        self.verify_tree(
            &root.evidence.kuc_tree,
            &root.kuc_tree_fingerprint,
            "KUC tree",
        )?;
        self.verify_tree(
            &[
                root.evidence.generator_binary.clone(),
                root.evidence.generator_schema.clone(),
            ],
            &root.generator_fingerprint,
            "generator binary/schema",
        )?;
        if root.evidence.release_profile_matrix.len()
            != super::root_model::CANONICAL_PROFILE_IDS.len()
        {
            return Err("release profile matrix evidence must contain three records".into());
        }
        for (expected, evidence) in ["macos-latest", "windows-latest", "ubuntu-latest"]
            .into_iter()
            .zip(&root.evidence.release_profile_matrix)
        {
            let bytes = self.read_evidence(evidence, expected)?;
            if !String::from_utf8_lossy(&bytes).contains(expected) {
                return Err(format!("release profile matrix evidence omits {expected}"));
            }
        }
        Ok(())
    }

    fn verify_tree(
        &mut self,
        evidence: &[EvidenceRef],
        expected: &str,
        label: &str,
    ) -> Result<(), String> {
        if evidence.is_empty() {
            return Err(format!("{label} evidence must not be empty"));
        }
        for entry in evidence {
            self.read_evidence(entry, "read-only-local-source")?;
        }
        if !evidence.windows(2).all(|pair| pair[0].path < pair[1].path) {
            return Err(format!("{label} evidence is not in canonical order"));
        }
        if super::operational_input::EvidenceRef::set_fingerprint(evidence) != expected {
            return Err(format!("{label} fingerprint mismatch"));
        }
        Ok(())
    }

    fn verify_katana_tree(
        &mut self,
        evidence: &[KatanaTreeEvidence],
        expected: &str,
        label: &str,
    ) -> Result<(), String> {
        if evidence.is_empty() {
            return Err(format!("{label} evidence must not be empty"));
        }
        let mut source_paths = std::collections::BTreeSet::new();
        for entry in evidence {
            super::operational_evidence::validate_rust_source_path(&entry.source_path)?;
            if !source_paths.insert(&entry.source_path) {
                return Err(format!(
                    "duplicate KatanA tree source path: {}",
                    entry.source_path
                ));
            }
            self.read_evidence(&entry.evidence, "read-only-local-source")?;
        }
        if !evidence
            .windows(2)
            .all(|pair| pair[0].source_path < pair[1].source_path)
        {
            return Err(format!("{label} evidence is not in canonical order"));
        }
        if KatanaTreeEvidence::set_fingerprint(evidence) != expected {
            return Err(format!("{label} fingerprint mismatch"));
        }
        Ok(())
    }
}
