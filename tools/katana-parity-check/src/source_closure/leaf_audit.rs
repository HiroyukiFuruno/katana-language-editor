use std::path::PathBuf;

use super::artifact_paths::ArtifactPaths;
use super::artifact_validator::{ArtifactValidationMode, SourceClosureArtifactValidator};
use super::load::read_artifact;
use super::model::ExecutionRecordArtifact;

#[derive(Debug)]
pub struct LeafCapabilityAudit;

impl LeafCapabilityAudit {
    pub(crate) fn validate(
        repo_root: &Result<PathBuf, String>,
        mode: ArtifactValidationMode,
    ) -> Result<(), String> {
        if let Err(error) = repo_root.as_ref() {
            return Err(format!("failed to resolve KatanA reference: {error}"));
        }

        let artifact_dir = artifact_dir()?;
        if !artifact_dir.exists() {
            return Err(format!(
                "source-closure artifacts are required but missing: set KATANA_PARITY_SOURCE_CLOSURE_ARTIFACT_DIR or place artifacts under {}",
                artifact_dir.display()
            ));
        }

        SourceClosureArtifactValidator::validate_for_mode(
            &ArtifactPaths::from_dir(artifact_dir),
            mode,
        )
        .map_err(|error| {
            format!(
                "source-closure canonical artifacts missing or invalid: {error}; current flow is fail-closed until a generator/materializer provides complete canonical files"
            )
        })
    }

    pub(crate) fn validate_host_e2e_execution() -> Result<(), String> {
        Self::validate_host_e2e_execution_at(&artifact_dir()?)
    }

    fn validate_host_e2e_execution_at(artifact_dir: &PathBuf) -> Result<(), String> {
        let execution_record = ArtifactPaths::from_dir(artifact_dir).execution_record;
        if !execution_record.is_file() {
            return Err(format!(
                "generated source-closure execution artifact is required but missing: {}",
                execution_record.display()
            ));
        }

        let artifact: ExecutionRecordArtifact =
            read_artifact(&execution_record).map_err(|error| {
                format!("generated source-closure execution artifact is invalid: {error}")
            })?;
        if artifact.executions.is_empty() {
            return Err(format!(
                "generated source-closure execution artifact has no execution evidence: {}",
                execution_record.display()
            ));
        }
        Ok(())
    }
}

fn artifact_dir() -> Result<PathBuf, String> {
    std::env::var("KATANA_PARITY_SOURCE_CLOSURE_ARTIFACT_DIR")
        .map(PathBuf::from)
        .or_else(|_| std::env::var("SOURCE_CLOSURE_ARTIFACT_DIR").map(PathBuf::from))
        .or_else(|_| {
            std::env::current_dir()
                .map(|dir| dir.join("target/katana-parity-check/artifacts"))
                .map_err(|error| format!("failed to resolve current directory: {error}"))
        })
}

#[cfg(test)]
mod tests {
    use super::LeafCapabilityAudit;

    #[test]
    fn host_e2e_execution_rejects_missing_generated_execution_artifact() {
        let missing = std::env::temp_dir().join(format!(
            "katana-parity-missing-execution-{}",
            std::process::id()
        ));
        let result = LeafCapabilityAudit::validate_host_e2e_execution_at(&missing);
        assert!(matches!(
            result,
            Err(message) if message.contains("generated source-closure execution artifact is required")
        ));
    }
}
