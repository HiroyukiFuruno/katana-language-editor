use crate::native_ax_observation::{
    NativeAxObservation, NativeAxObservationError, NativeAxSourceContract,
    NativeAxWorkspaceObservation,
};
use crate::physical_bootstrap_types::{
    AxApplicationElement, AxApplicationElementError, KatanAChild,
};

impl KatanAChild {
    pub fn observe_ax_application_element(
        &self,
    ) -> Result<AxApplicationElement, AxApplicationElementError> {
        AxApplicationElement::from_observation_pid(self.process_id())
    }
}

#[cfg(target_os = "macos")]
impl AxApplicationElement {
    pub fn observe_workspace_frame(
        &self,
        source: &NativeAxSourceContract,
        workspace_basename: &str,
    ) -> Result<NativeAxWorkspaceObservation, NativeAxObservationError> {
        use objc2_application_services::AXError;
        use std::ptr::NonNull;
        let mut pid = 0;
        let status = unsafe { self.element.pid(NonNull::from(&mut pid)) };
        if status != AXError::Success {
            return Err(NativeAxObservationError::AccessibilityUnavailable);
        }
        let digest = <sha2::Sha256 as sha2::Digest>::digest(format!("pid:{pid}").as_bytes());
        NativeAxObservation::observe(self, source, digest.into(), workspace_basename)
    }
}

#[cfg(not(target_os = "macos"))]
impl AxApplicationElement {
    pub fn observe_workspace_frame(
        &self,
        _source: &NativeAxSourceContract,
        _workspace_basename: &str,
    ) -> Result<NativeAxWorkspaceObservation, NativeAxObservationError> {
        Err(NativeAxObservationError::UnsupportedPlatform)
    }
}
