use std::fmt::Write as _;

use serde::{Deserialize, Serialize};

use aksara_ast::Span;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    Error,
    Warning,
    Info,
}

impl Severity {
    pub fn label(self) -> &'static str {
        match self {
            Severity::Error => "error",
            Severity::Warning => "peringatan",
            Severity::Info => "info",
        }
    }

    pub fn is_error(self) -> bool {
        matches!(self, Severity::Error)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Label {
    pub span: Span,
    pub message: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Diagnostic {
    pub code: String,
    pub severity: Severity,
    pub message: String,
    pub span: Span,
    pub labels: Vec<Label>,
    pub notes: Vec<String>,
    pub help: Option<String>,
}

impl Diagnostic {
    pub fn new(
        severity: Severity,
        code: impl Into<String>,
        message: impl Into<String>,
        span: Span,
    ) -> Self {
        Self {
            code: code.into(),
            severity,
            message: message.into(),
            span,
            labels: Vec::new(),
            notes: Vec::new(),
            help: None,
        }
    }

    pub fn error(code: impl Into<String>, message: impl Into<String>, span: Span) -> Self {
        Self::new(Severity::Error, code, message, span)
    }

    pub fn warning(code: impl Into<String>, message: impl Into<String>, span: Span) -> Self {
        Self::new(Severity::Warning, code, message, span)
    }

    pub fn with_label(mut self, span: Span, message: impl Into<String>) -> Self {
        self.labels.push(Label {
            span,
            message: message.into(),
        });
        self
    }

    pub fn with_note(mut self, note: impl Into<String>) -> Self {
        self.notes.push(note.into());
        self
    }

    pub fn with_help(mut self, help: impl Into<String>) -> Self {
        self.help = Some(help.into());
        self
    }

    pub fn is_error(&self) -> bool {
        self.severity.is_error()
    }
}

/// Sumber yang sudah dinormalisasi.
#[derive(Clone, Debug)]
pub struct SourceFile {
    pub name: String,
    pub text: String,
    line_starts: Vec<usize>,
}

impl SourceFile {
    pub fn new(name: impl Into<String>, text: impl Into<String>) -> Self {
        let text = normalize_source(&text.into());
        let mut line_starts = vec![0];
        for (index, byte) in text.bytes().enumerate() {
            if byte == b'\n' {
                line_starts.push(index + 1);
            }
        }
        Self {
            name: name.into(),
            text,
            line_starts,
        }
    }

    pub fn line_count(&self) -> usize {
        self.line_starts.len()
    }

    pub fn line_col(&self, offset: usize) -> (usize, usize) {
        let offset = offset.min(self.text.len());
        let line_index = match self.line_starts.binary_search(&offset) {
            Ok(index) => index,
            Err(index) => index.saturating_sub(1),
        };
        let line_start = self.line_starts[line_index];
        let col = self.text[line_start..offset].chars().count() + 1;
        (line_index + 1, col)
    }

    pub fn line_text(&self, line: usize) -> &str {
        if line == 0 || line > self.line_starts.len() {
            return "";
        }
        let start = self.line_starts[line - 1];
        let end = self
            .line_starts
            .get(line)
            .copied()
            .unwrap_or(self.text.len());
        self.text[start..end].trim_end_matches('\n')
    }

    pub fn line_span(&self, line: usize) -> Span {
        if line == 0 || line > self.line_starts.len() {
            return Span::empty(0);
        }
        let start = self.line_starts[line - 1];
        let end = self
            .line_starts
            .get(line)
            .copied()
            .unwrap_or(self.text.len());
        Span::new(start as u32, end as u32)
    }
}

pub fn normalize_source(text: &str) -> String {
    let text = text.strip_prefix('\u{feff}').unwrap_or(text);
    let mut out = String::with_capacity(text.len());
    let mut chars = text.chars().peekable();
    while let Some(ch) = chars.next() {
        match ch {
            '\r' => {
                if chars.peek() == Some(&'\n') {
                    chars.next();
                }
                out.push('\n');
            }
            _ => out.push(ch),
        }
    }
    out
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ColorMode {
    Auto,
    Always,
    Never,
}

impl ColorMode {
    pub fn enabled(self) -> bool {
        match self {
            ColorMode::Always => true,
            ColorMode::Never => false,
            ColorMode::Auto => std::io::IsTerminal::is_terminal(&std::io::stderr()),
        }
    }
}

const RESET: &str = "\x1b[0m";
const BOLD: &str = "\x1b[1m";
const RED: &str = "\x1b[31m";
const YELLOW: &str = "\x1b[33m";
const BLUE: &str = "\x1b[34m";
const CYAN: &str = "\x1b[36m";

pub fn render_human(diagnostic: &Diagnostic, source: &SourceFile, color: bool) -> String {
    let (line, col) = source.line_col(diagnostic.span.start as usize);
    let severity_color = match diagnostic.severity {
        Severity::Error => RED,
        Severity::Warning => YELLOW,
        Severity::Info => CYAN,
    };
    let mut out = String::new();
    if color {
        let _ = writeln!(
            out,
            "{BOLD}{severity_color}{}[{}]{RESET}{BOLD}: {}{RESET}",
            diagnostic.severity.label(),
            diagnostic.code,
            diagnostic.message
        );
    } else {
        let _ = writeln!(
            out,
            "{}[{}]: {}",
            diagnostic.severity.label(),
            diagnostic.code,
            diagnostic.message
        );
    }
    if color {
        let _ = writeln!(
            out,
            "  {BLUE}-->{RESET} {}:{line}:{col}",
            source.name
        );
    } else {
        let _ = writeln!(out, "  --> {}:{line}:{col}", source.name);
    }

    let line_text = source.line_text(line);
    let line_number = line.to_string();
    let gutter = " ".repeat(line_number.len());
    if color {
        let _ = writeln!(out, "{gutter} {BLUE}|{RESET}");
    } else {
        let _ = writeln!(out, "{gutter} |");
    }
    if color {
        let _ = writeln!(out, "{BLUE}{line_number} |{RESET} {line_text}");
    } else {
        let _ = writeln!(out, "{line_number} | {line_text}");
    }

    let line_span = source.line_span(line);
    let underline_start = diagnostic.span.start.max(line_span.start);
    let underline_end = diagnostic
        .span
        .end
        .min(line_span.end.saturating_sub(1))
        .max(underline_start);
    let start_col = source
        .text
        .get(line_span.start as usize..underline_start as usize)
        .map(|s| s.chars().count())
        .unwrap_or(0);
    let width = source
        .text
        .get(underline_start as usize..underline_end as usize)
        .map(|s| s.chars().count())
        .unwrap_or(1)
        .max(1);
    let caret = "^".repeat(width);
    let padding = " ".repeat(start_col);
    if color {
        let _ = writeln!(
            out,
            "{gutter} {BLUE}|{RESET} {padding}{severity_color}{caret}{RESET} {}",
            diagnostic
                .labels
                .first()
                .map(|label| label.message.as_str())
                .unwrap_or("")
        );
    } else {
        let _ = writeln!(
            out,
            "{gutter} | {padding}{caret} {}",
            diagnostic
                .labels
                .first()
                .map(|label| label.message.as_str())
                .unwrap_or("")
        );
    }

    for note in &diagnostic.notes {
        if color {
            let _ = writeln!(out, "{gutter} {BLUE}={RESET} catatan: {note}");
        } else {
            let _ = writeln!(out, "{gutter} = catatan: {note}");
        }
    }
    if let Some(help) = &diagnostic.help {
        if color {
            let _ = writeln!(out, "{gutter} {BLUE}={RESET} bantuan: {help}");
        } else {
            let _ = writeln!(out, "{gutter} = bantuan: {help}");
        }
    }
    out
}

pub fn render_short(diagnostic: &Diagnostic, source: &SourceFile) -> String {
    let (line, col) = source.line_col(diagnostic.span.start as usize);
    format!(
        "{}:{line}:{col}: {}[{}]: {}",
        source.name,
        diagnostic.severity.label(),
        diagnostic.code,
        diagnostic.message
    )
}

pub fn render_json(diagnostic: &Diagnostic, source: &SourceFile) -> serde_json::Value {
    let (line, col) = source.line_col(diagnostic.span.start as usize);
    let (end_line, end_col) = source.line_col(diagnostic.span.end as usize);
    serde_json::json!({
        "kode": diagnostic.code,
        "tingkat": diagnostic.severity.label(),
        "pesan": diagnostic.message,
        "berkas": source.name,
        "lokasi": {
            "baris": line,
            "kolom": col,
            "baris_akhir": end_line,
            "kolom_akhir": end_col,
        },
        "span": {
            "mulai": diagnostic.span.start,
            "akhir": diagnostic.span.end,
        },
        "label": diagnostic.labels.iter().map(|label| serde_json::json!({
            "pesan": label.message,
            "mulai": label.span.start,
            "akhir": label.span.end,
        })).collect::<Vec<_>>(),
        "catatan": diagnostic.notes,
        "bantuan": diagnostic.help,
    })
}

pub fn render_all(
    diagnostics: &[Diagnostic],
    source: &SourceFile,
    color: bool,
) -> String {
    let mut out = String::new();
    let mut sorted: Vec<&Diagnostic> = diagnostics.iter().collect();
    sorted.sort_by_key(|d| (d.span.start, d.span.end, d.code.clone()));
    for diagnostic in &sorted {
        out.push_str(&render_human(diagnostic, source, color));
        out.push('\n');
    }
    let errors = diagnostics.iter().filter(|d| d.is_error()).count();
    let warnings = diagnostics
        .iter()
        .filter(|d| d.severity == Severity::Warning)
        .count();
    if errors > 0 || warnings > 0 {
        let _ = writeln!(
            out,
            "{errors} error, {warnings} peringatan pada {}",
            source.name
        );
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn line_col_unicode() {
        let source = SourceFile::new("uji.aks", "nama = \"Ani\"\ncetak(nama)\n");
        assert_eq!(source.line_col(0), (1, 1));
        assert_eq!(source.line_col(7), (1, 8));
        assert_eq!(source.line_col(12), (2, 1));
    }

    #[test]
    fn normalisasi_baris() {
        let source = SourceFile::new("uji.aks", "a\r\nb\rc\n");
        assert_eq!(source.text, "a\nb\nc\n");
        assert_eq!(source.line_count(), 4);
    }

    #[test]
    fn render_menyebut_lokasi() {
        let source = SourceFile::new("uji.aks", "jika x = 1:\n    lewati\n");
        let diagnostic = Diagnostic::error("E-SINTAKS", "ekspresi diharapkan", Span::new(7, 8))
            .with_help("gunakan `==` untuk perbandingan");
        let rendered = render_human(&diagnostic, &source, false);
        assert!(rendered.contains("uji.aks:1:8"));
        assert!(rendered.contains("E-SINTAKS"));
        assert!(rendered.contains("gunakan `==`"));
    }
}
