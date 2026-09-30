use std::{collections::BTreeSet, path::Path};

use crate::{
    capability_manifest::{HostEffectKind, KleHostE2eEvidence},
    system::ProcessService,
};

const HOST_E2E_MANIFEST: &str = "tools/katana-host-e2e/Cargo.toml";
const CARGO_ENVIRONMENT_VARIABLE: &str = "CARGO";
const CARGO_EXECUTABLE: &str = "cargo";

pub(super) struct HostE2eExecutionAudit;

impl HostE2eExecutionAudit {
    pub(super) fn validate(
        workspace_root: &Path,
        evidence: &[KleHostE2eEvidence],
    ) -> Result<(), String> {
        for target in Self::targets(evidence) {
            Self::run_target(workspace_root, &target)?;
        }
        Ok(())
    }

    fn targets(evidence: &[KleHostE2eEvidence]) -> BTreeSet<String> {
        evidence
            .iter()
            .filter(|item| item.effect != HostEffectKind::Missing)
            .map(|item| item.test.target.to_string())
            .collect()
    }

    fn run_target(workspace_root: &Path, target: &str) -> Result<(), String> {
        let cargo =
            std::env::var_os(CARGO_ENVIRONMENT_VARIABLE).unwrap_or_else(|| CARGO_EXECUTABLE.into());
        let status = ProcessService::create_command(cargo)
            .current_dir(workspace_root)
            .args([
                "test",
                "--manifest-path",
                HOST_E2E_MANIFEST,
                "--locked",
                "--test",
                target,
            ])
            .status()
            .map_err(|error| {
                format!(
                    "failed to execute KLE host E2E Cargo target {target}; structural evidence cannot be treated as executed: {error}"
                )
            })?;
        if status.success() {
            Ok(())
        } else {
            Err(format!(
                "KLE host E2E Cargo target {target} failed with status {}; structural evidence cannot be treated as executed",
                status
                    .code()
                    .map_or_else(|| "signal".to_string(), |code| code.to_string())
            ))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const ACTUAL: KleHostE2eEvidence = KleHostE2eEvidence {
        test: crate::capability_manifest::KleHostE2eTestLocator {
            target: "actual_host",
            source_path: "tools/katana-host-e2e/tests/actual_host.rs",
            selector: "actual_kle_context_menu_authoring_raw_input_changes_katana_document",
        },
        effect: HostEffectKind::ContextAuthoring,
    };
    const MISSING: KleHostE2eEvidence = KleHostE2eEvidence {
        effect: HostEffectKind::Missing,
        ..ACTUAL
    };

    #[test]
    fn execution_requires_each_non_missing_exact_cargo_target_once() {
        let targets = HostE2eExecutionAudit::targets(&[ACTUAL, ACTUAL, MISSING]);
        assert_eq!(targets.into_iter().collect::<Vec<_>>(), ["actual_host"]);
    }
}
