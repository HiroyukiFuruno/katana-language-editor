use std::collections::{HashMap, HashSet};

use crate::source_inventory_doc::SourceInventoryRow;

pub(crate) struct SourceInventoryFeatureMap;

impl SourceInventoryFeatureMap {
    pub(crate) fn validate_feature_id_sets(
        matrix_feature_ids: &HashSet<String>,
        documented_feature_ids: &HashSet<String>,
    ) -> Result<(), String> {
        for id in matrix_feature_ids {
            if !documented_feature_ids.contains(id) {
                return Err(format!(
                    "source inventory feature-id section is missing matrix feature id {id}"
                ));
            }
        }

        for id in documented_feature_ids {
            if !matrix_feature_ids.contains(id) {
                return Err(format!(
                    "source inventory feature-id section contains unknown id {id}"
                ));
            }
        }

        Ok(())
    }

    pub(crate) fn validate_feature_coverage(
        matrix_feature_ids: &HashSet<String>,
        documented_inventory: &HashMap<String, SourceInventoryRow>,
    ) -> Result<(), String> {
        let mut coverage = HashSet::new();
        for row in documented_inventory.values() {
            for feature_id in &row.feature_ids {
                coverage.insert(feature_id.clone());
            }
        }

        for id in matrix_feature_ids {
            if !coverage.contains(id) {
                return Err(format!(
                    "feature id {id} from matrix is not mapped in source inventory rows"
                ));
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validate_feature_id_sets_detects_unknown_ids() {
        let matrix_ids: HashSet<String> = ["text-editing"].into_iter().map(String::from).collect();
        let documented = HashSet::from(["text-editing".to_string(), "unknown-id".to_string()]);

        assert!(matches!(
            SourceInventoryFeatureMap::validate_feature_id_sets(&matrix_ids, &documented).as_ref(),
            Err(error) if error.contains("source inventory feature-id section contains unknown id unknown-id")
        ));
    }

    #[test]
    fn validate_feature_id_sets_detects_missing_matrix_ids() {
        let matrix_ids: HashSet<String> = ["text-editing", "document-search"]
            .into_iter()
            .map(String::from)
            .collect();
        let documented = HashSet::from(["text-editing".to_string()]);
        assert!(matches!(
            SourceInventoryFeatureMap::validate_feature_id_sets(&matrix_ids, &documented).as_ref(),
            Err(error) if error.contains("source inventory feature-id section is missing matrix feature id document-search")
        ));
    }

    #[test]
    fn validate_feature_coverage_requires_all_matrix_ids() {
        let matrix_ids: HashSet<String> = ["text-editing", "document-search"]
            .into_iter()
            .map(String::from)
            .collect();
        let documented_inventory = HashMap::from([(
            "a.rs".to_string(),
            SourceInventoryRow {
                integration_functions: vec!["test_a".to_string()],
                feature_ids: vec!["text-editing".to_string()],
            },
        )]);

        assert!(matches!(
            SourceInventoryFeatureMap::validate_feature_coverage(&matrix_ids, &documented_inventory)
                .as_ref(),
            Err(error) if error.contains("feature id document-search from matrix is not mapped in source inventory rows")
        ));
    }
}
