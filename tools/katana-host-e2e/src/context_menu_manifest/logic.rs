impl ContextMenuManifest {
    pub fn cases(&self) -> &[ContextMenuCaseDescriptor] {
        &self.cases
    }

    pub const fn routes() -> [ContextMenuRoute; ROUTE_COUNT] {
        [
            ContextMenuRoute::SecondaryPointer,
            ContextMenuRoute::ShiftF10,
            ContextMenuRoute::AccessKitInvoke,
        ]
    }
}
impl ContextMenuManifestLoader {
    pub fn load(
        manifest_path: impl AsRef<Path>,
        profile_path: impl AsRef<Path>,
        fixed_source: impl AsRef<Path>,
    ) -> Result<ContextMenuManifest, ContextMenuManifestError> {
        let manifest_bytes = std::fs::read(manifest_path.as_ref())
            .map_err(|_| ContextMenuManifestError::Unavailable)?;
        let raw: RawManifest = serde_json::from_slice(&manifest_bytes)
            .map_err(|_| ContextMenuManifestError::Malformed)?;
        let profile_bytes = std::fs::read(profile_path.as_ref())
            .map_err(|_| ContextMenuManifestError::Unavailable)?;
        let profile: ProfileRoot = serde_json::from_slice(&profile_bytes)
            .map_err(|_| ContextMenuManifestError::Malformed)?;
        validate_root(&raw, &profile)?;
        validate_fixed_source(fixed_source.as_ref())?;
        let expected = Expected::from_source(fixed_source.as_ref())?;
        expected.validate(&raw)?;
        Ok(ContextMenuManifest {
            katana_revision: raw.katana_revision,
            profile_fingerprint: raw.profile_fingerprint,
            source_closure_fingerprint: raw.source_closure_fingerprint,
            surface_source_span_digest: raw.surface.source_span_digest,
            surface_role: raw.surface.role,
            menu_path_digest: raw.menu.path_digest,
            cases: raw
                .leaves
                .into_iter()
                .map(|leaf| ContextMenuCaseDescriptor {
                    path_digest: leaf.path_digest,
                    parent_path_digest: leaf.parent_path_digest,
                    source_span_digest: leaf.source_span_digest,
                    enabled_condition_digest: leaf.enabled_condition_digest,
                    locale_label_digests: leaf.locale_label_digests,
                })
                .collect(),
        })
    }
}
impl fmt::Display for ContextMenuManifestError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Unavailable => "context menu manifest is unavailable",
            Self::Malformed => "context menu manifest is malformed",
            Self::Invalid => "context menu manifest failed validation",
        })
    }
}

impl std::error::Error for ContextMenuManifestError {}
fn validate_root(raw: &RawManifest, profile: &ProfileRoot) -> Result<(), ContextMenuManifestError> {
    let profile_fingerprint = profile
        .root
        .get("release_profile_matrix_fingerprint")
        .and_then(serde_json::Value::as_str)
        .ok_or(ContextMenuManifestError::Invalid)?;
    let source_fingerprint = profile
        .root
        .get("katana_tree_fingerprint")
        .and_then(serde_json::Value::as_str)
        .ok_or(ContextMenuManifestError::Invalid)?;
    if raw.schema_version != SCHEMA_VERSION
        || raw.generated_by != GENERATED_BY
        || raw.katana_revision != FIXED_REVISION
        || raw.profile_fingerprint != profile_fingerprint
        || raw.source_closure_fingerprint != source_fingerprint
        || !is_digest(&raw.profile_fingerprint)
        || !is_digest(&raw.source_closure_fingerprint)
        || raw.surface.role != "MultilineTextInput"
        || raw.menu.role != "Menu"
        || !is_digest(&raw.surface.source_span_digest)
        || !is_digest(&raw.menu.source_span_digest)
        || !is_digest(&raw.menu.path_digest)
        || !matches_routes(&raw.routes)
    {
        return Err(ContextMenuManifestError::Invalid);
    }
    Ok(())
}

fn matches_routes(routes: &[Route]) -> bool {
    matches!(
        routes,
        [
            Route::SecondaryPointer,
            Route::ShiftF10,
            Route::AccesskitInvoke
        ]
    )
}
