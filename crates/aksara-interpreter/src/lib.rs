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
pub mod modul_bawaan;
pub mod modul_data;
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

/// Teks objek Python dengan menghormati `__str__`.
#[cfg(feature = "python")]
pub fn python_teks(nilai: &Nilai) -> String {
    python::teks(nilai)
}

/// Fallback ketika interpreter dibangun tanpa jembatan Python.
#[cfg(not(feature = "python"))]
pub fn python_repr(_nilai: &Nilai) -> String {
    "<objek python>".to_string()
}

#[cfg(not(feature = "python"))]
pub fn python_teks(_nilai: &Nilai) -> String {
    "<objek python>".to_string()
}

#[cfg(feature = "python")]
pub use python::{
    ambil_atribut as python_ambil_atribut, ambil_indeks as python_ambil_indeks,
    atur_atribut as python_atur_atribut, atur_indeks as python_atur_indeks,
    bandingkan as python_bandingkan, hapus_atribut as python_hapus_atribut,
    hapus_indeks as python_hapus_indeks, identik as python_identik, iter as python_iter,
    kamus_pasangan as python_kamus_pasangan, kebenaran as python_kebenaran,
    keluar as python_keluar, masuk as python_masuk, mengandung as python_mengandung,
    operasi as python_operasi, panggil as python_panggil,
    panggil_metode as python_panggil_metode, panjang as python_panjang,
    sama as python_sama, tunggu as python_tunggu, hash as python_hash,
    tulis as python_tulis, impor as python_impor, semua as python_semua,
};

#[cfg(not(feature = "python"))]
mod python_tanpa {
    use aksara_ast::Span;

    use crate::nilai::{Hasil, JenisKelas, Kelas, Kesalahan, Lemparan, Nilai};

    fn galat<T>(span: Span) -> Hasil<T> {
        Err(Lemparan::baru(
            Kesalahan::dari_pesan(
                Kelas::baru_jenis("KesalahanImpor", Vec::new(), JenisKelas::Kesalahan),
                "interpreter dibangun tanpa dukungan Python",
            ),
            span,
        ))
    }

    pub fn panggil(
        _nilai: &Nilai,
        _args: Vec<Nilai>,
        _kwargs: Vec<(String, Nilai)>,
        span: Span,
    ) -> Hasil<Nilai> {
        galat(span)
    }

    pub fn masuk(_nilai: &Nilai, span: Span) -> Hasil<Nilai> {
        galat(span)
    }

    pub fn keluar(
        _nilai: &Nilai,
        _tipe: Nilai,
        _galat: Nilai,
        _jejak: Nilai,
        span: Span,
    ) -> Hasil<Nilai> {
        galat(span)
    }

    pub fn tunggu(_nilai: &Nilai, span: Span) -> Hasil<Nilai> {
        galat(span)
    }

    pub fn kebenaran(_nilai: &Nilai) -> Hasil<bool> {
        Ok(true)
    }

    pub fn sama(_a: &Nilai, _b: &Nilai) -> Hasil<bool> {
        Ok(false)
    }

    pub fn identik(_a: &Nilai, _b: &Nilai) -> Hasil<bool> {
        Ok(false)
    }

    pub fn bandingkan(_op: aksara_ast::CmpOp, _a: &Nilai, _b: &Nilai) -> Hasil<bool> {
        Ok(false)
    }

    pub fn mengandung(_wadah: &Nilai, _item: &Nilai) -> Hasil<bool> {
        Ok(false)
    }

    pub fn iter(_nilai: &Nilai) -> Hasil<Vec<Nilai>> {
        Ok(Vec::new())
    }

    pub fn panjang(_nilai: &Nilai) -> Hasil<usize> {
        Ok(0)
    }

    pub fn operasi(_op: aksara_ast::BinOp, _a: &Nilai, _b: &Nilai) -> Hasil<Nilai> {
        Ok(Nilai::Kosong)
    }

    pub fn ambil_indeks(_wadah: &Nilai, _kunci: &Nilai) -> Hasil<Nilai> {
        Ok(Nilai::Kosong)
    }

    pub fn atur_indeks(_wadah: &Nilai, _kunci: &Nilai, _nilai: Nilai) -> Hasil<()> {
        Ok(())
    }

    pub fn hapus_indeks(_wadah: &Nilai, _kunci: &Nilai) -> Hasil<()> {
        Ok(())
    }

    pub fn ambil_atribut(_objek: &Nilai, _nama: &str) -> Hasil<Nilai> {
        Ok(Nilai::Kosong)
    }

    pub fn atur_atribut(_objek: &Nilai, _nama: &str, _nilai: Nilai) -> Hasil<()> {
        Ok(())
    }

    pub fn hapus_atribut(_objek: &Nilai, _nama: &str) -> Hasil<()> {
        Ok(())
    }

    pub fn kamus_pasangan(_nilai: &Nilai) -> Hasil<Vec<(Nilai, Nilai)>> {
        Ok(Vec::new())
    }

    pub fn panggil_metode(
        _nilai: &Nilai,
        _nama: &str,
        _args: Vec<Nilai>,
        _kwargs: Vec<(String, Nilai)>,
        span: Span,
    ) -> Hasil<Nilai> {
        galat(span)
    }

    pub fn hash(_nilai: &Nilai) -> Hasil<i64> {
        Ok(0)
    }

    pub fn tulis(_nilai: &Nilai, _teks: &str) -> Hasil<Nilai> {
        Ok(Nilai::Kosong)
    }

    pub fn impor(_nama: &str, span: Span) -> Hasil<Nilai> {
        galat(span)
    }

    pub fn semua(_nilai: &Nilai, span: Span) -> Hasil<Vec<String>> {
        galat(span)
    }
}

#[cfg(not(feature = "python"))]
pub use python_tanpa::{
    ambil_atribut as python_ambil_atribut, ambil_indeks as python_ambil_indeks,
    atur_atribut as python_atur_atribut, atur_indeks as python_atur_indeks,
    bandingkan as python_bandingkan, hapus_atribut as python_hapus_atribut,
    hapus_indeks as python_hapus_indeks, identik as python_identik, iter as python_iter,
    kamus_pasangan as python_kamus_pasangan, kebenaran as python_kebenaran,
    keluar as python_keluar, masuk as python_masuk, mengandung as python_mengandung,
    operasi as python_operasi, panggil as python_panggil,
    panggil_metode as python_panggil_metode, panjang as python_panjang,
    sama as python_sama, tunggu as python_tunggu, hash as python_hash,
    tulis as python_tulis, impor as python_impor, semua as python_semua,
};
