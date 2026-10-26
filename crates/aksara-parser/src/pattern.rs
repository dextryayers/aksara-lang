use aksara_ast::{ExprKind, Pattern, UnaryOp};
use aksara_lexer::TokenKind;

use crate::parser::Parser;

impl Parser {
    pub(crate) fn parse_pattern(&mut self) -> Pattern {
        let first = self.parse_closed_pattern();
        if !self.at(&TokenKind::Pipe) {
            return first;
        }
        let mut patterns = vec![first];
        while self.eat(&TokenKind::Pipe) {
            patterns.push(self.parse_closed_pattern());
        }
        Pattern::Or(patterns)
    }

    fn parse_closed_pattern(&mut self) -> Pattern {
        match self.current().kind.clone() {
            TokenKind::Int(value) => {
                let span = self.bump().span;
                Pattern::Literal(self.make_expr(span, ExprKind::Int(value)))
            }
            TokenKind::Float(value) => {
                let span = self.bump().span;
                Pattern::Literal(self.make_expr(span, ExprKind::Float(value)))
            }
            TokenKind::Imaginary(value) => {
                let span = self.bump().span;
                Pattern::Literal(self.make_expr(span, ExprKind::Imaginary(value)))
            }
            TokenKind::Str(_) => Pattern::Literal(self.parse_string_literal()),
            TokenKind::Benar => {
                let span = self.bump().span;
                Pattern::Literal(self.make_expr(span, ExprKind::Bool(true)))
            }
            TokenKind::Salah => {
                let span = self.bump().span;
                Pattern::Literal(self.make_expr(span, ExprKind::Bool(false)))
            }
            TokenKind::Kosong => {
                let span = self.bump().span;
                Pattern::Literal(self.make_expr(span, ExprKind::None))
            }
            TokenKind::Minus | TokenKind::Plus => {
                let token = self.bump();
                let start = token.span;
                let op = if matches!(token.kind, TokenKind::Minus) {
                    UnaryOp::Neg
                } else {
                    UnaryOp::Pos
                };
                let value = match self.current().kind.clone() {
                    TokenKind::Int(text) => {
                        let span = self.bump().span;
                        self.make_expr(span, ExprKind::Int(text))
                    }
                    TokenKind::Float(text) => {
                        let span = self.bump().span;
                        self.make_expr(span, ExprKind::Float(text))
                    }
                    TokenKind::Imaginary(text) => {
                        let span = self.bump().span;
                        self.make_expr(span, ExprKind::Imaginary(text))
                    }
                    _ => self.missing_expression("bilangan pada pola"),
                };
                let span = start.merge(value.span);
                let expr = self.make_expr(
                    span,
                    ExprKind::Unary {
                        op,
                        operand: Box::new(value),
                    },
                );
                Pattern::Literal(expr)
            }
            TokenKind::Name(name) => self.parse_name_pattern(name),
            TokenKind::LBracket => self.parse_sequence_pattern(&TokenKind::RBracket, false),
            TokenKind::LParen => self.parse_sequence_pattern(&TokenKind::RParen, true),
            TokenKind::LBrace => self.parse_mapping_pattern(),
            TokenKind::Star => {
                let span = self.bump().span;
                let name = if self.at_name() {
                    self.expect_name("nama pola bintang").map(|(name, _)| name)
                } else {
                    None
                };
                self.error_help(
                    "E-POLA",
                    "pola bintang hanya boleh berada di dalam pola urutan",
                    span,
                    "bungkus dengan `[...]` atau `(...)`",
                );
                Pattern::Star(name)
            }
            _ => {
                let span = self.current().span;
                self.error_help(
                    "E-POLA",
                    format!("pola diharapkan, ditemukan {}", self.describe_current()),
                    span,
                    "gunakan literal, nama, `_`, urutan, kamus, atau pola kelas",
                );
                Pattern::Wildcard
            }
        }
    }

    fn parse_name_pattern(&mut self, first: String) -> Pattern {
        let first_span = self.bump().span;
        if first == "_" && !self.at(&TokenKind::Dot) && !self.at(&TokenKind::LParen) {
            return Pattern::Wildcard;
        }

        let mut name = first;
        let mut expr = self.make_expr(first_span, ExprKind::Name(name.clone()));
        while self.at(&TokenKind::Dot) {
            self.bump();
            match self.expect_name("setelah titik pada pola") {
                Some((part, part_span)) => {
                    name.push('.');
                    name.push_str(&part);
                    let span = expr.span.merge(part_span);
                    expr = self.make_expr(
                        span,
                        ExprKind::Attribute {
                            value: Box::new(expr),
                            attr: part,
                            safe: false,
                        },
                    );
                }
                None => break,
            }
        }

        if self.at(&TokenKind::LParen) {
            self.parse_class_pattern(name)
        } else if name.contains('.') {
            Pattern::Literal(expr)
        } else {
            Pattern::Capture(name)
        }
    }

    fn parse_class_pattern(&mut self, name: String) -> Pattern {
        self.expect(&TokenKind::LParen, "`(` pada pola kelas");
        let mut positional = Vec::new();
        let mut keywords = Vec::new();

        if !self.at(&TokenKind::RParen) {
            loop {
                if self.at_name() && self.nth_is(1, &TokenKind::Eq) {
                    let (keyword, _) = self
                        .expect_name("nama atribut pola kelas")
                        .expect("nama sudah diperiksa");
                    self.bump();
                    let pattern = self.parse_pattern();
                    keywords.push((keyword, pattern));
                } else {
                    positional.push(self.parse_pattern());
                }

                if self.eat(&TokenKind::Comma) {
                    if self.at(&TokenKind::RParen) {
                        break;
                    }
                    continue;
                }
                break;
            }
        }
        self.expect(&TokenKind::RParen, "`)` penutup pola kelas");
        Pattern::Class {
            name,
            positional,
            keywords,
        }
    }

    fn parse_sequence_pattern(&mut self, terminator: &TokenKind, parenthesized: bool) -> Pattern {
        self.bump();
        if self.eat(terminator) {
            return Pattern::Sequence(Vec::new());
        }

        let first = self.parse_sequence_item();
        if parenthesized && !self.at(&TokenKind::Comma) {
            self.expect(terminator, "`)` penutup pola");
            return first;
        }

        let mut patterns = vec![first];
        while self.eat(&TokenKind::Comma) {
            if self.at(terminator) {
                break;
            }
            patterns.push(self.parse_sequence_item());
        }
        self.expect(terminator, "penutup pola urutan");
        Pattern::Sequence(patterns)
    }

    fn parse_sequence_item(&mut self) -> Pattern {
        if self.at(&TokenKind::Star) {
            self.bump();
            if self.at(&TokenKind::Comma)
                || self.at(&TokenKind::RBracket)
                || self.at(&TokenKind::RParen)
            {
                return Pattern::Star(None);
            }
            return match self.expect_name("nama setelah `*` pada pola") {
                Some((name, _)) => Pattern::Star(Some(name)),
                None => Pattern::Star(None),
            };
        }
        self.parse_pattern()
    }

    fn parse_mapping_pattern(&mut self) -> Pattern {
        self.bump();
        let mut items = Vec::new();
        let mut rest = None;

        if !self.at(&TokenKind::RBrace) {
            loop {
                if self.eat(&TokenKind::StarStar) {
                    if let Some((name, _)) = self.expect_name("nama setelah `**` pada pola") {
                        rest = Some(name);
                    }
                    if self.eat(&TokenKind::Comma) && !self.at(&TokenKind::RBrace) {
                        let span = self.current().span;
                        self.error_help(
                            "E-POLA",
                            "`**` harus menjadi bagian terakhir pola kamus",
                            span,
                            "pindahkan `**` ke akhir pola",
                        );
                    }
                    break;
                }

                let key = self.parse_expression();
                self.expect(&TokenKind::Colon, "`:` antara kunci dan pola");
                let value = self.parse_pattern();
                items.push((key, value));

                if self.eat(&TokenKind::Comma) {
                    if self.at(&TokenKind::RBrace) {
                        break;
                    }
                    continue;
                }
                break;
            }
        }

        self.expect(&TokenKind::RBrace, "`}` penutup pola kamus");
        Pattern::Mapping { items, rest }
    }
}
