//! Lexer untuk bahasa Aksara.

pub mod lexer;
pub mod token;

pub use lexer::{LexResult, Lexer};
pub use token::{Token, TokenKind};

/// Melakukan lexing pada sumber yang sudah dinormalisasi.
pub fn lex(source: &str) -> LexResult {
    Lexer::new(source).tokenize()
}
