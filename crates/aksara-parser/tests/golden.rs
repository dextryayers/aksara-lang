//! Uji AST emas: keluaran parser harus stabil dan dapat dideserialisasi ulang.
//!
//! Perbarui berkas emas dengan:
//!
//! ```text
//! AKSARA_UPDATE_GOLDEN=1 cargo test -p aksara-parser --test golden
//! ```

use std::fs;
use std::path::{Path, PathBuf};

use aksara_ast::Module;
use aksara_parser::parse;

fn direktori_emas() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("crate berada di crates/aksara-parser")
        .join("uji/parser/golden")
}

fn berkas_emas() -> Vec<PathBuf> {
    let mut hasil = fs::read_dir(direktori_emas())
        .expect("direktori emas ada")
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| path.extension().and_then(|ext| ext.to_str()) == Some("aks"))
        .collect::<Vec<_>>();
    hasil.sort();
    assert!(
        hasil.len() >= 6,
        "berkas emas terlalu sedikit: {}",
        hasil.len()
    );
    hasil
}

fn selisih(harapan: &str, aktual: &str) -> String {
    let baris_harapan = harapan.lines().collect::<Vec<_>>();
    let baris_aktual = aktual.lines().collect::<Vec<_>>();
    let jumlah = baris_harapan.len().max(baris_aktual.len());
    for index in 0..jumlah {
        let kiri = baris_harapan.get(index).copied().unwrap_or("<tidak ada>");
        let kanan = baris_aktual.get(index).copied().unwrap_or("<tidak ada>");
        if kiri != kanan {
            let awal = index.saturating_sub(3);
            let akhir = (index + 4).min(jumlah);
            let mut pesan = format!("perbedaan pertama pada baris {}\n", index + 1);
            for baris in awal..akhir {
                let kiri = baris_harapan.get(baris).copied().unwrap_or("<tidak ada>");
                let kanan = baris_aktual.get(baris).copied().unwrap_or("<tidak ada>");
                let penanda = if kiri == kanan { " " } else { "!" };
                pesan.push_str(&format!("{penanda} harapan: {kiri}\n{penanda} aktual: {kanan}\n"));
            }
            return pesan;
        }
    }
    "tidak ada perbedaan baris".to_string()
}

#[test]
fn ast_emas_stabil_dan_dapat_dideserialisasi() {
    let perbarui = std::env::var_os("AKSARA_UPDATE_GOLDEN").is_some();
    for path in berkas_emas() {
        let source = fs::read_to_string(&path)
            .unwrap_or_else(|error| panic!("gagal membaca {}: {error}", path.display()));
        let result = parse(&source);
        assert!(
            result.diagnostics.is_empty(),
            "{} menghasilkan diagnostik: {:#?}",
            path.display(),
            result.diagnostics
        );

        let aktual = serde_json::to_string_pretty(&result.module).expect("AST dapat diserialisasi");
        let aktual = format!("{aktual}\n");
        let path_json = path.with_extension("json");

        if perbarui {
            fs::write(&path_json, &aktual)
                .unwrap_or_else(|error| panic!("gagal menulis {}: {error}", path_json.display()));
        }

        let harapan = fs::read_to_string(&path_json).unwrap_or_else(|error| {
            panic!(
                "gagal membaca {}: {error}. Jalankan AKSARA_UPDATE_GOLDEN=1 cargo test -p aksara-parser --test golden",
                path_json.display()
            )
        });
        assert_eq!(
            harapan,
            aktual,
            "AST emas berubah untuk {}:\n{}",
            path.display(),
            selisih(&harapan, &aktual)
        );

        let module: Module = serde_json::from_str(&harapan)
            .unwrap_or_else(|error| panic!("JSON emas tidak valid {}: {error}", path_json.display()));
        assert_eq!(module, result.module, "roundtrip AST gagal untuk {}", path.display());
    }
}
