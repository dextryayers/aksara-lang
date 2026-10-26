# Visi Aksara

## Pernyataan visi

Aksara adalah bahasa pemrograman yang membuat orang berbahasa Indonesia dapat menulis program dengan kalimat yang mereka pahami, tanpa kehilangan akses ke ekosistem Python yang sudah ada.

Aksara tidak menerjemahkan Python ke bahasa Indonesia. Aksara adalah bahasa baru dengan lexer, parser, AST, bytecode, runtime, dan standard library sendiri. Ekosistem Python diakses melalui jembatan CPython yang eksplisit.

## Masalah yang diselesaikan

1. Banyak pemula harus mempelajari kosakata Inggris, simbol cryptic, dan pesan error berbahasa Inggris pada saat yang sama.
2. Banyak bahasa "mirip Python" hanya transpiler. Mereka mewarisi semantik Python, tetapi tidak membangun identitas, diagnostik, atau tooling sendiri.
3. Ekosistem Python sangat besar dan tidak realistis ditulis ulang. Bahasa baru yang mengabaikannya akan kehilangan library scraping, data, gambar, dan sains.
4. Pesan error sering menyebut gejala, bukan penyebab dan perbaikan.

Aksara menjawab empat masalah itu dengan sintaks Indonesia formal, front end Rust yang cepat, runtime C yang terkontrol, diagnostik ber-kode, dan jembatan Python yang mengikuti kontrak kemampuan.

## Persona

### Pemula

Siswa atau mahasiswa yang baru belajar programming. Ia butuh variabel, percabangan, perulangan, fungsi, dan koleksi. Ia belum butuh async, metaclass, atau buffer protocol. Ia butuh pesan error yang menyebut baris, kolom, dan perbaikan.

### Pengajar

Guru atau dosen yang mengajar algoritma. Ia butuh bahasa yang dapat dibaca di papan tulis, contoh yang konsisten, dan materi ajar 10 pelajaran. Ia tidak ingin menjelaskan mengapa `elif` bukan `lainnya jika`.

### Praktisi data dan web

Developer yang memakai `requests`, `pandas`, `numpy`, `opencv-python`, atau `sqlalchemy`. Ia butuh akses penuh ke library tersebut, termasuk callback, iterator, context manager, buffer protocol, dan eksepsi dua arah. Ia tidak ingin menulis ulang library.

### Kontributor bahasa

Engineer yang mengembangkan compiler, runtime, atau tooling. Ia butuh AST bertipe, spesifikasi yang jelas, uji diferensial, dan batas FFI yang sempit.

## Sepuluh program contoh

Sepuluh program ini adalah target awal Fase 0 dan harus selalu dapat diparse.

1. Halo dunia dan operasi teks.
2. Kalkulator CLI dengan penanganan kesalahan.
3. Pengelola nilai mahasiswa dengan kelas dan pewarisan.
4. Pembaca dan penulis JSON.
5. Scraper halaman web dengan `requests` dan `bs4`.
6. Analisis CSV dengan `pandas`.
7. Pengolahan gambar dengan OpenCV.
8. Server HTTP sederhana dengan `jaringan`.
9. Permainan tebak angka dengan perulangan dan input.
10. Aplikasi daftar tugas berbasis berkas dan argumen CLI.

## Prinsip desain

1. Satu tugas, satu cara utama.
2. Baca seperti kalimat.
3. Tidak ada simbol misterius.
4. Operator tidak mengubah arti secara diam-diam.
5. Error adalah bagian dari bahasa.
6. Fitur lanjutan bersifat opsional.
7. Nama Indonesia untuk objek Aksara, nama asli untuk objek Python.
8. Indentasi 4 spasi, satu pernyataan per baris.
9. Tidak ada efek samping tersembunyi.
10. Setiap fitur punya contoh, uji, dan alasan.

## Bukan target

- Menjalankan berkas `.py` sebagai program sumber tanpa perubahan.
- Kompatibilitas bytecode CPython.
- Menulis ulang seluruh standard library Python secara native.
- Sandbox untuk kode tidak tepercaya.
- Menautkan ekstensi C Python langsung ke VM Aksara.
- JIT pada v1.0.
- Perangkat mobile dan embedded.
- CPython free-threaded, subinterpreter, dan arsitektur 32-bit.

## Ukuran keberhasilan

- Pemula dapat menyelesaikan 10 pelajaran tanpa membaca dokumentasi Python.
- 10 program contoh berjalan dan terparse tanpa perubahan.
- 200+ paket Python lulus uji Tier A atau B pada setiap rilis.
- 2000 paket Python lulus gerbang rilis.
- 1000+ uji kepatuhan lulus di oracle Rust dan VM C.
- Pesan error menyebut baris, kolom, kode stabil, dan saran perbaikan.
- Performa 3x CPython pada 10 benchmark inti.
- Startup tanpa jembatan kurang dari 50 ms.

## Prinsip anti-slop

Dokumen dan kode Aksara tidak memakai klaim tanpa bukti. Setiap klaim kompatibilitas harus memiliki uji. Setiap fitur harus memiliki contoh. Setiap error harus memiliki kode. Tidak ada em dash di dokumentasi.
