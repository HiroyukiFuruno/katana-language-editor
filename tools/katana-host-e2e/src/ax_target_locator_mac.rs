use super::selector::{Candidate, CandidateDigest, CandidateSelector};
use super::types::{AxTargetLocator, AxTargetProof, AxTargetSelectionError};
use objc2_application_services::{AXError, AXUIElement};
use objc2_core_foundation::{CFArray, CFRetained, CFString, CFType};
use std::collections::HashSet;
use std::ptr::NonNull;

const MAX_DEPTH: usize = 64;
const AX_WINDOWS: &str = "AXWindows";
const AX_CHILDREN: &str = "AXChildren";
const AX_ROLE: &str = "AXRole";
const AX_TITLE: &str = "AXTitle";
const AX_DESCRIPTION: &str = "AXDescription";

pub(crate) struct AxTargetTraversal;

pub(crate) struct AxTargetSnapshot {
    pub(crate) element: CFRetained<AXUIElement>,
    pub(crate) proof: AxTargetProof,
}

impl AxTargetTraversal {
    pub(crate) fn locate(
        application: &CFRetained<AXUIElement>,
        locator: &AxTargetLocator,
    ) -> Result<AxTargetProof, AxTargetSelectionError> {
        Ok(Self::locate_snapshot(application, locator)?.proof)
    }

    pub(crate) fn locate_snapshot(
        application: &CFRetained<AXUIElement>,
        locator: &AxTargetLocator,
    ) -> Result<AxTargetSnapshot, AxTargetSelectionError> {
        let mut candidates = Vec::new();
        let mut elements = Vec::new();
        let mut active = HashSet::new();
        let windows = Self::attribute_array(application, AX_WINDOWS)?;
        for index in 0..windows.len() {
            let window = windows
                .get(index)
                .ok_or(AxTargetSelectionError::TraversalAmbiguous)?;
            Self::visit(&window, 0, &mut active, &mut candidates, &mut elements)?;
        }
        let index = CandidateSelector::select(locator, &candidates)?;
        let element = elements
            .get(index)
            .cloned()
            .ok_or(AxTargetSelectionError::TraversalAmbiguous)?;
        Self::revalidate(&element, locator)?;
        Ok(AxTargetSnapshot {
            element,
            proof: CandidateSelector::proof(locator, &candidates, index),
        })
    }

    fn visit(
        element: &CFRetained<AXUIElement>,
        depth: usize,
        active: &mut HashSet<usize>,
        candidates: &mut Vec<Candidate>,
        elements: &mut Vec<CFRetained<AXUIElement>>,
    ) -> Result<(), AxTargetSelectionError> {
        if depth > MAX_DEPTH {
            return Err(AxTargetSelectionError::TraversalDepthExceeded);
        }
        let identity = (&**element) as *const AXUIElement as usize;
        if !active.insert(identity) {
            return Err(AxTargetSelectionError::TraversalCycle);
        }
        let result = Self::visit_attributes(element, depth, active, candidates, elements);
        active.remove(&identity);
        result
    }

    fn visit_attributes(
        element: &CFRetained<AXUIElement>,
        depth: usize,
        active: &mut HashSet<usize>,
        candidates: &mut Vec<Candidate>,
        elements: &mut Vec<CFRetained<AXUIElement>>,
    ) -> Result<(), AxTargetSelectionError> {
        let role = Self::attribute_string(element, AX_ROLE)?
            .ok_or(AxTargetSelectionError::AttributeMissing)?;
        let name = Self::attribute_string(element, AX_TITLE)?
            .filter(|value| !value.is_empty())
            .or(Self::attribute_string(element, AX_DESCRIPTION)?.filter(|value| !value.is_empty()));
        if let Some(name) = name {
            candidates.push(Candidate {
                role_digest: CandidateDigest::of(&role),
                name_digest: CandidateDigest::of(&name),
            });
            elements.push(element.clone());
        }
        if let Some(children) = Self::attribute_array_optional(element, AX_CHILDREN)? {
            for index in 0..children.len() {
                let child = children
                    .get(index)
                    .ok_or(AxTargetSelectionError::TraversalAmbiguous)?;
                Self::visit(&child, depth + 1, active, candidates, elements)?;
            }
        }
        Ok(())
    }

    fn revalidate(
        element: &CFRetained<AXUIElement>,
        locator: &AxTargetLocator,
    ) -> Result<(), AxTargetSelectionError> {
        let role = Self::attribute_string(element, AX_ROLE)?
            .ok_or(AxTargetSelectionError::AttributeMissing)?;
        let name = Self::attribute_string(element, AX_TITLE)?
            .filter(|value| !value.is_empty())
            .or(Self::attribute_string(element, AX_DESCRIPTION)?.filter(|value| !value.is_empty()))
            .ok_or(AxTargetSelectionError::AttributeMissing)?;
        if CandidateDigest::of(&role) != locator.role_digest
            || !locator.name_digests.contains(&CandidateDigest::of(&name))
        {
            return Err(AxTargetSelectionError::TargetMissing);
        }
        Ok(())
    }

    fn attribute_array(
        element: &CFRetained<AXUIElement>,
        name: &str,
    ) -> Result<CFRetained<CFArray<AXUIElement>>, AxTargetSelectionError> {
        let value = Self::copy_attribute(element, name)?;
        let value: CFRetained<CFArray> = value
            .downcast()
            .map_err(|_| AxTargetSelectionError::AttributeTypeMismatch)?;
        Ok(unsafe { CFRetained::cast_unchecked(value) })
    }

    fn attribute_array_optional(
        element: &CFRetained<AXUIElement>,
        name: &str,
    ) -> Result<Option<CFRetained<CFArray<AXUIElement>>>, AxTargetSelectionError> {
        let Some(value) = Self::copy_attribute_optional(element, name)? else {
            return Ok(None);
        };
        let value: CFRetained<CFArray> = value
            .downcast()
            .map_err(|_| AxTargetSelectionError::AttributeTypeMismatch)?;
        Ok(Some(unsafe { CFRetained::cast_unchecked(value) }))
    }

    fn attribute_string(
        element: &CFRetained<AXUIElement>,
        name: &str,
    ) -> Result<Option<String>, AxTargetSelectionError> {
        let Some(value) = Self::copy_attribute_optional(element, name)? else {
            return Ok(None);
        };
        let string: CFRetained<CFString> = value
            .downcast()
            .map_err(|_| AxTargetSelectionError::AttributeTypeMismatch)?;
        Ok(Some(string.to_string()))
    }

    fn copy_attribute(
        element: &CFRetained<AXUIElement>,
        name: &str,
    ) -> Result<CFRetained<CFType>, AxTargetSelectionError> {
        Self::copy_attribute_optional(element, name)?
            .ok_or(AxTargetSelectionError::AttributeMissing)
    }

    fn copy_attribute_optional(
        element: &CFRetained<AXUIElement>,
        name: &str,
    ) -> Result<Option<CFRetained<CFType>>, AxTargetSelectionError> {
        let attribute = CFString::from_str(name);
        let mut value = std::ptr::null();
        let status = unsafe { element.copy_attribute_value(&attribute, NonNull::from(&mut value)) };
        if status == AXError::AttributeUnsupported {
            return Ok(None);
        }
        if status != AXError::Success {
            return Err(AxTargetSelectionError::AccessibilityFailure);
        }
        let value =
            NonNull::new(value as *mut CFType).ok_or(AxTargetSelectionError::NullAttribute)?;
        Ok(Some(unsafe { CFRetained::from_raw(value) }))
    }
}
