use std::collections::BTreeSet;

use syn::visit::Visit;

use super::leaf_inventory_entries::{CONTEXT, TOOLBAR};

pub(super) fn find_test_function(source: &str, expected: &str) -> Option<syn::ItemFn> {
    syn::parse_file(source)
        .ok()?
        .items
        .into_iter()
        .find_map(|item| {
            let syn::Item::Fn(function) = item else {
                return None;
            };
            (function.sig.ident == expected
                && function
                    .attrs
                    .iter()
                    .any(|attribute| attribute.path().is_ident("test")))
            .then_some(function)
        })
}

pub(super) struct LeafCaseRequirements {
    pub(super) leaf_assertion: &'static str,
    pub(super) artifact_assertion: &'static str,
    pub(super) accesskit_assertion: &'static str,
}

impl LeafCaseRequirements {
    pub(super) fn for_parent(parent_group: &str) -> Result<Self, String> {
        let (artifact_assertion, accesskit_assertion) = match parent_group {
            TOOLBAR => (
                "assert_toolbar_artifact_leaf",
                "assert_toolbar_accesskit_leaf",
            ),
            CONTEXT => (
                "assert_context_artifact_leaf",
                "assert_context_accesskit_leaf",
            ),
            "code-block-menu" => ("assert_code_artifact_leaf", "assert_code_accesskit_leaf"),
            "image-ingest" => ("assert_image_artifact_leaf", "assert_image_accesskit_leaf"),
            _ => {
                return Err(format!(
                    "leaf has no artifact/AccessKit category: {parent_group}"
                ));
            }
        };
        Ok(Self {
            leaf_assertion: "assert_leaf_case",
            artifact_assertion,
            accesskit_assertion,
        })
    }
}

#[derive(Default)]
pub(super) struct ExactLeafAssertionCalls {
    calls: BTreeSet<(String, String)>,
}

impl ExactLeafAssertionCalls {
    pub(super) fn collect(block: &syn::Block) -> Self {
        let mut assertions = Self::default();
        assertions.visit_block(block);
        assertions
    }

    pub(super) fn contains(&self, assertion: &str, leaf_id: &str) -> bool {
        self.calls
            .contains(&(assertion.to_string(), leaf_id.to_string()))
    }
}

impl<'ast> Visit<'ast> for ExactLeafAssertionCalls {
    fn visit_expr_call(&mut self, call: &'ast syn::ExprCall) {
        let Some(assertion) = call_name(call) else {
            syn::visit::visit_expr_call(self, call);
            return;
        };
        let Some(syn::Expr::Lit(syn::ExprLit {
            lit: syn::Lit::Str(leaf_id),
            ..
        })) = call.args.first()
        else {
            syn::visit::visit_expr_call(self, call);
            return;
        };
        self.calls.insert((assertion, leaf_id.value()));
        syn::visit::visit_expr_call(self, call);
    }
}

fn call_name(call: &syn::ExprCall) -> Option<String> {
    let syn::Expr::Path(path) = call.func.as_ref() else {
        return None;
    };
    path.path
        .segments
        .last()
        .map(|segment| segment.ident.to_string())
}
