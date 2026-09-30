use serde::{Deserialize, Serialize};

#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TextDirection {
    Ltr,
    Rtl,
}

#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Locale {
    pub language: String,
    pub direction: TextDirection,
}

impl Locale {
    pub fn new(language: impl Into<String>, direction: TextDirection) -> Self {
        Self {
            language: language.into(),
            direction,
        }
    }
}

#[non_exhaustive]
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Strings {
    pub accessibility_label: String,
    pub context_copy: String,
    pub context_cut: String,
    pub context_paste: String,
    pub diagnostics_label: String,
    pub diagnostic_fix_label: String,
    pub diagnostic_fix_all_label: String,
    pub diagnostic_docs_label: String,
    pub read_only_label: String,
}

impl Strings {
    pub fn new(input: StringsInput) -> Self {
        let mut strings = Self::default();
        strings.apply_context_strings(&input);
        strings.apply_diagnostic_strings(&input);
        strings
    }

    fn apply_context_strings(&mut self, input: &StringsInput) {
        self.accessibility_label = input.accessibility_label.clone();
        self.context_copy = input.context_copy.clone();
        self.context_cut = input.context_cut.clone();
        self.context_paste = input.context_paste.clone();
    }

    fn apply_diagnostic_strings(&mut self, input: &StringsInput) {
        self.diagnostics_label = input.diagnostics_label.clone();
        self.diagnostic_fix_label = input.diagnostic_fix_label.clone();
        self.diagnostic_fix_all_label = input.diagnostic_fix_all_label.clone();
        self.diagnostic_docs_label = input.diagnostic_docs_label.clone();
        self.read_only_label = input.read_only_label.clone();
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct StringsInput {
    pub accessibility_label: String,
    pub context_copy: String,
    pub context_cut: String,
    pub context_paste: String,
    pub diagnostics_label: String,
    pub diagnostic_fix_label: String,
    pub diagnostic_fix_all_label: String,
    pub diagnostic_docs_label: String,
    pub read_only_label: String,
}

pub type EditorStrings = Strings;
pub type EditorStringsInput = StringsInput;
