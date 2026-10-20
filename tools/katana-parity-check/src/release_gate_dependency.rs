use crate::release_gate::ReleaseGateAudit;
use crate::release_gate_sources::{JUSTFILE, LEFTHOOK, PRE_PUSH_GUARD_HOOK, PRE_PUSH_STRICT_HOOK};

const PRE_PUSH_LOCAL_GATES: &[&str] = &[
    "governance-test",
    "fmt-check",
    "check-types",
    "lint",
    "unit-test",
    "ast-lint",
    "kuc-contract-check",
    "katana-interface-check",
    "katana-downstream-check",
    "storybook-motion-artifact-gate",
];

impl ReleaseGateAudit {
    pub(crate) fn validate_release_dependency_graph_ordering() -> Result<(), String> {
        Self::validate_release_dependency_graph_from_lines(&JUSTFILE.lines().collect::<Vec<_>>())
    }

    pub(crate) fn validate_release_dependency_graph_from_lines(
        lines: &[&str],
    ) -> Result<(), String> {
        let release_check_deps = Self::dependencies_for_recipe(lines, "release-check")?;
        if release_check_deps.contains(&"release-verify") {
            return Err(
                "release-check must invoke release-verify in its command body, not as a dependency"
                    .to_string(),
            );
        }
        if release_check_deps.contains(&"katana-repo-adapter-check") {
            return Err(
                "release-check must not depend on a downstream KatanA adapter check".to_string(),
            );
        }

        let release_verify_deps = Self::dependencies_for_recipe(lines, "release-verify")?;
        for required in ["check", "coverage"] {
            if !release_verify_deps.contains(&required) {
                return Err(format!(
                    "release-verify must depend on {required} in release-verify dependency declaration"
                ));
            }
        }

        let check_deps = Self::dependencies_for_recipe(lines, "check")?;
        for required in [
            "kuc-contract-check",
            "storybook-motion-artifact-gate",
            "katana-parity-rc-check",
        ] {
            if !check_deps.contains(&required) {
                return Err(format!("check must include {required} dependency"));
            }
        }

        Ok(())
    }

    pub(crate) fn validate_pre_push_gate() -> Result<(), String> {
        crate::release_gate_hook::ReleaseGateHookAudit::validate_pre_push_hook_sources(
            LEFTHOOK,
            PRE_PUSH_STRICT_HOOK,
            PRE_PUSH_GUARD_HOOK,
        )?;

        let lines = JUSTFILE.lines().collect::<Vec<_>>();
        let dependencies = Self::dependencies_for_recipe(&lines, "pre-push-check")?;
        for required in PRE_PUSH_LOCAL_GATES {
            if !dependencies.contains(required) {
                return Err(format!("pre-push-check must include {required} dependency"));
            }
        }
        if dependencies.contains(&"katana-parity-rc-check") {
            return Err(
                "pre-push-check must leave three-OS source-closure artifact validation to CI"
                    .to_string(),
            );
        }
        crate::release_gate_hook::ReleaseGateHookAudit::validate_pre_push_recipe_body(&lines)
    }
}

#[cfg(test)]
#[path = "release_gate_hook_tests.rs"]
mod hook_tests;

#[cfg(test)]
#[path = "release_gate_postcheck_tests.rs"]
mod postcheck_tests;
