use super::input::InputOriginCandidate;

#[derive(Clone, Debug)]
pub struct ActionDefinition {
    pub file: String,
    pub symbol: String,
    pub enum_span: String,
    pub variant: String,
    pub variant_span: String,
}

#[derive(Clone, Debug)]
pub struct ActionConstruction {
    pub file: String,
    pub symbol: String,
    pub span: String,
    pub enum_name: String,
    pub variant: Option<String>,
    pub style: String,
    pub unresolved_reason: Option<String>,
    pub input_origin_candidates: Vec<InputOriginCandidate>,
}
