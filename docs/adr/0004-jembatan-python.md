# ADR 0004: Jembatan Python

## Status

Diterima.

## Konteks

Aksara tidak menulis ulang seluruh ekosistem Python. Library Python harus dapat dipakai, termasuk ekstensi native, callback, iterator, context manager, async, dan buffer protocol.

## Keputusan

- Jalur utama adalah embedded CPython in-process melalui C API.
- CPython 3.11, 3.12, dan 3.13 didukung, dengan 3.12 sebagai versi utama.
- Objek Python diakses melalui proxy `objek_python`.
- Tipe primitif dan koleksi dikonversi by value.
- `susunan` dan `numpy.ndarray` memakai buffer protocol tanpa salinan.
- Mode isolasi subprocess disediakan untuk paket yang membutuhkan fork, signal, `sys.executable`, atau proses sendiri.
- Tidak ada daftar putih paket. Dukungan diukur dengan kontrak kemampuan dan tier A/B/C/D.
- Jembatan bukan sandbox.

## Konsekuensi

- Overhead konversi dan GIL harus diukur.
- Eksepsi, callback, dan pembersihan sumber daya harus diuji dua arah.
- Distribusi harus menemukan `libpython` yang cocok.
- Paket yang memeriksa internal CPython diuji per versi minor.

## Alternatif yang ditolak

- Menulis ulang semua library: biaya tidak sebanding.
- Transpiler sebagai jalur utama: semantik sulit dijaga dan debugging buruk.
- Subprocess sebagai jalur utama: overhead tinggi dan tidak mendukung callback serta zero-copy.
