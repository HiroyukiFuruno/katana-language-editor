use super::branch::HandlerBodyFact;

#[derive(Clone, Debug)]
pub struct TerminalEffectCandidate {
    pub kind: String,
    pub syntax: String,
    pub span: String,
    pub enclosing_branch_facts: Vec<HandlerBodyFact>,
    pub terminal: bool,
}
