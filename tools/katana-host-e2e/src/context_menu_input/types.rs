#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ContextMenuOpenRoute {
    Secondary,
    ShiftF10,
    AccessKit,
}

impl ContextMenuOpenRoute {
    pub const ALL: [Self; 3] = [Self::Secondary, Self::ShiftF10, Self::AccessKit];

    pub const fn name(self) -> &'static str {
        match self {
            Self::Secondary => "secondary",
            Self::ShiftF10 => "shift-f10",
            Self::AccessKit => "accesskit",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ContextMenuSourceCase {
    pub source_label: &'static str,
    pub source_anchor: &'static str,
}

impl ContextMenuSourceCase {
    pub const fn new(source_label: &'static str, source_anchor: &'static str) -> Self {
        Self {
            source_label,
            source_anchor,
        }
    }
}

pub struct ContextMenuSourceInventory;

impl ContextMenuSourceInventory {
    pub const fn case_specs() -> [ContextMenuSourceCase; SOURCE_CASE_COUNT] {
        [
            ContextMenuSourceCase::new("Save", "AppAction::SaveDocument"),
            ContextMenuSourceCase::new("Format", "AppAction::FormatMarkdownFile"),
            ContextMenuSourceCase::new("Markdown authoring", "AppAction::AuthorMarkdown"),
            ContextMenuSourceCase::new("Code block", "CodeBlockMenuOps::show"),
            ContextMenuSourceCase::new("Image file", "AppAction::IngestImageFile"),
            ContextMenuSourceCase::new("Clipboard image", "AppAction::IngestClipboardImage"),
        ]
    }

    pub fn assert_root_order_and_format_visibility(
        audit: &ContextMenuSourceAudit,
    ) -> Result<(), ContextMenuSourceAuditError> {
        audit.assert_root_order_and_format_visibility()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ContextMenuSourceAudit {
    pub fixed_root: PathBuf,
    pub revision: String,
    source: String,
    ingest_source: String,
}

#[derive(Debug)]
pub enum ContextMenuSourceAuditError {
    MissingRoot(PathBuf),
    MissingSource(PathBuf),
    Git(String),
    RevisionMismatch {
        expected: &'static str,
        actual: String,
    },
    MissingAnchor(&'static str),
    InvalidRootOrder,
}

#[derive(Debug)]
pub enum ContextMenuHostE2eError {
    UnavailableBridge {
        fixed_revision: String,
        evidence: &'static str,
    },
}

impl fmt::Display for ContextMenuHostE2eError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnavailableBridge {
                fixed_revision,
                evidence,
            } => write!(
                f,
                "Context Menu host E2E bridge unavailable for fixed KatanA {fixed_revision}: {evidence}"
            ),
        }
    }
}

impl std::error::Error for ContextMenuHostE2eError {}

impl fmt::Display for ContextMenuSourceAuditError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingRoot(path) => {
                write!(f, "fixed KatanA root is missing: {}", path.display())
            }
            Self::MissingSource(path) => {
                write!(f, "fixed KatanA source is missing: {}", path.display())
            }
            Self::Git(error) => write!(f, "fixed KatanA revision could not be read: {error}"),
            Self::RevisionMismatch { expected, actual } => {
                write!(f, "fixed KatanA revision is {actual}, expected {expected}")
            }
            Self::MissingAnchor(anchor) => {
                write!(f, "fixed KatanA context-menu anchor is missing: {anchor}")
            }
            Self::InvalidRootOrder => f.write_str("fixed KatanA context-menu root order changed"),
        }
    }
}

impl std::error::Error for ContextMenuSourceAuditError {}
