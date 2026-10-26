# Fuzz Parser

Target fuzz utama adalah lexer dan parser. Tujuan fuzz bukan menemukan hasil semantik yang salah, melainkan memastikan tidak ada panic, hang, atau konsumsi memori tak terbatas pada input arbitrer.

## Smoke test deterministik

```bash
cargo test -p aksara-parser --test fuzz_smoke -- --nocapture
```

Smoke test memutasi sumber valid dengan beberapa strategi dan memastikan parser selalu mengembalikan hasil.

## Fuzz 24 jam

Fase 1 mensyaratkan fuzz 24 jam tanpa panic. Jalankan pada mesin khusus:

```bash
cargo install cargo-fuzz
cargo fuzz run parser -- -max_total_time=86400
```

Target cargo-fuzz berada di `uji/fuzz/` dan tidak menjadi anggota workspace agar build normal tidak membutuhkan dependensi fuzz.

## Triage

1. Simpan input yang memicu crash.
2. Minimalkan dengan `cargo fuzz tmin parser input`.
3. Tambahkan input ke `uji/parser/korpus/regresi/`.
4. Perbaiki parser dan jalankan seluruh suite.
