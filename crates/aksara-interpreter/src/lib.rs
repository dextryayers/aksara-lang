//! Interpreter referensi Aksara.
//!
//! Interpreter ini menjadi oracle semantik untuk VM C pada Fase 3. Nilai
//! memakai `Rc<RefCell<...>>` agar grafik objek tetap sederhana; kebocoran
//! siklus diterima pada oracle dan tidak dipakai pada runtime produksi.

pub mod bawaan;
pub mod format;
pub mod interpreter;
pub mod metode;
pub mod modul;
pub mod nilai;
pub mod operasi;
pub mod pola;
#[cfg(feature = "python")]
pub mod python;

pub use interpreter::{HasilJalan, Interpreter};
pub use nilai::{Env, Hasil, Kesalahan, Lemparan, Nilai};

/// Representasi objek Python untuk pesan kesalahan dan REPL.
#[cfg(feature = "python")]
pub fn python_repr(nilai: &Nilai) -> String {
    python::repr(nilai)
}

/// Fallback ketika interpreter dibangun tanpa jembatan Python.
#[cfg(not(feature = "python"))]
pub fn python_repr(_nilai: &Nilai) -> String {
    "<objek python>".to_string()
}
