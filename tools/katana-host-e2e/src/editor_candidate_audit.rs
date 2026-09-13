use crate::fixed_source_harness::FIXED_KATANA_REVISION;
use accesskit::{NodeId, Role, TreeUpdate};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
#[path = "editor_candidate_audit_types.rs"]
mod types;
pub use types::{
    EditorCandidateAuditError, EditorCandidateFacts, EditorCandidateObservation,
    EditorCandidateState, EditorSourceContract,
};
use types::{EditorCandidateFacts as CandidateFacts, EditorCandidateObservation as Observation};

const DIGEST_BYTES: usize = 32;
const DIGEST_PREFIX: &str = "sha256:";
const HEX_BYTES_PER_DIGEST: usize = 2;
const HEX_RADIX: u32 = 16;
const EDITOR_ROLE: Role = Role::MultilineTextInput;

impl EditorSourceContract {
    pub fn multiline_text_input(source_span_digest: impl Into<String>) -> Self {
        Self {
            katana_revision: FIXED_KATANA_REVISION.to_owned(),
            source_span_digest: source_span_digest.into(),
            role: "MultilineTextInput".to_owned(),
        }
    }
}

pub struct EditorCandidateAudit {
    source_span_digest: [u8; DIGEST_BYTES],
}

impl EditorCandidateAudit {
    pub fn new(contract: Option<EditorSourceContract>) -> Result<Self, EditorCandidateAuditError> {
        let contract = contract.ok_or(EditorCandidateAuditError::UnsupportedEditorIdentity)?;
        if contract.katana_revision != FIXED_KATANA_REVISION
            || contract.role != "MultilineTextInput"
        {
            return Err(EditorCandidateAuditError::UnsupportedEditorIdentity);
        }
        let source_span_digest = decode_digest(&contract.source_span_digest)
            .ok_or(EditorCandidateAuditError::UnsupportedEditorIdentity)?;
        Ok(Self { source_span_digest })
    }

    pub fn observe_current_frame(
        &self,
        update: &TreeUpdate,
        frame_generation: u64,
    ) -> Result<Observation, EditorCandidateAuditError> {
        let parents = parent_map(update);
        let mut candidates = update
            .nodes
            .iter()
            .filter(|(_, node)| node.role() == EDITOR_ROLE)
            .map(|(node_id, node)| self.facts(*node_id, node, update, &parents))
            .collect::<Result<Vec<_>, _>>()?;
        match candidates.len() {
            0 => Err(EditorCandidateAuditError::TargetMissing),
            1 => Ok(Observation {
                frame_generation,
                source_span_digest: self.source_span_digest,
                candidate: candidates.pop().expect("candidate count checked"),
            }),
            _ => Err(EditorCandidateAuditError::TargetAmbiguous),
        }
    }

    fn facts(
        &self,
        node_id: NodeId,
        node: &accesskit::Node,
        update: &TreeUpdate,
        parents: &HashMap<NodeId, Vec<NodeId>>,
    ) -> Result<CandidateFacts, EditorCandidateAuditError> {
        let name = node
            .label()
            .filter(|value| !value.is_empty())
            .ok_or(EditorCandidateAuditError::UnsupportedEditorIdentity)?;
        let ancestor_role_digests = ancestors(node_id, update, parents)?;
        Ok(CandidateFacts {
            role_digest: digest_role(EDITOR_ROLE),
            name_digest: digest_text(name),
            description_digest: node
                .description()
                .filter(|value| !value.is_empty())
                .map(digest_text),
            ancestor_role_digests,
            state: EditorCandidateState {
                disabled: node.is_disabled(),
                hidden: node.is_hidden(),
                read_only: node.is_read_only(),
                focused: update.focus == node_id,
            },
        })
    }
}

fn parent_map(update: &TreeUpdate) -> HashMap<NodeId, Vec<NodeId>> {
    let mut parents = HashMap::new();
    for (parent_id, node) in &update.nodes {
        for child_id in node.children() {
            parents
                .entry(*child_id)
                .or_insert_with(Vec::new)
                .push(*parent_id);
        }
    }
    parents
}

fn ancestors(
    node_id: NodeId,
    update: &TreeUpdate,
    parents: &HashMap<NodeId, Vec<NodeId>>,
) -> Result<Vec<[u8; DIGEST_BYTES]>, EditorCandidateAuditError> {
    let mut result = Vec::new();
    let mut current = node_id;
    let mut visited = Vec::new();
    while let Some(parent_id) = parents.get(&current).and_then(|ids| ids.first()).copied() {
        if visited.contains(&parent_id) {
            return Err(EditorCandidateAuditError::UnsupportedEditorIdentity);
        }
        let parent = update
            .nodes
            .iter()
            .find_map(|(id, node)| (*id == parent_id).then_some(node))
            .ok_or(EditorCandidateAuditError::UnsupportedEditorIdentity)?;
        result.push(digest_role(parent.role()));
        visited.push(parent_id);
        current = parent_id;
    }
    Ok(result)
}

fn digest_role(role: Role) -> [u8; DIGEST_BYTES] {
    digest_text(&format!("{role:?}"))
}

fn digest_text(value: &str) -> [u8; DIGEST_BYTES] {
    Sha256::digest(value.as_bytes()).into()
}

fn decode_digest(value: &str) -> Option<[u8; DIGEST_BYTES]> {
    let hex = value.strip_prefix(DIGEST_PREFIX)?;
    if hex.len() != DIGEST_BYTES * HEX_BYTES_PER_DIGEST
        || !hex.bytes().all(|byte| byte.is_ascii_hexdigit())
    {
        return None;
    }
    let mut digest = [0; DIGEST_BYTES];
    for (index, byte) in digest.iter_mut().enumerate() {
        *byte = u8::from_str_radix(
            &hex[index * HEX_BYTES_PER_DIGEST..index * HEX_BYTES_PER_DIGEST + HEX_BYTES_PER_DIGEST],
            HEX_RADIX,
        )
        .ok()?;
    }
    Some(digest)
}

#[cfg(test)]
mod tests;
