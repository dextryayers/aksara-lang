//! Parser untuk bahasa Aksara.
//!
//! Parser memakai recursive descent untuk pernyataan dan Pratt parser untuk
//! operator biner. Semua kesalahan dicatat sebagai [`Diagnostic`] dan parser
//! berusaha melanjutkan ke pernyataan berikutnya agar satu berkas dapat
//! menghasilkan beberapa pesan sekaligus.

pub mod expr;
pub mod fstring;
pub mod parser;
pub mod pattern;
pub mod stmt;

pub use parser::{parse, parse_tokens, ParseResult, Parser};
