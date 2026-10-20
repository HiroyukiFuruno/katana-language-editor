use super::local_call::HandlerCallFact;

#[derive(Clone, Debug)]
pub struct DispatchArm {
    pub file: String,
    pub dispatch_symbol: String,
    pub dispatch_span: String,
    pub action_variant: String,
    pub variant_pattern_span: String,
    pub arm_span: String,
    pub receiver_type: Option<String>,
    pub handler_calls: Vec<String>,
    pub handler_call_facts: Vec<HandlerCallFact>,
    pub unresolved_reasons: Vec<String>,
}

#[derive(Clone, Debug)]
pub struct HandlerBodyFact {
    pub kind: String,
    pub syntax: String,
    pub span: String,
}

#[derive(Clone, Debug)]
pub struct BranchOutcomeCandidate {
    pub kind: String,
    pub syntax: String,
    pub span: String,
    pub body_span: String,
    pub body_shape: String,
    pub enclosing_branch_facts: Vec<HandlerBodyFact>,
    pub enclosing_outcome_syntax: Vec<String>,
    pub terminal_effect_candidates: Vec<super::terminal::TerminalEffectCandidate>,
    pub unresolved: Vec<String>,
}

#[derive(Clone, Debug)]
pub struct DispatchFallthrough {
    pub file: String,
    pub dispatch_symbol: String,
    pub dispatch_span: String,
    pub arm_span: String,
}
