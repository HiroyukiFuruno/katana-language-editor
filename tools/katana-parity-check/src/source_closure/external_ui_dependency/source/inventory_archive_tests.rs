use std::fs;
use std::io::Cursor;

use flate2::{Compression, write::GzEncoder};
use tar::{Builder, Header};

use super::super::super::super::capability_manifest::capability_manifest_test_fixtures::FixtureBuilder;
use super::super::super::fingerprint::sha256_hex;
use super::super::lock::LockPackage;
use super::dependency_sources;

const ENTRIES: &str = "struct TextEdit; impl TextEdit { fn multiline() {} fn load_state() {} fn store_state() {} fn show() {} } enum Event { Paste } struct InputState; impl InputState { fn consume_shortcut() {} }";
const MODE: u32 = 0o644;

fn archive(
    root: &std::path::Path,
    entries: &[(&str, &[u8])],
) -> Result<(std::path::PathBuf, String), String> {
    let mut tar = Builder::new(GzEncoder::new(Vec::new(), Compression::default()));
    for (path, bytes) in entries {
        let mut header = Header::new_gnu();
        header.set_size(bytes.len() as u64);
        header.set_mode(MODE);
        header.set_cksum();
        tar.append_data(
            &mut header,
            format!("egui-0.36.1/{path}"),
            Cursor::new(*bytes),
        )
        .map_err(|error| error.to_string())?;
    }
    let bytes = tar
        .into_inner()
        .map_err(|error| error.to_string())?
        .finish()
        .map_err(|error| error.to_string())?;
    let path = root.join("egui-0.36.1.crate");
    fs::write(&path, &bytes).map_err(|error| error.to_string())?;
    Ok((
        path,
        sha256_hex(&bytes).trim_start_matches("sha256:").into(),
    ))
}

#[test]
fn authenticated_archive_keeps_entryless_source_hash() -> Result<(), String> {
    let fixture = FixtureBuilder::root()?;
    let entryless = "pub struct 日本語;\r\n".as_bytes();
    let entries = [
        ("src/entries.rs", ENTRIES.as_bytes()),
        ("src/entryless.rs", entryless),
    ];
    fs::create_dir(fixture.path().join("src")).map_err(|error| error.to_string())?;
    for (path, bytes) in entries {
        fs::write(fixture.path().join(path), bytes).map_err(|error| error.to_string())?;
    }
    let (archive_path, checksum) = archive(fixture.path(), &entries)?;
    let lock = LockPackage {
        version: "0.36.1".into(),
        checksum,
        ..LockPackage::default()
    };
    let (files, symbols, spans, _, unresolved) =
        dependency_sources(fixture.path(), &archive_path, &lock);
    assert!(unresolved.is_empty(), "{unresolved:?}");
    assert_eq!(symbols.len(), 6);
    assert_eq!(spans.len(), 6);
    assert_eq!(files.len(), 2);
    assert_eq!(files[0].path, "egui/src/entries.rs");
    assert_eq!(files[1].path, "egui/src/entryless.rs");
    assert_eq!(files[1].sha256, sha256_hex(entryless));
    Ok(())
}
