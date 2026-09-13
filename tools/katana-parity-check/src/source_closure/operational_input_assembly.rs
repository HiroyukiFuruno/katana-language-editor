use std::path::Path;

use super::materializer::SourceClosureMaterializer;
use super::operational_cli::{CliOptions, PROFILE_IDS, PROFILE_SOURCE_PREFIX, PROVENANCE_PREFIX};
use super::operational_input::{ProfileProbeInput, RootProvenance, SourceClosureInput};
use super::operational_loader::SourceClosureInputLoaderVerifier;
use super::operational_output::{read_canonical_json, write_new};
use super::operational_paths::{canonical_dir, staging_root, verify_katana_revision};

pub(super) fn assemble_input(options: &CliOptions) -> Result<(), String> {
    let output_root = staging_root(options.one("output-dir")?, options)?;
    let root_path = output_root.join(PROVENANCE_PREFIX).join("root.json");
    let root: RootProvenance = read_canonical_json(&root_path, "root provenance")?;
    let seed_paths = resolve_seed_paths(options, &root)?;
    for seed in &seed_paths {
        super::operational_evidence::validate_rust_source_path(seed)?;
    }
    super::operational_staging::validate_profile_staging(&output_root)?;
    let probes = load_profile_probes(&output_root)?;
    verify_profile_revisions(&root, &probes)?;
    if root.release_profile_matrix_fingerprint
        != (SourceClosureInput {
            input_schema_version: "1".into(),
            katana_seed_paths: seed_paths.clone(),
            root: root.clone(),
            profile_probes: probes.clone(),
        })
        .matrix_fingerprint()
    {
        return Err(
            "root release profile matrix fingerprint does not match profile artifacts".into(),
        );
    }
    let input = SourceClosureInput {
        input_schema_version: "1".into(),
        katana_seed_paths: seed_paths,
        root,
        profile_probes: probes,
    };
    let input_path = output_root
        .join("assembled")
        .join("source-closure-input.json");
    write_new(&input_path, &input.canonical_json_bytes()?)?;
    SourceClosureInputLoaderVerifier::load(&input_path)?;
    Ok(())
}

fn resolve_seed_paths(options: &CliOptions, root: &RootProvenance) -> Result<Vec<String>, String> {
    let manifest_values = options.values.get("seed-manifest");
    let direct_paths = options.many("seed-path");
    match manifest_values {
        Some(values) if values.len() == 1 && direct_paths.is_empty() => {
            let manifest = super::operational_paths::canonical_file(
                &values[0],
                "source-root manifest",
            )?;
            let katana_root = canonical_dir(options.one("katana-repo")?, "KatanA")?;
            let source_universe = super::operational_paths::canonical_file(
                options.one("source-universe")?,
                "source-universe input",
            )?;
            verify_katana_revision(&katana_root)?;
            super::source_roots::resolve(
                &manifest,
                &source_universe,
                &katana_root,
                &root.source_universe_fingerprint,
            )
        }
        Some(values) if values.len() != 1 => {
            Err("option --seed-manifest must occur exactly once".into())
        }
        Some(_) => Err("--seed-manifest and --seed-path cannot be used together".into()),
        None if direct_paths.is_empty() => Err(
            "assemble-input requires --seed-manifest or at least one --seed-path; no static seed fallback exists"
                .into(),
        ),
        None if options.values.contains_key("katana-repo") => {
            Err("--katana-repo requires --seed-manifest".into())
        }
        None if options.values.contains_key("source-universe") => {
            Err("--source-universe requires --seed-manifest".into())
        }
        None => Ok(direct_paths.to_vec()),
    }
}

fn verify_profile_revisions(
    root: &RootProvenance,
    probes: &[ProfileProbeInput],
) -> Result<(), String> {
    if root.katana_revision != super::operational_input::FIXED_KATANA_REVISION {
        return Err("root provenance does not use the fixed KatanA revision".into());
    }
    for probe in probes {
        if probe.katana_revision != super::operational_input::FIXED_KATANA_REVISION {
            return Err(format!(
                "{} profile does not use the fixed KatanA revision",
                probe.id
            ));
        }
        if probe.katana_revision != root.katana_revision {
            return Err(format!(
                "{} profile revision does not match root provenance",
                probe.id
            ));
        }
    }
    Ok(())
}

pub(super) fn verify_profile_revisions_for_capture(
    probes: &[ProfileProbeInput],
) -> Result<(), String> {
    for probe in probes {
        if probe.katana_revision != super::operational_input::FIXED_KATANA_REVISION {
            return Err(format!(
                "{} profile does not use the fixed KatanA revision",
                probe.id
            ));
        }
    }
    Ok(())
}

pub(super) fn load_profile_probes(output_root: &Path) -> Result<Vec<ProfileProbeInput>, String> {
    PROFILE_IDS
        .iter()
        .map(|id| {
            let path = output_root
                .join(PROFILE_SOURCE_PREFIX)
                .join(id)
                .join("probe.json");
            read_canonical_json(&path, &format!("profile probe {id}"))
        })
        .collect()
}

pub(super) fn validate_input(options: &CliOptions) -> Result<(), String> {
    let input_path = Path::new(options.one("input")?);
    let staging = super::operational_staging::staging_root_for_input(input_path)?;
    super::operational_staging::validate_profile_staging(&staging)?;
    let verified = SourceClosureInputLoaderVerifier::load(input_path)?;
    super::operational_staging::write_validation_receipt(&staging, input_path, &verified)?;
    println!(
        "validated SourceClosureInput {}",
        verified.identity_fingerprint()
    );
    Ok(())
}

pub(super) fn materialize_closure(options: &CliOptions) -> Result<(), String> {
    let input_path = Path::new(options.one("input")?);
    let katana_root = canonical_dir(options.one("katana-repo")?, "KatanA")?;
    let canonical_root = Path::new(options.one("canonical-root")?);
    let staging = super::operational_staging::staging_root_for_input(input_path)?;
    super::operational_staging::validate_profile_staging(&staging)?;
    super::operational_staging::validate_validation_receipt(&staging, input_path)?;
    let verified = SourceClosureInputLoaderVerifier::load(input_path)?;
    let materialized = staging.join("materialized");
    let output = SourceClosureMaterializer::materialize(&verified, &katana_root, &materialized)?;
    super::operational_publication::publish(&staging, input_path, &verified, canonical_root)?;
    println!("materialized source closure {}", output.display());
    Ok(())
}
