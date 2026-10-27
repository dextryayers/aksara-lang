use aksara_ast::{Module, NodeId, Span, Stmt};
use aksara_diagnostik::{normalize_source, Diagnostic};
use aksara_lexer::{Lexer, Token, TokenKind};

/// Hasil parsing satu berkas.
#[derive(Clone, Debug)]
pub struct ParseResult {
    pub module: Module,
    pub diagnostics: Vec<Diagnostic>,
}

/// Melakukan lexing dan parsing pada teks sumber.
pub fn parse(source: &str) -> ParseResult {
    // Normalisasi hanya perlu salinan bila sumber memuat BOM atau akhir baris
    // CRLF. Berkas biasa langsung dipakai agar tidak menyalin seluruh isi.
    let normalized;
    let source = if source.starts_with('\u{feff}') || source.contains('\r') {
        normalized = normalize_source(source);
        normalized.as_str()
    } else {
        source
    };
    let lexed = Lexer::new(source).tokenize();
    parse_tokens(lexed.tokens, lexed.diagnostics)
}

/// Melakukan parsing pada token yang sudah dihasilkan lexer.
///
/// Token harus memakai offset terhadap sumber yang sudah dinormalisasi. Bila
/// token tidak diakhiri `Eof`, parser menambahkannya sendiri.
pub fn parse_tokens(tokens: Vec<Token>, diagnostics: Vec<Diagnostic>) -> ParseResult {
    let mut parser = Parser::new(tokens, diagnostics);
    let module = parser.parse_module();
    ParseResult {
        module,
        diagnostics: parser.diagnostics,
    }
}

/// Batas kedalaman struktur bersarang. Batas ini menjaga parser tetap
/// berhenti dengan diagnostik, bukan menghabiskan tumpukan panggilan.
pub(crate) const MAX_DEPTH: u32 = 256;

pub struct Parser {
    pub(crate) tokens: Vec<Token>,
    pub(crate) index: usize,
    pub(crate) diagnostics: Vec<Diagnostic>,
    pub(crate) next_id: u32,
    pub(crate) depth: u32,
}

impl Parser {
    pub fn new(mut tokens: Vec<Token>, diagnostics: Vec<Diagnostic>) -> Self {
        if tokens.is_empty() {
            tokens.push(Token::new(TokenKind::Eof, Span::empty(0)));
        } else if !matches!(tokens.last().map(|token| &token.kind), Some(TokenKind::Eof)) {
            let end = tokens.last().map(|token| token.span.end).unwrap_or(0);
            tokens.push(Token::new(TokenKind::Eof, Span::empty(end)));
        }
        Self {
            tokens,
            index: 0,
            diagnostics,
            next_id: 0,
            depth: 0,
        }
    }

    /// Parsing satu modul sampai `Eof`.
    pub fn parse_module(&mut self) -> Module {
        let start = self.current().span;
        let mut body = Vec::new();

        while !self.at_eof() {
            if self.at(&TokenKind::Newline) || self.at(&TokenKind::Semi) {
                self.bump();
                continue;
            }
            if self.at(&TokenKind::Dedent) || self.at(&TokenKind::Indent) {
                let span = self.current().span;
                self.error_help(
                    "E-SINTAKS",
                    "indentasi tidak diharapkan pada posisi ini",
                    span,
                    "periksa jumlah spasi dan tanda titik dua sebelum blok",
                );
                self.bump();
                continue;
            }

            let before = self.index;
            let statements = self.parse_statement();
            body.extend(statements);
            if self.index == before {
                let span = self.current().span;
                self.error("E-SINTAKS", "pernyataan tidak dapat dikenali", span);
                self.synchronize_statement();
            }
        }

        let end = self.current().span;
        let id = self.node_id();
        Module {
            id,
            span: start.merge(end),
            body,
        }
    }

    /// Membaca blok pernyataan setelah `Indent`.
    pub(crate) fn parse_block(&mut self) -> Vec<Stmt> {
        let mut body = Vec::new();

        while !self.at_eof() && !self.at(&TokenKind::Dedent) {
            if self.at(&TokenKind::Newline) {
                self.bump();
                continue;
            }
            if self.at(&TokenKind::Indent) {
                let span = self.current().span;
                self.error_help(
                    "E-SINTAKS",
                    "indentasi ganda tidak diharapkan",
                    span,
                    "gunakan satu tingkat indentasi untuk setiap blok",
                );
                self.bump();
                continue;
            }

            let before = self.index;
            let statements = self.parse_statement();
            body.extend(statements);
            if self.index == before {
                let span = self.current().span;
                self.error("E-SINTAKS", "pernyataan tidak dapat dikenali", span);
                self.synchronize_statement();
            }
        }

        if self.at(&TokenKind::Dedent) {
            self.bump();
        } else {
            let span = self.current().span;
            self.error_help(
                "E-SINTAKS",
                "blok indentasi tidak ditutup",
                span,
                "samakan indentasi dengan awal blok",
            );
        }
        body
    }

    /// Membaca suite setelah titik dua.
    pub(crate) fn parse_suite(&mut self) -> Vec<Stmt> {
        let span = self.current().span;
        if !self.enter_depth(span) {
            self.synchronize_statement();
            return Vec::new();
        }
        let body = self.parse_suite_inner();
        self.exit_depth();
        body
    }

    fn parse_suite_inner(&mut self) -> Vec<Stmt> {
        if self.at(&TokenKind::Newline) {
            self.bump();
            if self.at(&TokenKind::Indent) {
                self.bump();
                return self.parse_block();
            }
            let span = self.current().span;
            self.error_help(
                "E-SINTAKS",
                "blok setelah titik dua harus diindentasi",
                span,
                "tambahkan 4 spasi pada baris berikutnya",
            );
            return Vec::new();
        }

        self.parse_inline_suite()
    }

    fn parse_inline_suite(&mut self) -> Vec<Stmt> {
        let mut body = Vec::new();
        let start = self.current().span;

        loop {
            if self.at(&TokenKind::Newline) || self.at(&TokenKind::Dedent) || self.at_eof() {
                break;
            }
            if self.at(&TokenKind::Semi) {
                self.bump();
                continue;
            }
            if self.is_compound_start() {
                let span = self.current().span;
                self.error_help(
                    "E-SINTAKS",
                    "pernyataan majemuk tidak boleh ditulis pada baris yang sama setelah titik dua",
                    span,
                    "pindahkan pernyataan majemuk ke baris berikutnya dengan indentasi",
                );
                self.synchronize_statement();
                break;
            }

            let before = self.index;
            match self.parse_simple_statement() {
                Some(stmt) => body.push(stmt),
                None => {
                    self.synchronize_statement();
                    break;
                }
            }
            if self.index == before {
                self.synchronize_statement();
                break;
            }

            if self.at(&TokenKind::Semi) {
                self.bump();
                continue;
            }
            break;
        }

        if self.at(&TokenKind::Newline) {
            self.bump();
        } else if !self.at(&TokenKind::Dedent) && !self.at_eof() {
            let span = self.current().span;
            self.error_help(
                "E-SINTAKS",
                "akhir baris diharapkan",
                span,
                "tambahkan baris baru atau titik koma",
            );
            self.synchronize_statement();
        }

        if body.is_empty() {
            self.error_help(
                "E-SINTAKS",
                "suite tidak boleh kosong",
                start.merge(self.current().span),
                "tambahkan minimal satu pernyataan setelah titik dua",
            );
        }
        body
    }

    fn is_compound_start(&self) -> bool {
        matches!(
            self.current().kind,
            TokenKind::Jika
                | TokenKind::Selama
                | TokenKind::Untuk
                | TokenKind::Fungsi
                | TokenKind::Kelas
                | TokenKind::Coba
                | TokenKind::Dengan
                | TokenKind::Cocokkan
                | TokenKind::Asinkron
        )
    }

    /// Melakukan sinkronisasi setelah kesalahan pada satu pernyataan.
    pub(crate) fn synchronize_statement(&mut self) {
        while !self.at_eof() && !self.at(&TokenKind::Dedent) {
            if self.at(&TokenKind::Newline) {
                self.bump();
                return;
            }
            if self.at(&TokenKind::Semi) {
                self.bump();
                return;
            }
            self.bump();
        }
    }

    pub(crate) fn current(&self) -> &Token {
        self.tokens
            .get(self.index)
            .unwrap_or_else(|| self.tokens.last().expect("parser selalu memiliki Eof"))
    }

    pub(crate) fn nth(&self, offset: usize) -> &Token {
        self.tokens
            .get(self.index + offset)
            .unwrap_or_else(|| self.tokens.last().expect("parser selalu memiliki Eof"))
    }

    pub(crate) fn previous_span(&self) -> Span {
        if self.index == 0 {
            self.current().span
        } else {
            self.tokens[self.index - 1].span
        }
    }

    pub(crate) fn at(&self, kind: &TokenKind) -> bool {
        &self.current().kind == kind
    }

    pub(crate) fn nth_is(&self, offset: usize, kind: &TokenKind) -> bool {
        &self.nth(offset).kind == kind
    }

    pub(crate) fn at_eof(&self) -> bool {
        matches!(self.current().kind, TokenKind::Eof)
    }

    pub(crate) fn at_name(&self) -> bool {
        matches!(self.current().kind, TokenKind::Name(_))
    }

    pub(crate) fn at_name_text(&self, text: &str) -> bool {
        matches!(&self.current().kind, TokenKind::Name(name) if name == text)
    }

    /// Memajukan satu token dan mengembalikan span-nya tanpa menyalin token.
    pub(crate) fn bump(&mut self) -> Span {
        let span = self.current().span;
        if !self.at_eof() {
            self.index += 1;
        }
        span
    }

    pub(crate) fn eat(&mut self, kind: &TokenKind) -> bool {
        if self.at(kind) {
            self.bump();
            true
        } else {
            false
        }
    }

    pub(crate) fn expect(&mut self, kind: &TokenKind, what: &str) -> bool {
        if self.eat(kind) {
            true
        } else {
            let span = self.current().span;
            self.error(
                "E-SINTAKS",
                format!("{} diharapkan, ditemukan {}", what, self.describe_current()),
                span,
            );
            false
        }
    }

    pub(crate) fn expect_name(&mut self, what: &str) -> Option<(String, Span)> {
        match &self.current().kind {
            TokenKind::Name(name) => {
                let name = name.clone();
                let span = self.current().span;
                self.bump();
                Some((name, span))
            }
            _ => {
                let span = self.current().span;
                self.error(
                    "E-SINTAKS",
                    format!(
                        "nama {} diharapkan, ditemukan {}",
                        what,
                        self.describe_current()
                    ),
                    span,
                );
                None
            }
        }
    }

    pub(crate) fn describe_current(&self) -> String {
        match &self.current().kind {
            TokenKind::Name(name) => format!("nama `{name}`"),
            TokenKind::Error(text) => format!("token tidak valid `{text}`"),
            TokenKind::Int(text) | TokenKind::Float(text) | TokenKind::Imaginary(text) => {
                format!("literal `{text}`")
            }
            other => format!("`{}`", other.name()),
        }
    }

    pub(crate) fn error(&mut self, code: &str, message: impl Into<String>, span: Span) {
        self.diagnostics
            .push(Diagnostic::error(code, message, span));
    }

    pub(crate) fn error_help(
        &mut self,
        code: &str,
        message: impl Into<String>,
        span: Span,
        help: impl Into<String>,
    ) {
        self.diagnostics
            .push(Diagnostic::error(code, message, span).with_help(help));
    }

    pub(crate) fn enter_depth(&mut self, span: Span) -> bool {
        if self.depth >= MAX_DEPTH {
            self.error_help(
                "E-SINTAKS",
                "struktur bersarang terlalu dalam",
                span,
                "kurangi kedalaman tanda kurung, pola, atau blok",
            );
            false
        } else {
            self.depth += 1;
            true
        }
    }

    pub(crate) fn exit_depth(&mut self) {
        self.depth = self.depth.saturating_sub(1);
    }

    pub(crate) fn node_id(&mut self) -> NodeId {
        let id = NodeId::new(self.next_id);
        self.next_id = self.next_id.saturating_add(1);
        id
    }

    pub(crate) fn make_expr(&mut self, span: Span, kind: aksara_ast::ExprKind) -> aksara_ast::Expr {
        let id = self.node_id();
        aksara_ast::Expr { id, span, kind }
    }

    pub(crate) fn make_stmt(&mut self, span: Span, kind: aksara_ast::StmtKind) -> Stmt {
        let id = self.node_id();
        Stmt { id, span, kind }
    }

    pub(crate) fn at_statement_end(&self) -> bool {
        self.at(&TokenKind::Newline)
            || self.at(&TokenKind::Semi)
            || self.at(&TokenKind::Dedent)
            || self.at_eof()
    }

    pub(crate) fn span_of_statements(&self, statements: &[Stmt], fallback: Span) -> Span {
        statements.last().map(|stmt| stmt.span).unwrap_or(fallback)
    }
}
