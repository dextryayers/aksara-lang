# ADR 0002: Format Bytecode `.akc`

## Status

Diterima.

## Konteks

VM C membutuhkan representasi program yang stabil, dapat diverifikasi, dan tidak bergantung pada layout memori proses compiler.

## Keputusan

- Bytecode disimpan dalam berkas `.akc`.
- Header memuat magic `AKS1`, versi mayor, versi minor, jumlah konstanta, jumlah instruksi, offset line table, offset exception table, offset debug info, dan CRC32.
- Semua angka multibyte little-endian.
- Konstanta disimpan dalam pool dengan tipe eksplisit.
- Bytecode hanya menyimpan indeks, bukan pointer.
- Verifier berjalan sebelum eksekusi dan memeriksa stack depth, target lompatan, indeks lokal, tipe konstanta, dan exception table.
- Perubahan format yang tidak kompatibel menaikkan versi mayor. Penambahan yang kompatibel menaikkan versi minor.

## Konsekuensi

- Bytecode dapat diuji tanpa menjalankan compiler.
- Verifier menjadi komponen keamanan wajib.
- Debug info dapat dihapus pada build rilis.
- Format harus didokumentasikan dan diuji dengan golden file.

## Alternatif yang ditolak

- Menyimpan AST dan menafsirkannya langsung: lebih lambat dan tidak stabil untuk distribusi.
- Memakai bytecode CPython: mengikat pada detail internal CPython dan menghilangkan identitas bahasa.
