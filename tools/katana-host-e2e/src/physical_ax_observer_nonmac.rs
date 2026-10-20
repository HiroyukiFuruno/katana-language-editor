use crate::{
    AxObserverError,
    physical_bootstrap_types::{AxApplicationElement, KatanAChild},
};

pub struct AxWindowCreatedObserver;

impl AxWindowCreatedObserver {
    pub fn register(
        _application: &AxApplicationElement,
        _child: &KatanAChild,
    ) -> Result<Self, AxObserverError> {
        Err(AxObserverError::UnsupportedPlatform)
    }

    pub fn register_native_dialog(
        _application: &AxApplicationElement,
        _child: &KatanAChild,
    ) -> Result<Self, AxObserverError> {
        Err(AxObserverError::UnsupportedPlatform)
    }

    pub fn wait_for_notification(self) -> Result<(), AxObserverError> {
        Err(AxObserverError::UnsupportedPlatform)
    }

    pub fn wait_for_existing_window_or_notification(
        self,
        _application: &AxApplicationElement,
    ) -> Result<(), AxObserverError> {
        Err(AxObserverError::UnsupportedPlatform)
    }
}
