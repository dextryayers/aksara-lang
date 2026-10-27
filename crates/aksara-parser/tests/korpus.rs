//! Uji korpus: semua contoh dan korpus fitur harus bersih, ditambah ribuan
//! kasus valid yang dihasilkan secara deterministik.
//!
//! Generator memakai LCG sederhana agar tidak menambah dependensi dan agar
//! kegagalan selalu dapat direproduksi.

use std::fs;
use std::path::{Path, PathBuf};

use aksara_parser::parse;

fn akar_repo() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("crate berada di crates/aksara-parser")
        .to_path_buf()
}

fn kumpulkan_aks(dir: &Path, hasil: &mut Vec<PathBuf>) {
    let entries = fs::read_dir(dir).unwrap_or_else(|error| {
        panic!("gagal membaca {}: {error}", dir.display());
    });
    for entry in entries {
        let entry = entry.expect("entri direktori");
        let path = entry.path();
        if path.is_dir() {
            kumpulkan_aks(&path, hasil);
        } else if path.extension().and_then(|ext| ext.to_str()) == Some("aks") {
            hasil.push(path);
        }
    }
}

fn berkas_korpus() -> Vec<PathBuf> {
    let root = akar_repo();
    let mut hasil = Vec::new();
    kumpulkan_aks(&root.join("contoh"), &mut hasil);
    kumpulkan_aks(&root.join("uji/parser/korpus"), &mut hasil);
    hasil.sort();
    assert!(
        hasil.len() >= 18,
        "korpus terlalu kecil: {} berkas",
        hasil.len()
    );
    hasil
}

#[test]
fn semua_contoh_dan_korpus_bersih() {
    let mut gagal = Vec::new();
    for path in berkas_korpus() {
        let source = fs::read_to_string(&path)
            .unwrap_or_else(|error| panic!("gagal membaca {}: {error}", path.display()));
        let result = parse(&source);
        if !result.diagnostics.is_empty() {
            gagal.push((path, result.diagnostics));
        }
    }
    assert!(
        gagal.is_empty(),
        "berkas dengan diagnostik: {:#?}",
        gagal
            .iter()
            .map(|(path, diagnostics)| (
                path.display().to_string(),
                diagnostics
                    .iter()
                    .map(|diagnostic| format!(
                        "{}:{}:{} {}",
                        diagnostic.code,
                        diagnostic.span.start,
                        diagnostic.span.end,
                        diagnostic.message
                    ))
                    .collect::<Vec<_>>()
            ))
            .collect::<Vec<_>>()
    );
}

struct Rng(u64);

impl Rng {
    fn new(seed: u64) -> Self {
        Self(seed | 1)
    }

    fn next(&mut self) -> u64 {
        self.0 = self
            .0
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        self.0 >> 11
    }

    fn pick<'a, T>(&mut self, items: &'a [T]) -> &'a T {
        &items[(self.next() as usize) % items.len()]
    }

    fn range(&mut self, low: u64, high: u64) -> u64 {
        low + self.next() % (high - low + 1)
    }
}

fn expr(rng: &mut Rng, depth: u32) -> String {
    if depth == 0 {
        return atom(rng);
    }
    match rng.next() % 18 {
        0 => format!(
            "({}) {} ({})",
            expr(rng, depth - 1),
            rng.pick(&["+", "-", "*", "//", "%", "@", "<<", ">>", "&", "|", "^"]),
            expr(rng, depth - 1)
        ),
        1 => format!("({}) ** ({})", atom(rng), atom(rng)),
        2 => format!(
            "({}) {} ({})",
            expr(rng, depth - 1),
            rng.pick(&["<", "<=", ">", ">=", "==", "!="]),
            expr(rng, depth - 1)
        ),
        3 => format!(
            "{} {} {} {} {}",
            atom(rng),
            rng.pick(&["<", "<=", ">"]),
            atom(rng),
            rng.pick(&["<", "<=", ">"]),
            atom(rng)
        ),
        4 => format!(
            "({}) {} ({})",
            expr(rng, depth - 1),
            rng.pick(&["dan", "atau"]),
            expr(rng, depth - 1)
        ),
        5 => format!("bukan ({})", expr(rng, depth - 1)),
        6 => format!("({}) ?? ({})", expr(rng, depth - 1), expr(rng, depth - 1)),
        7 => format!(
            "({}) jika ({}) lainnya ({})",
            expr(rng, depth - 1),
            expr(rng, depth - 1),
            expr(rng, depth - 1)
        ),
        8 => format!(
            "{} {} antara {} dan {}",
            atom(rng),
            rng.pick(&["", "bukan"]),
            atom(rng),
            atom(rng)
        ),
        9 => format!(
            "{} {} dalam {}",
            atom(rng),
            rng.pick(&["", "bukan"]),
            atom(rng)
        ),
        10 => format!(
            "{} {} {}",
            atom(rng),
            rng.pick(&["adalah", "bukan adalah"]),
            atom(rng)
        ),
        11 => format!("({} := {})", nama(rng), expr(rng, depth - 1)),
        12 => format!("[{}]", daftar(rng, depth)),
        13 => format!("({},)", expr(rng, depth - 1)),
        14 => format!(
            "{{{}}}",
            (0..rng.range(1, 3))
                .map(|_| format!("\"{}\": {}", nama(rng), expr(rng, depth - 1)))
                .collect::<Vec<_>>()
                .join(", ")
        ),
        15 => format!(
            "(lambda {}: {})",
            rng.pick(&["x", "x, y=1", "*args, **kwargs"]),
            expr(rng, depth - 1)
        ),
        16 => format!(
            "{}[{}:{}:{}]",
            atom(rng),
            rng.range(0, 5),
            rng.range(6, 10),
            rng.pick(&["1", "2", "-1"])
        ),
        _ => format!(
            "{}({}, {})",
            rng.pick(&["cetak", "len", "f", "g"]),
            expr(rng, depth - 1),
            expr(rng, depth - 1)
        ),
    }
}

fn daftar(rng: &mut Rng, depth: u32) -> String {
    (0..rng.range(0, 4))
        .map(|_| expr(rng, depth.saturating_sub(1)))
        .collect::<Vec<_>>()
        .join(", ")
}

fn nama(rng: &mut Rng) -> String {
    format!("v{}", rng.range(0, 20))
}

fn atom(rng: &mut Rng) -> String {
    match rng.next() % 12 {
        0 => rng.range(0, 1000).to_string(),
        1 => format!("{}.{}", rng.range(0, 100), rng.range(0, 99)),
        2 => "\"teks\"".to_string(),
        3 => "'t'.join([\"a\", \"b\"])".to_string(),
        4 => "benar".to_string(),
        5 => "salah".to_string(),
        6 => "kosong".to_string(),
        7 => nama(rng),
        8 => "data[0]".to_string(),
        9 => "objek.atribut".to_string(),
        10 => "data?.nilai?[0]".to_string(),
        _ => "f\"{nama} dan {nilai!r:>8}\"".to_string(),
    }
}

fn blok(rng: &mut Rng, depth: u32, lines: &mut Vec<String>) {
    let count = rng.range(1, 3);
    for _ in 0..count {
        pernyataan(rng, depth, lines);
    }
}

fn pernyataan(rng: &mut Rng, depth: u32, lines: &mut Vec<String>) {
    let indent = "    ".repeat(depth as usize);
    if depth >= 2 {
        lines.push(format!("{indent}{} = {}", nama(rng), expr(rng, 2)));
        return;
    }
    match rng.next() % 16 {
        0 => lines.push(format!("{indent}{} = {}", nama(rng), expr(rng, 3))),
        1 => lines.push(format!(
            "{indent}{} {} {}",
            nama(rng),
            rng.pick(&["+=", "-=", "*=", "//=", "%=", "**=", "&=", "|=", "^=", ">>="]),
            expr(rng, 2)
        )),
        2 => {
            let a = nama(rng);
            let b = nama(rng);
            lines.push(format!("{indent}{a}, {b} = {b}, {a}"));
        }
        3 => {
            lines.push(format!("{indent}jika {}:", expr(rng, 2)));
            blok(rng, depth + 1, lines);
            if rng.next() % 2 == 0 {
                lines.push(format!("{indent}lainnya jika {}:", expr(rng, 2)));
                blok(rng, depth + 1, lines);
            }
            if rng.next() % 2 == 0 {
                lines.push(format!("{indent}lainnya:"));
                blok(rng, depth + 1, lines);
            }
        }
        4 => {
            lines.push(format!("{indent}selama {}:", expr(rng, 2)));
            blok(rng, depth + 1, lines);
            lines.push(format!("{indent}    hentikan"));
        }
        5 => {
            let target = nama(rng);
            match rng.next() % 4 {
                0 => lines.push(format!("{indent}untuk {target} dalam {}:", expr(rng, 2))),
                1 => lines.push(format!(
                    "{indent}untuk {target} sebanyak {}:",
                    rng.range(1, 9)
                )),
                2 => lines.push(format!(
                    "{indent}untuk {target} dari {} sampai {}:",
                    rng.range(0, 5),
                    rng.range(6, 10)
                )),
                _ => lines.push(format!(
                    "{indent}untuk {target} dari {} sampai {} langkah -1:",
                    rng.range(6, 10),
                    rng.range(0, 5)
                )),
            }
            blok(rng, depth + 1, lines);
            lines.push(format!("{indent}    lanjutkan"));
        }
        6 => {
            lines.push(format!("{indent}coba:"));
            blok(rng, depth + 1, lines);
            lines.push(format!("{indent}tangkap (KesalahanNilai, KesalahanTipe) sebagai e:"));
            blok(rng, depth + 1, lines);
            if rng.next() % 2 == 0 {
                lines.push(format!("{indent}lainnya:"));
                blok(rng, depth + 1, lines);
            }
            lines.push(format!("{indent}akhirnya:"));
            blok(rng, depth + 1, lines);
        }
        7 => {
            lines.push(format!("{indent}dengan {} sebagai sumber:", expr(rng, 1)));
            blok(rng, depth + 1, lines);
        }
        8 => {
            let fungsi = nama(rng);
            lines.push(format!(
                "{indent}fungsi {fungsi}(a, b=1, /, c=2, *args, d=3, **kwargs) -> teks:"
            ));
            lines.push(format!("{indent}    \"\"\"Dokumentasi.\"\"\""));
            lines.push(format!("{indent}    kembalikan f\"{{a}}{{b}}{{c}}{{d}}\""));
        }
        9 => {
            let kelas = nama(rng);
            lines.push(format!("{indent}kelas {kelas}(Induk):"));
            lines.push(format!("{indent}    atribut: bilangan = 1"));
            lines.push(format!("{indent}    fungsi __init__(diri, x):"));
            lines.push(format!("{indent}        induk().__init__()"));
            lines.push(format!("{indent}        diri.x = x"));
        }
        10 => {
            lines.push(format!("{indent}cocokkan {}:", expr(rng, 1)));
            lines.push(format!("{indent}    kasus [kepala, *ekor]:"));
            lines.push(format!("{indent}        lewati"));
            lines.push(format!("{indent}    kasus {{\"k\": v, **sisa}}:"));
            lines.push(format!("{indent}        lewati"));
            lines.push(format!("{indent}    kasus int() sebagai angka jika angka > 0:"));
            lines.push(format!("{indent}        lewati"));
            lines.push(format!("{indent}    kasus _:"));
            lines.push(format!("{indent}        lewati"));
        }
        11 => lines.push(format!("{indent}pastikan {}, \"pesan\"", expr(rng, 1))),
        12 => lines.push(format!("{indent}lempar KesalahanNilai(\"x\") dari kosong")),
        13 => lines.push(format!("{indent}impor modul_{} sebagai m", rng.range(0, 9))),
        14 => lines.push(format!(
            "{indent}dari paket.modul impor nama sebagai alias, lain"
        )),
        _ => lines.push(format!("{indent}{}", expr(rng, 3))),
    }
}

fn modul_hasil(rng: &mut Rng) -> String {
    let mut lines = Vec::new();
    let count = rng.range(1, 6);
    for _ in 0..count {
        pernyataan(rng, 0, &mut lines);
    }
    let mut source = lines.join("\n");
    source.push('\n');
    source
}

#[test]
fn seribu_kasus_valid_hasil_generator() {
    let mut rng = Rng::new(0x5EED_1234);
    let mut gagal = Vec::new();
    for index in 0..1200 {
        let source = modul_hasil(&mut rng);
        let result = parse(&source);
        if !result.diagnostics.is_empty() {
            if gagal.len() < 5 {
                gagal.push((index, source, result.diagnostics));
            } else {
                gagal.push((index, String::new(), Vec::new()));
            }
        }
    }
    assert!(
        gagal.is_empty(),
        "kasus hasil generator gagal: {:#?}",
        gagal
            .iter()
            .map(|(index, source, diagnostics)| (
                index,
                source.clone(),
                diagnostics
                    .iter()
                    .map(|diagnostic| format!(
                        "{}:{} {}",
                        diagnostic.code, diagnostic.span.start, diagnostic.message
                    ))
                    .collect::<Vec<_>>()
            ))
            .collect::<Vec<_>>()
    );
}

#[test]
fn generator_menghasilkan_lebih_dari_seribu_kasus() {
    let mut rng = Rng::new(0xA15A_2026);
    let mut unik = std::collections::BTreeSet::new();
    for _ in 0..1200 {
        unik.insert(modul_hasil(&mut rng));
    }
    assert!(unik.len() >= 1000, "hanya {} kasus unik", unik.len());
}
