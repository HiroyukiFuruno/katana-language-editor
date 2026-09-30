use super::{
    DIGEST_BYTES, NativeAxObservation, NativeAxObservationError, NativeAxSourceContract,
    NativeAxWorkspaceObservation, RedactedNode,
};
use crate::physical_bootstrap_types::AxApplicationElement;
use objc2_application_services::{AXError, AXUIElement};
use objc2_core_foundation::{CFArray, CFBoolean, CFRetained, CFString, CFType};
use std::collections::HashSet;
use std::ptr::NonNull;

const WINDOWS: &str = "AXWindows";
const CHILDREN: &str = "AXChildren";
const ROLE: &str = "AXRole";
const TITLE: &str = "AXTitle";
const DESCRIPTION: &str = "AXDescription";
const ENABLED: &str = "AXEnabled";
const FOCUSED: &str = "AXFocused";
const READ_ONLY: &str = "AXReadOnly";
const MAX_DEPTH: usize = 64;
const MAX_NODES: usize = 512;

impl NativeAxObservation {
    pub(crate) fn observe(
        application: &AxApplicationElement,
        source: &NativeAxSourceContract,
        process_identity_digest: [u8; DIGEST_BYTES],
        workspace_basename: &str,
    ) -> Result<NativeAxWorkspaceObservation, NativeAxObservationError> {
        let windows = array(&application.element, WINDOWS)?;
        let mut nodes = Vec::new();
        let mut active = HashSet::new();
        let mut ancestors = Vec::new();
        for index in 0..windows.len() {
            let element = windows
                .get(index)
                .ok_or(NativeAxObservationError::AttributeMissing)?;
            visit(&element, 0, source, &mut ancestors, &mut active, &mut nodes)?;
        }
        Self::observe_redacted_frame(&nodes, source, process_identity_digest, workspace_basename)
    }
}

fn visit(
    element: &CFRetained<AXUIElement>,
    depth: usize,
    source: &NativeAxSourceContract,
    ancestors: &mut Vec<String>,
    active: &mut HashSet<usize>,
    nodes: &mut Vec<RedactedNode>,
) -> Result<(), NativeAxObservationError> {
    if depth > MAX_DEPTH {
        return Err(NativeAxObservationError::TraversalDepthExceeded);
    }
    if nodes.len() >= MAX_NODES {
        return Err(NativeAxObservationError::TraversalNodeLimitExceeded);
    }
    let identity = (&**element) as *const AXUIElement as usize;
    if !active.insert(identity) {
        return Err(NativeAxObservationError::TraversalCycle);
    }
    let role = string(element, ROLE)?.ok_or(NativeAxObservationError::AttributeMissing)?;
    let is_editor = role == source.native_role.native_ax_role();
    let is_workspace = role == "AXTab" || role == "AXStaticText";
    nodes.push(RedactedNode {
        role: role.clone(),
        title: string(element, TITLE)?,
        description: string(element, DESCRIPTION)?,
        enabled: (is_editor || is_workspace)
            .then(|| boolean(element, ENABLED))
            .transpose()?,
        focused: is_editor.then(|| boolean(element, FOCUSED)).transpose()?,
        read_only: is_editor.then(|| boolean(element, READ_ONLY)).transpose()?,
        ancestor_roles: ancestors.clone(),
        depth,
    });
    ancestors.push(role);
    if let Some(children) = optional_array(element, CHILDREN)? {
        for index in 0..children.len() {
            let child = children
                .get(index)
                .ok_or(NativeAxObservationError::AttributeMissing)?;
            visit(&child, depth + 1, source, ancestors, active, nodes)?;
        }
    }
    ancestors.pop();
    active.remove(&identity);
    Ok(())
}

fn array(
    element: &CFRetained<AXUIElement>,
    name: &str,
) -> Result<CFRetained<CFArray<AXUIElement>>, NativeAxObservationError> {
    optional_value(element, name)?
        .ok_or(NativeAxObservationError::AttributeMissing)?
        .downcast()
        .map_err(|_| NativeAxObservationError::AttributeTypeMismatch)
        .map(|value: CFRetained<CFArray>| unsafe { CFRetained::cast_unchecked(value) })
}

fn optional_array(
    element: &CFRetained<AXUIElement>,
    name: &str,
) -> Result<Option<CFRetained<CFArray<AXUIElement>>>, NativeAxObservationError> {
    optional_value(element, name)?
        .map(|value| {
            value
                .downcast()
                .map_err(|_| NativeAxObservationError::AttributeTypeMismatch)
                .map(|value: CFRetained<CFArray>| unsafe { CFRetained::cast_unchecked(value) })
        })
        .transpose()
}

fn string(
    element: &CFRetained<AXUIElement>,
    name: &str,
) -> Result<Option<String>, NativeAxObservationError> {
    optional_value(element, name)?
        .map(|value| {
            value
                .downcast::<CFString>()
                .map_err(|_| NativeAxObservationError::AttributeTypeMismatch)
                .map(|value| value.to_string())
        })
        .transpose()
}

fn boolean(
    element: &CFRetained<AXUIElement>,
    name: &str,
) -> Result<bool, NativeAxObservationError> {
    optional_value(element, name)?
        .ok_or(NativeAxObservationError::AttributeMissing)?
        .downcast::<CFBoolean>()
        .map_err(|_| NativeAxObservationError::AttributeTypeMismatch)
        .map(|value| value.value())
}

fn optional_value(
    element: &CFRetained<AXUIElement>,
    name: &str,
) -> Result<Option<CFRetained<CFType>>, NativeAxObservationError> {
    let attribute = CFString::from_str(name);
    let mut raw = std::ptr::null();
    let status = unsafe { element.copy_attribute_value(&attribute, NonNull::from(&mut raw)) };
    if status == AXError::AttributeUnsupported {
        return Ok(None);
    }
    if status != AXError::Success {
        return Err(NativeAxObservationError::AccessibilityUnavailable);
    }
    let raw = NonNull::new(raw as *mut CFType).ok_or(NativeAxObservationError::NullAttribute)?;
    Ok(Some(unsafe { CFRetained::from_raw(raw) }))
}
