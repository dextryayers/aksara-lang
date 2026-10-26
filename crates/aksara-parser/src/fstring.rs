use aksara_ast::{Expr, ExprKind, FStringPart, Quote, Span, StringLit};
use aksara_lexer::{Lexer, Token, TokenKind};

use crate::parser::Parser;

impl Parser {
    /// Memecah literal f-string mentah menjadi bagian teks dan ekspresi.
    pub(crate) fn parse_fstring_parts(
        &mut self,
        lit: &StringLit,
        span: Span,
    ) -> Vec<FStringPart> {
        let quote_len = match lit.quote {
            Quote::Single | Quote::Double => 1,
            Quote::TripleSingle | Quote::TripleDouble => 3,
        };
        let content_start = span
            .start
            .saturating_add(lit.prefix.len() as u32)
            .saturating_add(quote_len);
        let raw_mode = lit.prefix.contains('r');
        let mut parts = Vec::new();
        self.parse_fstring_content(&lit.raw, content_start, raw_mode, &mut parts, 0);
        parts
    }

    fn parse_fstring_content(
        &mut self,
        raw: &str,
        abs_start: u32,
        raw_mode: bool,
        parts: &mut Vec<FStringPart>,
        depth: usize,
    ) {
        if depth > 16 {
            self.error_help(
                "E-FSTRING",
                "f-string bersarang terlalu dalam",
                Span::new(abs_start, abs_start.saturating_add(raw.len() as u32)),
                "pecah format menjadi beberapa variabel antara",
            );
            return;
        }

        let mut text = String::new();
        let mut index = 0usize;
        while index < raw.len() {
            let ch = raw[index..].chars().next().expect("indeks berada di char boundary");
            match ch {
                '{' => {
                    if raw[index..].starts_with("{{") {
                        text.push('{');
                        index += 2;
                        continue;
                    }
                    flush_fstring_text(parts, &mut text, raw_mode);
                    let field_start = index + 1;
                    let Some(field_end) = find_field_end(raw, field_start) else {
                        let span = Span::new(
                            abs_start.saturating_add(index as u32),
                            abs_start.saturating_add(raw.len() as u32),
                        );
                        self.error_help(
                            "E-FSTRING",
                            "kurung kurawal pembuka tidak memiliki penutup",
                            span,
                            "tambahkan `}` atau gunakan `{{` untuk kurung literal",
                        );
                        text.push_str(&raw[index..]);
                        break;
                    };
                    self.parse_fstring_field(
                        raw,
                        abs_start,
                        field_start,
                        field_end,
                        parts,
                        depth + 1,
                    );
                    index = field_end + 1;
                }
                '}' => {
                    if raw[index..].starts_with("}}") {
                        text.push('}');
                        index += 2;
                        continue;
                    }
                    let span = Span::new(
                        abs_start.saturating_add(index as u32),
                        abs_start.saturating_add(index as u32).saturating_add(1),
                    );
                    self.error_help(
                        "E-FSTRING",
                        "kurung kurawal penutup tidak memiliki pembuka",
                        span,
                        "gunakan `}}` untuk kurung literal",
                    );
                    index += 1;
                }
                '\\' if !raw_mode => {
                    text.push(ch);
                    index += ch.len_utf8();
                    if index < raw.len() {
                        let next = raw[index..]
                            .chars()
                            .next()
                            .expect("indeks berada di char boundary");
                        text.push(next);
                        index += next.len_utf8();
                    }
                }
                _ => {
                    text.push(ch);
                    index += ch.len_utf8();
                }
            }
        }
        flush_fstring_text(parts, &mut text, raw_mode);
    }

    fn parse_fstring_field(
        &mut self,
        raw: &str,
        abs_start: u32,
        field_start: usize,
        field_end: usize,
        parts: &mut Vec<FStringPart>,
        depth: usize,
    ) {
        let field = &raw[field_start..field_end];
        let split = split_fstring_field(field);
        let expression = &field[split.expr_start..split.expr_end];
        let trimmed = expression.trim();
        let leading = expression.len() - expression.trim_start().len();
        let expression_abs = abs_start
            .saturating_add(field_start as u32)
            .saturating_add(leading as u32);

        let value = if trimmed.is_empty() {
            let span = Span::new(
                abs_start.saturating_add(field_start as u32),
                abs_start.saturating_add(field_end as u32),
            );
            self.error_help(
                "E-FSTRING",
                "ekspresi f-string tidak boleh kosong",
                span,
                "tulis ekspresi di antara `{` dan `}`",
            );
            self.make_expr(span, ExprKind::None)
        } else {
            self.parse_embedded_expression(trimmed, expression_abs)
        };

        let format_spec = split.format_start.map(|start| {
            let spec = &field[start..];
            let mut nested = Vec::new();
            self.parse_fstring_content(
                spec,
                abs_start
                    .saturating_add(field_start as u32)
                    .saturating_add(start as u32),
                true,
                &mut nested,
                depth,
            );
            nested
        });

        parts.push(FStringPart::Expr {
            value: Box::new(value),
            conversion: split.conversion,
            format_spec,
        });
    }

    fn parse_embedded_expression(&mut self, source: &str, abs_start: u32) -> Expr {
        let lexed = Lexer::with_offset(source, abs_start).tokenize();
        let tokens: Vec<Token> = lexed
            .tokens
            .into_iter()
            .filter(|token| {
                !matches!(
                    token.kind,
                    TokenKind::Newline | TokenKind::Indent | TokenKind::Dedent
                )
            })
            .collect();

        let mut parser = Parser::new(tokens, Vec::new());
        parser.next_id = self.next_id;
        let expr = parser.parse_expression();
        if !parser.at_eof() {
            let span = parser.current().span;
            parser.error_help(
                "E-FSTRING",
                "ekspresi f-string memiliki token yang tidak terpakai",
                span,
                "periksa operator atau tanda kurung di dalam ekspresi",
            );
        }
        self.next_id = parser.next_id;
        self.diagnostics.extend(lexed.diagnostics);
        self.diagnostics.extend(parser.diagnostics);
        expr
    }
}

fn flush_fstring_text(parts: &mut Vec<FStringPart>, text: &mut String, raw_mode: bool) {
    if text.is_empty() {
        return;
    }
    let cooked = if raw_mode {
        text.clone()
    } else {
        let lit = StringLit {
            prefix: String::new(),
            quote: Quote::Double,
            raw: text.clone(),
        };
        Parser::simple_string_value(&lit).unwrap_or_else(|| text.clone())
    };
    parts.push(FStringPart::Text(cooked));
    text.clear();
}

struct FieldSplit {
    expr_start: usize,
    expr_end: usize,
    conversion: Option<char>,
    format_start: Option<usize>,
}

fn split_fstring_field(field: &str) -> FieldSplit {
    let mut index = 0usize;
    let mut bracket_depth = 0usize;

    while index < field.len() {
        let ch = field[index..].chars().next().expect("indeks valid");
        if ch == '"' || ch == '\'' {
            index = skip_string_literal(field, index);
            continue;
        }
        if ch == '\\' {
            index = skip_escape(field, index);
            continue;
        }
        match ch {
            '(' | '[' | '{' => {
                bracket_depth += 1;
                index += ch.len_utf8();
            }
            ')' | ']' | '}' => {
                bracket_depth = bracket_depth.saturating_sub(1);
                index += ch.len_utf8();
            }
            '!' if bracket_depth == 0 => {
                let mut look = index + 1;
                if look < field.len() {
                    let conversion = field[look..].chars().next().expect("indeks valid");
                    if matches!(conversion, 's' | 'r' | 'a') {
                        look += conversion.len_utf8();
                        if look >= field.len() || field[look..].starts_with(':') {
                            let format_start = if look < field.len() {
                                Some(look + 1)
                            } else {
                                None
                            };
                            return FieldSplit {
                                expr_start: 0,
                                expr_end: index,
                                conversion: Some(conversion),
                                format_start,
                            };
                        }
                    }
                }
                index += 1;
            }
            ':' if bracket_depth == 0 => {
                return FieldSplit {
                    expr_start: 0,
                    expr_end: index,
                    conversion: None,
                    format_start: Some(index + 1),
                };
            }
            _ => index += ch.len_utf8(),
        }
    }

    FieldSplit {
        expr_start: 0,
        expr_end: field.len(),
        conversion: None,
        format_start: None,
    }
}

fn find_field_end(raw: &str, start: usize) -> Option<usize> {
    let mut index = start;
    let mut depth = 0usize;
    while index < raw.len() {
        let ch = raw[index..].chars().next().expect("indeks valid");
        if ch == '"' || ch == '\'' {
            index = skip_string_literal(raw, index);
            continue;
        }
        if ch == '\\' {
            index = skip_escape(raw, index);
            continue;
        }
        match ch {
            '(' | '[' | '{' => {
                depth += 1;
                index += ch.len_utf8();
            }
            ')' | ']' => {
                depth = depth.saturating_sub(1);
                index += ch.len_utf8();
            }
            '}' => {
                if depth == 0 {
                    return Some(index);
                }
                depth -= 1;
                index += ch.len_utf8();
            }
            _ => index += ch.len_utf8(),
        }
    }
    None
}

fn skip_escape(text: &str, index: usize) -> usize {
    let mut next = index + 1;
    if next >= text.len() {
        return text.len();
    }
    let ch = text[next..].chars().next().expect("indeks valid");
    next += ch.len_utf8();
    next
}

fn skip_string_literal(text: &str, start: usize) -> usize {
    let quote = text[start..].chars().next().expect("indeks valid");
    let triple = text[start..].starts_with(&quote.to_string().repeat(3));
    let mut index = start + if triple { 3 } else { 1 };
    let closing = quote.to_string().repeat(if triple { 3 } else { 1 });
    while index < text.len() {
        let ch = text[index..].chars().next().expect("indeks valid");
        if ch == '\\' {
            index = skip_escape(text, index);
            continue;
        }
        if text[index..].starts_with(&closing) {
            return index + closing.len();
        }
        index += ch.len_utf8();
    }
    text.len()
}
