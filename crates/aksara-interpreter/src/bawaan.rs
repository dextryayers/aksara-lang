use crate::interpreter::Interpreter;
use crate::nilai::{Env, Hasil, Nilai};

/// Memasang seluruh nama bawaan pada lingkungan bawaan.
pub fn pasang(_interpreter: &mut Interpreter, _env: &Env) {
    // Diisi pada tahap berikutnya.
}

pub fn tidak_dikenal<T>(_interpreter: &mut Interpreter, nama: &str) -> Hasil<T> {
    let _ = nama;
    unimplemented!()
}

pub fn bawaan(_nama: &str, _fungsi: crate::nilai::BawaanFn) -> Nilai {
    unimplemented!()
}
