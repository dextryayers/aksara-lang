# Panduan Kontribusi

Terima kasih telah berkontribusi pada Aksara.

## Prinsip

1. Bahasa Indonesia formal dipakai untuk kata kunci, nama bawaan, diagnostik, dan dokumentasi pengguna.
2. Perubahan sintaks harus disertai spesifikasi, uji kepatuhan, dan contoh.
3. Front end Rust tidak boleh bergantung pada detail internal runtime C.
4. Setiap error memiliki kode stabil `E-*` dan saran perbaikan bila memungkinkan.
5. Tidak ada em dash di dokumentasi. Gunakan titik, koma, atau tanda kurung.

## Alur kerja

```bash
cargo fmt --all
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cmake -S . -B build
cmake --build build
ctest --test-dir build --output-on-failure
```

## Menambah konstruksi bahasa

1. Perbarui `docs/spesifikasi/tata_bahasa.ebnf`.
2. Tambahkan token di `crates/aksara-lexer`.
3. Tambahkan node AST di `crates/aksara-ast`.
4. Implementasikan parsing di `crates/aksara-parser`.
5. Tambahkan diagnostik dan pemulihan error.
6. Tambahkan uji unit, uji golden, dan contoh di `contoh/`.
7. Perbarui `plan.md` bila keputusan desain berubah.

## Gaya kode

- Rust memakai `rustfmt` bawaan.
- C memakai C11 dan `clang-format` bila tersedia.
- Nama tipe dan fungsi Rust memakai bahasa Inggris agar mudah dipelihara.
- Teks yang dilihat pengguna memakai bahasa Indonesia.

## Commit

Gunakan pesan ringkas dalam bahasa Indonesia atau Inggris:

```
lekser: tambah dukungan f-string bersarang
parser: perbaiki pemulihan error pada suite kosong
docs: perjelas aturan indentasi
```

## Lisensi kontribusi

Dengan mengirim pull request, Anda menyetujui kontribusi dilisensikan di bawah MIT.
