//! Jembatan CPython untuk Aksara.
//!
//! Jembatan ini memakai PyO3 dan registri thread-local. Nilai Python asli
//! disimpan sebagai [`Nilai::Python`] berisi indeks registri. Konversi
//! primitif dan koleksi dilakukan secara dalam (deep) agar operator Aksara
//! tetap bekerja; objek kompleks dipertahankan sebagai pegangan Python asli
//! sehingga identitas, metode, dan pustaka pihak ketiga tetap utuh.
//!
//! Registri sengaja tidak membebaskan objek selama proses hidup. Interpreter
//! referensi memang memakai `Rc` dan menerima kebocoran siklus; runtime C pada
//! Fase 3 akan memakai penghitungan referensi dan GC yang benar. Untuk sesi
//! REPL panjang, jumlah objek Python unik tetap menjadi batas atas memori.

use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::rc::Rc;

use aksara_ast::{BinOp, CmpOp, Span};
use pyo3::exceptions::{
    PyAssertionError, PyAttributeError, PyImportError, PyIndexError, PyKeyError,
    PyKeyboardInterrupt, PyMemoryError, PyNotImplementedError, PyOSError, PyOverflowError,
    PyRecursionError, PyRuntimeError, PyStopIteration, PySystemExit, PyTypeError, PyValueError,
    PyZeroDivisionError,
};
use pyo3::prelude::*;
use pyo3::pyclass::CompareOp;
use pyo3::types::{
    PyBool, PyByteArray, PyBytes, PyCFunction, PyComplex, PyDict, PyFloat, PyFrozenSet, PyInt,
    PyList, PySet, PyString, PyTuple,
};

use crate::interpreter::Interpreter;
use crate::nilai::{Hasil, JenisKelas, Kelas, Kesalahan, Lemparan, Nilai};

// ------------------------------------------------------------------
// Registri thread-local
// ------------------------------------------------------------------

thread_local! {
    static REGISTRI: RefCell<Vec<Py<PyAny>>> = const { RefCell::new(Vec::new()) };
    static PETA: RefCell<HashMap<usize, usize>> = RefCell::new(HashMap::new());
    static CALLBACK: RefCell<Vec<Nilai>> = const { RefCell::new(Vec::new()) };
    static AKTIF: Cell<*mut Interpreter> = const { Cell::new(std::ptr::null_mut()) };
    static KEDALAMAN: Cell<usize> = const { Cell::new(0) };
    static KELAS_AKTIF: RefCell<HashMap<String, Rc<Kelas>>> = RefCell::new(HashMap::new());
}

/// Penjaga interpreter aktif selama eksekusi Aksara.
///
/// Python dapat memanggil kembali fungsi Aksara (misalnya lewat
/// `functools.reduce`). Penjaga ini memberi jembatan akses ke interpreter
/// yang sedang menjalankan kode. Penjaga bersarang hanya menaikkan kedalaman;
/// penjaga terluar yang memasang dan membersihkan keadaan.
pub struct Aktif {
    _pribadi: (),
}

impl Aktif {
    pub fn baru(interpreter: &mut Interpreter) -> Self {
        let kedalaman = KEDALAMAN.with(Cell::get);
        if kedalaman == 0 {
            AKTIF.with(|sel| sel.set(interpreter as *mut Interpreter));
            KELAS_AKTIF.with(|kelas| *kelas.borrow_mut() = interpreter.kelas_bawaan.clone());
        }
        KEDALAMAN.with(|sel| sel.set(kedalaman + 1));
        Self { _pribadi: () }
    }
}

impl Drop for Aktif {
    fn drop(&mut self) {
        let kedalaman = KEDALAMAN.with(Cell::get).saturating_sub(1);
        KEDALAMAN.with(|sel| sel.set(kedalaman));
        if kedalaman == 0 {
            AKTIF.with(|sel| sel.set(std::ptr::null_mut()));
            KELAS_AKTIF.with(|kelas| kelas.borrow_mut().clear());
        }
    }
}

fn dengan_interpreter<R>(f: impl FnOnce(&mut Interpreter) -> R) -> Option<R> {
    let ptr = AKTIF.with(Cell::get);
    if ptr.is_null() {
        return None;
    }
    // Keselamatan: penjaga `Aktif` hanya dipasang selama interpreter hidup dan
    // tidak dipindahkan. Pemanggilan balik Python terjadi ketika bingkai luar
    // sedang menunggu, sehingga tidak ada akses bersamaan ke interpreter.
    let interpreter = unsafe { &mut *ptr };
    Some(f(interpreter))
}

fn daftarkan(obj: &Bound<'_, PyAny>) -> usize {
    let alamat = obj.as_ptr() as usize;
    if let Some(indeks) = PETA.with(|peta| peta.borrow().get(&alamat).copied()) {
        return indeks;
    }
    let indeks = REGISTRI.with(|registri| {
        let mut registri = registri.borrow_mut();
        registri.push(obj.clone().unbind());
        registri.len() - 1
    });
    PETA.with(|peta| peta.borrow_mut().insert(alamat, indeks));
    indeks
}

fn ambil(py: Python<'_>, indeks: usize) -> Option<Py<PyAny>> {
    REGISTRI.with(|registri| registri.borrow().get(indeks).map(|obj| obj.clone_ref(py)))
}

fn objek_python<'py>(py: Python<'py>, indeks: usize) -> PyResult<Bound<'py, PyAny>> {
    ambil(py, indeks)
        .map(|obj| obj.into_bound(py))
        .ok_or_else(|| PyRuntimeError::new_err("pegangan objek Python tidak valid"))
}

// ------------------------------------------------------------------
// Konversi Aksara ke Python
// ------------------------------------------------------------------

fn ke_python<'py>(py: Python<'py>, nilai: &Nilai) -> PyResult<Bound<'py, PyAny>> {
    match nilai {
        Nilai::Kosong => Ok(py.None().into_bound(py)),
        Nilai::Logika(nilai) => Ok(PyBool::new(py, *nilai).to_owned().into_any()),
        Nilai::Bilangan(nilai) => Ok(nilai.into_pyobject(py)?.into_any()),
        Nilai::Pecahan(nilai) => Ok(PyFloat::new(py, *nilai).into_any()),
        Nilai::Kompleks(nyata, imajiner) => {
            Ok(PyComplex::from_doubles(py, *nyata, *imajiner).into_any())
        }
        Nilai::Teks(teks) => Ok(PyString::new(py, teks).into_any()),
        Nilai::Bytes(bytes) => Ok(PyBytes::new(py, bytes).into_any()),
        Nilai::Daftar(item) => {
            let mut isi = Vec::with_capacity(item.borrow().len());
            for nilai in item.borrow().iter() {
                isi.push(ke_python(py, nilai)?);
            }
            Ok(PyList::new(py, isi)?.into_any())
        }
        Nilai::Tupel(item) => {
            let mut isi = Vec::with_capacity(item.len());
            for nilai in item.iter() {
                isi.push(ke_python(py, nilai)?);
            }
            Ok(PyTuple::new(py, isi)?.into_any())
        }
        Nilai::Himpunan(item) => {
            let himpunan = PySet::empty(py)?;
            for nilai in item.borrow().iter() {
                himpunan.add(ke_python(py, nilai)?)?;
            }
            Ok(himpunan.into_any())
        }
        Nilai::Kamus(pasangan) => {
            let kamus = PyDict::new(py);
            for (kunci, nilai) in pasangan.borrow().iter() {
                kamus.set_item(ke_python(py, kunci)?, ke_python(py, nilai)?)?;
            }
            Ok(kamus.into_any())
        }
        Nilai::Rentang(rentang) => {
            let builtins = PyModule::import(py, "builtins")?;
            let range = builtins.getattr("range")?;
            Ok(range
                .call1((rentang.mulai, rentang.akhir, rentang.langkah))?
                .into_any())
        }
        Nilai::Slice(slice) => {
            let builtins = PyModule::import(py, "builtins")?;
            let slice_class = builtins.getattr("slice")?;
            let mut bagian = Vec::with_capacity(3);
            for nilai in [slice.mulai, slice.akhir, slice.langkah] {
                bagian.push(match nilai {
                    Some(nilai) => nilai.into_pyobject(py)?.into_any(),
                    None => py.None().into_bound(py),
                });
            }
            let argumen = PyTuple::new(py, bagian)?;
            Ok(slice_class.call1(argumen)?.into_any())
        }
        Nilai::Python(indeks) => objek_python(py, *indeks),
        Nilai::Ellipsis => Ok(py.Ellipsis().into_bound(py)),
        Nilai::Fungsi(_)
        | Nilai::Bawaan(_)
        | Nilai::BawaanKhusus(_)
        | Nilai::Metode(_)
        | Nilai::MetodeBawaan(_)
        | Nilai::MetodeStatis(_)
        | Nilai::MetodeKelas(_)
        | Nilai::Kelas(_)
        | Nilai::Instance(_) => bungkus_callable(py, nilai.clone()),
        lain => Err(PyTypeError::new_err(format!(
            "nilai {} belum dapat diteruskan ke Python",
            lain.nama_tipe()
        ))),
    }
}

fn bungkus_callable<'py>(py: Python<'py>, nilai: Nilai) -> PyResult<Bound<'py, PyAny>> {
    let indeks = CALLBACK.with(|daftar| {
        let mut daftar = daftar.borrow_mut();
        daftar.push(nilai);
        daftar.len() - 1
    });
    let fungsi = PyCFunction::new_closure(
        py,
        None,
        None,
        move |args: &Bound<'_, PyTuple>, kwargs: Option<&Bound<'_, PyDict>>| {
            panggil_balik(indeks, args, kwargs)
        },
    )?;
    Ok(fungsi.into_any())
}

fn panggil_balik(
    indeks: usize,
    args: &Bound<'_, PyTuple>,
    kwargs: Option<&Bound<'_, PyDict>>,
) -> PyResult<Py<PyAny>> {
    let py = args.py();
    let fungsi = CALLBACK
        .with(|daftar| daftar.borrow().get(indeks).cloned())
        .ok_or_else(|| PyRuntimeError::new_err("fungsi Aksara pada registri tidak ditemukan"))?;
    let mut argumen = Vec::with_capacity(args.len());
    for item in args.iter() {
        argumen.push(dari_python(&item, 0));
    }
    let mut kata_kunci = Vec::new();
    if let Some(kwargs) = kwargs {
        for (nama, nilai) in kwargs.iter() {
            let nama = nama
                .extract::<String>()
                .map_err(|_| PyTypeError::new_err("nama argumen kata kunci harus teks"))?;
            kata_kunci.push((nama, dari_python(&nilai, 0)));
        }
    }
    let hasil = dengan_interpreter(|interpreter| {
        interpreter.panggil(&fungsi, argumen, kata_kunci, Span::empty(0))
    })
    .ok_or_else(|| PyRuntimeError::new_err("tidak ada interpreter Aksara yang aktif"))?;
    match hasil {
        Ok(nilai) => Ok(ke_python(py, &nilai)?.unbind()),
        Err(lemparan) => Err(pyerr_dari_lemparan(py, &lemparan)),
    }
}

// ------------------------------------------------------------------
// Konversi Python ke Aksara
// ------------------------------------------------------------------

fn dari_python(obj: &Bound<'_, PyAny>, kedalaman: usize) -> Nilai {
    if kedalaman > 64 {
        return Nilai::Python(daftarkan(obj));
    }
    if obj.is_none() {
        return Nilai::Kosong;
    }
    if obj.is_instance_of::<PyBool>() {
        return Nilai::Logika(obj.extract::<bool>().unwrap_or(false));
    }
    if obj.is_instance_of::<PyInt>() {
        if let Ok(nilai) = obj.extract::<i64>() {
            return Nilai::Bilangan(nilai);
        }
        return Nilai::Python(daftarkan(obj));
    }
    if obj.is_instance_of::<PyFloat>() {
        if let Ok(nilai) = obj.extract::<f64>() {
            return Nilai::Pecahan(nilai);
        }
        return Nilai::Python(daftarkan(obj));
    }
    if obj.is_instance_of::<PyComplex>() {
        if let Ok(complex) = obj.downcast::<PyComplex>() {
            use pyo3::types::PyComplexMethods;
            return Nilai::Kompleks(complex.real(), complex.imag());
        }
        return Nilai::Python(daftarkan(obj));
    }
    if obj.is_instance_of::<PyString>() {
        if let Ok(teks) = obj.downcast::<PyString>() {
            return Nilai::teks(teks.to_string_lossy().into_owned());
        }
        return Nilai::Python(daftarkan(obj));
    }
    if obj.is_instance_of::<PyBytes>() {
        if let Ok(bytes) = obj.downcast::<PyBytes>() {
            return Nilai::Bytes(Rc::new(bytes.as_bytes().to_vec()));
        }
        return Nilai::Python(daftarkan(obj));
    }
    if obj.is_instance_of::<PyByteArray>() {
        if let Ok(bytes) = obj.downcast::<PyByteArray>() {
            return Nilai::Bytes(Rc::new(bytes.to_vec()));
        }
        return Nilai::Python(daftarkan(obj));
    }
    if let Ok(daftar) = obj.downcast::<PyList>() {
        let mut isi = Vec::with_capacity(daftar.len());
        for item in daftar.iter() {
            isi.push(dari_python(&item, kedalaman + 1));
        }
        return Nilai::daftar(isi);
    }
    if let Ok(tupel) = obj.downcast::<PyTuple>() {
        let mut isi = Vec::with_capacity(tupel.len());
        for item in tupel.iter() {
            isi.push(dari_python(&item, kedalaman + 1));
        }
        return Nilai::tupel(isi);
    }
    if let Ok(kamus) = obj.downcast::<PyDict>() {
        let mut isi = Vec::with_capacity(kamus.len());
        for (kunci, nilai) in kamus.iter() {
            isi.push((
                dari_python(&kunci, kedalaman + 1),
                dari_python(&nilai, kedalaman + 1),
            ));
        }
        return Nilai::kamus(isi);
    }
    if let Ok(himpunan) = obj.downcast::<PySet>() {
        let mut isi = Vec::with_capacity(himpunan.len());
        for item in himpunan.iter() {
            isi.push(dari_python(&item, kedalaman + 1));
        }
        return Nilai::himpunan(isi);
    }
    if let Ok(himpunan) = obj.downcast::<PyFrozenSet>() {
        let mut isi = Vec::with_capacity(himpunan.len());
        for item in himpunan.iter() {
            isi.push(dari_python(&item, kedalaman + 1));
        }
        return Nilai::himpunan(isi);
    }
    if obj
        .get_type()
        .name()
        .map(|nama| nama.to_string_lossy().as_ref() == "range")
        .unwrap_or(false)
    {
        let mulai = obj.getattr("start").and_then(|n| n.extract::<i64>());
        let akhir = obj.getattr("stop").and_then(|n| n.extract::<i64>());
        let langkah = obj.getattr("step").and_then(|n| n.extract::<i64>());
        if let (Ok(mulai), Ok(akhir), Ok(langkah)) = (mulai, akhir, langkah) {
            if let Some(rentang) = crate::nilai::Rentang::baru(mulai, akhir, langkah) {
                return Nilai::Rentang(rentang);
            }
        }
        return Nilai::Python(daftarkan(obj));
    }
    Nilai::Python(daftarkan(obj))
}

// ------------------------------------------------------------------
// Kesalahan Python ke kesalahan Aksara
// ------------------------------------------------------------------

fn nama_indonesia(nama: &str) -> &'static str {
    match nama {
        "TypeError" => "KesalahanTipe",
        "ValueError" => "KesalahanNilai",
        "NameError" | "UnboundLocalError" => "KesalahanNama",
        "IndexError" => "KesalahanIndeks",
        "KeyError" => "KesalahanKunci",
        "AttributeError" => "KesalahanAtribut",
        "ImportError" | "ModuleNotFoundError" => "KesalahanImpor",
        "RuntimeError" => "KesalahanRuntime",
        "ZeroDivisionError" => "KesalahanNolPembagian",
        "StopIteration" | "StopAsyncIteration" => "KesalahanHentikanIterasi",
        "RecursionError" => "KesalahanRekursi",
        "AssertionError" => "KesalahanKlaim",
        "SyntaxError" | "IndentationError" => "KesalahanSintaks",
        "OverflowError" => "KesalahanOverflow",
        "OSError" | "IOError" | "FileNotFoundError" | "PermissionError" | "IsADirectoryError"
        | "NotADirectoryError" => "KesalahanBerkas",
        "NotImplementedError" => "KesalahanBelumDiimplementasikan",
        "SystemExit" => "KesalahanKeluar",
        "KeyboardInterrupt" => "KesalahanInterupsi",
        "ArithmeticError" => "KesalahanAritmetika",
        "LookupError" => "KesalahanPencarian",
        "MemoryError" => "KesalahanMemori",
        "TimeoutError" => "KesalahanWaktu",
        "ConnectionError" => "KesalahanKoneksi",
        "UnicodeError" | "UnicodeDecodeError" | "UnicodeEncodeError" => "KesalahanUnicode",
        _ => "Kesalahan",
    }
}

fn kelas_untuk_python(nama: &str) -> Rc<Kelas> {
    let indonesia = nama_indonesia(nama);
    if let Some(kelas) =
        KELAS_AKTIF.with(|daftar| daftar.borrow().get(indonesia).cloned())
    {
        return kelas;
    }
    Kelas::baru_jenis(indonesia, Vec::new(), JenisKelas::Kesalahan)
}

fn lempar_python(galat: PyErr, span: Span) -> Lemparan {
    let (nama, pesan) = Python::with_gil(|py| {
        let nama = galat
            .get_type(py)
            .name()
            .map(|nama| nama.to_string_lossy().into_owned())
            .unwrap_or_else(|_| "Kesalahan".to_string());
        let pesan = galat
            .value(py)
            .str()
            .map(|teks| teks.to_string_lossy().into_owned())
            .unwrap_or_else(|_| galat.to_string());
        (nama, pesan)
    });
    let kelas = kelas_untuk_python(&nama);
    Lemparan::baru(Kesalahan::dari_pesan(kelas, pesan), span)
}

fn jalankan_py<T>(
    span: Span,
    f: impl for<'py> FnOnce(Python<'py>) -> PyResult<T>,
) -> Hasil<T> {
    match Python::with_gil(f) {
        Ok(nilai) => Ok(nilai),
        Err(galat) => Err(lempar_python(galat, span)),
    }
}

fn pyerr_dari_lemparan(py: Python<'_>, lemparan: &Lemparan) -> PyErr {
    let nama = lemparan.kesalahan.tipe.nama.as_str();
    let pesan = lemparan.kesalahan.pesan.clone();
    match nama {
        "KesalahanTipe" => PyTypeError::new_err(pesan),
        "KesalahanNilai" => PyValueError::new_err(pesan),
        "KesalahanIndeks" => PyIndexError::new_err(pesan),
        "KesalahanKunci" => PyKeyError::new_err(pesan),
        "KesalahanAtribut" => PyAttributeError::new_err(pesan),
        "KesalahanImpor" => PyImportError::new_err(pesan),
        "KesalahanRuntime" => PyRuntimeError::new_err(pesan),
        "KesalahanNolPembagian" => PyZeroDivisionError::new_err(pesan),
        "KesalahanHentikanIterasi" => PyStopIteration::new_err(pesan),
        "KesalahanRekursi" => PyRecursionError::new_err(pesan),
        "KesalahanKlaim" => PyAssertionError::new_err(pesan),
        "KesalahanOverflow" => PyOverflowError::new_err(pesan),
        "KesalahanBerkas" => PyOSError::new_err(pesan),
        "KesalahanBelumDiimplementasikan" => PyNotImplementedError::new_err(pesan),
        "KesalahanKeluar" => PySystemExit::new_err(pesan),
        "KesalahanInterupsi" => PyKeyboardInterrupt::new_err(pesan),
        "KesalahanMemori" => PyMemoryError::new_err(pesan),
        _ => {
            let _ = py;
            PyRuntimeError::new_err(pesan)
        }
    }
}

// ------------------------------------------------------------------
// Utilitas argumen
// ------------------------------------------------------------------

fn bikin_tuple<'py>(py: Python<'py>, args: &[Nilai]) -> PyResult<Bound<'py, PyTuple>> {
    let mut isi = Vec::with_capacity(args.len());
    for nilai in args {
        isi.push(ke_python(py, nilai)?);
    }
    PyTuple::new(py, isi)
}

fn bikin_kwargs<'py>(
    py: Python<'py>,
    kwargs: &[(String, Nilai)],
) -> PyResult<Option<Bound<'py, PyDict>>> {
    if kwargs.is_empty() {
        return Ok(None);
    }
    let kamus = PyDict::new(py);
    for (nama, nilai) in kwargs {
        kamus.set_item(nama, ke_python(py, nilai)?)?;
    }
    Ok(Some(kamus))
}

fn objek(nilai: &Nilai, span: Span, aksi: &str) -> Hasil<usize> {
    match nilai {
        Nilai::Python(indeks) => Ok(*indeks),
        lain => Err(Lemparan::baru(
            Kesalahan::dari_pesan(
                kelas_untuk_python("TypeError"),
                format!(
                    "`{aksi}` hanya berlaku untuk objek Python, ditemukan {}",
                    lain.nama_tipe()
                ),
            ),
            span,
        )),
    }
}

// ------------------------------------------------------------------
// API jembatan
// ------------------------------------------------------------------

pub fn panggil(
    nilai: &Nilai,
    args: Vec<Nilai>,
    kwargs: Vec<(String, Nilai)>,
    span: Span,
) -> Hasil<Nilai> {
    let indeks = objek(nilai, span, "panggil")?;
    jalankan_py(span, |py| {
        let fungsi = objek_python(py, indeks)?;
        let args = bikin_tuple(py, &args)?;
        let kwargs = bikin_kwargs(py, &kwargs)?;
        let hasil = fungsi.call(args, kwargs.as_ref())?;
        Ok(dari_python(&hasil, 0))
    })
}

pub fn panggil_metode(
    nilai: &Nilai,
    nama: &str,
    args: Vec<Nilai>,
    kwargs: Vec<(String, Nilai)>,
    span: Span,
) -> Hasil<Nilai> {
    let indeks = objek(nilai, span, nama)?;
    jalankan_py(span, |py| {
        let penerima = objek_python(py, indeks)?;
        let args = bikin_tuple(py, &args)?;
        let kwargs = bikin_kwargs(py, &kwargs)?;
        let hasil = penerima.call_method(nama, args, kwargs.as_ref())?;
        Ok(dari_python(&hasil, 0))
    })
}

pub fn ambil_atribut(nilai: &Nilai, nama: &str) -> Hasil<Nilai> {
    let indeks = objek(nilai, Span::empty(0), nama)?;
    jalankan_py(Span::empty(0), |py| {
        let objek = objek_python(py, indeks)?;
        let hasil = objek.getattr(nama)?;
        Ok(dari_python(&hasil, 0))
    })
}

pub fn atur_atribut(nilai: &Nilai, nama: &str, baru: Nilai) -> Hasil<()> {
    let indeks = objek(nilai, Span::empty(0), nama)?;
    jalankan_py(Span::empty(0), |py| {
        let objek = objek_python(py, indeks)?;
        objek.setattr(nama, ke_python(py, &baru)?)?;
        Ok(())
    })
}

pub fn hapus_atribut(nilai: &Nilai, nama: &str) -> Hasil<()> {
    let indeks = objek(nilai, Span::empty(0), nama)?;
    jalankan_py(Span::empty(0), |py| {
        let objek = objek_python(py, indeks)?;
        objek.delattr(nama)?;
        Ok(())
    })
}

pub fn ambil_indeks(wadah: &Nilai, kunci: &Nilai) -> Hasil<Nilai> {
    let indeks = objek(wadah, Span::empty(0), "indeks")?;
    jalankan_py(Span::empty(0), |py| {
        let objek = objek_python(py, indeks)?;
        let hasil = objek.get_item(ke_python(py, kunci)?)?;
        Ok(dari_python(&hasil, 0))
    })
}

pub fn atur_indeks(wadah: &Nilai, kunci: &Nilai, baru: Nilai) -> Hasil<()> {
    let indeks = objek(wadah, Span::empty(0), "indeks")?;
    jalankan_py(Span::empty(0), |py| {
        let objek = objek_python(py, indeks)?;
        objek.set_item(ke_python(py, kunci)?, ke_python(py, &baru)?)?;
        Ok(())
    })
}

pub fn hapus_indeks(wadah: &Nilai, kunci: &Nilai) -> Hasil<()> {
    let indeks = objek(wadah, Span::empty(0), "indeks")?;
    jalankan_py(Span::empty(0), |py| {
        let objek = objek_python(py, indeks)?;
        objek.del_item(ke_python(py, kunci)?)?;
        Ok(())
    })
}

pub fn kebenaran(nilai: &Nilai) -> Hasil<bool> {
    let indeks = objek(nilai, Span::empty(0), "kebenaran")?;
    jalankan_py(Span::empty(0), |py| {
        objek_python(py, indeks)?.is_truthy()
    })
}

pub fn sama(kiri: &Nilai, kanan: &Nilai) -> Hasil<bool> {
    jalankan_py(Span::empty(0), |py| {
        let kiri = ke_python(py, kiri)?;
        let kanan = ke_python(py, kanan)?;
        kiri.rich_compare(kanan, CompareOp::Eq)?.is_truthy()
    })
}

pub fn identik(kiri: &Nilai, kanan: &Nilai) -> Hasil<bool> {
    jalankan_py(Span::empty(0), |py| {
        let kiri = ke_python(py, kiri)?;
        let kanan = ke_python(py, kanan)?;
        Ok(kiri.is(&kanan))
    })
}

pub fn bandingkan(op: CmpOp, kiri: &Nilai, kanan: &Nilai) -> Hasil<bool> {
    jalankan_py(Span::empty(0), |py| {
        match op {
            CmpOp::In | CmpOp::NotIn => {
                let wadah = ke_python(py, kanan)?;
                let item = ke_python(py, kiri)?;
                let hasil = wadah.contains(item)?;
                Ok(if op == CmpOp::NotIn { !hasil } else { hasil })
            }
            CmpOp::Is | CmpOp::IsNot => {
                let kiri = ke_python(py, kiri)?;
                let kanan = ke_python(py, kanan)?;
                let hasil = kiri.is(&kanan);
                Ok(if op == CmpOp::IsNot { !hasil } else { hasil })
            }
            _ => {
                let kiri = ke_python(py, kiri)?;
                let kanan = ke_python(py, kanan)?;
                let op = match op {
                    CmpOp::Eq => CompareOp::Eq,
                    CmpOp::NotEq => CompareOp::Ne,
                    CmpOp::Lt => CompareOp::Lt,
                    CmpOp::Le => CompareOp::Le,
                    CmpOp::Gt => CompareOp::Gt,
                    CmpOp::Ge => CompareOp::Ge,
                    _ => unreachable!(),
                };
                kiri.rich_compare(kanan, op)?.is_truthy()
            }
        }
    })
}

pub fn mengandung(wadah: &Nilai, item: &Nilai) -> Hasil<bool> {
    jalankan_py(Span::empty(0), |py| {
        let wadah = ke_python(py, wadah)?;
        let item = ke_python(py, item)?;
        wadah.contains(item)
    })
}

pub fn iter(nilai: &Nilai) -> Hasil<Vec<Nilai>> {
    let indeks = objek(nilai, Span::empty(0), "iterasi")?;
    jalankan_py(Span::empty(0), |py| {
        let objek = objek_python(py, indeks)?;
        let mut hasil = Vec::new();
        for item in objek.try_iter()? {
            hasil.push(dari_python(&item?, 0));
        }
        Ok(hasil)
    })
}

pub fn panjang(nilai: &Nilai) -> Hasil<usize> {
    let indeks = objek(nilai, Span::empty(0), "panjang")?;
    jalankan_py(Span::empty(0), |py| {
        objek_python(py, indeks)?.len()
    })
}

pub fn operasi(op: BinOp, kiri: &Nilai, kanan: &Nilai) -> Hasil<Nilai> {
    jalankan_py(Span::empty(0), |py| {
        let kiri = ke_python(py, kiri)?;
        if op == BinOp::NullCoalesce {
            return Ok(if kiri.is_none() {
                dari_python(&ke_python(py, kanan)?, 0)
            } else {
                dari_python(&kiri, 0)
            });
        }
        let kanan = ke_python(py, kanan)?;
        let operator = PyModule::import(py, "operator")?;
        let nama = match op {
            BinOp::Add => "add",
            BinOp::Sub => "sub",
            BinOp::Mul => "mul",
            BinOp::Div => "truediv",
            BinOp::FloorDiv => "floordiv",
            BinOp::Mod => "mod",
            BinOp::Pow => "pow",
            BinOp::MatMul => "matmul",
            BinOp::LShift => "lshift",
            BinOp::RShift => "rshift",
            BinOp::BitAnd => "and_",
            BinOp::BitOr => "or_",
            BinOp::BitXor => "xor",
            BinOp::NullCoalesce => unreachable!(),
        };
        let fungsi = operator.getattr(nama)?;
        let hasil = fungsi.call1((kiri, kanan))?;
        Ok(dari_python(&hasil, 0))
    })
}

pub fn masuk(nilai: &Nilai, span: Span) -> Hasil<Nilai> {
    let indeks = objek(nilai, span, "masuk")?;
    jalankan_py(span, |py| {
        let objek = objek_python(py, indeks)?;
        let hasil = objek.call_method0("__enter__")?;
        Ok(dari_python(&hasil, 0))
    })
}

pub fn keluar(
    nilai: &Nilai,
    tipe: Nilai,
    galat: Nilai,
    jejak: Nilai,
    span: Span,
) -> Hasil<Nilai> {
    let indeks = objek(nilai, span, "keluar")?;
    jalankan_py(span, |py| {
        let objek = objek_python(py, indeks)?;
        let hasil = objek.call_method1(
            "__exit__",
            (
                ke_python(py, &tipe)?,
                ke_python(py, &galat)?,
                ke_python(py, &jejak)?,
            ),
        )?;
        Ok(Nilai::Logika(hasil.is_truthy()?))
    })
}

pub fn tunggu(nilai: &Nilai, span: Span) -> Hasil<Nilai> {
    let indeks = objek(nilai, span, "tunggu")?;
    jalankan_py(span, |py| {
        let objek = objek_python(py, indeks)?;
        let inspect = PyModule::import(py, "inspect")?;
        let awaitable = inspect.call_method1("isawaitable", (&objek,))?.is_truthy()?;
        if !awaitable {
            return Ok(dari_python(&objek, 0));
        }
        let asyncio = PyModule::import(py, "asyncio")?;
        let hasil = asyncio.call_method1("run", (&objek,))?;
        Ok(dari_python(&hasil, 0))
    })
}

pub fn hash(nilai: &Nilai) -> Hasil<i64> {
    let indeks = objek(nilai, Span::empty(0), "hash")?;
    jalankan_py(Span::empty(0), |py| {
        let hasil = objek_python(py, indeks)?.hash()?;
        Ok(hasil as i64)
    })
}

pub fn tulis(nilai: &Nilai, teks: &str) -> Hasil<Nilai> {
    let indeks = objek(nilai, Span::empty(0), "tulis")?;
    jalankan_py(Span::empty(0), |py| {
        let objek = objek_python(py, indeks)?;
        objek.call_method1("write", (teks,))?;
        Ok(Nilai::Kosong)
    })
}

pub fn impor(nama: &str, span: Span) -> Hasil<Nilai> {
    jalankan_py(span, |py| {
        let modul = PyModule::import(py, nama)?;
        Ok(Nilai::Python(daftarkan(modul.as_any())))
    })
}

pub fn semua(nilai: &Nilai, span: Span) -> Hasil<Vec<String>> {
    let indeks = objek(nilai, span, "semua")?;
    jalankan_py(span, |py| {
        let objek = objek_python(py, indeks)?;
        objek.dir()?.extract::<Vec<String>>()
    })
}

pub fn kamus_pasangan(nilai: &Nilai) -> Hasil<Vec<(Nilai, Nilai)>> {
    let indeks = objek(nilai, Span::empty(0), "kamus")?;
    jalankan_py(Span::empty(0), |py| {
        let objek = objek_python(py, indeks)?;
        if let Ok(kamus) = objek.downcast::<PyDict>() {
            let mut hasil = Vec::with_capacity(kamus.len());
            for (kunci, nilai) in kamus.iter() {
                hasil.push((dari_python(&kunci, 0), dari_python(&nilai, 0)));
            }
            return Ok(hasil);
        }
        let items = objek.call_method0("items")?;
        let mut hasil = Vec::new();
        for item in items.try_iter()? {
            let item = item?;
            let pasangan = item.extract::<(Bound<'_, PyAny>, Bound<'_, PyAny>)>()?;
            hasil.push((dari_python(&pasangan.0, 0), dari_python(&pasangan.1, 0)));
        }
        Ok(hasil)
    })
}

pub fn repr(nilai: &Nilai) -> String {
    let Nilai::Python(indeks) = nilai else {
        return format!("{nilai:?}");
    };
    Python::with_gil(|py| match ambil(py, *indeks) {
        Some(objek) => objek
            .into_bound(py)
            .repr()
            .map(|teks| teks.to_string_lossy().into_owned())
            .unwrap_or_else(|_| "<objek python>".to_string()),
        None => "<objek python>".to_string(),
    })
}

pub fn teks(nilai: &Nilai) -> String {
    let Nilai::Python(indeks) = nilai else {
        return format!("{nilai:?}");
    };
    Python::with_gil(|py| match ambil(py, *indeks) {
        Some(objek) => objek
            .into_bound(py)
            .str()
            .map(|teks| teks.to_string_lossy().into_owned())
            .unwrap_or_else(|_| "<objek python>".to_string()),
        None => "<objek python>".to_string(),
    })
}
