use aksara_ast::Span;

use crate::interpreter::Interpreter;
use crate::nilai::{Hasil, Nilai};

impl Interpreter {
    pub fn panggil_metode(
        &mut self,
        penerima: &Nilai,
        nama: &str,
        _args: Vec<Nilai>,
        _kwargs: Vec<(String, Nilai)>,
        span: Span,
    ) -> Hasil<Nilai> {
        Err(self.lempar_pesan(
            "KesalahanAtribut",
            format!("`{nama}` bukan metode yang dikenal untuk {}", penerima.nama_tipe()),
            span,
        ))
    }
}
