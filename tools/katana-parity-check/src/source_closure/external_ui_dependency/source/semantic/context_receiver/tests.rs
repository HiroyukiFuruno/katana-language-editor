use super::{collect, definition, visitor};

fn sources(source: &str) -> Result<Vec<(String, syn::File)>, syn::Error> {
    Ok(vec![(
        "src/widgets/text_edit/builder.rs".into(),
        syn::parse_file(source)?,
    )])
}

const DEFINITIONS: &str = "
    struct Ui; struct Context; struct TextEdit;
    impl Ui { fn ctx(&self) -> &Context { todo!() } }
    impl Context { fn layer_transform_to_global(&self, _: usize) -> Option<usize> { None } }
";

#[test]
fn records_the_inner_context_call_without_resolving_the_outer_chain() -> Result<(), syn::Error> {
    let source = format!(
        "{DEFINITIONS} impl TextEdit {{ fn show(ui: &mut Ui) {{ ui.ctx().layer_transform_to_global(1).unwrap_or_default(); }} }}"
    );
    let sources = sources(&source)?;
    assert!(
        definition::context_methods(&sources)
            .is_some_and(|methods| methods.contains("layer_transform_to_global"))
    );
    assert_eq!(definition::ui_ctx(&sources).as_deref(), Some("ctx"));
    assert!(visitor::is_ui_ctx_call(
        &syn::parse_str::<syn::Expr>("ui.ctx()")?,
        "ui",
        "ctx"
    ));
    assert_eq!(
        collect(&sources)
            .into_iter()
            .map(|edge| edge.target_symbol)
            .collect::<Vec<_>>(),
        ["Context::layer_transform_to_global"]
    );
    Ok(())
}

#[test]
fn rejects_shadowed_trait_and_dynamic_routes() -> Result<(), syn::Error> {
    for body in [
        "let ui = other; ui.ctx().layer_transform_to_global(1);",
        "let _ = || ui.ctx().layer_transform_to_global(1);",
        "route!(ui.ctx().layer_transform_to_global(1));",
    ] {
        assert!(
            collect(&sources(&format!(
                "{DEFINITIONS} impl TextEdit {{ fn show(ui: &mut Ui) {{ {body} }} }}"
            ))?)
            .is_empty()
        );
    }
    let trait_source = "
        struct Ui; struct Context; struct TextEdit; trait T { fn layer_transform_to_global(&self, _: usize); }
        impl Ui { fn ctx(&self) -> &Context { todo!() } }
        impl Context { fn layer_transform_to_global(&self, _: usize) {} }
        impl T for Context { fn layer_transform_to_global(&self, _: usize) {} }
        impl TextEdit { fn show(ui: &mut Ui) { ui.ctx().layer_transform_to_global(1); } }
    ";
    assert!(collect(&sources(trait_source)?).is_empty());
    Ok(())
}
