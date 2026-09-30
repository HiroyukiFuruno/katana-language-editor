use std::{
    collections::{HashMap, HashSet},
    path::Path,
};

use crate::{
    baseline::AUDIT_DOCUMENT,
    matrix,
    source_inventory_doc::{SourceInventoryDocument, SourceInventoryRow},
    source_inventory_features::SourceInventoryFeatureMap,
    source_inventory_importance::INVENTORY_IMPORTANCE,
    source_inventory_repo::SourceInventoryRepo,
};

pub(crate) struct SourceInventoryAudit;

impl SourceInventoryAudit {
    pub(crate) fn validate() -> Result<(), String> {
        let repo_root = SourceInventoryRepo::resolve_katana_repo()?;
        let matrix_feature_ids = collect_matrix_feature_ids();
        let requirement_ids =
            SourceInventoryDocument::parse_source_inventory_feature_ids(AUDIT_DOCUMENT)?;
        SourceInventoryFeatureMap::validate_feature_id_sets(&matrix_feature_ids, &requirement_ids)?;

        let expected_inventory = SourceInventoryRepo::collect_expected_inventory(&repo_root)?;
        let documented_inventory = SourceInventoryDocument::parse_source_inventory(
            AUDIT_DOCUMENT,
            &repo_root,
            &matrix_feature_ids,
        )?;
        validate_inventory_scope(&expected_inventory, &documented_inventory)?;
        SourceInventoryFeatureMap::validate_feature_coverage(
            &matrix_feature_ids,
            &documented_inventory,
        )?;
        validate_important_function_coverage(&repo_root, &documented_inventory)?;

        Ok(())
    }
}

fn collect_matrix_feature_ids() -> HashSet<String> {
    matrix::FEATURES
        .iter()
        .map(|feature| feature.id.to_string())
        .collect()
}

fn validate_inventory_scope(
    expected_inventory: &HashMap<String, Vec<String>>,
    documented_inventory: &HashMap<String, SourceInventoryRow>,
) -> Result<(), String> {
    for file in expected_inventory.keys() {
        if !documented_inventory.contains_key(file) {
            return Err(format!(
                "missing source inventory row for required file: {file}"
            ));
        }
    }

    for file in documented_inventory.keys() {
        if !expected_inventory.contains_key(file) {
            return Err(format!(
                "source inventory contains unknown file outside required scope: {file}"
            ));
        }
    }

    Ok(())
}

fn validate_important_function_coverage(
    repo_root: &Path,
    documented_inventory: &HashMap<String, SourceInventoryRow>,
) -> Result<(), String> {
    for &(file, required_functions) in INVENTORY_IMPORTANCE {
        let documented_functions = documented_inventory
            .get(file)
            .ok_or_else(|| format!("important source inventory file is not listed: {file}"))?;

        if documented_functions.integration_functions.is_empty() {
            return Err(format!(
                "important source inventory row must declare at least one integration function: {file}"
            ));
        }

        let path = repo_root.join(file);
        let source = SourceInventoryRepo::read_file(&path)?;
        let actual_functions = SourceInventoryRepo::collect_fn_names(&source);
        let documented_functions = &documented_functions.integration_functions;
        validate_documented_functions_exist(file, documented_functions, &actual_functions)?;
        validate_required_function_coverage(file, required_functions, documented_functions)?;
        validate_required_functions_exist(file, required_functions, &actual_functions)?;
    }

    Ok(())
}

fn validate_documented_functions_exist(
    file: &str,
    documented_functions: &[String],
    actual_functions: &[String],
) -> Result<(), String> {
    for documented_fn in documented_functions {
        if !actual_functions.contains(documented_fn) {
            return Err(format!(
                "source inventory function {documented_fn} does not exist in {file}"
            ));
        }
    }
    Ok(())
}

fn validate_required_function_coverage(
    file: &str,
    required_functions: &[&str],
    documented_functions: &[String],
) -> Result<(), String> {
    if required_functions
        .iter()
        .any(|name| documented_functions.iter().any(|actual| actual == name))
    {
        return Ok(());
    }

    Err(format!(
        "source inventory for {file} must include one of: {}",
        required_functions.join(", "),
    ))
}

fn validate_required_functions_exist(
    file: &str,
    required_functions: &[&str],
    actual_functions: &[String],
) -> Result<(), String> {
    for required_fn in required_functions {
        if !actual_functions.iter().any(|actual| actual == required_fn) {
            return Err(format!(
                "required verification function {required_fn} is missing from source: {file}"
            ));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn source_inventory_entrypoint_remains_test_only() {
        let _ = SourceInventoryAudit::validate as fn() -> Result<(), String>;
    }

    #[test]
    fn source_inventory_feature_ids_match_the_26_row_matrix() -> Result<(), String> {
        let matrix_feature_ids = collect_matrix_feature_ids();
        let documented_feature_ids =
            SourceInventoryDocument::parse_source_inventory_feature_ids(AUDIT_DOCUMENT)?;

        assert_eq!(matrix_feature_ids.len(), matrix::REQUIRED_FEATURE_COUNT);
        SourceInventoryFeatureMap::validate_feature_id_sets(
            &matrix_feature_ids,
            &documented_feature_ids,
        )
    }

    #[test]
    fn source_inventory_rejects_a_documented_id_missing_from_the_matrix() -> Result<(), String> {
        let mut matrix_feature_ids = collect_matrix_feature_ids();
        matrix_feature_ids.remove("text-surface");
        let documented_feature_ids =
            SourceInventoryDocument::parse_source_inventory_feature_ids(AUDIT_DOCUMENT)?;

        let error = SourceInventoryFeatureMap::validate_feature_id_sets(
            &matrix_feature_ids,
            &documented_feature_ids,
        )
        .err()
        .ok_or_else(|| "source inventory mismatch was accepted".to_string())?;

        assert!(
            error.contains("source inventory feature-id section contains unknown id text-surface")
        );
        Ok(())
    }

    #[test]
    fn source_inventory_rejects_missing_required_source_in_synthetic_inventory()
    -> Result<(), String> {
        let expected_inventory = HashMap::from([
            ("required.rs".to_string(), Vec::new()),
            ("decisive.rs".to_string(), Vec::new()),
        ]);
        let documented_inventory =
            HashMap::from([("required.rs".to_string(), SourceInventoryRow::default())]);
        let error = validate_inventory_scope(&expected_inventory, &documented_inventory)
            .err()
            .ok_or_else(|| "an undocumented required source was accepted".to_string())?;
        assert!(error.contains("missing source inventory row for required file"));
        Ok(())
    }

    #[test]
    fn source_inventory_rejects_the_nonexistent_adapter_when_documented_as_evidence()
    -> Result<(), String> {
        let expected_inventory = HashMap::from([("actual.rs".to_string(), Vec::new())]);
        let documented_inventory = HashMap::from([
            ("actual.rs".to_string(), SourceInventoryRow::default()),
            (
                "crates/katana-ui/tests/integration/editor/kle_downstream_adapter.rs".to_string(),
                SourceInventoryRow::default(),
            ),
        ]);

        let error = validate_inventory_scope(&expected_inventory, &documented_inventory)
            .err()
            .ok_or_else(|| "an undocumented source was accepted as parity evidence".to_string())?;
        assert!(error.contains("source inventory contains unknown file outside required scope"));
        Ok(())
    }
}
