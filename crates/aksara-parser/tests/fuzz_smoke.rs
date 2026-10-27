//! Fuzz smoke deterministik: parser tidak boleh panic dan semua span harus
//! berada di dalam sumber, bahkan untuk masukan acak atau rusak.
//!
//! Uji ini bukan pengganti fuzzing 24 jam. Target cargo-fuzz tersedia di
//! `uji/fuzz` untuk kampanye panjang.

use std::fs;
use std::path::{Path, PathBuf};

use aksara_parser::parse;

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

    fn range(&mut self, low: usize, high: usize) -> usize {
        low + (self.next() as usize) % (high - low + 1)
    }
}

fn akar_repo() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("crate berada di crates/aksara-parser")
        .to_path_buf()
}

fn kumpulkan_aks(dir: &Path, hasil: &mut Vec<PathBuf>) {
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                kumpulkan_aks(&path, hasil);
            } else if path.extension().and_then(|ext| ext.to_str()) == Some("aks") {
                hasil.push(path);
            }
        }
    }
}

fn sumber_korpus() -> Vec<String> {
    let root = akar_repo();
    let mut paths = Vec::new();
    kumpulkan_aks(&root.join("contoh"), &mut paths);
    kumpulkan_aks(&root.join("uji/parser/korpus"), &mut paths);
    paths.sort();
    assert!(!paths.is_empty(), "korpus fuzz kosong");
    paths
        .into_iter()
        .map(|path| {
            fs::read_to_string(&path)
                .unwrap_or_else(|error| panic!("gagal membaca {}: {error}", path.display()))
        })
        .collect()
}

fn periksa(source: &str, konteks: &str) {
    let result = parse(source);
    let panjang = source.len() as u32;
    assert!(
        result.diagnostics.len() < 10_000,
        "{konteks}: diagnostik meledak ({} buah)",
        result.diagnostics.len()
    );
    for diagnostic in &result.diagnostics {
        assert!(
            diagnostic.span.start <= diagnostic.span.end,
            "{konteks}: span terbalik {:?}",
            diagnostic.span
        );
        assert!(
            diagnostic.span.end <= panjang,
            "{konteks}: span {:?} melewati panjang sumber {panjang}",
            diagnostic.span
        );
        assert!(
            diagnostic.code.starts_with("E-"),
            "{konteks}: kode diagnostik tidak dikenal {}",
            diagnostic.code
        );
    }
    assert!(
        result.module.span.end <= panjang,
        "{konteks}: span modul {:?} melewati panjang sumber {panjang}",
        result.module.span
    );
}

fn potong_pada_batas_char(source: &str, batas: usize) -> &str {
    let mut akhir = batas.min(source.len());
    while akhir > 0 && !source.is_char_boundary(akhir) {
        akhir -= 1;
    }
    &source[..akhir]
}

const KARAKTER: &[char] = &[
    'a', 'b', 'c', 'd', 'e', 'i', 'k', 'l', 'm', 'n', 'o', 'p', 'r', 's', 't', 'u', 'x', 'y',
    'z', 'A', 'Z', '0', '1', '2', '3', '9', '_', ' ', ' ', ' ', '\n', '\n', '\t', ':', ';', ',',
    '.', '(', ')', '[', ']', '{', '}', '+', '-', '*', '/', '%', '@', '&', '|', '^', '~', '<',
    '>', '=', '!', '?', '"', '\'', '#', '\\', 'π', 'ä', '字', '🙂',
];

fn sisipkan_acak(rng: &mut Rng, chars: &mut Vec<char>) {
    let posisi = rng.range(0, chars.len());
    let karakter = KARAKTER[rng.range(0, KARAKTER.len() - 1)];
    chars.insert(posisi, karakter);
}

fn mutasi(rng: &mut Rng, source: &str) -> String {
    let mut chars = source.chars().collect::<Vec<_>>();
    let operasi = rng.range(1, 5);
    for _ in 0..operasi {
        if chars.is_empty() {
            sisipkan_acak(rng, &mut chars);
            continue;
        }
        match rng.next() % 6 {
            0 => {
                let posisi = rng.range(0, chars.len() - 1);
                chars.remove(posisi);
            }
            1 => sisipkan_acak(rng, &mut chars),
            2 => {
                let posisi = rng.range(0, chars.len() - 1);
                chars[posisi] = KARAKTER[rng.range(0, KARAKTER.len() - 1)];
            }
            3 => {
                let awal = rng.range(0, chars.len() - 1);
                let akhir = rng.range(awal, chars.len());
                let potongan = chars[awal..akhir].to_vec();
                let tujuan = rng.range(0, chars.len());
                for (offset, karakter) in potongan.into_iter().enumerate() {
                    chars.insert(tujuan + offset, karakter);
                }
            }
            4 => {
                let posisi = rng.range(0, chars.len());
                let panjang = rng.range(1, 12);
                for _ in 0..panjang {
                    chars.insert(posisi, KARAKTER[rng.range(0, KARAKTER.len() - 1)]);
                }
            }
            _ => {
                let awal = rng.range(0, chars.len() - 1);
                let akhir = rng.range(awal, chars.len());
                chars.drain(awal..akhir);
            }
        }
    }
    chars.into_iter().collect()
}

#[test]
fn semua_potongan_korpus_tidak_panic() {
    for source in sumber_korpus() {
        let mut batas = 0;
        while batas <= source.len() {
            let potongan = potong_pada_batas_char(&source, batas);
            periksa(potongan, &format!("potongan {batas} byte"));
            batas += 3;
        }
    }
}

#[test]
fn mutasi_korpus_tidak_panic() {
    let sumber = sumber_korpus();
    let mut rng = Rng::new(0xF022_2026);
    for kasus in 0..8_000 {
        let asal = &sumber[rng.range(0, sumber.len() - 1)];
        let source = mutasi(&mut rng, asal);
        periksa(&source, &format!("mutasi kasus {kasus}"));
    }
}

#[test]
fn masukan_acak_tidak_panic() {
    let mut rng = Rng::new(0xACAB_2026);
    for kasus in 0..8_000 {
        let panjang = rng.range(0, 300);
        let source = (0..panjang)
            .map(|_| KARAKTER[rng.range(0, KARAKTER.len() - 1)])
            .collect::<String>();
        periksa(&source, &format!("acak kasus {kasus}"));
    }
}

#[test]
fn masukan_ekstrem_tidak_panic() {
    let kasus = [
        "",
        "\0",
        "\u{feff}",
        "\r\n\r\n",
        "jika:",
        "fungsi:",
        "kelas:",
        "cocokkan:",
        "coba:",
        "dengan:",
        "untuk:",
        "selama:",
        "f\"{\"",
        "f\"}\"",
        "f\"{a!\"",
        "'''",
        "\"\"\"",
        "\\",
        "#",
        "0x",
        "1e",
        "1_",
        "?",
        "??",
        "?.",
        "@",
        "**",
        "->",
        ":=",
        "...",
        "a = ",
        "a = (",
        "a = [",
        "a = {",
        "a = f\"{",
        &"(".repeat(5_000),
        &"[".repeat(5_000),
        &"jika benar:\n    ".repeat(2_000),
        &"a + ".repeat(5_000),
        &"f\"{".repeat(2_000),
    ];
    for (index, source) in kasus.iter().enumerate() {
        periksa(source, &format!("ekstrem kasus {index}"));
    }
}
