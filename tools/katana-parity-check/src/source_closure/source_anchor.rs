mod generation;
mod model;
mod scan;
mod spans;
mod validation;

use super::operational_cli::CliOptions;

pub(super) fn emit(options: &CliOptions) -> Result<(), String> {
    generation::emit(options)
}

pub(super) fn validate(options: &CliOptions) -> Result<(), String> {
    validation::validate(options)
}

#[cfg(test)]
#[path = "source_anchor_tests.rs"]
mod tests;
