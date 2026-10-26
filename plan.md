# Rencana Induk Bahasa Aksara

| Atribut | Nilai |
|---|---|
| Versi dokumen | 1.0 |
| Tanggal | 27 Oktober 2026 |
| Status | Draf untuk ditinjau |
| Ekstensi berkas | `.aks` |
| Nama kerja repositori | `langku` |

Dokumen ini adalah rencana pengembangan, bukan spesifikasi final. Spesifikasi bahasa yang mengikat disimpan di `docs/spesifikasi/` dan dibekukan pada rilis 1.0.

## Daftar isi

1. Ringkasan
2. Keputusan yang ditetapkan
3. Arsitektur
4. Spesifikasi bahasa
5. Kompatibilitas Python
6. Standard library
7. Peta fase
8. Strategi pengujian
9. Alat dan ekosistem
10. Performa
11. Keamanan dan keandalan
12. Paket dan distribusi
13. Organisasi repositori
14. Risiko dan mitigasi
15. Definisi selesai v1.0
16. Pasca 1.0
- Lampiran A: Kata kunci
- Lampiran B: Fungsi bawaan
- Lampiran C: Modul standard library
- Lampiran D: Format bytecode
- Lampiran E: Referensi
- Lampiran F: Contoh integrasi Python
- Lampiran G: Ringkasan sintaks
- Lampiran H: Ringkasan operator

## 1. Ringkasan

Aksara adalah bahasa pemrograman dinamis dengan sintaks bahasa Indonesia formal. Semantik intinya mengikuti Python 3.12: tipe dinamis, model objek berbasis kelas, pewarisan ganda dengan C3 MRO, eksepsi, generator, context manager, modul, dan paket. Perbedaan utama terletak pada kata kunci, nama fungsi bawaan, pesan kesalahan, dan nama modul standard library.

Aksara tidak bertujuan menjalankan berkas `.py` sebagai program sumber. Aksara adalah bahasa baru dengan sintaks dan identitas sendiri. Kompatibilitas dengan ekosistem Python dicapai melalui jembatan CPython, sehingga modul Python murni maupun ekstensi native seperti `requests`, `pandas`, `numpy`, `opencv-python`, `sqlalchemy`, dan `matplotlib` dapat dimuat dan dipanggil dari kode Aksara.

Batas klaim perlu dinyatakan sejak awal. Yang dijamin adalah setiap paket yang berjalan pada CPython 3.11, 3.12, atau 3.13 di platform target, selama paket itu tidak melanggar batasan runtime yang didokumentasikan di Bagian 5.12. Tidak ada daftar putih paket. Jika sebuah paket berjalan di CPython biasa, jalur yang sama harus tersedia di Aksara, baik secara in-process maupun melalui mode isolasi.

Target rilis 1.0:

- 95% fitur bahasa Python 3.12 yang relevan.
- 80% modul standard library inti.
- Seluruh ekosistem Python yang kompatibel dengan CPython 3.11 sampai 3.13 di platform target dapat diimpor, diuji pada 200+ paket inti setiap rilis dan 2000 paket pada gerbang rilis, serta diperluas terus tanpa daftar putih.
- Performa 3x CPython pada 10 benchmark inti.
- Tooling: formatter, linter, LSP, debugger, package manager.
- Dukungan Linux, macOS, dan Windows.
- Sintaks pemula yang konsisten, operator lengkap, dan pesan kesalahan berbahasa Indonesia.

Konsekuensi teknis dari klaim ini:

- Jembatan harus mendukung modul Python murni, ekstensi C, buffer protocol, callback, iterator, context manager, async iterator, thread, dan eksepsi dua arah.
- Paket yang tidak dapat berjalan in-process karena fork, signal, atau asumsi internal CPython tetap harus berjalan melalui mode isolasi subprocess.
- Kegagalan paket diklasifikasikan sebagai bug jembatan, batasan paket, masalah lingkungan, atau platform yang tidak didukung. Tidak ada kategori "paket tidak ada di daftar".
- Matriks uji adalah sampel yang tumbuh, bukan gerbang yang membatasi. Jumlah 200+ paket inti adalah lantai pengujian setiap rilis, dan 2000 paket adalah gerbang rilis, bukan langit-langit dukungan.

Bukan target 1.0:

- Menjalankan berkas `.py` sebagai program sumber tanpa perubahan.
- Kompatibilitas bytecode CPython.
- Menulis ulang seluruh standard library Python secara native. Standard library Python tetap dapat diakses melalui jembatan.
- Sandbox untuk kode tidak tepercaya.
- Menautkan ekstensi C Python langsung ke VM Aksara. Ekstensi tersebut berjalan melalui CPython.
- JIT.
- Perangkat mobile dan embedded.
- CPython free-threaded, subinterpreter, dan arsitektur 32-bit.

Sepuluh program contoh yang menjadi target awal:

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

## 2. Keputusan yang ditetapkan

| Nomor | Keputusan | Nilai |
|---|---|---|
| K-01 | Nama bahasa | Aksara |
| K-02 | Ekstensi berkas | `.aks` |
| K-03 | Paradigma | Multi-paradigma, dinamis, imperatif, berorientasi objek, fungsional |
| K-04 | Model tipe | Dinamis, kuat, anotasi opsional |
| K-05 | Blok kode | Indentasi 4 spasi, tab ditolak |
| K-06 | Kata kunci | Bahasa Indonesia formal |
| K-07 | Front end | Rust |
| K-08 | Runtime dan VM | C11 |
| K-09 | Interpreter referensi | Rust |
| K-10 | Jembatan Python | CPython C API, opsional saat build |
| K-11 | Format bytecode | `.akc`, berversi, diverifikasi |
| K-12 | Manajemen memori | Mark-sweep non-moving, precise roots |
| K-13 | Lisensi | MIT untuk compiler, runtime, dan stdlib |
| K-14 | Platform 1.0 | Linux x86_64/aarch64, macOS 12+, Windows 10+ |
| K-15 | Repositori | Monorepo |
| K-16 | Nama metode khusus | Dipertahankan seperti Python (`__init__`, `__str__`) |
| K-17 | `lambda` | Dipertahankan sebagai pengecualian pragmatis |
| K-18 | Konkurensi 1.0 | Utas, kunci, antrean, async dasar, tanpa paralelisme CPU |
| K-19 | WebAssembly | Playground eksperimental, bukan target 1.0 |
| K-20 | Versi CPython | 3.11, 3.12, 3.13; versi utama 3.12 |
| K-21 | Cakupan ekosistem Python | Semua paket yang berjalan pada CPython yang didukung di platform target; tanpa daftar putih; 200+ paket inti setiap rilis dan 2000 paket pada gerbang rilis |
| K-22 | Mode jembatan | In-process sebagai jalur utama, isolasi subprocess sebagai fallback wajib |
| K-23 | Sintaks bahasa alami | `antara ... dan ...`, `untuk setiap`, `sebanyak`, `dari ... sampai ...`, dan ternary `x jika kondisi lainnya y` |
| K-24 | Operator tambahan | `??`, `??=`, `?.`, `?[]`, `?()`, `@`, `:=`, unpacking, dan operator bitwise |
| K-25 | Diagnostik | Kode stabil `E-*`, pesan Indonesia, saran perbaikan, dan `aksara jelaskan` |
| K-26 | Prinsip kemudahan | Satu cara utama per tugas, tanpa operator kejutan, error menyebut lokasi dan perbaikan |
| K-27 | Uji ekosistem | Nightly 250 paket, mingguan 1000 paket, rilis 2000 paket, dashboard publik |

Keputusan yang masih terbuka dan harus selesai di Fase 0:

- Nama domain dan registry paket.
- Model konkurensi final setelah v1.0.
- Apakah `lambda` diganti `anonim` pada v2.0.
- Dukungan `except*` dan exception group pada v1.1.

### 2.1 Prinsip kemudahan

Aksara tidak mengejar jumlah fitur terbanyak. Aksara mengejar waktu tercepat dari "saya punya masalah" menjadi "program saya berjalan dan saya paham mengapa". Prinsip berikut mengikat desain sintaks, standard library, diagnostik, dan tooling.

1. **Satu tugas, satu cara utama.** Sinonim sintaks hanya ditambahkan jika bentuk baru lebih mudah dibaca, bukan sekadar variasi. `untuk setiap x dalam data` dan `untuk x dalam data` adalah bentuk yang sama, bukan dua semantik.
2. **Baca seperti kalimat.** `jika nilai antara 70 dan 90:` lebih mudah dipahami daripada `if 70 <= nilai <= 90:`.
3. **Tidak ada simbol misterius.** `&&`, `||`, `!`, `++`, `--`, `===`, dan `<>` ditolak dengan pesan yang menyebut penggantinya.
4. **Operator tidak mengubah arti secara diam-diam.** `??` hanya memeriksa `kosong`. `atau` memeriksa kebenaran. `?.` hanya melindungi `kosong`, bukan atribut yang hilang.
5. **Error adalah bagian dari bahasa.** Setiap error memiliki kode stabil, lokasi, baris sumber, penunjuk karakter, pesan biasa, saran, dan tautan dokumentasi.
6. **Fitur lanjutan bersifat opsional.** Program pertama cukup memakai variabel, `cetak`, `jika`, dan `untuk`. Kelas, async, dan jembatan Python tidak muncul sebelum dibutuhkan.
7. **Nama Indonesia untuk objek Aksara, nama asli untuk objek Python.** `daftar.tambah(x)` dan `requests.get(url)` hidup berdampingan tanpa lapisan terjemahan yang membingungkan.
8. **Indentasi 4 spasi, satu pernyataan per baris.** Titik koma dan penggabungan baris tetap didukung untuk kasus khusus, bukan gaya default.
9. **Tidak ada efek samping tersembunyi.** Perbandingan tidak mengubah nilai. Iterasi tidak menghapus elemen. Penugasan tidak memanggil fungsi yang tidak terlihat.
10. **Setiap fitur punya contoh, uji, dan alasan.** Fitur yang tidak dapat dijelaskan dalam satu contoh dan satu uji kepatuhan tidak masuk v1.0.

## 3. Arsitektur

### 3.1 Gambaran sistem

```
Berkas .aks
   |
   v
[Lekser Rust] -> [Parser Rust] -> AST
                                    |
                                    v
                         [Pemeriksa semantik Rust]
                                    |
                     +--------------+--------------+
                     |                             |
                     v                             v
          [Kompiler bytecode Rust]        [Interpreter referensi Rust]
                     |                             |
                     v                             v
              .akc bytecode                    hasil
                     |
                     v
          [Verifier bytecode C]
                     |
                     v
          [VM C] -> [Model objek C] -> [GC C]
                     |                       |
                     |                       +-> [Jembatan CPython C]
                     |                       +-> [ABI ekstensi C]
                     v
                  hasil
```

Tooling Rust berjalan di luar diagram inti: CLI, REPL, formatter, linter, LSP, debugger, dan package manager.

### 3.2 Pembagian peran C dan Rust

| Komponen | Bahasa | Alasan |
|---|---|---|
| Lekser, parser, AST | Rust | Enum bertipe dan pattern matching mengurangi bug |
| Pemeriksa semantik | Rust | Analisis scope dan alur kendali lebih mudah |
| Kompiler bytecode | Rust | Menghasilkan `.akc` tanpa menyentuh memori runtime |
| Interpreter referensi | Rust | Cepat dibangun, menjadi oracle semantik |
| VM bytecode | C | Kontrol layout objek, alokasi, dan GC |
| Model objek dan GC | C | Dibutuhkan oleh VM, ABI C, dan jembatan Python |
| Builtin native | C | Akses langsung ke OS dan CPython C API |
| Jembatan Python | C | CPython C API adalah C |
| ABI ekstensi | C | Header stabil untuk modul native |
| CLI, REPL, formatter, linter, LSP | Rust | Ekosistem crate dan keamanan memori |
| Package manager | Rust | Jaringan, arsip, dan resolusi dependensi |
| Build orchestration | Cargo, CMake, just | Setiap alat dipakai sesuai fungsinya |

**Mengapa VM di C, bukan di Rust.** Jembatan Python memakai C API. Menempatkan VM dan model objek di C menghindari lapisan FFI ganda pada setiap konversi objek. C juga memberi kontrol eksplisit atas layout, alokasi, dan GC. Risiko memori dikelola dengan verifier bytecode, sanitizer, fuzzing, dan pengujian diferensial.

**Mengapa front end di Rust.** AST adalah pohon bertipe. Pattern matching memaksa semua kasus ditangani. Parser, diagnostik, dan tooling dapat dibangun jauh lebih cepat. Front end tidak menyentuh memori runtime, sehingga batas FFI tetap sempit.

**Mengapa ada interpreter referensi.** VM C adalah komponen paling berisiko. Interpreter referensi Rust memberi oracle semantik yang dapat dijalankan sebelum VM stabil, mempercepat eksperimen fitur, dan menjadi pembanding pada uji diferensial. Setelah v1.0, interpreter referensi dibekukan dan hanya dipakai untuk pengujian.

### 3.3 Tiga kontrak stabil

1. **AST internal.** Tidak stabil, hanya dipakai crate Rust.
2. **Bytecode `.akc`.** Stabil sejak v0.3, berversi, dan diverifikasi sebelum dieksekusi.
3. **ABI C `aksara.h`.** Stabil sejak v1.0, dipakai modul native dan jembatan Python.

### 3.4 Alur build

- Cargo workspace untuk seluruh crate Rust.
- CMake untuk runtime C.
- `just` sebagai pintu masuk perintah: `just build`, `just test`, `just bench`.
- CLI Rust menautkan `libaksara_runtime.a` secara statis pada build rilis.
- Jembatan Python diaktifkan dengan `--fitur python` atau variabel `AKSARA_PYTHON=1`.
- Jika `libpython` tidak ditemukan, build tetap berhasil tanpa jembatan.
- Build rilis memakai `-O2`, LTO untuk Rust, dan `-flto` untuk C.

### 3.5 Model memori

- Mark-sweep, non-moving, precise roots.
- Tidak ada compaction pada v1.0. Pointer objek stabil, sehingga ABI C dan jembatan Python tidak perlu pinning.
- Root berasal dari stack VM, frame, global, modul, interned string, dan tabel root jembatan.
- Stack VM memakai shadow stack: setiap frame mendaftarkan slot hidup ke root set, sehingga GC tidak perlu memindai stack C secara konservatif.
- Pemanggilan native mengembalikan status error. Tidak ada `longjmp` yang melewati frame native; unwinding dilakukan oleh VM berdasarkan exception table.
- Finalizer dijalankan pada antrean terpisah setelah sweep.
- Weak reference didukung untuk cache dan observer.
- Mode stres `AKSARA_GC_STRESS=1` memaksa GC pada setiap alokasi untuk pengujian.
- Pause target: tidak ada pause lebih dari 50 ms pada heap 512 MB.

### 3.6 Format bytecode

- Stack-based, sekitar 160 opcode.
- Header berisi magic `AKS1`, versi, jumlah konstanta, ukuran kode, offset line table, offset exception table, dan checksum CRC32.
- Exception table memetakan rentang program counter ke handler, sehingga tidak perlu opcode `SETUP_EXCEPT`.
- Line table memetakan program counter ke baris dan kolom sumber.
- Verifier memeriksa kedalaman stack, target lompatan, indeks lokal, tipe konstanta, dan validitas exception table.
- Bytecode yang gagal verifikasi ditolak dengan `KesalahanBytecode`.

## 4. Spesifikasi bahasa

### 4.1 Leksikal

- Encoding UTF-8. BOM opsional di awal berkas.
- Baris baru `\n`, `\r\n`, dan `\r` dinormalisasi menjadi `\n`.
- Komentar dimulai dengan `#` sampai akhir baris.
- Indentasi memakai 4 spasi per level. Tab ditolak dengan diagnostik `E-INDENTASI`.
- Baris kosong dan baris berisi komentar diabaikan saat menentukan indentasi.
- Lanjutan baris eksplisit memakai `\`.
- Lanjutan implisit berlaku di dalam `()`, `[]`, dan `{}`.
- Pemisah pernyataan adalah baris baru atau `;`.
- Identifier memakai huruf Unicode, angka, dan garis bawah. Tidak boleh diawali angka. Case-sensitive.
- Kata kunci dicadangkan dan tidak dapat dipakai sebagai identifier.
- Kata kunci kontekstual (`setiap`, `sebanyak`, `sampai`, `langkah`) hanya memiliki arti khusus pada posisi sintaks tertentu dan tetap dapat dipakai sebagai identifier di luar posisi itu.
- `dari` dan `lainnya` adalah kata kunci tetap yang memiliki peran tambahan di dalam perulangan dan ternary.

Literal:

- Bilangan bulat: `123`, `0b1010`, `0o755`, `0xFF`, `1_000_000`.
- Pecahan: `3.14`, `1e-9`, `.5`.
- Imajiner: `2j`.
- Teks: `"..."`, `'...'`, `"""..."""`, `'''...'''`.
- Prefiks teks: `r"..."` untuk raw, `f"..."` untuk format, `b"..."` untuk bytes, `fr"..."` untuk gabungan.
- Boolean: `benar`, `salah`.
- Kosong: `kosong`.
- Elipsis: `...` untuk slice dan stub.

Contoh f-string:

```aksara
nama = "Ani"
umur = 25
cetak(f"Halo, {nama}. Umur: {umur:03d}")
```

### 4.2 Tipe data

| Tipe | Nama Aksara | Contoh | Mutabel |
|---|---|---|---|
| Kosong | `kosong` | `kosong` | tidak |
| Logika | `logika` | `benar`, `salah` | tidak |
| Bilangan bulat | `bilangan` | `42`, `-7` | tidak |
| Pecahan | `pecahan` | `3.14` | tidak |
| Kompleks | `kompleks` | `2+3j` | tidak |
| Teks | `teks` | `"halo"` | tidak |
| Bytes | `bytes` | `b"abc"` | tidak |
| Daftar | `daftar` | `[1, 2, 3]` | ya |
| Susunan | `susunan` | `susunan([1, 2, 3])` | ya |
| Tupel | `tupel` | `(1, 2)` | tidak |
| Kamus | `kamus` | `{"a": 1}` | ya |
| Himpunan | `himpunan` | `{1, 2}` | ya |
| Rentang | `rentang` | `rentang(5)` | tidak |
| Fungsi | `fungsi` | `fungsi f(): ...` | tidak |
| Kelas | `kelas` | `kelas A: ...` | tidak |
| Modul | `modul` | hasil `impor` | tidak |
| Berkas | `berkas` | hasil `buka()` | ya |
| Kesalahan | `kesalahan` | hasil `tangkap` | tidak |
| Objek Python | `objek_python` | hasil jembatan | tidak |

`susunan` adalah array numerik yang mendukung buffer protocol dan zero-copy dengan NumPy. Tipe ini dipakai untuk data besar, pengolahan gambar, dan operasi matriks. Jika NumPy tersedia melalui jembatan, `susunan` dapat membungkus `numpy.ndarray` tanpa menyalin data.

Aturan kebenaran:

- `kosong`, `salah`, `0`, `0.0`, `""`, `[]`, `()`, `{}`, dan himpunan kosong dianggap salah.
- Objek lain dianggap benar.
- `__bool__` dan `__len__` dihormati.

Aturan kesamaan:

- `==` memanggil `__eq__`.
- `adalah` membandingkan identitas.
- `hash()` harus konsisten dengan `__eq__`.
- `!=` memanggil `__ne__`, dengan fallback negasi `__eq__`.

### 4.3 Operator

Aksara memakai operator Python untuk hal yang sudah lazim, lalu menambah operator yang menghilangkan boilerplate. Setiap operator memiliki satu arti. Tidak ada operator yang berubah arti berdasarkan mode atau tipe secara implisit.

#### 4.3.1 Aritmetika

| Operator | Nama | Contoh | Hasil | Catatan |
|---|---|---|---|---|
| `+` | tambah | `2 + 3` | `5` | juga menggabungkan teks, daftar, dan tupel |
| `-` | kurang | `7 - 2` | `5` | |
| `*` | kali | `4 * 3` | `12` | juga mengulang teks dan koleksi |
| `/` | bagi | `7 / 2` | `3.5` | selalu menghasilkan pecahan |
| `//` | bagi bulat | `7 // 2` | `3` | membulatkan ke bawah |
| `%` | sisa bagi | `7 % 2` | `1` | modulo, bukan sisa gaya C |
| `**` | pangkat | `2 ** 10` | `1024` | asosiasi kanan |
| `@` | kali matriks | `A @ B` | matriks | memanggil `__matmul__` |
| `+x` | positif | `+5` | `5` | |
| `-x` | negatif | `-5` | `-5` | |
| `~x` | komplemen bit | `~5` | `-6` | hanya bilangan bulat |
| `absolut(x)` | nilai absolut | `absolut(-5)` | `5` | fungsi bawaan |
| `bagi_sisa(a, b)` | bagi dan sisa | `bagi_sisa(7, 2)` | `(3, 1)` | fungsi bawaan |
| `pangkat(a, b[, m])` | pangkat | `pangkat(2, 10)` | `1024` | fungsi bawaan, mendukung modulus |

Aturan numerik:

- Pembagian dengan nol melempar `KesalahanPembagianNol`, termasuk `0 / 0`, `0 // 0`, dan `0 % 0`.
- `//` membulatkan ke arah negatif tak hingga, bukan ke nol. `-7 // 2` bernilai `-4`.
- `%` mengikuti tanda pembagi. `-7 % 3` bernilai `2`.
- `**` dengan eksponen negatif menghasilkan pecahan. `2 ** -1` bernilai `0.5`.
- `@` hanya valid jika operand mendukung `__matmul__`. Untuk `susunan` dan `numpy.ndarray`, operasi diteruskan ke NumPy.
- `pangkat(a, b, m)` menghitung `(a ** b) % m` secara efisien dan aman untuk bilangan besar.

#### 4.3.2 Perbandingan

| Operator | Nama | Contoh | Catatan |
|---|---|---|---|
| `==` | sama dengan | `a == b` | memanggil `__eq__` |
| `!=` | tidak sama dengan | `a != b` | memanggil `__ne__`, fallback negasi `__eq__` |
| `<`, `<=`, `>`, `>=` | urutan | `a < b` | memanggil metode perbandingan yang sesuai |
| `dalam` | keanggotaan | `x dalam data` | memanggil `__contains__`, fallback iterasi |
| `bukan dalam` | bukan keanggotaan | `x bukan dalam data` | negasi dari `dalam` |
| `adalah` | identitas | `a adalah kosong` | membandingkan identitas objek |
| `bukan adalah` | bukan identitas | `a bukan adalah kosong` | negasi dari `adalah` |
| `antara a dan b` | rentang tertutup | `x antara 1 dan 10` | setara `a <= x dan x <= b` |
| `bukan antara a dan b` | di luar rentang | `x bukan antara 1 dan 10` | negasi dari `antara` |

Aturan perbandingan:

- `x antara a dan b` mengevaluasi `x`, `a`, dan `b` satu kali, lalu menguji `a <= x dan x <= b`. Jika `a > b`, hasilnya `salah`; nilai batas tidak ditukar secara otomatis.
- `bukan antara` adalah negasi logis dari `antara`, termasuk untuk `NaN`. `NaN bukan antara 1 dan 10` bernilai `benar` karena `NaN antara 1 dan 10` bernilai `salah`.
- `antara` bekerja untuk bilangan, pecahan, desimal, tanggal, teks dengan urutan leksikografis, dan objek yang mendukung `<=`.
- `antara` tidak dapat dirangkai dengan operator perbandingan lain. Tulis `(x antara 1 dan 10) dan (y antara 5 dan 15)`, bukan `x antara 1 dan 10 antara 5 dan 15`.
- `dan` di dalam `antara ... dan ...` adalah pemisah sintaks, bukan operator logika. `x antara 1 dan 10 dan y` dibaca sebagai `(x antara 1 dan 10) dan y`.
- Perbandingan biasa dapat dirangkai: `0 < x <= 10` setara dengan `0 < x dan x <= 10`, dan `x` dievaluasi satu kali.
- `adalah` untuk `kosong`, `benar`, dan `salah` adalah idiomatis. Untuk angka dan teks, gunakan `==`. Linter memperingatkan `x adalah 5` karena identitas objek kecil tidak dijamin.
- `==` pada `NaN` bernilai `salah`, sama seperti Python. Gunakan `matematika.bukan_angka(x)` untuk memeriksa `NaN`.

#### 4.3.3 Logika dan nilai kosong

| Operator | Nama | Contoh | Catatan |
|---|---|---|---|
| `dan` | dan logika | `a dan b` | short-circuit, mengembalikan operand |
| `atau` | atau logika | `a atau b` | short-circuit, mengembalikan operand |
| `bukan` | negasi logika | `bukan a` | selalu mengembalikan `benar` atau `salah` |
| `??` | pengganti kosong | `a ?? b` | mengembalikan `a` jika bukan `kosong`, selain itu `b` |
| `??=` | penugasan pengganti kosong | `a ??= b` | menugaskan `b` jika `a` belum ada atau `kosong` |
| `?.` | akses atribut aman | `a?.b` | `kosong` jika `a` adalah `kosong` |
| `?[]` | indeks aman | `a?[i]` | `kosong` jika `a` adalah `kosong` |
| `?()` | panggilan aman | `a?()` | `kosong` jika `a` adalah `kosong` |

Aturan logika:

- `dan` dan `atau` mengembalikan operand, bukan hanya `benar` atau `salah`. `0 atau "cadangan"` bernilai `"cadangan"`. `1 dan 2` bernilai `2`.
- `bukan` selalu mengembalikan `benar` atau `salah`.
- `??` hanya memeriksa `kosong`, bukan nilai salah. `0 ?? 5` bernilai `0`, `salah ?? benar` bernilai `salah`, dan `"" ?? "x"` bernilai `""`. Gunakan `atau` jika yang diinginkan adalah nilai cadangan untuk semua nilai salah.
- `??` dan `??=` melakukan short-circuit. Sisi kanan tidak dievaluasi jika tidak dibutuhkan.
- `a ?? b ?? c` dibaca sebagai `a ?? (b ?? c)`.

Aturan `??=`:

- Pada nama: jika nama belum terdefinisi di scope saat ini, nama didefinisikan dengan nilai kanan. Jika nama sudah ada dan nilainya `kosong`, nilai diganti. Jika nilainya bukan `kosong`, tidak ada perubahan.
- Pada atribut: jika atribut belum ada atau nilainya `kosong`, atribut diisi. Jika atribut ada dan bukan `kosong`, tidak ada perubahan.
- Pada subskrip kamus: jika kunci belum ada atau nilainya `kosong`, kunci diisi. Jika kunci ada dan bukan `kosong`, tidak ada perubahan.
- Pada subskrip lain: subskrip dievaluasi seperti biasa. Jika hasilnya `kosong`, nilai kanan ditugaskan. Jika indeks atau kunci tidak ada, kesalahan tetap dilempar.
- Target dievaluasi satu kali. `data[f()] ??= nilai` memanggil `f()` satu kali.

Aturan navigasi aman:

- `?.`, `?[]`, dan `?()` hanya melindungi nilai `kosong`. Atribut yang hilang tetap melempar `KesalahanAtribut`. Kunci yang hilang tetap melempar `KesalahanKunci`. Indeks di luar batas tetap melempar `KesalahanIndeks`. Objek yang tidak dapat dipanggil tetap melempar `KesalahanTipe`.
- Setiap operator aman memeriksa penerimanya sendiri. `a?.b.c` mengembalikan `kosong` jika `a` adalah `kosong`; jika `a.b` adalah `kosong`, `.c` tetap melempar kesalahan. Gunakan `a?.b?.c` jika keduanya perlu dilindungi.
- `a?.b?()` aman jika `a` kosong atau `a.b` kosong.
- Rantai aman tidak menelan kesalahan pemrograman. Ini disengaja agar bug tidak berubah menjadi `kosong` secara diam-diam.

#### 4.3.4 Bitwise dan himpunan

| Operator | Nama | Contoh | Catatan |
|---|---|---|---|
| `&` | DAN bit, irisan | `x & y` | bilangan bulat, `logika`, atau himpunan |
| `\|` | ATAU bit, gabungan | `x \| y` | bilangan bulat, `logika`, himpunan, atau tipe gabungan |
| `^` | XOR bit, selisih simetris | `x ^ y` | bilangan bulat, `logika`, atau himpunan |
| `~` | NOT bit | `~x` | hanya bilangan bulat |
| `<<` | geser kiri | `x << 2` | hanya bilangan bulat, hasil dapat membesar |
| `>>` | geser kanan | `x >> 2` | hanya bilangan bulat, mempertahankan tanda |

Catatan:

- Untuk himpunan, `&` adalah irisan, `\|` adalah gabungan, `^` adalah selisih simetris, dan `-` adalah selisih.
- Untuk `logika`, `&` adalah DAN, `\|` adalah ATAU, dan `^` adalah XOR eksklusif. Hasilnya selalu `logika`. `dan` dan `atau` tetap mengembalikan operand, sedangkan `&` dan `\|` selalu mengembalikan `benar` atau `salah`.
- `~x` setara dengan `-x - 1` untuk bilangan bulat.
- Geser dengan jumlah negatif melempar `KesalahanNilai`.
- Prioritas bitwise mengikuti Python: `&` lebih tinggi dari `^`, dan `^` lebih tinggi dari `\|`. Campurkan dengan tanda kurung jika melibatkan `dan`, `atau`, atau perbandingan.

#### 4.3.5 Penugasan, unpacking, dan anotasi

| Bentuk | Nama | Contoh | Catatan |
|---|---|---|---|
| `=` | penugasan | `x = 1` | pernyataan, bukan ekspresi |
| `+=`, `-=`, `*=`, `/=`, `//=`, `%=`, `**=`, `@=` | penugasan gabungan | `x += 1` | target dievaluasi satu kali |
| `&=`, `\|=`, `^=`, `<<=`, `>>=` | penugasan bitwise | `mask \|= bit` | hanya tipe yang mendukung |
| `??=` | penugasan pengganti kosong | `x ??= 10` | lihat aturan di 4.3.3 |
| `:=` | walrus | `jika (n := panjang(data)) > 0:` | ekspresi, mengikat di scope saat ini |
| `a, b = ...` | unpacking | `a, b = 1, 2` | jumlah harus cocok |
| `kepala, *ekor = ...` | star unpacking | `kepala, *ekor = data` | satu bintang per target |
| `*args` | unpacking posisional | `f(*args)` | pada panggilan fungsi |
| `**kwargs` | unpacking kata kunci | `f(**opsi)` | pada panggilan fungsi |
| `[*a, *b]` | unpacking iterable | `[*a, *b]` | pada literal daftar |
| `{**a, **b}` | unpacking kamus | `{**a, **b}` | kunci kanan menimpa kunci kiri |
| `x: T = v` | penugasan beranotasi | `x: bilangan = 5` | anotasi tidak ditegakkan saat runtime |
| `a = b = 0` | penugasan berantai | `a = b = 0` | semua nama menunjuk objek yang sama |

Aturan penugasan:

- Penugasan adalah pernyataan. `jika x = 1:` adalah kesalahan sintaks dengan saran `==` atau `:=`.
- Penugasan gabungan mengevaluasi target satu kali. `data[f()] += 1` memanggil `f()` satu kali.
- `+=` pada objek mutable dapat memutasi objek di tempat. `daftar += [1]` memperluas daftar yang sama, sedangkan `daftar = daftar + [1]` membuat daftar baru.
- `:=` mengikat nama di scope yang melingkupi ekspresi. Di dalam comprehension, nama yang diikat `:=` terlihat di scope luar, bukan scope comprehension. Linter menyarankan tanda kurung agar maksudnya jelas.
- `:=` tidak boleh dipakai untuk menggantikan penugasan biasa hanya demi mempersingkat baris.
- Tidak ada operator `++` atau `--`. Gunakan `x += 1` dan `x -= 1`.
- Tidak ada operator koma. `a, b` adalah tupel.
- Tidak ada `===` atau `!==`. Gunakan `==` untuk kesamaan nilai dan `adalah` untuk identitas.
- Anotasi tipe tidak mengubah perilaku runtime. Pemeriksaan tipe hanya berjalan dengan `aksara periksa --tipe`.

#### 4.3.6 Operator yang bergantung konteks

| Simbol | Konteks | Arti | Contoh |
|---|---|---|---|
| `@` | awal baris sebelum `fungsi` atau `kelas` | dekorator | `@properti` |
| `@` | di dalam ekspresi | kali matriks | `A @ B` |
| `:` | setelah nama dalam pernyataan | anotasi tipe | `x: bilangan` |
| `:` | di dalam `[]` | slice | `data[1:5]` |
| `:` | akhir header blok | pembuka blok | `jika x:` |
| `->` | setelah parameter fungsi | anotasi nilai kembali | `fungsi f() -> teks:` |
| `...` | ekspresi | elipsis | `fungsi stub(): ...` |
| `;` | akhir pernyataan | pemisah pernyataan | `a = 1; b = 2` |
| `\` | akhir baris | lanjutan baris eksplisit | `total = a + \` |
| `\|` | di dalam anotasi tipe | tipe gabungan | `bilangan \| kosong` |
| `*` | di dalam daftar parameter | penanda keyword-only | `fungsi f(a, *, b)` |
| `/` | di dalam daftar parameter | penanda positional-only | `fungsi f(a, /, b)` |
| `**` | di dalam daftar parameter | penampung kata kunci | `fungsi f(**opsi)` |
| `?` | hanya sebagai bagian `??`, `??=`, `?.`, `?[]`, `?()` | operator kosong dan aman | `a ?? b` |

Catatan:

- `@dekorator` harus berada di awal baris logis. `a @ b` di dalam ekspresi adalah kali matriks.
- `A | B` di dalam anotasi menghasilkan tipe gabungan. Di luar anotasi, `|` adalah operator bitwise atau gabungan himpunan.
- `?` bukan operator ternary. Ternary memakai `x jika kondisi lainnya y`.
- `!` bukan operator. Gunakan `bukan`.
- `&&` dan `||` bukan operator. Gunakan `dan` dan `atau`.

#### 4.3.7 Prioritas lengkap

Prioritas dari rendah ke tinggi:

| Prioritas | Operator | Asosiasi |
|---|---|---|
| 1 | `lambda` | kanan |
| 2 | `:=` | kanan, satu target per ekspresi |
| 3 | `x jika kondisi lainnya y` | kanan |
| 4 | `??` | kanan |
| 5 | `atau` | kiri |
| 6 | `dan` | kiri |
| 7 | `bukan` | kanan |
| 8 | `==`, `!=`, `<`, `<=`, `>`, `>=`, `dalam`, `bukan dalam`, `adalah`, `bukan adalah`, `antara ... dan ...`, `bukan antara ... dan ...` | berantai, `antara` tidak dapat dirangkai |
| 9 | `\|` | kiri |
| 10 | `^` | kiri |
| 11 | `&` | kiri |
| 12 | `<<`, `>>` | kiri |
| 13 | `+`, `-` | kiri |
| 14 | `*`, `/`, `//`, `%`, `@` | kiri |
| 15 | `+x`, `-x`, `~x` | kanan |
| 16 | `**` | kanan |
| 17 | `tunggu` | kanan |
| 18 | `()`, `[]`, `.`, `?.`, `?[]`, `?()` | kiri |

Catatan prioritas:

- `**` mengikat lebih kuat daripada operator uner di sebelah kiri. `-2 ** 2` bernilai `-4`, bukan `4`.
- `bukan` lebih rendah daripada perbandingan. `bukan x == y` dibaca sebagai `bukan (x == y)`.
- `antara` lebih rendah daripada `dan` logika. `x antara 1 dan 10 dan y` dibaca sebagai `(x antara 1 dan 10) dan y`.
- Ternary dan `??` mengikat lebih kuat daripada `atau`. `a ?? b atau c` dibaca sebagai `(a ?? b) atau c`.
- Operator postfix mengikat paling kuat. `a?.b[0]()` dibaca sebagai `((a?.b)[0])()`.

#### 4.3.8 Contoh operator

```aksara
nilai = 85
status = "lulus" jika nilai >= 70 lainnya "gagal"

batas_atas = kosong
batas = batas_atas ?? 100

pengaturan = kosong
pengaturan ??= kamus()
pengaturan["mode"] ??= "produksi"

pengguna = kosong
nama = pengguna?.profil?.nama ?? "Tamu"

data = kosong
pertama = data?[0] ?? 0

dalam_rentang = nilai antara 0 dan 100
di_luar_rentang = nilai bukan antara 0 dan 100
berurutan = 0 < nilai <= 100

total = (10 + 5) * 2 ** 2
flag = (nilai > 0) dan (nilai < 100) atau bukan selesai
mask = (0b1010 & 0b1100) | 0b0001
```

#### 4.3.9 Kesalahan operator dan pesan bantuan

| Ditulis | Ditolak karena | Pesan dan saran |
|---|---|---|
| `x && y` | bukan operator Aksara | Gunakan `x dan y` |
| `x \|\| y` | bukan operator Aksara | Gunakan `x atau y` |
| `!x` | bukan operator Aksara | Gunakan `bukan x` |
| `x++` | tidak ada operator increment | Gunakan `x += 1` |
| `x--` | tidak ada operator decrement | Gunakan `x -= 1` |
| `x === y` | tidak ada kesamaan ketat | Gunakan `x == y` untuk nilai atau `x adalah y` untuk identitas |
| `x !== y` | tidak ada ketidaksamaan ketat | Gunakan `x != y` atau `x bukan adalah y` |
| `x <> y` | bukan operator Aksara | Gunakan `x != y` |
| `a ? b : c` | bukan ternary Aksara | Gunakan `b jika a lainnya c` |
| `jika x = 1:` | penugasan di dalam kondisi | Gunakan `x == 1` untuk perbandingan atau `(x := 1)` untuk penugasan |

Pesan di atas memiliki kode diagnostik stabil, misalnya `E-SINTAKS-014`, dan dapat dijelaskan dengan `aksara jelaskan E-SINTAKS-014`.

### 4.4 Pernyataan

Subskrip mendukung indeks tunggal, slice `[awal:akhir:langkah]`, dan beberapa indeks yang dipisahkan koma untuk objek yang mendukungnya, misalnya `gambar[100:300, 200:400]` pada NumPy. Indeks negatif dan slice negatif mengikuti aturan Python.

#### Penugasan

```aksara
x = 10
x += 1
a, b = 1, 2
kepala, *ekor = [1, 2, 3, 4]
x: bilangan = 5
a = b = 0
jika (n := panjang(data)) > 0:
    cetak(n)
```

Anotasi tipe tidak ditegakkan saat runtime kecuali mode `--periksa-tipe` dijalankan.

#### Percabangan

```aksara
jika nilai >= 90:
    cetak("A")
lainnya jika nilai >= 80:
    cetak("B")
lainnya:
    cetak("C")
```

Aksara juga menyediakan ternary yang dibaca seperti kalimat:

```aksara
status = "lulus" jika nilai >= 70 lainnya "gagal"
pesan = "besar" jika n > 100 lainnya "sedang" jika n > 10 lainnya "kecil"
```

Aturan ternary:

- Kondisi dievaluasi lebih dulu, lalu hanya cabang yang dipilih yang dievaluasi.
- Bentuknya `nilai_jika_benar jika kondisi lainnya nilai_jika_salah`.
- Ternary bersifat asosiatif kanan. Contoh kedua di atas dibaca sebagai `"besar" jika n > 100 lainnya ("sedang" jika n > 10 lainnya "kecil")`.
- `lainnya` pada ternary adalah kata kunci kontekstual. Ia tidak membuka blok.
- Ternary tidak boleh ditulis tanpa `lainnya`. Tidak ada bentuk satu cabang.
- Untuk kondisi bercabang lebih dari dua atau berisi beberapa pernyataan, gunakan `jika` biasa. Ternary hanya untuk ekspresi pendek.
- Tidak ada operator `? :` gaya C. Penulisan tersebut ditolak dengan saran bentuk Aksara.

#### Perulangan

```aksara
selama kondisi:
    kerjakan()
lainnya:
    cetak("kondisi menjadi salah")

untuk item dalam iterable:
    cetak(item)
lainnya:
    cetak("tidak ada hentikan")
```

`selama` dan `untuk` mendukung klausa `lainnya` dengan semantik yang sama seperti `while-else` dan `for-else` di Python. Klausa `lainnya` berjalan jika perulangan selesai tanpa `hentikan`.

Aksara menambahkan empat bentuk perulangan yang lebih dekat ke bahasa sehari-hari. Semua bentuk ini dikompilasi ke AST yang sama dengan `untuk` dan `rentang`, sehingga tidak ada biaya runtime tambahan.

```aksara
# Bentuk paling mudah dibaca
untuk setiap siswa dalam daftar_siswa:
    cetak(siswa.nama)

# Ulangi n kali, indeks 0 sampai n-1
untuk i sebanyak 5:
    cetak(i)

# Rentang tertutup, 1 sampai 5 ikut dihitung
untuk i dari 1 sampai 5:
    cetak(i)

# Arah menurun terdeteksi otomatis
untuk i dari 10 sampai 1:
    cetak(i)

# Langkah eksplisit
untuk i dari 1 sampai 10 langkah 2:
    cetak(i)
```

Aturan bentuk perulangan:

| Bentuk | Setara dengan | Kapan dipakai |
|---|---|---|
| `untuk setiap x dalam data:` | `untuk x dalam data:` | Default untuk pemula, dibaca seperti kalimat |
| `untuk x dalam data:` | Bentuk ringkas | Kode sehari-hari dan contoh yang ringkas |
| `untuk i sebanyak n:` | `untuk i dalam rentang(n):` | Mengulang sesuatu n kali tanpa peduli batas |
| `untuk i dari a sampai b:` | `rentang_tertutup(a, b)` | Rentang bilangan bulat yang kedua ujungnya ikut dihitung |
| `untuk i dari a sampai b langkah s:` | `rentang_tertutup(a, b, s)` | Rentang dengan langkah tertentu |

Aturan semantik:

- `sebanyak n` mengevaluasi `n` satu kali. `n` harus bilangan bulat. Jika `n <= 0`, perulangan tidak berjalan. Ini sama dengan `rentang(n)`.
- `dari a sampai b` mengevaluasi `a` dan `b` satu kali. Keduanya harus bilangan bulat. Kedua batas ikut dihitung.
- Jika `a <= b`, langkah default adalah `+1`. Jika `a > b`, langkah default adalah `-1`. Karena itu `untuk i dari 10 sampai 1:` berjalan menurun tanpa `langkah -1`.
- Jika `langkah s` ditulis, `s` tidak boleh nol. Jika arah `s` berlawanan dengan arah rentang, perulangan tidak berjalan. Contoh: `untuk i dari 1 sampai 10 langkah -1:` tidak menghasilkan iterasi.
- `untuk setiap` dan `untuk` menerima klausa `lainnya` dengan semantik yang sama.
- `setiap`, `sebanyak`, `sampai`, dan `langkah` adalah kata kunci kontekstual. Kata-kata ini hanya memiliki arti khusus di posisi setelah `untuk`. Di tempat lain, kata tersebut dapat dipakai sebagai nama variabel.
- `dari` tetap menjadi kata kunci impor. Di dalam `untuk i dari a sampai b`, parser mengenali `dari` dari posisi sintaks, bukan dari arti global.
- Bentuk `dari ... sampai ...` hanya untuk bilangan bulat. Untuk pecahan atau tanggal, gunakan `selama` dengan pembaruan eksplisit agar tidak ada pembulatan diam-diam.
- `rentang(a, b)` tetap setengah terbuka seperti Python. Perbedaan ini disengaja: `rentang` untuk kompatibilitas dan slice, `sampai` untuk pemula yang mengharapkan batas akhir ikut dihitung.

#### Kendali perulangan

```aksara
hentikan
lanjutkan
lewati
```

#### Eksepsi

```aksara
coba:
    nilai = bilangan("bukan angka")
tangkap KesalahanNilai sebagai e:
    cetak("Gagal:", e)
tangkap (KesalahanTipe, KesalahanKunci):
    cetak("Tipe atau kunci salah")
lainnya:
    cetak("Tidak ada kesalahan")
akhirnya:
    cetak("Selesai")
```

```aksara
lempar KesalahanNilai("pesan")
lempar KesalahanNilai("pesan") dari kesalahan_asli
```

#### Context manager

```aksara
dengan buka("data.txt", "r") sebagai berkas:
    untuk baris dalam berkas:
        cetak(baris.pangkas())

dengan buka("a.txt") sebagai a, buka("b.txt") sebagai b:
    cetak(a.baca(), b.baca())
```

#### Assert dan del

```aksara
pastikan x > 0, "x harus positif"
hapus x
hapus daftar[0]
```

#### Impor

```aksara
impor matematika
impor matematika sebagai mtk
dari matematika impor akar, pangkat
dari matematika impor *
impor python "cv2" sebagai cv
dari python "os.path" impor bergabung
```

#### Global dan nonlokal

```aksara
global penghitung
nonlokal nilai
```

#### Fungsi

```aksara
fungsi nama(parameter: tipe = nilai, *args, **kwargs) -> tipe:
    """Dokumentasi."""
    kembalikan nilai
```

#### Kelas

```aksara
@dekorator
kelas Nama(Induk1, Induk2, metaclass=Meta):
    """Dokumentasi."""
    atribut = nilai

    fungsi __init__(diri, parameter):
        diri.parameter = parameter
```

#### Asinkron

```aksara
asinkron fungsi ambil_data():
    hasil = tunggu operasi_jaringan()
    kembalikan hasil
```

#### Pencocokan pola

```aksara
cocokkan nilai:
    kasus 1:
        cetak("satu")
    kasus 2 | 3:
        cetak("dua atau tiga")
    kasus [x, y]:
        cetak(f"daftar dua elemen: {x}, {y}")
    kasus {"kunci": v}:
        cetak(f"kamus dengan kunci: {v}")
    kasus _:
        cetak("lainnya")
```

Pola mendukung penjaga kondisi: `kasus x jika x > 0:`. Pola juga dapat mengikat nama, memakai `|` untuk alternatif, dan mencocokkan kelas dengan `kasus Titik(x, y):`.

### 4.5 Fungsi

- Parameter posisional, keyword, default, `*args`, dan `**kwargs`.
- Parameter positional-only dengan `/` dan keyword-only dengan `*`.
- Anotasi tipe opsional.
- Closure dan `nonlokal`.
- Dekorator dengan sintaks `@`.
- Fungsi anonim `lambda`.
- Generator dengan `hasilkan`.
- `kembalikan` tanpa nilai mengembalikan `kosong`.
- Batas rekursi default 1000, dapat diubah lewat `sistem.batas_rekursi`.

Contoh lengkap:

```aksara
fungsi faktorial(n: bilangan) -> bilangan:
    jika n <= 1:
        kembalikan 1
    kembalikan n * faktorial(n - 1)

fungsi gabung(*bagian, pemisah=", ", **opsi):
    hasil = pemisah.gabung(bagian)
    jika opsi.ambil("huruf_besar"):
        hasil = hasil.huruf_besar()
    kembalikan hasil

cetak(gabung("a", "b", "c", huruf_besar=benar))
```

### 4.5.1 Metode inti tipe bawaan

Aturan penamaan:

- Objek bawaan Aksara memakai nama metode bahasa Indonesia.
- Objek Python yang diakses melalui jembatan mempertahankan nama metode Python aslinya.
- Nama metode khusus (`__init__`, `__str__`, dan seterusnya) tetap memakai nama Python.

Metode yang paling sering dipakai:

| Tipe | Metode Aksara | Padanan Python |
|---|---|---|
| `daftar` | `tambah(x)` | `append` |
| `daftar` | `perluas(iterable)` | `extend` |
| `daftar` | `sisip(i, x)` | `insert` |
| `daftar` | `hapus(x)` | `remove` |
| `daftar` | `keluarkan([i])` | `pop` |
| `daftar` | `indeks(x)` | `index` |
| `daftar` | `hitung(x)` | `count` |
| `daftar` | `urutkan(kunci=kosong, terbalik=salah)` | `sort` |
| `daftar` | `balik()` | `reverse` |
| `daftar` | `salin()` | `copy` |
| `daftar` | `bersihkan()` | `clear` |
| `kamus` | `ambil(kunci, bawaan=kosong)` | `get` |
| `kamus` | `kunci()` | `keys` |
| `kamus` | `nilai()` | `values` |
| `kamus` | `item()` | `items` |
| `kamus` | `perbarui(kamus_lain)` | `update` |
| `kamus` | `keluarkan(kunci[, bawaan])` | `pop` |
| `kamus` | `salin()` | `copy` |
| `himpunan` | `tambah(x)` | `add` |
| `himpunan` | `hapus(x)` | `remove` |
| `himpunan` | `buang(x)` | `discard` |
| `himpunan` | `gabungan(other)` | `union` |
| `himpunan` | `irisan(other)` | `intersection` |
| `himpunan` | `selisih(other)` | `difference` |
| `teks` | `huruf_besar()` | `upper` |
| `teks` | `huruf_kecil()` | `lower` |
| `teks` | `judul()` | `title` |
| `teks` | `pangkas([karakter])` | `strip` |
| `teks` | `ganti(lama, baru[, jumlah])` | `replace` |
| `teks` | `pisah([pemisah])` | `split` |
| `teks` | `gabung(iterable)` | `join` |
| `teks` | `mulai_dengan(awalan)` | `startswith` |
| `teks` | `berakhir_dengan(akhiran)` | `endswith` |
| `teks` | `temukan(sub)` | `find` |
| `teks` | `format(*args, **kwargs)` | `format` |
| `berkas` | `baca([ukuran])` | `read` |
| `berkas` | `baca_baris()` | `readline` |
| `berkas` | `tulis(teks)` | `write` |
| `berkas` | `tutup()` | `close` |
| `berkas` | `posisi()` | `tell` |
| `berkas` | `cari(offset)` | `seek` |

Daftar lengkap akan ditetapkan di `docs/spesifikasi/tipe.md`. Setiap metode memiliki uji kepatuhan sendiri.

### 4.6 Kelas dan OOP

- `kelas` mendukung pewarisan ganda.
- Urutan resolusi metode memakai C3 linearization.
- `diri` merujuk instance, `induk` merujuk kelas induk.
- Metode instance, `@metode_kelas`, `@metode_statis`, dan `@properti`.
- Metode khusus memakai nama Python: `__init__`, `__str__`, `__repr__`, `__len__`, `__iter__`, `__next__`, `__getitem__`, `__setitem__`, `__enter__`, `__exit__`, `__eq__`, `__hash__`, `__call__`, dan seterusnya.
- Deskriptor, `__slots__`, metaclass, dan abstract base class didukung pada Fase 4.
- `kelas_data` untuk dataclass, `tipe_pilihan` untuk enum, dan `abstrak` untuk abstract base class tersedia di standard library.

Contoh:

```aksara
kelas Hewan:
    jumlah = 0

    fungsi __init__(diri, nama: teks):
        diri.nama = nama
        Hewan.jumlah += 1

    fungsi bersuara(diri) -> teks:
        kembalikan "..."

    @properti
    fungsi nama_tampilan(diri) -> teks:
        kembalikan diri.nama.huruf_besar()

    @metode_kelas
    fungsi berapa_banyak(kelas_ini):
        kembalikan kelas_ini.jumlah

kelas Anjing(Hewan):
    fungsi __init__(diri, nama, ras):
        induk().__init__(nama)
        diri.ras = ras

    fungsi bersuara(diri) -> teks:
        kembalikan f"{diri.nama} menggonggong"

hewan = [Anjing("Buddy", "Golden"), Anjing("Rex", "Husky")]
untuk satu dalam hewan:
    cetak(satu.bersuara())

cetak(Hewan.berapa_banyak())
```

### 4.7 Eksepsi

Hierarki bawaan:

```
DasarKesalahan
├── Kesalahan
│   ├── KesalahanSistem
│   │   ├── KesalahanIO
│   │   ├── KesalahanBerkasTidakDitemukan
│   │   ├── KesalahanIzin
│   │   ├── KesalahanWaktuJalan
│   │   │   └── KesalahanRekursi
│   │   ├── KesalahanBytecode
│   │   └── KesalahanFiturTidakTersedia
│   ├── KesalahanNilai
│   ├── KesalahanTipe
│   ├── KesalahanNama
│   ├── KesalahanAtribut
│   ├── KesalahanIndeks
│   ├── KesalahanKunci
│   ├── KesalahanImpor
│   ├── KesalahanPembagianNol
│   ├── KesalahanPastikan
│   ├── KesalahanBelumDiimplementasikan
│   ├── KesalahanMemori
│   ├── HentikanIterasi
│   └── KesalahanPython
├── KeluarSistem
├── GangguanKetik
└── KeluarGenerator
```

`tangkap` tanpa tipe menangkap `Kesalahan`, bukan `DasarKesalahan`. `KeluarSistem`, `GangguanKetik`, dan `KeluarGenerator` hanya tertangkap jika disebut eksplisit, sama seperti Python.

`KesalahanPython` membungkus eksepsi dari CPython dan menyimpan tipe asli, pesan, serta traceback sebagai atribut.

### 4.8 Modul dan paket

- Satu berkas `.aks` adalah modul.
- Direktori dengan `__init__.aks` adalah paket.
- Urutan pencarian modul:
  1. Direktori berkas yang sedang dijalankan.
  2. Direktori `sumber/` proyek.
  3. Jalur pada `AKSARA_PATH`.
  4. Standard library.
  5. Cache paket di `~/.aksara/paket`.
- Atribut modul: `__nama__`, `__berkas__`, `__paket__`, `__dok__`, dan `__semua__` untuk daftar ekspor.
- Nama atribut modul memakai bahasa Indonesia. Nama metode khusus (`__init__`, `__str__`) tetap memakai bahasa Python agar jembatan dan protokol objek tetap kompatibel.
- Titik masuk program memakai `jika __nama__ == "__utama__":`.

### 4.9 Konkurensi

- v1.0 menyediakan `utas`, `kunci`, `antrean`, dan dukungan asinkron dasar.
- VM memakai satu global interpreter lock untuk menyederhanakan model objek dan jembatan Python.
- GIL VM dan GIL CPython adalah dua lock terpisah. Urutan akuisisi selalu GIL VM lalu GIL CPython; tidak ada jalur yang mengambil keduanya dalam urutan terbalik.
- Paralelisme CPU tidak menjadi target v1.0. Pekerjaan CPU-bound sebaiknya memakai proses atau library native.
- `alur_asinkron` menyediakan event loop dasar, `tunggu`, `asinkron untuk`, dan `asinkron dengan`.
- Uji konkurensi memakai TSan dan skenario deadlock terkontrol.

### 4.10 Anotasi tipe

- Anotasi opsional dan tidak ditegakkan saat runtime.
- `aksara periksa --tipe` menjalankan pemeriksaan statis bertahap.
- Tipe yang tersedia: `bilangan`, `pecahan`, `kompleks`, `teks`, `logika`, `kosong`, `daftar[T]`, `kamus[K, V]`, `tupel[T, ...]`, `himpunan[T]`, `fungsi[[A, B], C]`, `tipe[T]`, dan gabungan `A | B`.
- Modul `sistem_tipe` menyediakan alias, generik, `Protokol`, `Uni`, dan `Opsional`.

### 4.11 Contoh program lengkap

```aksara
# contoh/pengelola_nilai.aks
kelas NilaiTidakValid(KesalahanNilai):
    lewati

kelas Mahasiswa:
    fungsi __init__(diri, nama: teks, nilai: daftar[bilangan]):
        diri.nama = nama
        diri.nilai = nilai

    fungsi rata_rata(diri) -> pecahan:
        jika panjang(diri.nilai) == 0:
            lempar NilaiTidakValid("nilai kosong")
        kembalikan jumlah(diri.nilai) / panjang(diri.nilai)

    fungsi lulus(diri, batas: bilangan = 70) -> logika:
        kembalikan diri.rata_rata() >= batas

    fungsi __str__(diri) -> teks:
        kembalikan f"{diri.nama}: {diri.rata_rata():.2f}"


fungsi olah_data():
    data = [
        Mahasiswa("Ani", [90, 85, 88]),
        Mahasiswa("Budi", [70, 65, 72]),
        Mahasiswa("Citra", [95, 92, 98]),
    ]

    lulus = []
    untuk m dalam data:
        coba:
            jika m.lulus():
                lulus.tambah(m)
        tangkap NilaiTidakValid sebagai e:
            cetak(f"Data {m.nama} tidak valid: {e}")

    lulus.urutkan(kunci=lambda m: m.rata_rata(), terbalik=benar)
    untuk m dalam lulus:
        cetak(m)


jika __nama__ == "__utama__":
    olah_data()
```

### 4.12 Konvensi penulisan

Konvensi ini dipakai oleh formatter, linter, dan contoh resmi. Formatter tidak memaksa semua aturan, tetapi linter memberi peringatan.

| Aspek | Aturan | Contoh |
|---|---|---|
| Indentasi | 4 spasi per level, tab ditolak | `    cetak("halo")` |
| Panjang baris | Maksimum 88 karakter | |
| Nama variabel | Huruf kecil dengan garis bawah | `total_nilai` |
| Nama fungsi | Huruf kecil dengan garis bawah, kata kerja | `hitung_rata_rata` |
| Nama kelas | PascalCase | `PengelolaNilai` |
| Nama konstanta | HURUF_BESAR dengan garis bawah | `BATAS_LULUS` |
| Nama privat | Awalan garis bawah | `_hitung_internal` |
| Nama metode khusus | Nama Python dengan dua garis bawah | `__init__`, `__str__` |
| Baris kosong | Dua baris antar definisi tingkat atas, satu baris antar metode | |
| Impor | Di atas berkas, dikelompokkan: bawaan, paket Aksara, jembatan Python | |
| Impor bintang | Dilarang di kode produksi | Gunakan `dari modul impor nama` |
| Sumber daya | Gunakan `dengan` untuk berkas, soket, dan koneksi | `dengan buka(...) sebagai f:` |
| Format teks | Gunakan f-string untuk penyisipan sederhana | `f"Halo, {nama}"` |
| Argumen default | Jangan memakai koleksi mutable | Gunakan `kosong` lalu buat di dalam fungsi |
| Eksepsi | Tangkap tipe spesifik | `tangkap KesalahanNilai:` |
| Dokumentasi | Docstring untuk fungsi dan kelas publik | `"""Menghitung rata-rata."""` |
| Komentar | Jelaskan alasan, bukan mengulang kode | `# Hindari pembagian nol.` |
| Anotasi tipe | Dianjurkan untuk API publik | `fungsi f(x: bilangan) -> teks:` |
| Jalur berkas | Gunakan `jalur` | `jalur("data") / "nilai.csv"` |
| Iterasi | Gunakan `enumerasi` atau `pasangkan` | `untuk i, x dalam enumerasi(data):` |

Contoh gaya yang baik:

```aksara
BATAS_LULUS = 70

fungsi hitung_rata_rata(nilai: daftar[pecahan]) -> pecahan:
    """Mengembalikan rata-rata nilai, atau 0 jika daftar kosong."""
    jika bukan nilai:
        kembalikan 0.0
    kembalikan jumlah(nilai) / panjang(nilai)

fungsi siswa_lulus(nilai: daftar[pecahan]) -> logika:
    """Memeriksa apakah nilai memenuhi batas kelulusan."""
    kembalikan hitung_rata_rata(nilai) >= BATAS_LULUS
```

Contoh gaya yang kurang baik dan alasannya:

```aksara
# Kurang baik: argumen default mutable, indeks manual, dan pembagian nol.
fungsi hitung(nilai=[]):
    total = 0
    untuk i dalam rentang(panjang(nilai)):
        total += nilai[i]
    kembalikan total / panjang(nilai)
```

Perbaikan:

```aksara
fungsi hitung(nilai: daftar[pecahan] | kosong = kosong) -> pecahan:
    """Mengembalikan rata-rata, atau 0 jika tidak ada nilai."""
    jika nilai adalah kosong atau panjang(nilai) == 0:
        kembalikan 0.0
    kembalikan jumlah(nilai) / panjang(nilai)
```

### 4.13 Kurikulum pemula 10 pelajaran

Kurikulum ini menjadi acuan tutorial resmi dan urutan contoh di `contoh/pemula/`. Setiap pelajaran memiliki satu contoh, tiga latihan, satu kesalahan umum, dan satu proyek mini.

| Pelajaran | Konsep | Program contoh | Latihan |
|---|---|---|---|
| 1 | `cetak`, komentar, variabel, f-string | Sapaan dan biodata | Cetak profil, ubah variabel, gabungkan teks |
| 2 | Tipe data dan konversi | Kalkulator umur | Konversi teks ke bilangan, cek tipe, format angka |
| 3 | Operator aritmetika, perbandingan, `antara`, `??`, ternary | Kalkulator diskon | Hitung pajak, cek rentang, pilih nilai default |
| 4 | `jika`, `lainnya jika`, `lainnya`, truthiness | Penentu nilai huruf | Klasifikasi umur, cek kelulusan, validasi input |
| 5 | `untuk`, `selama`, `sebanyak`, `dari ... sampai ...`, `hentikan`, `lanjutkan` | Tabel perkalian | Segitiga bintang, tebak angka, jumlah deret |
| 6 | `daftar`, `kamus`, `himpunan`, comprehension | Daftar belanja | Hitung total, cari duplikat, kelompokkan data |
| 7 | `fungsi`, parameter, default, `*args`, `**kwargs`, lambda | Konversi suhu | Fungsi statistik, fungsi validasi, fungsi komposisi |
| 8 | `kelas`, `diri`, `__init__`, metode, properti, pewarisan | Inventaris toko | Kelas rekening, kelas hewan, kelas karyawan |
| 9 | `coba`, `tangkap`, `akhirnya`, `dengan`, `buka`, `lempar` | Pembaca CSV dengan validasi | Baca berkas, tangani input salah, tulis laporan |
| 10 | `impor`, paket, `impor python`, venv, pip | Scraper atau analisis CSV | Pasang paket, panggil library Python, buat proyek kecil |

Aturan kurikulum:

- Pelajaran 1 sampai 7 tidak memakai kelas. Pelajaran 8 memperkenalkan OOP setelah pembaca nyaman dengan fungsi dan koleksi.
- Async, pattern matching, dan jembatan Python tidak muncul sebelum pelajaran 10.
- Setiap pelajaran harus dapat diselesaikan dalam 60 sampai 90 menit.
- Setiap latihan memiliki solusi yang dapat dijalankan dan diuji di CI.
- Kesalahan umum di setiap pelajaran harus memiliki kode diagnostik yang benar-benar muncul.
- Proyek mini terakhir menggabungkan setidaknya lima konsep dari pelajaran sebelumnya.

### 4.14 Diagnostik ramah pemula

Diagnostik adalah fitur bahasa, bukan pesan tambahan. Semua error runtime dan error parser memakai format yang sama.

```text
KesalahanNama: nama 'jumblah' belum didefinisikan
  --> contoh.aks:3:13
   |
 3 |     total = jumblah + 1
   |             ^^^^^^^
   |
Saran: apakah maksud Anda 'jumlah'?
Bantuan: aksara jelaskan E-NAMA-001
```

Aturan diagnostik:

- Pesan memakai bahasa Indonesia, kalimat pendek, dan kata kerja aktif.
- Pesan menyebut token yang salah, lokasi baris dan kolom, serta harapan konkret.
- Saran harus dapat langsung diterapkan. "Periksa kembali kode Anda" bukan saran.
- Nama yang mirip dideteksi dengan jarak edit dan resolusi scope. Saran tidak boleh mengubah semantik secara diam-diam.
- Untuk nama yang belum didefinisikan, tampilkan definisi terdekat di scope yang sama jika ada.
- Untuk tipe yang salah, tampilkan tipe yang diharapkan dan tipe yang diterima.
- Untuk impor yang gagal, tampilkan jalur yang dicari dan perintah pemasangan yang mungkin.
- Untuk kesalahan Python, tampilkan tipe asli, pesan asli, dan satu baris konteks jembatan. Traceback Python lengkap hanya muncul dengan `--jejak`.
- Secara default, traceback internal VM disembunyikan. Pengguna melihat frame berkas `.aks` miliknya sendiri.
- `--ringkas` menampilkan satu baris error untuk CI. `--diagnostik=json` menghasilkan objek terstruktur untuk editor dan alat otomasi.
- Kode error stabil selama satu siklus mayor. Kode tidak berubah hanya karena teks pesan diperbaiki.
- Setiap kode error memiliki halaman dokumentasi, contoh salah, contoh benar, dan versi minimum bahasa.

Kategori kode error:

| Kode | Kategori | Contoh |
|---|---|---|
| `E-LEKS-*` | karakter atau literal tidak valid | `E-LEKS-003` |
| `E-INDENTASI-*` | indentasi tidak konsisten | `E-INDENTASI-001` |
| `E-SINTAKS-*` | susunan token tidak valid | `E-SINTAKS-014` |
| `E-NAMA-*` | nama tidak dikenal atau belum didefinisikan | `E-NAMA-001` |
| `E-TIPE-*` | tipe tidak cocok | `E-TIPE-002` |
| `E-NILAI-*` | nilai tidak valid | `E-NILAI-003` |
| `E-ATRIBUT-*` | atribut tidak ada | `E-ATRIBUT-001` |
| `E-INDEKS-*` | indeks di luar batas | `E-INDEKS-001` |
| `E-KUNCI-*` | kunci kamus tidak ada | `E-KUNCI-001` |
| `E-IMPOR-*` | modul tidak ditemukan | `E-IMPOR-002` |
| `E-PYTHON-*` | kesalahan dari CPython | `E-PYTHON-001` |
| `E-BYTECODE-*` | bytecode rusak atau tidak kompatibel | `E-BYTECODE-001` |
| `E-JEMBATAN-*` | protokol isolasi gagal | `E-JEMBATAN-004` |
| `E-IO-*` | berkas, jaringan, atau izin | `E-IO-001` |

Prinsip pesan:

- Jangan menyalahkan pengguna. Sebut keadaan, bukan sifat orang.
- Jangan memakai istilah internal seperti "opcode", "slot", atau "vtable" kecuali pengguna meminta detail.
- Jika ada dua perbaikan yang valid, tampilkan keduanya dan jelaskan perbedaannya.
- Jika error disebabkan library Python, katakan demikian dan jangan menyalahkan sintaks Aksara.
- Jika error disebabkan versi Python yang tidak didukung, tampilkan versi yang terdeteksi dan versi yang didukung.

### 4.15 Larangan gaya yang membingungkan

Linter menolak atau memperingatkan pola berikut:

- Blok satu baris seperti `jika x: kerjakan()`. Tulis header, baris baru, lalu indentasi.
- Argumen default berupa `daftar`, `kamus`, atau `himpunan`.
- `tangkap:` tanpa tipe, kecuali di batas atas program dengan komentar alasan.
- `tangkap ...: lewati` tanpa komentar.
- `global` di dalam fungsi yang tidak benar-benar membutuhkan mutasi global.
- `adalah` untuk membandingkan angka, teks, atau nilai yang bukan singleton.
- `== kosong`. Gunakan `adalah kosong`.
- `dari modul impor *` di kode produksi.
- Nama satu huruf selain `i`, `j`, `x`, dan `y` pada loop pendek.
- Sarang blok lebih dari empat level. Pecah menjadi fungsi.
- Angka ajaib tanpa nama konstanta.
- `evaluasi` atau `jalankan` pada masukan yang tidak tepercaya.
- Perbandingan `== benar` atau `== salah`. Gunakan nilai langsung atau `bukan`.
- `kamus.ambil(kunci, [])` dengan koleksi mutable sebagai nilai bawaan.
- Menangkap `Kesalahan` di library lalu melanjutkan seolah tidak terjadi apa-apa.

## 5. Kompatibilitas Python

### 5.1 Tiga tingkat kompatibilitas

| Tingkat | Mekanisme | Kapan | Kelebihan | Keterbatasan |
|---|---|---|---|---|
| 1 | Jembatan CPython | v0.5 | Library Python asli dapat dipakai | Overhead konversi, GIL, distribusi libpython |
| 2 | ABI ekstensi C Aksara | v1.0 | Modul native cepat, tanpa Python | Ekosistem harus dibangun sendiri |
| 3 | Transpiler ke Python | Pasca 1.0 | Kompatibilitas maksimum | Semantik dapat berbeda, performa CPython |

Jalur utama adalah Tingkat 1. Tingkat 1 dipecah menjadi Tier A, B, dan C pada Bagian 5.8 sesuai kemampuan paket dan mode eksekusi. Tingkat 2 disediakan untuk modul yang membutuhkan performa dan kontrol memori. Tingkat 3 hanya eksperimen dan bukan prioritas.

### 5.2 Sintaks jembatan

Jembatan dibuat eksplisit agar tidak ada ambiguitas antara modul Aksara dan modul Python.

```aksara
impor python "cv2" sebagai cv
impor python "pandas" sebagai pd
impor python "requests" sebagai requests
dari python "bs4" impor BeautifulSoup
dari python "os.path" impor bergabung
```

Aturan:

- `impor python "nama_modul"` memuat modul Python melalui `PyImport_ImportModule`.
- Alias wajib jika nama modul mengandung titik atau berbeda dari nama variabel.
- Modul Python di-cache per proses.
- `impor python` tanpa nama memuat namespace jembatan dan menginisialisasi CPython.
- Jika build tanpa jembatan, pernyataan ini melempar `KesalahanFiturTidakTersedia`.

### 5.3 Aturan marshaling

| Aksara | Python | Arah | Catatan |
|---|---|---|---|
| `kosong` | `None` | dua arah | |
| `logika` | `bool` | dua arah | |
| `bilangan` | `int` | dua arah | int64, bigint untuk nilai besar |
| `pecahan` | `float` | dua arah | |
| `kompleks` | `complex` | dua arah | |
| `teks` | `str` | dua arah | UTF-8 |
| `bytes` | `bytes` | dua arah | |
| `daftar` | `list` | dua arah | konversi by value |
| `tupel` | `tuple` | dua arah | konversi by value |
| `kamus` | `dict` | dua arah | kunci harus hashable |
| `himpunan` | `set` | dua arah | konversi by value |
| `rentang` | `range` | dua arah | konversi by value |
| `fungsi` | callable | Aksara ke Python | wrapper dengan GIL |
| `kelas` Aksara | class Python | belum didukung | gunakan proxy atau fungsi pembungkus |
| `objek_python` | objek apa pun | dua arah | proxy dengan refcount, internal `ObjekPython` |
| `susunan` | `numpy.ndarray` | dua arah | zero-copy melalui buffer protocol |

Aturan penting:

- Tipe primitif dan koleksi dikonversi by value. Mutasi koleksi hasil konversi tidak terlihat di sisi lain.
- Objek Python yang tidak dapat dikonversi menjadi tipe Aksara dibungkus sebagai `objek_python`.
- Untuk data besar, gunakan `susunan` agar tidak terjadi penyalinan.
- Identitas objek (`adalah`) pada `objek_python` mengacu pada identitas proxy, bukan identitas objek Python asli.
- Siklus lintas runtime tidak didukung pada v1.0. Jika terdeteksi, jembatan mencetak peringatan dan objek ditahan sampai proses selesai.

### 5.4 Perbedaan semantik yang disengaja

| Aspek | Python | Aksara | Alasan |
|---|---|---|---|
| Kata kunci | Inggris | Indonesia | Identitas bahasa |
| Fungsi bawaan | `print`, `len` | `cetak`, `panjang` | Konsistensi |
| Pesan kesalahan | Inggris | Indonesia | Pengalaman pengguna |
| Modul stdlib | `math`, `json` | `matematika`, `json` | Konsistensi |
| Metode khusus | `__init__` | `__init__` | Kompatibilitas jembatan |
| `lambda` | `lambda` | `lambda` | Familiaritas |
| Waktu penghancuran objek | Refcount deterministik | GC tidak deterministik | Model memori sederhana |
| Threading | GIL | GIL | Kompatibilitas dan kesederhanaan |
| `is` lintas jembatan | Identitas Python | Identitas proxy | Batas runtime |
| Mutasi koleksi lintas jembatan | Referensi | By value | Keamanan batas |

Perbedaan waktu penghancuran objek didokumentasikan. Pengguna harus memakai `dengan` atau `tutup()` untuk sumber daya seperti berkas, soket, dan koneksi basis data.

### 5.5 Matriks paritas fitur

| Fitur | Aksara | Fase |
|---|---|---|
| Variabel dan tipe dinamis | `nama = nilai` | 2 |
| Aritmetika dan perbandingan | Simbol sama, ditambah `antara` | 2 |
| Operator kosong dan aman | `??`, `??=`, `?.`, `?[]`, `?()` | 2 |
| Ternary | `x jika kondisi lainnya y` | 2 |
| `if`/`elif`/`else` | `jika`/`lainnya jika`/`lainnya` | 2 |
| `while`/`for` | `selama`/`untuk` | 2 |
| Perulangan bahasa alami | `untuk setiap`, `sebanyak`, `dari ... sampai ...` | 2 |
| Fungsi, default, `*args`, `**kwargs` | `fungsi` | 2 |
| Closure dan `nonlokal` | `nonlokal` | 2 |
| Daftar, tupel, kamus, himpunan | `daftar`, `tupel`, `kamus`, `himpunan` | 2 |
| Comprehension | Sintaks sama | 2 |
| Kelas dan pewarisan | `kelas` | 2 |
| Properti, `@metode_kelas`, `@metode_statis` | Dekorator | 2 |
| Dekorator | `@` | 2 |
| Eksepsi | `coba`/`tangkap`/`akhirnya` | 2 |
| Context manager | `dengan` | 2 |
| Generator | `hasilkan` | 2 |
| Modul dan paket | `impor`/`dari` | 2 |
| Diagnostik ber-kode | `E-*` dan `aksara jelaskan` | 1 |
| Pewarisan ganda dan C3 MRO | Didukung | 4 |
| Deskriptor dan `__slots__` | Didukung | 4 |
| Metaclass dan ABC | Didukung | 4 |
| `hasilkan dari`, `send`, `throw` | Didukung | 4 |
| Pencocokan pola | `cocokkan`/`kasus` | 4 |
| Async dan `tunggu` | `asinkron`/`tunggu` | 4 |
| Anotasi tipe | Opsional | 4 |
| `kelas_data`, `tipe_pilihan`, `sistem_tipe` | Stdlib | 5 |
| `sqlite3`, `logging`, `argparse` | Stdlib | 5 |
| NumPy, pandas, OpenCV, requests, bs4, lxml | Jembatan Tier A | 6 |
| Seluruh paket Python yang kompatibel | Jembatan in-process atau isolasi | 6 |
| pip, virtualenv, dan lockfile Python | `aksara python` | 6 |
| Uji ekosistem dan dashboard | 200+ paket, 1000+ mingguan | 6 |
| Kurikulum pemula | 10 pelajaran | 7 |

Fitur yang ditunda ke v1.1: `except*`, exception group, type parameter syntax (PEP 695), dan free-threaded Python.

### 5.6 Batasan jembatan

- CPython yang didukung: 3.11, 3.12, dan 3.13.
- Python free-threaded belum didukung.
- Subinterpreter belum didukung.
- Ekstensi C Python dapat dipakai melalui CPython, tetapi tidak dapat ditautkan langsung ke VM Aksara.
- Library yang membutuhkan fork, signal handler, atau thread khusus diuji secara terpisah.
- Distribusi harus menyertakan `libpython` yang kompatibel atau menemukannya di sistem.
- Jembatan bukan sandbox. Library Python dapat menjalankan kode native dan mengakses sistem berkas.
- Cross-runtime cycle dapat menyebabkan kebocoran memori. Batasi pada v1.0.

Daftar lengkap batasan, dampak, dan mitigasinya ada di Bagian 5.12. Ringkasan di atas hanya untuk pembaca yang membutuhkan batas cepat sebelum membaca kontrak ekosistem.

### 5.7 Alternatif yang ditolak

- **Menulis ulang seluruh standard library Python.** Waktu dan biaya tidak sebanding. Gunakan jembatan untuk library berat dan tulis stdlib inti dalam Aksara.
- **Menjalankan bytecode CPython.** Mengikat implementasi pada detail internal CPython dan menghilangkan identitas bahasa.
- **Transpiler sebagai jalur utama.** Semantik sulit dijaga dan debugging menghasilkan kode Python yang tidak informatif.
- **Subprocess bridge sebagai jalur utama.** Overhead tinggi dan tidak mendukung callback serta zero-copy. Tetap disediakan sebagai fallback.

### 5.8 Kontrak ekosistem penuh

"Dukungan penuh" bukan daftar nama paket. Dukungan penuh adalah kontrak kemampuan yang diuji. Sebuah paket dinyatakan didukung pada kombinasi sistem operasi, versi Python, dan mode jembatan tertentu jika lulus seluruh uji yang relevan untuk tier-nya.

Kemampuan yang diuji:

| Kemampuan | Bukti uji |
|---|---|
| Impor modul dan submodul | `impor python "paket"` dan `dari python "paket.sub" impor nama` |
| Membuat objek dan memanggil fungsi | Konstruktor, fungsi modul, metode instance, argumen posisional dan kata kunci |
| Pertukaran tipe primitif | `kosong`, `logika`, `bilangan`, `pecahan`, `kompleks`, `teks`, `bytes` |
| Pertukaran koleksi | `daftar`, `tupel`, `kamus`, `himpunan`, `rentang` |
| Eksepsi dua arah | Eksepsi Python menjadi `KesalahanPython`, eksepsi Aksara menjadi eksepsi Python |
| Iterator dan generator | `iterasi`, `berikutnya`, `untuk`, ekspresi generator, `hasilkan` |
| Context manager | `dengan`, `__enter__`, `__exit__`, penutupan sumber daya |
| Async | `tunggu`, awaitable, iterator asinkron, context manager asinkron |
| Callback | Fungsi Aksara dipanggil dari library Python, termasuk argumen dan nilai kembali |
| Buffer protocol | `susunan` dan `numpy.ndarray` tanpa salinan untuk data besar |
| Thread | Panggilan dari thread Python dan thread Aksara, dengan urutan GIL yang aman |
| Ekstensi native | Modul `.so`, `.pyd`, dan `.dylib` yang dibangun untuk CPython yang dipilih |
| Pembersihan sumber daya | Berkas, soket, koneksi basis data, dan objek native ditutup dengan benar |
| Isolasi | Impor dan panggilan tetap berfungsi saat worker terpisah, termasuk callback dan buffer |
| Packaging | Pemasangan melalui pip, virtualenv, wheel, dan lockfile |

Tier ekosistem:

| Tier | Mode | Cakupan | Irama uji | Contoh tipikal |
|---|---|---|---|---|
| A | In-process | Semua kemampuan, callback, zero-copy, thread | Setiap commit dan setiap rilis | NumPy, pandas, requests, BeautifulSoup, lxml, OpenCV headless, SQLAlchemy, Pillow |
| B | In-process dengan batasan | Sebagian besar kemampuan, batasan didokumentasikan | Nightly | matplotlib dengan GUI, scrapy, playwright, paket berat dengan event loop atau proses sendiri |
| C | Isolasi subprocess | Impor, panggilan, iterator, eksepsi, buffer melalui shared memory | Mingguan | Paket yang memakai fork, signal, atau asumsi `sys.executable` |
| D | Tidak didukung | Alasan teknis spesifik dan tanggal tinjauan | Kuartalan | Free-threaded CPython, subinterpreter, arsitektur 32-bit |

Aturan tier:

- Tier ditentukan oleh hasil uji dan probe kemampuan, bukan oleh nama paket atau popularitas.
- Satu paket dapat berada di Tier A pada Linux x86_64 dan Tier B pada Windows, atau berbeda antar versi Python.
- Penurunan tier wajib memiliki issue, alasan teknis, dan tanggal tinjauan. Kenaikan tier wajib memiliki bukti uji yang tersimpan.
- Tidak ada daftar putih. Paket baru yang dilaporkan pengguna masuk korpus uji paling lambat tujuh hari kerja.
- Kegagalan paket diperlakukan sebagai bug jembatan sampai terbukti berasal dari batasan paket, lingkungan, atau platform.
- Pengecualian Tier D harus disetujui melalui ADR dan tidak boleh dipakai untuk menyembunyikan bug yang dapat diperbaiki.
- Regresi Tier A memblokir rilis. Regresi Tier B harus didokumentasikan. Regresi Tier C ditangani sesuai jadwal. Tier D ditinjau setiap kuartal.
- `aksara kompatibilitas periksa nama_paket` menjalankan probe lokal dan mencetak tier, versi, mode, serta batasan yang terdeteksi.
- `aksara kompatibilitas laporan` menghasilkan laporan JSON atau Markdown untuk proyek pengguna.

Probe kemampuan:

- Probe berjalan di proses terpisah agar paket yang crash tidak mematikan CLI.
- Probe memeriksa versi Python, ABI, ukuran pointer, platform, dan ketersediaan `libpython`.
- Probe mencoba impor, membuat objek, memanggil fungsi, melempar eksepsi, dan menjalankan callback kecil.
- Probe mendeteksi penggunaan `fork`, `signal`, `sys.executable`, `ctypes.pythonapi`, `sys._getframe`, dan thread.
- Hasil probe disimpan di `uji/ekosistem/hasil/` dan menjadi input dashboard.
- Probe tidak menginstal paket secara otomatis. Pemasangan tetap eksplisit melalui `aksara python pasang`.

### 5.9 Isolasi subprocess

Jalur utama jembatan adalah in-process karena memberikan overhead terendah, callback langsung, dan zero-copy. Namun sebagian paket tidak aman dijalankan in-process. Untuk paket tersebut, Aksara menyediakan mode isolasi subprocess yang tetap memakai CPython asli.

Aktivasi:

```toml
# aksara.toml
[python]
isolasi = ["scrapy", "playwright"]
batas_waktu = 60
```

```bash
aksara jalankan --isolasi-python program.aks
aksara jalankan --isolasi-python --batas-python 120 program.aks
AKSARA_PYTHON_ISOLASI=1 aksara jalankan program.aks
```

Arsitektur isolasi:

- Proses induk adalah VM Aksara.
- Proses worker adalah `aksara-python-worker` yang menanam CPython dengan versi dan virtualenv yang sama.
- Komunikasi memakai protokol biner dengan pesan berpanjang awalan, bukan teks bebas.
- Handshake memeriksa versi protokol, versi Aksara, versi Python, ABI, endianness, dan ukuran pointer.
- Setiap permintaan memiliki ID, metode, argumen, batas waktu, dan token pembatalan.
- Worker dapat menjalankan beberapa permintaan secara berurutan. Paralelisme di dalam satu worker tidak didukung pada v1.0.
- Jika worker mati, induk menerima kode keluar, sinyal, dan ekor stderr. Panggilan yang sedang berjalan melempar `KesalahanJembatan`.
- Worker dapat dimulai ulang setelah crash. Panggilan yang tidak idempoten tidak diulang secara otomatis.
- Shared memory dipakai untuk `susunan` dan buffer besar. Jika shared memory tidak tersedia, data disalin dan pengguna diberi peringatan pada mode verbose.
- Callback dari Python ke Aksara dikirim sebagai permintaan balik. Kedalaman callback dibatasi untuk mencegah rekursi tak terbatas.
- Eksepsi Python dikirim sebagai tipe, pesan, traceback, dan rantai penyebab. Di sisi Aksara, semuanya dibungkus `KesalahanPython`.
- Timeout default 60 detik per panggilan. Nilai `0` berarti tanpa batas waktu dan hanya boleh dipakai untuk pekerjaan yang memang panjang.
- Protokol memiliki nomor versi sendiri. Ketidakcocokan versi minor ditolak dengan pesan yang menyebut versi worker dan induk.

Yang tetap tidak dijamin oleh isolasi:

- Isolasi bukan sandbox. Worker berjalan dengan pengguna dan izin yang sama.
- Isolasi tidak mempercepat panggilan. Overhead lebih tinggi daripada in-process.
- Callback lintas proses lebih lambat dan tidak mendukung pointer mentah.
- Objek Python tidak dapat dibagikan sebagai referensi langsung. Data besar harus lewat buffer atau serialisasi.
- Paket yang membutuhkan akses ke objek Python yang dibuat di proses induk tetap harus berjalan in-process.

Isolasi dipakai untuk tiga tujuan: kompatibilitas paket yang membutuhkan proses sendiri, ketahanan terhadap crash library native, dan debugging konflik runtime. Isolasi bukan jalur default dan bukan pengganti perbaikan jembatan in-process.

### 5.10 pip, virtualenv, dan lingkungan Python

Aksara tidak menulis ulang resolver dependensi Python. Aksara memakai pip dan virtualenv yang sudah ada. Tujuannya sederhana: paket Python harus dipasang dengan cara yang sama seperti pada proyek Python biasa, sehingga wheel, hash, index privat, dan konfigurasi pip tetap berfungsi.

Perintah:

| Perintah | Fungsi |
|---|---|
| `aksara python info` | Menampilkan interpreter, versi, ABI, venv, `sys.path`, pip, dan status jembatan |
| `aksara python pasang requests pandas opencv-python` | Membuat venv proyek jika belum ada, lalu memasang paket |
| `aksara python pasang -r requirements.txt` | Memasang dari berkas requirement |
| `aksara python pasang --index-url https://...` | Memakai index privat |
| `aksara python daftar` | Menampilkan paket terpasang dan versinya |
| `aksara python beku` | Menulis lockfile `aksara-python.kunci` berisi versi dan hash |
| `aksara python sinkron` | Memasang ulang sesuai lockfile |
| `aksara python hapus nama_paket` | Menghapus paket dari venv |
| `aksara python bersih` | Menghapus venv proyek |
| `aksara python dokter` | Memeriksa interpreter, shared library, pip, SSL, zlib, SQLite, dan impor NumPy |
| `aksara python jalankan skrip.py` | Menjalankan skrip Python di venv proyek untuk alat bantu |

Aturan lingkungan:

- Venv proyek berada di `.aksara/lingkungan`. Di POSIX, interpreter berada di `.aksara/lingkungan/bin/python`. Di Windows, di `.aksara/lingkungan/Scripts/python.exe`.
- Urutan pencarian interpreter: venv proyek, `AKSARA_PYTHON`, `python3.12`, `python3.13`, `python3.11`, `python3`, lalu interpreter yang ditemukan di `PATH`.
- Interpreter harus CPython 64-bit versi 3.11, 3.12, atau 3.13. PyPy, GraalPy, CPython free-threaded, dan arsitektur 32-bit ditolak dengan pesan yang menyebut versi yang didukung.
- Interpreter harus menyediakan shared library `libpython` yang dapat ditautkan. Jika tidak, `aksara python dokter` menyarankan pemasangan ulang Python dengan `--enable-shared` atau memakai distribusi Python resmi.
- `impor python` tidak pernah memasang paket secara otomatis. Jika modul tidak ditemukan, error menyebut perintah `aksara python pasang` yang tepat.
- Site-packages global tidak dipakai secara default. Opsi `--sistem` dapat mengaktifkannya, tetapi tidak dianjurkan untuk reproduksibilitas.
- `pyproject.toml` dan `requirements.txt` dibaca sebagai sumber dependensi. Aksara tidak menggantikan metadata paket Python.
- Lockfile menyimpan nama paket, versi, hash wheel, URL index, versi Python, platform, dan mode jembatan.
- Mode `--offline` hanya memakai cache wheel lokal. Jika wheel tidak tersedia, pemasangan gagal dengan daftar wheel yang dicari.
- Pemasangan dari sumber membutuhkan compiler C, header Python, dan dependensi sistem. `aksara python dokter` memeriksa semuanya dan mencetak paket sistem yang hilang.
- Pemasangan paket menjalankan kode dari paket tersebut. Ini bukan operasi yang aman untuk index tidak tepercaya. Pengguna harus memakai index tepercaya dan memeriksa hash.
- Container rilis menyertakan Python 3.12, pip, dan wheel umum untuk scraping, data, dan gambar. Paket lain dipasang oleh pengguna.

Matriks wheel yang diuji:

| Platform | Arsitektur | Wheel yang diutamakan |
|---|---|---|
| Linux | x86_64, aarch64 | `manylinux_2_28`, `manylinux2014` |
| macOS | arm64, x86_64 | `macosx_12_0`, universal2 |
| Windows | x64, arm64 | `win_amd64`, `win_arm64` |

Jika sebuah paket tidak menyediakan wheel untuk platform target, paket tetap dapat dipasang dari sumber selama dependensi sistem tersedia. Jika dependensi sistem tidak tersedia, paket masuk Tier C atau D dengan alasan yang terdokumentasi.

### 5.11 CI kompatibilitas dan dashboard

Matriks uji ekosistem adalah bagian dari definisi rilis. Uji ini tidak menebak dari nama paket. Uji ini menjalankan paket nyata di lingkungan nyata.

Korpus paket:

| Irama | Jumlah minimum | Tujuan |
|---|---|---|
| Setiap commit | 50 paket inti Tier A | Menangkap regresi jembatan secara cepat |
| Nightly | 250 paket Tier A dan B | Menjaga cakupan populer |
| Mingguan | 1000 paket lintas kategori | Menemukan kasus tepi |
| Setiap rilis | 2000 paket | Gerbang rilis |

Korpus disusun dari peringkat unduhan PyPI, kategori (web, data, gambar, sains, basis data, GUI, otomasi, keamanan), laporan pengguna, dan paket yang dipakai oleh contoh resmi. Korpus adalah sampel yang terus tumbuh, bukan daftar putih. Paket di luar korpus tetap harus berjalan; laporan pengguna menjadi jalur masuk.

Setiap paket memiliki manifes di `uji/ekosistem/paket.toml`:

```toml
[nama]
paket = "requests"
impor = "requests"
tier_target = "A"
kategori = "web"
batas_waktu = 60

[uji]
dasar = "requests_dasar.aks"
callback = "requests_callback.aks"
eksepsi = "requests_eksepsi.aks"
```

Tingkat uji per paket:

1. Impor dan versi.
2. Pembuatan objek dan pemanggilan fungsi dasar.
3. Pertukaran tipe primitif dan koleksi.
4. Callback Aksara dari library Python.
5. Pemetaan eksepsi dua arah.
6. Iterator, context manager, dan pembersihan sumber daya.
7. Thread, async, atau proses jika paket memakainya.
8. Buffer protocol dan zero-copy jika paket memakai NumPy.
9. Mode isolasi jika paket tidak aman in-process.
10. Pemasangan melalui pip dan venv jika paket membutuhkan dependensi.

Matriks CI:

| Dimensi | Nilai |
|---|---|
| Sistem operasi | Ubuntu 24.04, Ubuntu 26.04, macOS 14, macOS 15, Windows Server 2022, Windows Server 2025 |
| Python | 3.11, 3.12, 3.13 |
| Mode jembatan | in-process, isolasi, tanpa Python |
| Mode build | debug, release |
| Arsitektur | x86_64, aarch64 |

Dashboard kompatibilitas menampilkan:

- Nama dan versi paket.
- Tier saat ini dan tier target.
- Sistem operasi, versi Python, dan mode jembatan.
- Status: lulus, gagal, flaky, dilewati, tidak didukung.
- Tahap kegagalan: impor, konstruksi, panggilan, callback, eksepsi, buffer, thread, pembersihan.
- Kode error dan 20 baris terakhir log.
- Tanggal pertama gagal, tanggal terakhir lulus, dan issue terkait.
- Nama paket yang mengalami regresi sejak rilis sebelumnya.

Triase otomatis:

- Setiap kegagalan diberi fingerprint dari paket, versi, OS, Python, tahap, kode error, dan frame teratas.
- Fingerprint yang sama tidak membuat issue baru. Issue lama diperbarui dengan hitungan dan tautan build.
- Kegagalan diklasifikasikan menjadi bug jembatan, batasan paket, masalah lingkungan, timeout, atau dependensi sistem yang hilang.
- Issue otomatis diberi label `ekosistem/tier-a`, `ekosistem/tier-b`, atau `ekosistem/tier-c`.
- Kegagalan yang hilang pada percobaan ulang ditandai flaky. Setelah tiga kali flaky, paket dikarantina dengan issue dan tanggal tinjauan.
- Regresi Tier A memblokir rilis. Regresi Tier B wajib didokumentasikan di catatan rilis. Regresi Tier C ditangani pada siklus mingguan.
- Dashboard publik berada di `kompatibilitas.aksara.dev` setelah domain diperiksa di Fase 0. Data mentah tersedia sebagai JSON dan CSV.

Reproduksibilitas:

- Versi paket dipin di lockfile CI.
- Container uji memakai digest image, bukan tag yang bergerak.
- Wheel disimpan dengan hash SHA-256.
- Hasil uji menyimpan versi Aksara, versi Python, commit, dan checksum manifest.
- Kegagalan dapat direproduksi dengan `aksara kompatibilitas jalankan --manifest ... --os ... --python ...`.

### 5.12 Batasan yang tidak bisa dihindari

Tidak ada jembatan yang dapat membuat semua paket berjalan tanpa batasan. Batasan berikut diakui, diuji, dan didokumentasikan. Paket yang terkena batasan tetap mendapat jalur: perbaikan jembatan, mode isolasi, atau pengecualian Tier D dengan alasan.

| Batasan | Dampak | Mitigasi |
|---|---|---|
| CPython free-threaded 3.13t | Tidak didukung | Tolak dengan pesan versi, gunakan CPython biasa |
| Subinterpreter | Tidak didukung | Jalankan beberapa proses, bukan subinterpreter |
| Arsitektur 32-bit | Tidak didukung | Gunakan 64-bit |
| Paket yang menanam CPython sendiri | Dua runtime dapat bertabrakan | Mode isolasi atau jalankan sebagai proses Python terpisah |
| Paket yang memeriksa internal CPython | `sys._getframe`, `ctypes.pythonapi`, dan struktur internal dapat berubah | Uji per versi, dokumentasikan, isolasi |
| `fork()` setelah thread | Deadlock | Mode isolasi, start method `spawn` |
| Signal handler | Hanya aman di thread utama | Jalankan di thread utama, dokumentasikan |
| GUI dan event loop | Butuh thread utama dan display | Dukungan thread utama, backend headless untuk CI |
| `sys.executable` | Menunjuk interpreter Python, bukan `aksara` | Venv proyek, shim, mode isolasi |
| Async Python dan async Aksara | Dua event loop dapat bertabrakan | Satu loop Python per thread, larang loop bersarang |
| GIL | Panggilan Python panjang memblokir thread Aksara | Mode isolasi untuk pekerjaan CPU-bound |
| Finalisasi CPython | `Py_Finalize` dapat dilewati jika objek masih hidup | Pembersihan saat keluar proses, dokumentasi |
| Siklus lintas runtime | Kebocoran memori | Weakref, `tutup()`, batasi di v1.0 |
| Identitas proxy | `adalah` bukan identitas Python asli | Gunakan `id()` Python melalui atribut khusus jika perlu |
| Koleksi by value | Mutasi tidak terlihat di sisi lain | Gunakan `objek_python` atau `susunan` untuk referensi |
| Pickle lintas runtime | Objek Aksara tidak dapat di-pickle oleh Python | Pisahkan serialisasi Aksara dan Python |
| Callback `ctypes` | Umur fungsi harus lebih panjang dari callback | Pin fungsi, dokumentasikan, uji dengan sanitizer |
| Audit hook dan sandbox | Jembatan bukan sandbox | Jalankan kode tidak tepercaya di container |
| Konflik library native | Dua runtime OpenMP, MKL, atau CUDA dapat crash | Isolasi kelompok paket, satu proses per kelompok |
| Layanan eksternal | Database, browser, dan GPU harus tersedia | Dokumentasikan dependensi layanan |
| Output terminal | Urutan stdout Python dan Aksara dapat berbeda | Flush eksplisit, line buffering |
| Paket yang butuh root | Tidak dapat dijalankan sebagai pengguna biasa | Di luar cakupan |

Penjelasan batasan yang paling sering ditemui:

**Free-threaded CPython.** CPython 3.13 menyediakan build eksperimental tanpa GIL. Model memori Aksara dan jembatan belum siap untuk itu. Build tersebut ditolak saat handshake, bukan dicoba lalu crash.

**Subinterpreter.** CPython memiliki subinterpreter, tetapi ekstensi native dan objek global banyak yang tidak aman lintas subinterpreter. Aksara memakai satu interpreter per proses. Paralelisme dicapai dengan proses terpisah.

**Paket yang menanam CPython.** Beberapa paket membawa runtime Python sendiri atau memanggil `Py_Initialize` dari library native. Dua runtime dalam satu proses dapat merusak state global. Paket seperti ini harus berjalan di mode isolasi atau sebagai proses Python terpisah.

**Paket yang memeriksa internal CPython.** Paket yang memakai `ctypes.pythonapi`, `sys._getframe`, `PyInterpreterState`, atau layout objek internal bekerja selama versi CPython dan ABI cocok. Mereka diuji per versi minor. Jika gagal, paket masuk Tier B atau C, bukan dipaksa berjalan.

**Fork dan thread.** `fork()` pada proses yang memiliki thread dapat deadlock. Modul `multiprocessing` dengan start method `spawn` didukung. Start method `fork` hanya didukung di mode isolasi. `forkserver` diuji pada Linux dan macOS.

**GUI.** Tkinter, Qt, wxPython, dan library GUI lain membutuhkan thread utama dan event loop. Aksara menjalankan inisialisasi CPython di thread utama. GUI harus dibuat dan dijalankan di thread tersebut. CI memakai backend headless atau Xvfb.

**`sys.executable`.** Banyak paket menjalankan `sys.executable` untuk memulai proses Python baru. Di dalam jembatan, `sys.executable` menunjuk interpreter Python di venv proyek, bukan binary `aksara`. Ini membuat `multiprocessing` dan alat Python tetap bekerja. Paket yang mengharapkan `sys.executable` adalah aplikasi induk harus memakai mode isolasi atau konfigurasi eksplisit.

**Async.** Aksara memiliki event loop sendiri. CPython juga memiliki `asyncio`. Aturannya: satu event loop Python aktif per thread. `tunggu` dapat menunggu awaitable Python, tetapi `asyncio.run` di dalam loop yang sudah berjalan melempar kesalahan. Loop bersarang ditolak dengan `KesalahanJembatan`, bukan dijalankan diam-diam.

**GIL.** Panggilan ke library Python memegang GIL CPython. Selama panggilan panjang, thread Aksara lain tidak dapat masuk ke Python. Untuk pekerjaan CPU-bound, gunakan mode isolasi, proses terpisah, atau library native yang melepaskan GIL.

**Finalisasi.** Jika objek Python masih dipegang saat VM Aksara berhenti, `Py_Finalize` dapat dilewati. Proses tetap membersihkan sumber daya saat keluar. Finalizer Python tidak dijamin berjalan pada waktu tertentu. Gunakan `dengan` atau `tutup()` untuk berkas, soket, dan koneksi.

**Identitas dan koleksi.** `adalah` pada `objek_python` membandingkan identitas proxy, bukan identitas objek Python. Koleksi bawaan dikonversi by value. Jika Anda perlu berbagi referensi, bungkus objek sebagai `objek_python` atau gunakan `susunan` untuk buffer.

**Pickle.** `simpan` adalah serialisasi Aksara. `pickle` adalah serialisasi Python. Keduanya tidak saling menggantikan. Objek Aksara tidak dapat di-pickle oleh Python kecuali dibungkus sebagai objek Python yang mendukung pickle.

**Konflik native.** Dua library yang membawa OpenMP, MKL, atau CUDA dengan versi berbeda dapat crash jika dimuat dalam satu proses. Ini bukan bug jembatan. Solusinya adalah memisahkan kelompok paket ke proses terpisah atau memilih build yang kompatibel.

Klaim yang tidak dibuat:

- Aksara tidak menjamin paket yang bergantung pada detail internal CPython yang tidak stabil.
- Aksara tidak menjamin paket yang hanya berjalan pada platform atau arsitektur yang tidak didukung.
- Aksara tidak menjamin paket yang membutuhkan layanan eksternal tanpa menyediakan layanan tersebut.
- Aksara tidak menjamin paket yang melanggar lisensi atau hukum setempat.
- Aksara bukan sandbox dan tidak menjanjikan keamanan untuk kode tidak tepercaya.

## 6. Standard library

Modul ditulis dalam Aksara kecuali membutuhkan akses sistem atau performa. Modul native memakai ABI C.

| Modul Aksara | Padanan Python | Fase | Dependensi native |
|---|---|---|---|
| `matematika` | `math` | 5 | tidak |
| `statistika` | `statistics` | 5 | tidak |
| `rasional` | `fractions` | 5 | tidak |
| `desimal` | `decimal` | 5 | tidak |
| `acak` | `random` | 5 | tidak |
| `alat_teks` | `string` | 5 | tidak |
| `pola` | `re` | 5 | PCRE2 |
| `json` | `json` | 5 | tidak |
| `csv` | `csv` | 5 | tidak |
| `tanggal` | `datetime` | 5 | tidak |
| `waktu` | `time` | 5 | tidak |
| `zona_waktu` | `zoneinfo` | 5 | tzdata |
| `alat_berkas` | `io` | 5 | tidak |
| `jalur` | `pathlib` | 5 | tidak |
| `direktori` | `os`, `os.path` | 5 | tidak |
| `sistem` | `sys`, `platform` | 5 | tidak |
| `proses` | `subprocess` | 5 | tidak |
| `lingkungan` | `os.environ` | 5 | tidak |
| `koleksi` | `collections` | 5 | tidak |
| `alat_daftar` | pembantu daftar | 5 | tidak |
| `alat_kamus` | pembantu kamus | 5 | tidak |
| `alat_iterasi` | `itertools` | 5 | tidak |
| `alat_fungsi` | `functools` | 5 | tidak |
| `salin` | `copy` | 5 | tidak |
| `simpan` | `pickle` | 5 | tidak |
| `basis64` | `base64` | 5 | tidak |
| `sidik` | `hashlib` | 5 | OpenSSL atau platform |
| `kompresi` | `gzip`, `zipfile`, `tarfile` | 5 | zlib |
| `codec` | `codecs` | 5 | tidak |
| `jaringan` | `socket` | 5 | tidak |
| `http` | `http.client` | 5 | tidak |
| `tls` | `ssl` | 5 | OpenSSL |
| `url` | `urllib.parse` | 5 | tidak |
| `utas` | `threading` | 5 | tidak |
| `antrean` | `queue` | 5 | tidak |
| `kunci` | `threading.Lock` | 5 | tidak |
| `alur_asinkron` | `asyncio` | 5 | tidak |
| `sqlite` | `sqlite3` | 5 | SQLite |
| `pencatatan` | `logging` | 5 | tidak |
| `argumen` | `argparse` | 5 | tidak |
| `pengujian` | `unittest` | 5 | tidak |
| `pengaturan` | `configparser` | 5 | tidak |
| `sistem_tipe` | `typing` | 5 | tidak |
| `abstrak` | `abc` | 5 | tidak |
| `kelas_data` | `dataclasses` | 5 | tidak |
| `tipe_pilihan` | `enum` | 5 | tidak |
| `matriks` | pembungkus NumPy | 6 | jembatan Python |
| `tabel` | pembungkus pandas | 6 | jembatan Python |
| `gambar` | pembungkus OpenCV | 6 | jembatan Python |

Modul `matriks`, `tabel`, dan `gambar` adalah pembungkus opsional. Pengguna tetap dapat memanggil library Python asli secara langsung.

## 7. Peta fase

### 7.1 Ringkasan

| Fase | Nama | Durasi | Effort | Rilis |
|---|---|---|---|---|
| 0 | Landasan | 3 minggu | 1 orang-bulan | |
| 1 | Lekser dan parser | 6 minggu | 3 orang-bulan | |
| 2 | Interpreter referensi | 10 minggu | 5 orang-bulan | v0.1 |
| 3 | Runtime C dan VM bytecode | 14 minggu | 8 orang-bulan | v0.2 |
| 4 | Kelengkapan bahasa | 12 minggu | 7 orang-bulan | v0.3 |
| 5 | Standard library | 14 minggu | 8 orang-bulan | v0.4 |
| 6 | Jembatan Python dan ekosistem penuh | 18 minggu | 11 orang-bulan | v0.5 |
| 7 | Alat pengembang | 10 minggu | 6 orang-bulan | v0.6 |
| 8 | Performa | 12 minggu | 8 orang-bulan | v0.7 |
| 9 | Stabilisasi dan rilis | 8 minggu | 5 orang-bulan | v0.8 beta, v0.9 RC, v1.0 |
| Total | | 107 minggu | 62 orang-bulan | |

Asumsi:

- Tim inti 3 sampai 4 orang penuh waktu: 1 compiler engineer, 1 runtime C engineer, 1 tooling engineer, 1 QA dan dokumentasi.
- Dengan 3 FTE, total sekitar 21 bulan. Dengan solo developer, sekitar 4 sampai 5 tahun.
- Fase 5 dan 7 dapat berjalan paralel dengan Fase 6 jika tim mencukupi.
- Estimasi adalah perkiraan, bukan komitmen. Setiap fase memiliki gerbang yang harus lulus sebelum fase berikutnya dimulai.

### 7.2 Fase 0: Landasan

**Durasi:** 3 minggu. **Effort:** 1 orang-bulan. **Prasyarat:** tidak ada.

**Tujuan:** Menetapkan ruang lingkup, spesifikasi awal, arsitektur, dan kerangka kerja.

| Sub-fase | Pekerjaan | Keluaran |
|---|---|---|
| 0.1 | Visi, persona, 10 program contoh, non-goals | `docs/visi.md` |
| 0.2 | Spesifikasi leksikal: kata kunci, literal, indentasi, komentar | `docs/spesifikasi/leksikal.md` |
| 0.3 | Spesifikasi sintaks: EBNF, skema AST, contoh per konstruksi | `docs/spesifikasi/tata_bahasa.ebnf` |
| 0.4 | Spesifikasi tipe, truthiness, operator, protokol | `docs/spesifikasi/tipe.md` |
| 0.5 | ADR: pembagian C/Rust, bytecode, GC, jembatan Python, format paket | `docs/adr/0001-0005` |
| 0.6 | Kerangka repo: Cargo workspace, CMake, justfile, CI | repositori |
| 0.7 | Harness uji: golden test, benchmark CPython, target fuzz | `uji/` |
| 0.8 | Panduan kontribusi, lisensi, kode etik | `CONTRIBUTING.md` |

**Kriteria lulus:**

- Semua ADR disetujui dan dicatat.
- `cargo test` dan `ctest` hijau di Linux, macOS, dan Windows.
- 10 contoh program tersimpan dan menjadi target parser.
- Baseline benchmark CPython 3.12 tercatat.
- Struktur repo dan CI berjalan.

**Risiko:** desain berlebihan. **Mitigasi:** timebox 3 minggu, spesifikasi boleh berevolusi melalui ADR.

### 7.3 Fase 1: Lekser dan parser

**Durasi:** 6 minggu. **Effort:** 3 orang-bulan. **Prasyarat:** Fase 0.

**Tujuan:** Mengubah `.aks` menjadi AST dengan diagnostik yang dapat ditindaklanjuti.

| Sub-fase | Pekerjaan | Keluaran |
|---|---|---|
| 1.1 | Lekser: token, INDENT/DEDENT, string, f-string, angka, komentar, Unicode | `crates/aksara-lexer` |
| 1.2 | Parser ekspresi: Pratt, prioritas, literal, comprehension, lambda, slice, call, walrus, `antara`, `??`, `?.`, ternary | `crates/aksara-parser` |
| 1.3 | Parser pernyataan: penugasan, kendali, bentuk perulangan bahasa alami, fungsi, kelas, eksepsi, with, import, dekorator, async | `crates/aksara-parser` |
| 1.4 | AST: span, ID, visitor, serialisasi untuk pengujian | `crates/aksara-ast` |
| 1.5 | Diagnostik: pemulihan error, beberapa error, pesan Indonesia, saran, frame sumber | `crates/aksara-diagnostik` |
| 1.6 | CLI `aksara periksa`: parse-only, output JSON | `crates/aksara-cli` |
| 1.7 | Uji: golden, 1000+ kasus, fuzz, properti | `uji/parser` |
| 1.8 | Dokumentasi tata bahasa | `docs/spesifikasi/` |

**Kriteria lulus:**

- 100% contoh di `contoh/` terparse.
- Fuzz 24 jam tanpa panic.
- Berkas 1 MB terparse kurang dari 100 ms di mesin referensi.
- 1000+ kasus uji lulus.
- Pesan error menyebut baris, kolom, dan saran perbaikan.

**Risiko:** indentasi, f-string, dan pemulihan error. **Mitigasi:** korpus uji khusus, fuzzing, perbandingan dengan Python untuk kasus yang setara.

### 7.4 Fase 2: Interpreter referensi

**Durasi:** 10 minggu. **Effort:** 5 orang-bulan. **Prasyarat:** Fase 1.

**Tujuan:** Semantik inti berjalan di Rust dan menjadi oracle untuk VM C.

| Sub-fase | Pekerjaan | Keluaran |
|---|---|---|
| 2.1 | Nilai, lingkungan, scope, truthiness, equality, hash | `crates/aksara-interpreter` |
| 2.2 | Ekspresi: aritmetika, perbandingan, logika, operator kosong dan aman, ternary, bitwise, call, atribut, indeks, slice, comprehension | `crates/aksara-interpreter` |
| 2.3 | Kendali: jika, selama, untuk, bentuk setiap/sebanyak/dari sampai, hentikan, lanjutkan, kembalikan, lewati | `crates/aksara-interpreter` |
| 2.4 | Fungsi: parameter, default, varargs, kwargs, closure, rekursi, dekorator | `crates/aksara-interpreter` |
| 2.5 | Koleksi: daftar, tupel, kamus, himpunan, rentang, protokol iterasi | `crates/aksara-interpreter` |
| 2.6 | Kelas: pewarisan, MRO, diri, induk, properti, metode kelas dan statis, dunder | `crates/aksara-interpreter` |
| 2.7 | Eksepsi: hierarki, coba/tangkap/lainnya/akhirnya, lempar, traceback | `crates/aksara-interpreter` |
| 2.8 | Generator, context manager, hasilkan, dengan | `crates/aksara-interpreter` |
| 2.9 | Modul dan impor: modul native, paket, impor relatif | `crates/aksara-interpreter` |
| 2.10 | Builtin inti dan REPL | `crates/aksara-repl` |
| 2.11 | Suite kepatuhan inti | `uji/kepatuhan` |

**Kriteria lulus:**

- 500+ uji kepatuhan lulus.
- Semua contoh berjalan.
- REPL multiline berfungsi.
- Tidak ada panic pada input valid.
- Harness uji diferensial siap dipakai.

**Risiko:** borrow checker pada graf objek. **Mitigasi:** arena atau `Rc<RefCell>`, kebocoran siklus diterima di oracle.

### 7.5 Fase 3: Runtime C dan VM bytecode

**Durasi:** 14 minggu. **Effort:** 8 orang-bulan. **Prasyarat:** Fase 2.

**Tujuan:** Mesin produksi di C yang kompatibel dengan oracle.

| Sub-fase | Pekerjaan | Keluaran |
|---|---|---|
| 3.1 | Model objek C: Nilai, Objek, Tipe, registry | `runtime/src/objek.c` |
| 3.2 | Alokator: pool, alignment, OOM, statistik | `runtime/src/alokasi.c` |
| 3.3 | GC mark-sweep: precise roots, finalizer, weakref, mode stres | `runtime/src/gc.c` |
| 3.4 | Teks dan Unicode: UTF-8, intern, hash, slice per code point | `runtime/src/teks.c` |
| 3.5 | Koleksi C: daftar, tupel, kamus, himpunan, rentang | `runtime/src/koleksi.c` |
| 3.6 | Format `.akc`: header, versi, konstanta, kode, line table, exception table | `docs/spesifikasi/bytecode.md` |
| 3.7 | Verifier: stack, lompatan, lokal, exception, konstanta | `runtime/src/verifier.c` |
| 3.8 | VM loop: dispatch, frame, call stack, error, traceback | `runtime/src/vm.c` |
| 3.9 | Kompiler bytecode Rust: scope, closure, exception table, peephole | `crates/aksara-kompiler` |
| 3.10 | Builtin C dan antarmuka modul native | `runtime/src/builtin.c` |
| 3.11 | Uji diferensial Rust vs C | `uji/diferensial` |
| 3.12 | Profiler dan debug hook | `runtime/src/profil.c` |

**Kriteria lulus:**

- Semua uji Fase 2 lulus di VM C.
- ASan, UBSan, dan MSan bersih.
- Fuzz 24 jam tanpa crash.
- Verifier menolak bytecode rusak.
- VM C 5x lebih cepat dari oracle pada `fib(30)`.
- Startup kurang dari 100 ms.

**Risiko:** bug GC dan VM. **Mitigasi:** GC non-moving, sanitizer, fuzzing, uji diferensial, dan review wajib untuk `gc.c` serta `vm.c`.

### 7.6 Fase 4: Kelengkapan bahasa

**Durasi:** 12 minggu. **Effort:** 7 orang-bulan. **Prasyarat:** Fase 3.

**Tujuan:** Mencapai 95% fitur bahasa Python 3.12 yang relevan.

| Sub-fase | Pekerjaan | Keluaran |
|---|---|---|
| 4.1 | Numerik: bigint, overflow, float, kompleks, pembagian, modulo, pangkat, NaN/inf, repr float shortest round-trip (Ryu) | `runtime/src/numerik.c` |
| 4.2 | Teks: metode lengkap, format mini-language, f-string, raw/bytes, encode/decode | `runtime/src/teks.c` |
| 4.3 | Koleksi: slice, sort stabil, comprehension bersarang, unpacking, `*`/`**` | `runtime/src/koleksi.c` |
| 4.4 | OOP: C3 MRO, descriptor, `__slots__`, metaclass, ABC, `super()` berargumen | `runtime/src/kelas.c` |
| 4.5 | Fungsi: positional-only, keyword-only, anotasi, closure, `nonlokal`, batas rekursi | `runtime/src/fungsi.c` |
| 4.6 | Generator: `hasilkan dari`, `send`, `throw`, `close`, ekspresi generator | `runtime/src/generator.c` |
| 4.7 | Eksepsi: chaining, `__cause__`, `__context__`, traceback, `except*` (opsional) | `runtime/src/eksepsi.c` |
| 4.8 | Modul: paket, impor relatif, `__semua__`, `__nama__`, `__utama__`, reload | `runtime/src/modul.c` |
| 4.9 | Pattern matching: `cocokkan`/`kasus` | `runtime/src/cocok.c` |
| 4.10 | Asinkron: `asinkron fungsi`, `tunggu`, event loop dasar, `asinkron untuk`, `asinkron dengan` | `pustaka/alur_asinkron.aks` |
| 4.11 | Anotasi tipe dan modul `sistem_tipe` | `pustaka/sistem_tipe.aks` |
| 4.12 | Pembaruan matriks paritas | `docs/kompatibilitas/` |

**Kriteria lulus:**

- 95% matriks paritas terpenuhi.
- 1500+ uji kepatuhan lulus.
- Tidak ada deviasi semantik yang tidak didokumentasikan.
- Semua contoh berjalan di VM C dan oracle.

**Risiko:** cakupan OOP lanjutan dan async. **Mitigasi:** matriks paritas, penundaan fitur ke v1.1 jika diperlukan, uji diferensial.

### 7.7 Fase 5: Standard library

**Durasi:** 14 minggu. **Effort:** 8 orang-bulan. **Prasyarat:** Fase 4.

**Tujuan:** Menyediakan 80% modul inti yang dibutuhkan aplikasi nyata.

| Sub-fase | Pekerjaan | Keluaran |
|---|---|---|
| 5.1 | Numerik dan statistik: matematika, statistika, rasional, desimal, acak | `pustaka/` |
| 5.2 | Teks dan data: alat_teks, pola, json, csv, tanggal, waktu, zona_waktu | `pustaka/` |
| 5.3 | Berkas dan sistem: alat_berkas, jalur, direktori, sistem, proses, lingkungan | `pustaka/` |
| 5.4 | Koleksi dan iterasi: koleksi, alat_iterasi, alat_fungsi, salin, simpan | `pustaka/` |
| 5.5 | Encoding dan kompresi: basis64, sidik, kompresi, codec | `pustaka/` |
| 5.6 | Jaringan dasar: jaringan, http, url | `pustaka/` |
| 5.7 | Konkurensi: utas, antrean, kunci, alur_asinkron | `pustaka/` |
| 5.8 | Basis data: sqlite | `pustaka/` |
| 5.9 | Alat: pencatatan, argumen, pengujian, pengaturan | `pustaka/` |
| 5.10 | Dokumentasi dan type stub setiap modul | `docs/pustaka/` |

**Kriteria lulus:**

- 80% daftar modul Fase 5 selesai.
- Setiap modul memiliki dokumentasi dan uji.
- Studi kasus pengolahan teks, JSON/CSV, dan berkas berjalan tanpa jembatan Python.
- Tidak ada dependensi native yang bocor ke antarmuka publik.

**Risiko:** cakupan stdlib terlalu besar. **Mitigasi:** prioritaskan modul untuk scraping, data, dan gambar. Modul niche ditunda ke v1.1.

### 7.8 Fase 6: Jembatan Python dan ekosistem penuh

**Durasi:** 18 minggu. **Effort:** 11 orang-bulan. **Prasyarat:** Fase 4 dan 5.

**Tujuan:** Library Python asli dapat dipakai dari Aksara, tanpa daftar putih, dengan jalur in-process dan isolasi.

| Sub-fase | Pekerjaan | Keluaran |
|---|---|---|
| 6.1 | Embedding CPython: deteksi build, init/finalize, konfigurasi, sys.path, cek versi | `runtime/jembatan/python.c` |
| 6.2 | Marshaling tipe dasar: int, float, bool, None, str, bytes, list, tuple, dict, set | `runtime/jembatan/marshal.c` |
| 6.3 | Proxy objek: atribut, panggilan, indeks, iterasi, context manager, repr, str | `runtime/jembatan/proxy.c` |
| 6.4 | Numerik dan NumPy: buffer protocol, zero-copy ndarray, tipe `susunan`, dtype, slice | `runtime/jembatan/numpy.c` |
| 6.5 | Callback: fungsi Aksara menjadi callable Python, lifetime, GIL | `runtime/jembatan/callback.c` |
| 6.6 | Eksepsi dua arah: pemetaan, traceback, `KesalahanPython` | `runtime/jembatan/eksepsi.c` |
| 6.7 | Memori dan siklus: finalizer, root, cross-runtime cycle, weakref | `runtime/jembatan/memori.c` |
| 6.8 | Sintaks `impor python` dan `dari python`, cache modul, alias | `crates/aksara-interpreter` |
| 6.9 | Integrasi nyata (antara lain): requests, bs4, lxml, pandas, numpy, cv2, matplotlib, sqlalchemy, scrapy, playwright | `uji/jembatan/` |
| 6.10 | Packaging: dynamic linking, Windows/macOS/Linux, `--info-python`, fallback subprocess | `docs/distribusi/` |
| 6.11 | Performa: overhead panggilan, batch API, strategi GIL | `benchmark/jembatan/` |
| 6.12 | Kontrak ekosistem dan probe kemampuan otomatis | `runtime/jembatan/probe.c`, `crates/aksara-kompatibilitas` |
| 6.13 | Isolasi subprocess: protokol biner, worker, shared memory, timeout, restart | `runtime/jembatan/isolasi.c`, `runtime/jembatan/protokol.c`, `worker/` |
| 6.14 | pip, virtualenv, requirements, pyproject, lockfile, offline cache | `crates/aksara-python`, `runtime/jembatan/pip.c` |
| 6.15 | Harness ekosistem dan manifes paket Tier A/B/C | `uji/ekosistem/` |
| 6.16 | CI nightly, sharding, dashboard, triase otomatis, bisect regresi | `alat/kompatibilitas/`, `docs/kompatibilitas/` |
| 6.17 | Deteksi konflik runtime: fork, signal, `sys.executable`, `ctypes.pythonapi`, event loop | `runtime/jembatan/deteksi.c` |
| 6.18 | Dokumentasi tier, batasan, panduan laporan pengguna, dan kebijakan Tier D | `docs/kompatibilitas/` |
| 6.19 | Gerbang rilis ekosistem: 2000 paket, laporan tier, tanda tangan hasil | `uji/ekosistem/laporan/` |

**Kriteria lulus:**

- 200+ paket Python lulus uji Tier A atau B di Linux, macOS, dan Windows.
- 1000+ paket masuk korpus mingguan dan 2000 paket masuk gerbang rilis.
- Tier A lulus seluruh kemampuan: impor, panggilan, callback, eksepsi, iterator, context manager, buffer, thread, dan pembersihan.
- NumPy dan OpenCV memakai zero-copy untuk data gambar tanpa salinan penuh.
- pandas dapat membaca CSV, melakukan agregasi, dan menulis hasil.
- requests, bs4, dan lxml dapat mengambil serta mem-parsing halaman nyata.
- Mode isolasi menangani paket yang memakai fork, signal, atau `sys.executable`, termasuk crash worker, timeout, dan callback.
- pip, virtualenv, requirements, pyproject, dan lockfile berfungsi di tiga sistem operasi.
- `aksara python dokter` mendeteksi interpreter, ABI, shared library, dan dependensi sistem yang hilang.
- Tidak ada daftar putih paket. Semua pengecualian Tier D memiliki ADR dan alasan teknis.
- Dashboard kompatibilitas menampilkan tier, status, dan regresi.
- Overhead panggilan sederhana kurang dari 1 mikrodetik pada mode in-process.
- Build `--tanpa-python` tetap berfungsi.
- Berjalan di Linux, macOS, dan Windows dengan Python 3.11, 3.12, dan 3.13.

**Risiko:** ABI, GIL, packaging, dan skala matriks uji. **Mitigasi:** tier otomatis, isolasi subprocess, CI sharding, dashboard, ADR untuk pengecualian, dan penundaan finalisasi CPython jika tidak aman.

### 7.9 Fase 7: Alat pengembang

**Durasi:** 10 minggu. **Effort:** 6 orang-bulan. **Prasyarat:** Fase 4.

**Tujuan:** Pengalaman pengembangan setara dengan Python modern.

| Sub-fase | Pekerjaan | Keluaran |
|---|---|---|
| 7.1 | Formatter: berbasis AST, idempoten, komentar, lebar baris, konfigurasi | `crates/aksara-format` |
| 7.2 | Linter: 30+ aturan, saran perbaikan, konfigurasi | `crates/aksara-lint` |
| 7.3 | LSP: diagnostik, completion, hover, signature, definition, references, rename, format | `crates/aksara-lsp` |
| 7.4 | Debugger: DAP, breakpoint, step, variabel, call stack, evaluate | `crates/aksara-debug` |
| 7.5 | Package manager: manifest, lockfile, install, registry client, cache, workspace | `crates/aksara-paket` |
| 7.6 | Generator dokumentasi: docstring, HTML/Markdown, pencarian | `crates/aksara-dok` |
| 7.7 | Integrasi editor: ekstensi VS Code, grammar Tree-sitter, konfigurasi Neovim | `editor/` |
| 7.8 | Playground WASM eksperimental | `playground/` |
| 7.9 | Kurikulum pemula 10 pelajaran, tutorial interaktif, latihan tervalidasi | `contoh/pemula/`, `docs/panduan/` |

**Kriteria lulus:**

- Ekstensi VS Code menampilkan diagnostik dan completion.
- Formatter idempoten pada 10.000 berkas uji.
- LSP merespons kurang dari 100 ms untuk berkas 5.000 baris.
- Debugger dapat melangkah melalui contoh OOP dan eksepsi.
- Package manager dapat memasang paket lokal dan dari registry uji.
- `aksara tutorial` menyelesaikan 10 pelajaran dan seluruh latihan lulus CI.

**Risiko:** cakupan LSP dan registry. **Mitigasi:** MVP lebih dulu, registry lokal, rilis publik pasca 1.0.

### 7.10 Fase 8: Performa

**Durasi:** 12 minggu. **Effort:** 8 orang-bulan. **Prasyarat:** Fase 6 dan 7.

**Tujuan:** Mencapai 3x CPython pada benchmark inti tanpa mengorbankan stabilitas.

| Sub-fase | Pekerjaan | Keluaran |
|---|---|---|
| 8.1 | Profiler dan harness benchmark | `benchmark/` |
| 8.2 | Direct-threaded dispatch dengan computed goto | `runtime/src/vm.c` |
| 8.3 | Inline caching untuk atribut, metode, dan global | `runtime/src/vm.c` |
| 8.4 | Opcode khusus untuk int, float, teks, daftar, kamus | `runtime/src/vm.c` |
| 8.5 | Local variable slot dan frame cepat | `runtime/src/frame.c` |
| 8.6 | Tuning GC: generational dan incremental | `runtime/src/gc.c` |
| 8.7 | Benchmark 10 program inti dan analisis regresi | `benchmark/hasil/` |

JIT tidak menjadi target v1.0. JIT direncanakan untuk v1.2 sebagai komponen Rust terpisah yang terhubung melalui ABI C.

**Kriteria lulus:**

- 3x CPython 3.12 pada 10 benchmark inti.
- Memori kurang dari 1.5x CPython pada program yang sama.
- Startup kurang dari 50 ms.
- Tidak ada regresi pada uji kepatuhan.
- Tidak ada sanitizer error.

**Risiko:** optimisasi dini. **Mitigasi:** ukur dulu, optimalkan tiga bottleneck teratas, sediakan fallback interpreter.

### 7.11 Fase 9: Stabilisasi dan rilis 1.0

**Durasi:** 8 minggu. **Effort:** 5 orang-bulan. **Prasyarat:** semua fase.

**Tujuan:** Merilis v1.0 yang stabil dan terdokumentasi.

| Sub-fase | Pekerjaan | Keluaran |
|---|---|---|
| 9.1 | Pembekuan spesifikasi v1.0 | `docs/spesifikasi/` |
| 9.2 | Kebijakan kompatibilitas dan deprecation | `docs/kebijakan/` |
| 9.3 | Audit keamanan, fuzzing, sanitizer | laporan |
| 9.4 | Dokumentasi: referensi, tutorial, cookbook, panduan migrasi | `docs/` |
| 9.5 | Paket instalasi: tarball, installer, container, checksum, signature | `dist/` |
| 9.6 | 10 aplikasi contoh: scraper, analisis data, pengolahan gambar, CLI, API | `contoh/` |
| 9.7 | Uji regresi performa | `benchmark/` |
| 9.8 | Release candidate, bug bash, rilis v1.0 | tag `v1.0.0` |

**Kriteria lulus:**

- Semua gerbang rilis terpenuhi.
- Tidak ada bug P0 atau P1.
- 10 aplikasi contoh berjalan di tiga sistem operasi.
- Dokumentasi lengkap dalam bahasa Indonesia.
- Audit keamanan selesai.
- Paket instalasi tersedia dan terverifikasi.

**Risiko:** bug menjelang rilis. **Mitigasi:** pembekuan fitur, periode RC, larangan fitur baru.

## 8. Strategi pengujian

### 8.1 Piramida pengujian

| Lapisan | Alat | Cakupan |
|---|---|---|
| Unit Rust | `cargo test` | Lekser, parser, AST, kompiler, tooling |
| Unit C | `ctest` dan cmocka | Objek, koleksi, GC, VM, verifier |
| Golden | Runner khusus | AST, bytecode, diagnostik |
| Kepatuhan | `.aks` dan ekspektasi | Semantik bahasa |
| Diferensial | Runner khusus | Oracle Rust vs VM C |
| Integrasi | Pytest atau runner Aksara | Jembatan Python, CLI, LSP |
| Fuzz | cargo-fuzz, libFuzzer, AFL++ | Parser, kompiler, verifier, VM, GC |
| Sanitizer | ASan, UBSan, MSan, TSan | Memori dan konkurensi |
| Benchmark | hyperfine dan harness khusus | Performa dan memori |

### 8.2 Suite kepatuhan

Struktur direktori:

```
uji/kepatuhan/
  bahasa/
    penugasan.aks
    kendali.aks
    fungsi.aks
    kelas.aks
    eksepsi.aks
    generator.aks
    modul.aks
    pola.aks
    asinkron.aks
  pustaka/
    matematika.aks
    alat_teks.aks
    json.aks
    sqlite.aks
  jembatan/
    numpy.aks
    pandas.aks
    cv2.aks
    requests.aks
```

Setiap uji memiliki:

- `nama.aks`: program.
- `nama.keluaran`: stdout yang diharapkan.
- `nama.kesalahan`: stderr yang diharapkan, opsional.
- `nama.kode`: kode keluar yang diharapkan, default 0.
- `nama.argumen`: argumen CLI, opsional.

Runner menjalankan program di oracle dan VM C, lalu membandingkan hasil. Perbedaan yang disengaja dicatat di `uji/kepatuhan/pengecualian.toml`.

### 8.3 Uji diferensial

- Oracle Rust vs VM C: semua uji kepatuhan dijalankan di kedua mesin.
- Aksara vs Python: pasangan program dengan semantik setara, output dibandingkan.
- Fuzz diferensial: AST acak dievaluasi di kedua mesin, hasil dibandingkan.
- Perbedaan yang ditemukan menjadi bug atau entri pengecualian yang harus disetujui.

### 8.4 Fuzzing

- `cargo-fuzz` untuk lekser, parser, dan kompiler.
- libFuzzer atau AFL++ untuk verifier, VM, dan GC.
- Korpus awal berasal dari suite kepatuhan.
- Setiap rilis menjalankan fuzz minimal 24 jam.
- Sebelum v1.0, fuzz dijalankan 7 hari berturut-turut.

### 8.5 Sanitizer

- ASan dan UBSan untuk seluruh uji C.
- MSan untuk memori yang tidak diinisialisasi.
- TSan untuk uji utas dan async.
- Valgrind untuk pemeriksaan kebocoran.
- Setiap sanitizer memiliki job CI terpisah.

### 8.6 Benchmark

- 10 program inti: fibonacci, loop numerik, operasi teks, daftar, kamus, OOP, eksepsi, generator, JSON, dan jembatan NumPy.
- Pembanding: CPython 3.12 dan 3.13.
- Startup diukur dengan hyperfine.
- Memori diukur dengan `/usr/bin/time -v` dan penghitung internal.
- Hasil disimpan di `benchmark/hasil/` dan dibandingkan per commit.

### 8.7 Matriks CI

| Dimensi | Nilai |
|---|---|
| Sistem operasi | Ubuntu 24.04, Ubuntu 26.04, macOS 14, macOS 15, Windows Server 2022, Windows Server 2025 |
| Arsitektur | x86_64, aarch64 |
| Kompiler C | GCC 13, GCC 14, Clang 18, Clang 19, MSVC 2022 |
| Rust | stable, beta, MSRV |
| Python | 3.11, 3.12, 3.13 |
| Mode build | tanpa Python, dengan Python, debug, release |
| Ekosistem | 50 paket per commit, 250 nightly, 1000 mingguan, 2000 per rilis |
| Job | test, sanitizer, fuzz smoke, benchmark, docs, lint, ekosistem-tier-a, ekosistem-isolasi, ekosistem-pip |

### 8.8 Gerbang rilis

- 100% uji kepatuhan lulus.
- Tidak ada sanitizer error.
- Tidak ada bug P0 atau P1.
- Benchmark memenuhi target.
- Dokumentasi diperbarui.
- Changelog ditulis.
- Audit keamanan selesai.
- 200+ paket Python lulus Tier A atau B.
- 1000+ paket lulus korpus mingguan dan 2000 paket lulus gerbang rilis.
- Tidak ada regresi Tier A.
- Setiap pengecualian Tier D memiliki ADR dan tanggal tinjauan.
- Dashboard kompatibilitas diperbarui dan dapat diakses publik.
- pip, virtualenv, requirements, pyproject, dan lockfile diuji di tiga sistem operasi.

### 8.9 Uji ekosistem Python

Uji ekosistem berbeda dari uji kepatuhan bahasa. Uji kepatuhan memeriksa semantik Aksara. Uji ekosistem memeriksa apakah paket Python nyata dapat dipasang, diimpor, dipanggil, dan dibersihkan dengan benar melalui jembatan.

Struktur:

```
uji/ekosistem/
  paket.toml
  tier_a/
    numpy/
      dasar.aks
      buffer.aks
      callback.aks
      eksepsi.aks
    pandas/
    requests/
    bs4/
    lxml/
    opencv/
    sqlalchemy/
    matplotlib/
  tier_b/
  tier_c/
  isolasi/
  pip/
  laporan/
```

Setiap paket memiliki manifes dan empat jenis uji minimum:

1. **Impor dan versi.** Modul dapat diimpor, versi terbaca, dan simbol utama tersedia.
2. **Panggilan dasar.** Fungsi atau kelas utama dapat dipanggil dengan argumen posisional dan kata kunci.
3. **Pertukaran data.** Tipe primitif, koleksi, dan buffer bolak-balik tanpa kehilangan data.
4. **Pembersihan.** Sumber daya ditutup, worker berhenti, dan tidak ada proses zombie.

Uji tambahan sesuai kemampuan paket:

- Callback Aksara yang dipanggil library Python.
- Eksepsi Python yang ditangkap `KesalahanPython` dan eksepsi Aksara yang ditangkap Python.
- Iterator dan context manager.
- Async iterator dan awaitable.
- Thread dan GIL.
- Buffer protocol dan zero-copy NumPy.
- Mode isolasi, timeout, dan restart worker.
- Pemasangan pip, venv, dan lockfile.

Aturan uji:

- Setiap uji berjalan di proses terpisah agar crash tidak menghentikan suite.
- Setiap uji memiliki batas waktu. Default 60 detik, dapat diperbesar per manifes.
- Paket dipin ke versi tertentu di CI. Pengujian versi terbaru dilakukan di job terpisah yang tidak memblokir rilis.
- Uji tidak memakai kredensial rahasia. Layanan eksternal diganti dengan server lokal atau mock.
- Uji yang membutuhkan jaringan diberi label `jaringan` dan dapat dilewati pada build offline.
- Uji yang membutuhkan GPU, browser, atau database diberi label `eksternal` dan dijalankan di runner khusus.
- Kegagalan uji menyimpan log, versi paket, versi Python, OS, dan perintah reproduksi.
- Uji yang flaky dijalankan ulang satu kali. Jika lulus, ditandai flaky. Jika gagal lagi, menjadi bug.
- Uji ekosistem tidak boleh bergantung pada urutan eksekusi. Setiap uji membersihkan state global Python yang dibuatnya.
- Uji isolasi memeriksa worker yang mati, pesan rusak, timeout, callback bersarang, shared memory yang dilepas, dan exception yang tidak dapat diserialisasi.

Pemilihan paket:

- 50 paket inti per commit dipilih dari Tier A yang paling banyak dipakai.
- 250 paket nightly mencakup web, data, gambar, sains, basis data, GUI, otomasi, dan keamanan.
- 1000 paket mingguan ditambah dari peringkat PyPI dan laporan pengguna.
- 2000 paket rilis mencakup seluruh Tier A/B dan sampel Tier C.
- Tidak ada daftar putih. Paket di luar korpus tetap didukung secara kontraktual dan dapat diuji melalui `aksara kompatibilitas periksa`.
- Paket yang dilaporkan pengguna masuk antrean uji dalam tujuh hari kerja.

Keamanan CI:

- Paket Python menjalankan kode arbitrer saat pemasangan dan impor. Runner uji memakai container sekali pakai.
- Tidak ada token rilis, kredensial produksi, atau kunci tanda tangan di runner ekosistem.
- Jaringan dibatasi untuk job yang tidak memerlukan unduhan.
- Artefak wheel disimpan dengan hash dan tidak dieksekusi di luar container uji.
- Kegagalan yang mencurigakan dapat dilaporkan sebagai masalah keamanan, bukan bug biasa.

## 9. Alat dan ekosistem

### 9.1 CLI

| Perintah | Fungsi |
|---|---|
| `aksara` | Membuka REPL |
| `aksara jalankan berkas.aks` | Menjalankan program |
| `aksara periksa berkas.aks` | Memeriksa sintaks dan tipe |
| `aksara format` | Memformat kode |
| `aksara uji` | Menjalankan uji proyek |
| `aksara pasang` | Memasang dependensi Aksara |
| `aksara debug` | Menjalankan debugger |
| `aksara dokumentasi` | Membuat dokumentasi |
| `aksara info` | Menampilkan informasi build dan Python |
| `aksara versi` | Menampilkan versi |
| `aksara jelaskan E-NAMA-001` | Menjelaskan kode diagnostik |
| `aksara python info` | Menampilkan interpreter dan venv Python |
| `aksara python pasang` | Memasang paket Python |
| `aksara python beku` | Menulis lockfile Python |
| `aksara python sinkron` | Memasang ulang dari lockfile Python |
| `aksara python dokter` | Memeriksa kesehatan jembatan Python |
| `aksara kompatibilitas periksa` | Menguji satu paket Python |
| `aksara kompatibilitas laporan` | Membuat laporan tier dan batasan |
| `aksara dokter` | Memeriksa instalasi Aksara secara keseluruhan |
| `aksara tutorial` | Membuka kurikulum pemula 10 pelajaran |
| `aksara contoh` | Menjalankan contoh resmi |

Opsi global: `--tanpa-python`, `--isolasi-python`, `--python JALUR`, `--izinkan-evaluasi`, `--pemula`, `--ringkas`, `--jejak`, `--diagnostik=json`, `--mesin oracle|c`, `--verbose`, `--warna auto|selalu|tidak`.

### 9.2 REPL

- Multiline dengan indentasi otomatis.
- Riwayat perintah dan pencarian.
- Completion untuk nama, atribut, dan modul.
- Syntax highlight.
- Perintah `:bantuan`, `:keluar`, `:muat`, `:simpan`.
- Memakai oracle sampai Fase 3, lalu VM C secara default.

### 9.3 Formatter

- Berbasis AST, bukan teks.
- Idempoten: format dua kali menghasilkan berkas yang sama.
- 4 spasi, lebar baris default 88, trailing comma pada koleksi multiline.
- Komentar dipertahankan dan ditempatkan pada posisi terdekat.
- Konfigurasi di `aksara.toml` bagian `[format]`.
- Mode `--periksa` untuk CI.

### 9.4 Linter

Aturan awal:

- Variabel tidak terpakai.
- Variabel bayangan.
- Kode tak terjangkau.
- Default argumen mutable.
- `tangkap` tanpa tipe.
- Perbandingan `adalah` dengan literal.
- Import tidak terpakai.
- Fungsi terlalu panjang.
- Kompleksitas siklomatik tinggi.
- Nama tidak sesuai konvensi.
- `lewati` yang tidak perlu.
- Penggunaan `global` berlebihan.

### 9.5 LSP

Kemampuan:

- Diagnostik saat mengetik.
- Completion.
- Hover dan dokumentasi.
- Signature help.
- Go to definition dan references.
- Rename simbol.
- Formatting.
- Code actions.
- Semantic tokens.

### 9.6 Debugger

- Protokol DAP.
- Breakpoint biasa dan kondisional.
- Step in, step over, step out.
- Variabel lokal, global, dan closure.
- Call stack.
- Evaluate expression.
- Watch.
- Exception breakpoint.

### 9.7 Package manager

- Manifest `aksara.toml`.
- Lockfile `aksara.kunci`.
- Registry default `pustaka.aksara.dev` (nama domain diperiksa di Fase 0).
- Cache di `~/.aksara/paket`.
- Lingkungan virtual per proyek di `.aksara/lingkungan`.
- Workspace untuk monorepo.
- Vendoring.
- Checksum SHA-256.
- Tanda tangan paket pasca 1.0.

Contoh `aksara.toml`:

```toml
[paket]
nama = "proyek_saya"
versi = "0.1.0"
penulis = ["Nama <email>"]
deskripsi = "Contoh proyek Aksara"
lisensi = "MIT"

[dependensi]
pustaka_web = "1.2.0"
pustaka_data = { versi = "0.3", sumber = "registry" }

[python]
aktif = benar
versi = "3.12"
isolasi = ["scrapy", "playwright"]
batas_waktu = 60

[skrip]
uji = "aksara uji"
format = "aksara format ."
python = "aksara python sinkron"
```

### 9.8 Dokumentasi

- mdBook untuk situs dokumentasi.
- Docstring diekstrak oleh `aksara dokumentasi`.
- Referensi bahasa, tutorial, cookbook, dan panduan migrasi.
- Pencarian sisi klien.
- Versi dokumentasi per rilis.

### 9.9 Integrasi editor

- Ekstensi VS Code dengan LSP dan debugger.
- Grammar Tree-sitter untuk syntax highlight.
- Konfigurasi Neovim.
- Dukungan dasar untuk JetBrains melalui LSP.
- Completion untuk objek `impor python` melalui introspeksi CPython.
- Hover menampilkan docstring Python asli.
- Signature help memakai `inspect.signature` jika tersedia.
- Diagnostik menandai paket Python yang belum terpasang dan menyarankan `aksara python pasang`.
- Debugger menampilkan frame Python di samping frame Aksara saat `--jejak` aktif.

### 9.10 Alat jembatan dan kompatibilitas

Alat jembatan dibuat terpisah dari compiler inti agar pengguna dapat mendiagnosis masalah Python tanpa membaca log VM.

`aksara python`:

- `info` mencetak versi Python, ABI, ukuran pointer, jalur `libpython`, venv, `sys.path`, versi pip, dan mode jembatan.
- `pasang` memakai pip di venv proyek. Perintah ini tidak menulis resolver sendiri.
- `beku` menulis `aksara-python.kunci` dengan versi dan hash.
- `sinkron` memverifikasi lockfile lalu memasang ulang.
- `dokter` memeriksa interpreter, shared library, SSL, zlib, SQLite, pip, dan impor NumPy.
- `jalankan` menjalankan skrip Python di venv proyek untuk alat bantu atau migrasi.

`aksara kompatibilitas`:

- `periksa nama_paket` menjalankan probe di proses terpisah dan mencetak tier, mode, serta batasan.
- `laporan` menghasilkan Markdown atau JSON untuk proyek pengguna.
- `jalankan --manifest ...` mereproduksi satu kasus uji ekosistem.
- `bandingkan` membandingkan hasil dua versi Aksara atau dua versi Python.

`aksara jelaskan`:

- Menampilkan penjelasan kode error, contoh salah, contoh benar, dan tautan dokumentasi.
- Bekerja offline karena halaman penjelasan disertakan dalam instalasi.
- Mendukung output JSON untuk editor.

`aksara dokter`:

- Memeriksa binary, runtime C, standard library, bytecode, dan izin berkas.
- Memeriksa konfigurasi `aksara.toml`.
- Memeriksa interpreter Python dan venv.
- Memeriksa apakah `AKSARA_PATH` menunjuk ke direktori yang tidak ada.
- Menyarankan perbaikan konkret, bukan hanya mencetak status.

`aksara tutorial`:

- Menjalankan kurikulum 10 pelajaran secara interaktif.
- Setiap pelajaran memiliki contoh, latihan, dan pemeriksaan jawaban.
- Mode `--tanpa-jaringan` memakai contoh lokal.
- Progres disimpan di `~/.aksara/tutorial.toml`.

Skema diagnostik JSON:

```json
{
  "kode": "E-NAMA-001",
  "kategori": "nama",
  "pesan": "nama 'jumblah' belum didefinisikan",
  "berkas": "contoh.aks",
  "baris": 3,
  "kolom": 13,
  "panjang": 7,
  "saran": ["jumlah"],
  "tautan": "https://docs.aksara.dev/diagnostik/E-NAMA-001",
  "tingkat": "kesalahan"
}
```

Skema ini stabil selama satu siklus mayor dan dipakai oleh LSP, linter, formatter, dan CI.

## 10. Performa

### 10.1 Target

| Metrik | Target |
|---|---|
| Startup tanpa jembatan | Kurang dari 50 ms |
| Startup dengan jembatan | Kurang dari 150 ms |
| Benchmark inti | 3x CPython 3.12 |
| Memori | Kurang dari 1.5x CPython |
| Panggilan jembatan in-process | Kurang dari 1 mikrodetik |
| Panggilan jembatan isolasi | Kurang dari 100 mikrodetik |
| Buffer NumPy zero-copy | Kurang dari 5 mikrodetik per operasi |
| Pause GC | Kurang dari 50 ms pada heap 512 MB |
| Kompilasi 10.000 baris | Kurang dari 1 detik |

### 10.2 Metodologi

- Ukur sebelum mengoptimalkan.
- Profil dengan `perf`, `valgrind --tool=callgrind`, dan profiler internal.
- Benchmark dijalankan di mesin CI khusus dengan CPU yang konsisten.
- Setiap optimisasi harus memiliki bukti peningkatan dan tidak menurunkan keterbacaan kode secara berlebihan.
- Regresi lebih dari 5% memblokir merge.

### 10.3 Teknik

- Direct-threaded dispatch dengan computed goto pada GCC dan Clang.
- Inline caching untuk atribut, metode, dan variabel global.
- Opcode khusus untuk operasi int, float, teks, daftar, dan kamus.
- Local variable slot, bukan pencarian nama di kamus.
- Constant folding dan peephole optimization.
- Interned string untuk nama atribut dan identifier.
- Kamus dengan compact layout dan insertion order.
- Generational GC dan incremental marking pada Fase 8.
- Batch API untuk jembatan NumPy.

JIT tidak menjadi target v1.0. JIT direncanakan untuk v1.2 sebagai komponen Rust terpisah yang berkomunikasi melalui ABI C.

## 11. Keamanan dan keandalan

### 11.1 Model ancaman

| Ancaman | Dampak | Mitigasi |
|---|---|---|
| Berkas `.aks` berbahaya | Eksekusi kode | Bukan sandbox, dokumentasikan dengan jelas |
| Bytecode `.akc` rusak | Crash atau memori tidak aman | Verifier ketat sebelum eksekusi |
| Paket berbahaya | Eksekusi kode saat instalasi | Checksum, tinjauan, tanda tangan pasca 1.0 |
| Library Python berbahaya | Akses sistem penuh | Dokumentasi, bukan sandbox |
| Bug memori di VM | Crash atau kerentanan | Sanitizer, fuzzing, review |
| Cross-runtime cycle | Kebocoran memori | Deteksi, batasi, weakref |

### 11.2 Prinsip

- Bytecode tidak dapat mengakses pointer mentah.
- Semua akses memori dari bytecode melewati objek dan tipe yang terdaftar.
- Verifier memeriksa bytecode sebelum dieksekusi.
- Hash string diacak per proses. Seed dapat diatur dengan `AKSARA_HASH_SEED` untuk pengujian.
- Build rilis memakai hardening: `-fstack-protector-strong`, `-D_FORTIFY_SOURCE=2`, RELRO, dan PIE.
- Tidak ada `eval` implisit dari string.
- `evaluasi`, `jalankan`, dan `kompilasi` hanya aktif dengan `--izinkan-evaluasi`. Tanpa flag tersebut, ketiganya melempar `KesalahanFiturTidakTersedia`.
- Jembatan Python tidak menjanjikan sandbox.
- Kode tidak tepercaya harus dijalankan di container atau proses terpisah.

### 11.3 Pelaporan kerentanan

- Alamat surel keamanan diumumkan sebelum v1.0.
- Kebijakan pengungkapan bertanggung jawab 90 hari.
- Kredit untuk pelapor.

## 12. Paket dan distribusi

### 12.1 Artefak

| Platform | Format |
|---|---|
| Linux | `.tar.gz`, `.deb`, `.rpm`, container OCI |
| macOS | `.pkg`, Homebrew |
| Windows | `.msi`, Scoop, Winget |
| Sumber | Tarball sumber dengan skrip build |

### 12.2 Isi paket

- Binary `aksara`.
- Binary `aksara-python-worker` untuk mode isolasi.
- Runtime `libaksara_runtime`.
- Standard library Aksara.
- Katalog diagnostik offline.
- Dokumentasi.
- Lisensi dan pemberitahuan lisensi pihak ketiga.
- Skrip pelengkapan untuk shell.
- Contoh dan proyek pemula.
- Berkas konfigurasi default `aksara.toml`.

### 12.3 Jembatan Python

- Linux: tautan dinamis ke `libpython3.xx.so`. Pencarian memakai `RPATH` relatif, `LD_LIBRARY_PATH`, lalu jalur interpreter.
- macOS: framework Python, Homebrew Python, atau python.org. Binary ditandatangani dan dinotarisasi.
- Windows: `python3xx.dll` dari instalasi Python resmi atau embeddable distribution.
- `aksara info` dan `aksara python info` menampilkan versi Python, jalur `libpython`, ABI, venv, dan status jembatan.
- Jika `libpython` tidak ditemukan, installer menawarkan mode tanpa jembatan. Build `--tanpa-python` tetap berfungsi penuh untuk bahasa inti.
- Varian installer `bundel` menyertakan CPython 3.12 relocatable, pip, dan venv. Varian ini untuk pengguna yang belum memiliki Python atau ingin versi yang konsisten.
- Varian `bundel` tidak menyertakan seluruh paket Python. Paket dipasang melalui `aksara python pasang` agar pengguna memilih versi dan lisensi sendiri.
- Container OCI menyertakan Python 3.12, pip, dan paket umum untuk demo: NumPy, pandas, OpenCV headless, requests, BeautifulSoup, lxml, dan matplotlib. Versi dipin dan hash disimpan.
- Installer menyediakan mode offline dengan cache wheel lokal. Mode ini tidak menjamin semua paket tersedia.
- Lisensi CPython, pip, dan paket pihak ketiga disertakan dalam `PEMBERITAHUAN-LISENSI`.
- Distribusi tidak menautkan `libpython` secara statis ke binary utama. Tautan dinamis menjaga lisensi, ukuran, dan kompatibilitas ABI.
- Di Windows, runtime Visual C++ yang dibutuhkan dideteksi oleh `aksara python dokter`. Jika tidak ada, installer menautkan halaman unduhan resmi.
- Di Linux, `manylinux_2_28` menjadi baseline. Distribusi lama memakai container atau build dari sumber.
- Di macOS, versi minimum adalah 12.0. Binary arm64 dan x86_64 dibangun terpisah, lalu digabung menjadi universal2 jika dependensi mendukung.

### 12.4 Versioning

- Bahasa mengikuti SemVer.
- Bytecode memiliki versi mayor dan minor. Verifier menolak versi mayor yang berbeda.
- Paket mengikuti SemVer.
- Kebijakan deprecation: minimal dua rilis minor sebelum penghapusan.
- Setelah v1.0, penambahan sintaks hanya dilakukan pada rilis minor dan tidak merusak kode yang ada.

## 13. Organisasi repositori

```
.
├── Cargo.toml
├── justfile
├── CMakeLists.txt
├── README.md
├── LICENSE
├── plan.md
├── crates/
│   ├── aksara-lexer/
│   ├── aksara-parser/
│   ├── aksara-ast/
│   ├── aksara-semantik/
│   ├── aksara-kompiler/
│   ├── aksara-diagnostik/
│   ├── aksara-interpreter/
│   ├── aksara-repl/
│   ├── aksara-cli/
│   ├── aksara-format/
│   ├── aksara-lint/
│   ├── aksara-lsp/
│   ├── aksara-debug/
│   ├── aksara-paket/
│   ├── aksara-python/
│   ├── aksara-kompatibilitas/
│   └── aksara-dok/
├── runtime/
│   ├── include/
│   │   └── aksara.h
│   ├── src/
│   │   ├── objek.c
│   │   ├── alokasi.c
│   │   ├── gc.c
│   │   ├── teks.c
│   │   ├── koleksi.c
│   │   ├── numerik.c
│   │   ├── kelas.c
│   │   ├── fungsi.c
│   │   ├── frame.c
│   │   ├── generator.c
│   │   ├── eksepsi.c
│   │   ├── modul.c
│   │   ├── cocok.c
│   │   ├── verifier.c
│   │   ├── vm.c
│   │   ├── builtin.c
│   │   └── profil.c
│   ├── jembatan/
│   │   ├── python.c
│   │   ├── marshal.c
│   │   ├── proxy.c
│   │   ├── numpy.c
│   │   ├── callback.c
│   │   ├── eksepsi.c
│   │   ├── memori.c
│   │   ├── probe.c
│   │   ├── isolasi.c
│   │   ├── protokol.c
│   │   ├── sharedmem.c
│   │   ├── pip.c
│   │   ├── deteksi.c
│   │   └── worker/
│   └── uji/
├── pustaka/
│   ├── matematika.aks
│   ├── alat_teks.aks
│   ├── json.aks
│   ├── tanggal.aks
│   ├── alat_berkas.aks
│   ├── jaringan.aks
│   ├── sqlite.aks
│   ├── pengujian.aks
│   └── ...
├── uji/
│   ├── kepatuhan/
│   ├── diferensial/
│   ├── integrasi/
│   ├── ekosistem/
│   │   ├── paket.toml
│   │   ├── tier_a/
│   │   ├── tier_b/
│   │   ├── tier_c/
│   │   ├── isolasi/
│   │   ├── pip/
│   │   └── laporan/
│   ├── fuzz/
│   └── benchmark/
├── docs/
│   ├── visi.md
│   ├── spesifikasi/
│   ├── adr/
│   ├── kompatibilitas/
│   ├── diagnostik/
│   ├── pustaka/
│   ├── distribusi/
│   └── panduan/
├── contoh/
│   ├── pemula/
│   ├── bahasa/
│   ├── ekosistem/
│   └── aplikasi/
├── editor/
│   ├── vscode/
│   ├── tree-sitter/
│   └── neovim/
└── alat/
    ├── kompatibilitas/
    └── rilis/
```

Aturan:

- Semua folder utama berada di akar repositori. Tidak ada folder pembungkus tambahan seperti `langku/` di dalam tree.
- Setiap crate memiliki README dan batas tanggung jawab yang jelas.
- Kode C berada di `runtime/`, tidak di crate Rust.
- Kode Aksara untuk standard library berada di `pustaka/`.
- Semua contoh harus dapat dijalankan di CI.
- Dokumen spesifikasi diperlakukan sebagai kode: perubahan harus melalui review.

## 14. Risiko dan mitigasi

| Risiko | Dampak | Kemungkinan | Mitigasi |
|---|---|---|---|
| Cakupan ekosistem terlalu luas | Klaim tidak dapat dibuktikan | Tinggi | Kontrak kemampuan, tier otomatis, 2000 paket per rilis, batasan 5.12, dashboard |
| Matriks uji ekosistem meledak | Biaya CI dan waktu | Tinggi | Sharding, 50/250/1000/2000, cache wheel, runner spot |
| Bug memori di VM C | Crash dan kerentanan | Tinggi | GC non-moving, sanitizer, fuzzing, review wajib |
| Divergensi oracle dan VM | Semantik tidak konsisten | Sedang | Uji diferensial dan suite kepatuhan |
| Jembatan Python rapuh | Library tidak berjalan | Tinggi | Tier, uji tiga OS, isolasi subprocess, probe otomatis |
| ABI CPython berubah | Build rusak | Sedang | Dukungan 3.11 sampai 3.13, CI matriks, uji per versi |
| Paket memeriksa internal CPython | Crash atau hasil salah | Sedang | Probe, Tier B/C, isolasi, dokumentasi versi |
| Paket memakai fork, signal, atau `sys.executable` | Deadlock atau proses gagal | Tinggi | Deteksi otomatis, isolasi, start method `spawn` |
| Konflik library native | Crash sulit direproduksi | Sedang | Isolasi kelompok paket, dokumentasi OpenMP/MKL/CUDA |
| pip atau venv tidak konsisten | Paket tidak terpasang | Sedang | Pakai pip asli, lockfile, `aksara python dokter` |
| Wheel tidak tersedia untuk platform | Build dari sumber gagal | Sedang | Matriks wheel, pemeriksaan compiler, Tier C/D |
| Ekosistem bergerak cepat | Regresi pada versi baru | Tinggi | Nightly, laporan pengguna, pin versi, job versi terbaru |
| Overhead jembatan tinggi | Performa buruk | Sedang | Fast path, batch API, zero-copy, benchmark |
| Sintaks bahasa alami ambigu | Parser salah membaca | Sedang | Lookahead terbatas, uji golden, larangan chaining `antara` |
| `??` dan `?.` disalahpahami | Bug logika | Sedang | Diagnostik, linter, contoh, perbedaan dengan `atau` |
| Keamanan pemasangan paket | Eksekusi kode saat install | Tinggi | Peringatan, index tepercaya, container, hash |
| Performa di bawah CPython | Adopsi rendah | Sedang | Benchmark awal, inline caching, profil |
| Standard library terlalu besar | Fase 5 molor | Tinggi | Prioritaskan modul untuk teks, data, berkas, dan jaringan |
| Package manager terlambat | Ekosistem tidak tumbuh | Sedang | MVP lokal di Fase 7, registry publik pasca 1.0 |
| Kurang kontributor | Pengembangan lambat | Tinggi | Dokumentasi, good first issue, lisensi terbuka |
| Konflik nama Aksara | Masalah hukum | Rendah | Pemeriksaan merek dan domain di Fase 0 |
| Lisensi dependensi | Masalah distribusi | Rendah | Audit lisensi sebelum v0.5 |
| Cross-runtime cycle | Kebocoran memori | Sedang | Batasi di v1.0, deteksi, weakref |
| Threading dan GIL | Deadlock | Sedang | GIL global, uji TSan, skenario terkontrol |
| Dokumentasi tidak sinkron | Kebingungan pengguna | Sedang | Docs as code, CI menjalankan contoh |

## 15. Definisi selesai v1.0

- Spesifikasi bahasa v1.0 dibekukan dan dipublikasikan.
- 95% matriks paritas fitur terpenuhi.
- 80% modul standard library inti selesai.
- 200+ paket Python lulus uji Tier A atau B.
- 1000+ paket lulus korpus mingguan dan 2000 paket lulus gerbang rilis.
- Tidak ada daftar putih paket. Semua pengecualian Tier D memiliki ADR dan alasan teknis.
- pip, virtualenv, requirements, pyproject, dan lockfile Python berfungsi di tiga sistem operasi.
- Mode isolasi Python berfungsi untuk paket yang membutuhkan fork, signal, atau proses sendiri.
- Dashboard kompatibilitas publik menampilkan tier, status, dan regresi.
- 1500+ uji kepatuhan lulus di oracle dan VM C.
- Performa 3x CPython pada 10 benchmark inti.
- Memori kurang dari 1.5x CPython pada program yang sama.
- Startup tanpa jembatan kurang dari 50 ms dan startup dengan jembatan kurang dari 150 ms.
- Berjalan di Linux, macOS, dan Windows dengan Python 3.11, 3.12, dan 3.13.
- Formatter, linter, LSP, debugger, dan package manager berfungsi.
- Diagnostik ber-kode `E-*` tersedia offline dan dapat dijelaskan dengan `aksara jelaskan`.
- Kurikulum pemula 10 pelajaran, cheat sheet sintaks, dan cheat sheet operator tersedia.
- 10 aplikasi contoh berjalan.
- Dokumentasi lengkap dalam bahasa Indonesia.
- Tidak ada bug P0 atau P1.
- Audit keamanan selesai.
- Paket instalasi tersedia dan terverifikasi.
- Kebijakan kompatibilitas dan deprecation dipublikasikan.

## 16. Pasca 1.0

| Versi | Fokus |
|---|---|
| v1.1 | `except*`, exception group, type parameter syntax, modul stdlib tambahan |
| v1.2 | JIT eksperimental berbasis Rust, profiling lebih dalam |
| v1.3 | WebAssembly untuk playground dan aplikasi browser |
| v1.4 | Registry paket publik, tanda tangan paket |
| v2.0 | ABI ekstensi C versi 2, kemungkinan transpiler Python, paralelisme CPU |

Setelah v1.0, prioritas berubah dari kelengkapan fitur menjadi stabilitas, performa, dan pertumbuhan ekosistem.

## Lampiran A: Kata kunci

| Python | Aksara | Catatan |
|---|---|---|
| `True` | `benar` | |
| `False` | `salah` | |
| `None` | `kosong` | |
| `and` | `dan` | |
| `or` | `atau` | |
| `not` | `bukan` | |
| `if` | `jika` | |
| `elif` | `lainnya jika` | Dua token |
| `else` | `lainnya` | |
| `while` | `selama` | |
| `for` | `untuk` | |
| `in` | `dalam` | |
| `break` | `hentikan` | |
| `continue` | `lanjutkan` | |
| `pass` | `lewati` | |
| `def` | `fungsi` | |
| `return` | `kembalikan` | |
| `yield` | `hasilkan` | |
| `class` | `kelas` | |
| `self` | `diri` | |
| `super` | `induk` | |
| `try` | `coba` | |
| `except` | `tangkap` | |
| `finally` | `akhirnya` | |
| `raise` | `lempar` | |
| `with` | `dengan` | |
| `as` | `sebagai` | |
| `import` | `impor` | |
| `from` | `dari` | |
| `global` | `global` | |
| `nonlocal` | `nonlokal` | |
| `del` | `hapus` | |
| `assert` | `pastikan` | |
| `lambda` | `lambda` | Pengecualian pragmatis |
| `async` | `asinkron` | |
| `await` | `tunggu` | |
| `is` | `adalah` | |
| `is not` | `bukan adalah` | Dua token |
| `not in` | `bukan dalam` | Dua token |
| `match` | `cocokkan` | Fase 4 |
| `case` | `kasus` | Fase 4 |
| `_` pada pola | `_` | Wildcard, bukan identifier biasa |

Kata kunci dan operator kata tambahan yang tidak memiliki padanan langsung di Python:

| Bentuk | Jenis | Arti | Posisi yang sah |
|---|---|---|---|
| `antara a dan b` | operator perbandingan | Rentang tertutup `a <= x <= b` | Di dalam ekspresi |
| `bukan antara a dan b` | operator perbandingan | Negasi `antara` | Di dalam ekspresi |
| `setiap` | kata kunci kontekstual | Penanda iterasi yang dibaca seperti kalimat | Hanya setelah `untuk` |
| `sebanyak` | kata kunci kontekstual | Ulangi n kali | Hanya setelah `untuk nama` |
| `sampai` | kata kunci kontekstual | Batas akhir tertutup | Hanya di dalam bentuk `untuk ... dari ... sampai ...` |
| `langkah` | kata kunci kontekstual | Langkah perulangan | Hanya setelah bentuk `dari ... sampai ...` |
| `lainnya` pada ternary | kata kunci dengan peran ganda | Pemisah cabang salah | Hanya di dalam ekspresi ternary |
| `dari` pada perulangan | kata kunci dengan peran ganda | Awal rentang tertutup | Hanya setelah `untuk nama` |

Aturan:

- Kata kunci kontekstual tidak dicadangkan secara global. `setiap`, `sebanyak`, `sampai`, dan `langkah` tetap dapat dipakai sebagai nama variabel di luar posisi tersebut.
- `antara` adalah kata kunci operator dan tidak dapat dipakai sebagai nama variabel.
- `dari` tetap kata kunci impor. Parser membedakan pemakaian impor dan pemakaian rentang dari posisi sintaks.
- Bentuk Inggris seperti `True`, `False`, `None`, `elif`, dan `else` ditolak dengan saran padanan Indonesia.

## Lampiran B: Fungsi bawaan

| Python | Aksara | Fase |
|---|---|---|
| `print` | `cetak` | 2 |
| `len` | `panjang` | 2 |
| `range` | `rentang` | 2 |
| `range` dengan batas tertutup | `rentang_tertutup` | 2 |
| `enumerate` | `enumerasi` | 2 |
| `zip` | `pasangkan` | 2 |
| `map` | `peta` | 2 |
| `filter` | `saring` | 2 |
| `sum` | `jumlah` | 2 |
| `min` | `minimum` | 2 |
| `max` | `maksimum` | 2 |
| `all` | `semua` | 2 |
| `any` | `ada` | 2 |
| `sorted` | `urutkan` | 2 |
| `reversed` | `terbalik` | 2 |
| `abs` | `absolut` | 2 |
| `divmod` | `bagi_sisa` | 2 |
| `pow` | `pangkat` | 2 |
| `round` | `bulat` | 2 |
| `type` | `tipe` | 2 |
| `isinstance` | `termasuk` | 2 |
| `hasattr` | `memiliki` | 2 |
| `getattr` | `ambil` | 2 |
| `setattr` | `pasang` | 2 |
| `open` | `buka` | 2 |
| `input` | `masukan` | 2 |
| `help` | `bantuan` | 2 |
| `dir` | `daftar_atribut` | 2 |
| `id` | `identitas` | 2 |
| `hash` | `hash` | 2 |
| `iter` | `iterasi` | 2 |
| `next` | `berikutnya` | 2 |
| `repr` | `representasi` | 2 |
| `format` | `format` | 2 |
| `eval` | `evaluasi` | 4 (opt-in) |
| `exec` | `jalankan` | 4 (opt-in) |
| `compile` | `kompilasi` | 4 (opt-in) |

Konstruktor tipe:

| Python | Aksara | Fase |
|---|---|---|
| `int` | `bilangan` | 2 |
| `float` | `pecahan` | 2 |
| `complex` | `kompleks` | 2 |
| `bool` | `logika` | 2 |
| `str` | `teks` | 2 |
| `bytes` | `bytes` | 2 |
| `list` | `daftar` | 2 |
| `tuple` | `tupel` | 2 |
| `dict` | `kamus` | 2 |
| `set` | `himpunan` | 2 |

`susunan` adalah konstruktor khusus Aksara untuk array numerik. Tidak ada padanan langsung di Python; padanan terdekatnya adalah `numpy.ndarray`.

Tanda tangan fungsi yang paling sering dipakai:

- `cetak(*objek, pemisah=" ", akhir="\n", berkas=sistem.keluaran_standar, segarkan=salah)`.
- `buka(berkas, mode="r", encoding="utf-8", buffering=-1, errors="strict", baris_baru=kosong)`.
- `rentang(berhenti)` atau `rentang(mulai, berhenti[, langkah])`. Batas akhir tidak ikut dihitung.
- `rentang_tertutup(mulai, berhenti[, langkah])`. Batas akhir ikut dihitung. Tanpa `langkah`, arah menurun terdeteksi otomatis. Langkah tidak boleh nol.
- `bagi_sisa(a, b)` mengembalikan `(a // b, a % b)` dengan aturan pembulatan Python.
- `pangkat(a, b[, modulus])`. Tanpa modulus setara `a ** b`. Dengan modulus menghitung `(a ** b) % modulus` secara efisien.
- `semua(iterable)` mengembalikan `benar` jika semua elemen benar, termasuk untuk iterable kosong.
- `ada(iterable)` mengembalikan `benar` jika sedikitnya satu elemen benar, dan `salah` untuk iterable kosong.
- `urutkan(iterable, kunci=kosong, terbalik=salah)` mengembalikan daftar baru.
- `terbalik(iterable)` mengembalikan iterator, bukan daftar.
- `enumerasi(iterable, mulai=0)` menghasilkan pasangan `(indeks, nilai)`.
- `pasangkan(*iterable, ketat=salah)` berhenti pada iterable terpendek, atau melempar `KesalahanNilai` jika `ketat=benar`.
- `masukan(petunjuk="")` membaca satu baris dari masukan standar dan mengembalikan `teks` tanpa karakter baris baru.
- `bulat(angka[, digit])` membulatkan ke bilangan bulat atau ke jumlah digit tertentu. Pembulatan memakai aturan bankir seperti Python.

Padanan nama parameter Python: `pemisah` sama dengan `sep`, `akhir` sama dengan `end`, `berkas` sama dengan `file`, `segarkan` sama dengan `flush`, `baris_baru` sama dengan `newline`, `kunci` sama dengan `key`, `terbalik` sama dengan `reverse`, `ketat` sama dengan `strict`, dan `digit` sama dengan `ndigits`.

Fungsi bawaan yang tidak disediakan:

- `eval`, `exec`, dan `compile` hanya tersedia dengan izin eksplisit `--izinkan-evaluasi`. Secara default ketiganya melempar `KesalahanFiturTidakTersedia` agar kode yang menerima masukan tidak tepercaya tidak dapat menjalankan teks sebagai program.
- `globals`, `locals`, `vars`, dan `__import__` tidak diekspos sebagai fungsi bawaan. Gunakan `sistem` dan `impor` sesuai kebutuhan.
- `ascii` diganti `representasi`, karena `representasi` sudah menghasilkan representasi yang dapat dibaca dan aman untuk teks non-ASCII.
- `bin`, `oct`, dan `hex` tersedia di modul `matematika` sebagai `biner`, `oktal`, dan `heksadesimal`, bukan sebagai fungsi bawaan.
- `chr` dan `ord` tersedia di modul `alat_teks` sebagai `karakter` dan `kode_karakter`.

## Lampiran C: Modul standard library

Daftar lengkap berada di Bagian 6. Prioritas implementasi:

1. Modul yang dibutuhkan untuk menjalankan program dasar: `matematika`, `alat_teks`, `alat_daftar`, `alat_kamus`, `alat_berkas`, `jalur`, `sistem`.
2. Modul untuk scraping dan data: `json`, `csv`, `pola`, `http`, `url`, `tanggal`, `sqlite`.
3. Modul untuk alat: `argumen`, `pencatatan`, `pengujian`, `pengaturan`.
4. Modul untuk konkurensi: `utas`, `antrean`, `kunci`, `alur_asinkron`.
5. Modul untuk kompresi dan encoding: `basis64`, `sidik`, `kompresi`, `codec`.
6. Pembungkus Python: `matriks`, `tabel`, `gambar`.

## Lampiran D: Format bytecode

Header `.akc`:

| Offset | Ukuran | Isi |
|---|---|---|
| 0 | 4 | Magic `AKS1` |
| 4 | 2 | Versi mayor |
| 6 | 2 | Versi minor |
| 8 | 4 | Jumlah konstanta |
| 12 | 4 | Jumlah instruksi |
| 16 | 4 | Offset line table |
| 20 | 4 | Offset exception table |
| 24 | 4 | Offset debug info |
| 28 | 4 | Checksum CRC32 |
| 32+ | bervariasi | Konstanta |
| bervariasi | bervariasi | Kode |
| bervariasi | bervariasi | Line table |
| bervariasi | bervariasi | Exception table |
| bervariasi | bervariasi | Debug info |

Prinsip:

- Semua angka multibyte memakai little-endian.
- Konstanta disimpan dalam pool dengan tipe eksplisit.
- String disimpan sebagai panjang UTF-8 diikuti bytes.
- Verifier berjalan sebelum eksekusi.
- Bytecode tidak menyimpan pointer, hanya indeks.
- Debug info bersifat opsional dan dapat dihapus pada build rilis.

## Lampiran E: Referensi

- Python 3.12 Language Reference.
- CPython source code, terutama `Objects/`, `Python/ceval.c`, dan `Include/`.
- Robert Nystrom, *Crafting Interpreters*.
- Keith Cooper dan Linda Torczon, *Engineering a Compiler*.
- Rust Programming Language, edisi kedua.
- ISO/IEC 9899:2018, C11.
- Dokumentasi PCRE2.
- Dokumentasi utf8proc.
- Dokumentasi libtommath.
- Dokumentasi SQLite.
- Dokumentasi CPython C API.

## Lampiran F: Contoh integrasi Python

### F.1 Scraping

```aksara
impor python "requests" sebagai requests
dari python "bs4" impor BeautifulSoup

jawaban = requests.get("https://contoh.id", timeout=10)
jawaban.raise_for_status()

halaman = BeautifulSoup(jawaban.text, "html.parser")
untuk judul dalam halaman.select("h1"):
    cetak(judul.get_text(strip=benar))
```

### F.2 Analisis data

```aksara
impor python "pandas" sebagai pd

tabel = pd.read_csv("penjualan.csv")
cetak(tabel.head())
cetak(tabel.groupby("kategori")["total"].sum())

tabel["total_bersih"] = tabel["total"] * 0.9
tabel.to_csv("penjualan_bersih.csv", index=salah)
```

### F.3 Pengolahan gambar

```aksara
impor python "cv2" sebagai cv

gambar = cv.imread("masukan.jpg")
jika gambar adalah kosong:
    lempar KesalahanIO("gambar tidak dapat dibaca")

abu = cv.cvtColor(gambar, cv.COLOR_BGR2GRAY)
tepi = cv.Canny(abu, 100, 200)

cv.imwrite("keluaran.jpg", tepi)
cv.imshow("Hasil", tepi)
cv.waitKey(0)
cv.destroyAllWindows()
```

### F.4 NumPy zero-copy

```aksara
impor python "numpy" sebagai np
impor python "cv2" sebagai cv

gambar = cv.imread("masukan.jpg")
cetak(tipe(gambar))
cetak(gambar.shape)
cetak(gambar.dtype)

# Potong tanpa menyalin data
bagian = gambar[100:300, 200:400]
cetak(bagian.shape)
```

### F.5 Callback

```aksara
impor python "functools" sebagai functools

fungsi kali_dua(x):
    kembalikan x * 2

kali_dua_py = functools.partial(kali_dua, 2)
cetak(kali_dua_py())
```

Contoh-contoh ini menjadi target uji integrasi Fase 6. Jika sebuah contoh gagal, jembatan belum dianggap selesai.

## Lampiran G: Ringkasan sintaks

Lampiran ini adalah referensi cepat, bukan spesifikasi. Setiap bentuk di bawah memiliki uji kepatuhan dan contoh di `contoh/bahasa/`.

### G.1 Program minimal

```aksara
# halo.aks
cetak("Halo, dunia")
```

Program berjalan dari atas ke bawah. Tidak ada fungsi `utama` yang wajib. Titik masuk eksplisit dipakai jika berkas juga diimpor sebagai modul:

```aksara
fungsi utama():
    cetak("dijalankan langsung")

jika __nama__ == "__utama__":
    utama()
```

Aturan:

- Berkas sumber memakai ekstensi `.aks`.
- Indentasi 4 spasi. Tab ditolak.
- Komentar dimulai dengan `#`.
- Satu pernyataan per baris. `;` hanya untuk kasus khusus.
- Blok dibuka dengan `:` di akhir header.

### G.2 Variabel dan tipe

```aksara
nama = "Ani"
umur: bilangan = 25
tinggi = 162.5
aktif = benar
kosong_saja = kosong

nilai = [90, 85, 78]
profil = {"nama": nama, "umur": umur}
unik = {1, 2, 3}
titik = (10, 20)
matriks = susunan([[1.0, 2.0], [3.0, 4.0]])
```

Konversi:

```aksara
bilangan("42")        # 42
pecahan("3.14")       # 3.14
teks(42)              # "42"
logika(0)             # salah
logika("halo")        # benar
daftar("abc")         # ["a", "b", "c"]
kamus([("a", 1)])     # {"a": 1}
susunan([1, 2, 3])    # array numerik
```

Aturan:

- Penugasan adalah pernyataan, bukan ekspresi.
- Nama bersifat case-sensitive.
- Anotasi tipe tidak ditegakkan saat runtime kecuali `aksara periksa --tipe`.
- `susunan` adalah array numerik dengan buffer protocol untuk NumPy.

### G.3 Percabangan dan ternary

```aksara
jika nilai >= 90:
    cetak("A")
lainnya jika nilai >= 80:
    cetak("B")
lainnya:
    cetak("C")
```

```aksara
status = "lulus" jika nilai >= 70 lainnya "gagal"
pesan = "besar" jika n > 100 lainnya "sedang" jika n > 10 lainnya "kecil"
```

Aturan:

- `lainnya jika` adalah dua token.
- Ternary hanya untuk ekspresi pendek dan wajib memiliki `lainnya`.
- Tidak ada `? :`, `elif`, atau `else`.

### G.4 Perulangan

```aksara
untuk x dalam [1, 2, 3]:
    cetak(x)

untuk setiap siswa dalam daftar_siswa:
    cetak(siswa.nama)

untuk i sebanyak 5:
    cetak(i)

untuk i dari 1 sampai 5:
    cetak(i)

untuk i dari 10 sampai 1:
    cetak(i)

untuk i dari 1 sampai 10 langkah 2:
    cetak(i)

selama kondisi:
    kerjakan()
lainnya:
    cetak("kondisi menjadi salah")
```

Kendali:

```aksara
hentikan
lanjutkan
lewati
```

Aturan:

- `rentang(n)` setengah terbuka. `dari a sampai b` tertutup.
- `dari a sampai b` hanya untuk bilangan bulat.
- Tanpa `langkah`, arah menurun terdeteksi otomatis.
- `lainnya` pada perulangan berjalan jika tidak ada `hentikan`.
- `setiap`, `sebanyak`, `sampai`, dan `langkah` adalah kata kunci kontekstual.

### G.5 Fungsi

```aksara
fungsi sapa(nama: teks, sapaan: teks = "Halo") -> teks:
    """Mengembalikan sapaan untuk nama."""
    kembalikan f"{sapaan}, {nama}"

fungsi jumlahkan(*angka: bilangan) -> bilangan:
    total = 0
    untuk n dalam angka:
        total += n
    kembalikan total

fungsi buat_konfigurasi(**opsi) -> kamus:
    kembalikan opsi

fungsi bagi(a, /, b, *, bulatkan=salah):
    hasil = a / b
    kembalikan bulat(hasil) jika bulatkan lainnya hasil
```

Closure:

```aksara
fungsi buat_penghitung():
    hitung = 0

    fungsi tambah():
        nonlokal hitung
        hitung += 1
        kembalikan hitung

    kembalikan tambah
```

Lambda dan dekorator:

```aksara
kuadrat = lambda x: x * x

fungsi catat(f):
    fungsi pembungkus(*args, **kwargs):
        cetak("memanggil fungsi")
        kembalikan f(*args, **kwargs)
    kembalikan pembungkus

@catat
fungsi halo():
    cetak("halo")
```

Generator:

```aksara
fungsi hitung_mundur(n: bilangan):
    selama n > 0:
        hasilkan n
        n -= 1

untuk x dalam hitung_mundur(3):
    cetak(x)
```

Aturan:

- Parameter positional-only memakai `/`, keyword-only memakai `*`.
- `kembalikan` tanpa nilai mengembalikan `kosong`.
- `hasilkan` membuat generator. `hasilkan dari` meneruskan iterable lain.
- Batas rekursi default 1000.
- `lambda` dipertahankan sebagai pengecualian pragmatis.

### G.6 Kelas dan OOP

```aksara
kelas Kendaraan:
    jumlah = 0

    fungsi __init__(diri, nama: teks):
        diri.nama = nama
        Kendaraan.jumlah += 1

    fungsi info(diri) -> teks:
        kembalikan diri.nama

    @properti
    fungsi nama_besar(diri) -> teks:
        kembalikan diri.nama.huruf_besar()

    @metode_kelas
    fungsi berapa_banyak(kelas_ini) -> bilangan:
        kembalikan kelas_ini.jumlah

    @metode_statis
    fungsi roda_default() -> bilangan:
        kembalikan 4

kelas Mobil(Kendaraan):
    fungsi __init__(diri, nama: teks, pintu: bilangan):
        induk().__init__(nama)
        diri.pintu = pintu

    fungsi info(diri) -> teks:
        kembalikan f"{diri.nama}, {diri.pintu} pintu"
```

Aturan:

- `diri` adalah nama konvensional untuk instance, bukan kata kunci wajib.
- `induk()` memanggil kelas induk berikutnya dalam MRO.
- Pewarisan ganda didukung dengan C3 linearization.
- Nama metode khusus tetap Python: `__init__`, `__str__`, `__repr__`, `__eq__`.
- Metode Aksara memakai nama Indonesia. Objek Python mempertahankan nama aslinya.

### G.7 Eksepsi dan context manager

```aksara
coba:
    nilai = bilangan("bukan angka")
tangkap KesalahanNilai sebagai e:
    cetak("Gagal:", e)
tangkap (KesalahanTipe, KesalahanKunci):
    cetak("Tipe atau kunci salah")
lainnya:
    cetak("Tidak ada kesalahan")
akhirnya:
    cetak("Selesai")
```

```aksara
lempar KesalahanNilai("pesan")
lempar KesalahanNilai("pesan") dari kesalahan_asli
```

```aksara
dengan buka("data.txt", "r", encoding="utf-8") sebagai berkas:
    untuk baris dalam berkas:
        cetak(baris.pangkas())

dengan buka("masukan.txt") sebagai sumber, buka("keluaran.txt", "w") sebagai tujuan:
    tujuan.tulis(sumber.baca())
```

Aturan:

- `tangkap` tanpa tipe menangkap `Kesalahan`, bukan `DasarKesalahan`.
- `lainnya` berjalan jika tidak ada eksepsi.
- `akhirnya` selalu berjalan.
- Gunakan `dengan` untuk berkas, soket, koneksi, dan kunci.
- `KeluarSistem`, `GangguanKetik`, dan `KeluarGenerator` hanya tertangkap jika disebut eksplisit.

### G.8 Koleksi dan comprehension

```aksara
angka = [1, 2, 3, 4, 5]

kuadrat_genap = [x * x untuk x dalam angka jika x % 2 == 0]
peta_panjang = {nama: panjang(nama) untuk nama dalam ["Ani", "Budi"]}
sisa_unik = {x % 3 untuk x dalam angka}
generator = (x * 2 untuk x dalam angka)

angka.tambah(6)
angka.perluas([7, 8])
angka.urutkan(terbalik=benar)

profil = {"nama": "Ani", "umur": 25}
profil["kota"] = "Bandung"
profil.perbarui({"aktif": benar})
umur = profil.ambil("umur", 0)
```

Aturan:

- Comprehension memakai urutan `untuk` lalu `jika`.
- `daftar`, `kamus`, dan `himpunan` adalah tipe bawaan.
- `tupel` tidak dapat diubah setelah dibuat.
- `rentang` setengah terbuka.
- Gunakan `susunan` untuk data numerik besar.

### G.9 Modul dan paket

```aksara
impor matematika
impor matematika sebagai mat
dari matematika impor akar, pi

dari paket.modul impor fungsi_penting
dari .modul_saudara impor nama
```

```aksara
# paket/__init__.aks
__semua__ = ["modul_a", "modul_b"]
```

Aturan:

- Satu berkas `.aks` adalah satu modul.
- Direktori dengan `__init__.aks` adalah paket.
- Impor relatif memakai awalan titik.
- Impor di-cache per proses.
- `dari modul impor *` dilarang di kode produksi.
- Modul Python dimuat dengan `impor python "nama"` dan `dari python "nama" impor simbol`.

### G.10 Async

```aksara
asinkron fungsi ambil_data(url: teks):
    hasil = tunggu operasi_jaringan(url)
    kembalikan hasil

asinkron fungsi utama():
    data = tunggu ambil_data("https://contoh.id")
    cetak(data)
```

Aturan:

- `asinkron fungsi` mendefinisikan coroutine.
- `tunggu` hanya valid di dalam `asinkron fungsi`.
- `asinkron untuk` dan `asinkron dengan` didukung.
- Satu event loop per thread.
- `asyncio.run` Python tidak boleh dipanggil di dalam loop yang sudah berjalan.

### G.11 Pencocokan pola

```aksara
cocokkan perintah:
    kasus "mulai":
        mulai()
    kasus ["simpan", nama]:
        simpan(nama)
    kasus {"aksi": "hapus", "id": id_}:
        hapus(id_)
    kasus Titik(x=0, y=0):
        cetak("titik asal")
    kasus x jika x > 0:
        cetak("positif")
    kasus _:
        cetak("tidak dikenal")
```

Aturan:

- `kasus` mendukung literal, urutan, pemetaan, kelas, alternatif `|`, penjaga `jika`, dan wildcard `_`.
- Nama yang diikat hanya terlihat di cabang tersebut.
- Urutan kasus diperiksa dari atas ke bawah.
- `_` tidak mengikat nilai.

### G.12 Anotasi tipe

```aksara
fungsi rata_rata(nilai: daftar[pecahan]) -> pecahan:
    kembalikan jumlah(nilai) / panjang(nilai)

variabel: kamus[teks, bilangan] = {}
pilihan: bilangan | kosong = kosong
pasangan: tupel[teks, bilangan] = ("umur", 25)
```

Aturan:

- Anotasi tidak mengubah perilaku runtime.
- Pemeriksaan statis memakai `aksara periksa --tipe`.
- Tipe gabungan memakai `|`.
- Modul `sistem_tipe` menyediakan alias, generik, `Protokol`, `Uni`, dan `Opsional`.

### G.13 Jembatan Python

```aksara
impor python "requests" sebagai requests
dari python "bs4" impor BeautifulSoup
impor python "numpy" sebagai np

jawaban = requests.get("https://contoh.id", timeout=10)
jawaban.raise_for_status()

halaman = BeautifulSoup(jawaban.text, "html.parser")
untuk judul dalam halaman.select("h1"):
    cetak(judul.get_text(strip=benar))

data = np.array([1, 2, 3])
cetak(data.mean())
```

Aturan:

- `impor python` memuat modul melalui CPython.
- Objek Python tetap memakai nama metode Python.
- Tipe primitif dan koleksi dikonversi by value.
- `susunan` dan `numpy.ndarray` memakai buffer protocol tanpa salinan.
- Paket dipasang dengan `aksara python pasang`, bukan otomatis saat impor.
- Mode isolasi diaktifkan dengan `--isolasi-python` atau konfigurasi `aksara.toml`.

### G.14 Alat dan diagnostik

```bash
aksara jalankan program.aks
aksara periksa --tipe program.aks
aksara format .
aksara jelaskan E-NAMA-001
aksara python info
aksara python pasang requests pandas opencv-python
aksara kompatibilitas periksa requests
aksara dokter
```

Aturan:

- Setiap error memiliki kode stabil `E-*`.
- `aksara jelaskan` bekerja offline.
- `--diagnostik=json` menghasilkan output terstruktur untuk editor.
- `--ringkas` menghasilkan satu baris error untuk CI.
- `--jejak` menampilkan traceback Python lengkap.

## Lampiran H: Ringkasan operator

### H.1 Prioritas dari rendah ke tinggi

| Tingkat | Operator | Asosiasi | Contoh |
|---|---|---|---|
| 1 | `lambda` | kanan | `lambda x: x + 1` |
| 2 | `:=` | kanan | `(n := panjang(data))` |
| 3 | ternary `jika ... lainnya ...` | kanan | `a jika c lainnya b` |
| 4 | `??` | kanan | `a ?? b ?? c` |
| 5 | `atau` | kiri | `a atau b` |
| 6 | `dan` | kiri | `a dan b` |
| 7 | `bukan` | kanan | `bukan a` |
| 8 | perbandingan, `dalam`, `adalah`, `antara` | berantai, `antara` tidak dapat dirangkai | `0 < x <= 10` |
| 9 | `\|` | kiri | `a \| b` |
| 10 | `^` | kiri | `a ^ b` |
| 11 | `&` | kiri | `a & b` |
| 12 | `<<`, `>>` | kiri | `a << 2` |
| 13 | `+`, `-` | kiri | `a + b` |
| 14 | `*`, `/`, `//`, `%`, `@` | kiri | `a * b` |
| 15 | `+x`, `-x`, `~x` | kanan | `-x` |
| 16 | `**` | kanan | `2 ** 10` |
| 17 | `tunggu` | kanan | `tunggu f()` |
| 18 | `()`, `[]`, `.`, `?.`, `?[]`, `?()` | kiri | `a?.b[0]()` |

### H.2 Aritmetika

| Operator | Arti | Contoh | Hasil |
|---|---|---|---|
| `+` | tambah, gabung | `2 + 3` | `5` |
| `-` | kurang | `7 - 2` | `5` |
| `*` | kali, ulang | `4 * 3` | `12` |
| `/` | bagi | `7 / 2` | `3.5` |
| `//` | bagi bulat | `7 // 2` | `3` |
| `%` | sisa bagi | `7 % 2` | `1` |
| `**` | pangkat | `2 ** 10` | `1024` |
| `@` | kali matriks | `A @ B` | hasil `__matmul__` |
| `+x` | positif | `+5` | `5` |
| `-x` | negatif | `-5` | `-5` |
| `~x` | komplemen bit | `~5` | `-6` |

Aturan numerik:

- `/` selalu menghasilkan pecahan.
- `//` membulatkan ke negatif tak hingga. `-7 // 2` bernilai `-4`.
- `%` mengikuti tanda pembagi. `-7 % 3` bernilai `2`.
- `**` mengikat lebih kuat daripada uner kiri. `-2 ** 2` bernilai `-4`.
- Pembagian dengan nol melempar `KesalahanPembagianNol`.
- `bagi_sisa(a, b)` mengembalikan hasil bagi dan sisa dalam satu panggilan.
- `pangkat(a, b, m)` menghitung modulo secara efisien.

### H.3 Perbandingan, keanggotaan, dan identitas

| Operator | Arti | Contoh |
|---|---|---|
| `==` | sama nilai | `a == b` |
| `!=` | tidak sama nilai | `a != b` |
| `<`, `<=`, `>`, `>=` | urutan | `a < b` |
| `dalam` | keanggotaan | `x dalam data` |
| `bukan dalam` | bukan keanggotaan | `x bukan dalam data` |
| `adalah` | identitas | `a adalah kosong` |
| `bukan adalah` | bukan identitas | `a bukan adalah kosong` |
| `antara a dan b` | rentang tertutup | `x antara 1 dan 10` |
| `bukan antara a dan b` | di luar rentang | `x bukan antara 1 dan 10` |

Aturan:

- `antara` mengevaluasi ketiga operand satu kali dan setara `a <= x dan x <= b`.
- `antara` tidak dapat dirangkai. Tulis dua perbandingan terpisah dengan `dan`.
- `bukan antara` adalah negasi logis, termasuk untuk `NaN`.
- `adalah` idiomatis untuk `kosong`, `benar`, dan `salah`. Untuk angka dan teks, gunakan `==`.
- `==` pada `NaN` bernilai `salah`. Gunakan `matematika.bukan_angka(x)`.
- Perbandingan biasa dapat dirangkai: `0 < x <= 10`.

### H.4 Logika dan nilai kosong

| Operator | Arti | Contoh | Hasil |
|---|---|---|---|
| `dan` | dan logika | `1 dan 2` | `2` |
| `atau` | atau logika | `0 atau "x"` | `"x"` |
| `bukan` | negasi | `bukan 0` | `benar` |
| `??` | pengganti kosong | `kosong ?? 5` | `5` |
| `??=` | penugasan pengganti kosong | `x ??= 5` | `x` diisi `5` jika belum ada atau `kosong` |
| `?.` | akses atribut aman | `a?.b` | `kosong` jika `a` kosong |
| `?[]` | indeks aman | `a?[0]` | `kosong` jika `a` kosong |
| `?()` | panggilan aman | `a?()` | `kosong` jika `a` kosong |

Aturan:

- `dan` dan `atau` mengembalikan operand, bukan hanya `benar` atau `salah`.
- `bukan` selalu mengembalikan `benar` atau `salah`.
- `??` hanya memeriksa `kosong`. `0 ?? 5` bernilai `0`, `"" ?? "x"` bernilai `""`.
- `atau` memeriksa kebenaran. `0 atau 5` bernilai `5`.
- `??=` mendefinisikan nama yang belum ada, atau mengganti nilai yang `kosong`.
- `?.`, `?[]`, dan `?()` tidak menelan atribut, kunci, atau indeks yang hilang.
- Setiap operator aman memeriksa penerimanya sendiri. `a?.b.c` hanya melindungi `a`.

### H.5 Bitwise dan himpunan

| Operator | Bilangan | `logika` | Himpunan |
|---|---|---|---|
| `&` | DAN bit | DAN | irisan |
| `\|` | ATAU bit | ATAU | gabungan |
| `^` | XOR bit | XOR eksklusif | selisih simetris |
| `~` | NOT bit | tidak berlaku | tidak berlaku |
| `<<` | geser kiri | tidak berlaku | tidak berlaku |
| `>>` | geser kanan | tidak berlaku | tidak berlaku |
| `-` | kurang | tidak berlaku | selisih |

Contoh:

```aksara
0b1010 & 0b1100    # 0b1000
0b1010 | 0b1100    # 0b1110
0b1010 ^ 0b1100    # 0b0110
~5                 # -6
5 << 1             # 10
5 >> 1             # 2

benar & salah      # salah
benar | salah      # benar
benar ^ salah      # benar
benar ^ benar      # salah

{1, 2} & {2, 3}    # {2}
{1, 2} | {2, 3}    # {1, 2, 3}
{1, 2} ^ {2, 3}    # {1, 3}
{1, 2} - {2, 3}    # {1}
```

Aturan:

- Geser dengan jumlah negatif melempar `KesalahanNilai`.
- `~x` setara `-x - 1` untuk bilangan bulat.
- Prioritas bitwise mengikuti Python: `&` lebih tinggi dari `^`, dan `^` lebih tinggi dari `|`.
- Campurkan dengan tanda kurung jika melibatkan `dan`, `atau`, atau perbandingan.

### H.6 Penugasan

| Bentuk | Contoh | Catatan |
|---|---|---|
| `=` | `x = 1` | pernyataan |
| `+=`, `-=`, `*=`, `/=`, `//=`, `%=`, `**=`, `@=` | `x += 1` | target dievaluasi satu kali |
| `&=`, `\|=`, `^=`, `<<=`, `>>=` | `mask \|= bit` | hanya tipe yang mendukung |
| `??=` | `x ??= 10` | jika belum ada atau `kosong` |
| `:=` | `jika (n := panjang(data)) > 0:` | ekspresi |
| `a, b = ...` | `a, b = 1, 2` | jumlah harus cocok |
| `kepala, *ekor = ...` | `kepala, *ekor = data` | satu bintang per target |
| `*args` | `f(*args)` | unpacking posisional |
| `**kwargs` | `f(**opsi)` | unpacking kata kunci |
| `[*a, *b]` | `[*a, *b]` | unpacking daftar |
| `{**a, **b}` | `{**a, **b}` | kunci kanan menang |
| `x: T = v` | `x: bilangan = 5` | anotasi tidak ditegakkan |
| `a = b = 0` | `a = b = 0` | objek yang sama |

Aturan:

- Penugasan gabungan mengevaluasi target satu kali.
- `+=` pada daftar memutasi di tempat. `x = x + [1]` membuat daftar baru.
- Tidak ada `++`, `--`, `===`, `!==`, atau `<>`.
- `:=` mengikat di scope sekitarnya, termasuk di dalam comprehension.
- Penugasan berantai tidak boleh mencampur anotasi.

### H.7 Simbol yang bergantung konteks

| Simbol | Konteks | Arti | Contoh |
|---|---|---|---|
| `@` | awal baris | dekorator | `@properti` |
| `@` | ekspresi | kali matriks | `A @ B` |
| `:` | pernyataan nama | anotasi | `x: bilangan` |
| `:` | di dalam `[]` | slice | `data[1:5]` |
| `:` | akhir header | pembuka blok | `jika x:` |
| `->` | setelah parameter | anotasi kembali | `fungsi f() -> teks:` |
| `...` | ekspresi | elipsis | `fungsi stub(): ...` |
| `;` | akhir pernyataan | pemisah | `a = 1; b = 2` |
| `\` | akhir baris | lanjutan eksplisit | `total = a + \` |
| `\|` | anotasi tipe | tipe gabungan | `bilangan \| kosong` |
| `*` | parameter | keyword-only | `fungsi f(a, *, b)` |
| `/` | parameter | positional-only | `fungsi f(a, /, b)` |
| `**` | parameter | penampung kata kunci | `fungsi f(**opsi)` |
| `?` | bagian `??`, `?.`, `?[]`, `?()` | operator kosong dan aman | `a ?? b` |

### H.8 Kesalahan operator yang sering terjadi

| Ditulis | Masalah | Perbaikan |
|---|---|---|
| `x && y` | bukan operator Aksara | `x dan y` |
| `x \|\| y` | bukan operator Aksara | `x atau y` |
| `!x` | bukan operator Aksara | `bukan x` |
| `x++` | tidak ada increment | `x += 1` |
| `x--` | tidak ada decrement | `x -= 1` |
| `x === y` | tidak ada kesamaan ketat | `x == y` atau `x adalah y` |
| `x !== y` | tidak ada ketidaksamaan ketat | `x != y` atau `x bukan adalah y` |
| `x <> y` | bukan operator Aksara | `x != y` |
| `a ? b : c` | bukan ternary Aksara | `b jika a lainnya c` |
| `jika x = 1:` | penugasan di kondisi | `jika x == 1:` atau `jika (x := 1):` |
| `x antara 1 dan 10 antara 5 dan 15` | `antara` tidak dapat dirangkai | `(x antara 1 dan 10) dan (x antara 5 dan 15)` |
| `a?.b.c` | hanya `a` yang dilindungi | `a?.b?.c` jika keduanya boleh kosong |
| `x == kosong` | `==` tidak menjamin identitas | `x adalah kosong` |
| `x ?? 0` untuk nilai salah | `??` hanya memeriksa `kosong` | `x atau 0` jika semua nilai salah perlu diganti |

### H.9 Tabel kebenaran singkat

| `a` | `b` | `a dan b` | `a atau b` | `a ^ b` | `bukan a` |
|---|---|---|---|---|---|
| `benar` | `benar` | `benar` | `benar` | `salah` | `salah` |
| `benar` | `salah` | `salah` | `benar` | `benar` | `salah` |
| `salah` | `benar` | `salah` | `benar` | `benar` | `benar` |
| `salah` | `salah` | `salah` | `salah` | `salah` | `benar` |

`dan` dan `atau` mengembalikan operand asli. Tabel di atas hanya menunjukkan nilai kebenaran, bukan nilai yang dikembalikan.

### H.10 Aturan evaluasi

- Pada operator biner, operand kiri dievaluasi sebelum operand kanan, kecuali operator short-circuit.
- Pada penugasan, sisi kanan dievaluasi sebelum target.
- `dan`, `atau`, dan `??` melakukan short-circuit.
- `x antara a dan b` mengevaluasi `x`, `a`, dan `b` satu kali.
- Target penugasan gabungan dievaluasi satu kali.
- `a?.b` tidak mengevaluasi `.b` jika `a` adalah `kosong`.
- `??` tidak mengevaluasi sisi kanan jika sisi kiri bukan `kosong`.
- `??=` tidak mengevaluasi sisi kanan jika target sudah berisi nilai bukan `kosong`.
- Urutan evaluasi argumen fungsi adalah kiri ke kanan.
- Urutan evaluasi kunci dan nilai kamus adalah sesuai urutan penulisan.
