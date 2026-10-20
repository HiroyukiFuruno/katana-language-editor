use std::collections::BTreeSet;

pub(super) struct ReachableEvidence {
    pub(super) paths: BTreeSet<String>,
    pub(super) methods: BTreeSet<String>,
    pub(super) fields: BTreeSet<String>,
    pub(super) string_literals: BTreeSet<String>,
    pub(super) integer_literals: BTreeSet<String>,
    pub(super) binary_operators: BTreeSet<&'static str>,
    macro_tokens: BTreeSet<String>,
}

impl ReachableEvidence {
    pub(super) fn collect(file: &syn::File, test: &syn::ItemFn) -> Self {
        let functions = file
            .items
            .iter()
            .filter_map(|item| match item {
                syn::Item::Fn(function) => Some((function.sig.ident.to_string(), function)),
                _ => None,
            })
            .collect::<Vec<_>>();
        let mut evidence = Self::empty();
        let mut visited = BTreeSet::new();
        Self::collect_function(test, &functions, &mut visited, &mut evidence);
        evidence
    }

    pub(super) fn from_block(block: &syn::Block) -> Self {
        let mut evidence = Self::empty();
        syn::visit::Visit::visit_block(&mut evidence, block);
        evidence
    }

    pub(super) fn path_starts_with(&self, expected: &str) -> bool {
        self.paths
            .iter()
            .any(|path| path == expected || path.starts_with(&format!("{expected}::")))
            || self
                .macro_tokens
                .iter()
                .any(|tokens| tokens.contains(&compact(expected)))
    }

    pub(super) fn contains_text(&self, expected: &str) -> bool {
        self.string_literals
            .iter()
            .any(|literal| literal.contains(expected))
            || self
                .macro_tokens
                .iter()
                .any(|tokens| tokens.contains(&compact(expected)))
    }

    fn empty() -> Self {
        Self {
            paths: BTreeSet::new(),
            methods: BTreeSet::new(),
            fields: BTreeSet::new(),
            string_literals: BTreeSet::new(),
            integer_literals: BTreeSet::new(),
            binary_operators: BTreeSet::new(),
            macro_tokens: BTreeSet::new(),
        }
    }

    fn collect_function(
        function: &syn::ItemFn,
        functions: &[(String, &syn::ItemFn)],
        visited: &mut BTreeSet<String>,
        evidence: &mut Self,
    ) {
        if !visited.insert(function.sig.ident.to_string()) {
            return;
        }
        syn::visit::Visit::visit_item_fn(evidence, function);
        for call in LocalFunctionCalls::collect(&function.block) {
            if let Some((_, callee)) = functions.iter().find(|(name, _)| name == &call) {
                Self::collect_function(callee, functions, visited, evidence);
            }
        }
    }
}

impl<'ast> syn::visit::Visit<'ast> for ReachableEvidence {
    fn visit_expr_path(&mut self, node: &'ast syn::ExprPath) {
        self.paths.insert(path_string(&node.path));
        syn::visit::visit_expr_path(self, node);
    }

    fn visit_type_path(&mut self, node: &'ast syn::TypePath) {
        self.paths.insert(path_string(&node.path));
        syn::visit::visit_type_path(self, node);
    }

    fn visit_expr_method_call(&mut self, node: &'ast syn::ExprMethodCall) {
        self.methods.insert(node.method.to_string());
        syn::visit::visit_expr_method_call(self, node);
    }

    fn visit_expr_field(&mut self, node: &'ast syn::ExprField) {
        self.fields.insert(member_name(&node.member));
        syn::visit::visit_expr_field(self, node);
    }

    fn visit_expr_lit(&mut self, node: &'ast syn::ExprLit) {
        match &node.lit {
            syn::Lit::Str(value) => {
                self.string_literals.insert(value.value());
            }
            syn::Lit::Int(value) => {
                self.integer_literals
                    .insert(value.base10_digits().to_string());
            }
            _ => {}
        }
        syn::visit::visit_expr_lit(self, node);
    }

    fn visit_expr_binary(&mut self, node: &'ast syn::ExprBinary) {
        self.binary_operators.insert(binary_operator(&node.op));
        syn::visit::visit_expr_binary(self, node);
    }

    fn visit_macro(&mut self, node: &'ast syn::Macro) {
        self.macro_tokens.insert(compact(&node.tokens.to_string()));
        syn::visit::visit_macro(self, node);
    }
}

struct LocalFunctionCalls {
    calls: BTreeSet<String>,
}

impl LocalFunctionCalls {
    fn collect(block: &syn::Block) -> BTreeSet<String> {
        let mut visitor = Self {
            calls: BTreeSet::new(),
        };
        syn::visit::Visit::visit_block(&mut visitor, block);
        visitor.calls
    }
}

impl<'ast> syn::visit::Visit<'ast> for LocalFunctionCalls {
    fn visit_expr_call(&mut self, node: &'ast syn::ExprCall) {
        if let syn::Expr::Path(path) = node.func.as_ref()
            && path.qself.is_none()
            && path.path.segments.len() == 1
        {
            self.calls.insert(path.path.segments[0].ident.to_string());
        }
        syn::visit::visit_expr_call(self, node);
    }
}

fn path_string(path: &syn::Path) -> String {
    path.segments
        .iter()
        .map(|segment| segment.ident.to_string())
        .collect::<Vec<_>>()
        .join("::")
}

fn member_name(member: &syn::Member) -> String {
    match member {
        syn::Member::Named(identifier) => identifier.to_string(),
        syn::Member::Unnamed(index) => index.index.to_string(),
    }
}

fn compact(source: &str) -> String {
    source
        .chars()
        .filter(|character| !character.is_whitespace())
        .collect()
}

fn binary_operator(operator: &syn::BinOp) -> &'static str {
    match operator {
        syn::BinOp::Eq(_) => "==",
        syn::BinOp::Ne(_) => "!=",
        syn::BinOp::Lt(_) => "<",
        syn::BinOp::Le(_) => "<=",
        syn::BinOp::Gt(_) => ">",
        syn::BinOp::Ge(_) => ">=",
        syn::BinOp::And(_) => "&&",
        syn::BinOp::Or(_) => "||",
        _ => "other",
    }
}
