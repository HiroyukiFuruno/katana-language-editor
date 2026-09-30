use crate::physical_bootstrap_types::{AxApplicationElement, AxApplicationElementError};

impl AxApplicationElement {
    pub fn has_existing_window(&self) -> Result<bool, AxApplicationElementError> {
        platform::has_existing_window(self)
    }
}

#[cfg(target_os = "macos")]
mod platform {
    use super::{AxApplicationElement, AxApplicationElementError};
    use objc2_application_services::{AXError, AXUIElement};
    use objc2_core_foundation::{CFArray, CFRetained, CFString, CFType};
    use std::ptr::NonNull;

    pub(super) fn has_existing_window(
        application: &AxApplicationElement,
    ) -> Result<bool, AxApplicationElementError> {
        let attribute = CFString::from_str("AXWindows");
        let mut raw = std::ptr::null();
        let status = unsafe {
            application
                .element
                .copy_attribute_value(&attribute, NonNull::from(&mut raw))
        };
        if status != AXError::Success {
            return Err(AxApplicationElementError::Unreachable);
        }
        let raw = NonNull::new(raw as *mut CFType).ok_or(AxApplicationElementError::SystemFailure)?;
        let value = unsafe { CFRetained::from_raw(raw) };
        let windows: CFRetained<CFArray> = value
            .downcast()
            .map_err(|_| AxApplicationElementError::SystemFailure)?;
        let windows: CFRetained<CFArray<AXUIElement>> = unsafe { CFRetained::cast_unchecked(windows) };
        Ok(windows.len() > 0)
    }
}

#[cfg(not(target_os = "macos"))]
mod platform {
    use super::{AxApplicationElement, AxApplicationElementError};

    pub(super) fn has_existing_window(
        _application: &AxApplicationElement,
    ) -> Result<bool, AxApplicationElementError> {
        Err(AxApplicationElementError::UnsupportedPlatform)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn non_macos_existing_window_query_is_typed_failure() {
        #[cfg(not(target_os = "macos"))]
        assert_eq!(
            AxApplicationElement {}.has_existing_window(),
            Err(AxApplicationElementError::UnsupportedPlatform)
        );
    }
}
