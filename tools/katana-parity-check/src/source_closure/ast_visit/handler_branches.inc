macro_rules! handler_body_visit_branches {
    () => {
    fn visit_expr_if(&mut self, node: &'ast ExprIf) {
        self.visit_expr(&node.cond);
        let cfg_facts = node
            .attrs
            .iter()
            .filter_map(|attr| {
                super::ast_resolution::cfg_predicate(attr)
                    .map(|predicate| self.make_fact("cfg", predicate, attr.span()))
            })
            .collect::<Vec<_>>();
        self.branch_stack.extend(cfg_facts.iter().cloned());
        let branch_fact = self.make_fact("if_branch", expr_to_syntax(&node.cond), node.span());
        self.facts.push(branch_fact.clone());
        self.candidate("if_branch", branch_fact.syntax.clone(), node.span(), false);
        self.branch_stack.push(branch_fact);
        let then_shape = self.branch_body_shape(&node.then_branch);
        self.record_outcome(
            "if_then",
            format!("if {} then", expr_to_syntax(&node.cond)),
            node.span(),
            node.then_branch.span(),
            then_shape,
            |this| this.visit_block(&node.then_branch),
        );
        if let Some((_, else_branch)) = &node.else_branch {
            let (body_span, body_shape) = match else_branch.as_ref() {
                Expr::Block(block) => (block.span(), self.branch_body_shape(&block.block)),
                Expr::Return(_) => (else_branch.span(), "early_return"),
                _ => (else_branch.span(), "fallthrough"),
            };
            self.record_outcome(
                "if_else",
                format!("if {} else", expr_to_syntax(&node.cond)),
                node.span(),
                body_span,
                body_shape,
                |this| this.visit_expr(else_branch),
            );
        } else {
            self.branch_outcome_candidates
                .push(super::scan_state::BranchOutcomeCandidate {
                    kind: "if_absent_else".to_string(),
                    syntax: format!("if {} else <absent>", expr_to_syntax(&node.cond)),
                    span: super::ast_resolution::span_to_text(&self.file, &node.span()),
                    body_span: super::ast_resolution::span_to_text(&self.file, &node.span()),
                    body_shape: "absent".to_string(),
                    enclosing_branch_facts: self.branch_stack.clone(),
                    enclosing_outcome_syntax: self.outcome_stack.clone(),
                    terminal_effect_candidates: Vec::new(),
                    unresolved: vec![
                        "absent else has no source body span".to_string(),
                        "branch outcome semantic effect remains unresolved".to_string(),
                        "profile, visibility, availability, action origin, and host effect are not classified".to_string(),
                    ],
                });
        }
        self.branch_stack.pop();
        for _ in cfg_facts {
            self.branch_stack.pop();
        }
    }

    fn visit_expr_match(&mut self, node: &'ast ExprMatch) {
        self.visit_expr(&node.expr);
        let cfg_facts = node
            .attrs
            .iter()
            .filter_map(|attr| {
                super::ast_resolution::cfg_predicate(attr)
                    .map(|predicate| self.make_fact("cfg", predicate, attr.span()))
            })
            .collect::<Vec<_>>();
        self.branch_stack.extend(cfg_facts.iter().cloned());
        let branch_fact = self.make_fact("match_branch", expr_to_syntax(&node.expr), node.span());
        self.facts.push(branch_fact.clone());
        self.candidate(
            "match_branch",
            branch_fact.syntax.clone(),
            node.span(),
            false,
        );
        self.branch_stack.push(branch_fact);
        for arm in &node.arms {
            let (pattern, guard, has_guard) = match &arm.pat {
                syn::Pat::Guard(guard) => (
                    pat_to_syntax(&guard.pat),
                    format!(" if {}", expr_to_syntax(&guard.guard)),
                    true,
                ),
                pattern => (pat_to_syntax(pattern), String::new(), false),
            };
            let pattern_unresolved = pattern.contains("<pattern-unresolved>");
            let body_shape = match arm.body.as_ref() {
                Expr::Block(block) => self.branch_body_shape(&block.block),
                Expr::Return(_) => "early_return",
                _ => "fallthrough",
            };
            self.record_outcome(
                "match_arm",
                format!("{pattern}{guard} => {}", expr_to_syntax(&arm.body)),
                arm.span(),
                arm.body.span(),
                body_shape,
                |this| this.visit_expr(&arm.body),
            );
            if let Some(candidate) = self.branch_outcome_candidates.last_mut() {
                if has_guard {
                    candidate.unresolved.push(
                        "match guard parse and semantic resolution remains unresolved".to_string(),
                    );
                }
                if pattern_unresolved {
                    candidate
                        .unresolved
                        .push("match pattern syntax is partially unresolved".to_string());
                }
                candidate.unresolved.sort();
                candidate.unresolved.dedup();
            }
        }
        self.branch_stack.pop();
        for _ in cfg_facts {
            self.branch_stack.pop();
        }
    }

    fn visit_expr_closure(&mut self, node: &'ast ExprClosure) {
        self.fact("closure_boundary", "closure".to_string(), node.span());
        self.candidate(
            "closure_boundary",
            "closure".to_string(),
            node.span(),
            false,
        );
        self.unresolved.push("closure boundary".to_string());
        self.boundary_stack.push("closure".to_string());
        visit::visit_expr_closure(self, node);
        self.boundary_stack.pop();
    }

    fn visit_macro(&mut self, node: &'ast Macro) {
        self.fact("macro_boundary", path_to_syntax(&node.path), node.span());
        self.candidate(
            "macro_boundary",
            path_to_syntax(&node.path),
            node.span(),
            false,
        );
        self.unresolved.push("macro boundary".to_string());
        self.boundary_stack.push("macro".to_string());
        visit::visit_macro(self, node);
        self.boundary_stack.pop();
    }
    };
}
