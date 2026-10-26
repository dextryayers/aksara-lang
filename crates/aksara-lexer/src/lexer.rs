use aksara_ast::{Quote, Span, StringLit};
use aksara_diagnostik::Diagnostic;

use crate::token::{Token, TokenKind};

#[derive(Clone, Debug, Default)]
pub struct LexResult {
    pub tokens: Vec<Token>,
    pub diagnostics: Vec<Diagnostic>,
}

pub struct Lexer<'a> {
    src: &'a str,
    pos: usize,
    base: u32,
    tokens: Vec<Token>,
    diagnostics: Vec<Diagnostic>,
    indent_stack: Vec<usize>,
    at_line_start: bool,
    bracket_depth: usize,
}

impl<'a> Lexer<'a> {
    pub fn new(src: &'a str) -> Self {
        Self::with_offset(src, 0)
    }

    pub fn with_offset(src: &'a str, base: u32) -> Self {
        let mut pos = 0;
        if src.starts_with('\u{feff}') {
            pos = '\u{feff}'.len_utf8();
        }
        Self {
            src,
            pos,
            base,
            tokens: Vec::new(),
            diagnostics: Vec::new(),
            indent_stack: vec![0],
            at_line_start: true,
            bracket_depth: 0,
        }
    }

    pub fn tokenize(mut self) -> LexResult {
        while self.pos < self.src.len() {
            if self.at_line_start && self.bracket_depth == 0 {
                if self.handle_line_start() {
                    continue;
                }
            }

            let ch = match self.peek() {
                Some(ch) => ch,
                None => break,
            };

            if ch == '\n' {
                self.bump();
                if self.bracket_depth == 0 {
                    self.push_newline();
                    self.at_line_start = true;
                }
                continue;
            }

            if ch == ' ' || ch == '\r' {
                self.bump();
                continue;
            }

            if ch == '\t' {
                let start = self.pos;
                self.bump();
                self.diagnostics.push(
                    Diagnostic::error(
                        "E-INDENTASI",
                        "tab tidak diizinkan di dalam kode",
                        self.span(start),
                    )
                    .with_help("gunakan 4 spasi untuk setiap level indentasi"),
                );
                continue;
            }

            if ch == '#' {
                self.skip_comment();
                continue;
            }

            if ch == '\\' && self.peek_next() == Some('\n') {
                self.bump();
                self.bump();
                self.at_line_start = false;
                continue;
            }

            if ch.is_ascii_digit() || (ch == '.' && self.peek_next().is_some_and(|c| c.is_ascii_digit())) {
                self.lex_number();
                continue;
            }

            if is_ident_start(ch) {
                if let Some(prefix_len) = self.string_prefix_len() {
                    self.lex_string(prefix_len);
                } else {
                    self.lex_identifier();
                }
                continue;
            }

            if ch == '"' || ch == '\'' {
                self.lex_string(0);
                continue;
            }

            self.lex_operator(ch);
        }

        if !matches!(self.tokens.last().map(|t| &t.kind), Some(TokenKind::Newline)) {
            self.push_newline();
        }
        while self.indent_stack.len() > 1 {
            self.indent_stack.pop();
            self.emit(TokenKind::Dedent, self.pos);
        }
        self.emit(TokenKind::Eof, self.pos);
        LexResult {
            tokens: self.tokens,
            diagnostics: self.diagnostics,
        }
    }

    fn handle_line_start(&mut self) -> bool {
        let start = self.pos;
        let mut indent = 0usize;
        loop {
            match self.peek() {
                Some(' ') => {
                    self.bump();
                    indent += 1;
                }
                Some('\t') => {
                    let tab_start = self.pos;
                    self.bump();
                    self.diagnostics.push(
                        Diagnostic::error(
                            "E-INDENTASI",
                            "tab tidak diizinkan pada awal baris",
                            self.span(tab_start),
                        )
                        .with_help("gunakan 4 spasi untuk setiap level indentasi"),
                    );
                    indent += 4;
                }
                _ => break,
            }
        }

        if matches!(self.peek(), Some('#') | Some('\n') | None) {
            if self.peek() == Some('#') {
                self.skip_comment();
            }
            if self.peek() == Some('\n') {
                self.bump();
            }
            return true;
        }

        let current = *self.indent_stack.last().unwrap_or(&0);
        if indent > current {
            if (indent - current) % 4 != 0 {
                self.diagnostics.push(
                    Diagnostic::error(
                        "E-INDENTASI",
                        "indentasi harus kelipatan 4 spasi",
                        self.span(start),
                    )
                    .with_help("tambahkan atau kurangi spasi agar menjadi kelipatan 4"),
                );
            }
            self.indent_stack.push(indent);
            self.emit(TokenKind::Indent, self.pos);
        } else if indent < current {
            while self.indent_stack.len() > 1
                && *self.indent_stack.last().unwrap_or(&0) > indent
            {
                self.indent_stack.pop();
                self.emit(TokenKind::Dedent, self.pos);
            }
            if *self.indent_stack.last().unwrap_or(&0) != indent {
                self.diagnostics.push(
                    Diagnostic::error(
                        "E-INDENTASI",
                        "dedentasi tidak cocok dengan level indentasi mana pun",
                        self.span(start),
                    )
                    .with_help("samakan jumlah spasi dengan blok sebelumnya"),
                );
                self.indent_stack.push(indent);
            }
        }

        self.at_line_start = false;
        false
    }

    fn push_newline(&mut self) {
        if matches!(self.tokens.last().map(|t| &t.kind), Some(TokenKind::Newline)) {
            return;
        }
        self.emit(TokenKind::Newline, self.pos);
    }

    fn emit(&mut self, kind: TokenKind, start: usize) {
        let span = Span::new(self.base + start as u32, self.base + self.pos as u32);
        self.tokens.push(Token::new(kind, span));
    }

    fn span(&self, start: usize) -> Span {
        Span::new(self.base + start as u32, self.base + self.pos as u32)
    }

    fn peek(&self) -> Option<char> {
        self.src[self.pos..].chars().next()
    }

    fn peek_next(&self) -> Option<char> {
        let mut chars = self.src[self.pos..].chars();
        chars.next()?;
        chars.next()
    }

    fn bump(&mut self) -> Option<char> {
        let ch = self.peek()?;
        self.pos += ch.len_utf8();
        Some(ch)
    }

    fn eat(&mut self, expected: char) -> bool {
        if self.peek() == Some(expected) {
            self.bump();
            true
        } else {
            false
        }
    }

    fn skip_comment(&mut self) {
        while let Some(ch) = self.peek() {
            if ch == '\n' {
                break;
            }
            self.bump();
        }
    }

    fn lex_identifier(&mut self) {
        let start = self.pos;
        self.bump();
        while let Some(ch) = self.peek() {
            if is_ident_continue(ch) {
                self.bump();
            } else {
                break;
            }
        }
        let text = &self.src[start..self.pos];
        let kind = TokenKind::keyword(text).unwrap_or_else(|| TokenKind::Name(text.to_string()));
        self.emit(kind, start);
    }

    fn string_prefix_len(&self) -> Option<usize> {
        let bytes = self.src[self.pos..].as_bytes();
        let mut len = 0;
        while len < 2 && len < bytes.len() && matches!(bytes[len], b'r' | b'R' | b'f' | b'F' | b'b' | b'B') {
            len += 1;
        }
        if len == 0 {
            return None;
        }
        match bytes.get(len) {
            Some(b'"') | Some(b'\'') => Some(len),
            _ => None,
        }
    }

    fn lex_string(&mut self, prefix_len: usize) {
        let start = self.pos;
        let prefix = self.src[self.pos..self.pos + prefix_len].to_ascii_lowercase();
        self.pos += prefix_len;

        let quote_char = match self.peek() {
            Some(ch @ ('"' | '\'')) => ch,
            _ => {
                self.diagnostics.push(Diagnostic::error(
                    "E-STRING",
                    "awalan teks harus diikuti tanda kutip",
                    self.span(start),
                ));
                return;
            }
        };
        let triple = self.src[self.pos..].starts_with(&quote_char.to_string().repeat(3));
        if triple {
            self.pos += 3;
        } else {
            self.pos += 1;
        }

        let raw = prefix.contains('r');
        let is_bytes = prefix.contains('b');
        let is_fstring = prefix.contains('f');
        if is_bytes && is_fstring {
            self.diagnostics.push(
                Diagnostic::error(
                    "E-STRING",
                    "prefix `f` dan `b` tidak boleh digabung",
                    self.span(start),
                )
                .with_help("gunakan `f\"...\"` atau `b\"...\"`, bukan keduanya"),
            );
        }

        let content_start = self.pos;
        let mut closed = false;
        while let Some(ch) = self.peek() {
            if ch == '\\' {
                self.bump();
                if self.peek().is_some() {
                    self.bump();
                }
                continue;
            }

            if ch == '\n' && !triple {
                self.diagnostics.push(
                    Diagnostic::error(
                        "E-STRING",
                        "teks tidak ditutup sebelum akhir baris",
                        self.span(start),
                    )
                    .with_help("tambahkan tanda kutip penutup atau gunakan teks triple-quoted"),
                );
                break;
            }

            if ch == quote_char {
                if triple {
                    let closing = quote_char.to_string().repeat(3);
                    if self.src[self.pos..].starts_with(&closing) {
                        self.pos += 3;
                        closed = true;
                        break;
                    }
                    self.bump();
                } else {
                    self.bump();
                    closed = true;
                    break;
                }
                continue;
            }

            self.bump();
        }

        if !closed && self.pos >= self.src.len() {
            self.diagnostics.push(
                Diagnostic::error(
                    "E-STRING",
                    "teks tidak ditutup sampai akhir berkas",
                    self.span(start),
                )
                .with_help("tambahkan tanda kutip penutup"),
            );
        }

        let content_end = if closed {
            self.pos - if triple { 3 } else { 1 }
        } else {
            self.pos
        };
        let raw_text = self.src[content_start..content_end].to_string();
        let quote = match (quote_char, triple) {
            ('"', false) => Quote::Double,
            ('"', true) => Quote::TripleDouble,
            ('\'', false) => Quote::Single,
            ('\'', true) => Quote::TripleSingle,
            _ => Quote::Double,
        };
        self.emit(
            TokenKind::Str(StringLit {
                prefix,
                quote,
                raw: raw_text,
            }),
            start,
        );
    }

    fn lex_number(&mut self) {
        let start = self.pos;
        let mut is_float = false;
        let mut is_imaginary = false;

        if self.peek() == Some('.') {
            is_float = true;
            self.bump();
            self.consume_digits();
        } else if self.peek() == Some('0') {
            self.bump();
            match self.peek() {
                Some('x') | Some('X') => {
                    self.bump();
                    self.consume_radix_digits(16);
                    self.emit_number(start, false, false);
                    return;
                }
                Some('o') | Some('O') => {
                    self.bump();
                    self.consume_radix_digits(8);
                    self.emit_number(start, false, false);
                    return;
                }
                Some('b') | Some('B') => {
                    self.bump();
                    self.consume_radix_digits(2);
                    self.emit_number(start, false, false);
                    return;
                }
                _ => {
                    self.consume_digits();
                }
            }
        } else {
            self.consume_digits();
        }

        if self.peek() == Some('.') && self.peek_next() != Some('.') {
            is_float = true;
            self.bump();
            self.consume_digits();
        }

        if matches!(self.peek(), Some('e') | Some('E')) {
            is_float = true;
            self.bump();
            if matches!(self.peek(), Some('+') | Some('-')) {
                self.bump();
            }
            let before = self.pos;
            self.consume_digits();
            if self.pos == before {
                self.diagnostics.push(
                    Diagnostic::error(
                        "E-ANGKA",
                        "eksponen bilangan tidak memiliki digit",
                        self.span(start),
                    )
                    .with_help("tulis eksponen seperti `1e-9`"),
                );
            }
        }

        if matches!(self.peek(), Some('j') | Some('J')) {
            is_imaginary = true;
            self.bump();
        }

        self.emit_number(start, is_float, is_imaginary);
    }

    fn consume_digits(&mut self) {
        while let Some(ch) = self.peek() {
            if ch.is_ascii_digit() || ch == '_' {
                self.bump();
            } else {
                break;
            }
        }
    }

    fn consume_radix_digits(&mut self, radix: u32) {
        let before = self.pos;
        while let Some(ch) = self.peek() {
            if ch == '_' || ch.is_digit(radix) {
                self.bump();
            } else {
                break;
            }
        }
        if self.pos == before {
            self.diagnostics.push(
                Diagnostic::error(
                    "E-ANGKA",
                    "literal bilangan tidak memiliki digit setelah awalan",
                    self.span(self.pos),
                )
                .with_help("tulis digit setelah `0x`, `0o`, atau `0b`"),
            );
        }
    }

    fn emit_number(&mut self, start: usize, is_float: bool, is_imaginary: bool) {
        let text = self.src[start..self.pos].to_string();
        if !is_float && !is_imaginary && is_invalid_decimal_prefix(&text) {
            self.diagnostics.push(
                Diagnostic::error(
                    "E-ANGKA",
                    "bilangan desimal tidak boleh diawali nol",
                    self.span(start),
                )
                .with_help("hapus nol di depan atau gunakan awalan `0o` untuk oktal"),
            );
        }
        let kind = if is_imaginary {
            TokenKind::Imaginary(text)
        } else if is_float {
            TokenKind::Float(text)
        } else {
            TokenKind::Int(text)
        };
        self.emit(kind, start);
    }

    fn lex_operator(&mut self, ch: char) {
        let start = self.pos;
        self.bump();
        let kind = match ch {
            '+' => {
                if self.eat('=') {
                    TokenKind::PlusEq
                } else {
                    TokenKind::Plus
                }
            }
            '-' => {
                if self.eat('>') {
                    TokenKind::Arrow
                } else if self.eat('=') {
                    TokenKind::MinusEq
                } else {
                    TokenKind::Minus
                }
            }
            '*' => {
                if self.eat('*') {
                    if self.eat('=') {
                        TokenKind::StarStarEq
                    } else {
                        TokenKind::StarStar
                    }
                } else if self.eat('=') {
                    TokenKind::StarEq
                } else {
                    TokenKind::Star
                }
            }
            '/' => {
                if self.eat('/') {
                    if self.eat('=') {
                        TokenKind::SlashSlashEq
                    } else {
                        TokenKind::SlashSlash
                    }
                } else if self.eat('=') {
                    TokenKind::SlashEq
                } else {
                    TokenKind::Slash
                }
            }
            '%' => {
                if self.eat('=') {
                    TokenKind::PercentEq
                } else {
                    TokenKind::Percent
                }
            }
            '@' => {
                if self.eat('=') {
                    TokenKind::AtEq
                } else {
                    TokenKind::At
                }
            }
            '&' => {
                if self.eat('=') {
                    TokenKind::AmpEq
                } else {
                    TokenKind::Amp
                }
            }
            '|' => {
                if self.eat('=') {
                    TokenKind::PipeEq
                } else {
                    TokenKind::Pipe
                }
            }
            '^' => {
                if self.eat('=') {
                    TokenKind::CaretEq
                } else {
                    TokenKind::Caret
                }
            }
            '~' => TokenKind::Tilde,
            '<' => {
                if self.eat('<') {
                    if self.eat('=') {
                        TokenKind::LShiftEq
                    } else {
                        TokenKind::LShift
                    }
                } else if self.eat('=') {
                    TokenKind::Le
                } else {
                    TokenKind::Lt
                }
            }
            '>' => {
                if self.eat('>') {
                    if self.eat('=') {
                        TokenKind::RShiftEq
                    } else {
                        TokenKind::RShift
                    }
                } else if self.eat('=') {
                    TokenKind::Ge
                } else {
                    TokenKind::Gt
                }
            }
            '=' => {
                if self.eat('=') {
                    TokenKind::EqEq
                } else {
                    TokenKind::Eq
                }
            }
            '!' => {
                if self.eat('=') {
                    TokenKind::NotEq
                } else {
                    self.diagnostics.push(
                        Diagnostic::error(
                            "E-LEKSIKAL",
                            "operator `!` tidak dikenal",
                            self.span(start),
                        )
                        .with_help("gunakan `bukan` untuk negasi"),
                    );
                    TokenKind::Error("!".to_string())
                }
            }
            '.' => {
                if self.src[self.pos..].starts_with("..") {
                    self.pos += 2;
                    TokenKind::Ellipsis
                } else {
                    TokenKind::Dot
                }
            }
            ':' => {
                if self.eat('=') {
                    TokenKind::ColonEq
                } else {
                    TokenKind::Colon
                }
            }
            ';' => TokenKind::Semi,
            ',' => TokenKind::Comma,
            '(' => {
                self.bracket_depth += 1;
                TokenKind::LParen
            }
            ')' => {
                self.bracket_depth = self.bracket_depth.saturating_sub(1);
                TokenKind::RParen
            }
            '[' => {
                self.bracket_depth += 1;
                TokenKind::LBracket
            }
            ']' => {
                self.bracket_depth = self.bracket_depth.saturating_sub(1);
                TokenKind::RBracket
            }
            '{' => {
                self.bracket_depth += 1;
                TokenKind::LBrace
            }
            '}' => {
                self.bracket_depth = self.bracket_depth.saturating_sub(1);
                TokenKind::RBrace
            }
            '?' => {
                if self.eat('?') {
                    if self.eat('=') {
                        TokenKind::QuestionQuestionEq
                    } else {
                        TokenKind::QuestionQuestion
                    }
                } else if self.eat('.') {
                    TokenKind::QuestionDot
                } else if self.eat('[') {
                    self.bracket_depth += 1;
                    TokenKind::QuestionLBracket
                } else if self.eat('(') {
                    self.bracket_depth += 1;
                    TokenKind::QuestionLParen
                } else {
                    self.diagnostics.push(
                        Diagnostic::error(
                            "E-LEKSIKAL",
                            "operator `?` harus menjadi bagian dari `??`, `?.`, `?[`, atau `?(`",
                            self.span(start),
                        )
                        .with_help("gunakan `??` untuk nilai pengganti `kosong`"),
                    );
                    TokenKind::Error("?".to_string())
                }
            }
            _ => {
                self.diagnostics.push(
                    Diagnostic::error(
                        "E-LEKSIKAL",
                        format!("karakter `{ch}` tidak dikenal"),
                        self.span(start),
                    )
                    .with_help("hapus karakter tersebut atau periksa dokumentasi operator"),
                );
                TokenKind::Error(ch.to_string())
            }
        };
        self.emit(kind, start);
    }
}

fn is_ident_start(ch: char) -> bool {
    ch == '_' || ch.is_alphabetic()
}

fn is_ident_continue(ch: char) -> bool {
    ch == '_' || ch.is_alphanumeric()
}

fn is_invalid_decimal_prefix(text: &str) -> bool {
    let bytes = text.as_bytes();
    bytes.len() > 1
        && bytes[0] == b'0'
        && bytes[1].is_ascii_digit()
        && bytes.iter().all(|b| b.is_ascii_digit() || *b == b'_')
}
