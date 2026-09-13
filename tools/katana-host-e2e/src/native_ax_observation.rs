use crate::context_menu_manifest::ContextMenuManifest;
use crate::fixed_source_harness::FIXED_KATANA_REVISION;
use sha2::{Digest, Sha256};
use std::sync::atomic::{AtomicU64, Ordering};

include!("native_ax_observation_types.rs");

const WORKSPACE_ROLES: [&str; 2] = ["AXTab", "AXStaticText"];
static NEXT_GENERATION: AtomicU64 = AtomicU64::new(1);

pub(crate) struct NativeAxObservation;

impl NativeAxObservation {
    pub(crate) fn source_contract_from_context_menu_manifest(
        manifest: &ContextMenuManifest,
    ) -> Result<NativeAxSourceContract, NativeAxObservationError> {
        let native_role = NativeAxRole::from_accesskit_role(&manifest.surface_role)?;
        let source_span_digest = decode_digest(&manifest.surface_source_span_digest)
            .ok_or(NativeAxObservationError::AttributeTypeMismatch)?;
        Ok(NativeAxSourceContract {
            source_span_digest,
            accesskit_role_digest: digest_text(&manifest.surface_role),
            native_role,
        })
    }

    pub(crate) fn observe_redacted_frame(
        nodes: &[RedactedNode],
        source: &NativeAxSourceContract,
        process_identity_digest: [u8; DIGEST_BYTES],
        workspace_basename: &str,
    ) -> Result<NativeAxWorkspaceObservation, NativeAxObservationError> {
        let expected = Self::workspace_digest(workspace_basename)?;
        let workspace_count = Self::workspace_count(nodes, expected);
        match workspace_count {
            0 => return Err(NativeAxObservationError::WorkspaceMismatch),
            1 => {}
            _ => return Err(NativeAxObservationError::WorkspaceDuplicate),
        }
        let editors = Self::editor_candidates(nodes, source)?;
        match editors.len() {
            0 => return Err(NativeAxObservationError::EditorTargetMissing),
            1 => {}
            _ => return Err(NativeAxObservationError::EditorTargetAmbiguous),
        }
        let mut observation = NativeAxWorkspaceObservation {
            schema: "kle.native-ax-workspace-observation.v1",
            katana_revision: FIXED_KATANA_REVISION,
            source_span_digest: source.source_span_digest,
            process_identity_digest,
            observation_generation: NEXT_GENERATION.fetch_add(1, Ordering::Relaxed),
            workspace_basename_digest: expected,
            workspace_correlation_candidates: workspace_count,
            editor_candidates: editors,
            canonical_digest: [0; DIGEST_BYTES],
        };
        observation.canonical_digest = Self::canonical_digest(&observation)?;
        Ok(observation)
    }

    fn workspace_digest(value: &str) -> Result<[u8; DIGEST_BYTES], NativeAxObservationError> {
        if value.is_empty() || value.contains('/') || value.contains('\\') {
            return Err(NativeAxObservationError::WorkspaceMismatch);
        }
        Ok(digest_text(value))
    }

    fn workspace_count(nodes: &[RedactedNode], expected: [u8; DIGEST_BYTES]) -> usize {
        nodes
            .iter()
            .filter(|node| {
                WORKSPACE_ROLES.contains(&node.role.as_str()) && node.enabled == Some(true)
            })
            .filter(|node| {
                node.title
                    .as_deref()
                    .is_some_and(|value| digest_text(value) == expected)
                    || node
                        .description
                        .as_deref()
                        .is_some_and(|value| digest_text(value) == expected)
            })
            .count()
    }

    fn editor_candidates(
        nodes: &[RedactedNode],
        source: &NativeAxSourceContract,
    ) -> Result<Vec<NativeAxEditorCandidate>, NativeAxObservationError> {
        nodes
            .iter()
            .filter(|node| node.role == source.native_role.native_ax_role())
            .map(|node| {
                Ok(NativeAxEditorCandidate {
                    role_digest: digest_text(&node.role),
                    title_digest: node.title.as_deref().map(digest_text),
                    description_digest: node.description.as_deref().map(digest_text),
                    enabled: node
                        .enabled
                        .ok_or(NativeAxObservationError::AttributeMissing)?,
                    focused: node
                        .focused
                        .ok_or(NativeAxObservationError::AttributeMissing)?,
                    read_only: node
                        .read_only
                        .ok_or(NativeAxObservationError::AttributeMissing)?,
                    ancestor_role_digests: node
                        .ancestor_roles
                        .iter()
                        .map(|role| digest_text(role))
                        .collect(),
                    depth: node.depth,
                })
            })
            .collect()
    }

    fn canonical_digest(
        observation: &NativeAxWorkspaceObservation,
    ) -> Result<[u8; DIGEST_BYTES], NativeAxObservationError> {
        let bytes = serde_json::to_vec(&(
            observation.schema,
            observation.katana_revision,
            observation.source_span_digest,
            observation.process_identity_digest,
            observation.observation_generation,
            observation.workspace_basename_digest,
            observation.workspace_correlation_candidates,
            &observation.editor_candidates,
        ))
        .map_err(|_| NativeAxObservationError::AttributeTypeMismatch)?;
        Ok(Sha256::digest(bytes).into())
    }
}

fn digest_text(value: &str) -> [u8; DIGEST_BYTES] {
    Sha256::digest(value.as_bytes()).into()
}

fn decode_digest(value: &str) -> Option<[u8; DIGEST_BYTES]> {
    let hex = value.strip_prefix("sha256:")?;
    if hex.len() != DIGEST_BYTES * 2 || !hex.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return None;
    }
    let mut digest = [0; DIGEST_BYTES];
    for (index, byte) in digest.iter_mut().enumerate() {
        *byte = u8::from_str_radix(&hex[index * 2..index * 2 + 2], 16).ok()?;
    }
    Some(digest)
}

#[cfg(target_os = "macos")]
#[path = "native_ax_observation_macos.rs"]
pub(crate) mod macos;

#[cfg(test)]
#[path = "native_ax_observation_tests.rs"]
mod tests;
