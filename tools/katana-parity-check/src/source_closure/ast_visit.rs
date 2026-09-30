mod dispatch;
mod handler_core;
mod input;
mod syntax;

use syn::visit::{self, Visit};
use syn::{
    Attribute, Expr, ExprBinary, ExprBreak, ExprCall, ExprClosure, ExprContinue, ExprForLoop,
    ExprIf, ExprLoop, ExprMatch, ExprMethodCall, ExprPath, ExprReturn, ExprStruct, ExprTry,
    ExprWhile, File, ImplItem, ItemEnum, ItemFn, ItemImpl, ItemMod, ItemTrait, ItemUse, Local,
    Macro, Pat, spanned::Spanned,
};

use super::ast_resolution::collect_lexical_imports;
use super::ast_scan::SourceClosureVisitor;
pub(super) use super::{ast_resolution, lexical_resolver, scan_state};
use dispatch::{app_action_pattern_variants, direct_handler_calls};
use input::input_candidates_for_condition;

include!("ast_visit/items.inc");
include!("ast_visit/controlflow.inc");
include!("ast_visit/expressions.inc");

impl<'ast, 'a> Visit<'ast> for SourceClosureVisitor<'a> {
    source_closure_visit_items!();
    source_closure_visit_controlflow!();
    source_closure_visit_expressions!();
}

pub(super) fn direct_path_calls(block: &syn::Block) -> Vec<String> {
    dispatch::direct_path_calls(block)
}

pub(super) fn index_handler_body(
    block: &syn::Block,
    attrs: &[syn::Attribute],
    file: &str,
) -> super::scan_state::HandlerBodyIndex {
    handler_core::index_handler_body(block, attrs, file)
}
