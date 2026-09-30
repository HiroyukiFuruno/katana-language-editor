mod definition;
#[cfg(test)]
mod tests;
mod visitor;

use super::super::super::super::edge_model::ExternalUiDirectSemanticEdge;

pub(super) fn collect(sources: &[(String, syn::File)]) -> Vec<ExternalUiDirectSemanticEdge> {
    let Some(targets) = definition::context_methods(sources) else {
        return Vec::new();
    };
    let Some(ctx) = definition::ui_ctx(sources) else {
        return Vec::new();
    };
    sources
        .iter()
        .flat_map(|(path, file)| visitor::collect_file(path, file, &ctx, &targets))
        .collect()
}
