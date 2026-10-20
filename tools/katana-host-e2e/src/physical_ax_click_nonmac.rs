use crate::ax_target_locator::AxTargetLocator;
use crate::physical_bootstrap_types::{AxApplicationElement, AxClickError, AxPhysicalInput};

pub(super) fn input(
    _application: &AxApplicationElement,
    _locator: &AxTargetLocator,
    _input: AxPhysicalInput,
) -> Result<(), AxClickError> {
    Err(AxClickError::UnsupportedPlatform)
}
