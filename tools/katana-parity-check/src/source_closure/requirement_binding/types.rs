use serde::{Deserialize, Serialize};

use super::super::root_model::ManifestRoot;

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct RequirementBinding {
    pub(crate) requirement_id: String,
    pub(crate) short_reference: String,
    pub(crate) source_path: String,
    pub(crate) source_sha256: String,
    pub(crate) branch_id: String,
    pub(crate) span_start_line: usize,
    pub(crate) span_end_line: usize,
    pub(crate) source_excerpt_sha256: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct RequirementBindingUnresolved {
    pub(crate) requirement_id: String,
    pub(crate) short_reference: String,
    pub(crate) source_path: String,
    pub(crate) reason: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct UnboundInactiveProfilePredicate {
    pub(crate) profile_id: String,
    pub(crate) predicate: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct UnboundBranch {
    pub(crate) branch_id: String,
    pub(crate) file: String,
    pub(crate) symbol: String,
    pub(crate) span_start_line: usize,
    pub(crate) span_end_line: usize,
    pub(crate) kind: String,
    pub(crate) condition: String,
    pub(crate) active_profile_ids: Vec<String>,
    pub(crate) inactive_profile_predicates: Vec<UnboundInactiveProfilePredicate>,
    pub(crate) incoming_edges: Vec<String>,
    pub(crate) source_excerpt_sha256: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct RequirementBindingResult {
    pub(crate) root: ManifestRoot,
    pub(crate) bindings: Vec<RequirementBinding>,
    pub(crate) unresolved: Vec<RequirementBindingUnresolved>,
    pub(crate) unbound_branches: Vec<UnboundBranch>,
    pub(crate) fingerprint: String,
}

impl RequirementBindingResult {
    pub(crate) fn validate_fingerprint(&self) -> Result<(), String> {
        let expected = self
            .fingerprint_value()
            .map_err(|error| format!("serialize requirement binding fingerprint: {error}"))?;
        if expected != self.fingerprint {
            return Err("requirement binding fingerprint mismatch".into());
        }
        Ok(())
    }
}

#[derive(Serialize)]
pub(super) struct FingerprintInput<'a> {
    pub(super) root: &'a ManifestRoot,
    pub(super) bindings: &'a [RequirementBinding],
    pub(super) unresolved: &'a [RequirementBindingUnresolved],
    pub(super) unbound_branches: &'a [UnboundBranch],
}
