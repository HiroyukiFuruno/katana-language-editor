use super::super::edge_model::ExternalUiDependency;
use super::super::fingerprint::sha256_hex;
use super::super::validator_helpers::duplicates;

#[cfg(test)]
#[path = "external_tests.rs"]
mod external_tests;

const REQUIRED_PACKAGE: &str = "egui";

pub(super) fn validate(dependencies: &[ExternalUiDependency], errors: &mut Vec<String>) {
    if !dependencies
        .iter()
        .any(|dependency| dependency.package == REQUIRED_PACKAGE)
    {
        errors.push("external UI dependency egui is missing".into());
    }
    if duplicates(dependencies.iter().map(|dependency| &dependency.package)) {
        errors.push("external UI dependency has duplicate package entries".into());
    }
    for dependency in dependencies {
        validate_package_name(dependency, errors);
        validate_completion(dependency, errors);
        validate_evidence(dependency, errors);
    }
}

fn package_name(dependency: &ExternalUiDependency) -> &str {
    let package = dependency.package.trim();
    if package.is_empty() {
        "<empty>"
    } else {
        package
    }
}

fn validate_package_name(dependency: &ExternalUiDependency, errors: &mut Vec<String>) {
    if dependency.package.trim().is_empty() {
        errors.push("external UI dependency package name is empty".into());
    }
}

fn validate_completion(dependency: &ExternalUiDependency, errors: &mut Vec<String>) {
    let package = package_name(dependency);
    if !dependency.complete {
        errors.push(format!("external UI dependency {package} is incomplete"));
    }
    if !dependency.unresolved_evidence.is_empty() {
        errors.push(format!(
            "external UI dependency {package} has unresolved evidence"
        ));
    }
    if dependency.replacement_leafs.is_empty()
        || dependency
            .replacement_leafs
            .iter()
            .any(|leaf| leaf.trim().is_empty())
    {
        errors.push(format!(
            "external UI dependency {package} has no replacement leafs"
        ));
    }
}

fn validate_evidence(dependency: &ExternalUiDependency, errors: &mut Vec<String>) {
    validate_metadata(dependency, errors);
    validate_sources(dependency, errors);
    validate_symbols(dependency, errors);
    validate_spans(dependency, errors);
    validate_invocations(dependency, errors);
    validate_fingerprint(dependency, errors);
}

fn validate_metadata(dependency: &ExternalUiDependency, errors: &mut Vec<String>) {
    if [
        dependency.version.as_str(),
        dependency.source.as_str(),
        dependency.package_checksum.as_str(),
        dependency.lock_sha256.as_str(),
    ]
    .iter()
    .any(|value| value.trim().is_empty())
    {
        errors.push(format!(
            "external UI dependency {} has incomplete package metadata",
            package_name(dependency)
        ));
    }
}

fn validate_sources(dependency: &ExternalUiDependency, errors: &mut Vec<String>) {
    if dependency.source_files.is_empty()
        || dependency
            .source_files
            .iter()
            .any(|file| file.path.trim().is_empty() || file.sha256.trim().is_empty())
    {
        errors.push(format!(
            "external UI dependency {} has incomplete source evidence",
            package_name(dependency)
        ));
    }
}

fn validate_symbols(dependency: &ExternalUiDependency, errors: &mut Vec<String>) {
    if dependency.symbols.is_empty()
        || dependency
            .symbols
            .iter()
            .any(|symbol| symbol.trim().is_empty())
    {
        errors.push(format!(
            "external UI dependency {} has incomplete symbol evidence",
            package_name(dependency)
        ));
    }
}

fn validate_spans(dependency: &ExternalUiDependency, errors: &mut Vec<String>) {
    if dependency.symbol_spans.is_empty()
        || dependency
            .symbol_spans
            .iter()
            .any(|span| span.symbol.trim().is_empty() || span.span.trim().is_empty())
    {
        errors.push(format!(
            "external UI dependency {} has incomplete symbol span evidence",
            package_name(dependency)
        ));
    }
}

fn validate_invocations(dependency: &ExternalUiDependency, errors: &mut Vec<String>) {
    if dependency.katana_invoking_edges.is_empty()
        || dependency.katana_invoking_edges.iter().any(|edge| {
            [
                edge.kind.as_str(),
                edge.source_file.as_str(),
                edge.span.as_str(),
                edge.katana_symbol.as_str(),
                edge.dependency_symbol.as_str(),
            ]
            .iter()
            .any(|value| value.trim().is_empty())
        })
    {
        errors.push(format!(
            "external UI dependency {} has incomplete invocation evidence",
            package_name(dependency)
        ));
    }
}

fn validate_fingerprint(dependency: &ExternalUiDependency, errors: &mut Vec<String>) {
    let mut content = dependency.clone();
    let fingerprint = content.fingerprint.clone();
    content.fingerprint.clear();
    let matches = !fingerprint.trim().is_empty()
        && serde_json::to_vec(&content)
            .map(|bytes| sha256_hex(&bytes) == fingerprint)
            .unwrap_or(false);
    if !matches {
        errors.push(format!(
            "external UI dependency {} has invalid fingerprint",
            package_name(dependency)
        ));
    }
}
