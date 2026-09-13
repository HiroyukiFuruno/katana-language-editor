use super::model::ManifestRoot;
use super::validator_helpers::is_placeholder;

pub(super) fn validate_root_consistency<'a>(
    roots: impl IntoIterator<Item = (&'static str, &'a ManifestRoot)>,
    errors: &mut Vec<String>,
) {
    let list: Vec<(&'static str, &'a ManifestRoot)> = roots.into_iter().collect();
    if list.is_empty() {
        return;
    }
    let reference = match make_root_fingerprint(list[0].1) {
        Ok(fingerprint) => fingerprint,
        Err(error) => {
            errors.push(format!("{}: {error}", list[0].0));
            return;
        }
    };
    for (name, root) in list {
        match make_root_fingerprint(root) {
            Ok(fingerprint) if fingerprint == reference => {}
            Ok(_) => errors.push(format!("{name}: mismatched manifest root metadata")),
            Err(error) => errors.push(format!("{name}: {error}")),
        }
    }
}

pub(super) fn has_static_leaf_claim(root: &ManifestRoot) -> bool {
    root.static_leaf_count.is_some()
        || root
            .expected_leaf_ids
            .as_ref()
            .is_some_and(|ids| !ids.is_empty())
}

pub(super) fn reject_static_leaf_claims<'a>(
    roots: impl IntoIterator<Item = (&'static str, &'a ManifestRoot)>,
    errors: &mut Vec<String>,
) {
    if roots
        .into_iter()
        .any(|(_, root)| has_static_leaf_claim(root))
    {
        errors.push("source-closure root contains static leaf count / expected IDs".to_string());
    }
}

fn make_root_fingerprint(root: &ManifestRoot) -> Result<RootFingerprint, String> {
    validate_schema_version(&root.schema_version)?;
    if root.generated_at_utc.trim().is_empty() {
        return Err("manifest root has missing generated_at_utc".to_string());
    }
    Ok(RootFingerprint {
        schema_version: root.schema_version.clone(),
        katana_revision: root.katana_revision.clone(),
        katana_tree_fingerprint: root.katana_tree_fingerprint.clone(),
        katana_external_ui_fingerprint: root.katana_external_ui_fingerprint.clone(),
        user_mandated_extensions_fingerprint: root.user_mandated_extensions_fingerprint.clone(),
        source_universe_fingerprint: root.source_universe_fingerprint.clone(),
        requirement_source_aliases_fingerprint: root.requirement_source_aliases_fingerprint.clone(),
        kle_tree_fingerprint: root.kle_tree_fingerprint.clone(),
        kuc_tree_fingerprint: root.kuc_tree_fingerprint.clone(),
        release_profile_matrix_fingerprint: root.release_profile_matrix_fingerprint.clone(),
        generator_fingerprint: root.generator_fingerprint.clone(),
    })
}

fn validate_schema_version(value: &str) -> Result<(), String> {
    if value.trim().is_empty() || is_placeholder(value) {
        return Err("manifest root has placeholder schema version".to_string());
    }
    if value != "1" {
        return Err(format!(
            "unsupported canonical manifest schema version: {value}"
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    #[test]
    fn canonical_schema_rejects_diagnostic_and_unknown_versions() {
        assert!(super::validate_schema_version("1").is_ok());
        for version in [
            "requirement-binding-diagnostic-v1",
            "source-closure.v1",
            "2",
            " 1",
        ] {
            assert!(matches!(
                super::validate_schema_version(version),
                Err(error) if error.contains("unsupported canonical manifest schema")
            ));
        }
        for version in ["", " ", "TODO"] {
            assert!(matches!(
                super::validate_schema_version(version),
                Err(error) if error.contains("placeholder schema version")
            ));
        }
    }
}

#[derive(Hash, Eq, PartialEq)]
struct RootFingerprint {
    schema_version: String,
    katana_revision: String,
    katana_tree_fingerprint: String,
    katana_external_ui_fingerprint: String,
    user_mandated_extensions_fingerprint: String,
    source_universe_fingerprint: String,
    requirement_source_aliases_fingerprint: String,
    kle_tree_fingerprint: String,
    kuc_tree_fingerprint: String,
    release_profile_matrix_fingerprint: String,
    generator_fingerprint: String,
}
