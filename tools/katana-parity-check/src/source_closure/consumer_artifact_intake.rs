mod files;
mod model;
mod validation;

pub(crate) use model::EvidenceStage;

#[cfg(test)]
#[path = "consumer_artifact_intake/tests_support.rs"]
mod tests_support;

#[cfg(test)]
mod tests;

pub(crate) struct ConsumerArtifactIntake;

pub(crate) struct ValidatedConsumerArtifact {
    evidence: model::Evidence,
    receipts: std::collections::BTreeSet<String>,
}

impl ValidatedConsumerArtifact {
    pub(crate) fn receipt_fingerprints(&self) -> &std::collections::BTreeSet<String> {
        &self.receipts
    }
    pub(crate) fn profile_id(&self) -> &str {
        &self.evidence.profile_id
    }

    pub(crate) fn stages(&self) -> &[model::EvidenceStage] {
        &self.evidence.stages
    }
}

impl ConsumerArtifactIntake {
    pub(crate) fn validate_profile_artifact(
        plan_bytes: &[u8],
        artifact_dir: &std::path::Path,
    ) -> Result<ValidatedConsumerArtifact, String> {
        validation::validate_profile_artifact(plan_bytes, artifact_dir)
            .map(|(evidence, receipts)| ValidatedConsumerArtifact { evidence, receipts })
    }
}
