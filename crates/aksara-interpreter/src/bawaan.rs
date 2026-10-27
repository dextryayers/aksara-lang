//! Fungsi dan kelas bawaan Aksara.
//!
//! Nama Indonesia adalah nama utama. Alias Python disediakan agar program
//! yang ditulis untuk CPython dapat dijalankan dengan perubahan minimal.

use std::collections::HashMap;
use std::fs::OpenOptions;
use std::io::{Read, Write};
use std::rc::Rc;

use aksara_ast::CmpOp;

use crate::format::{format_nilai, nilai_ke_teks, teks_kesalahan};
use crate::interpreter::Interpreter;
use crate::nilai::*;

/// Memasang seluruh nama bawaan pada lingkungan bawaan.
pub fn pasang(interpreter: &mut Interpreter, env: &Env) {
    siapkan_kelas(interpreter);
    let daftar: &[(&str, BawaanFn, &str)] = &[
        ("cetak", bawaan_cetak, "cetak(*nilai, sep=' ', end='\\n')"),
        ("print", bawaan_cetak, "cetak(*nilai, sep=' ', end='\\n')"),
        ("panjang", bawaan_panjang, "panjang(nilai) -> bilangan"),
        ("len", bawaan_panjang, "panjang(nilai) -> bilangan"),
        ("rentang", bawaan_rentang, "rentang(mulai, akhir, langkah=1)"),
        ("range", bawaan_rentang, "rentang(mulai, akhir, langkah=1)"),
        ("buka", bawaan_buka, "buka(jalur, mode='r', encoding='utf-8')"),
        ("open", bawaan_buka, "buka(jalur, mode='r', encoding='utf-8')"),
        ("daftar", bawaan_daftar, "daftar(*nilai)"),
        ("list", bawaan_daftar, "daftar(*nilai)"),
        ("tupel", bawaan_tupel, "tupel(*nilai)"),
        ("tuple", bawaan_tupel, "tupel(*nilai)"),
        ("kamus", bawaan_kamus, "kamus(**pasangan)"),
        ("dict", bawaan_kamus, "kamus(**pasangan)"),
        ("susunan", bawaan_susunan, "susunan(*nilai)"),
        ("set", bawaan_susunan, "susunan(*nilai)"),
        ("teks", bawaan_teks, "teks(nilai) -> teks"),
        ("str", bawaan_teks, "teks(nilai) -> teks"),
        ("repr", bawaan_repr, "repr(nilai) -> teks"),
        ("bilangan", bawaan_bilangan, "bilangan(nilai, basis=10) -> bilangan"),
        ("int", bawaan_bilangan, "bilangan(nilai, basis=10) -> bilangan"),
        ("pecahan", bawaan_pecahan, "pecahan(nilai) -> pecahan"),
        ("float", bawaan_pecahan, "pecahan(nilai) -> pecahan"),
        ("kompleks", bawaan_kompleks, "kompleks(nyata, imajiner=0)"),
        ("complex", bawaan_kompleks, "kompleks(nyata, imajiner=0)"),
        ("logika", bawaan_logika, "logika(nilai) -> logika"),
        ("bool", bawaan_logika, "logika(nilai) -> logika"),
        ("bytes", bawaan_bytes, "bytes(nilai) -> bytes"),
        ("jenis", bawaan_jenis, "jenis(nilai) -> kelas"),
        ("type", bawaan_jenis, "jenis(nilai) -> kelas"),
        ("abs", bawaan_abs, "abs(nilai)"),
        ("maks", bawaan_maks, "maks(*nilai, key=..., default=...)"),
        ("max", bawaan_maks, "maks(*nilai, key=..., default=...)"),
        ("min", bawaan_min, "min(*nilai, key=..., default=...)"),
        ("bulat", bawaan_bulat, "bulat(nilai, digit=None)"),
        ("round", bawaan_bulat, "bulat(nilai, digit=None)"),
        ("jumlah", bawaan_jumlah, "jumlah(iterable, awal=0)"),
        ("sum", bawaan_jumlah, "jumlah(iterable, awal=0)"),
        ("semua", bawaan_semua, "semua(iterable) -> logika"),
        ("all", bawaan_semua, "semua(iterable) -> logika"),
        ("apa_saja", bawaan_apa_saja, "apa_saja(iterable) -> logika"),
        ("any", bawaan_apa_saja, "apa_saja(iterable) -> logika"),
        ("enumerasi", bawaan_enumerasi, "enumerasi(iterable, mulai=0)"),
        ("enumerate", bawaan_enumerasi, "enumerasi(iterable, mulai=0)"),
        ("zip", bawaan_zip, "zip(*iterable, strict=False)"),
        ("peta", bawaan_peta, "peta(fungsi, *iterable)"),
        ("map", bawaan_peta, "peta(fungsi, *iterable)"),
        ("saring", bawaan_saring, "saring(fungsi, iterable)"),
        ("filter", bawaan_saring, "saring(fungsi, iterable)"),
        ("isinstance", bawaan_isinstance, "isinstance(nilai, kelas)"),
        ("issubclass", bawaan_issubclass, "issubclass(kelas, kelas_induk)"),
        ("globals", bawaan_globals, "globals() -> kamus"),
        ("locals", bawaan_locals, "locals() -> kamus"),
        ("hash", bawaan_hash, "hash(nilai) -> bilangan"),
        ("id", bawaan_id, "id(nilai) -> bilangan"),
        ("iter", bawaan_iter, "iter(nilai)"),
        ("next", bawaan_next, "next(iterator, bawaan=None)"),
        ("input", bawaan_input, "input(petunjuk='') -> teks"),
        ("format", bawaan_format, "format(nilai, spesifikasi='') -> teks"),
        ("chr", bawaan_chr, "chr(kode) -> teks"),
        ("ord", bawaan_ord, "ord(teks) -> bilangan"),
        ("hex", bawaan_hex, "hex(bilangan) -> teks"),
        ("oct", bawaan_oct, "oct(bilangan) -> teks"),
        ("bin", bawaan_bin, "bin(bilangan) -> teks"),
        ("sorted", bawaan_sorted, "sorted(iterable, key=..., reverse=False)"),
        ("reversed", bawaan_reversed, "reversed(iterable)"),
        ("divmod", bawaan_divmod, "divmod(a, b)"),
        ("pow", bawaan_pow, "pow(a, b, modulo=None)"),
        ("callable", bawaan_callable, "callable(nilai) -> logika"),
        ("getattr", bawaan_getattr, "getattr(objek, nama, bawaan=None)"),
        ("setattr", bawaan_setattr, "setattr(objek, nama, nilai)"),
        ("hasattr", bawaan_hasattr, "hasattr(objek, nama) -> logika"),
        ("delattr", bawaan_delattr, "delattr(objek, nama)"),
        ("vars", bawaan_vars, "vars(objek=None) -> kamus"),
        ("dir", bawaan_dir, "dir(objek=None) -> daftar"),
        ("help", bawaan_help, "help(objek=None)"),
        ("exit", bawaan_exit, "exit(kode=0)"),
        ("quit", bawaan_exit, "exit(kode=0)"),
        ("staticmethod", bawaan_staticmethod, "staticmethod(fungsi)"),
        ("classmethod", bawaan_classmethod, "classmethod(fungsi)"),
        ("property", bawaan_property, "property(getter, setter=None)"),
        ("induk", bawaan_induk, "induk() -> objek induk"),
        ("super", bawaan_induk, "induk() -> objek induk"),
        ("slice", bawaan_slice, "slice(mulai, akhir, langkah=None)"),
        ("repr_python", bawaan_repr, "repr(nilai)"),
        ("ascii", bawaan_repr, "ascii(nilai)"),
    ];
    for (nama, fungsi, doc) in daftar {
        env.borrow_mut().atur_lokal(
            *nama,
            Nilai::Bawaan(Rc::new(Bawaan {
                nama: (*nama).to_string(),
                fungsi: *fungsi,
                doc: Some((*doc).to_string()),
            })),
        );
    }
    for (nama, kelas) in interpreter.kelas_bawaan.clone() {
        env.borrow_mut()
            .atur_lokal(nama, Nilai::Kelas(kelas));
    }
    // Alias umum Python untuk tipe bawaan.
    let alias = [
        ("object", "objek"),
        ("int", "Bilangan"),
        ("float", "Pecahan"),
        ("complex", "Kompleks"),
        ("str", "Teks"),
        ("bool", "Logika"),
        ("list", "Daftar"),
        ("tuple", "Tupel"),
        ("dict", "Kamus"),
        ("set", "Himpunan"),
        ("range", "Rentang"),
        ("type", "Tipe"),
        ("Exception", "Kesalahan"),
        ("ValueError", "KesalahanNilai"),
        ("TypeError", "KesalahanTipe"),
        ("NameError", "KesalahanNama"),
        ("IndexError", "KesalahanIndeks"),
        ("KeyError", "KesalahanKunci"),
        ("AttributeError", "KesalahanAtribut"),
        ("ImportError", "KesalahanImpor"),
        ("RuntimeError", "KesalahanRuntime"),
        ("ZeroDivisionError", "KesalahanNolPembagian"),
        ("StopIteration", "KesalahanHentikanIterasi"),
        ("HentikanIterasi", "KesalahanHentikanIterasi"),
        ("NolPembagian", "KesalahanNolPembagian"),
        ("RecursionError", "KesalahanRekursi"),
        ("AssertionError", "KesalahanKlaim"),
        ("SyntaxError", "KesalahanSintaks"),
        ("OverflowError", "KesalahanOverflow"),
        ("OSError", "KesalahanBerkas"),
        ("FileNotFoundError", "KesalahanBerkas"),
        ("NotImplementedError", "KesalahanBelumDiimplementasikan"),
        ("SystemExit", "KesalahanKeluar"),
        ("KeyboardInterrupt", "KesalahanInterupsi"),
        ("ArithmeticError", "KesalahanAritmetika"),
        ("LookupError", "KesalahanPencarian"),
    ];
    for (python, indonesia) in alias {
        if let Some(kelas) = interpreter.kelas_bawaan.get(indonesia) {
            env.borrow_mut()
                .atur_lokal(python, Nilai::Kelas(kelas.clone()));
        }
    }
}

fn kelas_bawaan(
    interpreter: &mut Interpreter,
    nama: &str,
    basis: Vec<Rc<Kelas>>,
    jenis: JenisKelas,
) -> Rc<Kelas> {
    let kelas = Kelas::baru_jenis(nama, basis, jenis);
    kelas
        .atribut
        .borrow_mut()
        .insert("__nama__".to_string(), Nilai::teks(nama));
    interpreter
        .kelas_bawaan
        .insert(nama.to_string(), kelas.clone());
    kelas
}

fn siapkan_kelas(interpreter: &mut Interpreter) {
    if !interpreter.kelas_bawaan.is_empty() {
        return;
    }
    let objek = kelas_bawaan(interpreter, "objek", Vec::new(), JenisKelas::Pengguna);
    let tipe = kelas_bawaan(
        interpreter,
        "Tipe",
        vec![objek.clone()],
        JenisKelas::Tipe,
    );
    let kosong = kelas_bawaan(
        interpreter,
        "Kosong",
        vec![objek.clone()],
        JenisKelas::Kosong,
    );
    let bilangan = kelas_bawaan(
        interpreter,
        "Bilangan",
        vec![objek.clone()],
        JenisKelas::Bilangan,
    );
    let logika = kelas_bawaan(
        interpreter,
        "Logika",
        vec![bilangan.clone(), objek.clone()],
        JenisKelas::Logika,
    );
    let pecahan = kelas_bawaan(
        interpreter,
        "Pecahan",
        vec![objek.clone()],
        JenisKelas::Pecahan,
    );
    let kompleks = kelas_bawaan(
        interpreter,
        "Kompleks",
        vec![objek.clone()],
        JenisKelas::Kompleks,
    );
    let teks = kelas_bawaan(
        interpreter,
        "Teks",
        vec![objek.clone()],
        JenisKelas::Teks,
    );
    let bytes = kelas_bawaan(
        interpreter,
        "Bytes",
        vec![objek.clone()],
        JenisKelas::Bytes,
    );
    let daftar = kelas_bawaan(
        interpreter,
        "Daftar",
        vec![objek.clone()],
        JenisKelas::Daftar,
    );
    let tupel = kelas_bawaan(
        interpreter,
        "Tupel",
        vec![objek.clone()],
        JenisKelas::Tupel,
    );
    let himpunan = kelas_bawaan(
        interpreter,
        "Himpunan",
        vec![objek.clone()],
        JenisKelas::Himpunan,
    );
    let kamus = kelas_bawaan(
        interpreter,
        "Kamus",
        vec![objek.clone()],
        JenisKelas::Kamus,
    );
    let rentang = kelas_bawaan(
        interpreter,
        "Rentang",
        vec![objek.clone()],
        JenisKelas::Rentang,
    );
    let slice = kelas_bawaan(
        interpreter,
        "Slice",
        vec![objek.clone()],
        JenisKelas::Pengguna,
    );
    let fungsi = kelas_bawaan(
        interpreter,
        "Fungsi",
        vec![objek.clone()],
        JenisKelas::Fungsi,
    );
    let modul = kelas_bawaan(
        interpreter,
        "Modul",
        vec![objek.clone()],
        JenisKelas::Modul,
    );
    let berkas = kelas_bawaan(
        interpreter,
        "Berkas",
        vec![objek.clone()],
        JenisKelas::Berkas,
    );
    let generator = kelas_bawaan(
        interpreter,
        "Generator",
        vec![objek.clone()],
        JenisKelas::Generator,
    );
    let kesalahan = kelas_bawaan(
        interpreter,
        "Kesalahan",
        vec![objek.clone()],
        JenisKelas::Kesalahan,
    );
    {
        let mut atribut = kesalahan.atribut.borrow_mut();
        atribut.insert(
            "__init__".to_string(),
            Nilai::BawaanKhusus(Rc::new(BawaanKhusus {
                nama: "__init__".to_string(),
                fungsi: Box::new(bawaan_kesalahan_init),
            })),
        );
        atribut.insert(
            "__str__".to_string(),
            Nilai::BawaanKhusus(Rc::new(BawaanKhusus {
                nama: "__str__".to_string(),
                fungsi: Box::new(bawaan_kesalahan_str),
            })),
        );
        atribut.insert(
            "__repr__".to_string(),
            Nilai::BawaanKhusus(Rc::new(BawaanKhusus {
                nama: "__repr__".to_string(),
                fungsi: Box::new(bawaan_kesalahan_str),
            })),
        );
    }
    let _ = (tipe, kosong, logika, pecahan, kompleks, teks, bytes, daftar, tupel,
             himpunan, kamus, rentang, slice, fungsi, modul, berkas, generator);

    let turunan = [
        "KesalahanNilai",
        "KesalahanTipe",
        "KesalahanNama",
        "KesalahanIndeks",
        "KesalahanKunci",
        "KesalahanAtribut",
        "KesalahanImpor",
        "KesalahanRuntime",
        "KesalahanNolPembagian",
        "KesalahanHentikanIterasi",
        "KesalahanRekursi",
        "KesalahanKlaim",
        "KesalahanSintaks",
        "KesalahanOverflow",
        "KesalahanBerkas",
        "KesalahanBelumDiimplementasikan",
        "KesalahanKeluar",
        "KesalahanInterupsi",
        "KesalahanAritmetika",
        "KesalahanPencarian",
        "KesalahanMemori",
        "KesalahanWaktu",
        "KesalahanKoneksi",
        "KesalahanUnicode",
        "KesalahanIzin",
    ];
    for nama in turunan {
        let basis = match nama {
            "KesalahanNilai" | "KesalahanTipe" | "KesalahanNama" | "KesalahanAtribut"
            | "KesalahanImpor" | "KesalahanRuntime" | "KesalahanSintaks"
            | "KesalahanBelumDiimplementasikan" | "KesalahanMemori" | "KesalahanKoneksi"
            | "KesalahanUnicode" | "KesalahanIzin" => vec![kesalahan.clone()],
            "KesalahanIndeks" | "KesalahanKunci" => {
                vec![kesalahan.clone()]
            }
            "KesalahanNolPembagian" | "KesalahanOverflow" => vec![kesalahan.clone()],
            "KesalahanHentikanIterasi" => vec![kesalahan.clone()],
            "KesalahanRekursi" => vec![kesalahan.clone()],
            "KesalahanKlaim" => vec![kesalahan.clone()],
            "KesalahanBerkas" | "KesalahanWaktu" => vec![kesalahan.clone()],
            "KesalahanKeluar" | "KesalahanInterupsi" => vec![kesalahan.clone()],
            "KesalahanAritmetika" | "KesalahanPencarian" => vec![kesalahan.clone()],
            _ => vec![kesalahan.clone()],
        };
        kelas_bawaan(interpreter, nama, basis, JenisKelas::Kesalahan);
    }
    // Hirarki tambahan untuk isinstance.
    if let (Some(aritmetika), Some(pencarian)) = (
        interpreter.kelas_bawaan.get("KesalahanAritmetika").cloned(),
        interpreter.kelas_bawaan.get("KesalahanPencarian").cloned(),
    ) {
        for nama in ["KesalahanNolPembagian", "KesalahanOverflow"] {
            let baru = Kelas::baru_jenis(
                nama,
                vec![aritmetika.clone(), kesalahan.clone()],
                JenisKelas::Kesalahan,
            );
            interpreter.kelas_bawaan.insert(nama.to_string(), baru);
        }
        for nama in ["KesalahanIndeks", "KesalahanKunci"] {
            let baru = Kelas::baru_jenis(
                nama,
                vec![pencarian.clone(), kesalahan.clone()],
                JenisKelas::Kesalahan,
            );
            interpreter
                .kelas_bawaan
                .insert(nama.to_string(), baru);
        }
    }
}

fn galat<T>(interpreter: &mut Interpreter, pesan: impl Into<String>) -> Hasil<T> {
    Err(interpreter.lempar_pesan(
        "KesalahanTipe",
        pesan,
        interpreter.span_aktif,
    ))
}

fn galat_nilai<T>(interpreter: &mut Interpreter, pesan: impl Into<String>) -> Hasil<T> {
    Err(interpreter.lempar_pesan(
        "KesalahanNilai",
        pesan,
        interpreter.span_aktif,
    ))
}

fn arg<'a>(args: &'a [Nilai], indeks: usize) -> Option<&'a Nilai> {
    args.get(indeks)
}

fn opsi<'a>(kwargs: &'a [(String, Nilai)], nama: &str) -> Option<&'a Nilai> {
    kwargs
        .iter()
        .find(|(kunci, _)| kunci == nama)
        .map(|(_, nilai)| nilai)
}

fn teks_arg(interpreter: &mut Interpreter, nilai: &Nilai, apa: &str) -> Hasil<String> {
    match nilai {
        Nilai::Teks(teks) => Ok(teks.to_string()),
        other => galat(
            interpreter,
            format!("{apa} harus teks, ditemukan {}", other.nama_tipe()),
        ),
    }
}

fn iter_arg(interpreter: &mut Interpreter, nilai: &Nilai) -> Hasil<Vec<Nilai>> {
    interpreter.iter_nilai(nilai, interpreter.span_aktif)
}

// ------------------------------------------------------------------
// Keluaran dan masukan
// ------------------------------------------------------------------

fn bawaan_cetak(
    interpreter: &mut Interpreter,
    _env: &Env,
    args: Vec<Nilai>,
    kwargs: Vec<(String, Nilai)>,
) -> Hasil<Nilai> {
    let sep = opsi(&kwargs, "sep")
        .or_else(|| opsi(&kwargs, "pemisah"))
        .map(nilai_ke_teks)
        .unwrap_or_else(|| " ".to_string());
    let end = opsi(&kwargs, "end")
        .or_else(|| opsi(&kwargs, "akhir"))
        .map(nilai_ke_teks)
        .unwrap_or_else(|| "\n".to_string());
    let mut bagian = Vec::with_capacity(args.len());
    for nilai in &args {
        bagian.push(interpreter.teks_nilai(nilai, interpreter.span_aktif)?);
    }
    let teks = bagian.join(&sep) + &end;
    let tujuan = opsi(&kwargs, "file").or_else(|| opsi(&kwargs, "berkas"));
    match tujuan {
        Some(Nilai::Berkas(berkas)) => {
            let mut berkas = berkas.borrow_mut();
            let Some(file) = berkas.berkas.as_mut() else {
                return galat_nilai(interpreter, "berkas sudah ditutup");
            };
            file.write_all(teks.as_bytes()).map_err(|galat| {
                interpreter.lempar_pesan(
                    "KesalahanBerkas",
                    format!("gagal menulis: {galat}"),
                    interpreter.span_aktif,
                )
            })?;
        }
        Some(Nilai::Python(_)) => {
            return crate::python_tulis(tujuan.unwrap(), &teks);
        }
        Some(_) | None => interpreter.tulis(&teks),
    }
    Ok(Nilai::Kosong)
}

fn bawaan_input(
    interpreter: &mut Interpreter,
    _env: &Env,
    args: Vec<Nilai>,
    _kwargs: Vec<(String, Nilai)>,
) -> Hasil<Nilai> {
    if let Some(petunjuk) = args.first() {
        interpreter.tulis(nilai_ke_teks(petunjuk));
    }
    let mut baris = String::new();
    std::io::stdin()
        .read_line(&mut baris)
        .map_err(|galat| {
            interpreter.lempar_pesan(
                "KesalahanBerkas",
                format!("gagal membaca masukan: {galat}"),
                interpreter.span_aktif,
            )
        })?;
    if baris.ends_with('\n') {
        baris.pop();
        if baris.ends_with('\r') {
            baris.pop();
        }
    }
    Ok(Nilai::teks(baris))
}

// ------------------------------------------------------------------
// Tipe dan konversi
// ------------------------------------------------------------------

fn bawaan_panjang(
    interpreter: &mut Interpreter,
    _env: &Env,
    args: Vec<Nilai>,
    _kwargs: Vec<(String, Nilai)>,
) -> Hasil<Nilai> {
    let Some(nilai) = arg(&args, 0) else {
        return galat(interpreter, "`panjang` membutuhkan satu argumen");
    };
    let panjang = interpreter.panjang_nilai(nilai, interpreter.span_aktif)?;
    Ok(Nilai::Bilangan(panjang as i64))
}

fn bawaan_rentang(
    interpreter: &mut Interpreter,
    _env: &Env,
    args: Vec<Nilai>,
    _kwargs: Vec<(String, Nilai)>,
) -> Hasil<Nilai> {
    if args.is_empty() || args.len() > 3 {
        return galat(interpreter, "`rentang` menerima satu sampai tiga argumen");
    }
    let angka = |interpreter: &mut Interpreter, nilai: &Nilai| {
        interpreter.nilai_ke_i64(nilai, interpreter.span_aktif)
    };
    let (mulai, akhir, langkah) = match args.len() {
        1 => (0, angka(interpreter, &args[0])?, 1),
        2 => (
            angka(interpreter, &args[0])?,
            angka(interpreter, &args[1])?,
            1,
        ),
        _ => (
            angka(interpreter, &args[0])?,
            angka(interpreter, &args[1])?,
            angka(interpreter, &args[2])?,
        ),
    };
    match Rentang::baru(mulai, akhir, langkah) {
        Some(rentang) => Ok(Nilai::Rentang(rentang)),
        None => galat_nilai(interpreter, "langkah `rentang` tidak boleh nol"),
    }
}

fn bawaan_buka(
    interpreter: &mut Interpreter,
    _env: &Env,
    args: Vec<Nilai>,
    kwargs: Vec<(String, Nilai)>,
) -> Hasil<Nilai> {
    let Some(jalur) = arg(&args, 0) else {
        return galat(interpreter, "`buka` membutuhkan jalur berkas");
    };
    let jalur = teks_arg(interpreter, jalur, "jalur berkas")?;
    let mode = match arg(&args, 1) {
        Some(nilai) => teks_arg(interpreter, nilai, "mode")?,
        None => "r".to_string(),
    };
    if let Some(encoding) = opsi(&kwargs, "encoding") {
        let encoding = teks_arg(interpreter, encoding, "encoding")?;
        if !encoding.eq_ignore_ascii_case("utf-8") && !encoding.eq_ignore_ascii_case("utf8") {
            return galat_nilai(
                interpreter,
                format!("encoding `{encoding}` belum didukung; gunakan utf-8"),
            );
        }
    }
    let biner = mode.contains('b');
    let mut opsi = OpenOptions::new();
    if mode.contains('r') {
        opsi.read(true);
    }
    if mode.contains('w') {
        opsi.write(true).create(true).truncate(true);
    }
    if mode.contains('a') {
        opsi.append(true).create(true);
    }
    if mode.contains('x') {
        opsi.write(true).create_new(true);
    }
    if mode.contains('+') {
        opsi.read(true).write(true);
    }
    if !mode.contains('r') && !mode.contains('w') && !mode.contains('a') && !mode.contains('x') {
        opsi.read(true);
    }
    match opsi.open(&jalur) {
        Ok(berkas) => Ok(Nilai::Berkas(Rc::new(std::cell::RefCell::new(Berkas {
            berkas: Some(berkas),
            nama: jalur,
            mode,
            biner,
        })))),
        Err(galat) => Err(interpreter.lempar_pesan(
            "KesalahanBerkas",
            format!("tidak dapat membuka `{jalur}`: {galat}"),
            interpreter.span_aktif,
        )),
    }
}

fn bawaan_daftar(
    interpreter: &mut Interpreter,
    _env: &Env,
    args: Vec<Nilai>,
    _kwargs: Vec<(String, Nilai)>,
) -> Hasil<Nilai> {
    match args.len() {
        0 => Ok(Nilai::daftar(Vec::new())),
        1 => {
            let item = iter_arg(interpreter, &args[0])?;
            Ok(Nilai::daftar(item))
        }
        _ => Ok(Nilai::daftar(args)),
    }
}

fn bawaan_tupel(
    interpreter: &mut Interpreter,
    _env: &Env,
    args: Vec<Nilai>,
    _kwargs: Vec<(String, Nilai)>,
) -> Hasil<Nilai> {
    match args.len() {
        0 => Ok(Nilai::tupel(Vec::new())),
        1 => {
            let item = iter_arg(interpreter, &args[0])?;
            Ok(Nilai::tupel(item))
        }
        _ => Ok(Nilai::tupel(args)),
    }
}

fn bawaan_susunan(
    interpreter: &mut Interpreter,
    _env: &Env,
    args: Vec<Nilai>,
    _kwargs: Vec<(String, Nilai)>,
) -> Hasil<Nilai> {
    let mut hasil = Vec::new();
    if args.len() == 1 {
        let item = iter_arg(interpreter, &args[0])?;
        for nilai in item {
            interpreter.tambah_himpunan(&mut hasil, nilai);
        }
    } else {
        for nilai in args {
            interpreter.tambah_himpunan(&mut hasil, nilai);
        }
    }
    Ok(Nilai::himpunan(hasil))
}

fn bawaan_kamus(
    interpreter: &mut Interpreter,
    _env: &Env,
    args: Vec<Nilai>,
    kwargs: Vec<(String, Nilai)>,
) -> Hasil<Nilai> {
    let mut hasil = Vec::new();
    if let Some(nilai) = args.first() {
        for (kunci, nilai) in interpreter.kamus_pasangan(nilai, interpreter.span_aktif)? {
            interpreter.atur_kamus(&mut hasil, kunci, nilai);
        }
    }
    if args.len() > 1 {
        return galat(interpreter, "`kamus` menerima paling banyak satu argumen posisional");
    }
    for (nama, nilai) in kwargs {
        interpreter.atur_kamus(&mut hasil, Nilai::teks(nama), nilai);
    }
    Ok(Nilai::kamus(hasil))
}

fn bawaan_teks(
    interpreter: &mut Interpreter,
    _env: &Env,
    args: Vec<Nilai>,
    _kwargs: Vec<(String, Nilai)>,
) -> Hasil<Nilai> {
    match args.len() {
        0 => Ok(Nilai::teks("")),
        1 => match &args[0] {
            Nilai::Bytes(bytes) => match String::from_utf8(bytes.as_ref().clone()) {
                Ok(teks) => Ok(Nilai::teks(teks)),
                Err(_) => galat_nilai(interpreter, "bytes bukan UTF-8 valid"),
            },
            nilai => Ok(Nilai::teks(
                interpreter.teks_nilai(nilai, interpreter.span_aktif)?,
            )),
        },
        _ => galat(interpreter, "`teks` menerima paling banyak satu argumen"),
    }
}

fn bawaan_repr(
    interpreter: &mut Interpreter,
    _env: &Env,
    args: Vec<Nilai>,
    _kwargs: Vec<(String, Nilai)>,
) -> Hasil<Nilai> {
    let nilai = args.first().cloned().unwrap_or(Nilai::Kosong);
    Ok(Nilai::teks(
        interpreter.repr_nilai(&nilai, interpreter.span_aktif)?,
    ))
}

fn bawaan_bilangan(
    interpreter: &mut Interpreter,
    _env: &Env,
    args: Vec<Nilai>,
    _kwargs: Vec<(String, Nilai)>,
) -> Hasil<Nilai> {
    match args.len() {
        0 => Ok(Nilai::Bilangan(0)),
        1 => interpreter.ke_bilangan(&args[0], interpreter.span_aktif),
        2 => {
            let basis = interpreter.nilai_ke_i64(&args[1], interpreter.span_aktif)?;
            if basis != 0 && !(2..=36).contains(&basis) {
                return galat_nilai(interpreter, "basis harus 0 atau antara 2 dan 36");
            }
            let teks = teks_arg(interpreter, &args[0], "nilai")?;
            let bersih = teks.trim().replace('_', "");
            let (negatif, isi) = match bersih.strip_prefix('-') {
                Some(sisa) => (true, sisa.to_string()),
                None => (false, bersih),
            };
            let (basis_efektif, isi) = if basis == 0 {
                let (awalan, sisa) = if let Some(sisa) = isi.strip_prefix("0x") {
                    (16, sisa)
                } else if let Some(sisa) = isi.strip_prefix("0X") {
                    (16, sisa)
                } else if let Some(sisa) = isi.strip_prefix("0o") {
                    (8, sisa)
                } else if let Some(sisa) = isi.strip_prefix("0O") {
                    (8, sisa)
                } else if let Some(sisa) = isi.strip_prefix("0b") {
                    (2, sisa)
                } else if let Some(sisa) = isi.strip_prefix("0B") {
                    (2, sisa)
                } else {
                    (10, isi.as_str())
                };
                (awalan, sisa.to_string())
            } else {
                (basis, isi)
            };
            match i64::from_str_radix(&isi, basis_efektif as u32) {
                Ok(nilai) => Ok(Nilai::Bilangan(if negatif { -nilai } else { nilai })),
                Err(_) => galat_nilai(
                    interpreter,
                    format!("`{teks}` bukan bilangan basis {basis} yang valid"),
                ),
            }
        }
        _ => galat(interpreter, "`bilangan` menerima paling banyak dua argumen"),
    }
}

fn bawaan_pecahan(
    interpreter: &mut Interpreter,
    _env: &Env,
    args: Vec<Nilai>,
    _kwargs: Vec<(String, Nilai)>,
) -> Hasil<Nilai> {
    match args.len() {
        0 => Ok(Nilai::Pecahan(0.0)),
        1 => interpreter.ke_pecahan(&args[0], interpreter.span_aktif),
        _ => galat(interpreter, "`pecahan` menerima paling banyak satu argumen"),
    }
}

fn bawaan_kompleks(
    interpreter: &mut Interpreter,
    _env: &Env,
    args: Vec<Nilai>,
    _kwargs: Vec<(String, Nilai)>,
) -> Hasil<Nilai> {
    match args.len() {
        0 => Ok(Nilai::Kompleks(0.0, 0.0)),
        1 => interpreter.ke_kompleks(&args[0], interpreter.span_aktif),
        2 => {
            let nyata = interpreter.nilai_ke_f64(&args[0], interpreter.span_aktif)?;
            let imajiner = interpreter.nilai_ke_f64(&args[1], interpreter.span_aktif)?;
            Ok(Nilai::Kompleks(nyata, imajiner))
        }
        _ => galat(interpreter, "`kompleks` menerima paling banyak dua argumen"),
    }
}

fn bawaan_logika(
    interpreter: &mut Interpreter,
    _env: &Env,
    args: Vec<Nilai>,
    _kwargs: Vec<(String, Nilai)>,
) -> Hasil<Nilai> {
    let nilai = args.first().cloned().unwrap_or(Nilai::Logika(false));
    Ok(Nilai::Logika(interpreter.kebenaran(&nilai)?))
}

fn bawaan_bytes(
    interpreter: &mut Interpreter,
    _env: &Env,
    args: Vec<Nilai>,
    _kwargs: Vec<(String, Nilai)>,
) -> Hasil<Nilai> {
    match args.len() {
        0 => Ok(Nilai::Bytes(Rc::new(Vec::new()))),
        1 => match &args[0] {
            Nilai::Bilangan(n) => {
                if *n < 0 {
                    return galat_nilai(interpreter, "panjang bytes tidak boleh negatif");
                }
                Ok(Nilai::Bytes(Rc::new(vec![0u8; *n as usize])))
            }
            nilai => interpreter.ke_bytes(nilai, interpreter.span_aktif),
        },
        2 => {
            let teks = teks_arg(interpreter, &args[0], "nilai")?;
            let encoding = teks_arg(interpreter, &args[1], "encoding")?;
            if !encoding.eq_ignore_ascii_case("utf-8") && !encoding.eq_ignore_ascii_case("utf8") {
                return galat_nilai(
                    interpreter,
                    format!("encoding `{encoding}` belum didukung; gunakan utf-8"),
                );
            }
            Ok(Nilai::Bytes(Rc::new(teks.into_bytes())))
        }
        _ => galat(interpreter, "`bytes` menerima paling banyak dua argumen"),
    }
}

pub fn jenis_dari_nilai(nilai: &Nilai) -> JenisKelas {
    match nilai {
        Nilai::Kosong => JenisKelas::Kosong,
        Nilai::Logika(_) => JenisKelas::Logika,
        Nilai::Bilangan(_) => JenisKelas::Bilangan,
        Nilai::Pecahan(_) => JenisKelas::Pecahan,
        Nilai::Kompleks(_, _) => JenisKelas::Kompleks,
        Nilai::Teks(_) => JenisKelas::Teks,
        Nilai::Bytes(_) => JenisKelas::Bytes,
        Nilai::Daftar(_) => JenisKelas::Daftar,
        Nilai::Tupel(_) => JenisKelas::Tupel,
        Nilai::Himpunan(_) => JenisKelas::Himpunan,
        Nilai::Kamus(_) => JenisKelas::Kamus,
        Nilai::Rentang(_) => JenisKelas::Rentang,
        Nilai::Slice(_) => JenisKelas::Pengguna,
        Nilai::Fungsi(_)
        | Nilai::Bawaan(_)
        | Nilai::BawaanKhusus(_)
        | Nilai::Metode(_)
        | Nilai::MetodeBawaan(_)
        | Nilai::MetodeStatis(_)
        | Nilai::MetodeKelas(_)
        | Nilai::Properti(_) => JenisKelas::Fungsi,
        Nilai::Kelas(_) => JenisKelas::Tipe,
        Nilai::Instance(instance) => instance.kelas.jenis,
        Nilai::Modul(_) => JenisKelas::Modul,
        Nilai::Generator(_) => JenisKelas::Generator,
        Nilai::Kesalahan(_) => JenisKelas::Kesalahan,
        Nilai::Induk(_) => JenisKelas::Pengguna,
        Nilai::Berkas(_) => JenisKelas::Berkas,
        Nilai::Python(_) => JenisKelas::Pengguna,
        Nilai::Ellipsis => JenisKelas::Kosong,
    }
}

pub fn nama_kelas_bawaan(jenis: JenisKelas) -> &'static str {
    match jenis {
        JenisKelas::Pengguna => "objek",
        JenisKelas::Kosong => "Kosong",
        JenisKelas::Logika => "Logika",
        JenisKelas::Bilangan => "Bilangan",
        JenisKelas::Pecahan => "Pecahan",
        JenisKelas::Kompleks => "Kompleks",
        JenisKelas::Teks => "Teks",
        JenisKelas::Bytes => "Bytes",
        JenisKelas::Daftar => "Daftar",
        JenisKelas::Tupel => "Tupel",
        JenisKelas::Himpunan => "Himpunan",
        JenisKelas::Kamus => "Kamus",
        JenisKelas::Rentang => "Rentang",
        JenisKelas::Fungsi => "Fungsi",
        JenisKelas::Modul => "Modul",
        JenisKelas::Berkas => "Berkas",
        JenisKelas::Generator => "Generator",
        JenisKelas::Kesalahan => "Kesalahan",
        JenisKelas::Tipe => "Tipe",
    }
}

fn bawaan_jenis(
    interpreter: &mut Interpreter,
    _env: &Env,
    args: Vec<Nilai>,
    _kwargs: Vec<(String, Nilai)>,
) -> Hasil<Nilai> {
    let Some(nilai) = arg(&args, 0) else {
        return galat(interpreter, "`jenis` membutuhkan satu argumen");
    };
    match nilai {
        Nilai::Instance(instance) => Ok(Nilai::Kelas(instance.kelas.clone())),
        Nilai::Kesalahan(kesalahan) => Ok(Nilai::Kelas(kesalahan.tipe.clone())),
        Nilai::Kelas(kelas) => Ok(Nilai::Kelas(
            interpreter
                .kelas_bawaan
                .get("Tipe")
                .cloned()
                .unwrap_or_else(|| kelas.clone()),
        )),
        _ => {
            let nama = nama_kelas_bawaan(jenis_dari_nilai(nilai));
            match interpreter.kelas_bawaan.get(nama) {
                Some(kelas) => Ok(Nilai::Kelas(kelas.clone())),
                None => galat(interpreter, format!("kelas bawaan `{nama}` tidak tersedia")),
            }
        }
    }
}

fn bawaan_abs(
    interpreter: &mut Interpreter,
    _env: &Env,
    args: Vec<Nilai>,
    _kwargs: Vec<(String, Nilai)>,
) -> Hasil<Nilai> {
    let Some(nilai) = arg(&args, 0) else {
        return galat(interpreter, "`abs` membutuhkan satu argumen");
    };
    match nilai {
        Nilai::Logika(n) => Ok(Nilai::Bilangan(i64::from(*n))),
        Nilai::Bilangan(n) => n
            .checked_abs()
            .map(Nilai::Bilangan)
            .ok_or_else(|| {
                interpreter.lempar_pesan(
                    "KesalahanOverflow",
                    "nilai absolut melebihi kapasitas bilangan bulat",
                    interpreter.span_aktif,
                )
            }),
        Nilai::Pecahan(n) => Ok(Nilai::Pecahan(n.abs())),
        Nilai::Kompleks(a, b) => Ok(Nilai::Pecahan((a * a + b * b).sqrt())),
        Nilai::Instance(instance) => {
            if let Some(fungsi) = instance.kelas.cari_atribut("__abs__") {
                return interpreter.panggil(
                    &fungsi,
                    vec![Nilai::Instance(instance.clone())],
                    Vec::new(),
                    interpreter.span_aktif,
                );
            }
            galat(
                interpreter,
                format!("`{}` tidak mendukung `abs`", instance.kelas.nama),
            )
        }
        other => galat(
            interpreter,
            format!("`abs` tidak mendukung {}", other.nama_tipe()),
        ),
    }
}

// ------------------------------------------------------------------
// Agregasi dan pengurutan
// ------------------------------------------------------------------

fn kunci_nilai(
    interpreter: &mut Interpreter,
    kunci: &Option<Nilai>,
    nilai: &Nilai,
) -> Hasil<Nilai> {
    match kunci {
        Some(kunci) => interpreter.panggil(
            kunci,
            vec![nilai.clone()],
            Vec::new(),
            interpreter.span_aktif,
        ),
        None => Ok(nilai.clone()),
    }
}

pub fn urutkan_nilai(
    interpreter: &mut Interpreter,
    daftar: &mut [Nilai],
    kunci: &Option<Nilai>,
    terbalik: bool,
) -> Hasil<()> {
    for i in 1..daftar.len() {
        let mut j = i;
        while j > 0 {
            let a = kunci_nilai(interpreter, kunci, &daftar[j - 1])?;
            let b = kunci_nilai(interpreter, kunci, &daftar[j])?;
            // Python hanya memakai `__lt__` untuk pengurutan. Urutan menurun
            // dicapai dengan membalik operand, bukan dengan `__gt__`.
            let lebih = if terbalik {
                interpreter.bandingkan(CmpOp::Lt, &a, &b, interpreter.span_aktif)?
            } else {
                interpreter.bandingkan(CmpOp::Lt, &b, &a, interpreter.span_aktif)?
            };
            if !lebih {
                break;
            }
            daftar.swap(j - 1, j);
            j -= 1;
        }
    }
    Ok(())
}

fn bawaan_maks(
    interpreter: &mut Interpreter,
    _env: &Env,
    args: Vec<Nilai>,
    kwargs: Vec<(String, Nilai)>,
) -> Hasil<Nilai> {
    ekstrem(interpreter, args, kwargs, true)
}

fn bawaan_min(
    interpreter: &mut Interpreter,
    _env: &Env,
    args: Vec<Nilai>,
    kwargs: Vec<(String, Nilai)>,
) -> Hasil<Nilai> {
    ekstrem(interpreter, args, kwargs, false)
}

fn ekstrem(
    interpreter: &mut Interpreter,
    args: Vec<Nilai>,
    kwargs: Vec<(String, Nilai)>,
    maks: bool,
) -> Hasil<Nilai> {
    let kunci = opsi(&kwargs, "key")
        .or_else(|| opsi(&kwargs, "kunci"))
        .cloned();
    let bawaan = opsi(&kwargs, "default")
        .or_else(|| opsi(&kwargs, "bawaan"))
        .cloned();
    let mut item = if args.len() == 1 {
        iter_arg(interpreter, &args[0])?
    } else if args.is_empty() {
        return galat(interpreter, "`maks`/`min` membutuhkan argumen");
    } else {
        args
    };
    if item.is_empty() {
        return match bawaan {
            Some(nilai) => Ok(nilai),
            None => galat_nilai(
                interpreter,
                "`maks`/`min` tidak dapat memilih dari urutan kosong",
            ),
        };
    }
    let mut terbaik = item.remove(0);
    let mut kunci_terbaik = kunci_nilai(interpreter, &kunci, &terbaik)?;
    for nilai in item {
        let kunci_nilai_ini = kunci_nilai(interpreter, &kunci, &nilai)?;
        let lebih = if maks {
            interpreter.bandingkan(
                CmpOp::Gt,
                &kunci_nilai_ini,
                &kunci_terbaik,
                interpreter.span_aktif,
            )?
        } else {
            interpreter.bandingkan(
                CmpOp::Lt,
                &kunci_nilai_ini,
                &kunci_terbaik,
                interpreter.span_aktif,
            )?
        };
        if lebih {
            terbaik = nilai;
            kunci_terbaik = kunci_nilai_ini;
        }
    }
    Ok(terbaik)
}

fn bawaan_bulat(
    interpreter: &mut Interpreter,
    _env: &Env,
    args: Vec<Nilai>,
    _kwargs: Vec<(String, Nilai)>,
) -> Hasil<Nilai> {
    let Some(nilai) = arg(&args, 0) else {
        return galat(interpreter, "`bulat` membutuhkan satu argumen");
    };
    let digit = match arg(&args, 1) {
        Some(Nilai::Kosong) | None => None,
        Some(nilai) => Some(interpreter.nilai_ke_i64(nilai, interpreter.span_aktif)?),
    };
    match nilai {
        Nilai::Bilangan(n) => Ok(Nilai::Bilangan(*n)),
        Nilai::Pecahan(n) => match digit {
            None | Some(0) => Ok(Nilai::Bilangan(bulat_setengah_genap(*n) as i64)),
            Some(digit) => {
                let faktor = 10f64.powi(digit as i32);
                Ok(Nilai::Pecahan(bulat_setengah_genap(*n * faktor) / faktor))
            }
        },
        Nilai::Instance(instance) => {
            if let Some(fungsi) = instance.kelas.cari_atribut("__round__") {
                let mut argumen = vec![Nilai::Instance(instance.clone())];
                if let Some(digit) = digit {
                    argumen.push(Nilai::Bilangan(digit));
                }
                return interpreter.panggil(
                    &fungsi,
                    argumen,
                    Vec::new(),
                    interpreter.span_aktif,
                );
            }
            galat(interpreter, "nilai tidak mendukung `bulat`")
        }
        other => galat(
            interpreter,
            format!("`bulat` tidak mendukung {}", other.nama_tipe()),
        ),
    }
}

fn bulat_setengah_genap(nilai: f64) -> f64 {
    let bawah = nilai.floor();
    let selisih = nilai - bawah;
    if selisih > 0.5 {
        bawah + 1.0
    } else if selisih < 0.5 {
        bawah
    } else if (bawah as i64) % 2 == 0 {
        bawah
    } else {
        bawah + 1.0
    }
}

fn bawaan_jumlah(
    interpreter: &mut Interpreter,
    _env: &Env,
    args: Vec<Nilai>,
    _kwargs: Vec<(String, Nilai)>,
) -> Hasil<Nilai> {
    let Some(nilai) = arg(&args, 0) else {
        return galat(interpreter, "`jumlah` membutuhkan satu argumen");
    };
    let mut total = args.get(1).cloned().unwrap_or(Nilai::Bilangan(0));
    for item in iter_arg(interpreter, nilai)? {
        total = interpreter.operasi_biner(
            aksara_ast::BinOp::Add,
            total,
            item,
            interpreter.span_aktif,
        )?;
    }
    Ok(total)
}

fn bawaan_semua(
    interpreter: &mut Interpreter,
    _env: &Env,
    args: Vec<Nilai>,
    _kwargs: Vec<(String, Nilai)>,
) -> Hasil<Nilai> {
    let Some(nilai) = arg(&args, 0) else {
        return galat(interpreter, "`semua` membutuhkan satu argumen");
    };
    for item in iter_arg(interpreter, nilai)? {
        if !interpreter.kebenaran(&item)? {
            return Ok(Nilai::Logika(false));
        }
    }
    Ok(Nilai::Logika(true))
}

fn bawaan_apa_saja(
    interpreter: &mut Interpreter,
    _env: &Env,
    args: Vec<Nilai>,
    _kwargs: Vec<(String, Nilai)>,
) -> Hasil<Nilai> {
    let Some(nilai) = arg(&args, 0) else {
        return galat(interpreter, "`apa_saja` membutuhkan satu argumen");
    };
    for item in iter_arg(interpreter, nilai)? {
        if interpreter.kebenaran(&item)? {
            return Ok(Nilai::Logika(true));
        }
    }
    Ok(Nilai::Logika(false))
}

fn bawaan_enumerasi(
    interpreter: &mut Interpreter,
    _env: &Env,
    args: Vec<Nilai>,
    kwargs: Vec<(String, Nilai)>,
) -> Hasil<Nilai> {
    let Some(nilai) = arg(&args, 0) else {
        return galat(interpreter, "`enumerasi` membutuhkan satu argumen");
    };
    let mulai = match arg(&args, 1).or_else(|| opsi(&kwargs, "mulai").or_else(|| opsi(&kwargs, "start"))) {
        Some(nilai) => interpreter.nilai_ke_i64(nilai, interpreter.span_aktif)?,
        None => 0,
    };
    let item = iter_arg(interpreter, nilai)?;
    let hasil = item
        .into_iter()
        .enumerate()
        .map(|(i, nilai)| Nilai::tupel(vec![Nilai::Bilangan(mulai + i as i64), nilai]))
        .collect();
    Ok(Nilai::Generator(Rc::new(std::cell::RefCell::new(Generator {
        nama: "enumerasi".to_string(),
        nilai: hasil,
        indeks: 0,
    }))))
}

fn bawaan_zip(
    interpreter: &mut Interpreter,
    _env: &Env,
    args: Vec<Nilai>,
    kwargs: Vec<(String, Nilai)>,
) -> Hasil<Nilai> {
    if args.is_empty() {
        return Ok(Nilai::Generator(Rc::new(std::cell::RefCell::new(
            Generator {
                nama: "zip".to_string(),
                nilai: Vec::new(),
                indeks: 0,
            },
        ))));
    }
    let strict = opsi(&kwargs, "strict")
        .map(|nilai| interpreter.kebenaran(nilai))
        .transpose()?
        .unwrap_or(false);
    let semua: Vec<Vec<Nilai>> = args
        .iter()
        .map(|nilai| iter_arg(interpreter, nilai))
        .collect::<Hasil<Vec<_>>>()?;
    let panjang = if strict {
        let pertama = semua[0].len();
        if semua.iter().any(|item| item.len() != pertama) {
            return galat_nilai(interpreter, "argumen `zip` memiliki panjang berbeda");
        }
        pertama
    } else {
        semua.iter().map(Vec::len).min().unwrap_or(0)
    };
    let hasil = (0..panjang)
        .map(|i| Nilai::tupel(semua.iter().map(|item| item[i].clone()).collect()))
        .collect();
    Ok(Nilai::Generator(Rc::new(std::cell::RefCell::new(Generator {
        nama: "zip".to_string(),
        nilai: hasil,
        indeks: 0,
    }))))
}

fn bawaan_peta(
    interpreter: &mut Interpreter,
    _env: &Env,
    args: Vec<Nilai>,
    _kwargs: Vec<(String, Nilai)>,
) -> Hasil<Nilai> {
    let Some(fungsi) = arg(&args, 0).cloned() else {
        return galat(interpreter, "`peta` membutuhkan fungsi");
    };
    if args.len() < 2 {
        return galat(interpreter, "`peta` membutuhkan minimal satu iterable");
    }
    let semua: Vec<Vec<Nilai>> = args[1..]
        .iter()
        .map(|nilai| iter_arg(interpreter, nilai))
        .collect::<Hasil<Vec<_>>>()?;
    let panjang = semua.iter().map(Vec::len).min().unwrap_or(0);
    let mut hasil = Vec::with_capacity(panjang);
    for i in 0..panjang {
        let argumen = semua.iter().map(|item| item[i].clone()).collect();
        hasil.push(interpreter.panggil(
            &fungsi,
            argumen,
            Vec::new(),
            interpreter.span_aktif,
        )?);
    }
    Ok(Nilai::Generator(Rc::new(std::cell::RefCell::new(Generator {
        nama: "peta".to_string(),
        nilai: hasil,
        indeks: 0,
    }))))
}

fn bawaan_saring(
    interpreter: &mut Interpreter,
    _env: &Env,
    args: Vec<Nilai>,
    _kwargs: Vec<(String, Nilai)>,
) -> Hasil<Nilai> {
    let Some(fungsi) = arg(&args, 0).cloned() else {
        return galat(interpreter, "`saring` membutuhkan fungsi atau kosong");
    };
    let Some(nilai) = arg(&args, 1) else {
        return galat(interpreter, "`saring` membutuhkan iterable");
    };
    let mut hasil = Vec::new();
    for item in iter_arg(interpreter, nilai)? {
        let lolos = if nilai.adalah_kosong() || fungsi.adalah_kosong() {
            interpreter.kebenaran(&item)?
        } else {
            let nilai = interpreter.panggil(
                &fungsi,
                vec![item.clone()],
                Vec::new(),
                interpreter.span_aktif,
            )?;
            interpreter.kebenaran(&nilai)?
        };
        if lolos {
            hasil.push(item);
        }
    }
    Ok(Nilai::Generator(Rc::new(std::cell::RefCell::new(Generator {
        nama: "saring".to_string(),
        nilai: hasil,
        indeks: 0,
    }))))
}

fn bawaan_sorted(
    interpreter: &mut Interpreter,
    _env: &Env,
    args: Vec<Nilai>,
    kwargs: Vec<(String, Nilai)>,
) -> Hasil<Nilai> {
    let Some(nilai) = arg(&args, 0) else {
        return galat(interpreter, "`sorted` membutuhkan satu argumen");
    };
    let kunci = opsi(&kwargs, "key")
        .or_else(|| opsi(&kwargs, "kunci"))
        .cloned();
    let terbalik = opsi(&kwargs, "reverse")
        .or_else(|| opsi(&kwargs, "terbalik"))
        .map(|nilai| interpreter.kebenaran(nilai))
        .transpose()?
        .unwrap_or(false);
    let mut item = iter_arg(interpreter, nilai)?;
    urutkan_nilai(interpreter, &mut item, &kunci, terbalik)?;
    Ok(Nilai::daftar(item))
}

fn bawaan_reversed(
    interpreter: &mut Interpreter,
    _env: &Env,
    args: Vec<Nilai>,
    _kwargs: Vec<(String, Nilai)>,
) -> Hasil<Nilai> {
    let Some(nilai) = arg(&args, 0) else {
        return galat(interpreter, "`reversed` membutuhkan satu argumen");
    };
    let mut item = iter_arg(interpreter, nilai)?;
    item.reverse();
    Ok(Nilai::Generator(Rc::new(std::cell::RefCell::new(Generator {
        nama: "reversed".to_string(),
        nilai: item,
        indeks: 0,
    }))))
}

fn bawaan_divmod(
    interpreter: &mut Interpreter,
    _env: &Env,
    args: Vec<Nilai>,
    _kwargs: Vec<(String, Nilai)>,
) -> Hasil<Nilai> {
    if args.len() != 2 {
        return galat(interpreter, "`divmod` membutuhkan dua argumen");
    }
    let hasil_bagi = interpreter.operasi_biner(
        aksara_ast::BinOp::FloorDiv,
        args[0].clone(),
        args[1].clone(),
        interpreter.span_aktif,
    )?;
    let sisa = interpreter.operasi_biner(
        aksara_ast::BinOp::Mod,
        args[0].clone(),
        args[1].clone(),
        interpreter.span_aktif,
    )?;
    Ok(Nilai::tupel(vec![hasil_bagi, sisa]))
}

fn bawaan_pow(
    interpreter: &mut Interpreter,
    _env: &Env,
    args: Vec<Nilai>,
    _kwargs: Vec<(String, Nilai)>,
) -> Hasil<Nilai> {
    if args.len() < 2 {
        return galat(interpreter, "`pow` membutuhkan dua atau tiga argumen");
    }
    if args.len() == 3 {
        let a = interpreter.nilai_ke_i64(&args[0], interpreter.span_aktif)?;
        let b = interpreter.nilai_ke_i64(&args[1], interpreter.span_aktif)?;
        let m = interpreter.nilai_ke_i64(&args[2], interpreter.span_aktif)?;
        if m == 0 {
            return galat_nilai(interpreter, "modulo `pow` tidak boleh nol");
        }
        if b < 0 {
            return galat_nilai(
                interpreter,
                "eksponen negatif tidak didukung untuk `pow` bermodulo",
            );
        }
        return Ok(Nilai::Bilangan(pow_mod(a, b, m)));
    }
    interpreter.operasi_biner(
        aksara_ast::BinOp::Pow,
        args[0].clone(),
        args[1].clone(),
        interpreter.span_aktif,
    )
}

fn pow_mod(mut dasar: i64, mut eksponen: i64, modulo: i64) -> i64 {
    let modulo = modulo.abs();
    dasar = dasar.rem_euclid(modulo);
    let mut hasil: i64 = 1;
    while eksponen > 0 {
        if eksponen & 1 == 1 {
            hasil = (hasil as i128 * dasar as i128 % modulo as i128) as i64;
        }
        dasar = (dasar as i128 * dasar as i128 % modulo as i128) as i64;
        eksponen >>= 1;
    }
    hasil
}

// ------------------------------------------------------------------
// Pemeriksaan tipe dan objek
// ------------------------------------------------------------------

fn bawaan_isinstance(
    interpreter: &mut Interpreter,
    _env: &Env,
    args: Vec<Nilai>,
    _kwargs: Vec<(String, Nilai)>,
) -> Hasil<Nilai> {
    if args.len() != 2 {
        return galat(interpreter, "`isinstance` membutuhkan dua argumen");
    }
    let hasil = isinstance_nilai(interpreter, &args[0], &args[1])?;
    Ok(Nilai::Logika(hasil))
}

pub fn isinstance_nilai(
    interpreter: &mut Interpreter,
    nilai: &Nilai,
    kelas: &Nilai,
) -> Hasil<bool> {
    match kelas {
        Nilai::Tupel(item) => {
            for kandidat in item.iter() {
                if isinstance_nilai(interpreter, nilai, kandidat)? {
                    return Ok(true);
                }
            }
            Ok(false)
        }
        Nilai::Kelas(kelas) => match nilai {
            Nilai::Instance(instance) => Ok(instance.kelas.adalah_subkelas(kelas)),
            Nilai::Kesalahan(kesalahan) => Ok(kesalahan.tipe.adalah_subkelas(kelas)),
            Nilai::Kelas(nilai_kelas) => {
                if kelas.jenis == JenisKelas::Tipe {
                    return Ok(true);
                }
                Ok(nilai_kelas.adalah_subkelas(kelas))
            }
            _ => {
                let jenis = jenis_dari_nilai(nilai);
                if jenis == kelas.jenis {
                    return Ok(true);
                }
                // logika adalah subkelas bilangan pada model Python.
                if jenis == JenisKelas::Logika && kelas.jenis == JenisKelas::Bilangan {
                    return Ok(true);
                }
                Ok(false)
            }
        },
        other => galat(
            interpreter,
            format!(
                "argumen kedua `isinstance` harus kelas atau tupel kelas, ditemukan {}",
                other.nama_tipe()
            ),
        ),
    }
}

fn bawaan_issubclass(
    interpreter: &mut Interpreter,
    _env: &Env,
    args: Vec<Nilai>,
    _kwargs: Vec<(String, Nilai)>,
) -> Hasil<Nilai> {
    if args.len() != 2 {
        return galat(interpreter, "`issubclass` membutuhkan dua argumen");
    }
    let Nilai::Kelas(kelas) = &args[0] else {
        return galat(interpreter, "argumen pertama `issubclass` harus kelas");
    };
    let hasil = match &args[1] {
        Nilai::Tupel(item) => {
            let mut hasil = false;
            for kandidat in item.iter() {
                if let Nilai::Kelas(induk) = kandidat {
                    if kelas.adalah_subkelas(induk) {
                        hasil = true;
                        break;
                    }
                }
            }
            hasil
        }
        Nilai::Kelas(induk) => kelas.adalah_subkelas(induk),
        _ => return galat(interpreter, "argumen kedua `issubclass` harus kelas"),
    };
    Ok(Nilai::Logika(hasil))
}

fn bawaan_globals(
    _interpreter: &mut Interpreter,
    env: &Env,
    _args: Vec<Nilai>,
    _kwargs: Vec<(String, Nilai)>,
) -> Hasil<Nilai> {
    Ok(lingkungan_ke_kamus(env))
}

fn bawaan_locals(
    _interpreter: &mut Interpreter,
    env: &Env,
    _args: Vec<Nilai>,
    _kwargs: Vec<(String, Nilai)>,
) -> Hasil<Nilai> {
    Ok(lingkungan_ke_kamus(env))
}

fn lingkungan_ke_kamus(env: &Env) -> Nilai {
    let variabel = env.borrow().variabel.clone();
    let pasangan = variabel
        .into_iter()
        .map(|(nama, nilai)| (Nilai::teks(nama), nilai))
        .collect();
    Nilai::kamus(pasangan)
}

fn bawaan_hash(
    interpreter: &mut Interpreter,
    _env: &Env,
    args: Vec<Nilai>,
    _kwargs: Vec<(String, Nilai)>,
) -> Hasil<Nilai> {
    let Some(nilai) = arg(&args, 0) else {
        return galat(interpreter, "`hash` membutuhkan satu argumen");
    };
    let hash = hash_nilai(interpreter, nilai)?;
    Ok(Nilai::Bilangan(hash))
}

pub fn hash_nilai(interpreter: &mut Interpreter, nilai: &Nilai) -> Hasil<i64> {
    match nilai {
        Nilai::Kosong => Ok(0),
        Nilai::Ellipsis => Ok(1),
        Nilai::Logika(nilai) => Ok(i64::from(*nilai)),
        Nilai::Bilangan(nilai) => Ok(*nilai),
        Nilai::Pecahan(nilai) => Ok(nilai.to_bits() as i64),
        Nilai::Kompleks(a, b) => Ok((a.to_bits() as i64) ^ (b.to_bits() as i64).rotate_left(17)),
        Nilai::Teks(teks) => Ok(fnv1a(teks.as_bytes())),
        Nilai::Bytes(bytes) => Ok(fnv1a(bytes)),
        Nilai::Tupel(item) => {
            let mut hash = 0x345678i64;
            for nilai in item.iter() {
                hash = hash
                    .wrapping_mul(1000003)
                    .wrapping_add(hash_nilai(interpreter, nilai)?);
            }
            Ok(hash ^ item.len() as i64)
        }
        Nilai::Rentang(rentang) => Ok(rentang.mulai ^ rentang.akhir.rotate_left(21) ^ rentang.langkah.rotate_left(42)),
        Nilai::Daftar(_) | Nilai::Himpunan(_) | Nilai::Kamus(_) => galat(
            interpreter,
            format!("{} tidak dapat di-hash", nilai.nama_tipe()),
        ),
        Nilai::Instance(instance) => {
            if let Some(fungsi) = instance.kelas.cari_atribut("__hash__") {
                let hasil = interpreter.panggil(
                    &fungsi,
                    vec![Nilai::Instance(instance.clone())],
                    Vec::new(),
                    interpreter.span_aktif,
                )?;
                return interpreter.nilai_ke_i64(&hasil, interpreter.span_aktif);
            }
            Ok(Rc::as_ptr(instance) as i64)
        }
        Nilai::Kelas(kelas) => Ok(Rc::as_ptr(kelas) as i64),
        Nilai::Fungsi(fungsi) => Ok(Rc::as_ptr(fungsi) as i64),
        Nilai::Bawaan(bawaan) => Ok(Rc::as_ptr(bawaan) as i64),
        Nilai::Modul(modul) => Ok(Rc::as_ptr(modul) as i64),
        Nilai::Kesalahan(kesalahan) => Ok(Rc::as_ptr(kesalahan) as i64),
        Nilai::Berkas(berkas) => Ok(Rc::as_ptr(berkas) as i64),
        Nilai::Generator(generator) => Ok(Rc::as_ptr(generator) as i64),
        Nilai::Python(_) => crate::python_hash(nilai),
        _ => Ok(0),
    }
}

fn fnv1a(bytes: &[u8]) -> i64 {
    let mut hash: u64 = 0xcbf29ce484222325;
    for byte in bytes {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x100000001b3);
    }
    hash as i64
}

fn bawaan_id(
    _interpreter: &mut Interpreter,
    _env: &Env,
    args: Vec<Nilai>,
    _kwargs: Vec<(String, Nilai)>,
) -> Hasil<Nilai> {
    let Some(nilai) = arg(&args, 0) else {
        return Ok(Nilai::Bilangan(0));
    };
    let id = match nilai {
        Nilai::Teks(teks) => Rc::as_ptr(teks) as *const u8 as usize,
        Nilai::Bytes(bytes) => Rc::as_ptr(bytes) as usize,
        Nilai::Daftar(item) => Rc::as_ptr(item) as usize,
        Nilai::Tupel(item) => Rc::as_ptr(item) as usize,
        Nilai::Himpunan(item) => Rc::as_ptr(item) as usize,
        Nilai::Kamus(item) => Rc::as_ptr(item) as usize,
        Nilai::Fungsi(fungsi) => Rc::as_ptr(fungsi) as usize,
        Nilai::Bawaan(bawaan) => Rc::as_ptr(bawaan) as usize,
        Nilai::Kelas(kelas) => Rc::as_ptr(kelas) as usize,
        Nilai::Instance(instance) => Rc::as_ptr(instance) as usize,
        Nilai::Modul(modul) => Rc::as_ptr(modul) as usize,
        Nilai::Kesalahan(kesalahan) => Rc::as_ptr(kesalahan) as usize,
        Nilai::Berkas(berkas) => Rc::as_ptr(berkas) as usize,
        Nilai::Generator(generator) => Rc::as_ptr(generator) as usize,
        Nilai::Python(id) => *id,
        _ => 0,
    };
    Ok(Nilai::Bilangan(id as i64))
}

fn bawaan_iter(
    interpreter: &mut Interpreter,
    _env: &Env,
    args: Vec<Nilai>,
    _kwargs: Vec<(String, Nilai)>,
) -> Hasil<Nilai> {
    let Some(nilai) = arg(&args, 0) else {
        return galat(interpreter, "`iter` membutuhkan satu argumen");
    };
    if matches!(nilai, Nilai::Generator(_)) {
        return Ok(nilai.clone());
    }
    let item = iter_arg(interpreter, nilai)?;
    Ok(Nilai::Generator(Rc::new(std::cell::RefCell::new(Generator {
        nama: "iter".to_string(),
        nilai: item,
        indeks: 0,
    }))))
}

fn bawaan_next(
    interpreter: &mut Interpreter,
    _env: &Env,
    args: Vec<Nilai>,
    _kwargs: Vec<(String, Nilai)>,
) -> Hasil<Nilai> {
    let Some(nilai) = arg(&args, 0) else {
        return galat(interpreter, "`next` membutuhkan satu argumen");
    };
    let bawaan = args.get(1).cloned();
    match nilai {
        Nilai::Generator(generator) => {
            let mut generator = generator.borrow_mut();
            if generator.indeks < generator.nilai.len() {
                let hasil = generator.nilai[generator.indeks].clone();
                generator.indeks += 1;
                Ok(hasil)
            } else {
                match bawaan {
                    Some(nilai) => Ok(nilai),
                    None => Err(interpreter.lempar_pesan(
                        "KesalahanHentikanIterasi",
                        "iterasi selesai",
                        interpreter.span_aktif,
                    )),
                }
            }
        }
        Nilai::Instance(instance) => {
            if let Some(fungsi) = instance.kelas.cari_atribut("__next__") {
                return interpreter.panggil(
                    &fungsi,
                    vec![Nilai::Instance(instance.clone())],
                    Vec::new(),
                    interpreter.span_aktif,
                );
            }
            galat(interpreter, "nilai bukan iterator")
        }
        _ => galat(interpreter, "nilai bukan iterator"),
    }
}

fn bawaan_callable(
    _interpreter: &mut Interpreter,
    _env: &Env,
    args: Vec<Nilai>,
    _kwargs: Vec<(String, Nilai)>,
) -> Hasil<Nilai> {
    let callable = match args.first() {
        Some(Nilai::Fungsi(_))
        | Some(Nilai::Bawaan(_))
        | Some(Nilai::BawaanKhusus(_))
        | Some(Nilai::Kelas(_))
        | Some(Nilai::Metode(_))
        | Some(Nilai::MetodeBawaan(_))
        | Some(Nilai::MetodeStatis(_))
        | Some(Nilai::MetodeKelas(_))
        | Some(Nilai::Properti(_))
        | Some(Nilai::Python(_)) => true,
        Some(Nilai::Instance(instance)) => instance.kelas.cari_atribut("__call__").is_some(),
        _ => false,
    };
    Ok(Nilai::Logika(callable))
}

fn bawaan_getattr(
    interpreter: &mut Interpreter,
    _env: &Env,
    args: Vec<Nilai>,
    _kwargs: Vec<(String, Nilai)>,
) -> Hasil<Nilai> {
    if args.len() < 2 {
        return galat(interpreter, "`getattr` membutuhkan objek dan nama");
    }
    let nama = teks_arg(interpreter, &args[1], "nama atribut")?;
    match interpreter.ambil_atribut(&args[0], &nama, interpreter.span_aktif) {
        Ok(nilai) => Ok(nilai),
        Err(lemparan) => match args.get(2) {
            Some(bawaan) => Ok(bawaan.clone()),
            None => Err(lemparan),
        },
    }
}

fn bawaan_setattr(
    interpreter: &mut Interpreter,
    _env: &Env,
    args: Vec<Nilai>,
    _kwargs: Vec<(String, Nilai)>,
) -> Hasil<Nilai> {
    if args.len() != 3 {
        return galat(interpreter, "`setattr` membutuhkan objek, nama, dan nilai");
    }
    let nama = teks_arg(interpreter, &args[1], "nama atribut")?;
    interpreter.atur_atribut(&args[0], &nama, args[2].clone(), interpreter.span_aktif)?;
    Ok(Nilai::Kosong)
}

fn bawaan_hasattr(
    interpreter: &mut Interpreter,
    _env: &Env,
    args: Vec<Nilai>,
    _kwargs: Vec<(String, Nilai)>,
) -> Hasil<Nilai> {
    if args.len() != 2 {
        return galat(interpreter, "`hasattr` membutuhkan objek dan nama");
    }
    let nama = teks_arg(interpreter, &args[1], "nama atribut")?;
    Ok(Nilai::Logika(
        interpreter
            .ambil_atribut(&args[0], &nama, interpreter.span_aktif)
            .is_ok(),
    ))
}

fn bawaan_delattr(
    interpreter: &mut Interpreter,
    _env: &Env,
    args: Vec<Nilai>,
    _kwargs: Vec<(String, Nilai)>,
) -> Hasil<Nilai> {
    if args.len() != 2 {
        return galat(interpreter, "`delattr` membutuhkan objek dan nama");
    }
    let nama = teks_arg(interpreter, &args[1], "nama atribut")?;
    interpreter.hapus_atribut(&args[0], &nama, interpreter.span_aktif)?;
    Ok(Nilai::Kosong)
}

fn bawaan_vars(
    interpreter: &mut Interpreter,
    env: &Env,
    args: Vec<Nilai>,
    _kwargs: Vec<(String, Nilai)>,
) -> Hasil<Nilai> {
    match args.first() {
        None => Ok(lingkungan_ke_kamus(env)),
        Some(Nilai::Instance(instance)) => {
            let atribut = instance.atribut.borrow().clone();
            Ok(Nilai::kamus(
                atribut
                    .into_iter()
                    .map(|(nama, nilai)| (Nilai::teks(nama), nilai))
                    .collect(),
            ))
        }
        Some(Nilai::Kelas(kelas)) => {
            let atribut = kelas.atribut.borrow().clone();
            Ok(Nilai::kamus(
                atribut
                    .into_iter()
                    .map(|(nama, nilai)| (Nilai::teks(nama), nilai))
                    .collect(),
            ))
        }
        Some(Nilai::Modul(modul)) => {
            let atribut = modul.atribut.borrow().clone();
            Ok(Nilai::kamus(
                atribut
                    .into_iter()
                    .map(|(nama, nilai)| (Nilai::teks(nama), nilai))
                    .collect(),
            ))
        }
        Some(other) => galat(
            interpreter,
            format!("`vars` tidak mendukung {}", other.nama_tipe()),
        ),
    }
}

fn bawaan_dir(
    interpreter: &mut Interpreter,
    env: &Env,
    args: Vec<Nilai>,
    _kwargs: Vec<(String, Nilai)>,
) -> Hasil<Nilai> {
    let mut nama = Vec::new();
    match args.first() {
        None => {
            for (kunci, _) in env.borrow().variabel.iter() {
                nama.push(kunci.clone());
            }
        }
        Some(Nilai::Instance(instance)) => {
            for (kunci, _) in instance.atribut.borrow().iter() {
                nama.push(kunci.clone());
            }
            for kelas in mro_lengkap(&instance.kelas) {
                for (kunci, _) in kelas.atribut.borrow().iter() {
                    nama.push(kunci.clone());
                }
            }
        }
        Some(Nilai::Kelas(kelas)) => {
            for kelas in mro_lengkap(kelas) {
                for (kunci, _) in kelas.atribut.borrow().iter() {
                    nama.push(kunci.clone());
                }
            }
        }
        Some(Nilai::Modul(modul)) => {
            for (kunci, _) in modul.atribut.borrow().iter() {
                nama.push(kunci.clone());
            }
        }
        Some(other) => {
            return galat(
                interpreter,
                format!("`dir` tidak mendukung {}", other.nama_tipe()),
            )
        }
    }
    nama.sort();
    nama.dedup();
    Ok(Nilai::daftar(nama.into_iter().map(Nilai::teks).collect()))
}

fn bawaan_help(
    interpreter: &mut Interpreter,
    _env: &Env,
    args: Vec<Nilai>,
    _kwargs: Vec<(String, Nilai)>,
) -> Hasil<Nilai> {
    let Some(nilai) = args.first() else {
        interpreter.tulis_baris("Bantuan Aksara: gunakan `help(objek)` untuk melihat dokumentasi.");
        return Ok(Nilai::Kosong);
    };
    let dok = match nilai {
        Nilai::Bawaan(bawaan) => bawaan.doc.clone(),
        Nilai::Fungsi(fungsi) => fungsi.doc.clone(),
        Nilai::Kelas(kelas) => kelas
            .atribut
            .borrow()
            .get("__dok__")
            .map(nilai_ke_teks),
        Nilai::Modul(modul) => modul.doc.clone(),
        _ => None,
    };
    match dok {
        Some(dok) => interpreter.tulis_baris(dok),
        None => interpreter.tulis_baris(format!(
            "Tidak ada dokumentasi untuk {}.",
            nilai.nama_tipe()
        )),
    }
    Ok(Nilai::Kosong)
}

fn bawaan_exit(
    interpreter: &mut Interpreter,
    _env: &Env,
    args: Vec<Nilai>,
    _kwargs: Vec<(String, Nilai)>,
) -> Hasil<Nilai> {
    let kode = match args.first() {
        Some(Nilai::Bilangan(n)) => *n,
        Some(Nilai::Kosong) | None => 0,
        Some(nilai) => {
            interpreter.tulis_baris(nilai_ke_teks(nilai));
            1
        }
    };
    Err(interpreter.lempar_pesan(
        "KesalahanKeluar",
        format!("program keluar dengan kode {kode}"),
        interpreter.span_aktif,
    ))
}

// ------------------------------------------------------------------
// Dekorator dan kelas
// ------------------------------------------------------------------

fn bawaan_staticmethod(
    interpreter: &mut Interpreter,
    _env: &Env,
    args: Vec<Nilai>,
    _kwargs: Vec<(String, Nilai)>,
) -> Hasil<Nilai> {
    let Some(nilai) = args.first().cloned() else {
        return galat(interpreter, "`staticmethod` membutuhkan fungsi");
    };
    match nilai {
        Nilai::Fungsi(fungsi) => Ok(Nilai::MetodeStatis(fungsi)),
        Nilai::Metode(metode) => Ok(Nilai::MetodeStatis(metode.fungsi.clone())),
        other => galat(
            interpreter,
            format!(
                "`staticmethod` membutuhkan fungsi, ditemukan {}",
                other.nama_tipe()
            ),
        ),
    }
}

fn bawaan_classmethod(
    interpreter: &mut Interpreter,
    _env: &Env,
    args: Vec<Nilai>,
    _kwargs: Vec<(String, Nilai)>,
) -> Hasil<Nilai> {
    let Some(nilai) = args.first().cloned() else {
        return galat(interpreter, "`classmethod` membutuhkan fungsi");
    };
    let fungsi = match nilai {
        Nilai::Fungsi(fungsi) => fungsi,
        Nilai::Metode(metode) => metode.fungsi.clone(),
        other => {
            return galat(
                interpreter,
                format!(
                    "`classmethod` membutuhkan fungsi, ditemukan {}",
                    other.nama_tipe()
                ),
            )
        }
    };
    let kelas = Kelas::baru_jenis("__kelas__", Vec::new(), JenisKelas::Pengguna);
    Ok(Nilai::MetodeKelas(Rc::new(MetodeKelas { fungsi, kelas })))
}

fn bawaan_property(
    interpreter: &mut Interpreter,
    _env: &Env,
    args: Vec<Nilai>,
    kwargs: Vec<(String, Nilai)>,
) -> Hasil<Nilai> {
    let getter = args
        .first()
        .cloned()
        .or_else(|| opsi(&kwargs, "fget").cloned())
        .unwrap_or(Nilai::Kosong);
    if getter.adalah_kosong() {
        return galat(interpreter, "`property` membutuhkan getter");
    }
    let setter = args
        .get(1)
        .cloned()
        .or_else(|| opsi(&kwargs, "fset").cloned());
    Ok(Nilai::Properti(Rc::new(Properti { getter, setter })))
}

fn format_kesalahan_instance(nama: &str, argumen: &[Nilai], pesan: &str) -> String {
    if argumen.is_empty() {
        if pesan.is_empty() {
            nama.to_string()
        } else {
            format!("{nama}: {pesan}")
        }
    } else if argumen.len() == 1 {
        format!("{nama}: {}", nilai_ke_teks(&argumen[0]))
    } else {
        format!(
            "{nama}({})",
            argumen
                .iter()
                .map(nilai_ke_teks)
                .collect::<Vec<_>>()
                .join(", ")
        )
    }
}

fn bawaan_kesalahan_init(
    interpreter: &mut Interpreter,
    args: Vec<Nilai>,
    _kwargs: Vec<(String, Nilai)>,
) -> Hasil<Nilai> {
    let Some(penerima) = args.first().cloned() else {
        return galat(interpreter, "`__init__` kesalahan membutuhkan penerima");
    };
    let sisa = args[1..].to_vec();
    let pesan = match sisa.len() {
        0 => String::new(),
        1 => interpreter.teks_nilai(&sisa[0], interpreter.span_aktif)?,
        _ => {
            let mut bagian = Vec::with_capacity(sisa.len());
            for nilai in &sisa {
                bagian.push(interpreter.teks_nilai(nilai, interpreter.span_aktif)?);
            }
            bagian.join(", ")
        }
    };
    let argumen = Nilai::tupel(sisa);
    match penerima {
        Nilai::Instance(instance) => {
            let mut atribut = instance.atribut.borrow_mut();
            atribut.insert("argumen".to_string(), argumen);
            atribut.insert("pesan".to_string(), Nilai::teks(pesan));
        }
        Nilai::Kesalahan(kesalahan) => {
            let mut atribut = kesalahan.atribut.borrow_mut();
            atribut.insert("argumen".to_string(), argumen);
            atribut.insert("pesan".to_string(), Nilai::teks(pesan));
        }
        lain => {
            return galat(
                interpreter,
                format!(
                    "`__init__` kesalahan dipanggil pada {}",
                    lain.nama_tipe()
                ),
            )
        }
    }
    Ok(Nilai::Kosong)
}

fn bawaan_kesalahan_str(
    interpreter: &mut Interpreter,
    args: Vec<Nilai>,
    _kwargs: Vec<(String, Nilai)>,
) -> Hasil<Nilai> {
    let Some(penerima) = args.first() else {
        return galat(interpreter, "`__str__` kesalahan membutuhkan penerima");
    };
    match penerima {
        Nilai::Kesalahan(kesalahan) => Ok(Nilai::teks(teks_kesalahan(kesalahan))),
        Nilai::Instance(instance) => {
            let atribut = instance.atribut.borrow();
            let pesan = atribut
                .get("pesan")
                .map(nilai_ke_teks)
                .unwrap_or_default();
            let argumen = match atribut.get("argumen") {
                Some(Nilai::Tupel(item)) => item.as_ref().clone(),
                _ => Vec::new(),
            };
            drop(atribut);
            Ok(Nilai::teks(format_kesalahan_instance(
                &instance.kelas.nama,
                &argumen,
                &pesan,
            )))
        }
        lain => galat(
            interpreter,
            format!("`__str__` kesalahan dipanggil pada {}", lain.nama_tipe()),
        ),
    }
}

fn bawaan_induk(
    interpreter: &mut Interpreter,
    _env: &Env,
    args: Vec<Nilai>,
    _kwargs: Vec<(String, Nilai)>,
) -> Hasil<Nilai> {
    if args.len() >= 2 {
        let Nilai::Kelas(kelas) = &args[0] else {
            return galat(interpreter, "argumen pertama `induk` harus kelas");
        };
        let instance = match &args[1] {
            Nilai::Instance(instance) => Some(instance.clone()),
            Nilai::Kosong => None,
            other => {
                return galat(
                    interpreter,
                    format!(
                        "argumen kedua `induk` harus instance, ditemukan {}",
                        other.nama_tipe()
                    ),
                )
            }
        };
        return Ok(Nilai::Induk(Rc::new(Induk {
            instance,
            kelas: kelas.clone(),
            mulai: 1,
        })));
    }
    let Some(bingkai) = interpreter.tumpukan.last() else {
        return galat(interpreter, "`induk` hanya boleh dipakai di dalam metode");
    };
    let Some(kelas) = bingkai.kelas.clone() else {
        return galat(interpreter, "`induk` hanya boleh dipakai di dalam metode");
    };
    let instance = match &bingkai.diri {
        Some(Nilai::Instance(instance)) => Some(instance.clone()),
        _ => None,
    };
    Ok(Nilai::Induk(Rc::new(Induk {
        instance,
        kelas,
        mulai: 1,
    })))
}

fn bawaan_slice(
    interpreter: &mut Interpreter,
    _env: &Env,
    args: Vec<Nilai>,
    _kwargs: Vec<(String, Nilai)>,
) -> Hasil<Nilai> {
    if args.is_empty() || args.len() > 3 {
        return galat(interpreter, "`slice` menerima satu sampai tiga argumen");
    }
    let angka = |interpreter: &mut Interpreter, nilai: &Nilai| -> Hasil<Option<i64>> {
        match nilai {
            Nilai::Kosong => Ok(None),
            nilai => Ok(Some(interpreter.nilai_ke_i64(nilai, interpreter.span_aktif)?)),
        }
    };
    let (mulai, akhir, langkah) = match args.len() {
        1 => (None, angka(interpreter, &args[0])?, None),
        2 => (
            angka(interpreter, &args[0])?,
            angka(interpreter, &args[1])?,
            None,
        ),
        _ => (
            angka(interpreter, &args[0])?,
            angka(interpreter, &args[1])?,
            angka(interpreter, &args[2])?,
        ),
    };
    Ok(Nilai::Slice(Rc::new(Slice {
        mulai,
        akhir,
        langkah,
    })))
}

// ------------------------------------------------------------------
// Teks dan bilangan kecil
// ------------------------------------------------------------------

fn bawaan_format(
    interpreter: &mut Interpreter,
    _env: &Env,
    args: Vec<Nilai>,
    _kwargs: Vec<(String, Nilai)>,
) -> Hasil<Nilai> {
    let Some(nilai) = arg(&args, 0) else {
        return galat(interpreter, "`format` membutuhkan satu argumen");
    };
    let spec = match arg(&args, 1) {
        Some(nilai) => teks_arg(interpreter, nilai, "spesifikasi format")?,
        None => String::new(),
    };
    if spec.is_empty() {
        return Ok(Nilai::teks(
            interpreter.teks_nilai(nilai, interpreter.span_aktif)?,
        ));
    }
    match format_nilai(nilai, &spec) {
        Ok(teks) => Ok(Nilai::teks(teks)),
        Err(pesan) => galat_nilai(interpreter, pesan),
    }
}

fn bawaan_chr(
    interpreter: &mut Interpreter,
    _env: &Env,
    args: Vec<Nilai>,
    _kwargs: Vec<(String, Nilai)>,
) -> Hasil<Nilai> {
    let Some(nilai) = arg(&args, 0) else {
        return galat(interpreter, "`chr` membutuhkan satu argumen");
    };
    let kode = interpreter.nilai_ke_i64(nilai, interpreter.span_aktif)?;
    match u32::try_from(kode).ok().and_then(char::from_u32) {
        Some(ch) => Ok(Nilai::teks(ch.to_string())),
        None => galat_nilai(interpreter, format!("{kode} bukan kode karakter yang valid")),
    }
}

fn bawaan_ord(
    interpreter: &mut Interpreter,
    _env: &Env,
    args: Vec<Nilai>,
    _kwargs: Vec<(String, Nilai)>,
) -> Hasil<Nilai> {
    let Some(nilai) = arg(&args, 0) else {
        return galat(interpreter, "`ord` membutuhkan satu argumen");
    };
    match nilai {
        Nilai::Teks(teks) => {
            let mut chars = teks.chars();
            match (chars.next(), chars.next()) {
                (Some(ch), None) => Ok(Nilai::Bilangan(ch as i64)),
                _ => galat(interpreter, "`ord` membutuhkan tepat satu karakter"),
            }
        }
        Nilai::Bytes(bytes) if bytes.len() == 1 => Ok(Nilai::Bilangan(i64::from(bytes[0]))),
        _ => galat(interpreter, "`ord` membutuhkan teks satu karakter atau bytes satu byte"),
    }
}

fn bawaan_hex(
    interpreter: &mut Interpreter,
    _env: &Env,
    args: Vec<Nilai>,
    _kwargs: Vec<(String, Nilai)>,
) -> Hasil<Nilai> {
    let n = bilangan_arg(interpreter, &args, "hex")?;
    Ok(Nilai::teks(if n < 0 {
        format!("-0x{:x}", n.unsigned_abs())
    } else {
        format!("0x{n:x}")
    }))
}

fn bawaan_oct(
    interpreter: &mut Interpreter,
    _env: &Env,
    args: Vec<Nilai>,
    _kwargs: Vec<(String, Nilai)>,
) -> Hasil<Nilai> {
    let n = bilangan_arg(interpreter, &args, "oct")?;
    Ok(Nilai::teks(if n < 0 {
        format!("-0o{:o}", n.unsigned_abs())
    } else {
        format!("0o{n:o}")
    }))
}

fn bawaan_bin(
    interpreter: &mut Interpreter,
    _env: &Env,
    args: Vec<Nilai>,
    _kwargs: Vec<(String, Nilai)>,
) -> Hasil<Nilai> {
    let n = bilangan_arg(interpreter, &args, "bin")?;
    Ok(Nilai::teks(if n < 0 {
        format!("-0b{:b}", n.unsigned_abs())
    } else {
        format!("0b{n:b}")
    }))
}

fn bilangan_arg(
    interpreter: &mut Interpreter,
    args: &[Nilai],
    nama: &str,
) -> Hasil<i64> {
    let Some(nilai) = arg(args, 0) else {
        return galat(interpreter, format!("`{nama}` membutuhkan satu argumen"));
    };
    interpreter.nilai_ke_i64(nilai, interpreter.span_aktif)
}

/// Memastikan objek Python tetap dapat dipakai sebagai kunci kamus.
pub fn kunci_python(_interpreter: &mut Interpreter, nilai: &Nilai) -> Hasil<()> {
    if let Nilai::Python(_) = nilai {
        return Ok(());
    }
    Ok(())
}

/// Menyiapkan tabel kelas untuk modul bawaan.
pub fn kelas(interpreter: &Interpreter, nama: &str) -> Option<Rc<Kelas>> {
    interpreter.kelas_bawaan.get(nama).cloned()
}

/// Memeriksa apakah nama adalah nama kelas kesalahan bawaan.
pub fn adalah_kelas_kesalahan(interpreter: &Interpreter, nama: &str) -> bool {
    interpreter
        .kelas_bawaan
        .get(nama)
        .is_some_and(|kelas| kelas.jenis == JenisKelas::Kesalahan)
}

/// Daftar nama kelas kesalahan untuk modul `kesalahan`.
pub fn daftar_kesalahan(interpreter: &Interpreter) -> Vec<String> {
    let mut hasil: Vec<String> = interpreter
        .kelas_bawaan
        .iter()
        .filter(|(_, kelas)| kelas.jenis == JenisKelas::Kesalahan)
        .map(|(nama, _)| nama.clone())
        .collect();
    hasil.sort();
    hasil
}

/// Membuat kamus dari pasangan nama dan nilai.
pub fn kamus_dari(interpreter: &mut Interpreter, pasangan: Vec<(String, Nilai)>) -> Nilai {
    let mut hasil = Vec::new();
    for (nama, nilai) in pasangan {
        interpreter.atur_kamus(&mut hasil, Nilai::teks(nama), nilai);
    }
    Nilai::kamus(hasil)
}

/// Menyalin nama dari satu kamus ke kamus lain tanpa menduplikasi kunci.
pub fn gabung_kamus(interpreter: &mut Interpreter, tujuan: &mut Vec<(Nilai, Nilai)>, asal: &Nilai) {
    if let Nilai::Kamus(pasangan) = asal {
        for (kunci, nilai) in pasangan.borrow().iter() {
            interpreter.atur_kamus(tujuan, kunci.clone(), nilai.clone());
        }
    }
}

/// Mengubah nilai menjadi HashMap untuk modul yang membutuhkannya.
pub fn ke_peta(nilai: &Nilai) -> HashMap<String, Nilai> {
    let mut hasil = HashMap::new();
    if let Nilai::Kamus(pasangan) = nilai {
        for (kunci, nilai) in pasangan.borrow().iter() {
            if let Nilai::Teks(teks) = kunci {
                hasil.insert(teks.to_string(), nilai.clone());
            }
        }
    }
    hasil
}

/// Membaca seluruh isi berkas bila nilai adalah berkas.
pub fn baca_semua(berkas: &Rc<std::cell::RefCell<Berkas>>) -> std::io::Result<String> {
    let mut berkas = berkas.borrow_mut();
    let Some(file) = berkas.berkas.as_mut() else {
        return Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            "berkas sudah ditutup",
        ));
    };
    let mut isi = String::new();
    file.read_to_string(&mut isi)?;
    Ok(isi)
}
