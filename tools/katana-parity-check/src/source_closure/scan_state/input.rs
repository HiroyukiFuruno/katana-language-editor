#[derive(Clone, Debug)]
pub struct InputOriginCandidate {
    pub kind: String,
    pub condition_syntax: String,
    pub condition_span: String,
    pub boundary_syntax: String,
    pub boundary_span: String,
    pub unresolved_reason: Option<String>,
}
