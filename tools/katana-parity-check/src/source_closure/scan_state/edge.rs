#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ControlFlowKind {
    Cfg,
    If,
    IfThen,
    IfElse,
    Match,
    MatchGuard,
    MatchBody,
    LetElse,
    LetElseBody,
    LogicalAnd,
    LogicalOr,
    For,
    ForBody,
    While,
    WhileBody,
    Loop,
    LoopBody,
    Return,
    Break,
    Continue,
    Try,
}

impl ControlFlowKind {
    #[cfg(test)]
    pub(super) const fn all() -> &'static [Self] {
        &[
            Self::Cfg,
            Self::If,
            Self::IfThen,
            Self::IfElse,
            Self::Match,
            Self::MatchGuard,
            Self::MatchBody,
            Self::LetElse,
            Self::LetElseBody,
            Self::LogicalAnd,
            Self::LogicalOr,
            Self::For,
            Self::ForBody,
            Self::While,
            Self::WhileBody,
            Self::Loop,
            Self::LoopBody,
            Self::Return,
            Self::Break,
            Self::Continue,
            Self::Try,
        ]
    }

    pub(super) const fn as_str(self) -> &'static str {
        match self {
            Self::Cfg => "cfg",
            Self::If => "if",
            Self::IfThen => "if_then",
            Self::IfElse => "if_else",
            Self::Match => "match",
            Self::MatchGuard => "match_guard",
            Self::MatchBody => "match_body",
            Self::LetElse => "let_else",
            Self::LetElseBody => "let_else_body",
            Self::LogicalAnd => "logical_and",
            Self::LogicalOr => "logical_or",
            Self::For => "for",
            Self::ForBody => "for_body",
            Self::While => "while",
            Self::WhileBody => "while_body",
            Self::Loop => "loop",
            Self::LoopBody => "loop_body",
            Self::Return => "return",
            Self::Break => "break",
            Self::Continue => "continue",
            Self::Try => "try",
        }
    }

    pub fn parse(kind: &str) -> Option<Self> {
        match kind {
            "cfg" => Some(Self::Cfg),
            "if" => Some(Self::If),
            "if_then" => Some(Self::IfThen),
            "if_else" => Some(Self::IfElse),
            "match" => Some(Self::Match),
            "match_guard" => Some(Self::MatchGuard),
            "match_body" => Some(Self::MatchBody),
            "let_else" => Some(Self::LetElse),
            "let_else_body" => Some(Self::LetElseBody),
            "logical_and" => Some(Self::LogicalAnd),
            "logical_or" => Some(Self::LogicalOr),
            "for" => Some(Self::For),
            "for_body" => Some(Self::ForBody),
            "while" => Some(Self::While),
            "while_body" => Some(Self::WhileBody),
            "loop" => Some(Self::Loop),
            "loop_body" => Some(Self::LoopBody),
            "return" => Some(Self::Return),
            "break" => Some(Self::Break),
            "continue" => Some(Self::Continue),
            "try" => Some(Self::Try),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use proc_macro2::Span;

    use super::super::state::ScanState;
    use super::ControlFlowKind;

    #[test]
    fn known_control_flow_kinds_round_trip() {
        for kind in ControlFlowKind::all() {
            assert_eq!(ControlFlowKind::parse(kind.as_str()), Some(*kind));
        }
        assert_eq!(ControlFlowKind::parse("call"), None);
        assert_eq!(ControlFlowKind::parse("unknown26k"), None);
    }

    #[test]
    fn structural_edge_is_kept_without_becoming_unresolved() {
        let mut state = ScanState::default();

        let id = state.add_structural_edge(
            "src/lib.rs",
            ControlFlowKind::If,
            "run",
            Some("condition"),
            Span::call_site(),
        );

        assert_eq!(id, "edge:000000");
        assert_eq!(state.edges.len(), 1);
        assert!(state.unresolved_edges.is_empty());
        assert_eq!(state.edges[0].kind, "if");
        assert_eq!(state.edges[0].to_path, None);
        assert_eq!(state.edges[0].to_symbol.as_deref(), Some("condition"));
    }

    #[test]
    fn ordinary_edge_keeps_none_target_unresolved_even_for_control_flow_kind() {
        let mut state = ScanState::default();

        state.add_edge(
            "src/lib.rs",
            "if",
            "run",
            None,
            Some("condition"),
            Span::call_site(),
        );

        assert_eq!(state.edges.len(), 1);
        assert_eq!(state.unresolved_edges.len(), 1);
        assert_eq!(state.unresolved_edges[0].kind, "if");
    }

    #[test]
    fn ordinary_and_structural_edges_have_identical_facts_for_every_kind() {
        for kind in ControlFlowKind::all() {
            let mut ordinary = ScanState::default();
            let mut structural = ScanState::default();
            ordinary.add_edge(
                "src/lib.rs",
                kind.as_str(),
                "run",
                None,
                Some("label"),
                Span::call_site(),
            );
            structural.add_structural_edge(
                "src/lib.rs",
                *kind,
                "run",
                Some("label"),
                Span::call_site(),
            );

            let ordinary_edge = &ordinary.edges[0];
            let structural_edge = &structural.edges[0];
            assert_eq!(ordinary_edge.id, structural_edge.id);
            assert_eq!(ordinary_edge.from_file, structural_edge.from_file);
            assert_eq!(ordinary_edge.kind, structural_edge.kind);
            assert_eq!(ordinary_edge.from_symbol, structural_edge.from_symbol);
            assert_eq!(ordinary_edge.to_path, structural_edge.to_path);
            assert_eq!(ordinary_edge.to_symbol, structural_edge.to_symbol);
            assert_eq!(
                ordinary_edge.lexical_resolution,
                structural_edge.lexical_resolution
            );
            assert_eq!(ordinary_edge.span, structural_edge.span);
            assert_eq!(ordinary.unresolved_edges.len(), 1);
            assert!(structural.unresolved_edges.is_empty());
        }
    }
}
