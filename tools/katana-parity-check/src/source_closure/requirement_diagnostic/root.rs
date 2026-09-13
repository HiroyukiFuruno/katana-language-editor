use super::super::fingerprint::sha256_hex;
use super::super::model::ManifestRoot;
use super::super::operational_input::FIXED_KATANA_REVISION;
use super::super::scan_state::ScanState;
use super::DIAGNOSTIC_SCHEMA;

const NOT_CAPTURED: &str = "diagnostic:not-captured";

pub(super) fn diagnostic_root(
    state: &ScanState,
    universe: &str,
    aliases: &str,
    external_ui: &str,
) -> Result<ManifestRoot, String> {
    let tree = state
        .files
        .iter()
        .map(|(path, file)| format!("{path}\0{}\n", file.sha256))
        .collect::<String>();
    Ok(ManifestRoot {
        schema_version: DIAGNOSTIC_SCHEMA.into(),
        katana_revision: FIXED_KATANA_REVISION.into(),
        katana_tree_fingerprint: sha256_hex(tree.as_bytes()),
        katana_external_ui_fingerprint: external_ui.into(),
        user_mandated_extensions_fingerprint: NOT_CAPTURED.into(),
        source_universe_fingerprint: universe.into(),
        requirement_source_aliases_fingerprint: aliases.into(),
        kle_tree_fingerprint: NOT_CAPTURED.into(),
        kuc_tree_fingerprint: NOT_CAPTURED.into(),
        release_profile_matrix_fingerprint: NOT_CAPTURED.into(),
        generator_fingerprint: generator_fingerprint()?,
        generated_at_utc: format!("diagnostic:{FIXED_KATANA_REVISION}"),
        static_leaf_count: None,
        expected_leaf_ids: None,
    })
}

fn generator_fingerprint() -> Result<String, String> {
    let executable = std::env::current_exe()
        .map_err(|error| format!("failed to resolve diagnostic executable: {error}"))?;
    let bytes = std::fs::read(&executable)
        .map_err(|error| format!("failed to read diagnostic executable: {error}"))?;
    Ok(sha256_hex(&bytes))
}
