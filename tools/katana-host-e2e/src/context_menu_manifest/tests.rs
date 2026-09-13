use super::*;
use std::fs;
use std::path::PathBuf;

const D: &str = "sha256:0000000000000000000000000000000000000000000000000000000000000000";

#[test]
fn route_set_is_exact() {
    assert!(matches_routes(&[
        Route::SecondaryPointer,
        Route::ShiftF10,
        Route::AccesskitInvoke
    ]));
    assert!(!matches_routes(&[Route::SecondaryPointer, Route::ShiftF10]));
}

#[test]
fn unknown_fields_are_rejected() {
    let json = r#"{"schema_version":"1","generated_by":"x","katana_revision":"x","profile_fingerprint":"x","source_closure_fingerprint":"x","surface":{},"menu":{},"leaves":[],"routes":[],"leak":"x"}"#;
    assert!(serde_json::from_str::<RawManifest>(json).is_err());
}

#[test]
fn public_errors_are_redacted() {
    for error in [
        ContextMenuManifestError::Unavailable,
        ContextMenuManifestError::Malformed,
        ContextMenuManifestError::Invalid,
    ] {
        let rendered = error.to_string();
        assert!(!rendered.contains("/"));
        assert!(!rendered.contains("Save"));
        assert!(!rendered.contains("AppAction"));
    }
}

#[test]
fn digest_format_is_strict() {
    assert!(digest(D).starts_with("sha256:"));
    assert!(!D.is_empty());
}

#[test]
fn missing_manifest_fails_closed_without_path_leak() {
    let result = ContextMenuManifestLoader::load(
        PathBuf::from("/secret/missing.json"),
        PathBuf::from("/secret/profile.json"),
        PathBuf::from("/secret/source"),
    );
    assert_eq!(result, Err(ContextMenuManifestError::Unavailable));
    assert!(!result.unwrap_err().to_string().contains("secret"));
}

#[test]
fn profile_unknown_fields_are_rejected_by_schema() {
    let dir = std::env::temp_dir().join(format!("kle-context-manifest-{}", std::process::id()));
    let _ = fs::create_dir_all(&dir);
    fs::write(dir.join("manifest.json"), b"{}").expect("manifest");
    fs::write(dir.join("profile.json"), br#"{"root":{"katana_tree_fingerprint":"x","release_profile_matrix_fingerprint":"x","unknown":true}}"#).expect("profile");
    let result =
        ContextMenuManifestLoader::load(dir.join("manifest.json"), dir.join("profile.json"), &dir);
    assert_eq!(result, Err(ContextMenuManifestError::Malformed));
    let _ = fs::remove_dir_all(dir);
}

fn raw_manifest() -> RawManifest {
    RawManifest {
        schema_version: SCHEMA_VERSION.to_owned(),
        generated_by: GENERATED_BY.to_owned(),
        katana_revision: FIXED_REVISION.to_owned(),
        profile_fingerprint: digest("profile"),
        source_closure_fingerprint: digest("source"),
        surface: Surface {
            source_span_digest: D.to_owned(),
            role: "MultilineTextInput".to_owned(),
        },
        menu: Menu {
            source_span_digest: D.to_owned(),
            role: "Menu".to_owned(),
            path_digest: D.to_owned(),
        },
        leaves: Vec::new(),
        routes: vec![
            Route::SecondaryPointer,
            Route::ShiftF10,
            Route::AccesskitInvoke,
        ],
    }
}

#[test]
fn root_revision_and_profile_mismatch_fail_closed() {
    let mut raw = raw_manifest();
    let profile = ProfileRoot {
        root: serde_json::json!({
            "katana_tree_fingerprint": raw.source_closure_fingerprint,
            "release_profile_matrix_fingerprint": raw.profile_fingerprint,
        }),
    };
    raw.katana_revision = "wrong-revision".to_owned();
    assert_eq!(
        validate_root(&raw, &profile),
        Err(ContextMenuManifestError::Invalid)
    );
    raw.katana_revision = FIXED_REVISION.to_owned();
    raw.profile_fingerprint = digest("wrong-profile");
    assert_eq!(
        validate_root(&raw, &profile),
        Err(ContextMenuManifestError::Invalid)
    );
}

#[test]
fn profile_without_required_fingerprint_fails_closed() {
    let profile = ProfileRoot {
        root: serde_json::json!({}),
    };
    assert_eq!(
        validate_root(&raw_manifest(), &profile),
        Err(ContextMenuManifestError::Invalid)
    );
}

#[test]
fn source_operation_mutations_are_not_accepted_as_the_fixed_inventory() {
    let root = crate::FixedSourceHarnessBuilder::required_katana_repo()
        .expect("KATANA_REPO must be explicitly set");
    let source = fs::read_to_string(root.join(CONTEXT_SOURCE)).expect("context source");
    let removed = source.replacen("MarkdownAuthoringOp::InsertTable", "", 1);
    assert_eq!(
        direct_authoring_variants(&removed),
        Err(ContextMenuManifestError::Invalid)
    );
    let duplicated = source.replacen(
        "MarkdownAuthoringOp::InsertTable",
        "MarkdownAuthoringOp::Bold",
        1,
    );
    assert_eq!(
        direct_authoring_variants(&duplicated),
        Err(ContextMenuManifestError::Invalid)
    );
}

#[test]
fn inventory_omission_and_duplicate_fail_closed() {
    let root = crate::FixedSourceHarnessBuilder::required_katana_repo()
        .expect("KATANA_REPO must be explicitly set");
    let code = fs::read_to_string(root.join(CODE_SOURCE)).expect("code source");
    let removed = code.replacen("Self::Sql", "", 1);
    assert_eq!(
        code_block_kinds(&removed),
        Err(ContextMenuManifestError::Invalid)
    );
    let duplicated = code.replacen("Self::Sql", "Self::Text", 1);
    assert_eq!(
        code_block_kinds(&duplicated),
        Err(ContextMenuManifestError::Invalid)
    );
}
