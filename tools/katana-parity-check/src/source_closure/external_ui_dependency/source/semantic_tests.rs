use super::collect;

fn sources(source: &str) -> Result<Vec<(String, syn::File)>, syn::Error> {
    Ok(vec![(
        "src/widgets/text_edit.rs".into(),
        syn::parse_file(source)?,
    )])
}

#[test]
fn records_only_direct_unambiguous_associated_calls_from_seed_methods() -> Result<(), syn::Error> {
    let edges = collect(&sources(
        "
        struct TextEdit;
        struct TextEditState;
        impl TextEditState { fn load() {} fn store(&self) {} }
        impl TextEdit {
            fn show() { TextEditState::load(); }
            fn load_state() { TextEditState::load(); }
            fn store_state(state: TextEditState) { state.store(); }
        }
        ",
    )?);

    assert_eq!(edges.len(), 3);
    assert!(edges.iter().all(|edge| {
        matches!(edge.kind.as_str(), "call" | "method_call")
            && edge.source_file == "egui/src/widgets/text_edit.rs"
            && matches!(
                (edge.from_symbol.as_str(), edge.target_symbol.as_str()),
                ("TextEdit::show", "TextEditState::load")
                    | ("TextEdit::load_state", "TextEditState::load")
                    | ("TextEdit::store_state", "TextEditState::store")
            )
    }));
    Ok(())
}

#[test]
fn rejects_untyped_method_receivers() -> Result<(), syn::Error> {
    let edges = collect(&sources(
        "
        struct TextEdit;
        struct TextEditState;
        impl TextEditState { fn store(&self) {} }
        impl TextEdit {
            fn show() {}
            fn load_state() {}
            fn store_state() { let state = TextEditState; state.store(); }
        }
        ",
    )?);

    assert!(edges.is_empty(), "{edges:?}");
    Ok(())
}

#[test]
fn records_a_same_scope_exact_constructor_binding() -> Result<(), syn::Error> {
    let edges = collect(&sources(
        "
        struct TextEdit;
        struct TextEditState;
        impl TextEditState { fn new() -> Self { Self } fn store(&self) {} }
        impl TextEdit {
            fn show() {}
            fn load_state() {}
            fn store_state() { let state = TextEditState::new(); state.store(); }
        }
        ",
    )?);

    assert!(edges.iter().any(|edge| {
        edge.from_symbol == "TextEdit::store_state"
            && edge.kind == "method_call"
            && edge.target_symbol == "TextEditState::store"
    }));
    Ok(())
}

#[test]
fn records_a_unique_inherent_method_from_a_reference_parameter() -> Result<(), syn::Error> {
    let edges = collect(&sources(
        "
        struct TextEdit;
        struct Ui;
        struct Context;
        impl Ui { fn ctx(&self) -> Context { Context } }
        impl TextEdit {
            fn show(ui: &mut Ui) { ui.ctx(); }
            fn load_state() {}
            fn store_state() {}
        }
        ",
    )?);

    assert!(edges.iter().any(|edge| {
        edge.from_symbol == "TextEdit::show"
            && edge.kind == "method_call"
            && edge.target_symbol == "Ui::ctx"
    }));
    Ok(())
}

#[test]
fn rejects_ambiguous_reference_type_and_trait_only_method() -> Result<(), syn::Error> {
    let edges = collect(&sources(
        "
        struct TextEdit;
        struct Ui;
        struct OtherUi;
        trait HasContext { fn ctx(&self); }
        impl HasContext for Ui { fn ctx(&self) {} }
        impl TextEdit {
            fn show(ui: &mut Ui) { ui.ctx(); }
            fn load_state() {}
            fn store_state() {}
        }
        ",
    )?);
    assert!(edges.is_empty(), "{edges:?}");

    let ambiguous = collect(&[
        ("src/a.rs".into(), syn::parse_file("struct Ui;")?),
        ("src/b.rs".into(), syn::parse_file("struct Ui;")?),
        (
            "src/widgets/text_edit.rs".into(),
            syn::parse_file(
                "
                struct TextEdit;
                impl TextEdit {
                    fn show(ui: &mut Ui) { ui.ctx(); }
                    fn load_state() {}
                    fn store_state() {}
                }
                ",
            )?,
        ),
    ]);
    assert!(ambiguous.is_empty(), "{ambiguous:?}");
    Ok(())
}

#[test]
fn rejects_a_shadowed_exact_constructor_binding() -> Result<(), syn::Error> {
    let edges = collect(&sources(
        "
        struct TextEdit;
        struct TextEditState;
        struct Other;
        impl TextEditState { fn new() -> Self { Self } fn store(&self) {} }
        impl TextEdit {
            fn show() {}
            fn load_state() {}
            fn store_state() {
                let state = TextEditState::new();
                let state = Other;
                state.store();
            }
        }
        ",
    )?);

    assert!(
        !edges
            .iter()
            .any(|edge| edge.target_symbol == "TextEditState::store")
    );
    Ok(())
}

#[test]
fn records_derived_option_default_and_clone_chain() -> Result<(), syn::Error> {
    let edges = collect(&sources(
        "
        struct TextEdit;
        #[derive(Clone, Default)] struct TextEditState;
        impl TextEditState {
            fn load() -> Option<Self> { None }
            fn store(self) {}
        }
        impl TextEdit {
            fn show() {
                let state = TextEditState::load().unwrap_or_default();
                state.clone().store();
            }
            fn load_state() {}
            fn store_state() {}
        }
        ",
    )?);

    assert!(edges.iter().any(|edge| {
        edge.from_symbol == "TextEdit::show"
            && edge.kind == "method_call"
            && edge.target_symbol == "TextEditState::store"
    }));
    Ok(())
}

#[test]
fn rejects_option_default_and_clone_without_source_derives() -> Result<(), syn::Error> {
    let edges = collect(&sources(
        "
        struct TextEdit;
        struct TextEditState;
        impl TextEditState {
            fn load() -> Option<Self> { None }
            fn store(self) {}
        }
        impl TextEdit {
            fn show() {
                let state = TextEditState::load().unwrap_or_default();
                state.clone().store();
            }
            fn load_state() {}
            fn store_state() {}
        }
        ",
    )?);

    assert!(
        !edges
            .iter()
            .any(|edge| edge.target_symbol == "TextEditState::store")
    );
    Ok(())
}

#[test]
fn rejects_calls_inside_closures() -> Result<(), syn::Error> {
    let edges = collect(&sources(
        "
        struct TextEdit;
        struct TextEditState;
        impl TextEditState { fn load() {} }
        impl TextEdit {
            fn show() { let deferred = || TextEditState::load(); deferred(); }
            fn load_state() {}
            fn store_state() {}
        }
        ",
    )?);

    assert!(edges.is_empty(), "{edges:?}");
    Ok(())
}
