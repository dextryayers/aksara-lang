# Uji Parser

## Korpus

- `korpus/` berisi berkas `.aks` yang harus terparse tanpa diagnostik.
- `contoh/` di akar repositori adalah target wajib.
- Setiap konstruksi baru harus menambah minimal satu berkas korpus.
- Generator di `crates/aksara-parser/tests/korpus.rs` membuat 1.200 kasus valid secara deterministik.

## Menjalankan

```bash
cargo test -p aksara-parser
cargo test -p aksara-parser --test korpus -- --nocapture
cargo test -p aksara-parser --test golden
cargo test -p aksara-parser --test fuzz_smoke -- --nocapture
cargo run -p aksara-cli -- periksa contoh/*.aks
```

## Golden test

Golden test menyimpan AST dalam JSON. Berkas masukan berada di `golden/*.aks` dan hasilnya di `golden/*.json`.

Untuk memperbarui seluruh golden file setelah perubahan AST yang disengaja:

```bash
AKSARA_UPDATE_GOLDEN=1 cargo test -p aksara-parser --test golden
```

Periksa diff sebelum menyimpan. Golden file yang berubah tanpa alasan adalah regresi. Uji juga mendeserialisasi ulang JSON menjadi `Module` dan membandingkannya dengan AST asli.

## Target

- 1.000+ kasus valid.
- 100% contoh terparse.
- Tidak ada panic pada input apa pun.
- Berkas 1 MB terparse kurang dari 100 ms pada mesin referensi.
