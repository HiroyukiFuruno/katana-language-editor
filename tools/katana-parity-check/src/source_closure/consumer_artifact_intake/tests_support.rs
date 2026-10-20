use crate::source_closure::fingerprint::sha256_hex;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

const SOURCE_SHA: &str = "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
const PROFILE_SHA: &str = "sha256:bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";
const SHA256_HEX_LENGTH: usize = 64;

pub(super) fn fixture() -> Result<(Vec<u8>, PathBuf), String> {
    let root = std::env::temp_dir().join(format!("consumer-intake-{}", nonce()?));
    fs::create_dir(&root).map_err(|error| error.to_string())?;
    let stage_a = write_stage(
        &root,
        StageInput {
            index: 0,
            leaf: "leaf-a",
            interaction: "text_input",
            media: b"png-a",
            unicode: b"unicode-a",
            root_byte: 'c',
            accesskit_byte: 'd',
            receipt_byte: 'e',
        },
    )?;
    let stage_b = write_stage(
        &root,
        StageInput {
            index: 1,
            leaf: "leaf-b",
            interaction: "search",
            media: b"png-b",
            unicode: b"unicode-b",
            root_byte: 'f',
            accesskit_byte: 'a',
            receipt_byte: 'b',
        },
    )?;
    write_json(
        &root,
        "kle-consumer-artifact-evidence.json",
        json!({
            "schema":"kle-consumer-artifact-evidence.v1", "run_id":"run-1",
            "source_closure_sha256":SOURCE_SHA, "profile_fingerprint":PROFILE_SHA,
            "profile_id":"macos-latest", "stages":[stage_a, stage_b]
        }),
    )?;
    let plan = json!({
        "schema":"kle.consumer-artifact-plan.v1", "run_id":"run-1",
        "source_closure_sha256":SOURCE_SHA, "profile_fingerprint":PROFILE_SHA,
        "profile_id":"macos-latest", "stages":[
            {"leaf_id":"leaf-a","interaction":"text_input"},
            {"leaf_id":"leaf-b","interaction":"search"}
        ]
    });
    Ok((
        serde_json::to_vec(&plan).map_err(|error| error.to_string())?,
        root,
    ))
}

pub(super) fn read_evidence(root: &Path) -> Result<Value, String> {
    serde_json::from_slice(
        &fs::read(root.join("kle-consumer-artifact-evidence.json"))
            .map_err(|error| error.to_string())?,
    )
    .map_err(|error| error.to_string())
}

pub(super) fn write_evidence(root: &Path, evidence: &Value) -> Result<(), String> {
    fs::write(
        root.join("kle-consumer-artifact-evidence.json"),
        serde_json::to_vec(evidence).map_err(|error| error.to_string())?,
    )
    .map_err(|error| error.to_string())
}

pub(super) fn plain_hash(bytes: &[u8]) -> String {
    sha256_hex(bytes).trim_start_matches("sha256:").to_owned()
}

struct StageInput<'a> {
    index: usize,
    leaf: &'a str,
    interaction: &'a str,
    media: &'a [u8],
    unicode: &'a [u8],
    root_byte: char,
    accesskit_byte: char,
    receipt_byte: char,
}

fn write_stage(root: &Path, input: StageInput<'_>) -> Result<Value, String> {
    let stage = format!("consumer-stage-{:04}", input.index);
    let revision = input.index as u64 + 1;
    let media_path = format!("{stage}.png");
    let unicode_path = format!("{stage}.unicode-evidence.json");
    let root_record = input.root_byte.to_string().repeat(SHA256_HEX_LENGTH);
    let accesskit = input.accesskit_byte.to_string().repeat(SHA256_HEX_LENGTH);
    let receipt = input.receipt_byte.to_string().repeat(SHA256_HEX_LENGTH);
    let unicode_hash = bound_hash(
        input.unicode,
        &stage,
        input.leaf,
        revision,
        &root_record,
        &accesskit,
        &receipt,
    );
    write(root, &media_path, input.media)?;
    write(root, &unicode_path, input.unicode)?;
    write_json(
        root,
        &format!("{stage}.consumer-artifact.json"),
        json!({
            "schema":"kuc.consumer-full-editor-artifact.v1", "schema_version":1,
            "stage_id":stage, "leaf_id":input.leaf, "root_revision":revision,
            "png_sha256":plain_hash(input.media), "pixel_hash":"a".repeat(SHA256_HEX_LENGTH),
            "root_record_hash":root_record, "accesskit_snapshot_hash":accesskit,
            "unicode_evidence_hash":unicode_hash,
            "unicode_evidence_payload_sha256":plain_hash(input.unicode),
            "unicode_evidence_file":unicode_path,
            "forwarding_receipt_fingerprint":receipt
        }),
    )?;
    Ok(json!({
        "leaf_id":input.leaf, "interaction":input.interaction, "stage_id":stage,
        "media_path":media_path, "media_sha256":plain_hash(input.media),
        "pixel_hash":"a".repeat(SHA256_HEX_LENGTH), "root_record_hash":root_record,
        "accesskit_snapshot_hash":accesskit, "unicode_evidence_hash":unicode_hash,
        "root_revision":revision
    }))
}

fn write(root: &Path, name: &str, bytes: &[u8]) -> Result<(), String> {
    fs::write(root.join(name), bytes).map_err(|error| error.to_string())
}

fn write_json(root: &Path, name: &str, value: Value) -> Result<(), String> {
    write(
        root,
        name,
        &serde_json::to_vec(&value).map_err(|error| error.to_string())?,
    )
}

fn bound_hash(
    bytes: &[u8],
    stage: &str,
    leaf: &str,
    revision: u64,
    root: &str,
    accesskit: &str,
    receipt: &str,
) -> String {
    let mut hasher = Sha256::new();
    for field in [
        b"kuc.consumer-artifact.unicode-evidence.v1".as_slice(),
        bytes,
        stage.as_bytes(),
        leaf.as_bytes(),
        &revision.to_be_bytes(),
        root.as_bytes(),
        accesskit.as_bytes(),
        receipt.as_bytes(),
    ] {
        hasher.update((field.len() as u64).to_be_bytes());
        hasher.update(field);
    }
    hasher
        .finalize()
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

fn nonce() -> Result<u128, String> {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_nanos())
        .map_err(|error| error.to_string())
}
