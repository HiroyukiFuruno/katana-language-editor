#[cfg(target_os = "macos")]
#[path = "physical_ax_click_mac.rs"]
mod platform;
#[cfg(not(target_os = "macos"))]
#[path = "physical_ax_click_nonmac.rs"]
mod platform;

use crate::ax_target_locator::AxTargetLocator;
use crate::physical_bootstrap_types::{
    AxApplicationElement, AxClickError, AxPhysicalInput, AxPreflight, AxPreflightError,
};

pub(crate) struct AxClicker;

impl AxApplicationElement {
    pub fn secondary_pointer(&self, locator: &AxTargetLocator) -> Result<(), AxClickError> {
        AxClicker::input(self, locator, AxPhysicalInput::SecondaryPointer)
    }

    pub fn shift_f10(&self, locator: &AxTargetLocator) -> Result<(), AxClickError> {
        AxClicker::input(self, locator, AxPhysicalInput::ShiftF10)
    }

    pub fn accessibility_press(&self, locator: &AxTargetLocator) -> Result<(), AxClickError> {
        AxClicker::input(self, locator, AxPhysicalInput::AccessibilityPress)
    }
}

impl AxClicker {
    pub(crate) fn click(
        application: &AxApplicationElement,
        locator: &AxTargetLocator,
    ) -> Result<(), AxClickError> {
        AxPreflight::check().map_err(|error| match error {
            AxPreflightError::UnsupportedPlatform => AxClickError::UnsupportedPlatform,
            AxPreflightError::AccessibilityNotTrusted => AxClickError::AccessibilityNotTrusted,
            AxPreflightError::InputAuthorizationNotTrusted => {
                AxClickError::EventAuthorizationNotTrusted
            }
        })?;
        Self::input(application, locator, AxPhysicalInput::PrimaryPointer)
    }

    pub(crate) fn input(
        application: &AxApplicationElement,
        locator: &AxTargetLocator,
        input: AxPhysicalInput,
    ) -> Result<(), AxClickError> {
        AxPreflight::check().map_err(|error| match error {
            AxPreflightError::UnsupportedPlatform => AxClickError::UnsupportedPlatform,
            AxPreflightError::AccessibilityNotTrusted => AxClickError::AccessibilityNotTrusted,
            AxPreflightError::InputAuthorizationNotTrusted => {
                AxClickError::EventAuthorizationNotTrusted
            }
        })?;
        platform::input(application, locator, input)
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct AxBounds {
    pub(crate) x: f64,
    pub(crate) y: f64,
    pub(crate) width: f64,
    pub(crate) height: f64,
}

impl AxBounds {
    pub(crate) fn center(self) -> (f64, f64) {
        (self.x + self.width / 2.0, self.y + self.height / 2.0)
    }
}

pub(super) fn validate_bounds(bounds: AxBounds) -> Result<AxBounds, AxClickError> {
    if [bounds.x, bounds.y, bounds.width, bounds.height]
        .into_iter()
        .all(f64::is_finite)
        && bounds.width > 0.0
        && bounds.height > 0.0
    {
        Ok(bounds)
    } else {
        Err(AxClickError::BoundsValueInvalid)
    }
}

#[cfg(test)]
mod tests {
    use super::{AxBounds, validate_bounds};
    #[cfg(not(target_os = "macos"))]
    use crate::ax_target_locator::AxTargetLocator;
    #[cfg(not(target_os = "macos"))]
    use crate::physical_bootstrap_types::AxApplicationElement;
    use crate::physical_bootstrap_types::AxClickError;
    #[cfg(not(target_os = "macos"))]
    use crate::physical_bootstrap_types::AxPhysicalInput;

    #[test]
    fn non_macos_click_is_typed_failure() {
        #[cfg(not(target_os = "macos"))]
        {
            let application = AxApplicationElement {};
            let locator = AxTargetLocator::from_accessible_name("AXButton", "Run");
            assert_eq!(
                super::AxClicker::click(&application, &locator),
                Err(AxClickError::UnsupportedPlatform)
            );
            for input in [
                AxPhysicalInput::PrimaryPointer,
                AxPhysicalInput::SecondaryPointer,
                AxPhysicalInput::ShiftF10,
                AxPhysicalInput::AccessibilityPress,
            ] {
                assert_eq!(
                    super::AxClicker::input(&application, &locator, input),
                    Err(AxClickError::UnsupportedPlatform)
                );
            }
        }
    }

    #[test]
    fn bounds_require_finite_positive_dimensions() {
        assert_eq!(
            validate_bounds(AxBounds {
                x: 10.0,
                y: 20.0,
                width: 30.0,
                height: 40.0,
            }),
            Ok(AxBounds {
                x: 10.0,
                y: 20.0,
                width: 30.0,
                height: 40.0,
            })
        );
        for bounds in [
            AxBounds {
                x: 0.0,
                y: 0.0,
                width: 0.0,
                height: 1.0,
            },
            AxBounds {
                x: f64::NAN,
                y: 0.0,
                width: 1.0,
                height: 1.0,
            },
        ] {
            assert_eq!(
                validate_bounds(bounds),
                Err(AxClickError::BoundsValueInvalid)
            );
        }
    }
}
