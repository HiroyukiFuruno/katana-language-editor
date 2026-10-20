use super::{generation::scan_alias, model::AnchorKind, scan::scan_file};

const FIXED_SOURCE: &str = include_str!("source_anchor/fixtures/anchors.rs");

#[test]
fn anchors_retain_exact_utf8_byte_spans_and_crlf_locations() -> Result<(), String> {
    let fixture = FIXED_SOURCE.replace('\n', "\r\n");
    let syntax = syn::parse_file(&fixture).map_err(|error| error.to_string())?;
    let (anchors, unresolved) = scan_file(&fixture, &syntax)?;
    assert!(
        anchors
            .iter()
            .any(|anchor| anchor.kind == AnchorKind::EnumVariant)
    );
    assert!(
        anchors
            .iter()
            .any(|anchor| anchor.kind == AnchorKind::TestDeclaration)
    );
    assert!(
        anchors
            .iter()
            .any(|anchor| anchor.kind == AnchorKind::FunctionDeclaration)
    );
    assert!(
        anchors
            .iter()
            .filter(|anchor| anchor.kind == AnchorKind::StructConstruction)
            .count()
            >= 2
    );
    assert!(
        unresolved
            .iter()
            .any(|record| record.reason.contains("macro"))
    );
    for anchor in anchors {
        let excerpt = &fixture[anchor.span.start_byte..anchor.span.end_byte];
        assert_eq!(
            super::super::fingerprint::sha256_hex(excerpt.as_bytes()),
            anchor.node_sha256
        );
        assert_eq!(anchor.node_sha256, anchor.excerpt_sha256);
    }

    let same_line =
        "fn sample() { let _prefix = \"日本語✈️\"; let _entry = Entry { value: 2 }; }\r\n";
    let syntax = syn::parse_file(same_line).map_err(|error| error.to_string())?;
    let (anchors, _) = scan_file(same_line, &syntax)?;
    let excerpt = "Entry { value: 2 }";
    let expected_start = same_line
        .find(excerpt)
        .ok_or("expected construction literal is absent")?;
    let expected_end = expected_start + excerpt.len();
    let expected_column =
        |offset: usize| same_line[..offset].rsplit('\n').next().unwrap_or("").len();
    let construction = anchors
        .iter()
        .find(|anchor| anchor.kind == AnchorKind::StructConstruction && anchor.name == "Entry")
        .ok_or("same-line construction anchor is absent")?;
    assert_eq!(construction.span.start_byte, expected_start);
    assert_eq!(construction.span.end_byte, expected_end);
    assert_eq!(construction.span.start_line, 1);
    assert_eq!(construction.span.end_line, 1);
    assert_eq!(
        construction.span.start_column,
        expected_column(expected_start)
    );
    assert_eq!(construction.span.end_column, expected_column(expected_end));
    assert_eq!(&same_line[expected_start..expected_end], excerpt);
    assert_eq!(
        construction.node_sha256,
        super::super::fingerprint::sha256_hex(excerpt.as_bytes())
    );
    assert_eq!(construction.excerpt_sha256, construction.node_sha256);
    let fixture_syntax = syn::parse_file(&fixture).map_err(|error| error.to_string())?;
    let variant = scan_alias(&fixture, &fixture_syntax, "REQ-1", "Mode::開く")?
        .into_iter()
        .find(|anchor| anchor.kind == AnchorKind::EnumVariant)
        .ok_or("missing enum variant anchor")?;
    assert_eq!(variant.requirement_id, "REQ-1");
    assert_eq!(variant.short_reference, "Mode::開く");
    assert_eq!(variant.span.start_line, 2);
    Ok(())
}

#[test]
fn repeat_vec_is_recovered_once_and_keeps_macro_uncertainty() -> Result<(), String> {
    let source = "fn sample() { let _items = vec![Thing { value: 1 }; count]; }\n";
    let syntax = syn::parse_file(source).map_err(|error| error.to_string())?;
    let (anchors, unresolved) = scan_file(source, &syntax)?;
    assert_eq!(
        anchors
            .iter()
            .filter(|anchor| anchor.kind == AnchorKind::StructConstruction)
            .count(),
        1
    );
    assert_eq!(unresolved.len(), 1);
    assert!(unresolved[0].reason.contains("recovered once"));
    Ok(())
}

#[test]
fn malformed_vec_repeat_remains_unresolved() -> Result<(), String> {
    let source = "fn sample() { let _items = vec![value; count, extra]; }\n";
    let syntax = syn::parse_file(source).map_err(|error| error.to_string())?;
    let (_, unresolved) = scan_file(source, &syntax)?;
    assert_eq!(unresolved.len(), 1);
    assert!(unresolved[0].reason.contains("malformed"));
    Ok(())
}

#[test]
fn trait_method_facts_keep_trait_owner_and_generic_impl_identity_unresolved() -> Result<(), String>
{
    let source = "trait Renderer { fn render(&self); fn fallback(&self) {} }\nimpl<T> Renderer for Wrapper<T> { fn render(&self) {} }\n";
    let syntax = syn::parse_file(source).map_err(|error| error.to_string())?;
    let (anchors, unresolved) = scan_file(source, &syntax)?;
    for name in ["render", "fallback"] {
        assert!(anchors.iter().any(|anchor| {
            anchor.kind == AnchorKind::FunctionDeclaration
                && anchor.owner == "Renderer"
                && anchor.name == name
        }));
    }
    assert!(unresolved.iter().any(
        |record| record.reason.contains("generic impl owner") && record.owner == "impl Wrapper"
    ));
    Ok(())
}
