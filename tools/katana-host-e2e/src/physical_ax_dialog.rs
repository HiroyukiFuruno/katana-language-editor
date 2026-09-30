const NATIVE_DIALOG_ROLES: [&str; 2] = ["AXSheet", "AXDialog"];

#[cfg(target_os = "macos")]
pub(super) fn is_native_dialog(
    element: std::ptr::NonNull<objc2_application_services::AXUIElement>,
) -> bool {
    native_dialog_role(element).as_deref().is_some_and(has_native_dialog_role)
}

#[cfg(target_os = "macos")]
fn native_dialog_role(
    element: std::ptr::NonNull<objc2_application_services::AXUIElement>,
) -> Option<String> {
    use objc2_application_services::AXError;
    use objc2_core_foundation::{CFRetained, CFString, CFType};

    let attribute = CFString::from_str("AXRole");
    let mut raw = std::ptr::null();
    let status = unsafe {
        element
            .as_ref()
            .copy_attribute_value(&attribute, std::ptr::NonNull::from(&mut raw))
    };
    if status != AXError::Success {
        return None;
    }
    let raw = std::ptr::NonNull::new(raw as *mut CFType)?;
    let value = unsafe { CFRetained::from_raw(raw) };
    let role: CFRetained<CFString> = value.downcast().ok()?;
    Some(role.to_string())
}

fn has_native_dialog_role(role: &str) -> bool {
    NATIVE_DIALOG_ROLES.contains(&role)
}

#[cfg(test)]
mod tests {
    use super::has_native_dialog_role;

    #[test]
    fn native_dialog_roles_exclude_unrelated_windows() {
        assert!(has_native_dialog_role("AXSheet"));
        assert!(has_native_dialog_role("AXDialog"));
        assert!(!has_native_dialog_role("AXWindow"));
        assert!(!has_native_dialog_role("AXPopover"));
    }
}
