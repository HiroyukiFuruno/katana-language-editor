use std::path::Path;

use serde_json::to_writer_pretty;

use super::super::artifact_paths::ArtifactPaths;
use super::super::model::*;

pub(super) fn write_file<T: serde::Serialize>(path: &Path, artifact: &T) -> Result<(), String> {
    let parent = path
        .parent()
        .ok_or_else(|| "artifact path has no parent".to_string())?;
    std::fs::create_dir_all(parent)
        .map_err(|error| format!("failed to create temp dir {}: {error}", parent.display()))?;
    let file = std::fs::File::create(path)
        .map_err(|error| format!("failed to create temp artifact {}: {error}", path.display()))?;
    to_writer_pretty(file, artifact)
        .map_err(|error| format!("failed to serialize {}: {error}", path.display()))
}

fn sample_root() -> ManifestRoot {
    ManifestRoot {
        schema_version: "1".to_string(),
        katana_revision: "rev".to_string(),
        katana_tree_fingerprint: "katana-tree".to_string(),
        katana_external_ui_fingerprint: "external-ui".to_string(),
        user_mandated_extensions_fingerprint: "user-ext".to_string(),
        source_universe_fingerprint: "source-universe".to_string(),
        requirement_source_aliases_fingerprint: "aliases".to_string(),
        kle_tree_fingerprint: "kle-tree".to_string(),
        kuc_tree_fingerprint: "kuc-tree".to_string(),
        release_profile_matrix_fingerprint: "matrix".to_string(),
        generator_fingerprint: "generator".to_string(),
        generated_at_utc: "1970-01-01T00:00:00Z".to_string(),
        static_leaf_count: None,
        expected_leaf_ids: None,
    }
}

fn sample_profile(id: &str) -> ProfileRecord {
    ProfileRecord {
        id: id.to_string(),
        runner_label: id.to_string(),
        katana_revision: crate::source_closure::operational_input::FIXED_KATANA_REVISION
            .to_string(),
        fingerprint: format!("{id}-fp"),
        rustc_host_triple: "x86_64-unknown-linux-gnu".to_string(),
        rustc_cfg_sha256: format!("{id}-cfg"),
        cargo_resolution_sha256: format!("{id}-metadata"),
        lockfile_sha256: format!("{id}-lock"),
        active_edge_ids: vec![],
        inactive_cfg_edges: vec![],
    }
}

pub(super) fn complete_artifacts(dir: &Path) -> Result<ArtifactPaths, String> {
    let leaf_id = "menu.editor.format_bold".to_string();
    let root = sample_root();
    let source_root = SourceClosureArtifact {
        root: root.clone(),
        profiles: CANONICAL_PROFILE_IDS
            .iter()
            .map(|id| sample_profile(id))
            .collect(),
        files: vec![SourceClosureFile {
            path: "crates/katana-ui/src/views/panels/editor/text_edit.rs".to_string(),
            sha256: "sha256:text".to_string(),
            incoming_edges: vec![],
            classification: "editor_direct_ui".to_string(),
            classification_rationale: "direct editor source".to_string(),
        }],
        external_ui_semantic_dependencies: vec![],
        unresolved_edges: vec![],
        source_derived_native_target: None,
        context_menu_target_manifest: None,
    };
    let action_root = ActionOriginsArtifact {
        root: root.clone(),
        actions: vec![ActionOrigin {
            action_id: "AppAction::FormatBold".to_string(),
            definition_span: "src/lib.rs:10".to_string(),
            construction_sites: vec!["src/lib.rs:11".to_string()],
            origin_classification: "editor_direct_ui".to_string(),
            active_profile_ids: CANONICAL_PROFILE_IDS
                .iter()
                .map(|id| (*id).to_string())
                .collect(),
            origin_input: Some(ActionOriginInput {
                kind: "pointer".to_string(),
                source_span: "src/lib.rs:11".to_string(),
            }),
            forward_route: vec!["menu.editor.format_bold".to_string()],
            kle_leafs: vec![leaf_id.clone()],
            fingerprint: "sha256:action".to_string(),
            predecessor_actions: None,
            state_span: None,
            no_renderer_or_shortcut_rationale: None,
        }],
        unresolved_action_origins: vec![],
    };
    let branch_root = BranchCatalogArtifact {
        root: root.clone(),
        branches: vec![BranchRecord {
            branch_id: "branch-format-bold".to_string(),
            file: "crates/katana-ui/src/views/panels/editor/text_edit.rs".to_string(),
            symbol: "menu::format_bold".to_string(),
            span: TextSpan {
                start_line: 10,
                end_line: 20,
            },
            kind: "if".to_string(),
            condition: "has_selection".to_string(),
            active_profile_ids: CANONICAL_PROFILE_IDS
                .iter()
                .map(|id| (*id).to_string())
                .collect(),
            inactive_profile_predicates: vec![],
            outcomes: vec![format!("leaf:{leaf_id}")],
            incoming_edges: vec![],
            source_excerpt_sha256: "sha256:excerpt".to_string(),
        }],
        unclassified_branch_ids: vec![],
    };
    let canonical_binding =
        super::super::canonical_leaf_binding::CanonicalLeafBindingEvidence::build(
            &source_root,
            &branch_root,
            &super::super::scan_state::ScanState::default(),
        )?;
    let leaf_root = LeafManifestArtifact {
        root: root.clone(),
        canonical_binding,
        leafs: vec![LeafStatusRecord {
            leaf_id: leaf_id.clone(),
            source_candidate_fingerprint: "legacy-fixture-not-canonical".to_string(),
            requirement_origin: RequirementOrigin {
                kind: "katana_source".to_string(),
                span: "src/lib.rs:11".to_string(),
            },
            source_branches: vec!["branch-format-bold".to_string()],
            action_origin_ids: vec!["AppAction::FormatBold".to_string()],
            required_profile_ids: CANONICAL_PROFILE_IDS
                .iter()
                .map(|id| (*id).to_string())
                .collect(),
            visible_path: vec![
                "menu".to_string(),
                "editor".to_string(),
                "format_bold".to_string(),
            ],
            state_preconditions: vec!["is_text".to_string()],
            kuc_component: "Egui".to_string(),
            kle_public_show_signature: "public_show_leaf".to_string(),
            kle_opaque_transit: "transit".to_string(),
            declared_effect_kind: "in_process_host_effect".to_string(),
            host_e2e: KatanaHostE2eRequirement {
                state: KatanaHostE2eState::Passing,
                adoption_issue_url: KATANA_ADOPTION_ISSUE_URL.to_string(),
            },
            execution_id: Some("exec-format-bold".to_string()),
            storybook_stage_id: "story-format-bold".to_string(),
            status: "passing".to_string(),
        }],
    };
    let execution_root = ExecutionRecordArtifact {
        root: root.clone(),
        executions: CANONICAL_PROFILE_IDS
            .iter()
            .enumerate()
            .map(|(index, profile_id)| ExecutionRecord {
                execution_id: if index == 0 {
                    "exec-format-bold".to_string()
                } else {
                    format!("exec-format-bold-{profile_id}")
                },
                execution_profile_id: (*profile_id).to_string(),
                leaf_id: leaf_id.clone(),
                action_route_signature: "AppAction::FormatBold".to_string(),
                status: "passing".to_string(),
            })
            .collect(),
        stale_or_missing_leafs: vec![],
    };
    let story_root =
        super::validator_test_storybook::complete_storybook_artifacts(dir, root, &leaf_id)?;

    let paths = ArtifactPaths::from_dir(dir);
    write_file(&paths.source_closure, &source_root)?;
    write_file(&paths.action_origins, &action_root)?;
    write_file(&paths.branch_catalog, &branch_root)?;
    write_file(&paths.leaf_manifest, &leaf_root)?;
    write_file(&paths.execution_record, &execution_root)?;
    write_file(&paths.storybook_artifacts, &story_root)?;
    Ok(paths)
}
