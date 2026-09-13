#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ContextMenuRoute {
    SecondaryPointer,
    ShiftF10,
    AccessKitInvoke,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ContextMenuCaseDescriptor {
    pub path_digest: String,
    pub parent_path_digest: String,
    pub source_span_digest: String,
    pub enabled_condition_digest: String,
    pub locale_label_digests: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ContextMenuManifest {
    pub katana_revision: String,
    pub profile_fingerprint: String,
    pub source_closure_fingerprint: String,
    pub surface_source_span_digest: String,
    pub surface_role: String,
    pub menu_path_digest: String,
    cases: Vec<ContextMenuCaseDescriptor>,
}
pub struct ContextMenuManifestLoader;
#[derive(Debug, Eq, PartialEq)]
pub enum ContextMenuManifestError {
    Unavailable,
    Malformed,
    Invalid,
}
