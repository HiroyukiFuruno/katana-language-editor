const NATIVE_DIALOG_ROLES: [&str; 2] = ["AXSheet", "AXDialog"];
const NATIVE_DIALOG_SUBROLE: &str = "AXDialog";

#[cfg(target_os = "macos")]
pub(super) fn is_native_dialog(
    element: std::ptr::NonNull<objc2_application_services::AXUIElement>,
) -> bool {
    let role = native_attribute(element, "AXRole");
    let subrole = native_attribute(element, "AXSubrole");
    has_native_dialog_attributes(role.as_deref(), subrole.as_deref())
}

#[cfg(target_os = "macos")]
fn native_attribute(
    element: std::ptr::NonNull<objc2_application_services::AXUIElement>,
    name: &str,
) -> Option<String> {
    use objc2_application_services::AXError;
    use objc2_core_foundation::{CFRetained, CFString, CFType};

    let attribute = CFString::from_str(name);
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

fn has_native_dialog_attributes(role: Option<&str>, subrole: Option<&str>) -> bool {
    role.is_some_and(|value| NATIVE_DIALOG_ROLES.contains(&value))
        || (role == Some("AXWindow") && subrole == Some(NATIVE_DIALOG_SUBROLE))
}

#[cfg(test)]
mod tests {
    use super::has_native_dialog_attributes;

    #[test]
    fn native_dialog_attributes_require_a_dialog_role_or_subrole() {
        assert!(has_native_dialog_attributes(Some("AXSheet"), None));
        assert!(has_native_dialog_attributes(Some("AXDialog"), None));
        assert!(has_native_dialog_attributes(Some("AXWindow"), Some("AXDialog")));
        assert!(!has_native_dialog_attributes(Some("AXWindow"), None));
        assert!(!has_native_dialog_attributes(Some("AXPopover"), Some("AXDialog")));
    }
}
