use std::collections::BTreeSet;

use super::fingerprint::sha256_hex;
use super::operational_cfg_predicate::{eval_cfg, line_number, matching_paren, split_top_level};

pub(super) struct CfgEdgeRecord {
    pub(super) id: String,
    pub(super) predicate: String,
    pub(super) span: String,
    pub(super) active: bool,
}

impl CfgEdgeRecord {
    pub(super) fn as_line(&self) -> String {
        format!(
            "{}\t{}\t{}\t{}\n",
            if self.active { "active" } else { "inactive" },
            self.id,
            self.predicate,
            self.span
        )
    }
}

pub(super) fn scan_cfg_edges(
    source_path: &str,
    bytes: &[u8],
    cfg_values: &BTreeSet<String>,
) -> Result<Vec<CfgEdgeRecord>, String> {
    let source = std::str::from_utf8(bytes)
        .map_err(|error| format!("KatanA source is not UTF-8: {source_path}: {error}"))?;
    let mut records = Vec::new();
    let mut cursor = 0;
    while cursor < source.len() {
        let Some((offset, marker)) = ["#[cfg(", "#[cfg_attr("]
            .iter()
            .filter_map(|marker| {
                source[cursor..]
                    .find(marker)
                    .map(|offset| (cursor + offset, *marker))
            })
            .min_by_key(|(offset, _)| *offset)
        else {
            break;
        };
        let open = offset + marker.len() - 1;
        let close = matching_paren(source.as_bytes(), open)
            .ok_or_else(|| format!("unresolved cfg predicate span: {source_path}:{offset}"))?;
        let body = &source[open + 1..close];
        let predicate = if marker == "#[cfg_attr(" {
            split_top_level(body)
                .first()
                .map(String::as_str)
                .ok_or_else(|| format!("empty cfg_attr predicate: {source_path}:{offset}"))?
                .to_string()
        } else {
            body.to_string()
        };
        let active = eval_cfg(&predicate, cfg_values).map_err(|error| {
            format!(
                "unresolved cfg predicate at {source_path}:{}: {error}",
                line_number(source, offset)
            )
        })?;
        let span = format!(
            "{source_path}:{}:{}",
            line_number(source, offset),
            line_number(source, close)
        );
        let id = format!(
            "cfg:{}",
            sha256_hex(format!("{source_path}\0{span}\0{predicate}").as_bytes())
        );
        records.push(CfgEdgeRecord {
            id,
            predicate: predicate.trim().into(),
            span,
            active,
        });
        cursor = close + 1;
    }
    Ok(records)
}
