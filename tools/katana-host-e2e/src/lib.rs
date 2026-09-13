pub(crate) mod ax_target_locator;
mod bootstrap;
mod context_menu_input;
mod context_menu_manifest;
mod editor_candidate_audit;
#[cfg(feature = "fixed-host")]
mod fixed_host_session;
mod fixed_source_harness;
mod host_target_locator;
mod native_ax_observation;
mod physical_ax_click;
mod physical_ax_observer;
mod physical_bootstrap;
mod physical_bootstrap_ax;
mod physical_bootstrap_request;
mod physical_bootstrap_types;
mod physical_frame_capture;
mod physical_observation_bridge;
mod physical_run_layout;
mod physical_workspace_config;
mod source_derived_native_target;
mod system;

pub use ax_target_locator::{AxTargetLocator, AxTargetProof, AxTargetSelectionError};
pub use bootstrap::{BootstrapError, HostBootstrap};
pub use context_menu_input::{
    ContextMenuHostE2eError, ContextMenuOpenRoute, ContextMenuSourceAudit,
    ContextMenuSourceAuditError, ContextMenuSourceCase, ContextMenuSourceInventory,
};
pub use context_menu_manifest::{
    ContextMenuCaseDescriptor, ContextMenuManifest, ContextMenuManifestError,
    ContextMenuManifestLoader, ContextMenuRoute,
};
pub use editor_candidate_audit::{
    EditorCandidateAudit, EditorCandidateAuditError, EditorCandidateFacts,
    EditorCandidateObservation, EditorCandidateState, EditorSourceContract,
};
#[cfg(feature = "fixed-host")]
pub use fixed_host_session::{FixedHostSession, InitialFrameError};
pub use fixed_source_harness::{
    FixedSourceHarness, FixedSourceHarnessBuilder, FixedSourceHarnessError,
    FixedSourceHarnessMetadata, FixedSourceHarnessRequest,
};
pub use host_target_locator::{HostTargetLocator, HostTargetSelectionError};
pub use native_ax_observation::{
    NativeAxEditorCandidate, NativeAxObservationError, NativeAxRole, NativeAxSourceContract,
    NativeAxWorkspaceObservation,
};
pub use physical_ax_observer::{AxObserverError, AxWindowCreatedObserver};
pub use physical_bootstrap_types::{
    AxClickError, AxPreflight, AxPreflightError, ChildLaunchError, KatanAChild, KatanACommand,
    LaunchRequest, RequestValidationError,
};
pub use physical_frame_capture::{
    AccessKitEvidence, AccessKitMode, FrameCaptureError, FrameEvidence, PhysicalFrameCapture,
};
pub use physical_run_layout::NativePhysicalRunLayout;
pub use source_derived_native_target::{SourceDerivedNativeTarget, SourceDerivedNativeTargetError};
