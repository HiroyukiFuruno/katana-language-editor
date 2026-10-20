use super::*;

#[test]
fn handler_definition_index_rejects_same_name_on_unrelated_impl_type() -> TestResult {
    let source = r#"
        enum AppAction { Bold }
        struct Editor;
        struct Other;
        impl Editor {
            fn dispatch_action(&self, action: AppAction) {
                match action { AppAction::Bold => self.handle_bold(), _ => {} }
            }
        }
        impl Other { fn handle_bold(&self) {} }
    "#;
    let (state, _discovered, root) = scan_source_fixture(
        "handler-unrelated-impl",
        &[("src/lib.rs", source)],
        "src/lib.rs",
    )?;
    let artifact = crate::source_closure::action_origins::materialize_action_origins(
        test_manifest_root(),
        &state,
    );
    let route = &artifact.actions[0].forward_route[0];
    assert!(route.contains("handler_definition=unresolved"));
    assert!(route.contains("receiver-incompatible inherent handler definition"));
    cleanup_dir(root)?;
    Ok(())
}
