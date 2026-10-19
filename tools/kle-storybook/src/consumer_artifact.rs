#[path = "unicode_evidence.rs"]
mod unicode_evidence;

#[path = "consumer_artifact_execution.rs"]
mod consumer_artifact_execution;
#[path = "consumer_artifact_model.rs"]
mod consumer_artifact_model;
#[path = "consumer_artifact_runner.rs"]
mod consumer_artifact_runner;

pub(crate) use consumer_artifact_runner::ConsumerArtifactRunner;

#[cfg(test)]
#[path = "consumer_artifact_tests.rs"]
mod consumer_artifact_tests;
