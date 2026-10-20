use super::{HostTargetLocator, HostTargetSelectionError};
use crate::context_menu_manifest::ContextMenuCaseDescriptor;
use accesskit::{Node, NodeId, Rect, Role, Tree, TreeId, TreeUpdate};
use sha2::Digest;

const TARGET_NODE_ID: u64 = 3;
const SECONDARY_NODE_ID: u64 = 4;
const ROOT_NODE_ID: u64 = 1;
const TARGET_BOUNDS_X1: f64 = 11.0;
const TARGET_BOUNDS_Y1: f64 = 22.0;
const SMALL_BOUNDS_X1: f64 = 3.0;
const SMALL_BOUNDS_Y1: f64 = 3.0;

fn update(nodes: Vec<(NodeId, Node)>) -> TreeUpdate {
    let root = nodes
        .first()
        .map(|(id, _)| *id)
        .unwrap_or(NodeId(ROOT_NODE_ID));
    TreeUpdate {
        nodes,
        tree: Some(Tree::new(root)),
        tree_id: TreeId::ROOT,
        focus: root,
    }
}

fn digest(name: &str) -> String {
    let hex: String = sha2::Sha256::digest(name.as_bytes())
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect();
    format!("sha256:{hex}")
}

fn descriptor(names: &[&str]) -> ContextMenuCaseDescriptor {
    ContextMenuCaseDescriptor {
        path_digest: digest("path"),
        parent_path_digest: digest("parent"),
        source_span_digest: digest("span"),
        enabled_condition_digest: digest("condition"),
        locale_label_digests: names.iter().map(|name| digest(name)).collect(),
    }
}

fn button(name: Option<&str>) -> Node {
    let mut node = Node::new(Role::Button);
    if let Some(name) = name {
        node.set_label(name.to_owned());
    }
    node.set_bounds(Rect {
        x0: 1.0,
        y0: 2.0,
        x1: TARGET_BOUNDS_X1,
        y1: TARGET_BOUNDS_Y1,
    });
    node
}

fn menu_tree(target: Node, extra: Vec<(NodeId, Node)>) -> TreeUpdate {
    let mut menu = Node::new(Role::Menu);
    menu.set_children([NodeId(TARGET_NODE_ID)]);
    let mut nodes = vec![(NodeId(1), menu), (NodeId(TARGET_NODE_ID), target)];
    nodes.extend(extra);
    update(nodes)
}

fn locator(names: &[&str]) -> HostTargetLocator {
    HostTargetLocator::from_context_menu_case(&descriptor(names)).expect("valid descriptor")
}

#[test]
fn selects_unique_enabled_bounded_button_under_menu_and_binds_frame() {
    let target = locator(&["Run"]).select_current(&menu_tree(button(Some("Run")), vec![]), [7; 32]);
    let target = target.expect("unique target");
    assert_eq!(target.frame_hash, [7; 32]);
    assert_eq!(target.node_id, NodeId(TARGET_NODE_ID));
}

#[test]
fn accepts_any_opaque_locale_digest() {
    assert!(locator(&["Run", "実行"])
        .select_current(&menu_tree(button(Some("実行")), vec![]), [1; 32])
        .is_ok());
}

#[test]
fn duplicate_label_outside_menu_does_not_ambiguate() {
    let mut outside = button(Some("Run"));
    outside.set_bounds(Rect {
        x0: 2.0,
        y0: 2.0,
        x1: SMALL_BOUNDS_X1,
        y1: SMALL_BOUNDS_Y1,
    });
    assert!(locator(&["Run"])
        .select_current(
            &menu_tree(button(Some("Run")), vec![(NodeId(4), outside)]),
            [1; 32]
        )
        .is_ok());
}

#[test]
fn rejects_missing_target() {
    assert_eq!(
        locator(&["Run"]).select_current(&update(vec![]), [1; 32]),
        Err(HostTargetSelectionError::MissingTarget)
    );
}

#[test]
fn rejects_ambiguous_targets_under_menu() {
    let mut menu = Node::new(Role::Menu);
    menu.set_children([NodeId(TARGET_NODE_ID), NodeId(SECONDARY_NODE_ID)]);
    assert_eq!(
        locator(&["Run"]).select_current(
            &update(vec![
                (NodeId(1), menu),
                (NodeId(TARGET_NODE_ID), button(Some("Run"))),
                (NodeId(SECONDARY_NODE_ID), button(Some("Run")))
            ]),
            [1; 32]
        ),
        Err(HostTargetSelectionError::AmbiguousTarget)
    );
}

#[test]
fn rejects_non_menu_ancestor() {
    let mut root = Node::new(Role::Group);
    root.set_children([NodeId(TARGET_NODE_ID)]);
    assert_eq!(
        locator(&["Run"]).select_current(
            &update(vec![
                (NodeId(ROOT_NODE_ID), root),
                (NodeId(TARGET_NODE_ID), button(Some("Run")))
            ]),
            [1; 32]
        ),
        Err(HostTargetSelectionError::MenuAncestorMissing)
    );
}

#[test]
fn rejects_missing_name() {
    assert_eq!(
        locator(&["Run"]).select_current(&menu_tree(button(None), vec![]), [1; 32]),
        Err(HostTargetSelectionError::AccessibleNameMissing)
    );
}

#[test]
fn rejects_disabled_target() {
    let mut node = button(Some("Run"));
    node.set_disabled();
    assert_eq!(
        locator(&["Run"]).select_current(&menu_tree(node, vec![]), [1; 32]),
        Err(HostTargetSelectionError::DisabledTarget)
    );
}

#[test]
fn rejects_unbounded_target() {
    let mut node = button(Some("Run"));
    node.set_bounds(Rect {
        x0: 2.0,
        y0: 2.0,
        x1: f64::NAN,
        y1: SMALL_BOUNDS_Y1,
    });
    assert_eq!(
        locator(&["Run"]).select_current(&menu_tree(node, vec![]), [1; 32]),
        Err(HostTargetSelectionError::BoundsMissing)
    );
}

#[test]
fn rejects_inverted_bounds() {
    let mut node = button(Some("Run"));
    node.set_bounds(Rect {
        x0: 4.0,
        y0: 2.0,
        x1: SMALL_BOUNDS_X1,
        y1: SMALL_BOUNDS_Y1,
    });
    assert_eq!(
        locator(&["Run"]).select_current(&menu_tree(node, vec![]), [1; 32]),
        Err(HostTargetSelectionError::BoundsMissing)
    );
}

#[test]
fn rejects_invalid_descriptor_digest() {
    let mut invalid = descriptor(&["Run"]);
    invalid.locale_label_digests = vec!["not-a-digest".to_owned()];
    assert_eq!(
        HostTargetLocator::from_context_menu_case(&invalid),
        Err(HostTargetSelectionError::InvalidDescriptor)
    );
}

#[test]
fn rejects_accessible_name_mismatch_under_menu() {
    assert_eq!(
        locator(&["Run"]).select_current(&menu_tree(button(Some("Other")), vec![]), [1; 32]),
        Err(HostTargetSelectionError::AccessibleNameMismatch)
    );
}

#[test]
fn rejects_descriptor_digest_with_malformed_payload() {
    let mut invalid = descriptor(&["Run"]);
    invalid.locale_label_digests = vec!["sha256:zz".to_owned()];
    assert_eq!(
        HostTargetLocator::from_context_menu_case(&invalid),
        Err(HostTargetSelectionError::InvalidDescriptor)
    );
}

#[test]
fn rejects_descriptor_digest_with_uppercase_payload() {
    let mut invalid = descriptor(&["Run"]);
    invalid.locale_label_digests[0] = invalid.locale_label_digests[0].to_uppercase();
    assert_eq!(
        HostTargetLocator::from_context_menu_case(&invalid),
        Err(HostTargetSelectionError::InvalidDescriptor)
    );
}

#[test]
fn rejects_descriptor_digest_with_wrong_payload_length() {
    let mut invalid = descriptor(&["Run"]);
    invalid.locale_label_digests = vec!["sha256:00".to_owned()];
    assert_eq!(
        HostTargetLocator::from_context_menu_case(&invalid),
        Err(HostTargetSelectionError::InvalidDescriptor)
    );
}

#[test]
fn rejects_empty_descriptor_digest_set() {
    let mut invalid = descriptor(&["Run"]);
    invalid.locale_label_digests.clear();
    assert_eq!(
        HostTargetLocator::from_context_menu_case(&invalid),
        Err(HostTargetSelectionError::InvalidDescriptor)
    );
}

#[test]
fn legacy_locator_api_still_selects_without_menu_requirement() {
    let locator = HostTargetLocator::from_accessible_name(Role::Button, "Run");
    assert!(locator
        .select_current(
            &update(vec![(NodeId(ROOT_NODE_ID), button(Some("Run")))]),
            [1; 32]
        )
        .is_ok());
}
