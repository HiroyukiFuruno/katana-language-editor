#[derive(Debug)]
pub(super) struct ParsedSpan {
    pub(super) start_line: usize,
    pub(super) start_column: usize,
    pub(super) end_line: usize,
    pub(super) end_column: usize,
}

pub(super) fn branch_condition(
    kind: &str,
    to_symbol: Option<&str>,
    source: &str,
    span: &ParsedSpan,
) -> Result<String, String> {
    match kind {
        "cfg" => to_symbol
            .map(str::trim)
            .filter(|predicate| !predicate.is_empty())
            .map(std::string::ToString::to_string)
            .ok_or_else(|| "cfg branch is missing predicate".to_string()),
        kind if super::is_branch_kind(kind) => extract_span_text(source, span),
        _ => Err(format!("unsupported branch kind: {kind}")),
    }
}

impl ParsedSpan {
    pub(super) fn parse(span: &str) -> Result<Self, String> {
        let span = span
            .strip_prefix("katana:")
            .ok_or_else(|| format!("branch span is not a KatanA span: {span}"))?;
        let (start, end) = span
            .rsplit_once('-')
            .ok_or_else(|| format!("branch span is missing range: {span}"))?;
        let (file_and_line, start_column) = start
            .rsplit_once(':')
            .ok_or_else(|| format!("branch span is missing start column: {span}"))?;
        let (file, start_line) = file_and_line
            .rsplit_once(':')
            .ok_or_else(|| format!("branch span is missing start line: {span}"))?;
        if file.is_empty() {
            return Err(format!("branch span is missing file: {span}"));
        }
        let start_line = start_line
            .parse()
            .map_err(|error| format!("branch span start line is invalid: {error}"))?;
        let start_column = start_column
            .parse()
            .map_err(|error| format!("branch span start column is invalid: {error}"))?;
        let (end_line, end_column) = parse_line_column(end)?;
        Ok(Self {
            start_line,
            start_column,
            end_line,
            end_column,
        })
    }
}

fn extract_span_text(source: &str, span: &ParsedSpan) -> Result<String, String> {
    let start = line_col_offset(source, span.start_line, span.start_column)?;
    let end = line_col_offset(source, span.end_line, span.end_column)?;
    let text = source
        .get(start..end)
        .ok_or_else(|| "branch span is outside source".to_string())?
        .trim();
    non_empty_condition(text, "branch syntax")
}

fn non_empty_condition(value: &str, label: &str) -> Result<String, String> {
    if value.is_empty() {
        Err(format!("{label} is empty"))
    } else {
        Ok(value.to_string())
    }
}

fn line_col_offset(source: &str, line: usize, column: usize) -> Result<usize, String> {
    if line == 0 {
        return Err("branch span line is zero".to_string());
    }
    let line_start = source
        .split_inclusive('\n')
        .scan(0usize, |offset, segment| {
            let start = *offset;
            *offset += segment.len();
            Some((start, segment))
        })
        .nth(line - 1)
        .or_else(|| {
            source
                .ends_with('\n')
                .then_some((source.len(), ""))
                .filter(|_| line == source.lines().count() + 1)
        })
        .ok_or_else(|| format!("branch span line {line} is outside source"))?;
    let (start, segment) = line_start;
    let content = segment.trim_end_matches(['\n', '\r']);
    let byte_offset = content
        .char_indices()
        .nth(column)
        .map(|(offset, _)| offset)
        .or_else(|| (column == content.chars().count()).then_some(content.len()))
        .ok_or_else(|| format!("branch span column {column} is outside line {line}"))?;
    Ok(start + byte_offset)
}

fn parse_line_column(value: &str) -> Result<(usize, usize), String> {
    let (line, column) = value
        .split_once(':')
        .ok_or_else(|| format!("branch span coordinate is invalid: {value}"))?;
    Ok((
        line.parse()
            .map_err(|error| format!("branch span line is invalid: {error}"))?,
        column
            .parse()
            .map_err(|error| format!("branch span column is invalid: {error}"))?,
    ))
}
