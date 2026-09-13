impl ContextMenuSourceAudit {
    pub fn load(root: impl AsRef<Path>) -> Result<Self, ContextMenuSourceAuditError> {
        let root = root.as_ref().to_path_buf();
        if !root.is_dir() {
            return Err(ContextMenuSourceAuditError::MissingRoot(root));
        }
        let revision = git_revision(&root)?;
        if revision != FIXED_REVISION {
            return Err(ContextMenuSourceAuditError::RevisionMismatch {
                expected: FIXED_REVISION,
                actual: revision,
            });
        }
        let source = read_source(&root, CONTEXT_MENU_SOURCE)?;
        let ingest_source = read_source(&root, CONTEXT_MENU_INGEST_SOURCE)?;
        Ok(Self {
            fixed_root: root,
            revision: FIXED_REVISION.to_owned(),
            source,
            ingest_source,
        })
    }

    pub fn assert_complete_source_inventory(&self) -> Result<(), ContextMenuSourceAuditError> {
        for anchor in [
            "response.context_menu(|ui|",
            "AppAction::SaveDocument",
            "AppAction::FormatMarkdownFile",
            "AppAction::AuthorMarkdown",
            "CodeBlockMenuOps::show",
            "command_ingest_image_file",
            "command_ingest_clipboard_image",
            "AppAction::IngestImageFile",
            "AppAction::IngestClipboardImage",
        ] {
            if !self.source.contains(anchor) && !self.ingest_source.contains(anchor) {
                return Err(ContextMenuSourceAuditError::MissingAnchor(anchor));
            }
        }
        Ok(())
    }

    pub fn source_contains(&self, anchor: &str) -> bool {
        self.source.contains(anchor) || self.ingest_source.contains(anchor)
    }

    pub fn source_label_is_present(&self, label: &str) -> bool {
        self.source.contains(label) || self.ingest_source.contains(label)
    }

    pub fn assert_root_order_and_format_visibility(
        &self,
    ) -> Result<(), ContextMenuSourceAuditError> {
        self.assert_complete_source_inventory()?;
        let save = self.source.find("AppAction::SaveDocument").ok_or(
            ContextMenuSourceAuditError::MissingAnchor("AppAction::SaveDocument"),
        )?;
        let format = self.source.find("AppAction::FormatMarkdownFile").ok_or(
            ContextMenuSourceAuditError::MissingAnchor("AppAction::FormatMarkdownFile"),
        )?;
        if save >= format {
            return Err(ContextMenuSourceAuditError::InvalidRootOrder);
        }
        Ok(())
    }

    pub fn unavailable_host_bridge_error(&self) -> ContextMenuHostE2eError {
        ContextMenuHostE2eError::UnavailableBridge {
            fixed_revision: self.revision.clone(),
            evidence: "KatanaApp::new(AppState) does not expose document bootstrap or editor-target input; FixedHostSession only renders with RawInput::default(), while editor/context-menu rendering and AppAction assignment remain internal to the fixed source",
        }
    }
}

fn read_source(root: &Path, relative: &str) -> Result<String, ContextMenuSourceAuditError> {
    let path = root.join(relative);
    fs::read_to_string(&path).map_err(|_| ContextMenuSourceAuditError::MissingSource(path))
}

fn git_revision(root: &Path) -> Result<String, ContextMenuSourceAuditError> {
    let output = crate::system::ProcessService::create_command("git")
        .args(["rev-parse", "HEAD"])
        .current_dir(root)
        .output()
        .map_err(|error| ContextMenuSourceAuditError::Git(error.to_string()))?;
    if !output.status.success() {
        return Err(ContextMenuSourceAuditError::Git(
            String::from_utf8_lossy(&output.stderr).trim().to_owned(),
        ));
    }
    Ok(String::from_utf8_lossy(&output.stdout).trim().to_owned())
}
