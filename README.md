# Aksara

Aksara adalah bahasa pemrograman dinamis dengan sintaks bahasa Indonesia formal dan ekstensi berkas `.aks`. Semantik intinya mengikuti Python 3.12, sedangkan ekosistem Python diakses melalui jembatan CPython.

Repositori ini berisi rencana induk, front end Rust, runtime C, standard library Aksara, alat pengembang, dan suite uji.

## Status

Fase 0 selesai dan Fase 1 selesai pada tingkat front end:

- Fase 0: visi, spesifikasi, ADR, kerangka repositori, CI, dan harness uji.
- Fase 1: lekser, parser, AST, diagnostik, CLI `aksara periksa`, korpus, golden AST, fuzz smoke, dan benchmark.

Seluruh 10 program di `contoh/` dan 8 berkas korpus parser terparse tanpa diagnostik. Suite Rust menjalankan 59 uji, termasuk 1.200 kasus valid yang dihasilkan secara deterministik dan 16.000 kasus fuzz smoke. Benchmark parser pada mesin pengembangan mencatat median 81,85 ms untuk berkas 1 MB. Kampanye fuzz 24 jam tetap dijalankan pada mesin khusus sebelum rilis; target cargo-fuzz sudah tersedia di `uji/fuzz/`.

Bahasa belum siap dipakai untuk produksi. Spesifikasi yang mengikat berada di `docs/spesifikasi/`.

## Struktur

```
.
├── crates/          front end Rust
├── runtime/         runtime C11 dan VM bytecode
├── pustaka/         standard library Aksara
├── uji/             suite kepatuhan, fuzz, dan benchmark
├── docs/            visi, spesifikasi, ADR, dan panduan
├── contoh/          program contoh yang harus selalu dapat diparse
└── alat/            alat rilis dan kompatibilitas
```

## Build

Prasyarat:

- Rust 1.75 atau lebih baru.
- C11 compiler (GCC, Clang, atau MSVC).
- CMake 3.20 atau lebih baru untuk runtime C.

```bash
cargo build --workspace
cargo test --workspace
cmake -S . -B build
cmake --build build
ctest --test-dir build --output-on-failure
```

## Menggunakan CLI

```bash
cargo run -p aksara-cli -- periksa contoh/01_halo.aks
cargo run -p aksara-cli -- periksa --json contoh/01_halo.aks
cargo run -p aksara-cli -- periksa --ast contoh/01_halo.aks
```

## Lisensi

MIT. Lihat `LICENSE`.
# aksara-lang
