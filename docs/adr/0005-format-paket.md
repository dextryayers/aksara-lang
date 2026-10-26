# ADR 0005: Format Paket dan Registry

## Status

Diterima.

## Konteks

Aksara membutuhkan package manager sendiri untuk modul `.aks`, tetapi juga harus memakai pip dan virtualenv untuk paket Python. Dua ekosistem ini tidak boleh saling menimpa.

## Keputusan

- Paket Aksara memakai manifest `aksara.toml` dan lockfile `aksara.kunci`.
- Paket Python memakai `requirements.txt`, `pyproject.toml`, dan lockfile `aksara-python.kunci`.
- Instalasi paket Python memakai pip asli di virtualenv proyek.
- Registry Aksara bersifat opsional pada v1.0. Instalasi lokal dan git didukung lebih dulu.
- Setiap artefak memiliki checksum SHA-256.
- Tanda tangan artefak direncanakan setelah v1.0.
- Nama paket Aksara dan paket Python berada di namespace terpisah.

## Konsekuensi

- Pengguna dapat memakai index privat dan wheel yang sudah ada.
- Resolver dependensi Python tidak ditulis ulang.
- Package manager Aksara dapat berevolusi tanpa merusak ekosistem Python.
- Keamanan pemasangan tetap menjadi tanggung jawab pengguna dan registry.

## Alternatif yang ditolak

- Memakai PyPI untuk paket Aksara: mencampur dua model paket dan membingungkan pengguna.
- Menulis resolver Python sendiri: risiko tinggi dan tidak perlu.
- Tanpa lockfile: build tidak reproducible.
