use std::fs;

use syn::spanned::Spanned;
use syn::visit::{self, Visit};

use super::{VecMacroInput, parse_vec_macro};

struct IfSpans {
    condition: Option<proc_macro2::Span>,
    body: Option<proc_macro2::Span>,
}

impl<'ast> Visit<'ast> for IfSpans {
    fn visit_expr_if(&mut self, node: &'ast syn::ExprIf) {
        if self.condition.is_none() {
            self.condition = Some(node.cond.span());
            self.body = Some(node.then_branch.span());
        }
        visit::visit_expr_if(self, node);
    }
}

struct VecMacroFinder {
    macro_node: Option<syn::Macro>,
}

impl<'ast> Visit<'ast> for VecMacroFinder {
    fn visit_macro(&mut self, node: &'ast syn::Macro) {
        if self.macro_node.is_none() && node.path.is_ident("vec") {
            self.macro_node = Some(node.clone());
        }
        visit::visit_macro(self, node);
    }
}

#[test]
fn preserves_star_prefixed_branch_spans_and_excerpt_hash() -> Result<(), String> {
    let source = "fn commands() { vec![CommandInventoryItem { is_available: |state| { /* ⭐️ */ if state.ready { true } else { false } } }]; }";
    let file =
        syn::parse_file(source).map_err(|error| format!("star fixture parse failed: {error}"))?;
    let mut finder = VecMacroFinder { macro_node: None };
    finder.visit_file(&file);
    let macro_node = finder
        .macro_node
        .ok_or_else(|| "star fixture vec macro was not found".to_string())?;
    let items = match parse_vec_macro(&macro_node, false)? {
        VecMacroInput::List(items) => items,
        VecMacroInput::Repeat { .. } => {
            return Err("star fixture unexpectedly used repeat vec syntax".to_string());
        }
    };
    let mut spans = IfSpans {
        condition: None,
        body: None,
    };
    for item in &items {
        spans.visit_expr(item);
    }
    let condition = spans
        .condition
        .ok_or_else(|| "star fixture branch was not found".to_string())?;
    let body = spans
        .body
        .ok_or_else(|| "star fixture branch body was not found".to_string())?;

    let root = std::env::temp_dir().join(format!("kle-macro-recovery-star-{}", std::process::id()));
    let result = (|| -> Result<(), String> {
        fs::create_dir_all(root.join("src"))
            .map_err(|error| format!("star fixture directory failed: {error}"))?;
        fs::write(root.join("src/lib.rs"), source)
            .map_err(|error| format!("star fixture write failed: {error}"))?;
        let mut state = super::super::super::scan_state::ScanState::default();
        let mut discovered = Vec::new();
        super::super::SourceClosureVisitor::new(
            &root,
            &root.join("src/lib.rs"),
            "src/lib.rs",
            &mut state,
            &mut discovered,
        )
        .scan_file(&file);
        let if_edge = state
            .edges
            .iter()
            .find(|edge| edge.kind == "if")
            .ok_or_else(|| "star fixture if edge missing".to_string())?;
        let body_edge = state
            .edges
            .iter()
            .find(|edge| edge.kind == "if_then")
            .ok_or_else(|| "star fixture if body edge missing".to_string())?;
        let expected_condition =
            super::super::super::ast_resolution::span_to_text("src/lib.rs", &condition);
        let expected_body = super::super::super::ast_resolution::span_to_text("src/lib.rs", &body);
        if if_edge.span != expected_condition || body_edge.span != expected_body {
            return Err(format!(
                "star fixture spans changed: if={} body={} expected_if={} expected_body={}",
                if_edge.span, body_edge.span, expected_condition, expected_body
            ));
        }
        let catalog = super::super::super::branch_catalog::materialize_branch_catalog(
            super::super::super::model::ManifestRoot {
                schema_version: String::new(),
                katana_revision: String::new(),
                katana_tree_fingerprint: String::new(),
                katana_external_ui_fingerprint: String::new(),
                user_mandated_extensions_fingerprint: String::new(),
                source_universe_fingerprint: String::new(),
                requirement_source_aliases_fingerprint: String::new(),
                kle_tree_fingerprint: String::new(),
                kuc_tree_fingerprint: String::new(),
                release_profile_matrix_fingerprint: String::new(),
                generator_fingerprint: String::new(),
                generated_at_utc: String::new(),
                static_leaf_count: None,
                expected_leaf_ids: None,
            },
            &state,
            &[],
            &root,
        )?;
        let branch = catalog
            .branches
            .iter()
            .find(|branch| branch.kind == "if")
            .ok_or_else(|| "star fixture branch catalog entry missing".to_string())?;
        if branch.condition != "state.ready"
            || branch.source_excerpt_sha256
                != crate::source_closure::fingerprint::sha256_hex(branch.condition.as_bytes())
        {
            return Err(format!(
                "star fixture branch excerpt was not preserved: {:?}",
                branch
            ));
        }
        Ok(())
    })();
    let cleanup =
        fs::remove_dir_all(&root).map_err(|error| format!("star fixture cleanup failed: {error}"));
    result.and(cleanup)
}
