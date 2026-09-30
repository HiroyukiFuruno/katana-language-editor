use super::model::{ProfileProbeInput, ProfileRecord};
use super::operational_loader::VerifiedSourceClosureInput;

pub(super) fn materialize_profiles(
    verified: &VerifiedSourceClosureInput,
) -> Result<Vec<ProfileRecord>, String> {
    let input = verified.input();
    if input.profile_probes.len() != super::root_model::CANONICAL_PROFILE_IDS.len() {
        return Err("verified input did not contain three profile probes".into());
    }
    input
        .profile_probes
        .iter()
        .map(ProfileProbeInput::to_profile_record)
        .collect()
}

impl ProfileProbeInput {
    fn to_profile_record(&self) -> Result<ProfileRecord, String> {
        Ok(ProfileRecord {
            id: self.id.clone(),
            runner_label: self.runner_label.clone(),
            katana_revision: self.katana_revision.clone(),
            fingerprint: self.fingerprint.clone(),
            rustc_host_triple: self.rustc_host_triple.clone(),
            rustc_cfg_sha256: self.rustc_cfg_raw.sha256.clone(),
            cargo_resolution_sha256: self.cargo_resolution_raw.sha256.clone(),
            lockfile_sha256: self.cargo_lock_raw.sha256.clone(),
            active_edge_ids: self.active_edge_ids.clone(),
            inactive_cfg_edges: self.inactive_cfg_edges.clone(),
        })
    }
}
