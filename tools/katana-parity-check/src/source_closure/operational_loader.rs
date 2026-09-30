use std::path::Path;

use super::operational_input::SourceClosureInput;
use super::operational_validation::InputVerifier;

pub struct SourceClosureInputLoaderVerifier;

pub struct VerifiedSourceClosureInput {
    pub(super) input: SourceClosureInput,
    pub(super) identity_fingerprint: String,
}

impl SourceClosureInputLoaderVerifier {
    pub fn load(input_json: &Path) -> Result<VerifiedSourceClosureInput, String> {
        let input_json = input_json
            .canonicalize()
            .map_err(|error| format!("input JSON is missing or unreadable: {error}"))?;
        let input_parent = input_json
            .parent()
            .ok_or_else(|| "input JSON has no parent directory".to_string())?;
        let evidence_dir =
            if input_parent.file_name().and_then(|name| name.to_str()) == Some("assembled") {
                input_parent
                    .parent()
                    .ok_or_else(|| "assembled input has no staging root".to_string())?
                    .to_path_buf()
            } else {
                input_parent.to_path_buf()
            };
        let bytes = std::fs::read(&input_json)
            .map_err(|error| format!("failed to read input JSON: {error}"))?;
        let input: SourceClosureInput = serde_json::from_slice(&bytes)
            .map_err(|error| format!("invalid SourceClosureInput JSON: {error}"))?;
        if bytes != input.canonical_json_bytes()? {
            return Err("SourceClosureInput JSON is not canonical".into());
        }
        InputVerifier::new(&evidence_dir, &input_json).verify(&input)?;
        Ok(VerifiedSourceClosureInput {
            identity_fingerprint: input.identity_fingerprint(),
            input,
        })
    }
}

impl VerifiedSourceClosureInput {
    pub fn identity_fingerprint(&self) -> &str {
        &self.identity_fingerprint
    }

    pub(crate) fn input(&self) -> &SourceClosureInput {
        &self.input
    }
}
