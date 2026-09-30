use std::path::MAIN_SEPARATOR;
use std::path::{Path, PathBuf};

pub(super) fn resolve_module_path(root: &Path, current: &Path, module: &str) -> Option<PathBuf> {
    crate_roots(root, current)?;
    let current_file = current.file_name()?.to_str()?;
    let base = match current_file {
        "lib.rs" | "main.rs" | "mod.rs" => current.parent()?.to_path_buf(),
        _ => current.parent()?.join(current.file_stem()?.to_str()?),
    };
    let candidates = [
        base.join(format!("{module}.rs")),
        base.join(module).join("mod.rs"),
    ];
    candidates.into_iter().find(|candidate| candidate.exists())
}

pub(super) fn resolve_rust_name_path_candidates(
    root: &Path,
    current: &Path,
    segments: &[String],
) -> Vec<PathBuf> {
    if segments.is_empty() {
        return Vec::new();
    }
    let Some((_, crate_src)) = crate_roots(root, current) else {
        return Vec::new();
    };
    let first = segments.first().map(String::as_str).unwrap_or_default();
    if first == "Self"
        || (first == "super" && current_module_segments(current, &crate_src).is_empty())
    {
        return Vec::new();
    }

    let module_segments = module_segments(first, segments, current, &crate_src);

    if module_segments.is_empty() {
        return Vec::new();
    }

    let base_len = match first {
        "crate" => 0,
        "self" => current_module_segments(current, &crate_src).len(),
        "super" => current_module_segments(current, &crate_src)
            .len()
            .saturating_sub(1),
        _ => current_module_segments(current, &crate_src).len(),
    };
    for take in (base_len.saturating_add(1)..=module_segments.len()).rev() {
        let path = &module_segments[..take];
        let mut level = Vec::new();
        for candidate in path_candidates(&crate_src, path) {
            if let Ok(canonical) = candidate.canonicalize()
                && canonical.is_file()
                && !level.contains(&canonical)
            {
                level.push(canonical);
            }
        }
        if !level.is_empty() {
            return level;
        }
    }
    Vec::new()
}

fn path_candidates(base: &Path, segments: &[String]) -> Vec<PathBuf> {
    let mut joined = base.to_path_buf();
    for segment in segments {
        joined.push(segment);
    }
    let mut direct = joined.clone();
    direct.set_extension("rs");
    let mut module = joined;
    module.push("mod.rs");
    vec![direct, module]
}

fn module_segments(
    first: &str,
    segments: &[String],
    current: &Path,
    crate_src: &Path,
) -> Vec<String> {
    if first == "crate" {
        return segments.iter().skip(1).cloned().collect();
    }
    let mut base = current_module_segments(current, crate_src);
    match first {
        "self" => base.extend(segments.iter().skip(1).cloned()),
        "super" => {
            if !base.is_empty() {
                base.pop();
            }
            base.extend(segments.iter().skip(1).cloned());
        }
        _ => base.extend(segments.iter().cloned()),
    }
    base
}

fn current_module_segments(current: &Path, crate_src: &Path) -> Vec<String> {
    let relative = current.strip_prefix(crate_src).unwrap_or(current);
    let mut segments = relative
        .components()
        .filter_map(|component| {
            component
                .as_os_str()
                .to_str()
                .map(|value| value.to_string())
        })
        .collect::<Vec<_>>();
    if segments.last().map(String::as_str) == Some("mod.rs") {
        let _ = segments.pop();
        return segments;
    }
    normalize_file_module(segments, current)
}

fn normalize_file_module(mut segments: Vec<String>, current: &Path) -> Vec<String> {
    let Some(stem) = current.file_stem().and_then(|name| name.to_str()) else {
        return segments;
    };
    if matches!(stem, "lib" | "main") {
        let _ = segments.pop();
    } else if stem != "mod"
        && let Some(last) = segments.last_mut()
    {
        *last = stem.to_string();
    }
    segments
}

fn crate_roots(root: &Path, current: &Path) -> Option<(PathBuf, PathBuf)> {
    let mut cursor = current.parent();
    while let Some(dir) = cursor {
        if dir.file_name().is_some_and(|name| name == "src") {
            let crate_root = dir.parent()?.to_path_buf();
            return Some((crate_root, dir.to_path_buf()));
        }
        cursor = dir.parent();
    }
    let fallback = root.join("src");
    if fallback.exists() {
        return Some((root.to_path_buf(), fallback));
    }
    None
}

pub(super) fn relative_path(root: &Path, target: &Path) -> String {
    target
        .strip_prefix(root)
        .unwrap_or(target)
        .to_string_lossy()
        .replace(MAIN_SEPARATOR, "/")
}

#[cfg(test)]
mod path_resolution_tests;
