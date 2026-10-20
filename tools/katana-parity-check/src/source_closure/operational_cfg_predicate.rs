use std::collections::BTreeSet;

use super::operational_cfg::CfgEdgeRecord;
use super::operational_input::InactiveCfgEdge;

pub(super) fn classify_cfg_edges(records: &[CfgEdgeRecord]) -> (Vec<String>, Vec<InactiveCfgEdge>) {
    let mut active = records
        .iter()
        .filter(|record| record.active)
        .map(|record| record.id.clone())
        .collect::<Vec<_>>();
    let mut inactive = records
        .iter()
        .filter(|record| !record.active)
        .map(|record| InactiveCfgEdge {
            edge_id: record.id.clone(),
            predicate: record.predicate.clone(),
            span: record.span.clone(),
        })
        .collect::<Vec<_>>();
    active.sort();
    inactive.sort_by(|left, right| left.edge_id.cmp(&right.edge_id));
    (active, inactive)
}

pub(super) fn parse_cfg_values(cfg: &[u8]) -> Result<BTreeSet<String>, String> {
    let text = std::str::from_utf8(cfg)
        .map_err(|error| format!("rustc cfg output is not UTF-8: {error}"))?;
    Ok(text
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .map(String::from)
        .collect())
}

pub(super) fn eval_cfg(predicate: &str, values: &BTreeSet<String>) -> Result<bool, String> {
    let predicate = predicate.trim();
    for (name, operator) in [("any", true), ("all", true), ("not", false)] {
        if let Some(body) = predicate
            .strip_prefix(&format!("{name}("))
            .and_then(|body| body.strip_suffix(')'))
        {
            let terms = split_top_level(body);
            if terms.is_empty() {
                return Err(format!("{name} has no terms"));
            }
            let states = terms
                .iter()
                .map(|term| eval_cfg(term, values))
                .collect::<Result<Vec<_>, _>>()?;
            return Ok(match name {
                "any" => states.into_iter().any(|state| state),
                "all" => states.into_iter().all(|state| state),
                "not" if states.len() == 1 => !states[0],
                "not" => return Err("not requires exactly one term".into()),
                _ => operator,
            });
        }
    }
    if let Some((key, value)) = predicate.split_once('=') {
        let key = key.trim();
        let value = value.trim();
        if key.is_empty() || value.len() < 2 || !value.starts_with('"') || !value.ends_with('"') {
            return Err(format!("unsupported key/value predicate: {predicate}"));
        }
        return Ok(values.contains(&format!("{key}={value}")));
    }
    if predicate.is_empty()
        || predicate.bytes().any(|byte| byte.is_ascii_whitespace())
        || predicate.contains('"')
    {
        return Err(format!("unsupported predicate: {predicate}"));
    }
    Ok(values.contains(predicate))
}

pub(super) fn split_top_level(value: &str) -> Vec<String> {
    let mut result = Vec::new();
    let mut start = 0;
    let mut depth = 0;
    let mut quoted = false;
    for (index, byte) in value.bytes().enumerate() {
        match byte {
            b'"' => quoted = !quoted,
            b'(' if !quoted => depth += 1,
            b')' if !quoted && depth > 0 => depth -= 1,
            b',' if !quoted && depth == 0 => {
                result.push(value[start..index].trim().to_string());
                start = index + 1;
            }
            _ => {}
        }
    }
    if start < value.len() {
        result.push(value[start..].trim().to_string());
    }
    result.into_iter().filter(|term| !term.is_empty()).collect()
}

pub(super) fn matching_paren(bytes: &[u8], open: usize) -> Option<usize> {
    let mut depth = 0;
    let mut quoted = false;
    for (index, byte) in bytes.iter().enumerate().skip(open) {
        match byte {
            b'"' => quoted = !quoted,
            b'(' if !quoted => depth += 1,
            b')' if !quoted => {
                depth -= 1;
                if depth == 0 {
                    return Some(index);
                }
            }
            _ => {}
        }
    }
    None
}

pub(super) fn line_number(source: &str, offset: usize) -> usize {
    source[..offset]
        .bytes()
        .filter(|byte| *byte == b'\n')
        .count()
        + 1
}
