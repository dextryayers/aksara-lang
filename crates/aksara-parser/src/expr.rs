use aksara_ast::{
    Arg, BinOp, BoolOp, CmpOp, CompClause, DictItem, Expr, ExprKind, StringLit, UnaryOp,
};
use aksara_lexer::TokenKind;

use crate::parser::Parser;

const BP_BITOR: u8 = 10;
const BP_BITXOR: u8 = 20;
const BP_BITAND: u8 = 30;
const BP_SHIFT: u8 = 40;
const BP_SUM: u8 = 50;
const BP_TERM: u8 = 60;

impl Parser {
    pub(crate) fn parse_expression(&mut self) -> Expr {
        if self.at(&TokenKind::Lambda) {
            return self.parse_lambda();
        }
        if self.at(&TokenKind::Hasilkan) {
            return self.parse_yield_expression();
        }

        let expr = self.parse_ternary();
        if self.eat(&TokenKind::ColonEq) {
            let value = self.parse_expression();
            if !matches!(expr.kind, ExprKind::Name(_)) {
                self.error_help(
                    "E-SINTAKS",
                    "target `:=` harus berupa nama",
                    expr.span,
                    "bungkus ekspresi dengan tanda kurung bila bukan nama",
                );
            }
            let span = expr.span.merge(value.span);
            return self.make_expr(
                span,
                ExprKind::NamedExpr {
                    target: Box::new(expr),
                    value: Box::new(value),
                },
            );
        }
        expr
    }

    fn parse_ternary(&mut self) -> Expr {
        let body = self.parse_null_coalesce();
        if self.eat(&TokenKind::Jika) {
            let condition = self.parse_null_coalesce();
            self.expect(&TokenKind::Lainnya, "`lainnya` pada ekspresi ternary");
            let orelse = self.parse_expression();
            let span = body.span.merge(orelse.span);
            return self.make_expr(
                span,
                ExprKind::IfExpr {
                    body: Box::new(body),
                    condition: Box::new(condition),
                    orelse: Box::new(orelse),
                },
            );
        }
        body
    }

    fn parse_null_coalesce(&mut self) -> Expr {
        let mut left = self.parse_or();
        while self.eat(&TokenKind::QuestionQuestion) {
            let right = self.parse_or();
            let span = left.span.merge(right.span);
            left = self.make_expr(
                span,
                ExprKind::Binary {
                    left: Box::new(left),
                    op: BinOp::NullCoalesce,
                    right: Box::new(right),
                },
            );
        }
        left
    }

    fn parse_or(&mut self) -> Expr {
        let first = self.parse_and();
        if !self.at(&TokenKind::Atau) {
            return first;
        }
        let start = first.span;
        let mut values = vec![first];
        while self.eat(&TokenKind::Atau) {
            values.push(self.parse_and());
        }
        let end = values.last().map(|expr| expr.span).unwrap_or(start);
        self.make_expr(
            start.merge(end),
            ExprKind::BoolOp {
                op: BoolOp::Or,
                values,
            },
        )
    }

    fn parse_and(&mut self) -> Expr {
        let first = self.parse_not();
        if !self.at(&TokenKind::Dan) {
            return first;
        }
        let start = first.span;
        let mut values = vec![first];
        while self.eat(&TokenKind::Dan) {
            values.push(self.parse_not());
        }
        let end = values.last().map(|expr| expr.span).unwrap_or(start);
        self.make_expr(
            start.merge(end),
            ExprKind::BoolOp {
                op: BoolOp::And,
                values,
            },
        )
    }

    fn parse_not(&mut self) -> Expr {
        if self.at(&TokenKind::Bukan) {
            let start = self.bump().span;
            let operand = self.parse_not();
            let span = start.merge(operand.span);
            return self.make_expr(
                span,
                ExprKind::Unary {
                    op: UnaryOp::Not,
                    operand: Box::new(operand),
                },
            );
        }
        self.parse_comparison()
    }

    fn parse_comparison(&mut self) -> Expr {
        let left = self.parse_bit_or();
        let mut ops = Vec::new();
        let mut comparators = Vec::new();
        let mut between: Option<Expr> = None;

        loop {
            let op = match self.current().kind {
                TokenKind::EqEq => Some(CmpOp::Eq),
                TokenKind::NotEq => Some(CmpOp::NotEq),
                TokenKind::Lt => Some(CmpOp::Lt),
                TokenKind::Le => Some(CmpOp::Le),
                TokenKind::Gt => Some(CmpOp::Gt),
                TokenKind::Ge => Some(CmpOp::Ge),
                TokenKind::Dalam => Some(CmpOp::In),
                TokenKind::Adalah => Some(CmpOp::Is),
                _ => None,
            };
            if let Some(op) = op {
                self.bump();
                ops.push(op);
                comparators.push(self.parse_bit_or());
                continue;
            }

            if self.at(&TokenKind::Bukan) {
                if self.nth_is(1, &TokenKind::Dalam) {
                    self.bump();
                    self.bump();
                    ops.push(CmpOp::NotIn);
                    comparators.push(self.parse_bit_or());
                    continue;
                }
                if self.nth_is(1, &TokenKind::Adalah) {
                    self.bump();
                    self.bump();
                    ops.push(CmpOp::IsNot);
                    comparators.push(self.parse_bit_or());
                    continue;
                }
                if self.nth_is(1, &TokenKind::Antara) {
                    self.bump();
                    self.bump();
                    between = Some(self.parse_between(left.clone(), true));
                    break;
                }
                break;
            }

            if self.at(&TokenKind::Antara) {
                self.bump();
                between = Some(self.parse_between(left.clone(), false));
                break;
            }
            break;
        }

        if let Some(expr) = between {
            if !ops.is_empty() {
                self.error_help(
                    "E-SINTAKS",
                    "`antara` tidak dapat dirangkai dengan perbandingan lain",
                    expr.span,
                    "gunakan tanda kurung untuk memisahkan perbandingan",
                );
            }
            if self.at_comparison_operator() {
                self.error_help(
                    "E-SINTAKS",
                    "`antara` tidak dapat dirangkai dengan perbandingan lain",
                    self.current().span,
                    "gunakan tanda kurung untuk memisahkan perbandingan",
                );
            }
            return expr;
        }

        if ops.is_empty() {
            left
        } else {
            let end = comparators
                .last()
                .map(|expr| expr.span)
                .unwrap_or(left.span);
            self.make_expr(
                left.span.merge(end),
                ExprKind::Compare {
                    left: Box::new(left),
                    ops,
                    comparators,
                },
            )
        }
    }

    fn parse_between(&mut self, value: Expr, negated: bool) -> Expr {
        let low = self.parse_bit_or();
        self.expect(&TokenKind::Dan, "`dan` setelah batas bawah `antara`");
        let high = self.parse_bit_or();
        let span = value.span.merge(high.span);
        self.make_expr(
            span,
            ExprKind::Between {
                value: Box::new(value),
                low: Box::new(low),
                high: Box::new(high),
                negated,
            },
        )
    }

    fn at_comparison_operator(&self) -> bool {
        matches!(
            self.current().kind,
            TokenKind::EqEq
                | TokenKind::NotEq
                | TokenKind::Lt
                | TokenKind::Le
                | TokenKind::Gt
                | TokenKind::Ge
                | TokenKind::Dalam
                | TokenKind::Adalah
                | TokenKind::Antara
        ) || (self.at(&TokenKind::Bukan)
            && (self.nth_is(1, &TokenKind::Dalam)
                || self.nth_is(1, &TokenKind::Adalah)
                || self.nth_is(1, &TokenKind::Antara)))
    }

    fn parse_bit_or(&mut self) -> Expr {
        self.parse_binary(BP_BITOR)
    }

    fn parse_binary(&mut self, min_bp: u8) -> Expr {
        let mut left = self.parse_factor();
        loop {
            let (op, bp) = match self.current().kind {
                TokenKind::Pipe => (BinOp::BitOr, BP_BITOR),
                TokenKind::Caret => (BinOp::BitXor, BP_BITXOR),
                TokenKind::Amp => (BinOp::BitAnd, BP_BITAND),
                TokenKind::LShift => (BinOp::LShift, BP_SHIFT),
                TokenKind::RShift => (BinOp::RShift, BP_SHIFT),
                TokenKind::Plus => (BinOp::Add, BP_SUM),
                TokenKind::Minus => (BinOp::Sub, BP_SUM),
                TokenKind::Star => (BinOp::Mul, BP_TERM),
                TokenKind::Slash => (BinOp::Div, BP_TERM),
                TokenKind::SlashSlash => (BinOp::FloorDiv, BP_TERM),
                TokenKind::Percent => (BinOp::Mod, BP_TERM),
                TokenKind::At => (BinOp::MatMul, BP_TERM),
                _ => break,
            };
            if bp < min_bp {
                break;
            }
            self.bump();
            let right = self.parse_binary(bp + 1);
            let span = left.span.merge(right.span);
            left = self.make_expr(
                span,
                ExprKind::Binary {
                    left: Box::new(left),
                    op,
                    right: Box::new(right),
                },
            );
        }
        left
    }

    fn parse_factor(&mut self) -> Expr {
        let op = match self.current().kind {
            TokenKind::Plus => Some(UnaryOp::Pos),
            TokenKind::Minus => Some(UnaryOp::Neg),
            TokenKind::Tilde => Some(UnaryOp::Invert),
            _ => None,
        };
        if let Some(op) = op {
            let start = self.bump().span;
            let operand = self.parse_factor();
            let span = start.merge(operand.span);
            return self.make_expr(
                span,
                ExprKind::Unary {
                    op,
                    operand: Box::new(operand),
                },
            );
        }
        self.parse_power()
    }

    fn parse_power(&mut self) -> Expr {
        let left = self.parse_await();
        if self.eat(&TokenKind::StarStar) {
            let right = self.parse_factor();
            let span = left.span.merge(right.span);
            return self.make_expr(
                span,
                ExprKind::Binary {
                    left: Box::new(left),
                    op: BinOp::Pow,
                    right: Box::new(right),
                },
            );
        }
        left
    }

    fn parse_await(&mut self) -> Expr {
        if self.at(&TokenKind::Tunggu) {
            let start = self.bump().span;
            let operand = self.parse_await();
            let span = start.merge(operand.span);
            return self.make_expr(span, ExprKind::Await(Box::new(operand)));
        }
        self.parse_postfix()
    }

    fn parse_postfix(&mut self) -> Expr {
        let mut expr = self.parse_primary();
        loop {
            if self.at(&TokenKind::Dot) || self.at(&TokenKind::QuestionDot) {
                let safe = self.at(&TokenKind::QuestionDot);
                self.bump();
                let (attr, attr_span) = match self.expect_name("setelah titik") {
                    Some(value) => value,
                    None => break,
                };
                let span = expr.span.merge(attr_span);
                expr = self.make_expr(
                    span,
                    ExprKind::Attribute {
                        value: Box::new(expr),
                        attr,
                        safe,
                    },
                );
                continue;
            }

            if self.at(&TokenKind::LParen) || self.at(&TokenKind::QuestionLParen) {
                let safe = self.at(&TokenKind::QuestionLParen);
                self.bump();
                expr = self.parse_call(expr, safe);
                continue;
            }

            if self.at(&TokenKind::LBracket) || self.at(&TokenKind::QuestionLBracket) {
                let safe = self.at(&TokenKind::QuestionLBracket);
                self.bump();
                expr = self.parse_subscript(expr, safe);
                continue;
            }

            break;
        }
        expr
    }

    fn parse_primary(&mut self) -> Expr {
        let kind = self.current().kind.clone();
        match kind {
            TokenKind::Name(name) => {
                let span = self.bump().span;
                self.make_expr(span, ExprKind::Name(name))
            }
            TokenKind::Int(value) => {
                let span = self.bump().span;
                self.make_expr(span, ExprKind::Int(value))
            }
            TokenKind::Float(value) => {
                let span = self.bump().span;
                self.make_expr(span, ExprKind::Float(value))
            }
            TokenKind::Imaginary(value) => {
                let span = self.bump().span;
                self.make_expr(span, ExprKind::Imaginary(value))
            }
            TokenKind::Str(_) => self.parse_string_literal(),
            TokenKind::Benar => {
                let span = self.bump().span;
                self.make_expr(span, ExprKind::Bool(true))
            }
            TokenKind::Salah => {
                let span = self.bump().span;
                self.make_expr(span, ExprKind::Bool(false))
            }
            TokenKind::Kosong => {
                let span = self.bump().span;
                self.make_expr(span, ExprKind::None)
            }
            TokenKind::Ellipsis => {
                let span = self.bump().span;
                self.make_expr(span, ExprKind::Ellipsis)
            }
            TokenKind::LParen => self.parse_parenthesized(),
            TokenKind::LBracket => self.parse_list_literal(),
            TokenKind::LBrace => self.parse_set_or_dict(),
            _ => self.missing_expression("ekspresi"),
        }
    }

    pub(crate) fn missing_expression(&mut self, what: &str) -> Expr {
        let span = self.current().span;
        let found = self.describe_current();
        self.error_help(
            "E-SINTAKS",
            format!("{what} diharapkan, ditemukan {found}"),
            span,
            "periksa apakah ada tanda kurung, koma, atau operator yang belum lengkap",
        );
        self.make_expr(span, ExprKind::None)
    }

    pub(crate) fn parse_string_literal(&mut self) -> Expr {
        let mut parts = Vec::new();
        loop {
            let (lit, span) = match &self.current().kind {
                TokenKind::Str(lit) => (lit.clone(), self.current().span),
                _ => break,
            };
            self.bump();
            parts.push(self.string_expr(lit, span));
        }

        if parts.is_empty() {
            return self.missing_expression("literal teks");
        }
        if parts.len() == 1 {
            return parts.pop().expect("satu bagian teks");
        }

        let has_bytes = parts
            .iter()
            .any(|part| matches!(part.kind, ExprKind::Bytes(_)));
        let has_text = parts
            .iter()
            .any(|part| !matches!(part.kind, ExprKind::Bytes(_)));
        if has_bytes && has_text {
            let span = parts
                .first()
                .map(|part| part.span)
                .unwrap_or_else(|| self.current().span);
            self.error_help(
                "E-STRING",
                "literal bytes tidak dapat digabung dengan literal teks",
                span,
                "gunakan prefix `b` pada semua literal atau hapus prefix `b`",
            );
        }
        let span = parts
            .first()
            .map(|part| part.span)
            .unwrap_or_else(|| self.current().span)
            .merge(
                parts
                    .last()
                    .map(|part| part.span)
                    .unwrap_or_else(|| self.current().span),
            );
        self.make_expr(span, ExprKind::StringConcat(parts))
    }

    fn string_expr(&mut self, lit: StringLit, span: aksara_ast::Span) -> Expr {
        if lit.prefix.contains('b') {
            self.make_expr(span, ExprKind::Bytes(lit))
        } else if lit.prefix.contains('f') {
            let parts = self.parse_fstring_parts(&lit, span);
            self.make_expr(span, ExprKind::FString(parts))
        } else {
            self.make_expr(span, ExprKind::Str(lit))
        }
    }

    fn parse_parenthesized(&mut self) -> Expr {
        let open = self.bump().span;
        if self.eat(&TokenKind::RParen) {
            let span = open.merge(self.previous_span());
            return self.make_expr(span, ExprKind::Tuple(Vec::new()));
        }

        let first = self.parse_maybe_starred();
        if self.at(&TokenKind::Untuk)
            || (self.at(&TokenKind::Asinkron) && self.nth_is(1, &TokenKind::Untuk))
        {
            if matches!(first.kind, ExprKind::Starred(_)) {
                self.error_help(
                    "E-SINTAKS",
                    "pola bintang tidak dapat menjadi elemen generator",
                    first.span,
                    "hapus tanda bintang",
                );
            }
            let clauses = self.parse_comp_clauses();
            self.expect(&TokenKind::RParen, "`)` penutup generator");
            let span = open.merge(self.previous_span());
            return self.make_expr(
                span,
                ExprKind::GeneratorExp {
                    elt: Box::new(first),
                    clauses,
                },
            );
        }

        let mut items = vec![first];
        let mut saw_comma = false;
        while self.eat(&TokenKind::Comma) {
            saw_comma = true;
            if self.at(&TokenKind::RParen) {
                break;
            }
            items.push(self.parse_maybe_starred());
        }
        self.expect(&TokenKind::RParen, "`)` penutup ekspresi");
        let close = self.previous_span();

        if !saw_comma && items.len() == 1 {
            let mut expr = items.pop().expect("satu ekspresi");
            if matches!(expr.kind, ExprKind::Starred(_)) {
                self.error_help(
                    "E-SINTAKS",
                    "ekspresi bintang harus berada di dalam daftar atau tupel",
                    expr.span,
                    "tambahkan koma setelah ekspresi bintang",
                );
            }
            expr.span = open.merge(close);
            expr
        } else {
            let span = open.merge(close);
            self.make_expr(span, ExprKind::Tuple(items))
        }
    }

    fn parse_list_literal(&mut self) -> Expr {
        let open = self.bump().span;
        if self.eat(&TokenKind::RBracket) {
            let span = open.merge(self.previous_span());
            return self.make_expr(span, ExprKind::List(Vec::new()));
        }

        let first = self.parse_maybe_starred();
        if self.at(&TokenKind::Untuk)
            || (self.at(&TokenKind::Asinkron) && self.nth_is(1, &TokenKind::Untuk))
        {
            if matches!(first.kind, ExprKind::Starred(_)) {
                self.error_help(
                    "E-SINTAKS",
                    "pola bintang tidak dapat menjadi elemen comprehension",
                    first.span,
                    "hapus tanda bintang",
                );
            }
            let clauses = self.parse_comp_clauses();
            self.expect(&TokenKind::RBracket, "`]` penutup daftar");
            let span = open.merge(self.previous_span());
            return self.make_expr(
                span,
                ExprKind::ListComp {
                    elt: Box::new(first),
                    clauses,
                },
            );
        }

        let mut items = vec![first];
        while self.eat(&TokenKind::Comma) {
            if self.at(&TokenKind::RBracket) {
                break;
            }
            items.push(self.parse_maybe_starred());
        }
        self.expect(&TokenKind::RBracket, "`]` penutup daftar");
        let span = open.merge(self.previous_span());
        self.make_expr(span, ExprKind::List(items))
    }

    fn parse_set_or_dict(&mut self) -> Expr {
        let open = self.bump().span;
        if self.eat(&TokenKind::RBrace) {
            let span = open.merge(self.previous_span());
            return self.make_expr(span, ExprKind::Dict(Vec::new()));
        }

        if self.at(&TokenKind::StarStar) {
            self.bump();
            let value = self.parse_expression();
            let mut items = vec![DictItem::Unpack(value)];
            while self.eat(&TokenKind::Comma) {
                if self.at(&TokenKind::RBrace) {
                    break;
                }
                items.push(self.parse_dict_item());
            }
            self.expect(&TokenKind::RBrace, "`}` penutup kamus");
            let span = open.merge(self.previous_span());
            return self.make_expr(span, ExprKind::Dict(items));
        }

        if self.at(&TokenKind::Star) {
            let first = self.parse_set_item();
            if self.at(&TokenKind::Untuk)
                || (self.at(&TokenKind::Asinkron) && self.nth_is(1, &TokenKind::Untuk))
            {
                self.error_help(
                    "E-SINTAKS",
                    "comprehension himpunan tidak boleh memakai elemen bintang",
                    first.span,
                    "hapus tanda bintang",
                );
                let _ = self.parse_comp_clauses();
            }
            let mut items = vec![first];
            while self.eat(&TokenKind::Comma) {
                if self.at(&TokenKind::RBrace) {
                    break;
                }
                items.push(self.parse_set_item());
            }
            self.expect(&TokenKind::RBrace, "`}` penutup himpunan");
            let span = open.merge(self.previous_span());
            return self.make_expr(span, ExprKind::Set(items));
        }

        let first = self.parse_expression();
        if self.eat(&TokenKind::Colon) {
            let value = self.parse_expression();
            if self.at(&TokenKind::Untuk)
                || (self.at(&TokenKind::Asinkron) && self.nth_is(1, &TokenKind::Untuk))
            {
                let clauses = self.parse_comp_clauses();
                self.expect(&TokenKind::RBrace, "`}` penutup kamus");
                let span = open.merge(self.previous_span());
                return self.make_expr(
                    span,
                    ExprKind::DictComp {
                        key: Box::new(first),
                        value: Box::new(value),
                        clauses,
                    },
                );
            }

            let mut items = vec![DictItem::Pair(first, value)];
            while self.eat(&TokenKind::Comma) {
                if self.at(&TokenKind::RBrace) {
                    break;
                }
                items.push(self.parse_dict_item());
            }
            self.expect(&TokenKind::RBrace, "`}` penutup kamus");
            let span = open.merge(self.previous_span());
            return self.make_expr(span, ExprKind::Dict(items));
        }

        if self.at(&TokenKind::Untuk)
            || (self.at(&TokenKind::Asinkron) && self.nth_is(1, &TokenKind::Untuk))
        {
            let clauses = self.parse_comp_clauses();
            self.expect(&TokenKind::RBrace, "`}` penutup himpunan");
            let span = open.merge(self.previous_span());
            return self.make_expr(
                span,
                ExprKind::SetComp {
                    elt: Box::new(first),
                    clauses,
                },
            );
        }

        let mut items = vec![first];
        while self.eat(&TokenKind::Comma) {
            if self.at(&TokenKind::RBrace) {
                break;
            }
            items.push(self.parse_set_item());
        }
        self.expect(&TokenKind::RBrace, "`}` penutup himpunan");
        let span = open.merge(self.previous_span());
        self.make_expr(span, ExprKind::Set(items))
    }

    fn parse_dict_item(&mut self) -> DictItem {
        if self.eat(&TokenKind::StarStar) {
            return DictItem::Unpack(self.parse_expression());
        }
        let key = self.parse_expression();
        self.expect(&TokenKind::Colon, "`:` antara kunci dan nilai kamus");
        let value = self.parse_expression();
        DictItem::Pair(key, value)
    }

    fn parse_set_item(&mut self) -> Expr {
        if self.at(&TokenKind::Star) {
            let start = self.bump().span;
            let value = self.parse_expression();
            let span = start.merge(value.span);
            return self.make_expr(span, ExprKind::Starred(Box::new(value)));
        }
        self.parse_expression()
    }

    pub(crate) fn parse_expression_list(&mut self) -> Expr {
        let first = self.parse_maybe_starred();
        if !self.at(&TokenKind::Comma) {
            return first;
        }

        let start = first.span;
        let mut items = vec![first];
        while self.eat(&TokenKind::Comma) {
            if self.at_expression_list_end() {
                break;
            }
            items.push(self.parse_maybe_starred());
        }
        let end = items
            .last()
            .map(|expr| expr.span)
            .unwrap_or(start);
        self.make_expr(start.merge(end), ExprKind::Tuple(items))
    }

    /// Membaca daftar target penugasan tanpa menelan operator perbandingan.
    ///
    /// Target perulangan seperti `untuk x dalam data` tidak boleh membaca
    /// `dalam data` sebagai perbandingan, sehingga target dibatasi pada nama,
    /// atribut, indeks, dan pola tuple/daftar.
    pub(crate) fn parse_target_list(&mut self) -> Expr {
        let first = self.parse_target_item();
        if !self.at(&TokenKind::Comma) {
            return first;
        }

        let start = first.span;
        let mut items = vec![first];
        while self.eat(&TokenKind::Comma) {
            if self.at_expression_list_end() {
                break;
            }
            items.push(self.parse_target_item());
        }
        let end = items
            .last()
            .map(|expr| expr.span)
            .unwrap_or(start);
        self.make_expr(start.merge(end), ExprKind::Tuple(items))
    }

    fn parse_target_item(&mut self) -> Expr {
        if self.at(&TokenKind::Star) {
            let start = self.bump().span;
            let value = self.parse_target_item();
            let span = start.merge(value.span);
            return self.make_expr(span, ExprKind::Starred(Box::new(value)));
        }

        if self.at(&TokenKind::LParen) {
            let open = self.bump().span;
            if self.eat(&TokenKind::RParen) {
                let span = open.merge(self.previous_span());
                return self.make_expr(span, ExprKind::Tuple(Vec::new()));
            }
            let first = self.parse_target_item();
            let mut items = vec![first];
            let mut saw_comma = false;
            while self.eat(&TokenKind::Comma) {
                saw_comma = true;
                if self.at(&TokenKind::RParen) {
                    break;
                }
                items.push(self.parse_target_item());
            }
            self.expect(&TokenKind::RParen, "`)` penutup target");
            let span = open.merge(self.previous_span());
            if !saw_comma && items.len() == 1 {
                let mut expr = items.pop().expect("satu target");
                expr.span = span;
                return expr;
            }
            return self.make_expr(span, ExprKind::Tuple(items));
        }

        if self.at(&TokenKind::LBracket) {
            let open = self.bump().span;
            if self.eat(&TokenKind::RBracket) {
                let span = open.merge(self.previous_span());
                return self.make_expr(span, ExprKind::List(Vec::new()));
            }
            let mut items = vec![self.parse_target_item()];
            while self.eat(&TokenKind::Comma) {
                if self.at(&TokenKind::RBracket) {
                    break;
                }
                items.push(self.parse_target_item());
            }
            self.expect(&TokenKind::RBracket, "`]` penutup target");
            let span = open.merge(self.previous_span());
            return self.make_expr(span, ExprKind::List(items));
        }

        self.parse_postfix()
    }

    fn at_expression_list_end(&self) -> bool {
        matches!(
            self.current().kind,
            TokenKind::Newline
                | TokenKind::Semi
                | TokenKind::Dedent
                | TokenKind::Eof
                | TokenKind::RParen
                | TokenKind::RBracket
                | TokenKind::RBrace
                | TokenKind::Colon
                | TokenKind::Eq
                | TokenKind::PlusEq
                | TokenKind::MinusEq
                | TokenKind::StarEq
                | TokenKind::SlashEq
                | TokenKind::SlashSlashEq
                | TokenKind::PercentEq
                | TokenKind::StarStarEq
                | TokenKind::AtEq
                | TokenKind::AmpEq
                | TokenKind::PipeEq
                | TokenKind::CaretEq
                | TokenKind::LShiftEq
                | TokenKind::RShiftEq
                | TokenKind::QuestionQuestionEq
        )
    }

    fn parse_maybe_starred(&mut self) -> Expr {
        if self.at(&TokenKind::Star) {
            let start = self.bump().span;
            let value = self.parse_expression();
            let span = start.merge(value.span);
            return self.make_expr(span, ExprKind::Starred(Box::new(value)));
        }
        self.parse_expression()
    }

    fn parse_comp_clauses(&mut self) -> Vec<CompClause> {
        let mut clauses = Vec::new();
        loop {
            let is_async = if self.at(&TokenKind::Asinkron) {
                self.bump();
                true
            } else {
                false
            };

            if self.eat(&TokenKind::Untuk) {
                let target = self.parse_target_list();
                self.validate_target(&target, "perulangan comprehension");
                self.expect(&TokenKind::Dalam, "`dalam` pada comprehension");
                let iter = self.parse_null_coalesce();
                clauses.push(CompClause::For {
                    target,
                    iter,
                    is_async,
                });
                continue;
            }

            if is_async {
                let span = self.current().span;
                self.error_help(
                    "E-SINTAKS",
                    "`asinkron` pada comprehension harus diikuti `untuk`",
                    span,
                    "tulis `asinkron untuk`",
                );
                break;
            }

            if self.eat(&TokenKind::Jika) {
                let condition = self.parse_null_coalesce();
                clauses.push(CompClause::If(condition));
                continue;
            }
            break;
        }

        if clauses.is_empty() {
            let span = self.current().span;
            self.error_help(
                "E-SINTAKS",
                "comprehension membutuhkan minimal satu klausa `untuk`",
                span,
                "tulis `untuk nama dalam iterable`",
            );
        }
        clauses
    }

    fn parse_lambda(&mut self) -> Expr {
        let start = self.bump().span;
        let params = self.parse_parameters(&TokenKind::Colon, false);
        self.expect(&TokenKind::Colon, "`:` setelah parameter lambda");
        let body = self.parse_expression();
        let span = start.merge(body.span);
        self.make_expr(
            span,
            ExprKind::Lambda {
                params,
                body: Box::new(body),
            },
        )
    }

    fn parse_yield_expression(&mut self) -> Expr {
        let start = self.bump().span;
        let from = self.eat(&TokenKind::Dari);
        let value = if self.at_expression_end() {
            None
        } else {
            Some(self.parse_expression_list())
        };
        if from && value.is_none() {
            let span = start.merge(self.previous_span());
            self.error_help(
                "E-SINTAKS",
                "`hasilkan dari` membutuhkan ekspresi",
                span,
                "tambahkan iterable setelah `dari`",
            );
        }
        let span = start.merge(self.previous_span());
        self.make_expr(span, ExprKind::Yield { value: value.map(Box::new), from })
    }

    pub(crate) fn at_expression_end(&self) -> bool {
        matches!(
            self.current().kind,
            TokenKind::Newline
                | TokenKind::Semi
                | TokenKind::Dedent
                | TokenKind::Eof
                | TokenKind::RParen
                | TokenKind::RBracket
                | TokenKind::RBrace
                | TokenKind::Colon
                | TokenKind::Comma
                | TokenKind::Eq
                | TokenKind::Arrow
        )
    }

    fn parse_call(&mut self, func: Expr, safe: bool) -> Expr {
        let mut args = Vec::new();
        let mut seen_keyword = false;

        if !self.at(&TokenKind::RParen) {
            loop {
                if self.eat(&TokenKind::StarStar) {
                    let value = self.parse_expression();
                    args.push(Arg::KwStar(value));
                } else if self.eat(&TokenKind::Star) {
                    let value = self.parse_expression();
                    args.push(Arg::Star(value));
                } else if self.at_name() && self.nth_is(1, &TokenKind::Eq) {
                    let (name, _) = self
                        .expect_name("argumen kata kunci")
                        .expect("nama sudah diperiksa");
                    self.bump();
                    let value = self.parse_expression();
                    seen_keyword = true;
                    args.push(Arg::Keyword { name, value });
                } else {
                    let value = self.parse_expression();
                    if args.is_empty()
                        && (self.at(&TokenKind::Untuk)
                            || (self.at(&TokenKind::Asinkron)
                                && self.nth_is(1, &TokenKind::Untuk)))
                    {
                        let clauses = self.parse_comp_clauses();
                        let span = value.span.merge(self.previous_span());
                        let generator = self.make_expr(
                            span,
                            ExprKind::GeneratorExp {
                                elt: Box::new(value),
                                clauses,
                            },
                        );
                        args.push(Arg::Positional(generator));
                        break;
                    }
                    if seen_keyword {
                        self.error_help(
                            "E-SINTAKS",
                            "argumen posisional tidak boleh setelah argumen kata kunci",
                            value.span,
                            "pindahkan argumen posisional ke depan",
                        );
                    }
                    args.push(Arg::Positional(value));
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

        self.expect(&TokenKind::RParen, "`)` penutup argumen");
        let span = func.span.merge(self.previous_span());
        self.make_expr(
            span,
            ExprKind::Call {
                func: Box::new(func),
                args,
                safe,
            },
        )
    }

    fn parse_subscript(&mut self, value: Expr, safe: bool) -> Expr {
        let mut items = Vec::new();
        let mut saw_comma = false;

        if self.at(&TokenKind::RBracket) {
            let span = self.current().span;
            self.error_help(
                "E-SINTAKS",
                "indeks tidak boleh kosong",
                span,
                "tambahkan ekspresi atau slice di dalam `[]`",
            );
            self.bump();
            let span = value.span.merge(self.previous_span());
            let missing = self.make_expr(span, ExprKind::None);
            return self.make_expr(
                span,
                ExprKind::Subscript {
                    value: Box::new(value),
                    index: Box::new(missing),
                    safe,
                },
            );
        }

        loop {
            items.push(self.parse_subscript_item());
            if self.eat(&TokenKind::Comma) {
                saw_comma = true;
                if self.at(&TokenKind::RBracket) {
                    break;
                }
                continue;
            }
            break;
        }

        self.expect(&TokenKind::RBracket, "`]` penutup indeks");
        let index = if items.len() == 1 && !saw_comma {
            items.pop().expect("satu indeks")
        } else {
            let start = items
                .first()
                .map(|expr| expr.span)
                .unwrap_or_else(|| value.span);
            let end = items
                .last()
                .map(|expr| expr.span)
                .unwrap_or(start);
            self.make_expr(start.merge(end), ExprKind::Tuple(items))
        };
        let span = value.span.merge(self.previous_span());
        self.make_expr(
            span,
            ExprKind::Subscript {
                value: Box::new(value),
                index: Box::new(index),
                safe,
            },
        )
    }

    fn parse_subscript_item(&mut self) -> Expr {
        if self.at(&TokenKind::Colon) {
            return self.parse_slice(None, self.current().span);
        }
        if self.at(&TokenKind::Star) {
            let start = self.bump().span;
            let value = self.parse_expression();
            let span = start.merge(value.span);
            return self.make_expr(span, ExprKind::Starred(Box::new(value)));
        }
        let first = self.parse_expression();
        if self.at(&TokenKind::Colon) {
            let span = first.span;
            return self.parse_slice(Some(first), span);
        }
        first
    }

    fn parse_slice(&mut self, start: Option<Expr>, start_span: aksara_ast::Span) -> Expr {
        self.expect(&TokenKind::Colon, "`:` pada slice");
        let stop = if self.at(&TokenKind::Colon)
            || self.at(&TokenKind::Comma)
            || self.at(&TokenKind::RBracket)
            || self.at_eof()
        {
            None
        } else {
            Some(Box::new(self.parse_expression()))
        };
        let step = if self.eat(&TokenKind::Colon) {
            if self.at(&TokenKind::Comma)
                || self.at(&TokenKind::RBracket)
                || self.at_eof()
            {
                None
            } else {
                Some(Box::new(self.parse_expression()))
            }
        } else {
            None
        };
        let span = start_span.merge(self.previous_span());
        self.make_expr(
            span,
            ExprKind::Slice {
                start: start.map(Box::new),
                stop,
                step,
            },
        )
    }

    pub(crate) fn parse_dotted_name(&mut self) -> Option<(String, aksara_ast::Span)> {
        let (first, first_span) = self.expect_name("modul")?;
        let mut name = first;
        let mut span = first_span;
        while self.at(&TokenKind::Dot) {
            self.bump();
            match self.expect_name("setelah titik pada nama modul") {
                Some((part, part_span)) => {
                    name.push('.');
                    name.push_str(&part);
                    span = span.merge(part_span);
                }
                None => break,
            }
        }
        Some((name, span))
    }

    pub(crate) fn parse_name_list(&mut self) -> Vec<String> {
        let mut names = Vec::new();
        loop {
            match self.expect_name("variabel") {
                Some((name, _)) => names.push(name),
                None => break,
            }
            if self.eat(&TokenKind::Comma) {
                if self.at_statement_end() {
                    break;
                }
                continue;
            }
            break;
        }
        names
    }

    pub(crate) fn validate_target(&mut self, expr: &Expr, context: &str) {
        if !Self::is_valid_target(expr, false) {
            let span = expr.span;
            self.error_help(
                "E-PENUGASAN",
                format!("target {context} tidak valid"),
                span,
                "target harus berupa nama, atribut, indeks, atau pola tuple/daftar",
            );
        }
    }

    pub(crate) fn validate_simple_target(&mut self, expr: &Expr, context: &str) {
        if !Self::is_simple_target(expr) {
            let span = expr.span;
            self.error_help(
                "E-PENUGASAN",
                format!("target {context} harus berupa nama, atribut, atau indeks"),
                span,
                "anotasi dan penugasan gabungan tidak menerima pola tuple",
            );
        }
    }

    pub(crate) fn is_valid_target(expr: &Expr, allow_starred: bool) -> bool {
        match &expr.kind {
            ExprKind::Name(_) | ExprKind::Attribute { .. } | ExprKind::Subscript { .. } => true,
            ExprKind::Tuple(items) | ExprKind::List(items) => items
                .iter()
                .all(|item| Self::is_valid_target(item, true)),
            ExprKind::Starred(inner) => allow_starred && Self::is_valid_target(inner, false),
            _ => false,
        }
    }

    fn is_simple_target(expr: &Expr) -> bool {
        matches!(
            expr.kind,
            ExprKind::Name(_) | ExprKind::Attribute { .. } | ExprKind::Subscript { .. }
        )
    }

    /// Mengubah literal teks sederhana menjadi nilai string untuk nama modul.
    pub(crate) fn simple_string_value(lit: &StringLit) -> Option<String> {
        if lit.prefix.contains('b') || lit.prefix.contains('f') {
            return None;
        }
        let raw = lit.raw.as_str();
        if lit.prefix.contains('r') {
            return Some(raw.to_string());
        }
        let mut out = String::with_capacity(raw.len());
        let mut chars = raw.chars().peekable();
        while let Some(ch) = chars.next() {
            if ch != '\\' {
                out.push(ch);
                continue;
            }
            let Some(escape) = chars.next() else {
                out.push('\\');
                break;
            };
            match escape {
                '\\' => out.push('\\'),
                '\'' => out.push('\''),
                '"' => out.push('"'),
                'n' => out.push('\n'),
                'r' => out.push('\r'),
                't' => out.push('\t'),
                'b' => out.push('\u{0008}'),
                'f' => out.push('\u{000c}'),
                'v' => out.push('\u{000b}'),
                '0' => out.push('\0'),
                'x' => {
                    let hex: String = chars.by_ref().take(2).collect();
                    if let Ok(value) = u8::from_str_radix(&hex, 16) {
                        out.push(value as char);
                    } else {
                        out.push('\\');
                        out.push('x');
                        out.push_str(&hex);
                    }
                }
                'u' => {
                    let hex: String = chars.by_ref().take(4).collect();
                    if let Ok(value) = u32::from_str_radix(&hex, 16) {
                        if let Some(value) = char::from_u32(value) {
                            out.push(value);
                        }
                    }
                }
                'U' => {
                    let hex: String = chars.by_ref().take(8).collect();
                    if let Ok(value) = u32::from_str_radix(&hex, 16) {
                        if let Some(value) = char::from_u32(value) {
                            out.push(value);
                        }
                    }
                }
                other => {
                    out.push('\\');
                    out.push(other);
                }
            }
        }
        Some(out)
    }
}
