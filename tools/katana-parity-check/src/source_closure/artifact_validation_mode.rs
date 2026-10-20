#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ArtifactValidationMode {
    Rc,
    KleRelease,
    Full,
}
