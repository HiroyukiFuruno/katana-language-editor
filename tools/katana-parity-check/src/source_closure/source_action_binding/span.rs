use super::types::ParsedSpan;

pub(super) fn parse_span(span: &str) -> Result<ParsedSpan, String> {
    let rest = span
        .strip_prefix("katana:")
        .ok_or_else(|| format!("invalid source span prefix: {span}"))?;
    let (file, rest) = rest
        .split_once(':')
        .ok_or_else(|| format!("invalid source span file: {span}"))?;
    let (start_line, rest) = rest
        .split_once(':')
        .ok_or_else(|| format!("invalid source span start line: {span}"))?;
    let (start_column, rest) = rest
        .split_once('-')
        .ok_or_else(|| format!("invalid source span start column: {span}"))?;
    let (end_line, end_column) = rest
        .split_once(':')
        .ok_or_else(|| format!("invalid source span end: {span}"))?;
    let parsed = ParsedSpan {
        file: file.to_string(),
        start_line: parse_positive(start_line, span)?,
        start_column: parse_positive(start_column, span)?,
        end_line: parse_positive(end_line, span)?,
        end_column: parse_positive(end_column, span)?,
    };
    if (parsed.start_line, parsed.start_column) > (parsed.end_line, parsed.end_column) {
        return Err(format!("source span is reversed: {span}"));
    }
    Ok(parsed)
}

fn parse_positive(value: &str, span: &str) -> Result<usize, String> {
    let value = value
        .parse::<usize>()
        .map_err(|error| format!("invalid source span number in {span}: {error}"))?;
    if value == 0 {
        return Err(format!("source span number is zero: {span}"));
    }
    Ok(value)
}
