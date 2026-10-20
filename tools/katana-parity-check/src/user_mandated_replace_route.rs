use std::fs;
use std::path::{Path, PathBuf};

const UI_ROOT: &str = "crates/katana-ui/src";
const ACTION_TYPES: &str = "crates/katana-ui/src/app_action_types.rs";
const ACTION_DISPATCH: &str = "crates/katana-ui/src/app/action/dispatch.rs";
const DOCUMENT_EDIT: &str = "crates/katana-ui/src/app/document_edit.rs";
const DOCUMENT_SEARCH: &str = "crates/katana-ui/src/views/top_bar/search.rs";

pub(crate) struct UserMandatedReplaceRouteAudit;

impl UserMandatedReplaceRouteAudit {
    pub(crate) fn validate(repo_root: &Result<PathBuf, String>) -> Result<(), String> {
        let repo_root = repo_root.as_ref().map_err(Clone::clone)?;
        ReplaceRouteSources::read(repo_root)?.validate()
    }
}

struct ReplaceRouteSources {
    action_types: String,
    action_dispatch: String,
    document_edit: String,
    document_search: String,
    qualified_replace_mentions: Vec<String>,
    replace_struct_mentions: Vec<String>,
    replace_all_mentions: Vec<String>,
}

impl ReplaceRouteSources {
    fn read(repo_root: &Path) -> Result<Self, String> {
        let read = |relative: &str| read_source(&repo_root.join(relative));
        Ok(Self {
            action_types: read(ACTION_TYPES)?,
            action_dispatch: read(ACTION_DISPATCH)?,
            document_edit: read(DOCUMENT_EDIT)?,
            document_search: read(DOCUMENT_SEARCH)?,
            qualified_replace_mentions: source_mentions(repo_root, "AppAction::ReplaceText")?,
            replace_struct_mentions: source_mentions(repo_root, "ReplaceText {")?,
            replace_all_mentions: source_mentions(repo_root, "ReplaceAll")?,
        })
    }

    fn validate(&self) -> Result<(), String> {
        if !self.action_types.contains("ReplaceText {") {
            return Err("fixed KatanA is missing the ReplaceText single-span resolver".into());
        }
        if !self.replace_all_mentions.is_empty() {
            return Err(format!(
                "fixed KatanA must not expose a general ReplaceAll route; mentions: {}",
                self.replace_all_mentions.join(", ")
            ));
        }
        if !self
            .action_dispatch
            .contains("AppAction::ReplaceText { span, replacement }")
            || !self
                .action_dispatch
                .contains("self.handle_replace_text(ctx, span, replacement)")
        {
            return Err("ReplaceText dispatch must remain a single-span host resolver".into());
        }
        if !self.document_edit.contains("span: std::ops::Range<usize>")
            || !self
                .document_edit
                .contains("updated.replace_range(span, &replacement)")
        {
            return Err("ReplaceText handler must retain host-owned span replacement".into());
        }
        if self
            .document_search
            .to_ascii_lowercase()
            .contains("replace")
        {
            return Err("fixed document search must not claim a general replace control".into());
        }
        if self.qualified_replace_mentions.as_slice() != [ACTION_DISPATCH] {
            return Err(format!(
                "ReplaceText must have no fixed KatanA UI origin; qualified mentions: {}",
                self.qualified_replace_mentions.join(", ")
            ));
        }
        if self.replace_struct_mentions.as_slice() != [ACTION_DISPATCH, ACTION_TYPES] {
            return Err(format!(
                "ReplaceText construction must have no fixed KatanA UI origin; mentions: {}",
                self.replace_struct_mentions.join(", ")
            ));
        }
        Ok(())
    }
}

fn source_mentions(repo_root: &Path, needle: &str) -> Result<Vec<String>, String> {
    let mut paths = Vec::new();
    collect_rust_sources(&repo_root.join(UI_ROOT), &mut paths)?;
    paths.sort();

    let mut mentions = Vec::new();
    for path in paths {
        if !read_source(&path)?.contains(needle) {
            continue;
        }
        let relative = path
            .strip_prefix(repo_root)
            .map_err(|error| format!("source path is outside KatanA root: {error}"))?;
        mentions.push(relative.to_string_lossy().replace('\\', "/"));
    }
    Ok(mentions)
}

fn collect_rust_sources(dir: &Path, paths: &mut Vec<PathBuf>) -> Result<(), String> {
    let entries =
        fs::read_dir(dir).map_err(|error| format!("failed to read {}: {error}", dir.display()))?;
    for entry in entries {
        let entry = entry.map_err(|error| format!("failed to read directory entry: {error}"))?;
        let path = entry.path();
        let file_type = entry
            .file_type()
            .map_err(|error| format!("failed to inspect {}: {error}", path.display()))?;
        if file_type.is_dir() {
            collect_rust_sources(&path, paths)?;
        } else if file_type.is_file() && path.extension().is_some_and(|extension| extension == "rs")
        {
            paths.push(path);
        }
    }
    Ok(())
}

fn read_source(path: &Path) -> Result<String, String> {
    fs::read_to_string(path).map_err(|error| format!("failed to read {}: {error}", path.display()))
}

#[cfg(test)]
mod tests {
    use super::{ACTION_DISPATCH, ReplaceRouteSources};

    fn sources() -> ReplaceRouteSources {
        ReplaceRouteSources {
            action_types: "ReplaceText { span: Range<usize>, replacement: String }".into(),
            action_dispatch: "AppAction::ReplaceText { span, replacement } self.handle_replace_text(ctx, span, replacement)".into(),
            document_edit: "span: std::ops::Range<usize> updated.replace_range(span, &replacement)".into(),
            document_search: "DocSearchNext DocSearchPrev".into(),
            qualified_replace_mentions: vec![ACTION_DISPATCH.into()],
            replace_struct_mentions: vec![ACTION_DISPATCH.into(), super::ACTION_TYPES.into()],
            replace_all_mentions: Vec::new(),
        }
    }

    #[test]
    fn accepts_a_single_span_resolver_without_a_ui_origin() -> Result<(), String> {
        sources().validate()
    }

    #[test]
    fn rejects_a_general_replace_all_action() {
        let mut sources = sources();
        sources.replace_all_mentions.push("views/search.rs".into());
        assert!(sources.validate().is_err());
    }

    #[test]
    fn rejects_a_replace_control_origin() {
        let mut sources = sources();
        sources
            .qualified_replace_mentions
            .push("views/search.rs".into());
        assert!(sources.validate().is_err());
    }

    #[test]
    fn rejects_an_unqualified_replace_constructor() {
        let mut sources = sources();
        sources
            .replace_struct_mentions
            .push("views/search.rs".into());
        assert!(sources.validate().is_err());
    }
}
