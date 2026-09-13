use crate::release_gate::ReleaseGateAudit;
use crate::release_gate_sources::JUSTFILE;

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
}
