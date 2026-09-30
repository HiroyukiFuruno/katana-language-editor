use super::branch::HandlerBodyFact;
use super::terminal::TerminalEffectCandidate;

#[derive(Clone, Debug)]
pub struct ContinuationCallFact {
    pub method: String,
    pub syntax: String,
    pub span: String,
    pub enclosing_branch_facts: Vec<HandlerBodyFact>,
    pub boundary_kinds: Vec<String>,
}

pub type HandlerBodyIndex = (
    Vec<HandlerBodyFact>,
    Vec<super::branch::BranchOutcomeCandidate>,
    Vec<TerminalEffectCandidate>,
    Vec<ContinuationCallFact>,
    Vec<String>,
);
