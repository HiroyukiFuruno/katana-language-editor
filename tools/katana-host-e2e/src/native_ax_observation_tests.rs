use super::{
    NativeAxObservation, NativeAxObservationError, NativeAxRole, NativeAxSourceContract,
    RedactedNode, digest_text,
};
const DIGEST_BYTES: usize = 32;
const SOURCE_SPAN_BYTE: u8 = 7;

fn source() -> NativeAxSourceContract {
    NativeAxSourceContract {
        source_span_digest: [SOURCE_SPAN_BYTE; DIGEST_BYTES],
        accesskit_role_digest: digest_text("MultilineTextInput"),
        native_role: NativeAxRole::MultilineTextInput,
    }
}

fn node(role: &str, title: Option<&str>, state: bool) -> RedactedNode {
    RedactedNode {
        role: role.to_owned(),
        title: title.map(ToOwned::to_owned),
        description: None,
        enabled: Some(state),
        focused: Some(false),
        read_only: Some(false),
        ancestor_roles: vec!["AXWindow".to_owned()],
        depth: 2,
    }
}

fn frame(workspace: Option<&str>, editors: usize) -> Vec<RedactedNode> {
    let mut nodes = vec![node("AXWindow", None, true)];
    if let Some(name) = workspace {
        nodes.push(node("AXTab", Some(name), true));
    }
    for _ in 0..editors {
        nodes.push(node("AXTextArea", None, true));
    }
    nodes
}

#[test]
fn mismatched_workspace_fails() {
    assert_eq!(
        NativeAxObservation::observe_redacted_frame(
            &frame(Some("other"), 1),
            &source(),
            [1; 32],
            "run-a"
        ),
        Err(NativeAxObservationError::WorkspaceMismatch)
    );
}

#[test]
fn duplicate_workspace_fails() {
    let mut nodes = frame(Some("run-a"), 1);
    nodes.push(node("AXStaticText", Some("run-a"), true));
    assert_eq!(
        NativeAxObservation::observe_redacted_frame(&nodes, &source(), [1; 32], "run-a"),
        Err(NativeAxObservationError::WorkspaceDuplicate)
    );
}

#[test]
fn missing_editor_attribute_fails() {
    let mut nodes = frame(Some("run-a"), 1);
    nodes[2].focused = None;
    assert_eq!(
        NativeAxObservation::observe_redacted_frame(&nodes, &source(), [1; 32], "run-a"),
        Err(NativeAxObservationError::AttributeMissing)
    );
}

#[test]
fn zero_editor_candidates_fails() {
    assert_eq!(
        NativeAxObservation::observe_redacted_frame(
            &frame(Some("run-a"), 0),
            &source(),
            [1; 32],
            "run-a"
        ),
        Err(NativeAxObservationError::EditorTargetMissing)
    );
}

#[test]
fn multiple_editor_candidates_fail() {
    assert_eq!(
        NativeAxObservation::observe_redacted_frame(
            &frame(Some("run-a"), 2),
            &source(),
            [1; 32],
            "run-a"
        ),
        Err(NativeAxObservationError::EditorTargetAmbiguous)
    );
}

#[test]
fn exact_workspace_basename_digest_is_redacted() {
    let result = NativeAxObservation::observe_redacted_frame(
        &frame(Some("run-a"), 1),
        &source(),
        [1; 32],
        "run-a",
    )
    .expect("observation");
    assert_eq!(result.workspace_basename_digest, digest_text("run-a"));
    assert!(!format!("{result:?}").contains("run-a"));
}

#[test]
fn role_translation_is_explicit_and_unknown_roles_fail_typed() {
    assert_eq!(
        NativeAxRole::from_accesskit_role("MultilineTextInput"),
        Ok(NativeAxRole::MultilineTextInput)
    );
    assert_eq!(
        NativeAxRole::from_accesskit_role("TextField"),
        Err(NativeAxObservationError::UnsupportedRoleTranslation)
    );
}
