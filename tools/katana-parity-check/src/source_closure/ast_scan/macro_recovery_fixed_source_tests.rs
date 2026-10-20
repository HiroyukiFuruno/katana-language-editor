use std::fs;

use syn::visit::{self, Visit};

use crate::source_closure::fingerprint::sha256_hex;

use super::{VecMacroInput, file_has_bare_vec_ambiguity, parse_vec_macro};

const FIXED_VIEW_COMMANDS_SOURCE: &str = include_str!("testdata/view_commands.fixed.rs");
const FIXED_KATANA_REVISION: &str = "4f6a6287c650a38633c7baeb544a92e739c68567";
const FIXED_VIEW_COMMANDS_GIT_BLOB_SHA: &str = "33ab91a92e0ab869433e8c54172f16bcaa29d38b";
const FIXED_VIEW_COMMANDS_SHA256: &str =
    "sha256:f73e623798485c5e5e01ddf0cd3300590a957b1b9fe01d043715031aa7c7988a";

struct VecMacroCollector {
    list_lengths: Vec<usize>,
    errors: Vec<String>,
}

impl<'ast> Visit<'ast> for VecMacroCollector {
    fn visit_macro(&mut self, node: &'ast syn::Macro) {
        if node.path.is_ident("vec") {
            match parse_vec_macro(node, false) {
                Ok(VecMacroInput::List(items)) => self.list_lengths.push(items.len()),
                Ok(VecMacroInput::Repeat { .. }) => {}
                Err(error) => self.errors.push(error),
            }
        }
        visit::visit_macro(self, node);
    }
}

#[test]
fn parses_fixed_view_commands_vec_with_all_sixteen_items() -> Result<(), String> {
    if sha256_hex(FIXED_VIEW_COMMANDS_SOURCE.as_bytes()) != FIXED_VIEW_COMMANDS_SHA256 {
        return Err(format!(
            "fixed view_commands fixture hash changed for revision {FIXED_KATANA_REVISION} blob {FIXED_VIEW_COMMANDS_GIT_BLOB_SHA}"
        ));
    }
    let file = syn::parse_file(FIXED_VIEW_COMMANDS_SOURCE)
        .map_err(|error| format!("fixed view_commands parse failed: {error}"))?;
    if file_has_bare_vec_ambiguity(&file) {
        return Err("fixed view_commands has ambiguous bare vec resolution".to_string());
    }
    let mut collector = VecMacroCollector {
        list_lengths: Vec::new(),
        errors: Vec::new(),
    };
    collector.visit_file(&file);
    if !collector.errors.is_empty() {
        return Err(format!(
            "fixed view_commands recovery failed: {:?}",
            collector.errors
        ));
    }
    if collector.list_lengths != [16] {
        return Err(format!(
            "fixed view_commands vec list lengths were {:?}, expected [16]",
            collector.list_lengths
        ));
    }

    let root =
        std::env::temp_dir().join(format!("kle-macro-recovery-fixed-{}", std::process::id()));
    let result = (|| -> Result<(), String> {
        fs::create_dir_all(root.join("src"))
            .map_err(|error| format!("fixed fixture directory failed: {error}"))?;
        let path = root.join("src/view_commands.rs");
        fs::write(&path, FIXED_VIEW_COMMANDS_SOURCE)
            .map_err(|error| format!("fixed fixture write failed: {error}"))?;
        let mut state = super::super::super::scan_state::ScanState::default();
        let mut discovered = Vec::new();
        super::super::SourceClosureVisitor::new(
            &root,
            &path,
            "src/view_commands.rs",
            &mut state,
            &mut discovered,
        )
        .scan_file(&file);
        if state.edges.iter().filter(|edge| edge.kind == "if").count() != 1 {
            return Err("fixed callback branch was not recovered exactly once".to_string());
        }
        if !state.unresolved_edges.iter().any(|edge| {
            edge.kind == "macro_boundary"
                && edge
                    .to_symbol
                    .as_deref()
                    .is_some_and(|reason| reason.contains("syntax-only"))
        }) {
            return Err("fixed vec macro boundary was not retained".to_string());
        }
        Ok(())
    })();
    let cleanup =
        fs::remove_dir_all(&root).map_err(|error| format!("fixed fixture cleanup failed: {error}"));
    result.and(cleanup)
}
