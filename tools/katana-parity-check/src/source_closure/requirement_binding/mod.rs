mod build;
mod helpers;
mod types;

pub(crate) use types::{
    RequirementBinding, RequirementBindingResult, RequirementBindingUnresolved, UnboundBranch,
};

impl RequirementBindingResult {
    pub(crate) fn ensure_same_root(
        source: &super::root_model::ManifestRoot,
        other: &super::root_model::ManifestRoot,
    ) -> Result<(), String> {
        helpers::ensure_same_root(source, other)
    }
}

#[cfg(test)]
#[path = "../requirement_binding_tests/mod.rs"]
mod tests;
