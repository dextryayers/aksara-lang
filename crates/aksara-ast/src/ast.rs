use serde::{Deserialize, Serialize};

use crate::span::{NodeId, Span};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Quote {
    Single,
    Double,
    TripleSingle,
    TripleDouble,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct StringLit {
    pub prefix: String,
    pub quote: Quote,
    pub raw: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Module {
    pub id: NodeId,
    pub span: Span,
    pub body: Vec<Stmt>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Stmt {
    pub id: NodeId,
    pub span: Span,
    pub kind: StmtKind,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum StmtKind {
    Expr(Expr),
    Assign {
        targets: Vec<Expr>,
        value: Expr,
    },
    AnnAssign {
        target: Expr,
        annotation: Expr,
        value: Option<Expr>,
    },
    AugAssign {
        target: Expr,
        op: AugOp,
        value: Expr,
    },
    If {
        condition: Expr,
        body: Vec<Stmt>,
        orelse: Vec<Stmt>,
    },
    While {
        condition: Expr,
        body: Vec<Stmt>,
        orelse: Vec<Stmt>,
    },
    For {
        target: Expr,
        iter: ForIter,
        body: Vec<Stmt>,
        orelse: Vec<Stmt>,
        is_async: bool,
    },
    Function(FunctionDef),
    Class(ClassDef),
    Return(Option<Expr>),
    Yield {
        value: Option<Expr>,
        from: bool,
    },
    Break,
    Continue,
    Pass,
    Try {
        body: Vec<Stmt>,
        handlers: Vec<ExceptHandler>,
        orelse: Vec<Stmt>,
        finalbody: Vec<Stmt>,
    },
    Raise {
        exc: Option<Expr>,
        cause: Option<Expr>,
    },
    With {
        items: Vec<WithItem>,
        body: Vec<Stmt>,
        is_async: bool,
    },
    Import {
        modules: Vec<ImportAlias>,
    },
    FromImport {
        module: String,
        level: usize,
        names: Vec<ImportAlias>,
        python: bool,
    },
    Global(Vec<String>),
    Nonlocal(Vec<String>),
    Assert {
        test: Expr,
        msg: Option<Expr>,
    },
    Delete(Vec<Expr>),
    Match {
        subject: Expr,
        cases: Vec<MatchCase>,
    },
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct FunctionDef {
    pub name: String,
    pub params: Parameters,
    pub returns: Option<Expr>,
    pub body: Vec<Stmt>,
    pub decorators: Vec<Expr>,
    pub is_async: bool,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ClassDef {
    pub name: String,
    pub bases: Vec<Expr>,
    pub keywords: Vec<Arg>,
    pub body: Vec<Stmt>,
    pub decorators: Vec<Expr>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Parameters {
    pub posonly: Vec<Param>,
    pub args: Vec<Param>,
    pub vararg: Option<Box<Param>>,
    pub kwonly: Vec<Param>,
    pub kwarg: Option<Box<Param>>,
}

impl Parameters {
    pub fn empty() -> Self {
        Self {
            posonly: Vec::new(),
            args: Vec::new(),
            vararg: None,
            kwonly: Vec::new(),
            kwarg: None,
        }
    }

    pub fn all(&self) -> impl Iterator<Item = &Param> {
        self.posonly
            .iter()
            .chain(self.args.iter())
            .chain(self.vararg.iter().map(|p| p.as_ref()))
            .chain(self.kwonly.iter())
            .chain(self.kwarg.iter().map(|p| p.as_ref()))
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Param {
    pub name: String,
    pub kind: ParamKind,
    pub annotation: Option<Expr>,
    pub default: Option<Expr>,
    pub span: Span,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ParamKind {
    PosOnly,
    PosOrKw,
    VarArg,
    KwOnly,
    KwArg,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum ForIter {
    In(Expr),
    Each(Expr),
    Times(Expr),
    Range {
        start: Expr,
        end: Expr,
        step: Option<Expr>,
    },
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ExceptHandler {
    pub type_: Option<Expr>,
    pub name: Option<String>,
    pub body: Vec<Stmt>,
    pub span: Span,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct WithItem {
    pub context: Expr,
    pub target: Option<Expr>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ImportAlias {
    pub name: String,
    pub asname: Option<String>,
    /// Benar bila alias menunjuk modul Python melalui `impor python "..."`.
    pub python: bool,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct MatchCase {
    pub pattern: Pattern,
    pub guard: Option<Expr>,
    pub body: Vec<Stmt>,
    pub span: Span,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Expr {
    pub id: NodeId,
    pub span: Span,
    pub kind: ExprKind,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum ExprKind {
    Name(String),
    Int(String),
    Float(String),
    Imaginary(String),
    Str(StringLit),
    Bytes(StringLit),
    Bool(bool),
    None,
    Ellipsis,
    List(Vec<Expr>),
    Tuple(Vec<Expr>),
    Set(Vec<Expr>),
    Dict(Vec<DictItem>),
    Attribute {
        value: Box<Expr>,
        attr: String,
        safe: bool,
    },
    Subscript {
        value: Box<Expr>,
        index: Box<Expr>,
        safe: bool,
    },
    Slice {
        start: Option<Box<Expr>>,
        stop: Option<Box<Expr>>,
        step: Option<Box<Expr>>,
    },
    Call {
        func: Box<Expr>,
        args: Vec<Arg>,
        safe: bool,
    },
    Unary {
        op: UnaryOp,
        operand: Box<Expr>,
    },
    Binary {
        left: Box<Expr>,
        op: BinOp,
        right: Box<Expr>,
    },
    BoolOp {
        op: BoolOp,
        values: Vec<Expr>,
    },
    Compare {
        left: Box<Expr>,
        ops: Vec<CmpOp>,
        comparators: Vec<Expr>,
    },
    Between {
        value: Box<Expr>,
        low: Box<Expr>,
        high: Box<Expr>,
        negated: bool,
    },
    IfExpr {
        body: Box<Expr>,
        condition: Box<Expr>,
        orelse: Box<Expr>,
    },
    Lambda {
        params: Parameters,
        body: Box<Expr>,
    },
    NamedExpr {
        target: Box<Expr>,
        value: Box<Expr>,
    },
    Await(Box<Expr>),
    Yield {
        value: Option<Box<Expr>>,
        from: bool,
    },
    FString(Vec<FStringPart>),
    /// Gabungan literal teks yang bersebelahan, misalnya `"a" f"{b}"`.
    StringConcat(Vec<Expr>),
    Starred(Box<Expr>),
    ListComp {
        elt: Box<Expr>,
        clauses: Vec<CompClause>,
    },
    SetComp {
        elt: Box<Expr>,
        clauses: Vec<CompClause>,
    },
    DictComp {
        key: Box<Expr>,
        value: Box<Expr>,
        clauses: Vec<CompClause>,
    },
    GeneratorExp {
        elt: Box<Expr>,
        clauses: Vec<CompClause>,
    },
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum DictItem {
    Pair(Expr, Expr),
    Unpack(Expr),
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum Arg {
    Positional(Expr),
    Keyword { name: String, value: Expr },
    Star(Expr),
    KwStar(Expr),
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum FStringPart {
    Text(String),
    Expr {
        value: Box<Expr>,
        conversion: Option<char>,
        format_spec: Option<Vec<FStringPart>>,
    },
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum CompClause {
    For {
        target: Expr,
        iter: Expr,
        is_async: bool,
    },
    If(Expr),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum UnaryOp {
    Pos,
    Neg,
    Not,
    Invert,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum BinOp {
    Add,
    Sub,
    Mul,
    Div,
    FloorDiv,
    Mod,
    Pow,
    MatMul,
    LShift,
    RShift,
    BitAnd,
    BitOr,
    BitXor,
    NullCoalesce,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum BoolOp {
    And,
    Or,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum CmpOp {
    Eq,
    NotEq,
    Lt,
    Le,
    Gt,
    Ge,
    In,
    NotIn,
    Is,
    IsNot,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum AugOp {
    Add,
    Sub,
    Mul,
    Div,
    FloorDiv,
    Mod,
    Pow,
    MatMul,
    LShift,
    RShift,
    BitAnd,
    BitOr,
    BitXor,
    NullCoalesce,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum Pattern {
    Literal(Expr),
    Wildcard,
    Capture(String),
    /// Pola bintang pada urutan, misalnya `[kepala, *ekor]`.
    Star(Option<String>),
    Sequence(Vec<Pattern>),
    Mapping {
        items: Vec<(Expr, Pattern)>,
        rest: Option<String>,
    },
    Class {
        name: String,
        positional: Vec<Pattern>,
        keywords: Vec<(String, Pattern)>,
    },
    Or(Vec<Pattern>),
}
