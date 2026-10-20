#[cfg(target_os = "macos")]
#[path = "ax_target_locator_mac.rs"]
pub(crate) mod mac;
#[path = "ax_target_locator_selector.rs"]
mod selector;
#[path = "ax_target_locator_types.rs"]
mod types;

pub use types::{AxTargetLocator, AxTargetProof, AxTargetSelectionError};
