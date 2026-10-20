use proc_macro2::Span;

use super::super::fingerprint::sha256_hex;
use super::model::{AnchorFact, AnchorKind, AnchorRelation, SourceSpan, UnresolvedSyntax};

pub(super) fn make_fact(
    source: &str,
    owner: &[String],
    kind: AnchorKind,
    name: String,
    span: Span,
) -> Result<AnchorFact, String> {
    let (span, excerpt) = span_excerpt(source, span)?;
    let excerpt_sha256 = sha256_hex(excerpt.as_bytes());
    Ok(AnchorFact {
        relation: AnchorRelation::ReferenceCandidate,
        requirement_id: String::new(),
        short_reference: String::new(),
        kind,
        owner: owner_name(owner),
        name,
        span,
        node_sha256: excerpt_sha256.clone(),
        excerpt_sha256,
    })
}

pub(super) fn make_unresolved(
    source: &str,
    owner: &[String],
    span: Span,
    reason: String,
) -> Result<UnresolvedSyntax, String> {
    let (span, excerpt) = span_excerpt(source, span)?;
    Ok(UnresolvedSyntax {
        owner: owner_name(owner),
        span,
        syntax_sha256: sha256_hex(excerpt.as_bytes()),
        reason,
    })
}

fn owner_name(owner: &[String]) -> String {
    if owner.is_empty() {
        "crate".into()
    } else {
        owner.join("::")
    }
}

fn span_excerpt(source: &str, span: Span) -> Result<(SourceSpan, &str), String> {
    let range = span.byte_range();
    if range.start > range.end
        || range.end > source.len()
        || !source.is_char_boundary(range.start)
        || !source.is_char_boundary(range.end)
    {
        return Err("AST span is outside or misaligned with its original UTF-8 source".into());
    }
    let location = |offset: usize| {
        let prefix = &source[..offset];
        let line = prefix.bytes().filter(|byte| *byte == b'\n').count() + 1;
        let column = prefix.rsplit('\n').next().unwrap_or("").len();
        (line, column)
    };
    let (start_line, start_column) = location(range.start);
    let (end_line, end_column) = location(range.end);
    Ok((
        SourceSpan {
            start_byte: range.start,
            end_byte: range.end,
            start_line,
            start_column,
            end_line,
            end_column,
        },
        &source[range],
    ))
}
