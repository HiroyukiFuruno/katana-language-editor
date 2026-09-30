use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use serde_json::Value;

use super::artifact_model::SourceDerivedNativeTargetRecord;
use super::fingerprint::sha256_hex;
use super::operational_input::FIXED_KATANA_REVISION;
use super::root_model::ManifestRoot;

pub(super) const GENERATED_BY: &str = "katana-source-closure-generator";
pub(super) const TARGET_SOURCE_PATH: &str = "crates/katana-ui/src/views/panels/explorer/empty.rs";
const TARGET_SOURCE_START_LINE: usize = 52;
const TARGET_SOURCE_END_LINE: usize = 60;
const TARGET_SOURCE_SPAN: &str = "katana:crates/katana-ui/src/views/panels/explorer/empty.rs:52-60";
const LOCALES_DIRECTORY: &str = "crates/katana-ui/locales";
const LOCALES_METADATA_FILE: &str = "languages.json";

type LocaleSourceInput = (String, Vec<u8>);

pub(super) fn generate(
    root: &ManifestRoot,
    katana_root: &Path,
) -> Result<SourceDerivedNativeTargetRecord, String> {
    let source = read_file(katana_root, TARGET_SOURCE_PATH)?;
    let locales = read_locale_files(katana_root)?;
    generate_from_bytes(root, &source, &locales)
}

fn generate_from_bytes(
    root: &ManifestRoot,
    source: &[u8],
    locales: &[LocaleSourceInput],
) -> Result<SourceDerivedNativeTargetRecord, String> {
    if root.katana_revision != FIXED_KATANA_REVISION {
        return Err("native target source must use the fixed KatanA revision".into());
    }
    let source = std::str::from_utf8(source)
        .map_err(|error| format!("native target source is not UTF-8: {error}"))?;
    let lines = source.lines().collect::<Vec<_>>();
    let selected = lines
        .get(TARGET_SOURCE_START_LINE - 1..TARGET_SOURCE_END_LINE)
        .ok_or_else(|| "native target source span is outside the fixed source".to_string())?
        .join("\n");
    if !selected.contains("egui::Button::new")
        || !selected.contains("I18nOps::get().menu.open_workspace")
        || !selected.contains("pick_open_workspace")
    {
        return Err("fixed empty Explorer Open Workspace source span changed".into());
    }
    if locales.is_empty() {
        return Err("fixed KatanA locale directory contains no locale JSON files".into());
    }
    let mut name_digests = BTreeSet::new();
    for (locale_name, locale_bytes) in locales {
        let locale: Value = serde_json::from_slice(locale_bytes)
            .map_err(|error| format!("fixed locale {locale_name} is invalid JSON: {error}"))?;
        let resolved_label = locale
            .get("menu")
            .and_then(|menu| menu.get("open_workspace"))
            .and_then(Value::as_str)
            .filter(|label| !label.is_empty())
            .ok_or_else(|| {
                format!("fixed locale {locale_name} has no non-empty menu.open_workspace label")
            })?;
        name_digests.insert(sha256_hex(resolved_label.as_bytes()));
    }

    Ok(SourceDerivedNativeTargetRecord {
        schema_version: "1".into(),
        generated_by: GENERATED_BY.into(),
        katana_revision: root.katana_revision.clone(),
        profile_fingerprint: root.release_profile_matrix_fingerprint.clone(),
        source_span_digest: sha256_hex(TARGET_SOURCE_SPAN.as_bytes()),
        role_digest: sha256_hex(b"AXButton"),
        name_digests: name_digests.into_iter().collect(),
    })
}

fn read_locale_files(root: &Path) -> Result<Vec<LocaleSourceInput>, String> {
    let directory = root.join(LOCALES_DIRECTORY);
    let mut paths = std::fs::read_dir(&directory)
        .map_err(|error| format!("cannot read fixed KatanA locale directory: {error}"))?
        .map(|entry| {
            entry
                .map(|entry| entry.path())
                .map_err(|error| format!("cannot read fixed locale directory entry: {error}"))
        })
        .collect::<Result<Vec<PathBuf>, _>>()?;
    paths.retain(|path| {
        path.extension()
            .is_some_and(|extension| extension == "json")
            && path
                .file_name()
                .is_none_or(|name| name != LOCALES_METADATA_FILE)
    });
    paths.sort_unstable();
    if paths.is_empty() {
        return Err("fixed KatanA locale directory contains no locale JSON files".into());
    }
    paths
        .into_iter()
        .map(|path| {
            let name = path
                .file_name()
                .and_then(|name| name.to_str())
                .ok_or_else(|| format!("fixed locale path is not valid UTF-8: {}", path.display()))?
                .to_owned();
            let bytes = std::fs::read(&path)
                .map_err(|error| format!("cannot read fixed locale {name}: {error}"))?;
            Ok((name, bytes))
        })
        .collect()
}

fn read_file(root: &Path, relative: &str) -> Result<Vec<u8>, String> {
    std::fs::read(root.join(relative))
        .map_err(|error| format!("cannot read fixed KatanA source {relative}: {error}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn root() -> ManifestRoot {
        ManifestRoot {
            schema_version: "1".into(),
            katana_revision: FIXED_KATANA_REVISION.into(),
            katana_tree_fingerprint: "sha256:tree".into(),
            katana_external_ui_fingerprint: "sha256:external".into(),
            user_mandated_extensions_fingerprint: "sha256:user".into(),
            source_universe_fingerprint: "sha256:source-universe".into(),
            requirement_source_aliases_fingerprint: "sha256:aliases".into(),
            kle_tree_fingerprint: "sha256:kle".into(),
            kuc_tree_fingerprint: "sha256:kuc".into(),
            release_profile_matrix_fingerprint: "sha256:profile".into(),
            generator_fingerprint: "sha256:generator".into(),
            generated_at_utc: "1970-01-01T00:00:00Z".into(),
            static_leaf_count: None,
            expected_leaf_ids: None,
        }
    }

    #[test]
    fn generates_host_compatible_digest_only_record() -> Result<(), String> {
        let source = (1..=60)
            .map(|line| match line {
                52 => "                .add(egui::Button::new(".to_string(),
                53 => "                    egui::RichText::new(crate::i18n::I18nOps::get().menu.open_workspace.clone()),".to_string(),
                54 => "                ))".to_string(),
                55 => "                .clicked()".to_string(),
                56 => "            {".to_string(),
                57 => "                *self.action = crate::shell_ui::ShellUiOps::pick_open_workspace();".to_string(),
                58..=60 => "            }".to_string(),
                _ => String::new(),
            })
            .collect::<Vec<_>>()
            .join("\n");
        let locales = vec![
            (
                "de.json".to_string(),
                [
                    br#"{"menu":{"open_workspace":"Arbeitsbereich "#.as_slice(),
                    &[0xC3, 0xB6],
                    br#"ffnen"}}"#.as_slice(),
                ]
                .concat(),
            ),
            (
                "en.json".to_string(),
                br#"{"menu":{"open_workspace":"Open Workspace"}}"#.to_vec(),
            ),
            (
                "ja.json".to_string(),
                [
                    br#"{"menu":{"open_workspace":""#.as_slice(),
                    &[
                        0xE3, 0x83, 0xAF, 0xE3, 0x83, 0xBC, 0xE3, 0x82, 0xAF, 0xE3, 0x82, 0xB9,
                        0xE3, 0x83, 0x9A, 0xE3, 0x83, 0xBC, 0xE3, 0x82, 0xB9, 0xE3, 0x82, 0x92,
                        0xE9, 0x96, 0x8B, 0xE3, 0x81, 0x8F,
                    ],
                    br#""}}"#.as_slice(),
                ]
                .concat(),
            ),
        ];
        let record = generate_from_bytes(&root(), source.as_bytes(), &locales)?;
        let json = serde_json::to_string(&record)
            .map_err(|error| format!("native target record JSON serialization failed: {error}"))?;
        assert!(!json.contains("Open Workspace"));
        assert!(!json.contains("Arbeitsbereich"));
        assert!(!json.contains("AXButton"));
        assert_eq!(record.generated_by, GENERATED_BY);
        assert_eq!(record.profile_fingerprint, "sha256:profile");
        assert_eq!(record.name_digests.len(), 3);
        assert!(record.name_digests.windows(2).all(|pair| pair[0] < pair[1]));
        Ok(())
    }

    #[test]
    fn rejects_a_changed_fixed_source_span() {
        let result = generate_from_bytes(
            &root(),
            b"not the fixed Explorer source",
            &[(
                "en.json".to_string(),
                br#"{"menu":{"open_workspace":"Open Workspace"}}"#.to_vec(),
            )],
        );
        assert!(result.is_err());
    }

    #[test]
    fn rejects_empty_or_invalid_locale_labels() {
        let source = (1..=60)
            .map(|line| {
                if (52..=60).contains(&line) {
                    "egui::Button::new I18nOps::get().menu.open_workspace pick_open_workspace"
                } else {
                    ""
                }
            })
            .collect::<Vec<_>>()
            .join("\n");
        let empty = generate_from_bytes(
            &root(),
            source.as_bytes(),
            &[(
                "ja.json".into(),
                br#"{"menu":{"open_workspace":""}}"#.to_vec(),
            )],
        );
        assert!(empty.is_err());
        let invalid = generate_from_bytes(
            &root(),
            source.as_bytes(),
            &[("ja.json".into(), b"not json".to_vec())],
        );
        assert!(invalid.is_err());
    }
}
