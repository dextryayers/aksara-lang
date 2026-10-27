use aksara_ast::{ImportAlias, Span};

use crate::interpreter::Interpreter;
use crate::nilai::{Env, Hasil, Nilai};

impl Interpreter {
    pub fn impor_alias(
        &mut self,
        alias: &ImportAlias,
        _env: &Env,
        span: Span,
    ) -> Hasil<Nilai> {
        Err(self.lempar_pesan(
            "KesalahanImpor",
            format!("impor `{}` belum tersedia", alias.name),
            span,
        ))
    }

    pub fn impor_dari(
        &mut self,
        module: &str,
        _level: usize,
        _names: &[ImportAlias],
        _python: bool,
        _env: &Env,
        span: Span,
    ) -> Hasil<()> {
        Err(self.lempar_pesan(
            "KesalahanImpor",
            format!("impor `{module}` belum tersedia"),
            span,
        ))
    }
}
