use serde::{Deserialize, Serialize};

#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum EditorViewMode {
    PreviewOnly,
    Split,
    CodeOnly,
}

#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum EditorSplitDirection {
    Horizontal,
    Vertical,
}
