# Fuzz Parser

Target fuzz utama adalah lexer dan parser. Tujuan fuzz bukan menemukan hasil semantik yang salah, melainkan memastikan tidak ada panic, hang, atau konsumsi memori tak terbatas pada input arbitrer.

Target cargo-fuzz berada di `uji/fuzz/` dan tidak menjadi anggota workspace agar build normal tidak membutuhkan dependensi fuzz. Direktori ini memakai `libfuzzer-sys` dan harus dijalankan dengan `cargo fuzz`.

## Smoke test deterministik

```bash
cargo test -p aksara-parser --test fuzz_smoke -- --nocapture
```

Smoke test memotong setiap berkas korpus pada banyak batas karakter, memutasi sumber valid, menghasilkan masukan acak, dan menguji masukan ekstrem. Total sekitar 16.000 kasus per jalan.

## Fuzz 24 jam

Fase 1 mensyaratkan fuzz 24 jam tanpa panic. Jalankan pada mesin khusus:

```bash
cargo install cargo-fuzz
cd uji/fuzz
cargo fuzz run parser -- -max_total_time=86400
```

Seed awal tersedia di `corpus/parser/`. Tambahkan input baru dari korpus bila ingin cakupan yang lebih luas.

## Triage

1. Simpan input yang memicu crash.
2. Minimalkan dengan `cargo fuzz tmin parser input`.
3. Tambahkan input ke `uji/parser/korpus/regresi/`.
4. Perbaiki parser dan jalankan seluruh suite.

## Status

Kampanye 24 jam belum dijalankan pada mesin pengembangan ini. Smoke test deterministik dan target fuzz sudah siap; kampanye panjang adalah gerbang rilis, bukan pengganti uji harian.
