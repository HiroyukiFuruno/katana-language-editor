use super::{VecMacroInput, file_has_bare_vec_ambiguity, parse_vec_macro};

fn macro_node(source: &str) -> Result<syn::Macro, String> {
    let expression = syn::parse_str::<syn::Expr>(source)
        .map_err(|error| format!("fixture parse failed: {error}"))?;
    match expression {
        syn::Expr::Macro(expression) => Ok(expression.mac),
        _ => Err("fixture is not a macro expression".to_string()),
    }
}

fn scan_source(
    source: &str,
    label: &str,
) -> Result<super::super::super::scan_state::ScanState, String> {
    let file = syn::parse_file(source).map_err(|error| format!("source parse failed: {error}"))?;
    let root =
        std::env::temp_dir().join(format!("kle-macro-recovery-{label}-{}", std::process::id()));
    let path = root.join("src").join("lib.rs");
    let mut state = super::super::super::scan_state::ScanState::default();
    let mut discovered = Vec::new();
    super::super::SourceClosureVisitor::new(
        &root,
        &path,
        "src/lib.rs",
        &mut state,
        &mut discovered,
    )
    .scan_file(&file);
    Ok(state)
}

#[test]
fn parses_empty_two_three_element_trailing_list_and_repeat() -> Result<(), String> {
    let empty = macro_node("vec![]")?;
    if !matches!(parse_vec_macro(&empty, false), Ok(VecMacroInput::List(items)) if items.is_empty())
    {
        return Err("empty vec list was not recovered".to_string());
    }
    let two = macro_node("vec![first(), second(),]")?;
    if !matches!(parse_vec_macro(&two, false), Ok(VecMacroInput::List(items)) if items.len() == 2) {
        return Err("two-element vec list was not recovered".to_string());
    }
    let three = macro_node("vec![first(), second(), third(),]")?;
    if !matches!(parse_vec_macro(&three, false), Ok(VecMacroInput::List(items)) if items.len() == 3)
    {
        return Err("three-element vec list was not recovered".to_string());
    }
    let repeat = macro_node("vec![value(); count()]")?;
    if !matches!(
        parse_vec_macro(&repeat, false),
        Ok(VecMacroInput::Repeat { .. })
    ) {
        return Err("repeat vec form was not recovered".to_string());
    }
    Ok(())
}

#[test]
fn rejects_repetition_unknown_macros_and_malformed_tokens_deterministically() -> Result<(), String>
{
    let repetition = macro_node("vec![$($x:expr),*]")?;
    if parse_vec_macro(&repetition, false).is_ok() {
        return Err("macro repetition was incorrectly recovered".to_string());
    }
    let custom = macro_node("custom![value]")?;
    if parse_vec_macro(&custom, false).is_ok() {
        return Err("custom macro was incorrectly recovered".to_string());
    }
    let malformed = macro_node("vec![value,,]")?;
    let first = parse_vec_macro(&malformed, false)
        .err()
        .ok_or_else(|| "malformed vec unexpectedly parsed".to_string())?;
    let second = parse_vec_macro(&malformed, false)
        .err()
        .ok_or_else(|| "malformed vec unexpectedly parsed".to_string())?;
    if first != second {
        return Err("malformed vec recovery was nondeterministic".to_string());
    }
    Ok(())
}

#[test]
fn blocks_bare_vec_when_resolution_is_ambiguous_but_allows_absolute_paths() -> Result<(), String> {
    let shadowed =
        syn::parse_str::<syn::File>("macro_rules! vec { ($x:expr) => {}; } fn f() { vec![1]; }")
            .map_err(|error| format!("shadowing fixture parse failed: {error}"))?;
    if !file_has_bare_vec_ambiguity(&shadowed) {
        return Err("local vec macro did not block bare recovery".to_string());
    }
    let bare = macro_node("vec![1]")?;
    if parse_vec_macro(&bare, true).is_ok() {
        return Err("ambiguous bare vec was recovered".to_string());
    }
    let absolute = macro_node("::std::vec![1]")?;
    if parse_vec_macro(&absolute, true).is_err() {
        return Err("absolute std vec was not recovered".to_string());
    }
    Ok(())
}

#[test]
fn visitor_recovers_inventory_branches_once_and_keeps_macro_provenance_unresolved()
-> Result<(), String> {
    let source = r#"
        struct State { ready: bool }
        struct CommandInventoryItem { is_available: fn(&State) -> bool }
        enum AppAction { Available }
        fn commands() -> Vec<CommandInventoryItem> {
            vec![CommandInventoryItem {
                is_available: |state| if state.ready {
                    let _ = AppAction::Available;
                    true
                } else {
                    false
                },
            }]
        }
    "#;
    let state = scan_source(source, "inventory")?;
    if state.edges.iter().filter(|edge| edge.kind == "if").count() != 1 {
        return Err("inventory availability branch was not recovered exactly once".to_string());
    }
    if !state.unresolved_edges.iter().any(|edge| {
        edge.kind == "macro_boundary"
            && edge
                .to_symbol
                .as_deref()
                .is_some_and(|reason| reason.contains("syntax-only"))
    }) {
        return Err("inventory macro boundary was not retained".to_string());
    }
    let construction = state
        .action_constructions
        .iter()
        .find(|construction| construction.variant.as_deref() == Some("Available"))
        .ok_or_else(|| "recovered action construction missing".to_string())?;
    if !construction
        .input_origin_candidates
        .iter()
        .any(|candidate| {
            candidate.kind == "macro_token_boundary"
                && candidate
                    .unresolved_reason
                    .as_deref()
                    .is_some_and(|reason| reason.contains("does not prove"))
        })
    {
        return Err("macro token input boundary was not retained".to_string());
    }
    Ok(())
}

#[test]
fn visitor_retains_repeat_count_calls_and_branches_once() -> Result<(), String> {
    let source = "fn commands() { vec![if value() { 1 } else { 2 }; if flag() { count() } else { other() }]; }";
    let state = scan_source(source, "repeat-count")?;
    if state.edges.iter().filter(|edge| edge.kind == "if").count() != 2 {
        return Err("repeat value and count branches were not both recovered".to_string());
    }
    for symbol in ["value", "flag", "count", "other"] {
        if !state.edges.iter().any(|edge| {
            edge.kind == "call"
                && edge.to_symbol.as_deref().is_some_and(|target| {
                    target == symbol || target.starts_with(&format!("{symbol} ("))
                })
        }) {
            return Err(format!("repeat expression call was not retained: {symbol}"));
        }
    }
    if !state
        .unresolved_edges
        .iter()
        .any(|edge| edge.kind == "macro_boundary")
    {
        return Err("repeat macro boundary was not retained".to_string());
    }
    Ok(())
}

#[test]
fn visitor_does_not_open_nested_unknown_macro_body() -> Result<(), String> {
    let state = scan_source(
        "enum AppAction { Hidden } fn commands() { vec![custom![AppAction::Hidden]]; }",
        "nested",
    )?;
    if !state.action_constructions.is_empty() {
        return Err("nested unknown macro body was incorrectly opened".to_string());
    }
    if !state
        .unresolved_edges
        .iter()
        .any(|edge| edge.kind == "macro_boundary")
    {
        return Err("nested unknown macro boundary was not retained".to_string());
    }
    Ok(())
}
