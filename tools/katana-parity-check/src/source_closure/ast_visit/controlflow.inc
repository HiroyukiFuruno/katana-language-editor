macro_rules! source_closure_visit_controlflow {
    () => {
        fn visit_local(&mut self, node: &'ast Local) {
            if let Some(initializer) = &node.init
                && let Some((_, else_branch)) = &initializer.diverge
            {
                self.record_branch_edge("let_else", None, node.pat.span());
                self.record_branch_edge("let_else_body", None, else_branch.span());
            }
            visit::visit_local(self, node);
        }

        fn visit_expr_binary(&mut self, node: &'ast ExprBinary) {
            let kind = match node.op {
                syn::BinOp::And(_) => Some("logical_and"),
                syn::BinOp::Or(_) => Some("logical_or"),
                _ => None,
            };
            if let Some(kind) = kind {
                self.record_branch_edge(kind, None, node.span());
            }
            visit::visit_expr_binary(self, node);
        }

        fn visit_expr_for_loop(&mut self, node: &'ast ExprForLoop) {
            self.record_branch_edge("for", None, node.expr.span());
            self.record_branch_edge("for_body", None, node.body.span());
            visit::visit_expr_for_loop(self, node);
        }

        fn visit_expr_while(&mut self, node: &'ast ExprWhile) {
            self.record_branch_edge("while", None, node.cond.span());
            self.record_branch_edge("while_body", None, node.body.span());
            visit::visit_expr_while(self, node);
        }

        fn visit_expr_loop(&mut self, node: &'ast ExprLoop) {
            self.record_branch_edge("loop", None, node.loop_token.span);
            self.record_branch_edge("loop_body", None, node.body.span());
            visit::visit_expr_loop(self, node);
        }

        fn visit_expr_return(&mut self, node: &'ast ExprReturn) {
            self.record_branch_edge("return", None, node.span());
            visit::visit_expr_return(self, node);
        }

        fn visit_expr_break(&mut self, node: &'ast ExprBreak) {
            self.record_branch_edge("break", None, node.span());
            visit::visit_expr_break(self, node);
        }

        fn visit_expr_continue(&mut self, node: &'ast ExprContinue) {
            self.record_branch_edge("continue", None, node.span());
            visit::visit_expr_continue(self, node);
        }

        fn visit_expr_try(&mut self, node: &'ast ExprTry) {
            self.record_branch_edge("try", None, node.span());
            visit::visit_expr_try(self, node);
        }
    };
}
