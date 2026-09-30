use std::{fs, path::Path};

use super::operational_cfg::{CfgEdgeRecord, scan_cfg_edges};
use super::operational_cfg_predicate::{classify_cfg_edges, parse_cfg_values};
use super::operational_cli::{CliOptions, PROFILE_SOURCE_PREFIX};
use super::operational_input::{EvidenceRef, ProfileProbeInput};
use super::operational_output::{capture_bytes, write_canonical_json, write_checksums};
use super::operational_paths::{
    canonical_dir, staging_root, validate_profile_id, verify_katana_revision,
};
use super::operational_process::{
    cargo_metadata_args, command_text, git_files, read_repo_file, run_command, rustc_host,
    verify_native_profile,
};

pub(super) fn capture_profile(options: &CliOptions) -> Result<(), String> {
    let id = options.one("profile-id")?;
    validate_profile_id(id)?;
    let output_root = options.one("output-dir")?;
    let katana_root = canonical_dir(options.one("katana-repo")?, "KatanA")?;
    let run_id = super::operational_paths::capture_run_id(options)?;
    let output_root = staging_root(output_root, options)?;
    let profile_dir = output_root.join(PROFILE_SOURCE_PREFIX).join(id);
    if profile_dir.exists() {
        return Err(format!(
            "profile output already exists and is immutable: {}",
            profile_dir.display()
        ));
    }
    let stage_root = output_root
        .join(".staging")
        .join(format!("profile-{run_id}-{id}"));
    let stage_profile_dir = stage_root.join(PROFILE_SOURCE_PREFIX).join(id);
    fs::create_dir_all(&stage_profile_dir)
        .map_err(|error| format!("failed to create profile staging directory: {error}"))?;

    verify_katana_revision(&katana_root)?;
    let vv = run_command("rustc", &["-vV"], None)?;
    let host = rustc_host(&vv)?;
    verify_native_profile(id, &host, &[])?;
    let cfg = run_command("rustc", &["--print", "cfg"], None)?;
    verify_native_profile(id, &host, &cfg)?;
    let cargo_args = cargo_metadata_args(&katana_root);
    let cargo_resolution = run_command("cargo", &cargo_args, None)?;
    let lock_path = katana_root.join("Cargo.lock");
    let lock = fs::read(&lock_path)
        .map_err(|error| format!("failed to read KatanA Cargo.lock: {error}"))?;

    let profile_root = &stage_root;
    let runner = id.to_string();
    let mut evidence = Vec::new();
    let vv_ref = capture_bytes(
        profile_root,
        &format!("profiles/{id}/raw/rustc-vv.txt"),
        &vv,
        &run_id,
        id,
        "rustc -vV",
        &mut evidence,
    )?;
    let cfg_ref = capture_bytes(
        profile_root,
        &format!("profiles/{id}/raw/rustc-cfg.txt"),
        &cfg,
        &run_id,
        id,
        "rustc --print cfg",
        &mut evidence,
    )?;
    let resolution_command = command_text("cargo", &cargo_args);
    let resolution_ref = capture_bytes(
        profile_root,
        &format!("profiles/{id}/raw/cargo-resolution.json"),
        &cargo_resolution,
        &run_id,
        id,
        &resolution_command,
        &mut evidence,
    )?;
    let lock_ref = capture_bytes(
        profile_root,
        &format!("profiles/{id}/raw/Cargo.lock"),
        &lock,
        &run_id,
        id,
        &format!(
            "KatanA/{}",
            lock_path.file_name().unwrap_or_default().to_string_lossy()
        ),
        &mut evidence,
    )?;

    let source_paths = git_files(&katana_root, &["*.rs"])?;
    if source_paths.is_empty() {
        return Err("KatanA source tree capture is empty".into());
    }
    let mut source_tree = Vec::new();
    let mut cfg_edges = Vec::new();
    let cfg_values = parse_cfg_values(&cfg)?;
    for source_path in source_paths {
        let bytes = read_repo_file(&katana_root, &source_path)?;
        let raw_path = format!("profiles/{id}/raw/source-tree/{source_path}");
        let source_ref = capture_bytes(
            profile_root,
            &raw_path,
            &bytes,
            &run_id,
            id,
            &format!("KatanA/{source_path}"),
            &mut evidence,
        )?;
        source_tree.push(source_ref);
        cfg_edges.extend(scan_cfg_edges(&source_path, &bytes, &cfg_values)?);
    }
    if cfg_edges.is_empty() {
        return Err("KatanA source tree contains no cfg edges to classify".into());
    }
    let (active_edge_ids, inactive_cfg_edges) = classify_cfg_edges(&cfg_edges);
    let cfg_probe_bytes = cfg_edges
        .iter()
        .map(CfgEdgeRecord::as_line)
        .collect::<String>()
        .into_bytes();
    let cfg_probe_ref = capture_bytes(
        profile_root,
        &format!("profiles/{id}/raw/cfg-edge-probe.txt"),
        &cfg_probe_bytes,
        &run_id,
        id,
        &format!("katana-parity-check source-closure capture-profile --profile-id {id}"),
        &mut evidence,
    )?;

    let mut probe = ProfileProbeInput {
        id: id.into(),
        runner_label: runner,
        katana_revision: super::operational_input::FIXED_KATANA_REVISION.into(),
        fingerprint: String::new(),
        rustc_host_triple: host,
        rustc_vv_raw: vv_ref,
        rustc_cfg_raw: cfg_ref,
        cargo_resolution_raw: resolution_ref,
        cargo_lock_raw: lock_ref,
        source_tree_fingerprint: EvidenceRef::profile_tree_fingerprint(id, &source_tree),
        source_tree,
        active_edge_ids,
        inactive_cfg_edges,
        cfg_edge_probe: cfg_probe_ref,
    };
    probe.fingerprint = probe.identity_fingerprint();
    write_canonical_json(&stage_profile_dir.join("probe.json"), &probe)?;
    write_checksums(&stage_profile_dir.join("checksums.json"), id, &evidence)?;
    fs::create_dir_all(profile_dir.parent().unwrap_or(Path::new(".")))
        .map_err(|error| format!("failed to create profile output directory: {error}"))?;
    fs::rename(&stage_profile_dir, &profile_dir)
        .map_err(|error| format!("failed to finalize immutable profile output: {error}"))?;
    Ok(())
}
