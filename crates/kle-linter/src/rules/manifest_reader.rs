use crate::diagnostics::KleLintError;
use std::path::Path;

const COMMIT_HASH_LENGTH: usize = 40;

pub(super) struct ManifestReader;

impl ManifestReader {
    pub(super) fn read(path: &Path) -> Result<toml::Value, KleLintError> {
        let source = std::fs::read_to_string(path).map_err(|source| KleLintError::Read {
            path: path.to_path_buf(),
            source,
        })?;
        toml::from_str(&source).map_err(|source| KleLintError::TomlParse {
            path: path.to_path_buf(),
            source,
        })
    }

    pub(super) fn dependency_names(manifest: &toml::Value) -> Vec<String> {
        let mut names = Vec::new();
        for table in ["dependencies", "dev-dependencies", "build-dependencies"] {
            Self::push_dependency_table(manifest, table, &mut names);
        }
        names
    }

    pub(super) fn dependency_value<'a>(
        manifest: &'a toml::Value,
        name: &str,
    ) -> Option<&'a toml::Value> {
        manifest
            .get("dependencies")
            .and_then(toml::Value::as_table)
            .and_then(|it| it.get(name))
    }

    pub(super) fn workspace_dependency<'a>(
        manifest: &'a toml::Value,
        name: &str,
    ) -> Option<&'a toml::Value> {
        manifest
            .get("workspace")
            .and_then(|it| it.get("dependencies"))
            .and_then(toml::Value::as_table)
            .and_then(|it| it.get(name))
    }

    pub(super) fn has_git_url(dependency: &toml::Value, expected: &str) -> bool {
        dependency
            .get("git")
            .and_then(toml::Value::as_str)
            .is_some_and(|it| it == expected)
    }

    pub(super) fn has_pinned_rev(dependency: &toml::Value) -> bool {
        dependency
            .get("rev")
            .and_then(toml::Value::as_str)
            .is_some_and(Self::is_commit_hash)
    }

    pub(super) fn has_version(dependency: &toml::Value) -> bool {
        dependency.is_str()
            || dependency
                .get("version")
                .and_then(toml::Value::as_str)
                .is_some()
    }

    pub(super) fn is_workspace_dependency(dependency: &toml::Value) -> bool {
        dependency
            .get("workspace")
            .and_then(toml::Value::as_bool)
            .is_some_and(|it| it)
    }

    fn push_dependency_table(manifest: &toml::Value, table: &str, names: &mut Vec<String>) {
        let Some(dependencies) = manifest.get(table).and_then(toml::Value::as_table) else {
            return;
        };
        for (key, value) in dependencies {
            names.push(key.to_string());
            if let Some(package) = value.get("package").and_then(toml::Value::as_str) {
                names.push(package.to_string());
            }
        }
    }

    fn is_commit_hash(rev: &str) -> bool {
        rev.len() == COMMIT_HASH_LENGTH && rev.chars().all(|it| it.is_ascii_hexdigit())
    }
}
