mod files;
mod model;
mod validation;

#[cfg(test)]
#[path = "consumer_artifact_intake/tests_support.rs"]
mod tests_support;

#[cfg(test)]
mod tests;

pub(crate) struct ConsumerArtifactIntake;

impl ConsumerArtifactIntake {
    pub(crate) fn validate_profile_artifact(
        plan_bytes: &[u8],
        artifact_dir: &std::path::Path,
    ) -> Result<(), String> {
        validation::validate_profile_artifact(plan_bytes, artifact_dir)
    }
}
