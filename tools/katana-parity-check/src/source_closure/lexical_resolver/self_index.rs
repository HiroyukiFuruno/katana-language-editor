use std::collections::{BTreeMap, BTreeSet};

use syn::{ImplItem, Item, Type};

#[derive(Clone, Debug, Default)]
pub(super) struct SelfIndex {
    members: BTreeMap<String, BTreeSet<String>>,
}

impl SelfIndex {
    pub(super) fn from_items(items: &[Item]) -> Self {
        let mut index = Self::default();
        for item in items {
            index.add_item(item);
        }
        index
    }

    fn add_item(&mut self, item: &Item) {
        match item {
            Item::Impl(implementation) => self.add_inherent_impl(implementation),
            Item::Enum(enumeration) => self.add_enum(enumeration),
            _ => {}
        }
    }

    fn add_inherent_impl(&mut self, implementation: &syn::ItemImpl) {
        if implementation.trait_.is_some() {
            return;
        }
        let Type::Path(path) = implementation.self_ty.as_ref() else {
            return;
        };
        let members = self.members.entry(path_to_name(&path.path)).or_default();
        for item in &implementation.items {
            add_impl_member(members, item);
        }
    }

    fn add_enum(&mut self, enumeration: &syn::ItemEnum) {
        let members = self
            .members
            .entry(enumeration.ident.to_string())
            .or_default();
        members.extend(
            enumeration
                .variants
                .iter()
                .map(|variant| variant.ident.to_string()),
        );
    }

    pub(super) fn contains(&self, type_name: &str, segments: &[String]) -> bool {
        segments.len() == 2
            && self
                .members
                .get(type_name)
                .is_some_and(|members| members.contains(&segments[1]))
    }
}

fn add_impl_member(members: &mut BTreeSet<String>, item: &ImplItem) {
    match item {
        ImplItem::Fn(function) => {
            members.insert(function.sig.ident.to_string());
        }
        ImplItem::Const(constant) => {
            members.insert(constant.ident.to_string());
        }
        _ => {}
    }
}

fn path_to_name(path: &syn::Path) -> String {
    path.segments
        .iter()
        .map(|segment| segment.ident.to_string())
        .collect::<Vec<_>>()
        .join("::")
}
