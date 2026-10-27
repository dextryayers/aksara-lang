//! Benchmark parser Aksara.
//!
//! Contoh:
//!
//! ```text
//! cargo run --release -p aksara-parser --example bench_parse -- --bytes 1000000 --iterations 10
//! ```
//!
//! Keluaran JSON dapat disimpan sebagai baseline.

use std::fmt::Write as _;
use std::time::Instant;

use aksara_parser::parse;

fn buat_sumber(target: usize) -> String {
    let mut source = String::with_capacity(target + 512);
    source.push_str("# Berkas benchmark parser Aksara.\n");
    let mut blok = 0usize;
    loop {
        let i = blok;
        let mut potongan = String::new();
        let _ = writeln!(
            potongan,
            "\nnilai_{i}: bilangan = {i} + 2 * 3 ** 2"
        );
        let _ = writeln!(
            potongan,
            "salinan_{i} = nilai_{i} jika nilai_{i} > 0 lainnya 0"
        );
        let _ = writeln!(
            potongan,
            "di_antara_{i} = nilai_{i} antara 0 dan 1000"
        );
        let _ = writeln!(
            potongan,
            "data_{i} = [n * n untuk n dalam rentang(10) jika n % 2 == 0]"
        );
        let _ = writeln!(
            potongan,
            "kamus_{i} = {{\"nilai\": nilai_{i}, \"salinan\": salinan_{i}}}"
        );
        let _ = writeln!(potongan, "untuk setiap item_{i} dalam data_{i}:");
        let _ = writeln!(potongan, "    jika item_{i} > 10 dan bukan salah:");
        let _ = writeln!(potongan, "        cetak(f\"item {{item_{i}}}\")");
        let _ = writeln!(potongan, "    lainnya:");
        let _ = writeln!(potongan, "        lanjutkan");
        let _ = writeln!(
            potongan,
            "fungsi hitung_{i}(a, b=1, /, c=2, *args, d=3, **kwargs) -> bilangan:"
        );
        let _ = writeln!(potongan, "    \"\"\"Dokumentasi benchmark.\"\"\"");
        let _ = writeln!(potongan, "    total = a + b + c + d");
        let _ = writeln!(potongan, "    kembalikan total");
        let _ = writeln!(potongan, "kelas Bentuk_{i}(Induk):");
        let _ = writeln!(potongan, "    atribut: bilangan = 1");
        let _ = writeln!(potongan, "    fungsi __init__(diri, x):");
        let _ = writeln!(potongan, "        induk().__init__()");
        let _ = writeln!(potongan, "        diri.x = x");
        if source.len() + potongan.len() > target {
            break;
        }
        source.push_str(&potongan);
        blok += 1;
    }
    if source.len() < target {
        source.push('\n');
        while source.len() + 1 < target {
            source.push('#');
        }
        source.push('\n');
    }
    source
}

fn argumen(nama: &str, bawaan: usize) -> usize {
    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        if arg == nama {
            if let Some(nilai) = args.next() {
                return nilai.parse().unwrap_or(bawaan);
            }
        }
    }
    bawaan
}

fn main() {
    let target = argumen("--bytes", 1_000_000);
    let iterasi = argumen("--iterations", 10).max(1);
    let sumber = buat_sumber(target);
    let bytes = sumber.len();

    // Pemanasan agar alokasi awal tidak masuk pengukuran.
    let awal = parse(&sumber);
    if !awal.diagnostics.is_empty() {
        let diagnostic = &awal.diagnostics[0];
        let start = sumber[..diagnostic.span.start as usize]
            .rfind('\n')
            .map(|index| index + 1)
            .unwrap_or(0);
        let end = sumber[diagnostic.span.end as usize..]
            .find('\n')
            .map(|index| diagnostic.span.end as usize + index)
            .unwrap_or(sumber.len());
        eprintln!(
            "diagnostik pertama: {} pada {}..{}\n{}",
            diagnostic.message, diagnostic.span.start, diagnostic.span.end, &sumber[start..end]
        );
    }
    assert!(
        awal.diagnostics.is_empty(),
        "sumber benchmark tidak valid: {:#?}",
        awal.diagnostics
    );

    let mut waktu = Vec::with_capacity(iterasi);
    for _ in 0..iterasi {
        let mulai = Instant::now();
        let hasil = parse(&sumber);
        waktu.push(mulai.elapsed());
        assert!(
            hasil.diagnostics.is_empty(),
            "sumber benchmark tidak valid: {:#?}",
            hasil.diagnostics
        );
    }
    waktu.sort();
    let tercepat = waktu[0];
    let median = waktu[waktu.len() / 2];
    let p95 = waktu[(waktu.len() * 95).div_ceil(100).saturating_sub(1)];
    let rata = waktu.iter().sum::<std::time::Duration>() / waktu.len() as u32;
    let ms = |durasi: std::time::Duration| durasi.as_secs_f64() * 1000.0;
    let throughput = |durasi: std::time::Duration| {
        (bytes as f64 / (1024.0 * 1024.0)) / durasi.as_secs_f64().max(f64::MIN_POSITIVE)
    };

    let lulus = median.as_secs_f64() * 1000.0 < 100.0;
    println!("{{");
    println!("  \"alat\": \"aksara-parser/bench_parse\",");
    println!("  \"bytes\": {bytes},");
    println!("  \"iterations\": {iterasi},");
    println!("  \"min_ms\": {:.3},", ms(tercepat));
    println!("  \"median_ms\": {:.3},", ms(median));
    println!("  \"mean_ms\": {:.3},", ms(rata));
    println!("  \"p95_ms\": {:.3},", ms(p95));
    println!("  \"median_mib_per_s\": {:.2},", throughput(median));
    println!("  \"target_ms\": 100.0,");
    println!("  \"passed\": {lulus}");
    println!("}}");
}
