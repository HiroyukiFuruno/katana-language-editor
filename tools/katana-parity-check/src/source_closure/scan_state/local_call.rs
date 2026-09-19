#[derive(Clone, Debug)]
pub struct HandlerCallFact {
    pub syntax: String,
    pub kind: String,
    pub receiver_shape: String,
    pub method: Option<String>,
    pub span: String,
    pub path_resolution: Option<String>,
    pub path_target: Option<String>,
}
