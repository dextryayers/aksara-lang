//! Modul bawaan inti untuk interpreter referensi.
//!
//! Modul di sini adalah bagian minimum Fase 2. Standard library lengkap
//! dibangun pada Fase 5; modul ini menjaga contoh dan uji kepatuhan tetap
//! dapat berjalan tanpa jembatan Python.

use std::cell::RefCell;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use aksara_ast::Span;

use crate::bawaan::jenis_dari_nilai;
use crate::format::{nilai_ke_repr, nilai_ke_teks};
use crate::interpreter::Interpreter;
use crate::nilai::*;

type Khusus = Box<
    dyn Fn(&mut Interpreter, Vec<Nilai>, Vec<(String, Nilai)>) -> Hasil<Nilai> + 'static,
>;

fn fungsi(nama: &str, doc: &str, f: impl Fn(&mut Interpreter, Vec<Nilai>, Vec<(String, Nilai)>) -> Hasil<Nilai> + 'static) -> Nilai {
    let _ = doc;
    Nilai::BawaanKhusus(Rc::new(BawaanKhusus {
        nama: nama.to_string(),
        fungsi: Box::new(f) as Khusus,
    }))
}

fn modul(nama: &str, doc: &str, pasangan: Vec<(&str, Nilai)>) -> Rc<Modul> {
    let mut atribut = HashMap::new();
    atribut.insert("__nama__".to_string(), Nilai::teks(nama));
    atribut.insert("__berkas__".to_string(), Nilai::Kosong);
    atribut.insert("__paket__".to_string(), Nilai::teks(""));
    atribut.insert("__dok__".to_string(), Nilai::teks(doc));
    atribut.insert("__semua__".to_string(), Nilai::Kosong);
    for (kunci, nilai) in pasangan {
        atribut.insert(kunci.to_string(), nilai);
    }
    Rc::new(Modul {
        nama: nama.to_string(),
        atribut: RefCell::new(atribut),
        doc: Some(doc.to_string()),
        berkas: None,
    })
}

fn arg(args: &[Nilai], indeks: usize) -> Option<&Nilai> {
    args.get(indeks)
}

fn opsi<'a>(kwargs: &'a [(String, Nilai)], nama: &str) -> Option<&'a Nilai> {
    kwargs.iter().find(|(kunci, _)| kunci == nama).map(|(_, nilai)| nilai)
}

fn teks_arg(interpreter: &mut Interpreter, nilai: &Nilai, apa: &str) -> Hasil<String> {
    match nilai {
        Nilai::Teks(teks) => Ok(teks.to_string()),
        Nilai::Bytes(bytes) => Ok(String::from_utf8_lossy(bytes).to_string()),
        lain => Err(interpreter.lempar_pesan(
            "KesalahanTipe",
            format!("{apa} harus teks, ditemukan {}", lain.nama_tipe()),
            interpreter.span_aktif,
        )),
    }
}

fn angka_arg(interpreter: &mut Interpreter, nilai: &Nilai, apa: &str) -> Hasil<f64> {
    interpreter.nilai_ke_f64(nilai, interpreter.span_aktif).map_err(|_| {
        interpreter.lempar_pesan(
            "KesalahanTipe",
            format!("{apa} harus angka, ditemukan {}", nilai.nama_tipe()),
            interpreter.span_aktif,
        )
    })
}

fn bulat_arg(interpreter: &mut Interpreter, nilai: &Nilai, apa: &str) -> Hasil<i64> {
    interpreter.nilai_ke_i64(nilai, interpreter.span_aktif).map_err(|_| {
        interpreter.lempar_pesan(
            "KesalahanTipe",
            format!("{apa} harus bilangan bulat, ditemukan {}", nilai.nama_tipe()),
            interpreter.span_aktif,
        )
    })
}

fn iter_arg(interpreter: &mut Interpreter, nilai: &Nilai) -> Hasil<Vec<Nilai>> {
    interpreter.iter_nilai(nilai, interpreter.span_aktif)
}

fn galat<T>(interpreter: &mut Interpreter, pesan: impl Into<String>) -> Hasil<T> {
    Err(interpreter.lempar_pesan("KesalahanNilai", pesan, interpreter.span_aktif))
}

fn butuh(interpreter: &mut Interpreter, args: &[Nilai], jumlah: usize, nama: &str) -> Hasil<()> {
    if args.len() < jumlah {
        return galat(
            interpreter,
            format!("`{nama}` membutuhkan minimal {jumlah} argumen"),
        );
    }
    Ok(())
}

fn hasil_teks(nilai: impl Into<String>) -> Nilai {
    Nilai::teks(nilai.into())
}

fn hasil_daftar(nilai: Vec<Nilai>) -> Nilai {
    Nilai::daftar(nilai)
}

fn hasil_tupel(nilai: Vec<Nilai>) -> Nilai {
    Nilai::tupel(nilai)
}

fn hasil_bilangan(nilai: i64) -> Nilai {
    Nilai::Bilangan(nilai)
}

fn hasil_pecahan(nilai: f64) -> Nilai {
    Nilai::Pecahan(nilai)
}

fn hasil_logika(nilai: bool) -> Nilai {
    Nilai::Logika(nilai)
}

fn hasil_kamus(pasangan: Vec<(String, Nilai)>) -> Nilai {
    Nilai::kamus(
        pasangan
            .into_iter()
            .map(|(kunci, nilai)| (Nilai::teks(kunci), nilai))
            .collect(),
    )
}

fn hasil_generator(nama: &str, nilai: Vec<Nilai>) -> Nilai {
    Nilai::Generator(Rc::new(RefCell::new(Generator {
        nama: nama.to_string(),
        nilai,
        indeks: 0,
    })))
}

fn kelas_modul(interpreter: &Interpreter, nama: &str) -> Option<Rc<Kelas>> {
    interpreter.kelas_bawaan.get(nama).cloned()
}

/// Mengembalikan modul native berdasarkan nama.
pub fn ambil(interpreter: &mut Interpreter, nama: &str) -> Option<Rc<Modul>> {
    match nama {
        "matematika" => Some(matematika(interpreter)),
        "alat_teks" => Some(alat_teks()),
        "alat_berkas" => Some(alat_berkas()),
        "jalur" => Some(jalur_modul()),
        "alat_daftar" => Some(alat_daftar(interpreter)),
        "alat_kamus" => Some(alat_kamus(interpreter)),
        "alat_iterasi" => Some(alat_iterasi(interpreter)),
        "alat_fungsi" => Some(alat_fungsi(interpreter)),
        "sidik" => Some(sidik()),
        "alur_asinkron" => Some(alur_asinkron()),
        "sistem_tipe" => Some(sistem_tipe(interpreter)),
        "kelas_data" => Some(kelas_data(interpreter)),
        "tipe_pilihan" => Some(tipe_pilihan(interpreter)),
        "abstrak" => Some(abstrak(interpreter)),
        "waktu" => Some(waktu()),
        "acak" => Some(acak()),
        "sistem" => Some(sistem()),
        "json" => Some(json_modul()),
        "kesalahan" => Some(kesalahan(interpreter)),
        _ => None,
    }
}

// ------------------------------------------------------------------
// matematika
// ------------------------------------------------------------------

fn matematika(interpreter: &mut Interpreter) -> Rc<Modul> {
    let satu = |f: fn(f64) -> f64, nama: &'static str, doc: &'static str| {
        fungsi(nama, doc, move |interpreter, args, _kwargs| {
            butuh(interpreter, &args, 1, nama)?;
            let x = angka_arg(interpreter, &args[0], "nilai")?;
            Ok(hasil_pecahan(f(x)))
        })
    };
    let dua = |f: fn(f64, f64) -> f64, nama: &'static str, doc: &'static str| {
        fungsi(nama, doc, move |interpreter, args, _kwargs| {
            butuh(interpreter, &args, 2, nama)?;
            let a = angka_arg(interpreter, &args[0], "nilai pertama")?;
            let b = angka_arg(interpreter, &args[1], "nilai kedua")?;
            Ok(hasil_pecahan(f(a, b)))
        })
    };
    let mut isi = vec![
        ("pi", hasil_pecahan(std::f64::consts::PI)),
        ("e", hasil_pecahan(std::f64::consts::E)),
        ("tau", hasil_pecahan(std::f64::consts::TAU)),
        ("tak_hingga", hasil_pecahan(f64::INFINITY)),
        ("inf", hasil_pecahan(f64::INFINITY)),
        ("bukan_bilangan", hasil_pecahan(f64::NAN)),
        ("nan", hasil_pecahan(f64::NAN)),
        (
            "akar",
            fungsi("akar", "akar(x) -> pecahan", |interpreter, args, _kwargs| {
                butuh(interpreter, &args, 1, "akar")?;
                let x = angka_arg(interpreter, &args[0], "nilai")?;
                if x < 0.0 {
                    return galat(interpreter, "`akar` tidak menerima bilangan negatif");
                }
                Ok(hasil_pecahan(x.sqrt()))
            }),
        ),
        (
            "pangkat",
            fungsi("pangkat", "pangkat(a, b) -> bilangan atau pecahan", |interpreter, args, _kwargs| {
                butuh(interpreter, &args, 2, "pangkat")?;
                if let (Nilai::Bilangan(a), Nilai::Bilangan(b)) = (&args[0], &args[1]) {
                    if *b >= 0 {
                        if let Some(hasil) = a.checked_pow(*b as u32) {
                            return Ok(hasil_bilangan(hasil));
                        }
                    }
                }
                let a = angka_arg(interpreter, &args[0], "basis")?;
                let b = angka_arg(interpreter, &args[1], "eksponen")?;
                Ok(hasil_pecahan(a.powf(b)))
            }),
        ),
        (
            "eksp",
            satu(f64::exp, "eksp", "eksp(x) -> pecahan"),
        ),
        (
            "log",
            fungsi("log", "log(x, basis=e) -> pecahan", |interpreter, args, _kwargs| {
                butuh(interpreter, &args, 1, "log")?;
                let x = angka_arg(interpreter, &args[0], "nilai")?;
                if x <= 0.0 {
                    return galat(interpreter, "`log` membutuhkan nilai positif");
                }
                let hasil = match args.get(1) {
                    Some(basis) => {
                        let basis = angka_arg(interpreter, basis, "basis")?;
                        if basis <= 0.0 || basis == 1.0 {
                            return galat(interpreter, "basis `log` harus positif dan bukan 1");
                        }
                        x.log(basis)
                    }
                    None => x.ln(),
                };
                Ok(hasil_pecahan(hasil))
            }),
        ),
        ("log10", satu(f64::log10, "log10", "log10(x) -> pecahan")),
        ("log2", satu(f64::log2, "log2", "log2(x) -> pecahan")),
        ("sin", satu(f64::sin, "sin", "sin(x) -> pecahan")),
        ("cos", satu(f64::cos, "cos", "cos(x) -> pecahan")),
        ("tan", satu(f64::tan, "tan", "tan(x) -> pecahan")),
        ("asin", satu(f64::asin, "asin", "asin(x) -> pecahan")),
        ("acos", satu(f64::acos, "acos", "acos(x) -> pecahan")),
        ("atan", satu(f64::atan, "atan", "atan(x) -> pecahan")),
        ("sinh", satu(f64::sinh, "sinh", "sinh(x) -> pecahan")),
        ("cosh", satu(f64::cosh, "cosh", "cosh(x) -> pecahan")),
        ("tanh", satu(f64::tanh, "tanh", "tanh(x) -> pecahan")),
        ("atan2", dua(f64::atan2, "atan2", "atan2(y, x) -> pecahan")),
        (
            "lantai",
            fungsi("lantai", "lantai(x) -> bilangan", |interpreter, args, _kwargs| {
                butuh(interpreter, &args, 1, "lantai")?;
                let x = angka_arg(interpreter, &args[0], "nilai")?;
                Ok(hasil_bilangan(x.floor() as i64))
            }),
        ),
        (
            "langit_langit",
            fungsi("langit_langit", "langit_langit(x) -> bilangan", |interpreter, args, _kwargs| {
                butuh(interpreter, &args, 1, "langit_langit")?;
                let x = angka_arg(interpreter, &args[0], "nilai")?;
                Ok(hasil_bilangan(x.ceil() as i64))
            }),
        ),
        (
            "potong",
            fungsi("potong", "potong(x) -> bilangan", |interpreter, args, _kwargs| {
                butuh(interpreter, &args, 1, "potong")?;
                let x = angka_arg(interpreter, &args[0], "nilai")?;
                Ok(hasil_bilangan(x.trunc() as i64))
            }),
        ),
        (
            "sisa",
            dua(f64::rem_euclid, "sisa", "sisa(a, b) -> pecahan"),
        ),
        (
            "fmod",
            dua(|a, b| a % b, "fmod", "fmod(a, b) -> pecahan"),
        ),
        (
            "derajat",
            satu(f64::to_degrees, "derajat", "derajat(x) -> pecahan"),
        ),
        (
            "radian",
            satu(f64::to_radians, "radian", "radian(x) -> pecahan"),
        ),
        (
            "hipotenusa",
            dua(f64::hypot, "hipotenusa", "hipotenusa(a, b) -> pecahan"),
        ),
        (
            "faktorial",
            fungsi("faktorial", "faktorial(n) -> bilangan", |interpreter, args, _kwargs| {
                butuh(interpreter, &args, 1, "faktorial")?;
                let n = bulat_arg(interpreter, &args[0], "n")?;
                if n < 0 {
                    return galat(interpreter, "`faktorial` tidak menerima bilangan negatif");
                }
                let mut hasil: i64 = 1;
                for i in 2..=n {
                    hasil = match hasil.checked_mul(i) {
                        Some(nilai) => nilai,
                        None => return galat(interpreter, "hasil faktorial melebihi kapasitas bilangan"),
                    };
                }
                Ok(hasil_bilangan(hasil))
            }),
        ),
        (
            "gcd",
            fungsi("gcd", "gcd(*bilangan) -> bilangan", |interpreter, args, _kwargs| {
                if args.is_empty() {
                    return Ok(hasil_bilangan(0));
                }
                let mut hasil = 0i64;
                for nilai in &args {
                    let n = bulat_arg(interpreter, nilai, "bilangan")?;
                    hasil = gcd(hasil, n);
                }
                Ok(hasil_bilangan(hasil))
            }),
        ),
        (
            "lcm",
            fungsi("lcm", "lcm(*bilangan) -> bilangan", |interpreter, args, _kwargs| {
                if args.is_empty() {
                    return Ok(hasil_bilangan(1));
                }
                let mut hasil = 1i64;
                for nilai in &args {
                    let n = bulat_arg(interpreter, nilai, "bilangan")?;
                    if n == 0 {
                        return Ok(hasil_bilangan(0));
                    }
                    hasil = (hasil / gcd(hasil, n)).saturating_mul(n.abs());
                }
                Ok(hasil_bilangan(hasil))
            }),
        ),
        (
            "kombinasi",
            fungsi("kombinasi", "kombinasi(n, k) -> bilangan", |interpreter, args, _kwargs| {
                butuh(interpreter, &args, 2, "kombinasi")?;
                let n = bulat_arg(interpreter, &args[0], "n")?;
                let k = bulat_arg(interpreter, &args[1], "k")?;
                if n < 0 || k < 0 || k > n {
                    return galat(interpreter, "`kombinasi` membutuhkan 0 <= k <= n");
                }
                let k = k.min(n - k);
                let mut hasil: i128 = 1;
                for i in 0..k {
                    hasil = hasil * (n - i) as i128 / (i + 1) as i128;
                }
                Ok(hasil_bilangan(hasil as i64))
            }),
        ),
        (
            "permutasi",
            fungsi("permutasi", "permutasi(n, k=None) -> bilangan", |interpreter, args, _kwargs| {
                butuh(interpreter, &args, 1, "permutasi")?;
                let n = bulat_arg(interpreter, &args[0], "n")?;
                let k = match args.get(1) {
                    Some(nilai) => bulat_arg(interpreter, nilai, "k")?,
                    None => n,
                };
                if n < 0 || k < 0 || k > n {
                    return galat(interpreter, "`permutasi` membutuhkan 0 <= k <= n");
                }
                let mut hasil: i128 = 1;
                for i in 0..k {
                    hasil *= (n - i) as i128;
                }
                Ok(hasil_bilangan(hasil as i64))
            }),
        ),
        (
            "isqrt",
            fungsi("isqrt", "isqrt(n) -> bilangan", |interpreter, args, _kwargs| {
                butuh(interpreter, &args, 1, "isqrt")?;
                let n = bulat_arg(interpreter, &args[0], "n")?;
                if n < 0 {
                    return galat(interpreter, "`isqrt` tidak menerima bilangan negatif");
                }
                Ok(hasil_bilangan((n as f64).sqrt() as i64))
            }),
        ),
        (
            "bukan_angka",
            fungsi("bukan_angka", "bukan_angka(x) -> logika", |interpreter, args, _kwargs| {
                butuh(interpreter, &args, 1, "bukan_angka")?;
                let x = angka_arg(interpreter, &args[0], "nilai")?;
                Ok(hasil_logika(x.is_nan()))
            }),
        ),
        (
            "tak_hingga",
            fungsi("tak_hingga", "tak_hingga(x) -> logika", |interpreter, args, _kwargs| {
                butuh(interpreter, &args, 1, "tak_hingga")?;
                let x = angka_arg(interpreter, &args[0], "nilai")?;
                Ok(hasil_logika(x.is_infinite()))
            }),
        ),
        (
            "terbatas",
            fungsi("terbatas", "terbatas(x) -> logika", |interpreter, args, _kwargs| {
                butuh(interpreter, &args, 1, "terbatas")?;
                let x = angka_arg(interpreter, &args[0], "nilai")?;
                Ok(hasil_logika(x.is_finite()))
            }),
        ),
        (
            "biner",
            fungsi("biner", "biner(n) -> teks", |interpreter, args, _kwargs| {
                let Some(nilai) = arg(&args, 0) else {
                    return galat(interpreter, "`biner` membutuhkan satu argumen");
                };
                let n = bulat_arg(interpreter, nilai, "n")?;
                Ok(hasil_teks(if n < 0 { format!("-0b{:b}", n.unsigned_abs()) } else { format!("0b{n:b}") }))
            }),
        ),
        (
            "oktal",
            fungsi("oktal", "oktal(n) -> teks", |interpreter, args, _kwargs| {
                let Some(nilai) = arg(&args, 0) else {
                    return galat(interpreter, "`oktal` membutuhkan satu argumen");
                };
                let n = bulat_arg(interpreter, nilai, "n")?;
                Ok(hasil_teks(if n < 0 { format!("-0o{:o}", n.unsigned_abs()) } else { format!("0o{n:o}") }))
            }),
        ),
        (
            "heksadesimal",
            fungsi("heksadesimal", "heksadesimal(n) -> teks", |interpreter, args, _kwargs| {
                let Some(nilai) = arg(&args, 0) else {
                    return galat(interpreter, "`heksadesimal` membutuhkan satu argumen");
                };
                let n = bulat_arg(interpreter, nilai, "n")?;
                Ok(hasil_teks(if n < 0 { format!("-0x{:x}", n.unsigned_abs()) } else { format!("0x{n:x}") }))
            }),
        ),
    ];
    // Alias Python yang lazim.
    let alias = [
        ("sqrt", "akar"),
        ("pow", "pangkat"),
        ("exp", "eksp"),
        ("floor", "lantai"),
        ("ceil", "langit_langit"),
        ("trunc", "potong"),
        ("fabs", "fabs"),
        ("factorial", "faktorial"),
        ("comb", "kombinasi"),
        ("perm", "permutasi"),
        ("hypot", "hipotenusa"),
        ("degrees", "derajat"),
        ("radians", "radian"),
        ("isnan", "bukan_angka"),
        ("isinf", "tak_hingga"),
        ("isfinite", "terbatas"),
        ("bin", "biner"),
        ("oct", "oktal"),
        ("hex", "heksadesimal"),
    ];
    let salinan: Vec<(&str, Nilai)> = isi.clone();
    for (python, indonesia) in alias {
        if python == "fabs" {
            isi.push((
                "fabs",
                fungsi("fabs", "fabs(x) -> pecahan", |interpreter, args, _kwargs| {
                    butuh(interpreter, &args, 1, "fabs")?;
                    Ok(hasil_pecahan(angka_arg(interpreter, &args[0], "nilai")?.abs()))
                }),
            ));
            continue;
        }
        if let Some((_, nilai)) = salinan.iter().find(|(nama, _)| *nama == indonesia) {
            isi.push((python, nilai.clone()));
        }
    }
    let _ = interpreter;
    modul("matematika", "Fungsi matematika dasar.", isi)
}

fn gcd(mut a: i64, mut b: i64) -> i64 {
    a = a.abs();
    b = b.abs();
    while b != 0 {
        let sisa = a % b;
        a = b;
        b = sisa;
    }
    a
}

// ------------------------------------------------------------------
// alat_teks
// ------------------------------------------------------------------

fn alat_teks() -> Rc<Modul> {
    let huruf = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ";
    let isi = vec![
        ("ascii_huruf_kecil", hasil_teks("abcdefghijklmnopqrstuvwxyz")),
        ("ascii_huruf_besar", hasil_teks("ABCDEFGHIJKLMNOPQRSTUVWXYZ")),
        ("ascii_huruf", hasil_teks(huruf)),
        ("digit", hasil_teks("0123456789")),
        ("tanda_baca", hasil_teks("!\"#$%&'()*+,-./:;<=>?@[\\]^_`{|}~")),
        (
            "karakter",
            fungsi("karakter", "karakter(kode) -> teks", |interpreter, args, _kwargs| {
                butuh(interpreter, &args, 1, "karakter")?;
                let kode = bulat_arg(interpreter, &args[0], "kode")?;
                match u32::try_from(kode).ok().and_then(char::from_u32) {
                    Some(ch) => Ok(hasil_teks(ch.to_string())),
                    None => galat(interpreter, format!("{kode} bukan kode karakter yang valid")),
                }
            }),
        ),
        (
            "kode_karakter",
            fungsi("kode_karakter", "kode_karakter(teks) -> bilangan", |interpreter, args, _kwargs| {
                butuh(interpreter, &args, 1, "kode_karakter")?;
                let teks = teks_arg(interpreter, &args[0], "teks")?;
                let mut chars = teks.chars();
                match (chars.next(), chars.next()) {
                    (Some(ch), None) => Ok(hasil_bilangan(ch as i64)),
                    _ => galat(interpreter, "`kode_karakter` membutuhkan tepat satu karakter"),
                }
            }),
        ),
        (
            "kapitalisasi",
            fungsi("kapitalisasi", "kapitalisasi(teks) -> teks", |interpreter, args, _kwargs| {
                butuh(interpreter, &args, 1, "kapitalisasi")?;
                let teks = teks_arg(interpreter, &args[0], "teks")?;
                let mut chars = teks.chars();
                match chars.next() {
                    Some(pertama) => Ok(hasil_teks(format!(
                        "{}{}",
                        pertama.to_uppercase(),
                        chars.as_str().to_lowercase()
                    ))),
                    None => Ok(hasil_teks("")),
                }
            }),
        ),
        (
            "judul",
            fungsi("judul", "judul(teks) -> teks", |interpreter, args, _kwargs| {
                butuh(interpreter, &args, 1, "judul")?;
                let teks = teks_arg(interpreter, &args[0], "teks")?;
                let mut hasil = String::new();
                let mut awal = true;
                for ch in teks.chars() {
                    if ch.is_alphanumeric() {
                        if awal {
                            hasil.extend(ch.to_uppercase());
                            awal = false;
                        } else {
                            hasil.extend(ch.to_lowercase());
                        }
                    } else {
                        hasil.push(ch);
                        awal = true;
                    }
                }
                Ok(hasil_teks(hasil))
            }),
        ),
        (
            "huruf_besar",
            fungsi("huruf_besar", "huruf_besar(teks) -> teks", |interpreter, args, _kwargs| {
                butuh(interpreter, &args, 1, "huruf_besar")?;
                Ok(hasil_teks(teks_arg(interpreter, &args[0], "teks")?.to_uppercase()))
            }),
        ),
        (
            "huruf_kecil",
            fungsi("huruf_kecil", "huruf_kecil(teks) -> teks", |interpreter, args, _kwargs| {
                butuh(interpreter, &args, 1, "huruf_kecil")?;
                Ok(hasil_teks(teks_arg(interpreter, &args[0], "teks")?.to_lowercase()))
            }),
        ),
        (
            "pangkas",
            fungsi("pangkas", "pangkas(teks, karakter=None) -> teks", |interpreter, args, _kwargs| {
                butuh(interpreter, &args, 1, "pangkas")?;
                let teks = teks_arg(interpreter, &args[0], "teks")?;
                let hasil = match args.get(1) {
                    Some(nilai) => teks_arg(interpreter, nilai, "karakter")?,
                    None => " \t\r\n".to_string(),
                };
                Ok(hasil_teks(teks.trim_matches(|ch| hasil.contains(ch)).to_string()))
            }),
        ),
        (
            "potong",
            fungsi("potong", "potong(teks, mulai, akhir=None) -> teks", |interpreter, args, _kwargs| {
                butuh(interpreter, &args, 2, "potong")?;
                let teks = teks_arg(interpreter, &args[0], "teks")?;
                let chars: Vec<char> = teks.chars().collect();
                let mulai = bulat_arg(interpreter, &args[1], "mulai")?;
                let akhir = match args.get(2) {
                    Some(nilai) => bulat_arg(interpreter, nilai, "akhir")?,
                    None => chars.len() as i64,
                };
                let mulai = mulai.clamp(0, chars.len() as i64) as usize;
                let akhir = akhir.clamp(0, chars.len() as i64) as usize;
                if mulai > akhir {
                    return Ok(hasil_teks(""));
                }
                Ok(hasil_teks(chars[mulai..akhir].iter().collect::<String>()))
            }),
        ),
        (
            "gabung",
            fungsi("gabung", "gabung(pemisah, iterable) -> teks", |interpreter, args, _kwargs| {
                butuh(interpreter, &args, 2, "gabung")?;
                let pemisah = teks_arg(interpreter, &args[0], "pemisah")?;
                let item = iter_arg(interpreter, &args[1])?;
                let bagian: Vec<String> = item.iter().map(nilai_ke_teks).collect();
                Ok(hasil_teks(bagian.join(&pemisah)))
            }),
        ),
        (
            "pisah",
            fungsi("pisah", "pisah(teks, pemisah=None) -> daftar", |interpreter, args, _kwargs| {
                butuh(interpreter, &args, 1, "pisah")?;
                let teks = teks_arg(interpreter, &args[0], "teks")?;
                let hasil: Vec<Nilai> = match args.get(1) {
                    Some(nilai) => {
                        let pemisah = teks_arg(interpreter, nilai, "pemisah")?;
                        if pemisah.is_empty() {
                            return galat(interpreter, "pemisah tidak boleh kosong");
                        }
                        teks.split(&pemisah).map(|bagian| Nilai::teks(bagian.to_string())).collect()
                    }
                    None => teks.split_whitespace().map(|bagian| Nilai::teks(bagian.to_string())).collect(),
                };
                Ok(hasil_daftar(hasil))
            }),
        ),
        (
            "ganti",
            fungsi("ganti", "ganti(teks, lama, baru, jumlah=None) -> teks", |interpreter, args, _kwargs| {
                butuh(interpreter, &args, 3, "ganti")?;
                let teks = teks_arg(interpreter, &args[0], "teks")?;
                let lama = teks_arg(interpreter, &args[1], "teks lama")?;
                let baru = teks_arg(interpreter, &args[2], "teks baru")?;
                let hasil = match args.get(3) {
                    Some(nilai) => {
                        let jumlah = bulat_arg(interpreter, nilai, "jumlah")?;
                        if jumlah < 0 {
                            teks.replace(&lama, &baru)
                        } else {
                            teks.replacen(&lama, &baru, jumlah as usize)
                        }
                    }
                    None => teks.replace(&lama, &baru),
                };
                Ok(hasil_teks(hasil))
            }),
        ),
        (
            "cari",
            fungsi("cari", "cari(teks, bagian, mulai=0) -> bilangan", |interpreter, args, _kwargs| {
                butuh(interpreter, &args, 2, "cari")?;
                let teks = teks_arg(interpreter, &args[0], "teks")?;
                let bagian = teks_arg(interpreter, &args[1], "bagian")?;
                let mulai = match args.get(2) {
                    Some(nilai) => bulat_arg(interpreter, nilai, "mulai")?.max(0) as usize,
                    None => 0,
                };
                let posisi = if mulai <= teks.len() {
                    teks[mulai..].find(&bagian).map(|posisi| posisi + mulai)
                } else {
                    None
                };
                Ok(hasil_bilangan(posisi.map(|p| p as i64).unwrap_or(-1)))
            }),
        ),
        (
            "hitung",
            fungsi("hitung", "hitung(teks, bagian) -> bilangan", |interpreter, args, _kwargs| {
                butuh(interpreter, &args, 2, "hitung")?;
                let teks = teks_arg(interpreter, &args[0], "teks")?;
                let bagian = teks_arg(interpreter, &args[1], "bagian")?;
                if bagian.is_empty() {
                    return Ok(hasil_bilangan((teks.chars().count() + 1) as i64));
                }
                Ok(hasil_bilangan(teks.matches(&bagian).count() as i64))
            }),
        ),
        (
            "mulai_dengan",
            fungsi("mulai_dengan", "mulai_dengan(teks, awalan) -> logika", |interpreter, args, _kwargs| {
                butuh(interpreter, &args, 2, "mulai_dengan")?;
                let teks = teks_arg(interpreter, &args[0], "teks")?;
                let awalan = teks_arg(interpreter, &args[1], "awalan")?;
                Ok(hasil_logika(teks.starts_with(&awalan)))
            }),
        ),
        (
            "berakhir_dengan",
            fungsi("berakhir_dengan", "berakhir_dengan(teks, akhiran) -> logika", |interpreter, args, _kwargs| {
                butuh(interpreter, &args, 2, "berakhir_dengan")?;
                let teks = teks_arg(interpreter, &args[0], "teks")?;
                let akhiran = teks_arg(interpreter, &args[1], "akhiran")?;
                Ok(hasil_logika(teks.ends_with(&akhiran)))
            }),
        ),
    ];
    let salinan: Vec<(&str, Nilai)> = isi.clone();
    let alias = [
        ("ascii_lowercase", "ascii_huruf_kecil"),
        ("ascii_uppercase", "ascii_huruf_besar"),
        ("ascii_letters", "ascii_huruf"),
        ("punctuation", "tanda_baca"),
        ("chr", "karakter"),
        ("ord", "kode_karakter"),
        ("capitalize", "kapitalisasi"),
        ("title", "judul"),
        ("upper", "huruf_besar"),
        ("lower", "huruf_kecil"),
        ("strip", "pangkas"),
        ("join", "gabung"),
        ("split", "pisah"),
        ("replace", "ganti"),
        ("find", "cari"),
        ("count", "hitung"),
        ("startswith", "mulai_dengan"),
        ("endswith", "berakhir_dengan"),
    ];
    let mut isi = isi;
    for (python, indonesia) in alias {
        if let Some((_, nilai)) = salinan.iter().find(|(nama, _)| *nama == indonesia) {
            isi.push((python, nilai.clone()));
        }
    }
    modul("alat_teks", "Konstanta dan pembantu teks.", isi)
}

// ------------------------------------------------------------------
// alat_berkas dan jalur
// ------------------------------------------------------------------

fn jalur_modul() -> Rc<Modul> {
    modul(
        "jalur",
        "Pembantu jalur berkas.",
        vec![
            (
                "gabung",
                fungsi("gabung", "gabung(*bagian) -> teks", |interpreter, args, _kwargs| {
                    let mut hasil = PathBuf::new();
                    for nilai in &args {
                        hasil.push(teks_arg(interpreter, nilai, "bagian jalur")?);
                    }
                    Ok(hasil_teks(hasil.to_string_lossy().to_string()))
                }),
            ),
            (
                "nama_dasar",
                fungsi("nama_dasar", "nama_dasar(jalur) -> teks", |interpreter, args, _kwargs| {
                    butuh(interpreter, &args, 1, "nama_dasar")?;
                    let jalur = teks_arg(interpreter, &args[0], "jalur")?;
                    Ok(hasil_teks(
                        Path::new(&jalur)
                            .file_name()
                            .map(|nama| nama.to_string_lossy().to_string())
                            .unwrap_or_default(),
                    ))
                }),
            ),
            (
                "direktori",
                fungsi("direktori", "direktori(jalur) -> teks", |interpreter, args, _kwargs| {
                    butuh(interpreter, &args, 1, "direktori")?;
                    let jalur = teks_arg(interpreter, &args[0], "jalur")?;
                    Ok(hasil_teks(
                        Path::new(&jalur)
                            .parent()
                            .map(|induk| induk.to_string_lossy().to_string())
                            .unwrap_or_default(),
                    ))
                }),
            ),
            (
                "ekstensi",
                fungsi("ekstensi", "ekstensi(jalur) -> teks", |interpreter, args, _kwargs| {
                    butuh(interpreter, &args, 1, "ekstensi")?;
                    let jalur = teks_arg(interpreter, &args[0], "jalur")?;
                    Ok(hasil_teks(
                        Path::new(&jalur)
                            .extension()
                            .map(|ext| format!(".{}", ext.to_string_lossy()))
                            .unwrap_or_default(),
                    ))
                }),
            ),
            (
                "absolut",
                fungsi("absolut", "absolut(jalur) -> teks", |interpreter, args, _kwargs| {
                    butuh(interpreter, &args, 1, "absolut")?;
                    let jalur = teks_arg(interpreter, &args[0], "jalur")?;
                    match std::fs::canonicalize(&jalur) {
                        Ok(hasil) => Ok(hasil_teks(hasil.to_string_lossy().to_string())),
                        Err(_) => {
                            let path = Path::new(&jalur);
                            let absolut = if path.is_absolute() {
                                path.to_path_buf()
                            } else {
                                std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")).join(path)
                            };
                            Ok(hasil_teks(absolut.to_string_lossy().to_string()))
                        }
                    }
                }),
            ),
            (
                "ada",
                fungsi("ada", "ada(jalur) -> logika", |interpreter, args, _kwargs| {
                    butuh(interpreter, &args, 1, "ada")?;
                    let jalur = teks_arg(interpreter, &args[0], "jalur")?;
                    Ok(hasil_logika(Path::new(&jalur).exists()))
                }),
            ),
        ],
    )
}

fn alat_berkas() -> Rc<Modul> {
    modul(
        "alat_berkas",
        "Operasi berkas dan direktori.",
        vec![
            (
                "baca",
                fungsi("baca", "baca(jalur, encoding='utf-8') -> teks", |interpreter, args, _kwargs| {
                    butuh(interpreter, &args, 1, "baca")?;
                    let jalur = teks_arg(interpreter, &args[0], "jalur")?;
                    match std::fs::read_to_string(&jalur) {
                        Ok(isi) => Ok(hasil_teks(isi)),
                        Err(galat) => Err(interpreter.lempar_pesan(
                            "KesalahanBerkas",
                            format!("tidak dapat membaca `{jalur}`: {galat}"),
                            interpreter.span_aktif,
                        )),
                    }
                }),
            ),
            (
                "tulis",
                fungsi("tulis", "tulis(jalur, isi) -> kosong", |interpreter, args, _kwargs| {
                    butuh(interpreter, &args, 2, "tulis")?;
                    let jalur = teks_arg(interpreter, &args[0], "jalur")?;
                    let isi = teks_arg(interpreter, &args[1], "isi")?;
                    match std::fs::write(&jalur, isi) {
                        Ok(()) => Ok(Nilai::Kosong),
                        Err(galat) => Err(interpreter.lempar_pesan(
                            "KesalahanBerkas",
                            format!("tidak dapat menulis `{jalur}`: {galat}"),
                            interpreter.span_aktif,
                        )),
                    }
                }),
            ),
            (
                "tambah",
                fungsi("tambah", "tambah(jalur, isi) -> kosong", |interpreter, args, _kwargs| {
                    use std::io::Write;
                    butuh(interpreter, &args, 2, "tambah")?;
                    let jalur = teks_arg(interpreter, &args[0], "jalur")?;
                    let isi = teks_arg(interpreter, &args[1], "isi")?;
                    let hasil = std::fs::OpenOptions::new()
                        .create(true)
                        .append(true)
                        .open(&jalur)
                        .and_then(|mut berkas| berkas.write_all(isi.as_bytes()));
                    match hasil {
                        Ok(()) => Ok(Nilai::Kosong),
                        Err(galat) => Err(interpreter.lempar_pesan(
                            "KesalahanBerkas",
                            format!("tidak dapat menambah `{jalur}`: {galat}"),
                            interpreter.span_aktif,
                        )),
                    }
                }),
            ),
            (
                "ada",
                fungsi("ada", "ada(jalur) -> logika", |interpreter, args, _kwargs| {
                    butuh(interpreter, &args, 1, "ada")?;
                    let jalur = teks_arg(interpreter, &args[0], "jalur")?;
                    Ok(hasil_logika(Path::new(&jalur).exists()))
                }),
            ),
            (
                "hapus",
                fungsi("hapus", "hapus(jalur) -> kosong", |interpreter, args, _kwargs| {
                    butuh(interpreter, &args, 1, "hapus")?;
                    let jalur = teks_arg(interpreter, &args[0], "jalur")?;
                    match std::fs::remove_file(&jalur) {
                        Ok(()) => Ok(Nilai::Kosong),
                        Err(galat) => Err(interpreter.lempar_pesan(
                            "KesalahanBerkas",
                            format!("tidak dapat menghapus `{jalur}`: {galat}"),
                            interpreter.span_aktif,
                        )),
                    }
                }),
            ),
            (
                "daftar",
                fungsi("daftar", "daftar(direktori='.') -> daftar", |interpreter, args, _kwargs| {
                    let direktori = match args.first() {
                        Some(nilai) => teks_arg(interpreter, nilai, "direktori")?,
                        None => ".".to_string(),
                    };
                    match std::fs::read_dir(&direktori) {
                        Ok(entri) => {
                            let mut nama = Vec::new();
                            for entri in entri.flatten() {
                                nama.push(Nilai::teks(entri.file_name().to_string_lossy().to_string()));
                            }
                            nama.sort_by(|a, b| nilai_ke_teks(a).cmp(&nilai_ke_teks(b)));
                            Ok(hasil_daftar(nama))
                        }
                        Err(galat) => Err(interpreter.lempar_pesan(
                            "KesalahanBerkas",
                            format!("tidak dapat membaca direktori `{direktori}`: {galat}"),
                            interpreter.span_aktif,
                        )),
                    }
                }),
            ),
            (
                "buat_direktori",
                fungsi("buat_direktori", "buat_direktori(jalur, induk=False) -> kosong", |interpreter, args, _kwargs| {
                    butuh(interpreter, &args, 1, "buat_direktori")?;
                    let jalur = teks_arg(interpreter, &args[0], "jalur")?;
                    let induk = match args.get(1) {
                        Some(nilai) => interpreter.kebenaran(nilai)?,
                        None => false,
                    };
                    let hasil = if induk {
                        std::fs::create_dir_all(&jalur)
                    } else {
                        std::fs::create_dir(&jalur)
                    };
                    match hasil {
                        Ok(()) => Ok(Nilai::Kosong),
                        Err(galat) => Err(interpreter.lempar_pesan(
                            "KesalahanBerkas",
                            format!("tidak dapat membuat direktori `{jalur}`: {galat}"),
                            interpreter.span_aktif,
                        )),
                    }
                }),
            ),
            (
                "hapus_direktori",
                fungsi("hapus_direktori", "hapus_direktori(jalur, rekursif=False) -> kosong", |interpreter, args, _kwargs| {
                    butuh(interpreter, &args, 1, "hapus_direktori")?;
                    let jalur = teks_arg(interpreter, &args[0], "jalur")?;
                    let rekursif = match args.get(1) {
                        Some(nilai) => interpreter.kebenaran(nilai)?,
                        None => false,
                    };
                    let hasil = if rekursif {
                        std::fs::remove_dir_all(&jalur)
                    } else {
                        std::fs::remove_dir(&jalur)
                    };
                    match hasil {
                        Ok(()) => Ok(Nilai::Kosong),
                        Err(galat) => Err(interpreter.lempar_pesan(
                            "KesalahanBerkas",
                            format!("tidak dapat menghapus direktori `{jalur}`: {galat}"),
                            interpreter.span_aktif,
                        )),
                    }
                }),
            ),
            (
                "ukuran",
                fungsi("ukuran", "ukuran(jalur) -> bilangan", |interpreter, args, _kwargs| {
                    butuh(interpreter, &args, 1, "ukuran")?;
                    let jalur = teks_arg(interpreter, &args[0], "jalur")?;
                    match std::fs::metadata(&jalur) {
                        Ok(metadata) => Ok(hasil_bilangan(metadata.len() as i64)),
                        Err(galat) => Err(interpreter.lempar_pesan(
                            "KesalahanBerkas",
                            format!("tidak dapat membaca metadata `{jalur}`: {galat}"),
                            interpreter.span_aktif,
                        )),
                    }
                }),
            ),
            (
                "salin",
                fungsi("salin", "salin(asal, tujuan) -> kosong", |interpreter, args, _kwargs| {
                    butuh(interpreter, &args, 2, "salin")?;
                    let asal = teks_arg(interpreter, &args[0], "asal")?;
                    let tujuan = teks_arg(interpreter, &args[1], "tujuan")?;
                    match std::fs::copy(&asal, &tujuan) {
                        Ok(_) => Ok(Nilai::Kosong),
                        Err(galat) => Err(interpreter.lempar_pesan(
                            "KesalahanBerkas",
                            format!("tidak dapat menyalin `{asal}` ke `{tujuan}`: {galat}"),
                            interpreter.span_aktif,
                        )),
                    }
                }),
            ),
            (
                "pindah",
                fungsi("pindah", "pindah(asal, tujuan) -> kosong", |interpreter, args, _kwargs| {
                    butuh(interpreter, &args, 2, "pindah")?;
                    let asal = teks_arg(interpreter, &args[0], "asal")?;
                    let tujuan = teks_arg(interpreter, &args[1], "tujuan")?;
                    match std::fs::rename(&asal, &tujuan) {
                        Ok(()) => Ok(Nilai::Kosong),
                        Err(galat) => Err(interpreter.lempar_pesan(
                            "KesalahanBerkas",
                            format!("tidak dapat memindahkan `{asal}` ke `{tujuan}`: {galat}"),
                            interpreter.span_aktif,
                        )),
                    }
                }),
            ),
        ],
    )
}

// ------------------------------------------------------------------
// alat_daftar dan alat_kamus
// ------------------------------------------------------------------

fn alat_daftar(interpreter: &mut Interpreter) -> Rc<Modul> {
    let _ = interpreter;
    modul(
        "alat_daftar",
        "Pembantu operasi daftar.",
        vec![
            (
                "jumlah",
                fungsi("jumlah", "jumlah(daftar) -> bilangan", |interpreter, args, _kwargs| {
                    butuh(interpreter, &args, 1, "jumlah")?;
                    let item = iter_arg(interpreter, &args[0])?;
                    let mut total = Nilai::Bilangan(0);
                    for nilai in item {
                        total = interpreter.operasi_biner(
                            aksara_ast::BinOp::Add,
                            total,
                            nilai,
                            interpreter.span_aktif,
                        )?;
                    }
                    Ok(total)
                }),
            ),
            (
                "rata_rata",
                fungsi("rata_rata", "rata_rata(daftar) -> pecahan", |interpreter, args, _kwargs| {
                    butuh(interpreter, &args, 1, "rata_rata")?;
                    let item = iter_arg(interpreter, &args[0])?;
                    if item.is_empty() {
                        return galat(interpreter, "`rata_rata` tidak menerima daftar kosong");
                    }
                    let mut total = 0.0;
                    for nilai in &item {
                        total += angka_arg(interpreter, nilai, "item")?;
                    }
                    Ok(hasil_pecahan(total / item.len() as f64))
                }),
            ),
            (
                "unik",
                fungsi("unik", "unik(daftar) -> daftar", |interpreter, args, _kwargs| {
                    butuh(interpreter, &args, 1, "unik")?;
                    let item = iter_arg(interpreter, &args[0])?;
                    let mut hasil: Vec<Nilai> = Vec::new();
                    for nilai in item {
                        if !hasil.iter().any(|lama| interpreter.sama(lama, &nilai).unwrap_or(false)) {
                            hasil.push(nilai);
                        }
                    }
                    Ok(hasil_daftar(hasil))
                }),
            ),
            (
                "ratakan",
                fungsi("ratakan", "ratakan(daftar, kedalaman=1) -> daftar", |interpreter, args, _kwargs| {
                    butuh(interpreter, &args, 1, "ratakan")?;
                    let item = iter_arg(interpreter, &args[0])?;
                    let mut hasil = Vec::new();
                    for nilai in item {
                        match nilai {
                            Nilai::Daftar(isi) => hasil.extend(isi.borrow().iter().cloned()),
                            Nilai::Tupel(isi) => hasil.extend(isi.iter().cloned()),
                            lain => hasil.push(lain),
                        }
                    }
                    Ok(hasil_daftar(hasil))
                }),
            ),
            (
                "kelompok",
                fungsi("kelompok", "kelompok(daftar, ukuran) -> daftar", |interpreter, args, _kwargs| {
                    butuh(interpreter, &args, 2, "kelompok")?;
                    let item = iter_arg(interpreter, &args[0])?;
                    let ukuran = bulat_arg(interpreter, &args[1], "ukuran")?;
                    if ukuran <= 0 {
                        return galat(interpreter, "ukuran kelompok harus positif");
                    }
                    let hasil = item
                        .chunks(ukuran as usize)
                        .map(|bagian| Nilai::tupel(bagian.to_vec()))
                        .collect();
                    Ok(hasil_daftar(hasil))
                }),
            ),
            (
                "gabung",
                fungsi("gabung", "gabung(*daftar) -> daftar", |interpreter, args, _kwargs| {
                    let mut hasil = Vec::new();
                    for nilai in &args {
                        hasil.extend(iter_arg(interpreter, nilai)?);
                    }
                    Ok(hasil_daftar(hasil))
                }),
            ),
            (
                "urutkan",
                fungsi("urutkan", "urutkan(daftar, kunci=None, terbalik=False) -> daftar", |interpreter, args, kwargs| {
                    butuh(interpreter, &args, 1, "urutkan")?;
                    let mut item = iter_arg(interpreter, &args[0])?;
                    let kunci = args.get(1).cloned().or_else(|| opsi(&kwargs, "key").cloned());
                    let terbalik = match args.get(2) {
                        Some(nilai) => interpreter.kebenaran(nilai)?,
                        None => match opsi(&kwargs, "reverse") {
                            Some(nilai) => interpreter.kebenaran(nilai)?,
                            None => false,
                        },
                    };
                    crate::bawaan::urutkan_nilai(interpreter, &mut item, &kunci, terbalik)?;
                    Ok(hasil_daftar(item))
                }),
            ),
            (
                "indeks",
                fungsi("indeks", "indeks(daftar, nilai) -> bilangan", |interpreter, args, _kwargs| {
                    butuh(interpreter, &args, 2, "indeks")?;
                    let item = iter_arg(interpreter, &args[0])?;
                    for (i, nilai) in item.iter().enumerate() {
                        if interpreter.sama(nilai, &args[1])? {
                            return Ok(hasil_bilangan(i as i64));
                        }
                    }
                    Ok(hasil_bilangan(-1))
                }),
            ),
        ],
    )
}

fn alat_kamus(interpreter: &mut Interpreter) -> Rc<Modul> {
    let _ = interpreter;
    modul(
        "alat_kamus",
        "Pembantu operasi kamus.",
        vec![
            (
                "kunci",
                fungsi("kunci", "kunci(kamus) -> daftar", |interpreter, args, _kwargs| {
                    butuh(interpreter, &args, 1, "kunci")?;
                    match &args[0] {
                        Nilai::Kamus(pasangan) => Ok(hasil_daftar(
                            pasangan.borrow().iter().map(|(kunci, _)| kunci.clone()).collect(),
                        )),
                        lain => galat(interpreter, format!("`kunci` membutuhkan kamus, ditemukan {}", lain.nama_tipe())),
                    }
                }),
            ),
            (
                "nilai",
                fungsi("nilai", "nilai(kamus) -> daftar", |interpreter, args, _kwargs| {
                    butuh(interpreter, &args, 1, "nilai")?;
                    match &args[0] {
                        Nilai::Kamus(pasangan) => Ok(hasil_daftar(
                            pasangan.borrow().iter().map(|(_, nilai)| nilai.clone()).collect(),
                        )),
                        lain => galat(interpreter, format!("`nilai` membutuhkan kamus, ditemukan {}", lain.nama_tipe())),
                    }
                }),
            ),
            (
                "ambil",
                fungsi("ambil", "ambil(kamus, kunci, bawaan=None)", |interpreter, args, _kwargs| {
                    butuh(interpreter, &args, 2, "ambil")?;
                    match &args[0] {
                        Nilai::Kamus(pasangan) => {
                            for (kunci, nilai) in pasangan.borrow().iter() {
                                if interpreter.sama(kunci, &args[1])? {
                                    return Ok(nilai.clone());
                                }
                            }
                            Ok(args.get(2).cloned().unwrap_or(Nilai::Kosong))
                        }
                        lain => galat(interpreter, format!("`ambil` membutuhkan kamus, ditemukan {}", lain.nama_tipe())),
                    }
                }),
            ),
            (
                "gabung",
                fungsi("gabung", "gabung(*kamus) -> kamus", |interpreter, args, _kwargs| {
                    let mut hasil = Vec::new();
                    for nilai in &args {
                        if let Nilai::Kamus(pasangan) = nilai {
                            for (kunci, nilai) in pasangan.borrow().iter() {
                                interpreter.atur_kamus(&mut hasil, kunci.clone(), nilai.clone());
                            }
                        }
                    }
                    Ok(Nilai::kamus(hasil))
                }),
            ),
            (
                "balik",
                fungsi("balik", "balik(kamus) -> kamus", |interpreter, args, _kwargs| {
                    butuh(interpreter, &args, 1, "balik")?;
                    match &args[0] {
                        Nilai::Kamus(pasangan) => {
                            let mut hasil = Vec::new();
                            for (kunci, nilai) in pasangan.borrow().iter() {
                                interpreter.atur_kamus(&mut hasil, nilai.clone(), kunci.clone());
                            }
                            Ok(Nilai::kamus(hasil))
                        }
                        lain => galat(interpreter, format!("`balik` membutuhkan kamus, ditemukan {}", lain.nama_tipe())),
                    }
                }),
            ),
            (
                "dari_kunci",
                fungsi("dari_kunci", "dari_kunci(kunci, nilai=None) -> kamus", |interpreter, args, _kwargs| {
                    butuh(interpreter, &args, 1, "dari_kunci")?;
                    let kunci = iter_arg(interpreter, &args[0])?;
                    let nilai = args.get(1).cloned().unwrap_or(Nilai::Kosong);
                    let mut hasil = Vec::new();
                    for kunci in kunci {
                        interpreter.atur_kamus(&mut hasil, kunci, nilai.clone());
                    }
                    Ok(Nilai::kamus(hasil))
                }),
            ),
            (
                "saring",
                fungsi("saring", "saring(kamus, predikat) -> kamus", |interpreter, args, _kwargs| {
                    butuh(interpreter, &args, 2, "saring")?;
                    let Nilai::Kamus(pasangan) = &args[0] else {
                        return galat(interpreter, "`saring` membutuhkan kamus");
                    };
                    let predikat = args[1].clone();
                    let mut hasil = Vec::new();
                    for (kunci, nilai) in pasangan.borrow().iter() {
                        let lolos = interpreter.panggil(
                            &predikat,
                            vec![kunci.clone(), nilai.clone()],
                            Vec::new(),
                            interpreter.span_aktif,
                        )?;
                        if interpreter.kebenaran(&lolos)? {
                            interpreter.atur_kamus(&mut hasil, kunci.clone(), nilai.clone());
                        }
                    }
                    Ok(Nilai::kamus(hasil))
                }),
            ),
        ],
    )
}

// ------------------------------------------------------------------
// alat_iterasi
// ------------------------------------------------------------------

fn alat_iterasi(interpreter: &mut Interpreter) -> Rc<Modul> {
    let _ = interpreter;
    modul(
        "alat_iterasi",
        "Pembantu iterasi.",
        vec![
            (
                "rantai",
                fungsi("rantai", "rantai(*iterable) -> generator", |interpreter, args, _kwargs| {
                    let mut hasil = Vec::new();
                    for nilai in &args {
                        hasil.extend(iter_arg(interpreter, nilai)?);
                    }
                    Ok(hasil_generator("rantai", hasil))
                }),
            ),
            (
                "ulangi",
                fungsi("ulangi", "ulangi(nilai, jumlah) -> generator", |interpreter, args, _kwargs| {
                    butuh(interpreter, &args, 2, "ulangi")?;
                    let jumlah = bulat_arg(interpreter, &args[1], "jumlah")?;
                    if jumlah < 0 {
                        return galat(interpreter, "jumlah pengulangan tidak boleh negatif");
                    }
                    Ok(hasil_generator("ulangi", vec![args[0].clone(); jumlah as usize]))
                }),
            ),
            (
                "kombinasi",
                fungsi("kombinasi", "kombinasi(iterable, r) -> generator", |interpreter, args, _kwargs| {
                    butuh(interpreter, &args, 2, "kombinasi")?;
                    let item = iter_arg(interpreter, &args[0])?;
                    let r = bulat_arg(interpreter, &args[1], "r")?;
                    if r < 0 || r as usize > item.len() {
                        return Ok(hasil_generator("kombinasi", Vec::new()));
                    }
                    let mut hasil = Vec::new();
                    kombinasi_rekursif(&item, r as usize, 0, &mut Vec::new(), &mut hasil);
                    Ok(hasil_generator("kombinasi", hasil))
                }),
            ),
            (
                "permutasi",
                fungsi("permutasi", "permutasi(iterable, r=None) -> generator", |interpreter, args, _kwargs| {
                    butuh(interpreter, &args, 1, "permutasi")?;
                    let item = iter_arg(interpreter, &args[0])?;
                    let r = match args.get(1) {
                        Some(nilai) => bulat_arg(interpreter, nilai, "r")?,
                        None => item.len() as i64,
                    };
                    if r < 0 || r as usize > item.len() {
                        return Ok(hasil_generator("permutasi", Vec::new()));
                    }
                    let mut hasil = Vec::new();
                    permutasi_rekursif(&item, r as usize, &mut Vec::new(), &mut hasil);
                    Ok(hasil_generator("permutasi", hasil))
                }),
            ),
            (
                "produk",
                fungsi("produk", "produk(*iterable, ulang=1) -> generator", |interpreter, args, kwargs| {
                    let mut sumber: Vec<Vec<Nilai>> = Vec::new();
                    for nilai in &args {
                        sumber.push(iter_arg(interpreter, nilai)?);
                    }
                    let ulang = match opsi(&kwargs, "ulang").or_else(|| opsi(&kwargs, "repeat")) {
                        Some(nilai) => bulat_arg(interpreter, nilai, "ulang")?,
                        None => 1,
                    };
                    if ulang < 0 {
                        return galat(interpreter, "ulang tidak boleh negatif");
                    }
                    let dasar = sumber.clone();
                    for _ in 1..ulang {
                        sumber.extend(dasar.iter().cloned());
                    }
                    let mut hasil = Vec::new();
                    produk_rekursif(&sumber, 0, &mut Vec::new(), &mut hasil);
                    Ok(hasil_generator("produk", hasil))
                }),
            ),
            (
                "potong",
                fungsi("potong", "potong(iterable, mulai, akhir=None) -> generator", |interpreter, args, _kwargs| {
                    butuh(interpreter, &args, 2, "potong")?;
                    let item = iter_arg(interpreter, &args[0])?;
                    let mulai = bulat_arg(interpreter, &args[1], "mulai")?.max(0) as usize;
                    let akhir = match args.get(2) {
                        Some(nilai) => bulat_arg(interpreter, nilai, "akhir")?.max(0) as usize,
                        None => item.len(),
                    };
                    let mulai = mulai.min(item.len());
                    let akhir = akhir.min(item.len());
                    let hasil = if mulai < akhir { item[mulai..akhir].to_vec() } else { Vec::new() };
                    Ok(hasil_generator("potong", hasil))
                }),
            ),
            (
                "akumulasi",
                fungsi("akumulasi", "akumulasi(iterable, fungsi=None, awal=None) -> generator", |interpreter, args, kwargs| {
                    butuh(interpreter, &args, 1, "akumulasi")?;
                    let item = iter_arg(interpreter, &args[0])?;
                    let fungsi = args.get(1).cloned().or_else(|| opsi(&kwargs, "func").cloned());
                    let mut total = match opsi(&kwargs, "awal").or_else(|| opsi(&kwargs, "initial")) {
                        Some(nilai) => nilai.clone(),
                        None => match item.first() {
                            Some(nilai) => nilai.clone(),
                            None => return Ok(hasil_generator("akumulasi", Vec::new())),
                        },
                    };
                    let mulai = if opsi(&kwargs, "awal").or_else(|| opsi(&kwargs, "initial")).is_some() { 0 } else { 1 };
                    let mut hasil = vec![total.clone()];
                    for nilai in item.iter().skip(mulai) {
                        total = match &fungsi {
                            Some(fungsi) => interpreter.panggil(
                                fungsi,
                                vec![total, nilai.clone()],
                                Vec::new(),
                                interpreter.span_aktif,
                            )?,
                            None => interpreter.operasi_biner(
                                aksara_ast::BinOp::Add,
                                total,
                                nilai.clone(),
                                interpreter.span_aktif,
                            )?,
                        };
                        hasil.push(total.clone());
                    }
                    Ok(hasil_generator("akumulasi", hasil))
                }),
            ),
            (
                "saring_palsu",
                fungsi("saring_palsu", "saring_palsu(iterable) -> generator", |interpreter, args, _kwargs| {
                    butuh(interpreter, &args, 1, "saring_palsu")?;
                    let mut hasil = Vec::new();
                    for nilai in iter_arg(interpreter, &args[0])? {
                        if !interpreter.kebenaran(&nilai)? {
                            hasil.push(nilai);
                        }
                    }
                    Ok(hasil_generator("saring_palsu", hasil))
                }),
            ),
            (
                "pasangan",
                fungsi("pasangan", "pasangan(iterable) -> generator", |interpreter, args, _kwargs| {
                    butuh(interpreter, &args, 1, "pasangan")?;
                    let item = iter_arg(interpreter, &args[0])?;
                    let hasil = item
                        .windows(2)
                        .map(|pasangan| Nilai::tupel(pasangan.to_vec()))
                        .collect();
                    Ok(hasil_generator("pasangan", hasil))
                }),
            ),
        ],
    )
}

fn kombinasi_rekursif(
    item: &[Nilai],
    r: usize,
    mulai: usize,
    sekarang: &mut Vec<Nilai>,
    hasil: &mut Vec<Nilai>,
) {
    if sekarang.len() == r {
        hasil.push(Nilai::tupel(sekarang.clone()));
        return;
    }
    for i in mulai..item.len() {
        sekarang.push(item[i].clone());
        kombinasi_rekursif(item, r, i + 1, sekarang, hasil);
        sekarang.pop();
    }
}

fn permutasi_rekursif(
    item: &[Nilai],
    r: usize,
    sekarang: &mut Vec<Nilai>,
    hasil: &mut Vec<Nilai>,
) {
    fn dalam(
        item: &[Nilai],
        r: usize,
        dipakai: &mut Vec<bool>,
        sekarang: &mut Vec<Nilai>,
        hasil: &mut Vec<Nilai>,
    ) {
        if sekarang.len() == r {
            hasil.push(Nilai::tupel(sekarang.clone()));
            return;
        }
        for i in 0..item.len() {
            if dipakai[i] {
                continue;
            }
            dipakai[i] = true;
            sekarang.push(item[i].clone());
            dalam(item, r, dipakai, sekarang, hasil);
            sekarang.pop();
            dipakai[i] = false;
        }
    }
    let mut dipakai = vec![false; item.len()];
    dalam(item, r, &mut dipakai, sekarang, hasil);
}

fn produk_rekursif(
    sumber: &[Vec<Nilai>],
    indeks: usize,
    sekarang: &mut Vec<Nilai>,
    hasil: &mut Vec<Nilai>,
) {
    if indeks == sumber.len() {
        hasil.push(Nilai::tupel(sekarang.clone()));
        return;
    }
    for nilai in &sumber[indeks] {
        sekarang.push(nilai.clone());
        produk_rekursif(sumber, indeks + 1, sekarang, hasil);
        sekarang.pop();
    }
}

// ------------------------------------------------------------------
// alat_fungsi
// ------------------------------------------------------------------

fn alat_fungsi(interpreter: &mut Interpreter) -> Rc<Modul> {
    let _ = interpreter;
    modul(
        "alat_fungsi",
        "Pembantu fungsi tingkat tinggi.",
        vec![
            (
                "kurangi",
                fungsi("kurangi", "kurangi(fungsi, iterable, awal=None)", |interpreter, args, _kwargs| {
                    butuh(interpreter, &args, 2, "kurangi")?;
                    let fungsi = args[0].clone();
                    let item = iter_arg(interpreter, &args[1])?;
                    let (mut total, mulai) = match args.get(2) {
                        Some(nilai) => (nilai.clone(), 0),
                        None => match item.first() {
                            Some(nilai) => (nilai.clone(), 1),
                            None => return galat(interpreter, "`kurangi` tidak menerima iterable kosong tanpa awal"),
                        },
                    };
                    for nilai in item.iter().skip(mulai) {
                        total = interpreter.panggil(
                            &fungsi,
                            vec![total, nilai.clone()],
                            Vec::new(),
                            interpreter.span_aktif,
                        )?;
                    }
                    Ok(total)
                }),
            ),
            (
                "parsial",
                fungsi("parsial", "parsial(fungsi, *args, **kwargs) -> fungsi", |interpreter, args, kwargs| {
                    butuh(interpreter, &args, 1, "parsial")?;
                    let target = args[0].clone();
                    let awal = args[1..].to_vec();
                    let awal_kwargs = kwargs;
                    Ok(fungsi("parsial", "fungsi parsial", move |interpreter, args2, kwargs2| {
                        let mut semua = awal.clone();
                        semua.extend(args2);
                        let mut semua_kwargs = awal_kwargs.clone();
                        semua_kwargs.extend(kwargs2);
                        interpreter.panggil(&target, semua, semua_kwargs, interpreter.span_aktif)
                    }))
                }),
            ),
            (
                "cache",
                fungsi("cache", "cache(fungsi) -> fungsi", |interpreter, args, _kwargs| {
                    butuh(interpreter, &args, 1, "cache")?;
                    let target = args[0].clone();
                    let cache: Rc<RefCell<HashMap<String, Nilai>>> = Rc::new(RefCell::new(HashMap::new()));
                    Ok(fungsi("cache", "fungsi dengan cache", move |interpreter, args2, kwargs2| {
                        let kunci = format!(
                            "{}|{}",
                            args2.iter().map(nilai_ke_repr).collect::<Vec<_>>().join(","),
                            kwargs2.iter().map(|(nama, nilai)| format!("{nama}={}", nilai_ke_repr(nilai))).collect::<Vec<_>>().join(",")
                        );
                        if let Some(nilai) = cache.borrow().get(&kunci) {
                            return Ok(nilai.clone());
                        }
                        let nilai = interpreter.panggil(&target, args2, kwargs2, interpreter.span_aktif)?;
                        cache.borrow_mut().insert(kunci, nilai.clone());
                        Ok(nilai)
                    }))
                }),
            ),
            (
                "identitas",
                fungsi("identitas", "identitas(nilai) -> nilai", |_interpreter, args, _kwargs| {
                    Ok(args.first().cloned().unwrap_or(Nilai::Kosong))
                }),
            ),
            (
                "total_ordering",
                fungsi("total_ordering", "total_ordering(kelas) -> kelas", |_interpreter, args, _kwargs| {
                    Ok(args.first().cloned().unwrap_or(Nilai::Kosong))
                }),
            ),
            (
                "wraps",
                fungsi("wraps", "wraps(fungsi) -> dekorator", |_interpreter, args, _kwargs| {
                    let target = args.first().cloned().unwrap_or(Nilai::Kosong);
                    Ok(fungsi("wraps", "dekorator wraps", move |_interpreter, args2, _kwargs2| {
                        let _ = &target;
                        Ok(args2.first().cloned().unwrap_or(Nilai::Kosong))
                    }))
                }),
            ),
        ],
    )
}

// ------------------------------------------------------------------
// sidik
// ------------------------------------------------------------------

fn sidik() -> Rc<Modul> {
    use md5::Digest as _;
    use sha1::Sha1;
    use sha2::{Sha256, Sha512};

    fn hex(bytes: &[u8]) -> String {
        bytes.iter().map(|byte| format!("{byte:02x}")).collect()
    }

    modul(
        "sidik",
        "Fungsi sidik kriptografis.",
        vec![
            (
                "md5",
                fungsi("md5", "md5(data) -> teks", |interpreter, args, _kwargs| {
                    butuh(interpreter, &args, 1, "md5")?;
                    let data = bytes_arg(interpreter, &args[0])?;
                    let hasil = md5::Md5::digest(&data);
                    Ok(hasil_teks(hex(&hasil)))
                }),
            ),
            (
                "sha1",
                fungsi("sha1", "sha1(data) -> teks", |interpreter, args, _kwargs| {
                    butuh(interpreter, &args, 1, "sha1")?;
                    let data = bytes_arg(interpreter, &args[0])?;
                    let hasil = Sha1::digest(&data);
                    Ok(hasil_teks(hex(&hasil)))
                }),
            ),
            (
                "sha256",
                fungsi("sha256", "sha256(data) -> teks", |interpreter, args, _kwargs| {
                    butuh(interpreter, &args, 1, "sha256")?;
                    let data = bytes_arg(interpreter, &args[0])?;
                    let hasil = Sha256::digest(&data);
                    Ok(hasil_teks(hex(&hasil)))
                }),
            ),
            (
                "sha512",
                fungsi("sha512", "sha512(data) -> teks", |interpreter, args, _kwargs| {
                    butuh(interpreter, &args, 1, "sha512")?;
                    let data = bytes_arg(interpreter, &args[0])?;
                    let hasil = Sha512::digest(&data);
                    Ok(hasil_teks(hex(&hasil)))
                }),
            ),
        ],
    )
}

fn bytes_arg(interpreter: &mut Interpreter, nilai: &Nilai) -> Hasil<Vec<u8>> {
    match nilai {
        Nilai::Teks(teks) => Ok(teks.as_bytes().to_vec()),
        Nilai::Bytes(bytes) => Ok(bytes.as_ref().clone()),
        lain => galat(
            interpreter,
            format!("argumen harus teks atau bytes, ditemukan {}", lain.nama_tipe()),
        ),
    }
}

// ------------------------------------------------------------------
// alur_asinkron
// ------------------------------------------------------------------

fn alur_asinkron() -> Rc<Modul> {
    modul(
        "alur_asinkron",
        "Pembantu eksekusi asinkron eager.",
        vec![
            (
                "jalankan",
                fungsi("jalankan", "jalankan(nilai) -> nilai", |interpreter, args, _kwargs| {
                    butuh(interpreter, &args, 1, "jalankan")?;
                    match &args[0] {
                        Nilai::Python(_) => crate::python_tunggu(&args[0], interpreter.span_aktif),
                        nilai => Ok(nilai.clone()),
                    }
                }),
            ),
            (
                "tidur",
                fungsi("tidur", "tidur(detik) -> kosong", |interpreter, args, _kwargs| {
                    butuh(interpreter, &args, 1, "tidur")?;
                    let detik = angka_arg(interpreter, &args[0], "detik")?;
                    if detik > 0.0 {
                        std::thread::sleep(Duration::from_secs_f64(detik));
                    }
                    Ok(Nilai::Kosong)
                }),
            ),
            (
                "kumpulkan",
                fungsi("kumpulkan", "kumpulkan(iterable) -> daftar", |interpreter, args, _kwargs| {
                    butuh(interpreter, &args, 1, "kumpulkan")?;
                    Ok(hasil_daftar(iter_arg(interpreter, &args[0])?))
                }),
            ),
            (
                "selesai",
                fungsi("selesai", "selesai(nilai) -> nilai", |_interpreter, args, _kwargs| {
                    Ok(args.first().cloned().unwrap_or(Nilai::Kosong))
                }),
            ),
            (
                "tunggu",
                fungsi("tunggu", "tunggu(nilai) -> nilai", |interpreter, args, _kwargs| {
                    butuh(interpreter, &args, 1, "tunggu")?;
                    match &args[0] {
                        Nilai::Python(_) => crate::python_tunggu(&args[0], interpreter.span_aktif),
                        nilai => Ok(nilai.clone()),
                    }
                }),
            ),
        ],
    )
}

// ------------------------------------------------------------------
// sistem_tipe, kelas_data, tipe_pilihan, abstrak
// ------------------------------------------------------------------

fn sistem_tipe(interpreter: &mut Interpreter) -> Rc<Modul> {
    let _ = interpreter;
    modul(
        "sistem_tipe",
        "Pembantu anotasi tipe.",
        vec![
            (
                "apakah",
                fungsi("apakah", "apakah(nilai, kelas) -> logika", |interpreter, args, _kwargs| {
                    butuh(interpreter, &args, 2, "apakah")?;
                    let kelas = args[1].clone();
                    let cocok = crate::bawaan::isinstance_nilai(interpreter, &args[0], &kelas)?;
                    Ok(hasil_logika(cocok))
                }),
            ),
            (
                "opsional",
                fungsi("opsional", "opsional(kelas) -> tipe", |_interpreter, args, _kwargs| {
                    Ok(hasil_teks(format!("opsional<{}>", args.first().map(nilai_ke_teks).unwrap_or_else(|| "kosong".to_string()))))
                }),
            ),
            (
                "gabungan",
                fungsi("gabungan", "gabungan(*kelas) -> tipe", |_interpreter, args, _kwargs| {
                    Ok(hasil_teks(format!(
                        "gabungan<{}>",
                        args.iter().map(nilai_ke_teks).collect::<Vec<_>>().join(", ")
                    )))
                }),
            ),
            (
                "daftar",
                fungsi("daftar", "daftar(kelas) -> tipe", |_interpreter, args, _kwargs| {
                    Ok(hasil_teks(format!("daftar<{}>", args.first().map(nilai_ke_teks).unwrap_or_default())))
                }),
            ),
            (
                "kamus",
                fungsi("kamus", "kamus(kunci, nilai) -> tipe", |_interpreter, args, _kwargs| {
                    Ok(hasil_teks(format!(
                        "kamus<{}, {}>",
                        args.first().map(nilai_ke_teks).unwrap_or_default(),
                        args.get(1).map(nilai_ke_teks).unwrap_or_default()
                    )))
                }),
            ),
            (
                "apapun",
                fungsi("apapun", "apapun() -> tipe", |_interpreter, _args, _kwargs| {
                    Ok(hasil_teks("apapun"))
                }),
            ),
            (
                "tidak_ada",
                fungsi("tidak_ada", "tidak_ada() -> tipe", |_interpreter, _args, _kwargs| {
                    Ok(Nilai::Kosong)
                }),
            ),
            (
                "cast",
                fungsi("cast", "cast(tipe, nilai) -> nilai", |_interpreter, args, _kwargs| {
                    Ok(args.get(1).cloned().unwrap_or(Nilai::Kosong))
                }),
            ),
            (
                "get_type_hints",
                fungsi("get_type_hints", "get_type_hints(objek) -> kamus", |_interpreter, _args, _kwargs| {
                    Ok(Nilai::kamus(Vec::new()))
                }),
            ),
        ],
    )
}

fn kelas_data(interpreter: &mut Interpreter) -> Rc<Modul> {
    let _ = interpreter;
    modul(
        "kelas_data",
        "Pembantu kelas data.",
        vec![
            (
                "kelas_data",
                fungsi("kelas_data", "kelas_data(kelas) -> kelas", |interpreter, args, _kwargs| {
                    butuh(interpreter, &args, 1, "kelas_data")?;
                    let Nilai::Kelas(kelas) = &args[0] else {
                        return galat(interpreter, "`kelas_data` membutuhkan kelas");
                    };
                    crate::modul_data::lengkapi_kelas_data(interpreter, kelas)?;
                    Ok(Nilai::Kelas(kelas.clone()))
                }),
            ),
            (
                "dataclass",
                fungsi("dataclass", "dataclass(kelas) -> kelas", |interpreter, args, _kwargs| {
                    butuh(interpreter, &args, 1, "dataclass")?;
                    let Nilai::Kelas(kelas) = &args[0] else {
                        return galat(interpreter, "`dataclass` membutuhkan kelas");
                    };
                    crate::modul_data::lengkapi_kelas_data(interpreter, kelas)?;
                    Ok(Nilai::Kelas(kelas.clone()))
                }),
            ),
            (
                "bidang",
                fungsi("bidang", "bidang(bawaan=None) -> nilai", |_interpreter, args, _kwargs| {
                    Ok(args.first().cloned().unwrap_or(Nilai::Kosong))
                }),
            ),
            (
                "field",
                fungsi("field", "field(default=None) -> nilai", |_interpreter, args, _kwargs| {
                    Ok(args.first().cloned().unwrap_or(Nilai::Kosong))
                }),
            ),
            (
                "fields",
                fungsi("fields", "fields(kelas) -> daftar", |interpreter, args, _kwargs| {
                    butuh(interpreter, &args, 1, "fields")?;
                    let Nilai::Kelas(kelas) = &args[0] else {
                        return galat(interpreter, "`fields` membutuhkan kelas");
                    };
                    let anotasi = kelas.atribut.borrow().get("__anotasi__").cloned();
                    match anotasi {
                        Some(Nilai::Kamus(pasangan)) => Ok(hasil_daftar(
                            pasangan.borrow().iter().map(|(nama, _)| nama.clone()).collect(),
                        )),
                        _ => Ok(hasil_daftar(Vec::new())),
                    }
                }),
            ),
        ],
    )
}

fn tipe_pilihan(interpreter: &mut Interpreter) -> Rc<Modul> {
    let enum_kelas = kelas_modul(interpreter, "objek").map(|objek| Kelas::baru("Enum", vec![objek]));
    let mut isi = vec![
        (
            "otomatis",
            fungsi("otomatis", "otomatis() -> bilangan", |_interpreter, _args, _kwargs| {
                Ok(hasil_bilangan(1))
            }),
        ),
        (
            "buat",
            fungsi("buat", "buat(nama, anggota) -> kelas", |interpreter, args, _kwargs| {
                butuh(interpreter, &args, 2, "buat")?;
                let nama = teks_arg(interpreter, &args[0], "nama")?;
                let anggota = iter_arg(interpreter, &args[1])?;
                let basis = kelas_modul(interpreter, "objek").into_iter().collect();
                let kelas = Kelas::baru(nama, basis);
                for (indeks, anggota) in anggota.iter().enumerate() {
                    let nama_anggota = match anggota {
                        Nilai::Tupel(item) if item.len() == 2 => nilai_ke_teks(&item[0]),
                        lain => nilai_ke_teks(lain),
                    };
                    let nilai = match anggota {
                        Nilai::Tupel(item) if item.len() == 2 => item[1].clone(),
                        _ => hasil_bilangan(indeks as i64 + 1),
                    };
                    let instance = Rc::new(Instance {
                        kelas: kelas.clone(),
                        atribut: RefCell::new(HashMap::new()),
                    });
                    instance.atribut.borrow_mut().insert("nama".to_string(), hasil_teks(nama_anggota.clone()));
                    instance.atribut.borrow_mut().insert("nilai".to_string(), nilai);
                    kelas.atribut.borrow_mut().insert(nama_anggota, Nilai::Instance(instance));
                }
                Ok(Nilai::Kelas(kelas))
            }),
        ),
    ];
    if let Some(enum_kelas) = enum_kelas {
        isi.push(("Enum", Nilai::Kelas(enum_kelas.clone())));
        isi.push(("IntEnum", Nilai::Kelas(enum_kelas)));
    }
    modul("tipe_pilihan", "Pembantu tipe pilihan (enum).", isi)
}

fn abstrak(interpreter: &mut Interpreter) -> Rc<Modul> {
    let abc = kelas_modul(interpreter, "objek").map(|objek| Kelas::baru("ABC", vec![objek]));
    let mut isi = vec![
        (
            "metode_abstrak",
            fungsi("metode_abstrak", "metode_abstrak(fungsi) -> fungsi", |_interpreter, args, _kwargs| {
                Ok(args.first().cloned().unwrap_or(Nilai::Kosong))
            }),
        ),
        (
            "abstractmethod",
            fungsi("abstractmethod", "abstractmethod(fungsi) -> fungsi", |_interpreter, args, _kwargs| {
                Ok(args.first().cloned().unwrap_or(Nilai::Kosong))
            }),
        ),
        (
            "kelas_abstrak",
            fungsi("kelas_abstrak", "kelas_abstrak(kelas) -> kelas", |_interpreter, args, _kwargs| {
                Ok(args.first().cloned().unwrap_or(Nilai::Kosong))
            }),
        ),
    ];
    if let Some(abc) = abc {
        isi.push(("ABC", Nilai::Kelas(abc.clone())));
        isi.push(("ABCMeta", Nilai::Kelas(abc)));
    }
    modul("abstrak", "Pembantu kelas abstrak.", isi)
}

// ------------------------------------------------------------------
// waktu, acak, sistem
// ------------------------------------------------------------------

fn waktu() -> Rc<Modul> {
    let awal = Instant::now();
    modul(
        "waktu",
        "Fungsi waktu.",
        vec![
            (
                "waktu",
                fungsi("waktu", "waktu() -> pecahan", |_interpreter, _args, _kwargs| {
                    let detik = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default();
                    Ok(hasil_pecahan(detik.as_secs_f64()))
                }),
            ),
            (
                "monotonik",
                fungsi("monotonik", "monotonik() -> pecahan", move |_interpreter, _args, _kwargs| {
                    Ok(hasil_pecahan(awal.elapsed().as_secs_f64()))
                }),
            ),
            (
                "perf_counter",
                fungsi("perf_counter", "perf_counter() -> pecahan", move |_interpreter, _args, _kwargs| {
                    Ok(hasil_pecahan(awal.elapsed().as_secs_f64()))
                }),
            ),
            (
                "tidur",
                fungsi("tidur", "tidur(detik) -> kosong", |interpreter, args, _kwargs| {
                    butuh(interpreter, &args, 1, "tidur")?;
                    let detik = angka_arg(interpreter, &args[0], "detik")?;
                    if detik > 0.0 {
                        std::thread::sleep(Duration::from_secs_f64(detik));
                    }
                    Ok(Nilai::Kosong)
                }),
            ),
            (
                "struktur_waktu",
                fungsi("struktur_waktu", "struktur_waktu(detik=None) -> kamus", |interpreter, args, _kwargs| {
                    let detik = match args.first() {
                        Some(nilai) => angka_arg(interpreter, nilai, "detik")?,
                        None => SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs_f64(),
                    };
                    Ok(hasil_kamus(vec![
                        ("detik".to_string(), hasil_pecahan(detik)),
                        ("milidetik".to_string(), hasil_bilangan((detik * 1000.0) as i64)),
                    ]))
                }),
            ),
        ],
    )
}

fn acak() -> Rc<Modul> {
    let benih: Rc<RefCell<u64>> = Rc::new(RefCell::new(0x9E37_79B9_7F4A_7C15));
    let berikut = {
        let benih = benih.clone();
        move || {
            let mut nilai = benih.borrow_mut();
            *nilai ^= *nilai << 13;
            *nilai ^= *nilai >> 7;
            *nilai ^= *nilai << 17;
            *nilai
        }
    };
    let berikut2 = berikut.clone();
    let berikut3 = berikut.clone();
    let berikut4 = berikut.clone();
    let berikut5 = berikut.clone();
    modul(
        "acak",
        "Pembantu bilangan acak deterministik.",
        vec![
            (
                "benih",
                fungsi("benih", "benih(n) -> kosong", move |interpreter, args, _kwargs| {
                    butuh(interpreter, &args, 1, "benih")?;
                    let nilai = bulat_arg(interpreter, &args[0], "benih")?;
                    *benih.borrow_mut() = nilai as u64;
                    Ok(Nilai::Kosong)
                }),
            ),
            (
                "pecahan",
                fungsi("pecahan", "pecahan() -> pecahan", move |_interpreter, _args, _kwargs| {
                    let nilai = berikut();
                    Ok(hasil_pecahan((nilai >> 11) as f64 / (1u64 << 53) as f64))
                }),
            ),
            (
                "bulat",
                fungsi("bulat", "bulat(a, b) -> bilangan", move |interpreter, args, _kwargs| {
                    butuh(interpreter, &args, 2, "bulat")?;
                    let a = bulat_arg(interpreter, &args[0], "a")?;
                    let b = bulat_arg(interpreter, &args[1], "b")?;
                    if a >= b {
                        return galat(interpreter, "batas `bulat` tidak valid");
                    }
                    let nilai = berikut2();
                    Ok(hasil_bilangan(a + (nilai % (b - a) as u64) as i64))
                }),
            ),
            (
                "pilihan",
                fungsi("pilihan", "pilihan(urutan) -> nilai", move |interpreter, args, _kwargs| {
                    butuh(interpreter, &args, 1, "pilihan")?;
                    let item = iter_arg(interpreter, &args[0])?;
                    if item.is_empty() {
                        return galat(interpreter, "`pilihan` tidak menerima urutan kosong");
                    }
                    let nilai = berikut3();
                    Ok(item[(nilai % item.len() as u64) as usize].clone())
                }),
            ),
            (
                "kocok",
                fungsi("kocok", "kocok(daftar) -> daftar", move |interpreter, args, _kwargs| {
                    butuh(interpreter, &args, 1, "kocok")?;
                    let mut item = iter_arg(interpreter, &args[0])?;
                    for i in (1..item.len()).rev() {
                        let nilai = berikut4();
                        let j = (nilai % (i as u64 + 1)) as usize;
                        item.swap(i, j);
                    }
                    Ok(hasil_daftar(item))
                }),
            ),
            (
                "sampel",
                fungsi("sampel", "sampel(urutan, k) -> daftar", move |interpreter, args, _kwargs| {
                    butuh(interpreter, &args, 2, "sampel")?;
                    let mut item = iter_arg(interpreter, &args[0])?;
                    let k = bulat_arg(interpreter, &args[1], "k")?;
                    if k < 0 || k as usize > item.len() {
                        return galat(interpreter, "jumlah sampel tidak valid");
                    }
                    for i in (1..item.len()).rev() {
                        let nilai = berikut5();
                        let j = (nilai % (i as u64 + 1)) as usize;
                        item.swap(i, j);
                    }
                    item.truncate(k as usize);
                    Ok(hasil_daftar(item))
                }),
            ),
        ],
    )
}

fn sistem() -> Rc<Modul> {
    modul(
        "sistem",
        "Informasi sistem dan proses.",
        vec![
            ("versi", hasil_teks(env!("CARGO_PKG_VERSION"))),
            ("platform", hasil_teks(std::env::consts::OS)),
            ("arsitektur", hasil_teks(std::env::consts::ARCH)),
            (
                "argv",
                fungsi("argv", "argv() -> daftar", |_interpreter, _args, _kwargs| {
                    Ok(hasil_daftar(std::env::args().map(Nilai::teks).collect()))
                }),
            ),
            (
                "variabel",
                fungsi("variabel", "variabel(nama, bawaan=None)", |_interpreter, args, _kwargs| {
                    let nama = args.first().map(nilai_ke_teks).unwrap_or_default();
                    match std::env::var(&nama) {
                        Ok(nilai) => Ok(hasil_teks(nilai)),
                        Err(_) => Ok(args.get(1).cloned().unwrap_or(Nilai::Kosong)),
                    }
                }),
            ),
            (
                "direktori_kerja",
                fungsi("direktori_kerja", "direktori_kerja() -> teks", |interpreter, _args, _kwargs| {
                    match std::env::current_dir() {
                        Ok(jalur) => Ok(hasil_teks(jalur.to_string_lossy().to_string())),
                        Err(galat) => Err(interpreter.lempar_pesan(
                            "KesalahanBerkas",
                            format!("tidak dapat membaca direktori kerja: {galat}"),
                            interpreter.span_aktif,
                        )),
                    }
                }),
            ),
            (
                "keluar",
                fungsi("keluar", "keluar(kode=0)", |interpreter, args, _kwargs| {
                    let kode = match args.first() {
                        Some(nilai) => interpreter.nilai_ke_i64(nilai, interpreter.span_aktif)?,
                        None => 0,
                    };
                    Err(interpreter.lempar_pesan(
                        "KesalahanKeluar",
                        format!("program keluar dengan kode {kode}"),
                        interpreter.span_aktif,
                    ))
                }),
            ),
        ],
    )
}

// ------------------------------------------------------------------
// json
// ------------------------------------------------------------------

fn json_modul() -> Rc<Modul> {
    modul(
        "json",
        "Pembantu JSON.",
        vec![
            (
                "dumps",
                fungsi("dumps", "dumps(nilai, indent=None, ensure_ascii=True) -> teks", |interpreter, args, kwargs| {
                    butuh(interpreter, &args, 1, "dumps")?;
                    let indent = match opsi(&kwargs, "indent") {
                        Some(nilai) => interpreter.nilai_ke_i64(nilai, interpreter.span_aktif)?,
                        None => 0,
                    };
                    let hasil = ke_json(&args[0], indent.max(0) as usize, 0)?;
                    Ok(hasil_teks(hasil))
                }),
            ),
            (
                "loads",
                fungsi("loads", "loads(teks) -> nilai", |interpreter, args, _kwargs| {
                    butuh(interpreter, &args, 1, "loads")?;
                    let teks = teks_arg(interpreter, &args[0], "teks JSON")?;
                    match serde_json::from_str::<serde_json::Value>(&teks) {
                        Ok(nilai) => Ok(dari_json(&nilai)),
                        Err(galat) => Err(interpreter.lempar_pesan(
                            "KesalahanNilai",
                            format!("JSON tidak valid: {galat}"),
                            interpreter.span_aktif,
                        )),
                    }
                }),
            ),
            (
                "muat",
                fungsi("muat", "muat(jalur) -> nilai", |interpreter, args, _kwargs| {
                    butuh(interpreter, &args, 1, "muat")?;
                    let jalur = teks_arg(interpreter, &args[0], "jalur")?;
                    match std::fs::read_to_string(&jalur) {
                        Ok(teks) => match serde_json::from_str::<serde_json::Value>(&teks) {
                            Ok(nilai) => Ok(dari_json(&nilai)),
                            Err(galat) => Err(interpreter.lempar_pesan(
                                "KesalahanNilai",
                                format!("JSON tidak valid di `{jalur}`: {galat}"),
                                interpreter.span_aktif,
                            )),
                        },
                        Err(galat) => Err(interpreter.lempar_pesan(
                            "KesalahanBerkas",
                            format!("tidak dapat membaca `{jalur}`: {galat}"),
                            interpreter.span_aktif,
                        )),
                    }
                }),
            ),
            (
                "simpan",
                fungsi("simpan", "simpan(nilai, jalur, indent=None) -> kosong", |interpreter, args, kwargs| {
                    butuh(interpreter, &args, 2, "simpan")?;
                    let indent = match opsi(&kwargs, "indent") {
                        Some(nilai) => interpreter.nilai_ke_i64(nilai, interpreter.span_aktif)?,
                        None => 0,
                    };
                    let teks = ke_json(&args[0], indent.max(0) as usize, 0)?;
                    let jalur = teks_arg(interpreter, &args[1], "jalur")?;
                    match std::fs::write(&jalur, teks) {
                        Ok(()) => Ok(Nilai::Kosong),
                        Err(galat) => Err(interpreter.lempar_pesan(
                            "KesalahanBerkas",
                            format!("tidak dapat menulis `{jalur}`: {galat}"),
                            interpreter.span_aktif,
                        )),
                    }
                }),
            ),
        ],
    )
}

fn ke_json(nilai: &Nilai, indent: usize, level: usize) -> Hasil<String> {
    let spasi = |level: usize| " ".repeat(indent * level);
    let hasil = match nilai {
        Nilai::Kosong => "null".to_string(),
        Nilai::Logika(nilai) => if *nilai { "true" } else { "false" }.to_string(),
        Nilai::Bilangan(nilai) => nilai.to_string(),
        Nilai::Pecahan(nilai) => {
            if nilai.is_finite() {
                let teks = nilai.to_string();
                if teks.contains('.') || teks.contains('e') || teks.contains('E') {
                    teks
                } else {
                    format!("{teks}.0")
                }
            } else {
                "null".to_string()
            }
        }
        Nilai::Teks(teks) => serde_json::to_string(teks.as_ref()).unwrap_or_else(|_| "\"\"".to_string()),
        Nilai::Bytes(bytes) => serde_json::to_string(&String::from_utf8_lossy(bytes)).unwrap_or_else(|_| "\"\"".to_string()),
        Nilai::Daftar(item) => {
            let item = item.borrow();
            if item.is_empty() {
                "[]".to_string()
            } else {
                let bagian: Vec<String> = item.iter().map(|nilai| ke_json(nilai, indent, level + 1)).collect::<Hasil<Vec<_>>>()?;
                if indent == 0 {
                    format!("[{}]", bagian.join(", "))
                } else {
                    format!("[\n{}{}\n{}]", spasi(level + 1), bagian.join(&format!(",\n{}", spasi(level + 1))), spasi(level))
                }
            }
        }
        Nilai::Tupel(item) => ke_json(&Nilai::daftar(item.as_ref().clone()), indent, level)?,
        Nilai::Himpunan(item) => ke_json(&Nilai::daftar(item.borrow().clone()), indent, level)?,
        Nilai::Kamus(pasangan) => {
            let pasangan = pasangan.borrow();
            if pasangan.is_empty() {
                "{}".to_string()
            } else {
                let bagian: Vec<String> = pasangan
                    .iter()
                    .map(|(kunci, nilai)| {
                        let kunci = match kunci {
                            Nilai::Teks(teks) => serde_json::to_string(teks.as_ref()).unwrap_or_else(|_| "\"\"".to_string()),
                            lain => serde_json::to_string(&nilai_ke_teks(lain)).unwrap_or_else(|_| "\"\"".to_string()),
                        };
                        let nilai = ke_json(nilai, indent, level + 1)?;
                        Ok(if indent == 0 {
                            format!("{kunci}: {nilai}")
                        } else {
                            format!("{kunci}: {nilai}")
                        })
                    })
                    .collect::<Hasil<Vec<_>>>()?;
                if indent == 0 {
                    format!("{{{}}}", bagian.join(", "))
                } else {
                    format!("{{\n{}{}\n{}}}", spasi(level + 1), bagian.join(&format!(",\n{}", spasi(level + 1))), spasi(level))
                }
            }
        }
        Nilai::Rentang(rentang) => {
            let item = rentang.nilai();
            ke_json(&Nilai::daftar(item), indent, level)?
        }
        Nilai::Python(_) => crate::python_repr(nilai),
        lain => serde_json::to_string(&nilai_ke_teks(lain)).unwrap_or_else(|_| "\"\"".to_string()),
    };
    Ok(hasil)
}

fn dari_json(nilai: &serde_json::Value) -> Nilai {
    match nilai {
        serde_json::Value::Null => Nilai::Kosong,
        serde_json::Value::Bool(nilai) => Nilai::Logika(*nilai),
        serde_json::Value::Number(nilai) => {
            if let Some(bulat) = nilai.as_i64() {
                Nilai::Bilangan(bulat)
            } else if let Some(pecahan) = nilai.as_f64() {
                Nilai::Pecahan(pecahan)
            } else {
                Nilai::Kosong
            }
        }
        serde_json::Value::String(teks) => Nilai::teks(teks.clone()),
        serde_json::Value::Array(item) => Nilai::daftar(item.iter().map(dari_json).collect()),
        serde_json::Value::Object(peta) => Nilai::kamus(
            peta.iter()
                .map(|(kunci, nilai)| (Nilai::teks(kunci.clone()), dari_json(nilai)))
                .collect(),
        ),
    }
}

// ------------------------------------------------------------------
// kesalahan
// ------------------------------------------------------------------

fn kesalahan(interpreter: &mut Interpreter) -> Rc<Modul> {
    let mut isi = Vec::new();
    let mut nama: Vec<String> = interpreter.kelas_bawaan.keys().cloned().collect();
    nama.sort();
    for nama_kelas in nama {
        if let Some(kelas) = interpreter.kelas_bawaan.get(&nama_kelas) {
            if kelas.jenis == JenisKelas::Kesalahan {
                isi.push((Box::leak(nama_kelas.clone().into_boxed_str()) as &str, Nilai::Kelas(kelas.clone())));
            }
        }
    }
    modul("kesalahan", "Kelas kesalahan bawaan.", isi)
}

// ------------------------------------------------------------------
// Pembantu umum untuk modul
// ------------------------------------------------------------------

/// Menyesuaikan jalur relatif terhadap direktori modul yang sedang aktif.
pub fn jalur_relatif(interpreter: &Interpreter, jalur: &str) -> PathBuf {
    let path = Path::new(jalur);
    if path.is_absolute() {
        return path.to_path_buf();
    }
    match &interpreter.modul_dir {
        Some(direktori) => direktori.join(path),
        None => path.to_path_buf(),
    }
}

/// Menyalin atribut modul yang dapat diimpor bintang.
pub fn nama_publik(modul: &Modul) -> Vec<String> {
    if let Some(Nilai::Daftar(item)) = modul.atribut.borrow().get("__semua__") {
        return item.borrow().iter().map(nilai_ke_teks).collect();
    }
    let mut nama: Vec<String> = modul
        .atribut
        .borrow()
        .keys()
        .filter(|nama| !nama.starts_with('_'))
        .cloned()
        .collect();
    nama.sort();
    nama
}

/// Memeriksa apakah sebuah nilai adalah kelas kesalahan.
pub fn kelas_kesalahan(interpreter: &Interpreter, nilai: &Nilai) -> Option<Rc<Kelas>> {
    match nilai {
        Nilai::Kelas(kelas) if kelas.jenis == JenisKelas::Kesalahan => Some(kelas.clone()),
        _ => {
            let _ = interpreter;
            None
        }
    }
}

/// Menandai jenis nilai untuk pesan.
pub fn tipe_nilai(nilai: &Nilai) -> &'static str {
    match jenis_dari_nilai(nilai) {
        JenisKelas::Pengguna => "objek",
        JenisKelas::Kosong => "kosong",
        JenisKelas::Logika => "logika",
        JenisKelas::Bilangan => "bilangan",
        JenisKelas::Pecahan => "pecahan",
        JenisKelas::Kompleks => "kompleks",
        JenisKelas::Teks => "teks",
        JenisKelas::Bytes => "bytes",
        JenisKelas::Daftar => "daftar",
        JenisKelas::Tupel => "tupel",
        JenisKelas::Himpunan => "himpunan",
        JenisKelas::Kamus => "kamus",
        JenisKelas::Rentang => "rentang",
        JenisKelas::Fungsi => "fungsi",
        JenisKelas::Modul => "modul",
        JenisKelas::Berkas => "berkas",
        JenisKelas::Generator => "generator",
        JenisKelas::Kesalahan => "kesalahan",
        JenisKelas::Tipe => "tipe",
    }
}

/// Memeriksa kecocokan kelas tanpa mengekspos detail internal.
pub fn cocok_kelas(interpreter: &mut Interpreter, nilai: &Nilai, kelas: &Nilai) -> Hasil<bool> {
    crate::bawaan::isinstance_nilai(interpreter, nilai, kelas)
}

/// Nilai `Span` kosong untuk modul yang dibuat secara dinamis.
pub fn span_modul() -> Span {
    Span::empty(0)
}

/// Perbandingan sederhana untuk pengurutan nama.
pub fn bandingkan_nama(a: &str, b: &str) -> std::cmp::Ordering {
    a.cmp(b)
}

/// Memastikan jalur berada di dalam direktori yang diizinkan.
pub fn di_dalam(jalur: &Path, induk: &Path) -> bool {
    jalur.starts_with(induk)
}

/// Menggabungkan dua jalur tanpa mengubah jalur absolut.
pub fn gabung_jalur(a: &Path, b: &Path) -> PathBuf {
    if b.is_absolute() {
        b.to_path_buf()
    } else {
        a.join(b)
    }
}

/// Menghitung hash nama sederhana.
pub fn hash_nama(nama: &str) -> u64 {
    let mut hash: u64 = 0xcbf29ce484222325;
    for byte in nama.as_bytes() {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x100000001b3);
    }
    hash
}

/// Menormalkan nama modul.
pub fn nama_modul_normal(nama: &str) -> String {
    nama.trim().trim_matches('.').to_string()
}

/// Daftar modul native yang tersedia.
pub fn daftar_modul() -> Vec<&'static str> {
    vec![
        "matematika",
        "alat_teks",
        "alat_berkas",
        "jalur",
        "alat_daftar",
        "alat_kamus",
        "alat_iterasi",
        "alat_fungsi",
        "sidik",
        "alur_asinkron",
        "sistem_tipe",
        "kelas_data",
        "tipe_pilihan",
        "abstrak",
        "waktu",
        "acak",
        "sistem",
        "json",
        "kesalahan",
    ]
}

/// Mencari modul native berdasarkan nama.
pub fn cari(interpreter: &mut Interpreter, nama: &str) -> Option<Rc<Modul>> {
    ambil(interpreter, nama)
}

/// Menambahkan modul ke cache tanpa menjalankannya.
pub fn simpan_cache(interpreter: &mut Interpreter, nama: &str, modul: Rc<Modul>) {
    interpreter.modul_cache.insert(nama.to_string(), modul);
}

/// Mengambil modul dari cache.
pub fn ambil_cache(interpreter: &Interpreter, nama: &str) -> Option<Rc<Modul>> {
    interpreter.modul_cache.get(nama).cloned()
}

/// Menghapus modul dari cache.
pub fn hapus_cache(interpreter: &mut Interpreter, nama: &str) -> Option<Rc<Modul>> {
    interpreter.modul_cache.remove(nama)
}

/// Daftar nama modul yang sudah dimuat.
pub fn modul_dimuat(interpreter: &Interpreter) -> Vec<String> {
    let mut nama: Vec<String> = interpreter.modul_cache.keys().cloned().collect();
    nama.sort();
    nama
}

/// Menggabungkan dua peta atribut modul.
pub fn gabung_atribut(tujuan: &Rc<Modul>, asal: &Rc<Modul>) {
    for (nama, nilai) in asal.atribut.borrow().iter() {
        tujuan.atribut.borrow_mut().insert(nama.clone(), nilai.clone());
    }
}

/// Menyalin metadata modul.
pub fn salin_metadata(tujuan: &Rc<Modul>, asal: &Rc<Modul>) {
    tujuan
        .atribut
        .borrow_mut()
        .insert("__nama__".to_string(), Nilai::teks(asal.nama.clone()));
    if let Some(berkas) = &asal.berkas {
        tujuan
            .atribut
            .borrow_mut()
            .insert("__berkas__".to_string(), Nilai::teks(berkas.clone()));
    }
}

/// Menandai modul sebagai paket.
pub fn tandai_paket(modul: &Rc<Modul>, paket: &str) {
    modul
        .atribut
        .borrow_mut()
        .insert("__paket__".to_string(), Nilai::teks(paket));
}

/// Menambahkan submodul sebagai atribut.
pub fn pasang_submodul(induk: &Rc<Modul>, nama: &str, anak: Rc<Modul>) {
    induk
        .atribut
        .borrow_mut()
        .insert(nama.to_string(), Nilai::Modul(anak));
}
