use serde::{Deserialize, Serialize};

#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TokenKind {
    Keyword,
    String,
    Comment,
    Number,
    Operator,
    Heading,
    Code,
    Default,
    Custom(String),
}

#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HighlightedSpan {
    pub start: usize,
    pub end: usize,
    pub token_kind: TokenKind,
}

impl HighlightedSpan {
    pub const fn new(start: usize, end: usize, token_kind: TokenKind) -> Self {
        Self {
            start,
            end,
            token_kind,
        }
    }
}

#[non_exhaustive]
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct HighlightedText {
    pub spans: Vec<HighlightedSpan>,
}

pub trait SyntaxHighlighter: Send + Sync {
    fn highlight(&self, source: &str) -> HighlightedText;
}

#[derive(Debug)]
pub struct NoopSyntaxHighlighter;

impl SyntaxHighlighter for NoopSyntaxHighlighter {
    fn highlight(&self, _source: &str) -> HighlightedText {
        HighlightedText::default()
    }
}
