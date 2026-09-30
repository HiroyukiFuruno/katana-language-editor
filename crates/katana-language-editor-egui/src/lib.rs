//! katana-language-editor-egui: egui implementation of LanguageEditor.
//!
//! egui host boundary for the KUC retained editor root.

pub mod host_projection_provider;
pub mod kuc_root_binding;
pub mod root_editor;

#[cfg(feature = "storybook-artifacts")]
pub use host_projection_provider::HostProjectionArtifactResult;
pub use host_projection_provider::{
    HostProjectionBinding, HostProjectionBindingError, HostProjectionFrameResult,
    HostProjectionProvider, HostProjectionProviderError,
};
pub use kuc_root_binding::{KucRootBinding, KucRootBindingError, KucRootBindingReceipt};
#[cfg(feature = "storybook-artifacts")]
pub use root_editor::EguiTextCommandSurfaceEditorArtifactResult;
pub use root_editor::{
    EguiTextCommandSurfaceEditor, EguiTextCommandSurfaceEditorError,
    EguiTextCommandSurfaceEditorResult,
};

#[cfg(test)]
mod kuc_root_binding_tests;
