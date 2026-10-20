use std::collections::BTreeSet;
use std::fs;
use std::path::Path;

use crate::capability_manifest::capability_manifest_test_fixtures::{
    FixtureBuilder, FixtureDirectory,
};

use super::*;

fn test_manifest_root() -> ManifestRoot {
    ManifestRoot {
        schema_version: "source-closure.v1".to_string(),
        katana_revision: "revision".to_string(),
        katana_tree_fingerprint: "tree".to_string(),
        katana_external_ui_fingerprint: "external".to_string(),
        user_mandated_extensions_fingerprint: "extensions".to_string(),
        source_universe_fingerprint: "source-universe".to_string(),
        requirement_source_aliases_fingerprint: "aliases".to_string(),
        kle_tree_fingerprint: "kle".to_string(),
        kuc_tree_fingerprint: "kuc".to_string(),
        release_profile_matrix_fingerprint: "profiles".to_string(),
        generator_fingerprint: "generator".to_string(),
        generated_at_utc: "2026-09-05T00:00:00Z".to_string(),
        static_leaf_count: None,
        expected_leaf_ids: None,
    }
}

fn scan_source_fixture(source: &str) -> Result<(scan_state::ScanState, FixtureDirectory), String> {
    let fixture = FixtureBuilder::root()?;
    let root = fixture.path();
    let path = root.join("src/lib.rs");
    let parent = path
        .parent()
        .ok_or_else(|| "fixture path has no parent".to_string())?;
    fs::create_dir_all(parent).map_err(|error| error.to_string())?;
    fs::write(&path, source).map_err(|error| error.to_string())?;
    let root = root.canonicalize().map_err(|error| error.to_string())?;
    let mut state = scan_state::ScanState::default();
    let mut discovered = Vec::new();
    let mut pending = vec![root.join("src/lib.rs")];
    let mut seen = BTreeSet::new();
    while let Some(source_path) = pending.pop() {
        let source_path = source_path
            .canonicalize()
            .map_err(|error| error.to_string())?;
        if !seen.insert(source_path.clone()) {
            continue;
        }
        let relative = source_path
            .strip_prefix(&root)
            .map_err(|error| error.to_string())?
            .to_string_lossy()
            .replace(std::path::MAIN_SEPARATOR, "/");
        let source = fs::read_to_string(&source_path).map_err(|error| error.to_string())?;
        let file = syn::parse_file(&source).map_err(|error| error.to_string())?;
        let before = discovered.len();
        ast_scan::SourceClosureVisitor::new(
            Path::new(&root),
            &source_path,
            &relative,
            &mut state,
            &mut discovered,
        )
        .scan_file(&file);
        pending.extend(discovered[before..].iter().cloned());
    }
    Ok((state, fixture))
}

#[test]
fn qualified_app_action_constructions_remain_unresolved_with_path_variant_and_span()
-> Result<(), String> {
    let source = r#"
        enum AppAction { CopyPathToClipboard(String), Direct }
        enum OtherAction { CopyPathToClipboard(String) }
        use crate::AppAction as Alias;
        fn run() {
            let _ = AppAction::Direct;
            let _ = crate::app_state::AppAction::CopyPathToClipboard(String::new());
            let _ = self::app_state::AppAction::CopyPathToClipboard(String::new());
            let _ = super::app_state::AppAction::CopyPathToClipboard(String::new());
            let _ = OtherAction::CopyPathToClipboard(String::new());
            let _ = Alias::CopyPathToClipboard(String::new());
            crate::app_state::AppAction::CopyPathToClipboard!();
        }
    "#;
    let (state, _fixture) = scan_source_fixture(source)?;

    let qualified = state
        .action_constructions
        .iter()
        .filter(|construction| {
            construction.enum_name == "AppAction"
                && construction.variant.as_deref() == Some("CopyPathToClipboard")
        })
        .collect::<Vec<_>>();
    assert_eq!(qualified.len(), 4);
    for construction in &qualified {
        assert!(construction.span.contains("katana:src/lib.rs:"));
        assert!(
            construction
                .unresolved_reason
                .as_deref()
                .is_some_and(|reason| reason.contains("AppAction::CopyPathToClipboard"))
        );
    }
    for path in [
        "crate::app_state::AppAction::CopyPathToClipboard",
        "self::app_state::AppAction::CopyPathToClipboard",
        "super::app_state::AppAction::CopyPathToClipboard",
    ] {
        assert!(qualified.iter().any(|construction| {
            construction
                .unresolved_reason
                .as_deref()
                .is_some_and(|reason| reason.contains(path))
        }));
    }
    assert!(qualified.iter().any(|construction| {
        construction.style == "macro"
            && construction
                .unresolved_reason
                .as_deref()
                .is_some_and(|reason| {
                    reason.contains("crate::app_state::AppAction::CopyPathToClipboard")
                        && reason.contains("macro construction cannot be proven")
                })
    }));
    assert!(state.action_constructions.iter().any(|construction| {
        construction.enum_name == "Alias"
            && construction.variant.as_deref() == Some("CopyPathToClipboard")
            && construction.unresolved_reason.is_some()
    }));
    assert!(!state.action_constructions.iter().any(|construction| {
        construction.enum_name == "OtherAction"
            && construction.variant.as_deref() == Some("CopyPathToClipboard")
    }));

    let artifact = crate::source_closure::action_origins::materialize_action_origins(
        test_manifest_root(),
        &state,
    );
    let direct = artifact
        .actions
        .iter()
        .find(|action| action.action_id == "AppAction::Direct")
        .ok_or_else(|| "direct AppAction construction".to_string())?;
    assert_eq!(direct.construction_sites.len(), 1);
    assert!(direct.construction_sites[0].contains("style=path"));
    let copy_path = artifact
        .actions
        .iter()
        .find(|action| action.action_id == "AppAction::CopyPathToClipboard")
        .ok_or_else(|| "CopyPathToClipboard definition".to_string())?;
    assert!(copy_path.construction_sites.is_empty());
    assert!(artifact.unresolved_action_origins.iter().any(|entry| {
        entry.contains("qualified AppAction construction path `crate::app_state::AppAction::CopyPathToClipboard`")
    }));
    assert!(artifact.unresolved_action_origins.iter().any(|entry| {
        entry.contains("imported AppAction-like construction path `Alias::CopyPathToClipboard`")
    }));
    Ok(())
}
