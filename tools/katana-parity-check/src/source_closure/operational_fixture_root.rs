use std::path::Path;

use super::super::operational_input::{ProfileProbeInput, RootProvenance};

#[path = "operational_fixture_root_evidence.rs"]
mod evidence;
#[path = "operational_fixture_source_universe.rs"]
mod source_universe;

pub(super) use super::super::fixed_reference::fixed_reference_root;

pub(super) fn make_root(
    input_dir: &Path,
    profiles: &[ProfileProbeInput],
    katana_root: &Path,
    seed_path: &str,
    tree_paths: &[&str],
) -> Result<RootProvenance, String> {
    evidence::EvidenceRootBuilder::make_root(
        input_dir,
        profiles,
        katana_root,
        seed_path,
        tree_paths,
    )
}
