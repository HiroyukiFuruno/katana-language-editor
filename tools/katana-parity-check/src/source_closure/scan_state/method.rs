use super::branch::{BranchOutcomeCandidate, HandlerBodyFact};
use super::continuation::ContinuationCallFact;
use super::terminal::TerminalEffectCandidate;

#[derive(Clone, Debug)]
pub struct MethodDefinition {
    pub file: String,
    pub implementation: String,
    pub method: String,
    pub receiver_shape: String,
    pub method_span: String,
    pub source_span: String,
    pub inherent: bool,
    pub receiver_type: String,
    pub body_facts: Vec<HandlerBodyFact>,
    pub branch_outcome_candidates: Vec<BranchOutcomeCandidate>,
    pub terminal_effect_candidates: Vec<TerminalEffectCandidate>,
    pub continuation_calls: Vec<ContinuationCallFact>,
    pub body_unresolved: Vec<String>,
}

#[derive(Clone, Debug)]
pub struct FreeFunctionDefinition {
    pub file: String,
    pub symbol: String,
    pub function: String,
    pub function_span: String,
    pub source_span: String,
    pub direct_path_calls: Vec<String>,
}
