macro_rules! source_closure_visit_expressions {
    () => {
        fn visit_expr_match(&mut self, node: &'ast ExprMatch) {
            for (ordinal, arm) in node.arms.iter().enumerate() {
                let pattern = match &arm.pat {
                    Pat::Guard(guard) => {
                        self.record_branch_edge("match_guard", None, guard.guard.span());
                        guard.pat.as_ref()
                    }
                    pattern => pattern,
                };
                self.record_match_edge(ordinal, pattern.span());
                self.record_branch_edge("match_body", None, arm.body.span());
                if Self::is_dispatch_symbol(&self.current_symbol())
                    && self.dispatch_match_depth == 0
                {
                    let variants = app_action_pattern_variants(pattern);
                    if variants.is_empty() {
                        self.state.record_dispatch_fallthrough(
                            super::scan_state::DispatchFallthrough {
                                file: self.current_relative_path.to_string(),
                                dispatch_symbol: self.current_symbol(),
                                dispatch_span: super::ast_resolution::span_to_text(
                                    self.current_relative_path,
                                    &node.span(),
                                ),
                                arm_span: super::ast_resolution::span_to_text(
                                    self.current_relative_path,
                                    &arm.span(),
                                ),
                            },
                        );
                    } else {
                        let (handler_calls, handler_call_facts, unresolved_reasons) =
                            direct_handler_calls(
                                &arm.body,
                                self.current_relative_path,
                                self.katana_root,
                                self.current_path,
                                self.lexical_resolver.as_ref(),
                            );
                        for (variant, variant_span) in variants {
                            self.state
                                .record_dispatch_arm(super::scan_state::DispatchArm {
                                    file: self.current_relative_path.to_string(),
                                    dispatch_symbol: self.current_symbol(),
                                    dispatch_span: super::ast_resolution::span_to_text(
                                        self.current_relative_path,
                                        &node.span(),
                                    ),
                                    action_variant: variant,
                                    variant_pattern_span: super::ast_resolution::span_to_text(
                                        self.current_relative_path,
                                        &variant_span,
                                    ),
                                    arm_span: super::ast_resolution::span_to_text(
                                        self.current_relative_path,
                                        &arm.span(),
                                    ),
                                    receiver_type: self.current_impl_type(),
                                    handler_calls: handler_calls.clone(),
                                    handler_call_facts: handler_call_facts.clone(),
                                    unresolved_reasons: unresolved_reasons.clone(),
                                });
                        }
                    }
                }
            }
            self.dispatch_match_depth += 1;
            visit::visit_expr_match(self, node);
            self.dispatch_match_depth -= 1;
        }

        fn visit_expr_if(&mut self, node: &'ast ExprIf) {
            self.record_branch_edge("if", None, node.cond.span());
            self.record_branch_edge("if_then", None, node.then_branch.span());
            if let Some((_, else_branch)) = &node.else_branch {
                self.record_branch_edge("if_else", None, else_branch.span());
            }
            let candidates = input_candidates_for_condition(
                node.cond.as_ref(),
                self.current_relative_path,
                node.cond.span(),
            );
            self.visit_expr(node.cond.as_ref());
            self.with_input_candidates(candidates, |this| {
                visit::visit_block(this, &node.then_branch);
                if let Some((_, else_branch)) = &node.else_branch {
                    this.visit_expr(else_branch);
                }
            });
        }

        fn visit_expr_closure(&mut self, node: &'ast ExprClosure) {
            let candidate = super::scan_state::InputOriginCandidate {
                kind: "callback_closure_boundary".to_string(),
                condition_syntax: "closure".to_string(),
                condition_span: super::ast_resolution::span_to_text(
                    self.current_relative_path,
                    &node.span(),
                ),
                boundary_syntax: "closure".to_string(),
                boundary_span: super::ast_resolution::span_to_text(
                    self.current_relative_path,
                    &node.span(),
                ),
                unresolved_reason: Some(
                    "callback or closure boundary does not prove input provenance".to_string(),
                ),
            };
            self.with_input_candidate(candidate, |this| {
                visit::visit_expr_closure(this, node);
            });
        }

        fn visit_expr_path(&mut self, node: &'ast ExprPath) {
            if !self.call_was_recorded(node.path.span()) {
                if !self.in_pattern() {
                    self.record_app_action_construction(&node.path, node.span(), "path");
                }
                let first = node
                    .path
                    .segments
                    .first()
                    .map(|segment| segment.ident.to_string());
                if first.as_deref().is_some_and(|first| {
                    matches!(first, "crate" | "self" | "super")
                        || self.has_import_alias(first)
                        || node.path.segments.len() > 1
                }) {
                    self.record_call_target(&node.path, node.path.span());
                }
            }
            visit::visit_expr_path(self, node);
        }

        fn visit_pat(&mut self, node: &'ast Pat) {
            self.with_pattern(|this| {
                visit::visit_pat(this, node);
            });
        }
    };
}
