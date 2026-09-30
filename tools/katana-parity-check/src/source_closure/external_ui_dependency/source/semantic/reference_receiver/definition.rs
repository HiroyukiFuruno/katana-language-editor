use syn::visit::Visit;
use syn::{ImplItem, Item, PathArguments, ReceiverKind, Type};

pub(super) fn named_type(ty: &Type, name: &str) -> bool {
    let Type::Path(path) = ty else { return false };
    path.qself.is_none()
        && path.path.leading_colon.is_none()
        && path.path.segments.len() == 1
        && path.path.segments[0].ident == name
}

pub(super) fn plain_named_type(ty: &Type, name: &str) -> bool {
    named_type(ty, name)
        && matches!(ty, Type::Path(path)
        if matches!(path.path.segments[0].arguments, PathArguments::None))
}

pub(super) fn has_unique_ui_ctx(sources: &[(String, syn::File)]) -> bool {
    let mut definitions = Definitions::default();
    for (_, file) in sources {
        definitions.visit_file(file);
    }
    definitions.type_count == 1
        && definitions.method_count == 1
        && definitions.valid
        && !definitions.ambiguous
}

#[derive(Default)]
struct Definitions {
    type_count: usize,
    method_count: usize,
    valid: bool,
    ambiguous: bool,
}

impl<'ast> Visit<'ast> for Definitions {
    fn visit_item(&mut self, item: &'ast Item) {
        match item {
            Item::Struct(item) if item.ident == "Ui" => {
                self.type_count += 1;
                self.ambiguous |= !item.generics.params.is_empty();
            }
            Item::Type(item) if item.ident == "Ui" => self.ambiguous = true,
            Item::Enum(item) if item.ident == "Ui" => self.ambiguous = true,
            Item::Trait(item) if item.ident == "Ui" => self.ambiguous = true,
            Item::Union(item) if item.ident == "Ui" => self.ambiguous = true,
            Item::Impl(item) if named_type(&item.self_ty, "Ui") => self.inspect_impl(item),
            _ => {}
        }
        syn::visit::visit_item(self, item);
    }
}

impl Definitions {
    fn inspect_impl(&mut self, item: &syn::ItemImpl) {
        for member in &item.items {
            let ImplItem::Fn(function) = member else {
                continue;
            };
            if function.sig.ident != "ctx" {
                continue;
            }
            self.method_count += 1;
            self.valid = item.trait_.is_none()
                && item.generics.params.is_empty()
                && item.generics.where_clause.is_none()
                && plain_named_type(&item.self_ty, "Ui")
                && function.sig.generics.params.is_empty()
                && function.sig.generics.where_clause.is_none()
                && function.sig.inputs.len() == 1
                && function
                    .sig
                    .receiver()
                    .is_some_and(|receiver| matches!(receiver.kind, ReceiverKind::Reference(..)));
        }
    }
}
