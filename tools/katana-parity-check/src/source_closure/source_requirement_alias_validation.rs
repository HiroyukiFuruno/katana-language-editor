use std::collections::BTreeSet;

use super::operational_input::FIXED_KATANA_REVISION;
use super::source_requirement_alias_ledger::{
    AliasLedger, RequirementSourceAliasLedger, SourceRootManifest,
};

const SOURCE_PREFIX: &str = "crates/katana-ui/src/";
const RUST_SUFFIX: &str = ".rs";

impl RequirementSourceAliasLedger {
    pub(super) fn validate_coverage(
        requirements: &str,
        source_universe: &str,
        root_manifest: &str,
        alias_ledger: &str,
    ) -> Result<(), String> {
        Self::validated_entries(requirements, source_universe, root_manifest, alias_ledger)
            .map(|_| ())
    }

    pub(super) fn validated_entries(
        requirements: &str,
        source_universe: &str,
        root_manifest: &str,
        alias_ledger: &str,
    ) -> Result<Vec<super::source_requirement_alias_ledger::AliasEntry>, String> {
        let mut ledger: AliasLedger = serde_json::from_str(alias_ledger)
            .map_err(|error| format!("invalid requirement source alias ledger: {error}"))?;
        if ledger.schema_version != "1" || ledger.katana_revision != FIXED_KATANA_REVISION {
            return Err(
                "requirement source alias ledger does not use the fixed schema/revision".into(),
            );
        }
        let manifest: SourceRootManifest = serde_json::from_str(root_manifest)
            .map_err(|error| format!("invalid source-root manifest for alias ledger: {error}"))?;
        let required = Self::short_references(requirements)?;
        let classified = Self::classify(&ledger, source_universe, &manifest)?;
        Self::validate_classification_set(&required, &classified)?;
        ledger.entries.sort_by(|left, right| {
            (
                &left.requirement_id,
                &left.short_reference,
                &left.source_path,
            )
                .cmp(&(
                    &right.requirement_id,
                    &right.short_reference,
                    &right.source_path,
                ))
        });
        Ok(ledger.entries)
    }

    fn classify(
        ledger: &AliasLedger,
        source_universe: &str,
        manifest: &SourceRootManifest,
    ) -> Result<BTreeSet<(String, String)>, String> {
        let mut classified = BTreeSet::new();
        for entry in &ledger.entries {
            let key = (entry.requirement_id.clone(), entry.short_reference.clone());
            if !classified.insert(key.clone()) {
                return Err(format!(
                    "duplicate alias classification: {}:{}",
                    key.0, key.1
                ));
            }
            Self::validate_source_path(&entry.source_path, source_universe, manifest)?;
        }
        for entry in &ledger.non_katana_references {
            let key = (entry.requirement_id.clone(), entry.short_reference.clone());
            if !classified.insert(key.clone()) {
                return Err(format!(
                    "duplicate alias classification: {}:{}",
                    key.0, key.1
                ));
            }
            if entry.owner.trim().is_empty()
                || entry.reason.trim().is_empty()
                || entry.short_reference.starts_with(SOURCE_PREFIX)
            {
                return Err(format!(
                    "invalid external source classification: {}:{}",
                    key.0, key.1
                ));
            }
        }
        Ok(classified)
    }

    fn validate_source_path(
        source_path: &str,
        source_universe: &str,
        manifest: &SourceRootManifest,
    ) -> Result<(), String> {
        if !source_path.starts_with(SOURCE_PREFIX)
            || !source_path.ends_with(RUST_SUFFIX)
            || source_path.contains("..")
            || source_path.contains('\\')
        {
            return Err(format!(
                "alias source path is not a normalized KatanA UI source: {source_path}"
            ));
        }
        let in_universe = source_universe.contains(source_path)
            || manifest.directory_roots.iter().any(|root| {
                source_path.starts_with(&format!("{root}/"))
                    && source_universe.contains(&format!("{root}/**"))
            });
        if !in_universe {
            return Err(format!(
                "alias source path is absent from source-universe: {source_path}"
            ));
        }
        if !manifest.file_roots.iter().any(|root| root == source_path)
            && !manifest
                .directory_roots
                .iter()
                .any(|root| source_path.starts_with(&format!("{root}/")))
        {
            return Err(format!(
                "alias source path is absent from source-root manifest: {source_path}"
            ));
        }
        Ok(())
    }

    fn validate_classification_set(
        required: &BTreeSet<(String, String)>,
        classified: &BTreeSet<(String, String)>,
    ) -> Result<(), String> {
        if required == classified {
            return Ok(());
        }
        let missing = required.difference(classified).next();
        let unexpected = classified.difference(required).next();
        Err(match (missing, unexpected) {
            (Some((id, reference)), _) => {
                format!("unclassified requirement source: {id}:{reference}")
            }
            (_, Some((id, reference))) => {
                format!("unused requirement source classification: {id}:{reference}")
            }
            _ => "requirement source alias classifications do not match requirements".into(),
        })
    }
}
