# Fuzz Parser

Target fuzz utama adalah lexer dan parser. Tujuan fuzz bukan menemukan hasil semantik yang salah, melainkan memastikan tidak ada panic, hang, atau konsumsi memori tak terbatas pada input arbitrer.

Target cargo-fuzz berada di `uji/fuzz/` dan tidak menjadi anggota workspace agar build normal tidak membutuhkan dependensi fuzz. Direktori ini memakai `libfuzzer-sys` dan harus dijalankan dengan `cargo fuzz`.

## Smoke test deterministik

```bash
cargo test -p aksara-parser --test fuzz_smoke -- --nocapture
```

Smoke test memotong setiap berkas korpus pada banyak batas karakter, memutasi sumber valid, menghasilkan masukan acak, dan menguji masukan ekstrem. Total sekitar 25.000 kasus per jalan.

## Fuzz 24 jam

Fase 1 mensyaratkan fuzz 24 jam tanpa panic. Jalankan pada mesin khusus:

```bash
cargo install cargo-fuzz
cargo fuzz run --fuzz-dir uji/fuzz parser -- -max_total_time=86400
```

Seed awal tersedia di `corpus/parser/`. Tambahkan input baru dari korpus bila ingin cakupan yang lebih luas.

Pada mesin yang hanya memiliki Rust stabil tanpa runtime ASAN, gunakan sanitizer `none` dan `RUSTC_BOOTSTRAP=1`:

```bash
RUSTC_BOOTSTRAP=1 cargo fuzz run --fuzz-dir uji/fuzz -s none parser -- -max_total_time=60
```

Kampanye rilis tetap harus memakai nightly dan AddressSanitizer agar bug memori di runtime C juga tertangkap.

## Triage

1. Simpan input yang memicu crash.
2. Minimalkan dengan `cargo fuzz tmin --fuzz-dir uji/fuzz parser input`.
3. Tambahkan input ke `uji/parser/korpus/regresi/`.
4. Perbaiki parser dan jalankan seluruh suite.

## Status

Kampanye 601 detik dijalankan pada 27 Oktober 2026 dengan sanitizer `none` dan `RUSTC_BOOTSTRAP=1`. Hasilnya 3.373.913 eksekusi tanpa crash, dengan cakupan 2.993 edge dan 19.882 fitur. Kampanye ini membuktikan target fuzz berjalan stabil, tetapi bukan pengganti kampanye 24 jam.

Kampanye 24 jam belum dijalankan pada mesin pengembangan ini. Smoke test deterministik dan target fuzz sudah siap; kampanye panjang adalah gerbang rilis, bukan pengganti uji harian.
