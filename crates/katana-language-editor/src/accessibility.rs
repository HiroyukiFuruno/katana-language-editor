use crate::EditorResult;
use serde::{Deserialize, Serialize};

#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MotionPreference {
    NoPreference,
    Reduced,
}

#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AccessibilityConfig {
    pub role: String,
    pub label: String,
    pub motion: MotionPreference,
}

impl AccessibilityConfig {
    pub fn new(
        role: impl Into<String>,
        label: impl Into<String>,
        motion: MotionPreference,
    ) -> Self {
        Self {
            role: role.into(),
            label: label.into(),
            motion,
        }
    }
}

pub trait EditorAccessibility {
    fn set_accessibility(&mut self, config: AccessibilityConfig) -> EditorResult<()>;
}
