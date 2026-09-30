use syn::visit::Visit;

pub(crate) struct AssociatedCallAnalysis;

impl AssociatedCallAnalysis {
    pub(crate) fn collect(block: &syn::Block, symbol: &str, allow_self: bool) -> Vec<String> {
        let mut visitor = AssociatedCallVisitor {
            symbol,
            allow_self,
            calls: Vec::new(),
        };
        visitor.visit_block(block);
        visitor.calls
    }

    pub(crate) fn collect_receiver_method_calls(
        block: &syn::Block,
        receiver: &str,
        method: &str,
    ) -> Vec<String> {
        let mut visitor = ReceiverMethodCallVisitor {
            receiver,
            method,
            calls: Vec::new(),
        };
        visitor.visit_block(block);
        visitor.calls
    }
}

struct AssociatedCallVisitor<'a> {
    symbol: &'a str,
    allow_self: bool,
    calls: Vec<String>,
}

impl Visit<'_> for AssociatedCallVisitor<'_> {
    fn visit_expr_call(&mut self, call: &syn::ExprCall) {
        if let Some(method) = associated_call_method(call, self.symbol, self.allow_self)
            && !self.calls.contains(&method)
        {
            self.calls.push(method);
        }
        syn::visit::visit_expr_call(self, call);
    }
}

struct ReceiverMethodCallVisitor<'a> {
    receiver: &'a str,
    method: &'a str,
    calls: Vec<String>,
}

impl Visit<'_> for ReceiverMethodCallVisitor<'_> {
    fn visit_expr_method_call(&mut self, call: &syn::ExprMethodCall) {
        let receiver_matches = matches!(call.receiver.as_ref(),
            syn::Expr::Path(path)
                if path.qself.is_none()
                    && path.path.is_ident(self.receiver));
        if receiver_matches
            && call.method == self.method
            && !self.calls.contains(&self.method.to_string())
        {
            self.calls.push(self.method.to_string());
        }
        syn::visit::visit_expr_method_call(self, call);
    }
}

fn associated_call_method(call: &syn::ExprCall, symbol: &str, allow_self: bool) -> Option<String> {
    let syn::Expr::Path(path) = call.func.as_ref() else {
        return None;
    };
    if path.qself.is_some() || path.path.segments.len() != 2 {
        return None;
    }
    let receiver = path.path.segments.first()?.ident.to_string();
    if receiver == symbol || (allow_self && receiver == "Self") {
        Some(path.path.segments.last()?.ident.to_string())
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::AssociatedCallAnalysis;

    #[test]
    fn collects_direct_associated_call() -> Result<(), String> {
        let file = syn::parse_file("fn fixture() { DirectTextSurfaceScenario::run(); }")
            .map_err(|error| format!("fixture parses: {error}"))?;
        let function = file
            .items
            .first()
            .and_then(|item| match item {
                syn::Item::Fn(function) => Some(function),
                _ => None,
            })
            .ok_or_else(|| "fixture must contain a function".to_string())?;
        assert_eq!(
            AssociatedCallAnalysis::collect(&function.block, "DirectTextSurfaceScenario", false),
            ["run"]
        );
        Ok(())
    }

    #[test]
    fn collects_only_the_configured_public_root_receiver_call() -> Result<(), String> {
        let file = syn::parse_file(
            "fn fixture(host: &mut StorybookHost) { host.show_public_api_with_opaque_frame(); }",
        )
        .map_err(|error| format!("fixture parses: {error}"))?;
        let function = file
            .items
            .first()
            .and_then(|item| match item {
                syn::Item::Fn(function) => Some(function),
                _ => None,
            })
            .ok_or_else(|| "fixture must contain a function".to_string())?;
        assert_eq!(
            AssociatedCallAnalysis::collect_receiver_method_calls(
                &function.block,
                "host",
                "show_public_api_with_opaque_frame",
            ),
            ["show_public_api_with_opaque_frame"]
        );
        Ok(())
    }
}
