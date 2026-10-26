//! Abstract syntax tree untuk bahasa Aksara.
//!
//! Semua node menyimpan [`Span`] byte dan [`NodeId`] yang stabil dalam satu
//! hasil parse. AST tidak menyimpan pointer atau referensi ke sumber.

pub mod ast;
pub mod span;
pub mod visit;

pub use ast::*;
pub use span::{NodeId, Span};
pub use visit::{walk_expr, walk_module, walk_pattern, walk_stmt, Visitor};
