use super::super::super::model::ManifestRoot;
use super::super::GeneratedLeafJoinArtifact;
use super::fixtures;

type RootMutation = (&'static str, fn(&mut ManifestRoot));

const ROOT_MUTATIONS: [RootMutation; 14] = [
    ("schema_version", |root| {
        root.schema_version = "changed".into()
    }),
    ("katana_revision", |root| {
        root.katana_revision = "changed".into()
    }),
    ("katana_tree_fingerprint", |root| {
        root.katana_tree_fingerprint = "changed".into()
    }),
    ("katana_external_ui_fingerprint", |root| {
        root.katana_external_ui_fingerprint = "changed".into()
    }),
    ("user_mandated_extensions_fingerprint", |root| {
        root.user_mandated_extensions_fingerprint = "changed".into()
    }),
    ("source_universe_fingerprint", |root| {
        root.source_universe_fingerprint = "changed".into()
    }),
    ("requirement_source_aliases_fingerprint", |root| {
        root.requirement_source_aliases_fingerprint = "changed".into()
    }),
    ("kle_tree_fingerprint", |root| {
        root.kle_tree_fingerprint = "changed".into()
    }),
    ("kuc_tree_fingerprint", |root| {
        root.kuc_tree_fingerprint = "changed".into()
    }),
    ("release_profile_matrix_fingerprint", |root| {
        root.release_profile_matrix_fingerprint = "changed".into()
    }),
    ("generator_fingerprint", |root| {
        root.generator_fingerprint = "changed".into()
    }),
    ("generated_at_utc", |root| {
        root.generated_at_utc = "changed".into()
    }),
    ("static_leaf_count", |root| root.static_leaf_count = Some(1)),
    ("expected_leaf_ids", |root| {
        root.expected_leaf_ids = Some(vec!["leaf".into()])
    }),
];

#[test]
fn every_manifest_root_field_mismatch_in_branch_is_rejected() {
    for (field, mutate) in ROOT_MUTATIONS {
        let (source, mut branches, actions) = fixtures::inputs();
        mutate(&mut branches.root);
        let result =
            GeneratedLeafJoinArtifact::build_leaf_join_candidates(&source, &branches, &actions);
        assert!(result.is_err(), "branch root field {field} was accepted");
    }
}

#[test]
fn every_manifest_root_field_mismatch_in_action_is_rejected() {
    for (field, mutate) in ROOT_MUTATIONS {
        let (source, branches, mut actions) = fixtures::inputs();
        mutate(&mut actions.root);
        let result =
            GeneratedLeafJoinArtifact::build_leaf_join_candidates(&source, &branches, &actions);
        assert!(result.is_err(), "action root field {field} was accepted");
    }
}

#[test]
fn root_mismatch_is_rejected_when_branch_and_action_collections_are_empty() {
    for (field, mutate) in ROOT_MUTATIONS {
        let (source, mut branches, mut actions) = fixtures::inputs();
        branches.branches.clear();
        actions.actions.clear();
        mutate(&mut branches.root);
        let result =
            GeneratedLeafJoinArtifact::build_leaf_join_candidates(&source, &branches, &actions);
        assert!(
            result.is_err(),
            "empty collections accepted branch root field {field}"
        );

        let (source, mut branches, mut actions) = fixtures::inputs();
        branches.branches.clear();
        actions.actions.clear();
        mutate(&mut actions.root);
        let result =
            GeneratedLeafJoinArtifact::build_leaf_join_candidates(&source, &branches, &actions);
        assert!(
            result.is_err(),
            "empty collections accepted action root field {field}"
        );
    }
}

#[test]
fn duplicate_source_path_is_rejected_before_join() {
    let (mut source, branches, actions) = fixtures::inputs();
    source.files.push(source.files[0].clone());
    let result =
        GeneratedLeafJoinArtifact::build_leaf_join_candidates(&source, &branches, &actions);
    assert!(result.is_err());
}

#[test]
fn empty_source_set_rejects_branch_reference() {
    let (mut source, branches, actions) = fixtures::inputs();
    source.files.clear();
    let result =
        GeneratedLeafJoinArtifact::build_leaf_join_candidates(&source, &branches, &actions);
    assert!(result.is_err());
}

#[test]
fn duplicate_branch_id_is_rejected_before_join() {
    let (source, mut branches, actions) = fixtures::inputs();
    branches.branches.push(branches.branches[0].clone());
    let result =
        GeneratedLeafJoinArtifact::build_leaf_join_candidates(&source, &branches, &actions);
    assert!(result.is_err());
}
