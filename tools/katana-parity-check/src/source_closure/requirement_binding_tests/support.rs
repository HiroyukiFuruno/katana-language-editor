use crate::source_closure::model::{
    BranchCatalogArtifact, BranchRecord, ManifestRoot, SourceClosureArtifact, SourceClosureFile,
    TextSpan,
};
use crate::source_closure::source_requirement_alias_ledger::AliasEntry;

pub(super) type RootMutation = (&'static str, fn(&mut ManifestRoot));

pub(super) fn root(value: &str) -> ManifestRoot {
    ManifestRoot {
        schema_version: "source-closure.v1".into(),
        katana_revision: value.into(),
        katana_tree_fingerprint: "tree".into(),
        katana_external_ui_fingerprint: "external".into(),
        user_mandated_extensions_fingerprint: "extensions".into(),
        source_universe_fingerprint: "universe".into(),
        requirement_source_aliases_fingerprint: "aliases".into(),
        kle_tree_fingerprint: "kle".into(),
        kuc_tree_fingerprint: "kuc".into(),
        release_profile_matrix_fingerprint: "profiles".into(),
        generator_fingerprint: "generator".into(),
        generated_at_utc: "2026-09-05T00:00:00Z".into(),
        static_leaf_count: None,
        expected_leaf_ids: None,
    }
}

pub(super) fn source(root: &ManifestRoot, paths: &[(&str, &str)]) -> SourceClosureArtifact {
    SourceClosureArtifact {
        root: root.clone(),
        profiles: Vec::new(),
        files: paths
            .iter()
            .map(|(path, sha256)| SourceClosureFile {
                path: (*path).into(),
                sha256: (*sha256).into(),
                incoming_edges: Vec::new(),
                classification: "source".into(),
                classification_rationale: "test".into(),
            })
            .collect(),
        external_ui_semantic_dependencies: Vec::new(),
        unresolved_edges: Vec::new(),
        source_derived_native_target: None,
        context_menu_target_manifest: None,
    }
}

pub(super) fn branch(root: &ManifestRoot, file: &str, id: &str) -> BranchCatalogArtifact {
    BranchCatalogArtifact {
        root: root.clone(),
        branches: vec![BranchRecord {
            branch_id: id.into(),
            file: file.into(),
            symbol: "run".into(),
            span: TextSpan {
                start_line: 2,
                end_line: 2,
            },
            kind: "if".into(),
            condition: "ready".into(),
            active_profile_ids: Vec::new(),
            inactive_profile_predicates: Vec::new(),
            outcomes: Vec::new(),
            incoming_edges: Vec::new(),
            source_excerpt_sha256: "sha256:excerpt".into(),
        }],
        unclassified_branch_ids: vec![id.into()],
    }
}

pub(super) fn entry(requirement_id: &str, path: &str) -> AliasEntry {
    AliasEntry {
        requirement_id: requirement_id.into(),
        short_reference: "editor.rs".into(),
        source_path: path.into(),
    }
}
