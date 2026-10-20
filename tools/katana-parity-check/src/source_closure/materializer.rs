mod publication;
mod scan;

#[cfg(test)]
pub(crate) use publication::PublicationPaths;

use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::path::{Path, PathBuf};

use super::action_origins::materialize_action_origins;
use super::actual_reference::ActualReferenceVerifier;
use super::branch_catalog::materialize_branch_catalog;
use super::canonical_leaf_binding::CanonicalLeafBindingEvidence;
use super::context_menu_manifest;
use super::external_ui_dependency::materialize as materialize_external_ui_dependency;
use super::input_validation::resolve_seed_path;
use super::model::{LeafManifestArtifact, SourceClosureArtifact, SourceClosureFile};
use super::native_target;
use super::operational_loader::VerifiedSourceClosureInput;
use super::profile_validation::materialize_profiles;
use super::scan_state::ScanState;

pub struct SourceClosureMaterializer;

impl SourceClosureMaterializer {
    pub fn materialize(
        verified: &VerifiedSourceClosureInput,
        katana_root: &Path,
        artifact_dir: &Path,
    ) -> Result<PathBuf, String> {
        CanonicalLeafBindingEvidence::verify_runtime_requirements()?;
        let binding = ActualReferenceVerifier::bind(verified, katana_root)?;
        let input = binding.input();
        let profiles = materialize_profiles(verified)?;
        let mut state = ScanState::default();
        let mut discovered = input
            .katana_seed_paths
            .iter()
            .map(|seed| resolve_seed_path(binding.katana_root(), seed))
            .collect::<Result<VecDeque<_>, _>>()?;
        let mut seen = BTreeSet::new();

        while let Some(source_path) = discovered.pop_front() {
            scan::source(
                &binding,
                source_path,
                &mut state,
                &mut discovered,
                &mut seen,
            )?;
        }
        state.finalize_unscanned_targets();
        for (path, file) in &state.files {
            binding.verify_scanned_file(path, &file.sha256)?;
        }

        let root = input.root.to_manifest_root();
        let source_derived_native_target = native_target::generate(&root, binding.katana_root())?;
        let context_menu_target_manifest =
            context_menu_manifest::ContextMenuManifestGenerator::generate(
                &root,
                binding.katana_root(),
            )?;
        let branch_catalog =
            materialize_branch_catalog(root.clone(), &state, &profiles, binding.katana_root())?;
        let edge_index: BTreeMap<_, _> = state
            .edges
            .iter()
            .map(|edge| (edge.id.clone(), edge.clone()))
            .collect();
        let action_origins = materialize_action_origins(root.clone(), &state);
        let files = std::mem::take(&mut state.files)
            .into_iter()
            .map(|(path, file)| SourceClosureFile {
                path,
                sha256: file.sha256,
                incoming_edges: file
                    .incoming_edges
                    .iter()
                    .filter_map(|edge_id| edge_index.get(edge_id).cloned())
                    .collect(),
                classification: file.classification,
                classification_rationale: file.classification_rationale,
            })
            .collect();
        let source_closure = SourceClosureArtifact {
            root,
            profiles,
            files,
            external_ui_semantic_dependencies: vec![materialize_external_ui_dependency(&binding)],
            unresolved_edges: state.unresolved_edges.clone(),
            source_derived_native_target: Some(source_derived_native_target.clone()),
            context_menu_target_manifest: Some(context_menu_target_manifest.clone()),
        };
        let canonical_binding =
            CanonicalLeafBindingEvidence::build(&source_closure, &branch_catalog, &state)?;
        let leaf_manifest = LeafManifestArtifact {
            root: source_closure.root.clone(),
            leafs: canonical_binding.project_leafs(&branch_catalog)?,
            canonical_binding,
        };
        publication::write_artifacts(
            artifact_dir,
            &source_closure,
            &branch_catalog,
            &action_origins,
            &leaf_manifest,
            &source_derived_native_target,
            &context_menu_target_manifest,
        )
    }

    #[cfg(test)]
    pub(super) fn publish_prepared_artifacts_for_test(
        paths: &PublicationPaths<'_>,
    ) -> Result<(), String> {
        publication::publish_prepared_artifacts(paths)
    }
}
