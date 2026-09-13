mod actions;
mod definitions;
mod input_candidates;
mod recording;
mod resolution;
mod state;
mod visit;

pub(super) use state::SourceClosureVisitor;

pub(super) fn type_shape_for_index(ty: &syn::Type) -> String {
    SourceClosureVisitor::type_shape(ty)
}
