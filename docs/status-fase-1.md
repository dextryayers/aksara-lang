# Status Fase 0 dan Fase 1

Dokumen ini merangkum bukti pemenuhan kriteria Fase 0 dan Fase 1. Tanggal pengukuran: 27 Oktober 2026.

## Fase 0

| Kriteria | Status | Bukti |
|---|---|---|
| ADR disetujui dan dicatat | Selesai | `docs/adr/0001` sampai `0005` |
| `cargo test` hijau | Selesai di Linux | 68 uji Rust lulus |
| `ctest` hijau | Selesai di Linux | 1 uji runtime C lulus |
| CI Linux, macOS, Windows | Dikonfigurasi | `.github/workflows/ci.yml` |
| 10 contoh program | Selesai | `contoh/01_halo.aks` sampai `10_ekosistem.aks` |
| Baseline CPython 3.12 | Selesai | `uji/benchmark/cpython_3.12_baseline.json` |
| Struktur repo dan CI | Selesai | Cargo workspace, CMake, justfile, CI |

Baseline CPython 3.12 memakai CPython 3.12.16 yang dibangun dari sumber. Baseline tambahan CPython 3.13.5 tersedia di `uji/benchmark/cpython_3.13_baseline.json`.

## Fase 1

| Kriteria | Status | Bukti |
|---|---|---|
| 100% contoh terparse | Selesai | 10 dari 10 berkas `contoh/*.aks` |
| Fuzz 24 jam tanpa panic | Belum dijalankan penuh | Kampanye 601 detik: 3.373.913 eksekusi tanpa crash; smoke test 25.000 kasus lulus; target cargo-fuzz siap |
| Berkas 1 MB kurang dari 100 ms | Selesai | Median 76,66 ms, P95 83,25 ms, 20 iterasi |
| 1.000+ kasus uji | Selesai | 1.200 kasus generator, 18 berkas korpus, 34 uji parser, 16 uji lekser, 7 uji CLI |
| Diagnostik baris, kolom, saran | Selesai | `crates/aksara-diagnostik`, diuji melalui CLI |

## Perintah Verifikasi

```bash
cargo test --workspace
cargo test -p aksara-parser --test korpus -- --nocapture
cargo test -p aksara-parser --test golden
cargo test -p aksara-parser --test fuzz_smoke -- --nocapture
cargo run -p aksara-cli -- periksa contoh/*.aks
cargo run --release -p aksara-parser --example bench_parse -- --bytes 1000000 --iterations 20
cmake -S . -B build -DCMAKE_BUILD_TYPE=Release
cmake --build build
ctest --test-dir build --output-on-failure
```

## Catatan

- Uji fuzz 24 jam adalah gerbang rilis, bukan uji harian. Jalankan pada mesin khusus dengan `cargo fuzz run --fuzz-dir uji/fuzz parser -- -max_total_time=86400`.
- Kampanye 601 detik dijalankan pada 27 Oktober 2026 dengan sanitizer `none` karena mesin pengembangan tidak memiliki runtime ASAN. Hasilnya 3.373.913 eksekusi tanpa crash. Kampanye rilis tetap harus memakai nightly dan AddressSanitizer.
- Angka benchmark bergantung pada mesin. Mesin pengembangan memakai Linux 6.12, Rust 1.85.1, dan CPU x86_64.
- CI macOS dan Windows sudah didefinisikan, tetapi verifikasi lokal hanya dilakukan di Linux.
