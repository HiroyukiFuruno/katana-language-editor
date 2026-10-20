use crate::ax_target_locator::{AxTargetLocator, AxTargetProof, AxTargetSelectionError};
use crate::physical_bootstrap_types::{
    AxApplicationElement, AxApplicationElementError, AxPreflight, AxPreflightError, KatanAChild,
};
use std::fmt;

impl AxApplicationElement {
    fn from_pid(pid: u32) -> Result<Self, AxApplicationElementError> {
        platform::from_pid(pid)
    }

    pub(crate) fn from_observation_pid(pid: u32) -> Result<Self, AxApplicationElementError> {
        platform::from_pid(pid)
    }

    pub fn locate(
        &self,
        locator: &AxTargetLocator,
    ) -> Result<AxTargetProof, AxTargetSelectionError> {
        platform::locate(self, locator)
    }

    pub fn click(
        &self,
        locator: &AxTargetLocator,
    ) -> Result<(), crate::physical_bootstrap_types::AxClickError> {
        crate::physical_ax_click::AxClicker::click(self, locator)
    }
}

impl fmt::Display for AxPreflightError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::UnsupportedPlatform => {
                "macOS Accessibility preflight is unsupported on this platform"
            }
            Self::AccessibilityNotTrusted => "macOS Accessibility authorization is not trusted",
            Self::InputAuthorizationNotTrusted => "macOS input-event authorization is not trusted",
        })
    }
}

impl std::error::Error for AxPreflightError {}

impl fmt::Display for AxApplicationElementError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::UnsupportedPlatform => "AX application elements are unsupported on this platform",
            Self::AccessibilityNotTrusted => "macOS Accessibility authorization is not trusted",
            Self::InputAuthorizationNotTrusted => "macOS input-event authorization is not trusted",
            Self::NullElement => "macOS returned no AX application element",
            Self::Unreachable => "KatanA AX application element is not reachable",
            Self::SystemFailure => "macOS failed to create the AX application element",
        })
    }
}

impl std::error::Error for AxApplicationElementError {}

impl fmt::Display for crate::physical_bootstrap_types::AxClickError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::UnsupportedPlatform => "AX click is unsupported on this platform",
            Self::AccessibilityNotTrusted => "macOS Accessibility authorization is not trusted",
            Self::TargetSelection => "current AX snapshot did not yield one valid target",
            Self::TargetRevalidationFailed => "current AX target identity could not be revalidated",
            Self::BoundsMissing => "current AX target has no position or size",
            Self::BoundsTypeMismatch => "current AX target bounds have an unexpected type",
            Self::BoundsValueInvalid => "current AX target bounds are not finite and positive",
            Self::EventAuthorizationNotTrusted => "macOS input event authorization is not trusted",
            Self::EventSourceCreationFailed => "macOS failed to create an input event source",
            Self::EventCreationFailed => "macOS failed to create a mouse event",
            Self::EventPostFailed => "macOS failed to post a mouse event",
            Self::SystemFailure => "macOS input event system failed",
        })
    }
}

impl std::error::Error for crate::physical_bootstrap_types::AxClickError {}

impl AxPreflight {
    pub fn check() -> Result<(), AxPreflightError> {
        platform::check()
    }
}

impl KatanAChild {
    pub fn ax_application_element(
        &self,
    ) -> Result<AxApplicationElement, AxApplicationElementError> {
        AxPreflight::check().map_err(|error| match error {
            AxPreflightError::UnsupportedPlatform => AxApplicationElementError::UnsupportedPlatform,
            AxPreflightError::AccessibilityNotTrusted => {
                AxApplicationElementError::AccessibilityNotTrusted
            }
            AxPreflightError::InputAuthorizationNotTrusted => {
                AxApplicationElementError::InputAuthorizationNotTrusted
            }
        })?;
        AxApplicationElement::from_pid(self.process_id())
    }

    pub fn locate_ax_target(
        &self,
        locator: &AxTargetLocator,
    ) -> Result<AxTargetProof, AxTargetSelectionError> {
        self.ax_application_element()
            .map_err(|_| AxTargetSelectionError::AccessibilityFailure)?
            .locate(locator)
    }
}

#[cfg(target_os = "macos")]
mod platform {
    use super::{AxApplicationElement, AxApplicationElementError, AxPreflightError};
    use crate::ax_target_locator::{AxTargetLocator, AxTargetProof, AxTargetSelectionError};
    use objc2_application_services::{AXError, AXIsProcessTrusted, AXUIElement};
    use std::ffi::c_int;
    use std::ptr::NonNull;

    unsafe extern "C-unwind" {
        fn AXUIElementCreateApplication(pid: c_int) -> Option<NonNull<AXUIElement>>;
    }

    pub(super) fn check() -> Result<(), AxPreflightError> {
        if unsafe { !AXIsProcessTrusted() } {
            return Err(AxPreflightError::AccessibilityNotTrusted);
        }
        if !objc2_core_graphics::CGPreflightPostEventAccess() {
            return Err(AxPreflightError::InputAuthorizationNotTrusted);
        }
        Ok(())
    }

    pub(super) fn from_pid(pid: u32) -> Result<AxApplicationElement, AxApplicationElementError> {
        let element = unsafe { AXUIElementCreateApplication(pid as c_int) }
            .ok_or(AxApplicationElementError::NullElement)?;
        let element = unsafe { objc2_core_foundation::CFRetained::from_raw(element) };
        let application = AxApplicationElement { element };
        let mut observed_pid = 0 as c_int;
        let status = unsafe { application.element.pid(NonNull::from(&mut observed_pid)) };
        if status != AXError::Success {
            return Err(AxApplicationElementError::Unreachable);
        }
        if observed_pid != pid as c_int {
            return Err(AxApplicationElementError::SystemFailure);
        }
        Ok(application)
    }

    pub(super) fn locate(
        application: &AxApplicationElement,
        locator: &AxTargetLocator,
    ) -> Result<AxTargetProof, AxTargetSelectionError> {
        crate::ax_target_locator::mac::AxTargetTraversal::locate(&application.element, locator)
    }
}

#[cfg(not(target_os = "macos"))]
mod platform {
    use super::{AxApplicationElement, AxApplicationElementError, AxPreflightError};
    use crate::ax_target_locator::{AxTargetLocator, AxTargetProof, AxTargetSelectionError};

    pub(super) fn check() -> Result<(), AxPreflightError> {
        Err(AxPreflightError::UnsupportedPlatform)
    }
    pub(super) fn from_pid(_pid: u32) -> Result<AxApplicationElement, AxApplicationElementError> {
        Err(AxApplicationElementError::UnsupportedPlatform)
    }
    pub(super) fn locate(
        _application: &AxApplicationElement,
        _locator: &AxTargetLocator,
    ) -> Result<AxTargetProof, AxTargetSelectionError> {
        Err(AxTargetSelectionError::UnsupportedPlatform)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::physical_ax_click::{AxBounds, validate_bounds};
    use crate::physical_bootstrap_types::AxClickError;
    use std::process::Command;

    #[test]
    fn non_macos_ax_preflight_is_typed_failure() {
        #[cfg(not(target_os = "macos"))]
        assert_eq!(
            AxPreflight::check(),
            Err(AxPreflightError::UnsupportedPlatform)
        );
    }

    #[test]
    fn non_macos_ax_application_element_is_typed_failure() {
        #[cfg(not(target_os = "macos"))]
        assert_eq!(
            platform::from_pid(1),
            Err(AxApplicationElementError::UnsupportedPlatform)
        );
    }

    #[test]
    fn ax_application_element_errors_do_not_include_process_or_raw_data() {
        let errors = [
            AxApplicationElementError::UnsupportedPlatform,
            AxApplicationElementError::AccessibilityNotTrusted,
            AxApplicationElementError::InputAuthorizationNotTrusted,
            AxApplicationElementError::NullElement,
            AxApplicationElementError::Unreachable,
            AxApplicationElementError::SystemFailure,
        ];
        let rendered = format!("{:?}", errors[4]);
        assert_eq!(rendered, "Unreachable");
        assert!(!rendered.contains("pid"));
        assert!(!rendered.contains("0x"));
    }

    #[test]
    fn child_ax_binding_is_private_and_does_not_require_real_preflight() {
        let process = Command::new("true").spawn().expect("test process");
        let child = KatanAChild { child: process };
        let _ = child.ax_application_element();
        #[cfg(not(target_os = "macos"))]
        let _ = AxApplicationElement {};
    }

    #[test]
    fn invalid_bounds_fail_closed_without_exposing_values() {
        for bounds in [
            AxBounds {
                x: 0.0,
                y: 0.0,
                width: 0.0,
                height: 10.0,
            },
            AxBounds {
                x: 0.0,
                y: 0.0,
                width: 10.0,
                height: f64::NAN,
            },
            AxBounds {
                x: f64::INFINITY,
                y: 0.0,
                width: 10.0,
                height: 10.0,
            },
        ] {
            assert_eq!(
                validate_bounds(bounds),
                Err(AxClickError::BoundsValueInvalid)
            );
        }
        let rendered = format!("{:?}", AxClickError::BoundsValueInvalid);
        assert!(!rendered.contains("10"));
        assert!(!rendered.contains("0x"));
    }
}
