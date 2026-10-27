use std::collections::HashSet;

use aksara_ast::{
    Arg, AugOp, ClassDef, ExceptHandler, ForIter, FunctionDef, ImportAlias, MatchCase, Param,
    ParamKind, Parameters, Stmt, StmtKind, StringLit, WithItem,
};
use aksara_lexer::TokenKind;

use crate::parser::Parser;

impl Parser {
    pub(crate) fn parse_statement(&mut self) -> Vec<Stmt> {
        if self.at(&TokenKind::At) {
            return vec![self.parse_decorated()];
        }

        match self.current().kind {
            TokenKind::Jika => vec![self.parse_if()],
            TokenKind::Selama => vec![self.parse_while()],
            TokenKind::Untuk => {
                let start = self.current().span;
                vec![self.parse_for(false, start)]
            }
            TokenKind::Asinkron => self.parse_async_statement(),
            TokenKind::Fungsi => {
                let start = self.current().span;
                vec![self.parse_function(Vec::new(), false, start)]
            }
            TokenKind::Kelas => {
                let start = self.current().span;
                vec![self.parse_class(Vec::new(), start)]
            }
            TokenKind::Coba => vec![self.parse_try()],
            TokenKind::Dengan => {
                let start = self.current().span;
                vec![self.parse_with(false, start)]
            }
            TokenKind::Cocokkan => vec![self.parse_match()],
            _ => self.parse_simple_line(),
        }
    }

    fn parse_simple_line(&mut self) -> Vec<Stmt> {
        let mut statements = Vec::new();
        loop {
            match self.parse_simple_statement() {
                Some(stmt) => statements.push(stmt),
                None => {
                    self.synchronize_statement();
                    break;
                }
            }

            if self.at(&TokenKind::Semi) {
                self.bump();
                if self.at_statement_end() {
                    break;
                }
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
        statements
    }

    pub(crate) fn parse_simple_statement(&mut self) -> Option<Stmt> {
        let start = self.current().span;
        match self.current().kind {
            TokenKind::Kembalikan => {
                self.bump();
                let value = if self.at_statement_end() {
                    None
                } else {
                    Some(self.parse_expression_list())
                };
                let span = start.merge(self.previous_span());
                Some(self.make_stmt(span, StmtKind::Return(value)))
            }
            TokenKind::Hasilkan => {
                self.bump();
                let from = self.eat(&TokenKind::Dari);
                let value = if self.at_statement_end() {
                    None
                } else {
                    Some(self.parse_expression_list())
                };
                if from && value.is_none() {
                    self.error_help(
                        "E-SINTAKS",
                        "`hasilkan dari` membutuhkan ekspresi",
                        start.merge(self.previous_span()),
                        "tambahkan iterable setelah `dari`",
                    );
                }
                let span = start.merge(self.previous_span());
                Some(self.make_stmt(span, StmtKind::Yield { value, from }))
            }
            TokenKind::Hentikan => {
                self.bump();
                Some(self.make_stmt(start, StmtKind::Break))
            }
            TokenKind::Lanjutkan => {
                self.bump();
                Some(self.make_stmt(start, StmtKind::Continue))
            }
            TokenKind::Lewati => {
                self.bump();
                Some(self.make_stmt(start, StmtKind::Pass))
            }
            TokenKind::Global => {
                self.bump();
                let names = self.parse_name_list();
                if names.is_empty() {
                    self.error_help(
                        "E-SINTAKS",
                        "`global` membutuhkan minimal satu nama",
                        start.merge(self.previous_span()),
                        "tulis `global nama`",
                    );
                }
                let span = start.merge(self.previous_span());
                Some(self.make_stmt(span, StmtKind::Global(names)))
            }
            TokenKind::Nonlokal => {
                self.bump();
                let names = self.parse_name_list();
                if names.is_empty() {
                    self.error_help(
                        "E-SINTAKS",
                        "`nonlokal` membutuhkan minimal satu nama",
                        start.merge(self.previous_span()),
                        "tulis `nonlokal nama`",
                    );
                }
                let span = start.merge(self.previous_span());
                Some(self.make_stmt(span, StmtKind::Nonlocal(names)))
            }
            TokenKind::Pastikan => {
                self.bump();
                let test = self.parse_expression();
                let msg = if self.eat(&TokenKind::Comma) {
                    Some(self.parse_expression())
                } else {
                    None
                };
                let span = start.merge(self.previous_span());
                Some(self.make_stmt(span, StmtKind::Assert { test, msg }))
            }
            TokenKind::Hapus => {
                self.bump();
                let mut targets = Vec::new();
                loop {
                    let target = self.parse_expression();
                    self.validate_target(&target, "`hapus`");
                    targets.push(target);
                    if self.eat(&TokenKind::Comma) {
                        if self.at_statement_end() {
                            break;
                        }
                        continue;
                    }
                    break;
                }
                if targets.is_empty() {
                    self.error_help(
                        "E-SINTAKS",
                        "`hapus` membutuhkan minimal satu target",
                        start.merge(self.previous_span()),
                        "tulis `hapus nama`",
                    );
                }
                let span = start.merge(self.previous_span());
                Some(self.make_stmt(span, StmtKind::Delete(targets)))
            }
            TokenKind::Lempar => {
                self.bump();
                let exc = if self.at_statement_end() {
                    None
                } else {
                    Some(self.parse_expression())
                };
                let cause = if self.eat(&TokenKind::Dari) {
                    Some(self.parse_expression())
                } else {
                    None
                };
                let span = start.merge(self.previous_span());
                Some(self.make_stmt(span, StmtKind::Raise { exc, cause }))
            }
            TokenKind::Impor => Some(self.parse_import_statement(start)),
            TokenKind::Dari => Some(self.parse_from_import(start)),
            _ => self.parse_assignment_or_expression(start),
        }
    }

    fn parse_assignment_or_expression(&mut self, start: aksara_ast::Span) -> Option<Stmt> {
        let first = self.parse_expression_list();

        if self.at(&TokenKind::Colon) {
            self.validate_simple_target(&first, "anotasi");
            self.bump();
            let annotation = self.parse_expression();
            let value = if self.eat(&TokenKind::Eq) {
                Some(self.parse_expression())
            } else {
                None
            };
            let span = start.merge(self.previous_span());
            return Some(self.make_stmt(
                span,
                StmtKind::AnnAssign {
                    target: first,
                    annotation,
                    value,
                },
            ));
        }

        if let Some(op) = self.current_aug_op() {
            self.validate_simple_target(&first, "penugasan gabungan");
            self.bump();
            let value = self.parse_expression();
            let span = start.merge(self.previous_span());
            return Some(self.make_stmt(
                span,
                StmtKind::AugAssign {
                    target: first,
                    op,
                    value,
                },
            ));
        }

        if self.at(&TokenKind::Eq) {
            let mut targets = vec![first];
            let value;
            loop {
                self.bump();
                let next = self.parse_expression_list();
                if self.at(&TokenKind::Eq) {
                    self.validate_target(&next, "penugasan berantai");
                    targets.push(next);
                    continue;
                }
                value = next;
                break;
            }
            for target in &targets {
                self.validate_target(target, "penugasan");
            }
            let span = start.merge(self.previous_span());
            return Some(self.make_stmt(span, StmtKind::Assign { targets, value }));
        }

        let span = start.merge(first.span);
        Some(self.make_stmt(span, StmtKind::Expr(first)))
    }

    fn current_aug_op(&self) -> Option<AugOp> {
        Some(match self.current().kind {
            TokenKind::PlusEq => AugOp::Add,
            TokenKind::MinusEq => AugOp::Sub,
            TokenKind::StarEq => AugOp::Mul,
            TokenKind::SlashEq => AugOp::Div,
            TokenKind::SlashSlashEq => AugOp::FloorDiv,
            TokenKind::PercentEq => AugOp::Mod,
            TokenKind::StarStarEq => AugOp::Pow,
            TokenKind::AtEq => AugOp::MatMul,
            TokenKind::AmpEq => AugOp::BitAnd,
            TokenKind::PipeEq => AugOp::BitOr,
            TokenKind::CaretEq => AugOp::BitXor,
            TokenKind::LShiftEq => AugOp::LShift,
            TokenKind::RShiftEq => AugOp::RShift,
            TokenKind::QuestionQuestionEq => AugOp::NullCoalesce,
            _ => return None,
        })
    }

    fn parse_if(&mut self) -> Stmt {
        let start = self.bump();
        let condition = self.parse_expression();
        let condition = self.recover_condition_assignment(condition);
        self.expect(&TokenKind::Colon, "`:` setelah kondisi `jika`");
        let body = self.parse_suite();
        let orelse = self.parse_else_clause();
        let body_end = self.span_of_statements(&body, start);
        let end = self.span_of_statements(&orelse, body_end);
        let span = start.merge(end);
        self.make_stmt(
            span,
            StmtKind::If {
                condition,
                body,
                orelse,
            },
        )
    }

    /// Menangani `jika x = 1:` dengan pesan yang menyarankan `==` atau `:=`.
    fn recover_condition_assignment(&mut self, condition: aksara_ast::Expr) -> aksara_ast::Expr {
        if !self.at(&TokenKind::Eq) {
            return condition;
        }
        let span = self.current().span;
        self.error_help(
            "E-PENUGASAN",
            "penugasan tidak diizinkan di dalam kondisi",
            span,
            "gunakan `==` untuk perbandingan atau `(x := nilai)` untuk penugasan",
        );
        self.bump();
        let _ = self.parse_expression();
        condition
    }

    fn parse_else_clause(&mut self) -> Vec<Stmt> {
        if !self.at(&TokenKind::Lainnya) {
            return Vec::new();
        }
        let start = self.bump();
        if self.eat(&TokenKind::Jika) {
            let condition = self.parse_expression();
            self.expect(&TokenKind::Colon, "`:` setelah kondisi `lainnya jika`");
            let body = self.parse_suite();
            let orelse = self.parse_else_clause();
            let body_end = self.span_of_statements(&body, start);
            let end = self.span_of_statements(&orelse, body_end);
            let span = start.merge(end);
            let stmt = self.make_stmt(
                span,
                StmtKind::If {
                    condition,
                    body,
                    orelse,
                },
            );
            vec![stmt]
        } else {
            self.expect(&TokenKind::Colon, "`:` setelah `lainnya`");
            self.parse_suite()
        }
    }

    fn parse_else_suite(&mut self) -> Vec<Stmt> {
        if !self.at(&TokenKind::Lainnya) {
            return Vec::new();
        }
        self.bump();
        self.expect(&TokenKind::Colon, "`:` setelah `lainnya`");
        self.parse_suite()
    }

    fn parse_while(&mut self) -> Stmt {
        let start = self.bump();
        let condition = self.parse_expression();
        let condition = self.recover_condition_assignment(condition);
        self.expect(&TokenKind::Colon, "`:` setelah kondisi `selama`");
        let body = self.parse_suite();
        let orelse = self.parse_else_suite();
        let body_end = self.span_of_statements(&body, start);
        let end = self.span_of_statements(&orelse, body_end);
        self.make_stmt(
            start.merge(end),
            StmtKind::While {
                condition,
                body,
                orelse,
            },
        )
    }

    fn parse_for(&mut self, is_async: bool, start: aksara_ast::Span) -> Stmt {
        self.expect(&TokenKind::Untuk, "`untuk`");
        let setiap = if self.at_name_text("setiap") {
            self.bump();
            true
        } else {
            false
        };
        let target = self.parse_target_list();
        self.validate_target(&target, "perulangan");

        let iter = if self.eat(&TokenKind::Dalam) {
            let iterable = self.parse_expression();
            if setiap {
                ForIter::Each(iterable)
            } else {
                ForIter::In(iterable)
            }
        } else if setiap {
            let span = self.current().span;
            self.error_help(
                "E-SINTAKS",
                "`dalam` diharapkan setelah `untuk setiap`",
                span,
                "tulis `untuk setiap nama dalam iterable`",
            );
            ForIter::In(self.missing_expression("iterable"))
        } else if self.at_name_text("sebanyak") {
            self.bump();
            ForIter::Times(self.parse_expression())
        } else if self.eat(&TokenKind::Dari) {
            let range_start = self.parse_expression();
            self.expect_contextual("sampai", "`sampai` pada perulangan `dari`");
            let range_end = self.parse_expression();
            let step = if self.at_name_text("langkah") {
                self.bump();
                Some(self.parse_expression())
            } else {
                None
            };
            ForIter::Range {
                start: range_start,
                end: range_end,
                step,
            }
        } else {
            let span = self.current().span;
            self.error_help(
                "E-SINTAKS",
                "sumber perulangan tidak dikenali",
                span,
                "gunakan `dalam`, `setiap`, `sebanyak`, atau `dari ... sampai ...`",
            );
            ForIter::In(self.missing_expression("iterable"))
        };

        self.expect(&TokenKind::Colon, "`:` setelah perulangan");
        let body = self.parse_suite();
        let orelse = self.parse_else_suite();
        let body_end = self.span_of_statements(&body, start);
        let end = self.span_of_statements(&orelse, body_end);
        self.make_stmt(
            start.merge(end),
            StmtKind::For {
                target,
                iter,
                body,
                orelse,
                is_async,
            },
        )
    }

    fn parse_try(&mut self) -> Stmt {
        let start = self.bump();
        self.expect(&TokenKind::Colon, "`:` setelah `coba`");
        let body = self.parse_suite();
        let mut handlers = Vec::new();

        while self.at(&TokenKind::Tangkap) {
            let handler_start = self.bump();
            let mut type_ = None;
            let mut name = None;
            if !self.at(&TokenKind::Colon) {
                if self.eat(&TokenKind::Sebagai) {
                    if let Some((value, _)) = self.expect_name("variabel kesalahan") {
                        name = Some(value);
                    }
                } else {
                    type_ = Some(self.parse_expression());
                    if self.eat(&TokenKind::Sebagai) {
                        if let Some((value, _)) = self.expect_name("variabel kesalahan") {
                            name = Some(value);
                        }
                    }
                }
            }
            self.expect(&TokenKind::Colon, "`:` setelah `tangkap`");
            let handler_body = self.parse_suite();
            let handler_end = self.span_of_statements(&handler_body, handler_start);
            handlers.push(ExceptHandler {
                type_,
                name,
                body: handler_body,
                span: handler_start.merge(handler_end),
            });
        }

        let orelse = self.parse_else_suite();
        let finalbody = if self.at(&TokenKind::Akhirnya) {
            self.bump();
            self.expect(&TokenKind::Colon, "`:` setelah `akhirnya`");
            self.parse_suite()
        } else {
            Vec::new()
        };

        if handlers.is_empty() && finalbody.is_empty() {
            let span = start.merge(self.previous_span());
            self.error_help(
                "E-SINTAKS",
                "`coba` membutuhkan minimal satu `tangkap` atau `akhirnya`",
                span,
                "tambahkan `tangkap` atau `akhirnya`",
            );
        }

        let mut end = self.span_of_statements(&body, start);
        for handler in &handlers {
            end = end.merge(handler.span);
        }
        end = self.span_of_statements(&orelse, end);
        end = self.span_of_statements(&finalbody, end);
        self.make_stmt(
            start.merge(end),
            StmtKind::Try {
                body,
                handlers,
                orelse,
                finalbody,
            },
        )
    }

    fn parse_with(&mut self, is_async: bool, start: aksara_ast::Span) -> Stmt {
        self.expect(&TokenKind::Dengan, "`dengan`");
        let mut items = Vec::new();
        loop {
            let context = self.parse_expression();
            let target = if self.eat(&TokenKind::Sebagai) {
                let target = self.parse_expression();
                self.validate_target(&target, "`dengan ... sebagai`");
                Some(target)
            } else {
                None
            };
            items.push(WithItem { context, target });
            if self.eat(&TokenKind::Comma) {
                if self.at(&TokenKind::Colon) {
                    break;
                }
                continue;
            }
            break;
        }
        self.expect(&TokenKind::Colon, "`:` setelah `dengan`");
        let body = self.parse_suite();
        let end = self.span_of_statements(&body, start);
        self.make_stmt(
            start.merge(end),
            StmtKind::With {
                items,
                body,
                is_async,
            },
        )
    }

    fn parse_async_statement(&mut self) -> Vec<Stmt> {
        let start = self.bump();
        if self.at(&TokenKind::Fungsi) {
            vec![self.parse_function(Vec::new(), true, start)]
        } else if self.at(&TokenKind::Untuk) {
            vec![self.parse_for(true, start)]
        } else if self.at(&TokenKind::Dengan) {
            vec![self.parse_with(true, start)]
        } else {
            let span = self.current().span;
            self.error_help(
                "E-SINTAKS",
                "`asinkron` harus diikuti `fungsi`, `untuk`, atau `dengan`",
                span,
                "tulis salah satu bentuk asinkron yang didukung",
            );
            self.synchronize_statement();
            Vec::new()
        }
    }

    fn parse_decorated(&mut self) -> Stmt {
        let start = self.current().span;
        let mut decorators = Vec::new();
        while self.at(&TokenKind::At) {
            self.bump();
            decorators.push(self.parse_expression());
            if self.at(&TokenKind::Newline) {
                self.bump();
            } else {
                self.expect(&TokenKind::Newline, "akhir baris setelah dekorator");
            }
        }

        if self.at(&TokenKind::Asinkron) {
            let async_start = self.bump();
            if self.at(&TokenKind::Fungsi) {
                return self.parse_function(decorators, true, start.merge(async_start));
            }
            let span = self.current().span;
            self.error_help(
                "E-SINTAKS",
                "dekorator hanya dapat diberikan pada fungsi atau kelas",
                span,
                "letakkan dekorator tepat sebelum `fungsi`, `asinkron fungsi`, atau `kelas`",
            );
            self.synchronize_statement();
            return self.make_stmt(start, StmtKind::Pass);
        }
        if self.at(&TokenKind::Fungsi) {
            return self.parse_function(decorators, false, start);
        }
        if self.at(&TokenKind::Kelas) {
            return self.parse_class(decorators, start);
        }

        let span = self.current().span;
        self.error_help(
            "E-SINTAKS",
            "dekorator hanya dapat diberikan pada fungsi atau kelas",
            span,
            "letakkan dekorator tepat sebelum `fungsi`, `asinkron fungsi`, atau `kelas`",
        );
        self.synchronize_statement();
        self.make_stmt(start, StmtKind::Pass)
    }

    fn parse_function(
        &mut self,
        decorators: Vec<aksara_ast::Expr>,
        is_async: bool,
        start: aksara_ast::Span,
    ) -> Stmt {
        self.expect(&TokenKind::Fungsi, "`fungsi`");
        let (name, _) = self
            .expect_name("fungsi")
            .unwrap_or_else(|| ("<rusak>".to_string(), start));
        self.expect(&TokenKind::LParen, "`(` setelah nama fungsi");
        let params = self.parse_parameters(&TokenKind::RParen, true);
        self.expect(&TokenKind::RParen, "`)` penutup parameter");
        let returns = if self.eat(&TokenKind::Arrow) {
            Some(self.parse_expression())
        } else {
            None
        };
        self.expect(&TokenKind::Colon, "`:` sebelum badan fungsi");
        let body = self.parse_suite();
        let end = self.span_of_statements(&body, start);
        self.make_stmt(
            start.merge(end),
            StmtKind::Function(FunctionDef {
                name,
                params,
                returns,
                body,
                decorators,
                is_async,
            }),
        )
    }

    fn parse_class(&mut self, decorators: Vec<aksara_ast::Expr>, start: aksara_ast::Span) -> Stmt {
        self.expect(&TokenKind::Kelas, "`kelas`");
        let (name, _) = self
            .expect_name("kelas")
            .unwrap_or_else(|| ("<rusak>".to_string(), start));
        let mut bases = Vec::new();
        let mut keywords = Vec::new();

        if self.eat(&TokenKind::LParen) {
            if !self.at(&TokenKind::RParen) {
                loop {
                    if self.eat(&TokenKind::StarStar) {
                        let value = self.parse_expression();
                        keywords.push(Arg::KwStar(value));
                    } else if self.eat(&TokenKind::Star) {
                        let value = self.parse_expression();
                        let span = value.span;
                        let starred =
                            self.make_expr(span, aksara_ast::ExprKind::Starred(Box::new(value)));
                        bases.push(starred);
                    } else if self.at_name() && self.nth_is(1, &TokenKind::Eq) {
                        let (name, _) = self
                            .expect_name("argumen kata kunci kelas")
                            .expect("nama sudah diperiksa");
                        self.bump();
                        let value = self.parse_expression();
                        keywords.push(Arg::Keyword { name, value });
                    } else {
                        bases.push(self.parse_expression());
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
            self.expect(&TokenKind::RParen, "`)` penutup daftar kelas dasar");
        }

        self.expect(&TokenKind::Colon, "`:` sebelum badan kelas");
        let body = self.parse_suite();
        let end = self.span_of_statements(&body, start);
        self.make_stmt(
            start.merge(end),
            StmtKind::Class(ClassDef {
                name,
                bases,
                keywords,
                body,
                decorators,
            }),
        )
    }

    pub(crate) fn parse_parameters(
        &mut self,
        terminator: &TokenKind,
        allow_annotations: bool,
    ) -> Parameters {
        let mut params = Parameters::empty();
        let mut in_kwonly = false;
        let mut seen_slash = false;
        let mut seen_vararg = false;
        let mut seen_kwarg = false;
        let mut bare_star: Option<aksara_ast::Span> = None;
        let mut seen_default = false;

        loop {
            if self.at(terminator) || self.at_eof() || self.at(&TokenKind::Newline) {
                break;
            }

            if self.at(&TokenKind::Slash) {
                let span = self.bump();
                if seen_slash || seen_vararg || seen_kwarg || in_kwonly {
                    self.error_help(
                        "E-PARAMETER",
                        "penanda `/` tidak valid pada posisi ini",
                        span,
                        "letakkan `/` setelah parameter posisional sebelum `*`",
                    );
                }
                seen_slash = true;
                if params.args.is_empty() {
                    self.error_help(
                        "E-PARAMETER",
                        "`/` membutuhkan parameter posisional di sebelah kiri",
                        span,
                        "tambahkan minimal satu parameter sebelum `/`",
                    );
                }
                for mut param in params.args.drain(..) {
                    param.kind = ParamKind::PosOnly;
                    params.posonly.push(param);
                }
                if self.at(&TokenKind::Comma) {
                    self.bump();
                } else if !self.at(terminator) {
                    self.error_help(
                        "E-PARAMETER",
                        "`/` harus diikuti koma",
                        span,
                        "tulis `,` setelah `/`",
                    );
                }
                continue;
            }

            if self.at(&TokenKind::Star) {
                let span = self.bump();
                if seen_vararg || seen_kwarg {
                    self.error_help(
                        "E-PARAMETER",
                        "parameter bintang tidak valid pada posisi ini",
                        span,
                        "letakkan `*args` sebelum parameter kata kunci",
                    );
                }
                seen_vararg = true;
                in_kwonly = true;
                if self.at(&TokenKind::Comma) {
                    bare_star = Some(span);
                    self.bump();
                    continue;
                }
                if self.at(terminator) {
                    bare_star = Some(span);
                    continue;
                }
                let (name, name_span) = match self.expect_name("parameter bintang") {
                    Some(value) => value,
                    None => break,
                };
                let annotation = if allow_annotations && self.eat(&TokenKind::Colon) {
                    Some(self.parse_expression())
                } else {
                    None
                };
                let param_span = span.merge(self.previous_span());
                params.vararg = Some(Box::new(Param {
                    name,
                    kind: ParamKind::VarArg,
                    annotation,
                    default: None,
                    span: param_span,
                }));
                let _ = name_span;
                if self.at(&TokenKind::Comma) {
                    self.bump();
                }
                continue;
            }

            if self.at(&TokenKind::StarStar) {
                let span = self.bump();
                if seen_kwarg {
                    self.error_help(
                        "E-PARAMETER",
                        "hanya satu `**kwargs` yang diizinkan",
                        span,
                        "hapus deklarasi `**` yang berulang",
                    );
                }
                seen_kwarg = true;
                in_kwonly = true;
                let (name, _) = match self.expect_name("parameter `**`") {
                    Some(value) => value,
                    None => break,
                };
                let annotation = if allow_annotations && self.eat(&TokenKind::Colon) {
                    Some(self.parse_expression())
                } else {
                    None
                };
                let param_span = span.merge(self.previous_span());
                params.kwarg = Some(Box::new(Param {
                    name,
                    kind: ParamKind::KwArg,
                    annotation,
                    default: None,
                    span: param_span,
                }));
                if self.at(&TokenKind::Comma) {
                    self.bump();
                    if !self.at(terminator) {
                        self.error_help(
                            "E-PARAMETER",
                            "tidak boleh ada parameter setelah `**kwargs`",
                            self.current().span,
                            "pindahkan `**kwargs` ke akhir daftar parameter",
                        );
                    }
                }
                continue;
            }

            let (name, name_span) = match self.expect_name("parameter") {
                Some(value) => value,
                None => break,
            };
            let annotation = if allow_annotations && self.eat(&TokenKind::Colon) {
                Some(self.parse_expression())
            } else {
                None
            };
            let default = if self.eat(&TokenKind::Eq) {
                Some(self.parse_expression())
            } else {
                None
            };
            if default.is_some() {
                seen_default = true;
            } else if seen_default && !in_kwonly {
                self.error_help(
                    "E-PARAMETER",
                    "parameter tanpa nilai bawaan tidak boleh setelah parameter bernilai bawaan",
                    name_span,
                    "beri nilai bawaan atau pindahkan parameter ke setelah `*`",
                );
            }
            let kind = if in_kwonly {
                ParamKind::KwOnly
            } else {
                ParamKind::PosOrKw
            };
            let param_span = name_span.merge(self.previous_span());
            let param = Param {
                name,
                kind,
                annotation,
                default,
                span: param_span,
            };
            if in_kwonly {
                params.kwonly.push(param);
            } else {
                params.args.push(param);
            }

            if self.at(&TokenKind::Comma) {
                self.bump();
                continue;
            }
            break;
        }

        if let Some(span) = bare_star {
            if params.kwonly.is_empty() && params.kwarg.is_none() {
                self.error_help(
                    "E-PARAMETER",
                    "`*` harus diikuti minimal satu parameter kata kunci",
                    span,
                    "tambahkan parameter setelah `*` atau hapus tanda bintang",
                );
            }
        }

        let mut seen = HashSet::new();
        for param in params.all() {
            if !seen.insert(param.name.clone()) {
                let span = param.span;
                self.error_help(
                    "E-PARAMETER",
                    format!(
                        "parameter `{}` dideklarasikan lebih dari sekali",
                        param.name
                    ),
                    span,
                    "gunakan nama parameter yang berbeda",
                );
            }
        }

        params
    }

    fn expect_contextual(&mut self, text: &str, what: &str) -> bool {
        if self.at_name_text(text) {
            self.bump();
            true
        } else {
            let span = self.current().span;
            self.error_help(
                "E-SINTAKS",
                format!("{} diharapkan, ditemukan {}", what, self.describe_current()),
                span,
                format!("tulis `{text}`"),
            );
            false
        }
    }

    fn parse_import_statement(&mut self, start: aksara_ast::Span) -> Stmt {
        self.bump();
        let mut modules = Vec::new();
        while let Some(alias) = self.parse_import_target() {
            modules.push(alias);
            if !self.eat(&TokenKind::Comma) || self.at_statement_end() {
                break;
            }
        }
        if modules.is_empty() {
            self.error_help(
                "E-SINTAKS",
                "`impor` membutuhkan minimal satu modul",
                start.merge(self.previous_span()),
                "tulis `impor nama_modul`",
            );
        }
        let span = start.merge(self.previous_span());
        self.make_stmt(span, StmtKind::Import { modules })
    }

    fn parse_import_target(&mut self) -> Option<ImportAlias> {
        if self.at_name_text("python") && matches!(self.nth(1).kind, TokenKind::Str(_)) {
            self.bump();
            let literal = self.take_string_literal("nama modul Python")?;
            let name = Parser::simple_string_value(&literal).unwrap_or_else(|| literal.raw.clone());
            if Parser::simple_string_value(&literal).is_none() {
                self.error_help(
                    "E-IMPOR",
                    "nama modul Python harus literal teks biasa",
                    self.previous_span(),
                    "hapus prefix `f` atau `b` dari nama modul",
                );
            }
            let asname = if self.eat(&TokenKind::Sebagai) {
                self.expect_name("alias impor").map(|(name, _)| name)
            } else {
                None
            };
            return Some(ImportAlias {
                name,
                asname,
                python: true,
            });
        }

        let (name, _) = self.parse_dotted_name()?;
        let asname = if self.eat(&TokenKind::Sebagai) {
            self.expect_name("alias impor").map(|(name, _)| name)
        } else {
            None
        };
        Some(ImportAlias {
            name,
            asname,
            python: false,
        })
    }

    fn parse_from_import(&mut self, start: aksara_ast::Span) -> Stmt {
        self.bump();
        let mut level = 0usize;
        let mut module = String::new();
        let mut python = false;

        if self.at_name_text("python") && matches!(self.nth(1).kind, TokenKind::Str(_)) {
            self.bump();
            let literal = match self.take_string_literal("nama modul Python") {
                Some(value) => value,
                None => StringLit {
                    prefix: String::new(),
                    quote: aksara_ast::Quote::Double,
                    raw: String::new(),
                },
            };
            if Parser::simple_string_value(&literal).is_none() {
                self.error_help(
                    "E-IMPOR",
                    "nama modul Python harus literal teks biasa",
                    self.previous_span(),
                    "hapus prefix `f` atau `b` dari nama modul",
                );
            }
            module = Parser::simple_string_value(&literal).unwrap_or(literal.raw);
            python = true;
        } else {
            while self.at(&TokenKind::Dot) || self.at(&TokenKind::Ellipsis) {
                if self.at(&TokenKind::Ellipsis) {
                    level += 3;
                } else {
                    level += 1;
                }
                self.bump();
            }
            if self.at_name() {
                if let Some((name, _)) = self.parse_dotted_name() {
                    module = name;
                }
            } else if level == 0 {
                let span = self.current().span;
                self.error_help(
                    "E-SINTAKS",
                    "nama modul diharapkan setelah `dari`",
                    span,
                    "tulis `dari nama_modul impor nama`",
                );
            }
        }

        self.expect(&TokenKind::Impor, "`impor` setelah modul");
        let mut names = Vec::new();

        if self.eat(&TokenKind::Star) {
            names.push(ImportAlias {
                name: "*".to_string(),
                asname: None,
                python,
            });
        } else {
            let parenthesized = self.eat(&TokenKind::LParen);
            loop {
                if parenthesized && self.at(&TokenKind::RParen) {
                    break;
                }
                let Some((name, _)) = self.expect_name("nama yang diimpor") else {
                    break;
                };
                let asname = if self.eat(&TokenKind::Sebagai) {
                    self.expect_name("alias impor").map(|(name, _)| name)
                } else {
                    None
                };
                names.push(ImportAlias {
                    name,
                    asname,
                    python,
                });
                if self.eat(&TokenKind::Comma) {
                    if parenthesized && self.at(&TokenKind::RParen) {
                        break;
                    }
                    if self.at_statement_end() {
                        break;
                    }
                    continue;
                }
                break;
            }
            if parenthesized {
                self.expect(&TokenKind::RParen, "`)` penutup daftar impor");
            }
        }

        if names.is_empty() {
            self.error_help(
                "E-SINTAKS",
                "`dari ... impor` membutuhkan minimal satu nama",
                start.merge(self.previous_span()),
                "tambahkan nama yang diimpor atau gunakan `*`",
            );
        }
        let span = start.merge(self.previous_span());
        self.make_stmt(
            span,
            StmtKind::FromImport {
                module,
                level,
                names,
                python,
            },
        )
    }

    fn take_string_literal(&mut self, what: &str) -> Option<StringLit> {
        match &self.current().kind {
            TokenKind::Str(literal) => {
                let literal = literal.clone();
                self.bump();
                Some(literal)
            }
            _ => {
                let span = self.current().span;
                self.error_help(
                    "E-SINTAKS",
                    format!("{} diharapkan, ditemukan {}", what, self.describe_current()),
                    span,
                    "tulis literal teks di antara tanda kutip",
                );
                None
            }
        }
    }

    fn parse_match(&mut self) -> Stmt {
        let start = self.bump();
        let subject = self.parse_expression();
        self.expect(&TokenKind::Colon, "`:` setelah subjek `cocokkan`");

        if !self.at(&TokenKind::Newline) {
            let span = self.current().span;
            self.error_help(
                "E-SINTAKS",
                "`cocokkan` harus diikuti baris baru dan blok `kasus`",
                span,
                "tulis setiap `kasus` pada baris berindentasi",
            );
            return self.make_stmt(
                start.merge(self.previous_span()),
                StmtKind::Match {
                    subject,
                    cases: Vec::new(),
                },
            );
        }
        self.bump();
        if !self.at(&TokenKind::Indent) {
            let span = self.current().span;
            self.error_help(
                "E-SINTAKS",
                "blok `kasus` harus diindentasi",
                span,
                "tambahkan 4 spasi sebelum `kasus`",
            );
            return self.make_stmt(
                start.merge(self.previous_span()),
                StmtKind::Match {
                    subject,
                    cases: Vec::new(),
                },
            );
        }
        self.bump();

        let mut cases = Vec::new();
        loop {
            while self.at(&TokenKind::Newline) {
                self.bump();
            }
            if !self.at(&TokenKind::Kasus) {
                break;
            }
            let case_start = self.bump();
            let pattern = self.parse_pattern();
            let guard = if self.eat(&TokenKind::Jika) {
                Some(self.parse_expression())
            } else {
                None
            };
            self.expect(&TokenKind::Colon, "`:` setelah pola `kasus`");
            let body = self.parse_suite();
            let end = self.span_of_statements(&body, case_start);
            cases.push(MatchCase {
                pattern,
                guard,
                body,
                span: case_start.merge(end),
            });
        }

        if cases.is_empty() {
            let span = self.current().span;
            self.error_help(
                "E-SINTAKS",
                "`cocokkan` membutuhkan minimal satu `kasus`",
                span,
                "tambahkan blok `kasus`",
            );
        }
        if self.at(&TokenKind::Dedent) {
            self.bump();
        } else {
            let span = self.current().span;
            self.error_help(
                "E-SINTAKS",
                "blok `cocokkan` tidak ditutup",
                span,
                "samakan indentasi dengan awal `cocokkan`",
            );
        }
        let span = start.merge(self.previous_span());
        self.make_stmt(span, StmtKind::Match { subject, cases })
    }
}
