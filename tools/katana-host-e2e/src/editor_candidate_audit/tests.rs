use super::{DIGEST_BYTES, EditorCandidateAudit, EditorCandidateAuditError, EditorSourceContract};
use accesskit::{Node, NodeId, Role, Tree, TreeId, TreeUpdate};
use sha2::{Digest, Sha256};

const SPAN: &str = "sha256:0000000000000000000000000000000000000000000000000000000000000000";

fn update(nodes: Vec<(NodeId, Node)>) -> TreeUpdate {
    TreeUpdate {
        tree: Some(Tree::new(
            nodes.first().map(|(id, _)| *id).unwrap_or(NodeId(1)),
        )),
        nodes,
        tree_id: TreeId::ROOT,
        focus: NodeId(2),
    }
}

fn editor(name: Option<&str>) -> Node {
    let mut node = Node::new(Role::MultilineTextInput);
    if let Some(name) = name {
        node.set_label(name.to_owned());
    }
    node
}

fn audit(
    contract: Option<EditorSourceContract>,
) -> Result<EditorCandidateAudit, EditorCandidateAuditError> {
    EditorCandidateAudit::new(contract)
}

#[test]
fn unnamed_candidate_is_rejected_without_selecting_it() {
    let result = audit(Some(EditorSourceContract::multiline_text_input(SPAN)))
        .expect("contract")
        .observe_current_frame(&update(vec![(NodeId(2), editor(None))]), 7);
    assert_eq!(
        result,
        Err(EditorCandidateAuditError::UnsupportedEditorIdentity)
    );
}

#[test]
fn duplicate_roles_are_ambiguous_even_with_distinct_redacted_facts() {
    let result = audit(Some(EditorSourceContract::multiline_text_input(SPAN)))
        .expect("contract")
        .observe_current_frame(
            &update(vec![
                (NodeId(2), editor(Some("first"))),
                (NodeId(3), editor(Some("second"))),
            ]),
            8,
        );
    assert_eq!(result, Err(EditorCandidateAuditError::TargetAmbiguous));
}

#[test]
fn absent_or_mismatched_source_contract_is_rejected() {
    assert!(matches!(
        audit(None),
        Err(EditorCandidateAuditError::UnsupportedEditorIdentity)
    ));
    let mut mismatched = EditorSourceContract::multiline_text_input(SPAN);
    mismatched.katana_revision = "mutable".to_owned();
    assert!(matches!(
        audit(Some(mismatched)),
        Err(EditorCandidateAuditError::UnsupportedEditorIdentity)
    ));
}

#[test]
fn observation_is_redacted_and_records_current_frame_facts() {
    let mut parent = Node::new(Role::ScrollView);
    parent.set_children([NodeId(2)]);
    let result = audit(Some(EditorSourceContract::multiline_text_input(SPAN)))
        .expect("contract")
        .observe_current_frame(
            &update(vec![
                (NodeId(1), parent),
                (NodeId(2), editor(Some("secret"))),
            ]),
            9,
        )
        .expect("unique candidate");
    assert_eq!(result.frame_generation, 9);
    assert_eq!(result.source_span_digest, [0; DIGEST_BYTES]);
    let expected_name_digest: [u8; DIGEST_BYTES] = Sha256::digest(b"secret").into();
    assert_eq!(result.candidate.name_digest, expected_name_digest);
    assert_eq!(result.candidate.ancestor_role_digests.len(), 1);
    assert!(!format!("{result:?}").contains("secret"));
}
