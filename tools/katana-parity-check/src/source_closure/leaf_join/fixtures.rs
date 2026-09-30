use super::super::super::model::*;
use super::super::GeneratedLeafJoinArtifact;

const BRANCH_START_LINE: usize = 10;
const BRANCH_END_LINE: usize = 12;

pub(super) fn root() -> ManifestRoot {
    ManifestRoot {
        schema_version: "1".to_string(),
        katana_revision: "katana-rev".to_string(),
        katana_tree_fingerprint: "katana-tree".to_string(),
        katana_external_ui_fingerprint: "external-ui".to_string(),
        user_mandated_extensions_fingerprint: "user-ext".to_string(),
        source_universe_fingerprint: "source-universe".to_string(),
        requirement_source_aliases_fingerprint: "aliases".to_string(),
        kle_tree_fingerprint: "kle-tree".to_string(),
        kuc_tree_fingerprint: "kuc-tree".to_string(),
        release_profile_matrix_fingerprint: "matrix-is-input-only".to_string(),
        generator_fingerprint: "generator".to_string(),
        generated_at_utc: "1970-01-01T00:00:00Z".to_string(),
        static_leaf_count: None,
        expected_leaf_ids: None,
    }
}

pub(super) fn inputs() -> (
    SourceClosureArtifact,
    BranchCatalogArtifact,
    ActionOriginsArtifact,
) {
    let root = root();
    let file = "src/editor.rs".to_string();
    let branch = BranchRecord {
        branch_id: "branch:source-a".to_string(),
        file: file.clone(),
        symbol: "Editor::apply".to_string(),
        span: TextSpan {
            start_line: BRANCH_START_LINE,
            end_line: BRANCH_END_LINE,
        },
        kind: "if".to_string(),
        condition: "has_selection".to_string(),
        active_profile_ids: vec!["ubuntu-latest".to_string()],
        inactive_profile_predicates: vec![],
        outcomes: vec![],
        incoming_edges: vec!["edge:source-a".to_string()],
        source_excerpt_sha256: "sha256:branch".to_string(),
    };
    let action = ActionOrigin {
        action_id: "AppAction::Format".to_string(),
        definition_span: "src/actions.rs:8".to_string(),
        construction_sites: vec!["src/menu.rs:20".to_string()],
        origin_classification: "unresolved".to_string(),
        active_profile_ids: vec!["ubuntu-latest".to_string()],
        origin_input: None,
        forward_route: vec![format!("provisional_forward_route;file={file};arm_span=20")],
        kle_leafs: vec![],
        fingerprint: "sha256:action".to_string(),
        predecessor_actions: None,
        state_span: None,
        no_renderer_or_shortcut_rationale: None,
    };
    (
        SourceClosureArtifact {
            root: root.clone(),
            profiles: vec![],
            files: vec![SourceClosureFile {
                path: file,
                sha256: "sha256:file".to_string(),
                incoming_edges: vec![],
                classification: "editor_source".to_string(),
                classification_rationale: "source fact".to_string(),
            }],
            external_ui_semantic_dependencies: vec![],
            unresolved_edges: vec![],
            source_derived_native_target: None,
            context_menu_target_manifest: None,
        },
        BranchCatalogArtifact {
            root: root.clone(),
            branches: vec![branch],
            unclassified_branch_ids: vec!["branch:source-a".to_string()],
        },
        ActionOriginsArtifact {
            root,
            actions: vec![action],
            unresolved_action_origins: vec![
                "AppAction::Format: unresolved source action".to_string(),
            ],
        },
    )
}

pub(super) fn build() -> Result<GeneratedLeafJoinArtifact, String> {
    let (source, branches, actions) = inputs();
    GeneratedLeafJoinArtifact::build_leaf_join_candidates(&source, &branches, &actions)
}

pub(super) fn canonical_binding()
-> Result<super::super::super::canonical_leaf_binding::CanonicalLeafBindingEvidence, String> {
    let (source, branches, _) = inputs();
    super::super::super::canonical_leaf_binding::CanonicalLeafBindingEvidence::build(
        &source,
        &branches,
        &super::super::super::scan_state::ScanState::default(),
    )
}
