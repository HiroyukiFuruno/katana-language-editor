use serde::{Deserialize, Serialize};

const SHA256_HEX_LENGTH: usize = 64;

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub(super) struct ArtifactProvenance {
    pub(super) run_id: String,
    pub(super) source_closure_sha256: String,
    pub(super) profile_fingerprint: String,
}

impl ArtifactProvenance {
    pub(super) fn validate(&self) -> Result<(), Box<dyn std::error::Error>> {
        if self.run_id.is_empty()
            || self.run_id.chars().any(char::is_whitespace)
            || !valid_sha256(&self.source_closure_sha256)
            || !valid_sha256(&self.profile_fingerprint)
        {
            return Err("invalid source-leaf artifact provenance".into());
        }
        Ok(())
    }
}

fn valid_sha256(value: &str) -> bool {
    value.strip_prefix("sha256:").is_some_and(|digest| {
        digest.len() == SHA256_HEX_LENGTH
            && digest
                .bytes()
                .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    })
}
