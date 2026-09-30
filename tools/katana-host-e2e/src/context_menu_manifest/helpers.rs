fn direct_authoring_variants(source: &str) -> Result<Vec<&str>, ContextMenuManifestError> {
    let variants = source
        .lines()
        .filter_map(|line| line.split_once("MarkdownAuthoringOp::"))
        .filter_map(|(_, suffix)| suffix.split_once(',').map(|(variant, _)| variant.trim()))
        .filter(|variant| *variant != "CodeBlock")
        .collect::<Vec<_>>();
    if variants.len() != AUTHORING_COUNT || has_duplicate(variants.iter().copied()) {
        return Err(ContextMenuManifestError::Invalid);
    }
    Ok(variants)
}

fn code_block_kinds(source: &str) -> Result<Vec<String>, ContextMenuManifestError> {
    let all = source
        .split_once("const ALL:")
        .and_then(|(_, source)| source.split_once('['))
        .and_then(|(_, source)| source.split_once("];"))
        .map(|(body, _)| {
            body.lines()
                .filter_map(|line| line.trim().strip_prefix("Self::"))
                .map(|variant| variant.trim_end_matches(','))
                .filter(|variant| !variant.is_empty())
                .map(str::to_owned)
                .collect::<Vec<_>>()
        })
        .ok_or(ContextMenuManifestError::Invalid)?;
    if all.len() != CODE_KIND_COUNT || has_duplicate(all.iter().map(String::as_str)) {
        return Err(ContextMenuManifestError::Invalid);
    }
    Ok(all)
}

fn has_duplicate<'a>(mut values: impl Iterator<Item = &'a str>) -> bool {
    let mut seen = BTreeSet::new();
    values.any(|value| !seen.insert(value))
}

fn snake_case(value: &str) -> String {
    value
        .chars()
        .enumerate()
        .fold(String::new(), |mut result, (index, character)| {
            if character.is_ascii_uppercase() && index != 0 {
                result.push('_');
            }
            result.push(character.to_ascii_lowercase());
            result
        })
}

fn read_source(root: &Path, relative: &str) -> Result<String, ContextMenuManifestError> {
    std::fs::read_to_string(root.join(relative)).map_err(|_| ContextMenuManifestError::Unavailable)
}

fn span(path: &str, source: &str, marker: &str) -> Result<String, ContextMenuManifestError> {
    let line = source
        .lines()
        .position(|line| line.contains(marker))
        .map(|line| line + 1)
        .ok_or(ContextMenuManifestError::Invalid)?;
    Ok(digest(&format!("katana:{path}:{line}-{line}")))
}

fn code_label_digest(source: &str, kind: &str) -> Result<String, ContextMenuManifestError> {
    let marker = format!("Self::{kind} =>");
    let start = source
        .find(&marker)
        .ok_or(ContextMenuManifestError::Invalid)?
        + marker.len();
    let label = source[start..]
        .split('"')
        .nth(1)
        .ok_or(ContextMenuManifestError::Invalid)?;
    Ok(digest(label))
}

fn locale_digests(root: &Path, key: &str) -> Result<Vec<String>, ContextMenuManifestError> {
    let mut values = BTreeSet::new();
    let entries = std::fs::read_dir(root).map_err(|_| ContextMenuManifestError::Unavailable)?;
    for entry in entries {
        let path = entry
            .map_err(|_| ContextMenuManifestError::Unavailable)?
            .path();
        if path.extension().and_then(|value| value.to_str()) != Some("json")
            || path.file_name().and_then(|value| value.to_str()) == Some("languages.json")
        {
            continue;
        }
        let bytes = std::fs::read(path).map_err(|_| ContextMenuManifestError::Unavailable)?;
        let value: serde_json::Value =
            serde_json::from_slice(&bytes).map_err(|_| ContextMenuManifestError::Invalid)?;
        let mut current = &value;
        for part in key.split('.') {
            current = current.get(part).ok_or(ContextMenuManifestError::Invalid)?;
        }
        values.insert(digest(
            current
                .as_str()
                .filter(|value| !value.is_empty())
                .ok_or(ContextMenuManifestError::Invalid)?,
        ));
    }
    let values = values.into_iter().collect::<Vec<_>>();
    if values.is_empty() {
        Err(ContextMenuManifestError::Invalid)
    } else {
        Ok(values)
    }
}

fn digest(value: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(value.as_bytes());
    let bytes = hasher.finalize();
    let mut hex = String::with_capacity(DIGEST_HEX_LENGTH);
    for byte in bytes {
        use std::fmt::Write;
        let _ = write!(hex, "{byte:02x}");
    }
    format!("sha256:{hex}")
}

fn is_digest(value: &str) -> bool {
    value.strip_prefix("sha256:").is_some_and(|hex| {
        hex.len() == DIGEST_HEX_LENGTH && hex.bytes().all(|byte| byte.is_ascii_hexdigit())
    })
}

fn validate_fixed_source(root: &Path) -> Result<(), ContextMenuManifestError> {
    let mut command = crate::system::ProcessService::create_command("git");
    let output = command
        .args([
            "-C",
            root.to_str().ok_or(ContextMenuManifestError::Unavailable)?,
            "rev-parse",
            "HEAD",
        ])
        .output()
        .map_err(|_| ContextMenuManifestError::Unavailable)?;
    if !output.status.success() || String::from_utf8_lossy(&output.stdout).trim() != FIXED_REVISION
    {
        return Err(ContextMenuManifestError::Invalid);
    }
    let mut command = crate::system::ProcessService::create_command("git");
    let output = command
        .args([
            "-C",
            root.to_str().ok_or(ContextMenuManifestError::Unavailable)?,
            "status",
            "--porcelain",
            "--untracked-files=all",
        ])
        .output()
        .map_err(|_| ContextMenuManifestError::Unavailable)?;
    if !output.status.success() || !output.stdout.is_empty() {
        return Err(ContextMenuManifestError::Invalid);
    }
    Ok(())
}
