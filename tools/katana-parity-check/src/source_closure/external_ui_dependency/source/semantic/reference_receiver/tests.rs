use super::collect;

fn edges(definitions: &str, parameter: &str, body: &str) -> Result<usize, syn::Error> {
    let file = syn::parse_file(&format!(
        "{definitions} struct TextEdit; impl TextEdit {{ fn show({parameter}) {{ {body} }} }}"
    ))?;
    let sources = [("src/widgets/text_edit/builder.rs".into(), file)];
    let direct = collect(&sources).len();
    let integrated = super::super::collect(&sources)
        .iter()
        .filter(|edge| edge.target_symbol == "Ui::ctx")
        .count();
    assert_eq!(integrated, direct, "reference receiver policy was bypassed");
    Ok(direct)
}

const UI: &str = "struct Ui; impl Ui { fn ctx(&self) {} }";

#[test]
fn resolves_exact_reference_receiver() -> Result<(), syn::Error> {
    assert_eq!(edges(UI, "ui: &mut Ui", "let state = load(ui.ctx());")?, 1);
    assert_eq!(edges(UI, "ui: &Ui", "ui.ctx();")?, 1);
    assert_eq!(
        edges(
            UI,
            "ui: &mut Ui",
            "const MIN_WIDTH: f32 = 24.0; let state = load(ui.ctx());"
        )?,
        1
    );
    Ok(())
}

#[test]
fn rejects_ambiguous_trait_and_complex_types() -> Result<(), syn::Error> {
    for definitions in [
        "struct Ui; impl SomeTrait for Ui { fn ctx(&self) {} }",
        "struct Ui; struct Ui; impl Ui { fn ctx(&self) {} }",
        "struct Ui; mod nested { struct Ui; } impl Ui { fn ctx(&self) {} }",
        "type Ui = Other; impl Ui { fn ctx(&self) {} }",
        "struct Ui; impl Ui { fn ctx(&self) {} } impl Other for Ui { fn ctx(&self) {} }",
    ] {
        assert_eq!(edges(definitions, "ui: &mut Ui", "ui.ctx();")?, 0);
    }
    for parameter in [
        "ui: &&Ui",
        "ui: &dyn Ui",
        "ui: &other::Ui",
        "ui: &Ui<T>",
        "ui: &impl Ui",
    ] {
        assert_eq!(edges(UI, parameter, "ui.ctx();")?, 0);
    }
    Ok(())
}

#[test]
fn preserves_the_exact_call_span() -> Result<(), syn::Error> {
    let source = "struct Ui; impl Ui { fn ctx(&self) {} }\nstruct TextEdit; impl TextEdit { fn show(ui: &mut Ui) {\n    let state = load(ui.ctx());\n} }";
    let edges = collect(&[(
        "src/widgets/text_edit/builder.rs".into(),
        syn::parse_file(source)?,
    )]);
    assert_eq!(edges.len(), 1);
    assert_eq!(
        edges[0].span,
        "egui:src/widgets/text_edit/builder.rs:3:21-3:29"
    );
    assert_eq!(edges[0].target_symbol, "Ui::ctx");
    Ok(())
}

#[test]
fn rejects_shadow_macro_and_deferred_paths() -> Result<(), syn::Error> {
    for body in [
        "let ui = other; ui.ctx();",
        "let (ui, _) = other; ui.ctx();",
        "shadow!(); ui.ctx();",
        "let _ = (unknown!(), ui.ctx());",
        "let deferred = || ui.ctx();",
        "if let Some(ui) = other { ui.ctx(); }",
        "match other { Some(ui) => ui.ctx(), _ => () }",
        "ui.clone().ctx();",
        "ui.ctx::<Other>();",
    ] {
        assert_eq!(edges(UI, "ui: &mut Ui", body)?, 0, "{body}");
    }
    Ok(())
}
