use std::collections::BTreeSet;

use super::operational_input::ProfileProbeInput;
use super::operational_validation::InputVerifier;

impl<'a> InputVerifier<'a> {
    pub(super) fn verify_profile(&mut self, probe: &ProfileProbeInput) -> Result<(), String> {
        let id = probe.id.as_str();
        if probe.katana_revision != super::operational_input::FIXED_KATANA_REVISION {
            return Err(format!(
                "{id} profile KatanA revision does not match the fixed revision"
            ));
        }
        let rustc = self.read_evidence(&probe.rustc_vv_raw, id)?;
        if probe.rustc_vv_raw.command_or_source != "rustc -vV" {
            return Err(format!("{id} rustc -vV command mismatch"));
        }
        let host = rustc
            .split(|byte| *byte == b'\n')
            .find_map(|line| line.strip_prefix(b"host: "))
            .and_then(|line| std::str::from_utf8(line).ok())
            .map(str::trim)
            .ok_or_else(|| format!("{id} rustc -vV has no host line"))?;
        if host != probe.rustc_host_triple {
            return Err(format!("{id} rustc host triple mismatch"));
        }
        if !host_matches_profile(id, host) {
            return Err(format!(
                "{id} rustc host triple is not native to the profile: {host}"
            ));
        }
        let cfg = self.read_evidence(&probe.rustc_cfg_raw, id)?;
        if probe.rustc_cfg_raw.command_or_source != "rustc --print cfg" || cfg.is_empty() {
            return Err(format!("{id} rustc cfg evidence is invalid"));
        }
        let cfg_text = String::from_utf8_lossy(&cfg);
        let target_os = match id {
            "macos-latest" => "macos",
            "windows-latest" => "windows",
            "ubuntu-latest" => "linux",
            _ => return Err(format!("unknown profile id: {id}")),
        };
        if !cfg_text.contains(&format!("target_os=\"{target_os}\"")) {
            return Err(format!(
                "{id} rustc cfg does not prove target_os={target_os}"
            ));
        }
        if !probe
            .cargo_resolution_raw
            .command_or_source
            .starts_with("cargo metadata --locked --offline --format-version 1")
        {
            return Err(format!("{id} cargo resolution command is not registered"));
        }
        self.read_evidence(&probe.cargo_resolution_raw, id)?;
        if !probe
            .cargo_lock_raw
            .command_or_source
            .ends_with("Cargo.lock")
        {
            return Err(format!("{id} Cargo.lock source mismatch"));
        }
        self.read_evidence(&probe.cargo_lock_raw, id)?;
        if probe.source_tree.is_empty() {
            return Err(format!("{id} source-tree evidence must not be empty"));
        }
        for entry in &probe.source_tree {
            self.read_evidence(entry, id)?;
        }
        if !probe
            .source_tree
            .windows(2)
            .all(|pair| pair[0].path < pair[1].path)
        {
            return Err(format!(
                "{id} source-tree evidence is not in canonical order"
            ));
        }
        if super::operational_input::EvidenceRef::profile_tree_fingerprint(id, &probe.source_tree)
            != probe.source_tree_fingerprint
        {
            return Err(format!("{id} source-tree fingerprint mismatch"));
        }
        if probe.active_edge_ids.is_empty() || probe.inactive_cfg_edges.is_empty() {
            return Err(format!(
                "{id} active/inactive cfg edges must both be captured"
            ));
        }
        let mut edges = BTreeSet::new();
        for edge in &probe.active_edge_ids {
            if edge.trim().is_empty() || !edges.insert(edge) {
                return Err(format!("{id} active edge ids are invalid"));
            }
        }
        for edge in &probe.inactive_cfg_edges {
            if edge.edge_id.trim().is_empty()
                || edge.predicate.trim().is_empty()
                || edge.span.trim().is_empty()
                || !edges.insert(&edge.edge_id)
            {
                return Err(format!("{id} inactive cfg edges are invalid"));
            }
        }
        if !probe
            .active_edge_ids
            .windows(2)
            .all(|pair| pair[0] < pair[1])
            || !probe
                .inactive_cfg_edges
                .windows(2)
                .all(|pair| pair[0].edge_id < pair[1].edge_id)
        {
            return Err(format!("{id} cfg edges are not in canonical order"));
        }
        let cfg_probe = self.read_evidence(&probe.cfg_edge_probe, id)?;
        let cfg_text = String::from_utf8_lossy(&cfg_probe);
        if probe
            .active_edge_ids
            .iter()
            .any(|edge| !cfg_text.contains(edge))
        {
            return Err(format!("{id} cfg edge probe omits an active edge"));
        }
        if probe.inactive_cfg_edges.iter().any(|edge| {
            !cfg_text.contains(&edge.edge_id)
                || !cfg_text.contains(&edge.predicate)
                || !cfg_text.contains(&edge.span)
        }) {
            return Err(format!("{id} cfg edge probe omits an inactive edge"));
        }
        if probe.fingerprint != probe.identity_fingerprint() {
            return Err(format!("{id} profile fingerprint mismatch"));
        }
        Ok(())
    }
}

fn host_matches_profile(id: &str, host: &str) -> bool {
    match id {
        "macos-latest" => host.ends_with("-apple-darwin"),
        "windows-latest" => host.contains("-windows-"),
        "ubuntu-latest" => host.contains("-linux-"),
        _ => false,
    }
}
