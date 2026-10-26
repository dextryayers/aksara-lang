# Uji Parser

## Korpus

- `korpus/` berisi berkas `.aks` yang harus terparse tanpa diagnostik.
- `contoh/` di akar repositori adalah target wajib.
- Setiap konstruksi baru harus menambah minimal satu berkas korpus.

## Menjalankan

```bash
cargo test -p aksara-parser
cargo run -p aksara-cli -- periksa contoh/pemula/*.aks
```

## Golden test

Golden test menyimpan AST dalam JSON. Untuk memperbarui golden file:

```bash
cargo run -p aksara-cli -- periksa --ast contoh/bahasa/01_dasar.aks > uji/parser/golden/01_dasar.json
```

Periksa diff sebelum menyimpan. Golden file yang berubah tanpa alasan adalah regresi.

## Target

- 1000+ kasus valid.
- 100% contoh terparse.
- Tidak ada panic pada input apa pun.
- Berkas 1 MB terparse kurang dari 100 ms pada mesin referensi.
