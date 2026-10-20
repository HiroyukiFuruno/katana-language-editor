use super::{AxBounds, validate_bounds};
use crate::ax_target_locator::{AxTargetLocator, mac::AxTargetTraversal};
use crate::physical_bootstrap_types::{AxApplicationElement, AxClickError, AxPhysicalInput};
use core_graphics::event::{CGEvent, CGEventFlags, CGEventTapLocation, CGEventType, CGMouseButton};
use core_graphics::event_source::{CGEventSource, CGEventSourceStateID};
use objc2_application_services::{AXError, AXUIElement, AXValue, AXValueType};
use objc2_core_foundation::{CFBoolean, CFRetained, CFString, CFType, CGPoint, CGSize};
use objc2_core_graphics::CGPreflightPostEventAccess;
use std::ffi::c_void;
use std::ptr::{NonNull, null};

const AX_POSITION: &str = "AXPosition";
const AX_SIZE: &str = "AXSize";
const AX_PRESS: &str = "AXPress";
const AX_FOCUSED: &str = "AXFocused";
/* WHY: macOS Events.h defines F10 as 0x6d; 0x78 is F2 and must not be sent. */
const MACOS_VIRTUAL_KEY_F10: u16 = 0x6D;

pub(super) fn input(
    application: &AxApplicationElement,
    locator: &AxTargetLocator,
    input: AxPhysicalInput,
) -> Result<(), AxClickError> {
    if !CGPreflightPostEventAccess() {
        return Err(AxClickError::EventAuthorizationNotTrusted);
    }
    let snapshot = AxTargetTraversal::locate_snapshot(&application.element, locator)
        .map_err(|_| AxClickError::TargetSelection)?;
    let _bounds = validate_bounds(read_bounds(&snapshot.element)?)?;
    match input {
        AxPhysicalInput::PrimaryPointer => {
            let bounds = validate_bounds(read_bounds(&snapshot.element)?)?;
            let (x, y) = bounds.center();
            post_mouse(
                x,
                y,
                CGMouseButton::Left,
                CGEventType::LeftMouseDown,
                CGEventType::LeftMouseUp,
            )
        }
        AxPhysicalInput::SecondaryPointer => {
            let bounds = validate_bounds(read_bounds(&snapshot.element)?)?;
            let (x, y) = bounds.center();
            post_mouse(
                x,
                y,
                CGMouseButton::Right,
                CGEventType::RightMouseDown,
                CGEventType::RightMouseUp,
            )
        }
        AxPhysicalInput::ShiftF10 => {
            if !read_focused(&snapshot.element)? {
                return Err(AxClickError::TargetSelection);
            }
            post_shift_f10()
        }
        AxPhysicalInput::AccessibilityPress => perform_ax_press(&snapshot.element),
    }
}

fn read_focused(element: &CFRetained<AXUIElement>) -> Result<bool, AxClickError> {
    let attribute = CFString::from_str(AX_FOCUSED);
    let mut raw = null();
    let status = unsafe { element.copy_attribute_value(&attribute, NonNull::from(&mut raw)) };
    if status != AXError::Success {
        return Err(AxClickError::TargetSelection);
    }
    let raw = NonNull::new(raw as *mut CFType).ok_or(AxClickError::TargetSelection)?;
    let value: CFRetained<CFType> = unsafe { CFRetained::from_raw(raw) };
    let value: CFRetained<CFBoolean> = value
        .downcast()
        .map_err(|_| AxClickError::TargetSelection)?;
    Ok(value.value())
}

fn read_bounds(element: &CFRetained<AXUIElement>) -> Result<AxBounds, AxClickError> {
    let position = read_value(element, AX_POSITION, AXValueType::CGPoint)?;
    let size = read_value(element, AX_SIZE, AXValueType::CGSize)?;
    let position = decode_point(&position)?;
    let size = decode_size(&size)?;
    Ok(AxBounds {
        x: position.x,
        y: position.y,
        width: size.width,
        height: size.height,
    })
}

fn read_value(
    element: &CFRetained<AXUIElement>,
    attribute: &str,
    expected_type: AXValueType,
) -> Result<CFRetained<AXValue>, AxClickError> {
    let attribute = CFString::from_str(attribute);
    let mut raw = null();
    let status = unsafe { element.copy_attribute_value(&attribute, NonNull::from(&mut raw)) };
    if status != AXError::Success {
        return Err(if status == AXError::AttributeUnsupported {
            AxClickError::BoundsMissing
        } else {
            AxClickError::SystemFailure
        });
    }
    let raw = NonNull::new(raw as *mut CFType).ok_or(AxClickError::BoundsMissing)?;
    let value: CFRetained<CFType> = unsafe { CFRetained::from_raw(raw) };
    let value: CFRetained<AXValue> = value
        .downcast()
        .map_err(|_| AxClickError::BoundsTypeMismatch)?;
    if unsafe { value.r#type() } != expected_type {
        return Err(AxClickError::BoundsTypeMismatch);
    }
    Ok(value)
}

fn decode_point(value: &AXValue) -> Result<CGPoint, AxClickError> {
    let mut point = CGPoint::default();
    let pointer = NonNull::from(&mut point).cast::<c_void>();
    if unsafe { value.value(AXValueType::CGPoint, pointer) } {
        Ok(point)
    } else {
        Err(AxClickError::BoundsTypeMismatch)
    }
}

fn decode_size(value: &AXValue) -> Result<CGSize, AxClickError> {
    let mut size = CGSize::default();
    let pointer = NonNull::from(&mut size).cast::<c_void>();
    if unsafe { value.value(AXValueType::CGSize, pointer) } {
        Ok(size)
    } else {
        Err(AxClickError::BoundsTypeMismatch)
    }
}

fn post_mouse(
    x: f64,
    y: f64,
    button: CGMouseButton,
    down_type: CGEventType,
    up_type: CGEventType,
) -> Result<(), AxClickError> {
    let source = CGEventSource::new(CGEventSourceStateID::Private)
        .map_err(|_| AxClickError::EventSourceCreationFailed)?;
    let point = core_graphics::geometry::CGPoint::new(x, y);
    let down = CGEvent::new_mouse_event(source.clone(), down_type, point, button)
        .map_err(|_| AxClickError::EventCreationFailed)?;
    let up = CGEvent::new_mouse_event(source, up_type, point, button)
        .map_err(|_| AxClickError::EventCreationFailed)?;
    down.post(CGEventTapLocation::HID);
    up.post(CGEventTapLocation::HID);
    Ok(())
}

fn post_shift_f10() -> Result<(), AxClickError> {
    let source = CGEventSource::new(CGEventSourceStateID::Private)
        .map_err(|_| AxClickError::EventSourceCreationFailed)?;
    let virtual_key = shift_f10_virtual_key_code();
    let down = CGEvent::new_keyboard_event(source.clone(), virtual_key, true)
        .map_err(|_| AxClickError::EventCreationFailed)?;
    let up = CGEvent::new_keyboard_event(source, virtual_key, false)
        .map_err(|_| AxClickError::EventCreationFailed)?;
    down.set_flags(CGEventFlags::CGEventFlagShift);
    up.set_flags(CGEventFlags::CGEventFlagShift);
    down.post(CGEventTapLocation::HID);
    up.post(CGEventTapLocation::HID);
    Ok(())
}

const fn shift_f10_virtual_key_code() -> u16 {
    MACOS_VIRTUAL_KEY_F10
}

fn perform_ax_press(element: &CFRetained<AXUIElement>) -> Result<(), AxClickError> {
    let action = CFString::from_str(AX_PRESS);
    let status = unsafe { element.perform_action(&action) };
    if status == AXError::Success {
        Ok(())
    } else if status == AXError::ActionUnsupported {
        Err(AxClickError::TargetSelection)
    } else {
        Err(AxClickError::SystemFailure)
    }
}

#[cfg(test)]
mod tests {
    use super::{MACOS_VIRTUAL_KEY_F10, shift_f10_virtual_key_code};

    #[test]
    fn shift_f10_selects_macos_f10_and_not_f2() {
        assert_eq!(MACOS_VIRTUAL_KEY_F10, 0x6D);
        assert_eq!(shift_f10_virtual_key_code(), 0x6D);
        assert_ne!(shift_f10_virtual_key_code(), 0x78);
    }
}
