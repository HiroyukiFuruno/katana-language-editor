use super::super::super::edge_model::ExternalUiDependency;
use super::validate;
#[path = "external_fixture.rs"]
mod fixture;
use fixture::{dependency, rebuild_fingerprint};

fn errors(dependencies: Vec<ExternalUiDependency>) -> Vec<String> {
    let mut errors = Vec::new();
    validate(&dependencies, &mut errors);
    errors
}

#[test]
fn rejects_missing_and_duplicate_egui_packages() -> Result<(), Box<dyn std::error::Error>> {
    let missing = errors(vec![dependency("other")?]);
    assert!(missing.iter().any(|error| error.contains("missing")));
    let duplicate = errors(vec![dependency("egui")?, dependency("egui")?]);
    assert!(duplicate.iter().any(|error| error.contains("duplicate")));
    Ok(())
}

#[test]
fn accepts_complete_synthetic_shape() -> Result<(), Box<dyn std::error::Error>> {
    let rejected = errors(vec![dependency("egui")?]);
    assert!(rejected.is_empty(), "{rejected:?}");
    Ok(())
}

#[test]
fn rejects_unfinished_external_dependency_evidence() -> Result<(), Box<dyn std::error::Error>> {
    let mut value = dependency("egui")?;
    value.complete = false;
    value.unresolved_evidence.push("not captured".into());
    value.replacement_leafs.clear();
    let rejected = errors(vec![value]);

    assert!(rejected.iter().any(|error| error.contains("incomplete")));
    assert!(rejected.iter().any(|error| error.contains("unresolved")));
    assert!(rejected.iter().any(|error| error.contains("replacement")));
    Ok(())
}

#[test]
fn rejects_fake_complete_dependency_with_empty_evidence() -> Result<(), Box<dyn std::error::Error>>
{
    let mut value = dependency("egui")?;
    value.version.clear();
    value.source_files.clear();
    value.symbols.clear();
    value.symbol_spans.clear();
    value.katana_invoking_edges.clear();
    value.fingerprint.clear();
    let rejected = errors(vec![value]);

    assert!(rejected.iter().any(|error| error.contains("metadata")));
    assert!(rejected.iter().any(|error| error.contains("source")));
    assert!(
        rejected
            .iter()
            .any(|error| error.contains("symbol evidence"))
    );
    assert!(rejected.iter().any(|error| error.contains("symbol span")));
    assert!(rejected.iter().any(|error| error.contains("invocation")));
    assert!(rejected.iter().any(|error| error.contains("fingerprint")));
    Ok(())
}

fn assert_whitespace_field<F>(
    mutate: F,
    expected_reason: &str,
) -> Result<(), Box<dyn std::error::Error>>
where
    F: FnOnce(&mut ExternalUiDependency),
{
    let mut value = dependency("egui")?;
    mutate(&mut value);
    let value = rebuild_fingerprint(value)?;
    let rejected = errors(vec![value]);
    assert!(!rejected.iter().any(|error| error.contains("fingerprint")));
    assert!(
        rejected.iter().any(|error| error.contains(expected_reason)),
        "{rejected:?}"
    );
    Ok(())
}

#[test]
fn rejects_whitespace_metadata_and_sources() -> Result<(), Box<dyn std::error::Error>> {
    assert_whitespace_field(|value| value.package = " ".into(), "package name")?;
    assert_whitespace_field(|value| value.version = " ".into(), "metadata")?;
    assert_whitespace_field(|value| value.source = " \t".into(), "metadata")?;
    assert_whitespace_field(|value| value.package_checksum = "\n".into(), "metadata")?;
    assert_whitespace_field(|value| value.lock_sha256 = "\t".into(), "metadata")?;
    assert_whitespace_field(|value| value.source_files[0].path = " ".into(), "source")?;
    assert_whitespace_field(|value| value.source_files[0].sha256 = "\n".into(), "source")?;
    Ok(())
}

#[test]
fn rejects_whitespace_symbols_and_spans() -> Result<(), Box<dyn std::error::Error>> {
    assert_whitespace_field(|value| value.symbols[0] = "\t".into(), "symbol evidence")?;
    assert_whitespace_field(
        |value| value.symbol_spans[0].symbol = " ".into(),
        "symbol span",
    )?;
    Ok(())
}

#[test]
fn rejects_whitespace_invocation_origin() -> Result<(), Box<dyn std::error::Error>> {
    assert_whitespace_field(
        |value| value.symbol_spans[0].span = "\n".into(),
        "symbol span",
    )?;
    assert_whitespace_field(
        |value| value.katana_invoking_edges[0].kind = " ".into(),
        "invocation",
    )?;
    assert_whitespace_field(
        |value| value.katana_invoking_edges[0].source_file = "\t".into(),
        "invocation",
    )?;
    assert_whitespace_field(
        |value| value.katana_invoking_edges[0].span = "\n".into(),
        "invocation",
    )?;
    Ok(())
}

#[test]
fn rejects_whitespace_invocation_symbols() -> Result<(), Box<dyn std::error::Error>> {
    assert_whitespace_field(
        |value| value.katana_invoking_edges[0].katana_symbol = " ".into(),
        "invocation",
    )?;
    assert_whitespace_field(
        |value| value.katana_invoking_edges[0].dependency_symbol = "\t".into(),
        "invocation",
    )?;
    Ok(())
}

#[test]
fn rejects_whitespace_replacement_and_unresolved() -> Result<(), Box<dyn std::error::Error>> {
    assert_whitespace_field(
        |value| value.replacement_leafs[0] = " ".into(),
        "replacement",
    )?;
    assert_whitespace_field(
        |value| value.unresolved_evidence.push(" \t".into()),
        "unresolved",
    )?;
    Ok(())
}

#[test]
fn rejects_fingerprint_mutation() -> Result<(), Box<dyn std::error::Error>> {
    let mut value = dependency("egui")?;
    value.fingerprint.push('x');
    let rejected = errors(vec![value]);
    assert!(rejected.iter().any(|error| error.contains("fingerprint")));
    Ok(())
}
