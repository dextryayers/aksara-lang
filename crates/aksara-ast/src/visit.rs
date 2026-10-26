use crate::ast::*;

/// Visitor AST dengan implementasi default yang menelusuri seluruh anak.
///
/// Implementasi dapat mengganti `visit_expr` atau `visit_stmt` untuk berhenti
/// pada node tertentu, lalu memanggil fungsi `walk_*` bila ingin melanjutkan.
pub trait Visitor {
    fn visit_module(&mut self, module: &Module) {
        walk_module(self, module);
    }

    fn visit_stmt(&mut self, stmt: &Stmt) {
        walk_stmt(self, stmt);
    }

    fn visit_expr(&mut self, expr: &Expr) {
        walk_expr(self, expr);
    }

    fn visit_pattern(&mut self, pattern: &Pattern) {
        walk_pattern(self, pattern);
    }
}

pub fn walk_module<V: Visitor + ?Sized>(visitor: &mut V, module: &Module) {
    for stmt in &module.body {
        visitor.visit_stmt(stmt);
    }
}

pub fn walk_stmt<V: Visitor + ?Sized>(visitor: &mut V, stmt: &Stmt) {
    match &stmt.kind {
        StmtKind::Expr(expr) => visitor.visit_expr(expr),
        StmtKind::Assign { targets, value } => {
            for target in targets {
                visitor.visit_expr(target);
            }
            visitor.visit_expr(value);
        }
        StmtKind::AnnAssign {
            target,
            annotation,
            value,
        } => {
            visitor.visit_expr(target);
            visitor.visit_expr(annotation);
            if let Some(value) = value {
                visitor.visit_expr(value);
            }
        }
        StmtKind::AugAssign { target, value, .. } => {
            visitor.visit_expr(target);
            visitor.visit_expr(value);
        }
        StmtKind::If {
            condition,
            body,
            orelse,
        } => {
            visitor.visit_expr(condition);
            for stmt in body {
                visitor.visit_stmt(stmt);
            }
            for stmt in orelse {
                visitor.visit_stmt(stmt);
            }
        }
        StmtKind::While {
            condition,
            body,
            orelse,
        } => {
            visitor.visit_expr(condition);
            for stmt in body {
                visitor.visit_stmt(stmt);
            }
            for stmt in orelse {
                visitor.visit_stmt(stmt);
            }
        }
        StmtKind::For {
            target,
            iter,
            body,
            orelse,
            ..
        } => {
            visitor.visit_expr(target);
            match iter {
                ForIter::In(expr) | ForIter::Each(expr) | ForIter::Times(expr) => {
                    visitor.visit_expr(expr)
                }
                ForIter::Range { start, end, step } => {
                    visitor.visit_expr(start);
                    visitor.visit_expr(end);
                    if let Some(step) = step {
                        visitor.visit_expr(step);
                    }
                }
            }
            for stmt in body {
                visitor.visit_stmt(stmt);
            }
            for stmt in orelse {
                visitor.visit_stmt(stmt);
            }
        }
        StmtKind::Function(def) => {
            for decorator in &def.decorators {
                visitor.visit_expr(decorator);
            }
            walk_parameters(visitor, &def.params);
            if let Some(returns) = &def.returns {
                visitor.visit_expr(returns);
            }
            for stmt in &def.body {
                visitor.visit_stmt(stmt);
            }
        }
        StmtKind::Class(def) => {
            for decorator in &def.decorators {
                visitor.visit_expr(decorator);
            }
            for base in &def.bases {
                visitor.visit_expr(base);
            }
            for keyword in &def.keywords {
                match keyword {
                    Arg::Keyword { value, .. }
                    | Arg::Star(value)
                    | Arg::KwStar(value) => visitor.visit_expr(value),
                    Arg::Positional(value) => visitor.visit_expr(value),
                }
            }
            for stmt in &def.body {
                visitor.visit_stmt(stmt);
            }
        }
        StmtKind::Return(value) => {
            if let Some(value) = value {
                visitor.visit_expr(value);
            }
        }
        StmtKind::Yield { value, .. } => {
            if let Some(value) = value {
                visitor.visit_expr(value);
            }
        }
        StmtKind::Break | StmtKind::Continue | StmtKind::Pass => {}
        StmtKind::Try {
            body,
            handlers,
            orelse,
            finalbody,
        } => {
            for stmt in body {
                visitor.visit_stmt(stmt);
            }
            for handler in handlers {
                if let Some(type_) = &handler.type_ {
                    visitor.visit_expr(type_);
                }
                for stmt in &handler.body {
                    visitor.visit_stmt(stmt);
                }
            }
            for stmt in orelse {
                visitor.visit_stmt(stmt);
            }
            for stmt in finalbody {
                visitor.visit_stmt(stmt);
            }
        }
        StmtKind::Raise { exc, cause } => {
            if let Some(exc) = exc {
                visitor.visit_expr(exc);
            }
            if let Some(cause) = cause {
                visitor.visit_expr(cause);
            }
        }
        StmtKind::With { items, body, .. } => {
            for item in items {
                visitor.visit_expr(&item.context);
                if let Some(target) = &item.target {
                    visitor.visit_expr(target);
                }
            }
            for stmt in body {
                visitor.visit_stmt(stmt);
            }
        }
        StmtKind::Import { .. }
        | StmtKind::FromImport { .. }
        | StmtKind::Global(_)
        | StmtKind::Nonlocal(_) => {}
        StmtKind::Assert { test, msg } => {
            visitor.visit_expr(test);
            if let Some(msg) = msg {
                visitor.visit_expr(msg);
            }
        }
        StmtKind::Delete(targets) => {
            for target in targets {
                visitor.visit_expr(target);
            }
        }
        StmtKind::Match { subject, cases } => {
            visitor.visit_expr(subject);
            for case in cases {
                visitor.visit_pattern(&case.pattern);
                if let Some(guard) = &case.guard {
                    visitor.visit_expr(guard);
                }
                for stmt in &case.body {
                    visitor.visit_stmt(stmt);
                }
            }
        }
    }
}

pub fn walk_parameters<V: Visitor + ?Sized>(visitor: &mut V, params: &Parameters) {
    for param in params.all() {
        if let Some(annotation) = &param.annotation {
            visitor.visit_expr(annotation);
        }
        if let Some(default) = &param.default {
            visitor.visit_expr(default);
        }
    }
}

pub fn walk_expr<V: Visitor + ?Sized>(visitor: &mut V, expr: &Expr) {
    match &expr.kind {
        ExprKind::Name(_)
        | ExprKind::Int(_)
        | ExprKind::Float(_)
        | ExprKind::Imaginary(_)
        | ExprKind::Str(_)
        | ExprKind::Bytes(_)
        | ExprKind::Bool(_)
        | ExprKind::None
        | ExprKind::Ellipsis => {}
        ExprKind::List(items) | ExprKind::Tuple(items) | ExprKind::Set(items) => {
            for item in items {
                visitor.visit_expr(item);
            }
        }
        ExprKind::Dict(items) => {
            for item in items {
                match item {
                    DictItem::Pair(key, value) => {
                        visitor.visit_expr(key);
                        visitor.visit_expr(value);
                    }
                    DictItem::Unpack(value) => visitor.visit_expr(value),
                }
            }
        }
        ExprKind::Attribute { value, .. } => visitor.visit_expr(value),
        ExprKind::Subscript { value, index, .. } => {
            visitor.visit_expr(value);
            visitor.visit_expr(index);
        }
        ExprKind::Slice { start, stop, step } => {
            for part in [start, stop, step].into_iter().flatten() {
                visitor.visit_expr(part);
            }
        }
        ExprKind::Call { func, args, .. } => {
            visitor.visit_expr(func);
            for arg in args {
                match arg {
                    Arg::Positional(value)
                    | Arg::Keyword { value, .. }
                    | Arg::Star(value)
                    | Arg::KwStar(value) => visitor.visit_expr(value),
                }
            }
        }
        ExprKind::Unary { operand, .. } => visitor.visit_expr(operand),
        ExprKind::Binary { left, right, .. } => {
            visitor.visit_expr(left);
            visitor.visit_expr(right);
        }
        ExprKind::BoolOp { values, .. } => {
            for value in values {
                visitor.visit_expr(value);
            }
        }
        ExprKind::Compare {
            left, comparators, ..
        } => {
            visitor.visit_expr(left);
            for comparator in comparators {
                visitor.visit_expr(comparator);
            }
        }
        ExprKind::Between {
            value, low, high, ..
        } => {
            visitor.visit_expr(value);
            visitor.visit_expr(low);
            visitor.visit_expr(high);
        }
        ExprKind::IfExpr {
            body,
            condition,
            orelse,
        } => {
            visitor.visit_expr(body);
            visitor.visit_expr(condition);
            visitor.visit_expr(orelse);
        }
        ExprKind::Lambda { params, body } => {
            walk_parameters(visitor, params);
            visitor.visit_expr(body);
        }
        ExprKind::NamedExpr { target, value } => {
            visitor.visit_expr(target);
            visitor.visit_expr(value);
        }
        ExprKind::Await(value) => visitor.visit_expr(value),
        ExprKind::Yield { value, .. } => {
            if let Some(value) = value {
                visitor.visit_expr(value);
            }
        }
        ExprKind::FString(parts) => {
            for part in parts {
                if let FStringPart::Expr {
                    value, format_spec, ..
                } = part
                {
                    visitor.visit_expr(value);
                    if let Some(spec) = format_spec {
                        for part in spec {
                            if let FStringPart::Expr { value, .. } = part {
                                visitor.visit_expr(value);
                            }
                        }
                    }
                }
            }
        }
        ExprKind::StringConcat(parts) => {
            for part in parts {
                visitor.visit_expr(part);
            }
        }
        ExprKind::Starred(value) => visitor.visit_expr(value),
        ExprKind::ListComp { elt, clauses }
        | ExprKind::SetComp { elt, clauses }
        | ExprKind::GeneratorExp { elt, clauses } => {
            visitor.visit_expr(elt);
            walk_clauses(visitor, clauses);
        }
        ExprKind::DictComp {
            key,
            value,
            clauses,
        } => {
            visitor.visit_expr(key);
            visitor.visit_expr(value);
            walk_clauses(visitor, clauses);
        }
    }
}

fn walk_clauses<V: Visitor + ?Sized>(visitor: &mut V, clauses: &[CompClause]) {
    for clause in clauses {
        match clause {
            CompClause::For { target, iter, .. } => {
                visitor.visit_expr(target);
                visitor.visit_expr(iter);
            }
            CompClause::If(expr) => visitor.visit_expr(expr),
        }
    }
}

pub fn walk_pattern<V: Visitor + ?Sized>(visitor: &mut V, pattern: &Pattern) {
    match pattern {
        Pattern::Literal(expr) => visitor.visit_expr(expr),
        Pattern::Wildcard | Pattern::Capture(_) | Pattern::Star(_) => {}
        Pattern::Sequence(patterns) => {
            for pattern in patterns {
                visitor.visit_pattern(pattern);
            }
        }
        Pattern::Mapping { items, .. } => {
            for (key, value) in items {
                visitor.visit_expr(key);
                visitor.visit_pattern(value);
            }
        }
        Pattern::Class {
            positional,
            keywords,
            ..
        } => {
            for pattern in positional {
                visitor.visit_pattern(pattern);
            }
            for (_, pattern) in keywords {
                visitor.visit_pattern(pattern);
            }
        }
        Pattern::Or(patterns) => {
            for pattern in patterns {
                visitor.visit_pattern(pattern);
            }
        }
    }
}
