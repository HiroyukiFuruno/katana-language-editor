#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct KatanaSourceEvidence {
    pub(crate) path: &'static str,
    pub(crate) line: usize,
    pub(crate) marker: &'static str,
}
