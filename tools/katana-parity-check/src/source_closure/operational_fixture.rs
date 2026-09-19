use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use super::super::fingerprint::sha256_hex;
use super::super::operational_input::SourceClosureInput;
use super::super::operational_input::{EvidenceRef, InactiveCfgEdge, ProfileProbeInput};
use super::operational_fixture_root::{fixed_reference_root, make_root};
use super::operational_test_support::{TestResult, option_result, string_result};

pub(super) const DEFAULT_SEED_PATH: &str = "crates/katana-ui/src/widgets/toggle/mod.rs";
pub(super) const DEFAULT_TREE_PATHS: &[&str] = &[
    DEFAULT_SEED_PATH,
    "crates/katana-ui/src/widgets/toggle/types.rs",
    "crates/katana-ui/src/widgets/toggle/ui.rs",
];

pub(super) struct Fixture {
    pub(super) root: PathBuf,
    pub(super) input_json: PathBuf,
    pub(super) katana_root: PathBuf,
}

pub(super) fn fixture(name: &str) -> TestResult<Fixture> {
    let root = std::env::temp_dir().join(format!(
        "kpc-operational-input-{name}-{}-{}",
        std::process::id(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|error| format!("fixture clock failed: {error}"))?
            .as_nanos()
    ));
    fs::create_dir_all(root.join("input/profiles"))?;
    let katana_root = string_result(fixed_reference_root())?.to_path_buf();

    let input_dir = root.join("input");
    let profiles = [
        ("macos-latest", "aarch64-apple-darwin"),
        ("windows-latest", "x86_64-pc-windows-msvc"),
        ("ubuntu-latest", "x86_64-unknown-linux-gnu"),
    ]
    .into_iter()
    .map(|(id, host)| make_profile(&input_dir, id, host, &katana_root))
    .collect::<TestResult<Vec<_>>>()?;
    let input = SourceClosureInput {
        input_schema_version: "1".into(),
        katana_seed_paths: vec![DEFAULT_SEED_PATH.into()],
        root: string_result(make_root(
            &input_dir,
            &profiles,
            &katana_root,
            DEFAULT_SEED_PATH,
            DEFAULT_TREE_PATHS,
        ))?,
        profile_probes: profiles,
    };
    let input_json = input_dir.join("source-closure-input.json");
    write_json(&input_json, &input)?;
    Ok(Fixture {
        root,
        input_json,
        katana_root,
    })
}

fn make_profile(
    input_dir: &Path,
    id: &str,
    host: &str,
    katana_root: &Path,
) -> TestResult<ProfileProbeInput> {
    let prefix = format!("profiles/{id}/raw");
    let vv = write_evidence(
        input_dir,
        &format!("{prefix}/rustc-vv.txt"),
        format!("rustc 1.85.0\nhost: {host}\n").as_bytes(),
    )?;
    let cfg = write_evidence(
        input_dir,
        &format!("{prefix}/rustc-cfg.txt"),
        format!("target_os=\"{}\"\nfeature=editor\n", target_os(id)).as_bytes(),
    )?;
    let cargo = write_evidence(
        input_dir,
        &format!("{prefix}/cargo-resolution.json"),
        format!("{{\"profile\":\"{id}\",\"locked\":true}}").as_bytes(),
    )?;
    let lock = write_evidence(
        input_dir,
        &format!("{prefix}/Cargo.lock"),
        format!("# lock for {id}\n").as_bytes(),
    )?;
    let source = DEFAULT_TREE_PATHS
        .iter()
        .map(|source_path| {
            let actual_path = katana_root.join(source_path);
            let source_bytes = fs::read(&actual_path).map_err(|error| {
                format!(
                    "actual source read failed for {}: {error}",
                    actual_path.display()
                )
            })?;
            write_evidence(
                input_dir,
                &format!("{prefix}/source-tree/{source_path}"),
                &source_bytes,
            )
        })
        .collect::<TestResult<Vec<_>>>()?;
    let active = format!("edge:{id}:active");
    let inactive = InactiveCfgEdge {
        edge_id: format!("edge:{id}:inactive"),
        predicate: format!("target_os = {id}"),
        span: format!("{DEFAULT_SEED_PATH}:1:1-{id}"),
    };
    let probe = write_evidence(
        input_dir,
        &format!("{prefix}/cfg-edge-probe.txt"),
        format!(
            "{active}\n{}\n{}\n{}\n",
            inactive.edge_id, inactive.predicate, inactive.span
        )
        .as_bytes(),
    )?;
    let mut profile = ProfileProbeInput {
        id: id.into(),
        runner_label: id.into(),
        katana_revision: crate::source_closure::operational_input::FIXED_KATANA_REVISION.into(),
        fingerprint: String::new(),
        rustc_host_triple: host.into(),
        rustc_vv_raw: profile_evidence(vv, id, "rustc -vV"),
        rustc_cfg_raw: profile_evidence(cfg, id, "rustc --print cfg"),
        cargo_resolution_raw: profile_evidence(
            cargo,
            id,
            "cargo metadata --locked --offline --format-version 1",
        ),
        cargo_lock_raw: profile_evidence(lock, id, "KatanA/Cargo.lock"),
        source_tree_fingerprint: EvidenceRef::set_fingerprint(&source),
        source_tree: source
            .into_iter()
            .map(|entry| profile_evidence(entry, id, "source tree capture"))
            .collect(),
        active_edge_ids: vec![active],
        inactive_cfg_edges: vec![inactive],
        cfg_edge_probe: profile_evidence(probe, id, "cfg-edge-probe"),
    };
    profile.source_tree_fingerprint =
        EvidenceRef::profile_tree_fingerprint(id, &profile.source_tree);
    profile.fingerprint = profile.identity_fingerprint();
    Ok(profile)
}

fn profile_evidence(mut evidence: EvidenceRef, runner: &str, command: &str) -> EvidenceRef {
    evidence.runner_label = runner.into();
    evidence.command_or_source = command.into();
    evidence
}

fn write_evidence(input_dir: &Path, relative: &str, bytes: &[u8]) -> TestResult<EvidenceRef> {
    let path = input_dir.join(relative);
    let parent = option_result(
        path.parent(),
        format!("evidence path has no parent: {}", path.display()),
    )?;
    fs::create_dir_all(parent)?;
    fs::write(&path, bytes)?;
    Ok(EvidenceRef {
        capture_id: format!("run::fixture::{relative}"),
        path: relative.into(),
        sha256: sha256_hex(bytes),
        command_or_source: "fixture-capture".into(),
        runner_label: "read-only-local-source".into(),
        exit_status: 0,
    })
}

fn write_json<T: serde::Serialize>(path: &Path, value: &T) -> TestResult {
    let mut bytes = serde_json::to_vec_pretty(value)?;
    bytes.push(b'\n');
    fs::write(path, bytes)?;
    Ok(())
}

fn target_os(id: &str) -> &'static str {
    match id {
        "macos-latest" => "macos",
        "windows-latest" => "windows",
        "ubuntu-latest" => "linux",
        _ => "unknown",
    }
}
