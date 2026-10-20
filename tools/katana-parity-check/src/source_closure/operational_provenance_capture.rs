use std::{fs, path::Path};

use super::operational_capture_support::{capture_external_ui, capture_katana_tree};
use super::operational_cli::{CliOptions, PROFILE_IDS, PROFILE_SOURCE_PREFIX, PROVENANCE_PREFIX};
use super::operational_input::{
    EvidenceRef, FIXED_KATANA_REVISION, KatanaTreeEvidence, RootEvidence, RootProvenance,
    SourceClosureInput,
};
use super::operational_input_assembly::load_profile_probes;
use super::operational_output::{capture_bytes, capture_tree, write_canonical_json};
use super::operational_paths::{canonical_dir, canonical_file, capture_run_id, staging_root};
use super::operational_process::{git_files, git_output};
use super::operational_repository_provenance::capture_repository_revision;
use super::operational_time::utc_timestamp;

pub(super) fn capture_provenance(options: &CliOptions) -> Result<(), String> {
    let output_root = staging_root(options.one("output-dir")?, options)?;
    let katana_root = canonical_dir(options.one("katana-repo")?, "KatanA")?;
    let kle_root = canonical_dir(options.one("kle-repo")?, "KLE")?;
    let kuc_root = canonical_dir(options.one("kuc-repo")?, "KUC")?;
    let user_input = canonical_file(options.one("user-input")?, "user-mandated input")?;
    let source_universe = canonical_file(options.one("source-universe")?, "source-universe input")?;
    let requirement_source_aliases = canonical_file(
        options.one("requirement-source-aliases")?,
        "requirement source aliases",
    )?;
    let generator_schema = canonical_file(options.one("generator-schema")?, "generator schema")?;
    let run_id = capture_run_id(options)?;
    let provenance_dir = output_root.join(PROVENANCE_PREFIX);
    if provenance_dir.exists() {
        return Err(format!(
            "provenance output already exists and is immutable: {}",
            provenance_dir.display()
        ));
    }
    let stage_root = output_root
        .join(".staging")
        .join(format!("provenance-{run_id}"));
    let stage_provenance_dir = stage_root.join(PROVENANCE_PREFIX);
    fs::create_dir_all(&stage_provenance_dir)
        .map_err(|error| format!("failed to create provenance staging directory: {error}"))?;

    let revision_bytes = git_output(&katana_root, &["rev-parse", "HEAD"])?;
    if String::from_utf8_lossy(&revision_bytes).trim() != FIXED_KATANA_REVISION {
        return Err("KatanA checkout is not the fixed source-closure revision".into());
    }
    let mut local_evidence = Vec::new();
    let revision_ref = capture_bytes(
        &stage_root,
        "provenance/katana/revision.txt",
        &revision_bytes,
        &run_id,
        "read-only-local-source",
        "git rev-parse HEAD",
        &mut local_evidence,
    )?;
    let kle_revision =
        capture_repository_revision(&stage_root, &kle_root, "kle", &run_id, &mut local_evidence)?;
    let kuc_revision =
        capture_repository_revision(&stage_root, &kuc_root, "kuc", &run_id, &mut local_evidence)?;

    let katana_paths = git_files(&katana_root, &["*.rs"])?;
    let katana_tree = capture_katana_tree(
        &stage_root,
        &katana_root,
        &katana_paths,
        &run_id,
        &mut local_evidence,
    )?;
    let external_ui = capture_external_ui(&stage_root, &katana_root, &run_id, &mut local_evidence)?;
    let user_bytes = fs::read(&user_input)
        .map_err(|error| format!("failed to read user-mandated input: {error}"))?;
    let user_ref = capture_bytes(
        &stage_root,
        "provenance/user-input/user-mandated-leaves.json",
        &user_bytes,
        &run_id,
        "read-only-local-source",
        "docs/v0-1-0-user-mandated-leaves.json",
        &mut local_evidence,
    )?;
    let source_universe_ref = super::operational_provenance_source_universe::capture(
        &stage_root,
        &source_universe,
        &run_id,
        &mut local_evidence,
    )?;
    let alias_ref = super::operational_provenance_requirements::capture_aliases(
        &stage_root,
        &requirement_source_aliases,
        &run_id,
        &mut local_evidence,
    )?;
    let kle_tree = capture_tree(
        &stage_root,
        &kle_root,
        "kle-tree",
        &run_id,
        &mut local_evidence,
    )?;
    let kuc_tree = capture_tree(
        &stage_root,
        &kuc_root,
        "kuc-tree",
        &run_id,
        &mut local_evidence,
    )?;
    let (generator_binary, generator_schema_ref) =
        super::operational_provenance_generator::capture(
            &stage_root,
            &generator_schema,
            &run_id,
            &mut local_evidence,
        )?;

    let probes = load_profile_probes(&output_root)?;
    super::operational_input_assembly::verify_profile_revisions_for_capture(&probes)?;
    let mut matrix_evidence = Vec::new();
    for (id, probe) in PROFILE_IDS.iter().zip(&probes) {
        let probe_path = output_root
            .join(PROFILE_SOURCE_PREFIX)
            .join(id)
            .join("probe.json");
        let bytes = fs::read(&probe_path)
            .map_err(|error| format!("failed to read profile probe {id}: {error}"))?;
        let matrix_ref = capture_bytes(
            &stage_root,
            &format!("provenance/matrix/{id}.json"),
            &bytes,
            &run_id,
            id,
            &format!("profiles/{id}/probe.json"),
            &mut local_evidence,
        )?;
        if probe.id != *id {
            return Err(format!("profile probe id mismatch while assembling {id}"));
        }
        matrix_evidence.push(matrix_ref);
    }
    let mut root = RootProvenance {
        schema_version: "1".into(),
        katana_revision: FIXED_KATANA_REVISION.into(),
        kle_revision: kle_revision.revision.clone(),
        kle_worktree_clean: true,
        kuc_revision: kuc_revision.revision.clone(),
        kuc_worktree_clean: true,
        katana_tree_fingerprint: KatanaTreeEvidence::set_fingerprint(&katana_tree),
        katana_external_ui_fingerprint: EvidenceRef::set_fingerprint(&external_ui),
        user_mandated_extensions_fingerprint: user_ref.sha256.clone(),
        source_universe_fingerprint: source_universe_ref.sha256.clone(),
        requirement_source_aliases_fingerprint: alias_ref.sha256.clone(),
        kle_tree_fingerprint: EvidenceRef::set_fingerprint(&kle_tree),
        kuc_tree_fingerprint: EvidenceRef::set_fingerprint(&kuc_tree),
        release_profile_matrix_fingerprint: String::new(),
        generator_fingerprint: EvidenceRef::set_fingerprint(&[
            generator_binary.clone(),
            generator_schema_ref.clone(),
        ]),
        generated_at_utc: utc_timestamp()?,
        evidence: RootEvidence {
            katana_revision: revision_ref,
            kle_revision: kle_revision.revision_evidence,
            kle_worktree_status: kle_revision.worktree_status_evidence,
            kuc_revision: kuc_revision.revision_evidence,
            kuc_worktree_status: kuc_revision.worktree_status_evidence,
            katana_tree,
            katana_external_ui: external_ui,
            user_mandated_extensions: user_ref,
            source_universe: source_universe_ref,
            requirement_source_aliases: alias_ref,
            kle_tree,
            kuc_tree,
            release_profile_matrix: matrix_evidence,
            generator_binary,
            generator_schema: generator_schema_ref,
        },
    };
    let input_for_matrix = SourceClosureInput {
        input_schema_version: "1".into(),
        katana_seed_paths: vec!["capture-seeds-required-at-assemble".into()],
        root: root.clone(),
        profile_probes: probes,
    };
    root.release_profile_matrix_fingerprint = input_for_matrix.matrix_fingerprint();
    write_canonical_json(&stage_provenance_dir.join("root.json"), &root)?;
    fs::create_dir_all(provenance_dir.parent().unwrap_or(Path::new(".")))
        .map_err(|error| format!("failed to create provenance output directory: {error}"))?;
    fs::rename(&stage_provenance_dir, &provenance_dir)
        .map_err(|error| format!("failed to finalize immutable provenance output: {error}"))?;
    Ok(())
}
