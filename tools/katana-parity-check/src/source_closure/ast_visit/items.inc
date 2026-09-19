macro_rules! source_closure_visit_items {
    () => {
    fn visit_file(&mut self, file: &'ast File) {
        visit::visit_file(self, file);
    }

    fn visit_attribute(&mut self, attribute: &'ast Attribute) {
        if let Some(predicate) = super::ast_resolution::cfg_predicate(attribute) {
            self.record_cfg_edge(&predicate, attribute.span());
        }
        visit::visit_attribute(self, attribute);
    }

    fn visit_item_mod(&mut self, node: &'ast ItemMod) {
        self.visit_mod(node);
        let inline = node.content.as_ref().map(|(_, items)| items.as_slice());
        if let Some(items) = inline {
            self.push_inline_scope(items);
        }
        self.with_symbol(node.ident.to_string(), |this| {
            visit::visit_item_mod(this, node);
        });
        if inline.is_some() {
            self.pop_inline_scope();
        }
    }

    fn visit_item_use(&mut self, node: &'ast ItemUse) {
        self.with_symbol("use".to_string(), |this| {
            let mut imports = Vec::new();
            collect_lexical_imports(&node.tree, Vec::new(), &mut imports);
            for import in imports {
                let symbol = import.source.join("::");
                if import.glob {
                    this.record_unresolved(
                        "use",
                        &format!("{symbol} (glob import is intentionally unresolved)"),
                        import.span,
                    );
                } else {
                    let resolution = this.resolve_import(&import.source);
                    this.record_resolution("use", &symbol, resolution, import.span);
                }
            }
            visit::visit_item_use(this, node);
        });
    }

    fn visit_item_enum(&mut self, node: &'ast ItemEnum) {
        self.record_app_action_definition(node);
        self.with_symbol(node.ident.to_string(), |this| {
            this.record_cfg_attributes(&node.attrs);
            visit::visit_item_enum(this, node);
        });
    }

    fn visit_item_fn(&mut self, node: &'ast ItemFn) {
        self.with_symbol(node.sig.ident.to_string(), |this| {
            this.record_free_function(node);
            this.record_cfg_attributes(&node.attrs);
            visit::visit_item_fn(this, node);
        });
    }

    fn visit_impl_item(&mut self, node: &'ast ImplItem) {
        if let ImplItem::Fn(function) = node {
            self.with_symbol(function.sig.ident.to_string(), |this| {
                this.record_cfg_attributes(&function.attrs);
                visit::visit_impl_item(this, node);
            });
        } else {
            visit::visit_impl_item(self, node);
        }
    }

    fn visit_item_impl(&mut self, node: &'ast ItemImpl) {
        self.record_inherent_methods(node);
        self.with_impl_type(
            super::ast_scan::type_shape_for_index(&node.self_ty),
            |this| {
                this.with_symbol("impl".to_string(), |this| {
                    this.record_cfg_attributes(&node.attrs);
                    visit::visit_item_impl(this, node);
                });
            },
        );
    }

    fn visit_item_trait(&mut self, node: &'ast ItemTrait) {
        self.record_trait_methods(node);
        self.with_symbol(node.ident.to_string(), |this| {
            visit::visit_item_trait(this, node)
        });
    }

    fn visit_expr_call(&mut self, node: &'ast ExprCall) {
        if let Expr::Path(path) = node.func.as_ref() {
            self.record_app_action_construction(&path.path, node.span(), "call");
            self.record_call_target(&path.path, node.func.span());
        }
        visit::visit_expr_call(self, node);
    }

    fn visit_expr_struct(&mut self, node: &'ast ExprStruct) {
        self.record_app_action_construction(&node.path, node.span(), "struct");
        visit::visit_expr_struct(self, node);
    }

    fn visit_macro(&mut self, node: &'ast Macro) {
        self.record_app_action_macro(&node.path, node.span());
        visit::visit_macro(self, node);
    }

    fn visit_expr_method_call(&mut self, node: &'ast ExprMethodCall) {
        self.record_call_edge(&node.method.to_string(), None, node.method.span());
        visit::visit_expr_method_call(self, node);
    }
    };
}
