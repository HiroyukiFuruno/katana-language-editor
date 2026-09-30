use std::collections::{BTreeMap, BTreeSet};
use std::fs::File;
use std::io::{self, Read};
use std::path::Path;

use flate2::read::GzDecoder;
#[cfg(test)]
use serde_json::Value;
use tar::{Archive, EntryType};

use super::super::super::fingerprint::sha256_hex;
use super::super::lock::LockPackage;

const MAX_COMPRESSED_BYTES: u64 = 64 * 1024 * 1024;
const MAX_DECOMPRESSED_BYTES: u64 = 256 * 1024 * 1024;

#[cfg(test)]
pub(super) fn rust_file_hashes(
    archive_path: &Path,
    lock: &LockPackage,
    checksum_files: &serde_json::Map<String, Value>,
) -> Result<BTreeMap<String, String>, String> {
    let hashes = file_hashes_from_archive(archive_path, lock)?
        .into_iter()
        .filter(|(path, _)| path.ends_with(".rs"))
        .collect::<BTreeMap<_, _>>();
    let manifest_paths = checksum_files
        .keys()
        .filter(|path| path.ends_with(".rs"))
        .cloned()
        .collect::<BTreeSet<_>>();
    let archive_paths = hashes.keys().cloned().collect::<BTreeSet<_>>();
    if archive_paths != manifest_paths {
        return Err("egui registry archive Rust paths differ from checksum manifest".into());
    }
    Ok(hashes)
}

pub(super) fn file_hashes_from_archive(
    archive_path: &Path,
    lock: &LockPackage,
) -> Result<BTreeMap<String, String>, String> {
    let file = File::open(archive_path)
        .map_err(|error| format!("egui registry archive is unavailable: {error}"))?;
    let mut compressed = Vec::new();
    file.take(MAX_COMPRESSED_BYTES + 1)
        .read_to_end(&mut compressed)
        .map_err(|error| format!("egui registry archive is unreadable: {error}"))?;
    if compressed.len() as u64 > MAX_COMPRESSED_BYTES {
        return Err("egui registry archive exceeds 64MiB compressed limit".into());
    }
    if sha256_hex(&compressed).strip_prefix("sha256:") != Some(lock.checksum.as_str()) {
        return Err("egui registry archive differs from Cargo.lock checksum".into());
    }
    let prefix = format!("egui-{}/", lock.version);
    let decoder = GzDecoder::new(compressed.as_slice());
    let mut archive = Archive::new(CappedReader::new(decoder));
    let mut hashes = BTreeMap::new();
    let mut paths = BTreeSet::new();
    let mut rust_paths = BTreeSet::new();
    {
        let entries = archive
            .entries()
            .map_err(|error| format!("egui registry archive cannot be read: {error}"))?;
        for entry in entries {
            let mut entry =
                entry.map_err(|error| format!("egui registry archive is malformed: {error}"))?;
            let kind = entry.header().entry_type();
            let raw = raw_header_path(entry.header())?;
            archive_relative_path(&raw, &prefix, kind.is_dir())?;
            let path = entry.path_bytes().to_vec();
            let path = std::str::from_utf8(&path)
                .map_err(|_| "egui registry archive has a non-UTF-8 path".to_owned())?;
            let relative = archive_relative_path(path, &prefix, kind.is_dir())?;
            if !paths.insert(relative.to_owned()) {
                return Err(format!(
                    "egui registry archive has a duplicate path: {relative}"
                ));
            }
            if kind.is_dir() {
                continue;
            }
            if kind != EntryType::Regular {
                return Err("egui registry archive has an unsupported non-regular entry".into());
            }
            if relative.ends_with(".rs") && !rust_paths.insert(relative.to_owned()) {
                return Err(format!(
                    "egui registry archive has a duplicate Rust path: {relative}"
                ));
            }
            let mut bytes = Vec::new();
            entry.read_to_end(&mut bytes).map_err(|error| {
                format!("egui registry archive Rust member cannot be read: {error}")
            })?;
            hashes.insert(
                relative.to_owned(),
                sha256_hex(&bytes)
                    .strip_prefix("sha256:")
                    .unwrap_or_default()
                    .to_owned(),
            );
        }
    }
    let mut decoder = archive.into_inner();
    io::copy(&mut decoder, &mut io::sink())
        .map_err(|error| format!("egui registry archive is malformed: {error}"))?;
    Ok(hashes)
}

fn raw_header_path(header: &tar::Header) -> Result<String, String> {
    let path = header.path_bytes();
    std::str::from_utf8(&path)
        .map(str::to_owned)
        .map_err(|_| "egui registry archive has a non-UTF-8 raw header path".into())
}

fn archive_relative_path<'a>(
    path: &'a str,
    prefix: &str,
    directory: bool,
) -> Result<&'a str, String> {
    let Some(relative) = path.strip_prefix(prefix) else {
        return Err(format!(
            "egui registry archive entry has an invalid package prefix: {path:?}"
        ));
    };
    if directory && relative.is_empty() {
        return Ok(relative);
    }
    let relative = if directory {
        relative.strip_suffix('/').unwrap_or(relative)
    } else {
        relative
    };
    if relative.is_empty()
        || relative.starts_with('/')
        || relative.contains('\\')
        || has_windows_prefix(relative)
        || relative
            .split('/')
            .any(|part| part.is_empty() || part == "." || part == "..")
    {
        return Err(format!(
            "egui registry archive entry has an invalid path: {path:?}"
        ));
    }
    Ok(relative)
}

fn has_windows_prefix(path: &str) -> bool {
    path.as_bytes().get(1).is_some_and(|colon| *colon == b':')
        && path.as_bytes()[0].is_ascii_alphabetic()
}

struct CappedReader<R> {
    reader: R,
    read: u64,
}

impl<R> CappedReader<R> {
    fn new(reader: R) -> Self {
        Self { reader, read: 0 }
    }
}

impl<R: Read> Read for CappedReader<R> {
    fn read(&mut self, buffer: &mut [u8]) -> io::Result<usize> {
        if buffer.is_empty() {
            return Ok(0);
        }
        let remaining = MAX_DECOMPRESSED_BYTES.saturating_sub(self.read) as usize;
        if remaining == 0 {
            let mut byte = [0];
            return match self.reader.read(&mut byte)? {
                0 => Ok(0),
                _ => Err(io::Error::other(
                    "egui registry archive exceeds 256MiB decompressed limit",
                )),
            };
        }
        let length = buffer.len().min(remaining);
        let read = self.reader.read(&mut buffer[..length])?;
        self.read += read as u64;
        Ok(read)
    }
}

#[cfg(test)]
#[path = "archive_reader_tests.rs"]
mod reader_tests;
