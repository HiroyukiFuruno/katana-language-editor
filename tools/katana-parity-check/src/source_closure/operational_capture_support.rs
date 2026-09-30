use std::{fs, path::Path};

use super::operational_input::{EvidenceRef, KatanaTreeEvidence};
use super::operational_output::capture_bytes;
use super::operational_process::{git_output, read_repo_file};

pub(super) fn capture_katana_tree(
    output_root: &Path,
    katana_root: &Path,
    source_paths: &[String],
    run_id: &str,
    evidence: &mut Vec<EvidenceRef>,
) -> Result<Vec<KatanaTreeEvidence>, String> {
    source_paths
        .iter()
        .map(|source_path| {
            let bytes = read_repo_file(katana_root, source_path)?;
            let raw_path = format!("provenance/katana/tree/{source_path}");
            let raw = capture_bytes(
                output_root,
                &raw_path,
                &bytes,
                run_id,
                "read-only-local-source",
                &format!("KatanA/{source_path}"),
                evidence,
            )?;
            Ok(KatanaTreeEvidence {
                source_path: source_path.clone(),
                evidence: raw,
            })
        })
        .collect()
}

pub(super) fn capture_external_ui(
    output_root: &Path,
    katana_root: &Path,
    run_id: &str,
    evidence: &mut Vec<EvidenceRef>,
) -> Result<Vec<EvidenceRef>, String> {
    let lock = fs::read(katana_root.join("Cargo.lock"))
        .map_err(|error| format!("failed to read KatanA Cargo.lock for external UI: {error}"))?;
    let symbol_args = [
        "grep",
        "-n",
        "-E",
        "TextEdit::multiline|TextEdit::load_state|TextEdit::store_state|Event::Paste|consume_shortcut",
        "--",
        "crates/katana-ui/src",
    ];
    let symbols = git_output(katana_root, &symbol_args)?;
    let symbol_command = format!("git -C {} {}", katana_root.display(), symbol_args.join(" "));
    let lock_ref = capture_bytes(
        output_root,
        "provenance/external-ui/KatanA-Cargo.lock",
        &lock,
        run_id,
        "read-only-local-source",
        "KatanA/Cargo.lock",
        evidence,
    )?;
    let symbols_ref = capture_bytes(
        output_root,
        "provenance/external-ui/KatanA-ui-symbols.txt",
        &symbols,
        run_id,
        "read-only-local-source",
        &symbol_command,
        evidence,
    )?;
    Ok(vec![lock_ref, symbols_ref])
}
