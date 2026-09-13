//! Source-derived context-menu inventory for the fixed-host runner.
//!
//! This module deliberately does not dispatch KatanA actions. The fixed host
//! currently has no public bootstrap that opens a document and exposes the
//! editor surface, so claiming a host effect here would be false evidence.

include!("context_menu_input/preamble.rs");
include!("context_menu_input/types.rs");
include!("context_menu_input/logic.rs");
