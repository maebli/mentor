//! Shared building blocks for the tools' text formats.
//!
//! Every editor tool uses the same idea: a small, line-oriented, indentation
//! based text format that parses into a model and reports issues by line. This
//! module provides the scanner and the issue type so each tool only has to
//! describe its own keywords.

/// A problem found while parsing, anchored to a 1-based line number.
#[derive(Clone, PartialEq, Eq)]
pub struct ParseIssue {
    pub line: usize,
    pub message: String,
    pub severity: Severity,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Severity {
    /// The text can't be understood here.
    Error,
    /// Understood, but probably not what you meant.
    Warning,
}

impl ParseIssue {
    pub fn error(line: usize, message: impl Into<String>) -> Self {
        Self { line, message: message.into(), severity: Severity::Error }
    }
    pub fn warn(line: usize, message: impl Into<String>) -> Self {
        Self { line, message: message.into(), severity: Severity::Warning }
    }
}

/// One meaningful line of input after blanks and `#` comments are stripped.
#[derive(Clone)]
pub struct Line {
    /// 1-based line number in the original source.
    pub number: usize,
    /// Indentation depth, in levels of two spaces (tabs count as one level).
    pub indent: usize,
    /// The trimmed text of the line.
    pub content: String,
}

impl Line {
    /// Split `content` into a leading keyword and the rest, e.g.
    /// `"responsibility calculate total"` -> `("responsibility", "calculate total")`.
    pub fn keyword(&self) -> (&str, &str) {
        match self.content.split_once(char::is_whitespace) {
            Some((k, rest)) => (k, rest.trim()),
            None => (self.content.as_str(), ""),
        }
    }

    /// Split on the first colon, e.g. `"resp: calculate total"` ->
    /// `("resp", "calculate total")`. Falls back to the whole line as the key.
    #[allow(dead_code)] // used by tools still to be built
    pub fn colon(&self) -> (&str, &str) {
        match self.content.split_once(':') {
            Some((k, rest)) => (k.trim(), rest.trim()),
            None => (self.content.as_str(), ""),
        }
    }
}

/// Turn raw text into meaningful lines, computing indentation depth. Blank
/// lines and lines whose first non-space character is `#` are dropped.
pub fn scan(input: &str) -> Vec<Line> {
    let mut out = Vec::new();
    for (i, raw) in input.lines().enumerate() {
        let trimmed = raw.trim_start();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        let indent = indent_level(raw);
        out.push(Line {
            number: i + 1,
            indent,
            content: trimmed.trim_end().to_string(),
        });
    }
    out
}

/// Indentation in "levels": a tab is one level, and every two leading spaces is
/// one level. Mixed leading whitespace rounds down.
fn indent_level(raw: &str) -> usize {
    let mut spaces = 0usize;
    let mut level = 0usize;
    for ch in raw.chars() {
        match ch {
            '\t' => {
                level += 1;
                spaces = 0;
            }
            ' ' => {
                spaces += 1;
                if spaces == 2 {
                    level += 1;
                    spaces = 0;
                }
            }
            _ => break,
        }
    }
    level
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn skips_blanks_and_comments() {
        let lines = scan("a\n\n# note\n  b\n");
        assert_eq!(lines.len(), 2);
        assert_eq!(lines[0].content, "a");
        assert_eq!(lines[0].indent, 0);
        assert_eq!(lines[1].content, "b");
        assert_eq!(lines[1].indent, 1);
    }

    #[test]
    fn keyword_and_colon_split() {
        let lines = scan("responsibility calculate total\nresp: calculate total\n");
        assert_eq!(lines[0].keyword(), ("responsibility", "calculate total"));
        assert_eq!(lines[1].colon(), ("resp", "calculate total"));
    }
}
