mod facts;
mod materialize;
mod resolution;
mod routes;

use super::model::{ActionOriginsArtifact, ManifestRoot};
use super::scan_state::ScanState;

pub(super) fn materialize_action_origins(
    root: ManifestRoot,
    state: &ScanState,
) -> ActionOriginsArtifact {
    materialize::materialize(root, state)
}
