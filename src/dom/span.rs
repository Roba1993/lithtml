use serde::Serialize;

/// Span of the information in the parsed source.
#[derive(Debug, Default, Clone, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct SourceSpan<'s> {
    pub text: &'s str,
    pub start_line: usize,
    pub end_line: usize,
    pub start_column: usize,
    pub end_column: usize,
}

impl<'s> SourceSpan<'s> {
    pub fn new(
        text: &'s str,
        start_line: usize,
        end_line: usize,
        start_column: usize,
        end_column: usize,
    ) -> Self {
        Self {
            text,
            start_line,
            end_line,
            start_column,
            end_column,
        }
    }
}
