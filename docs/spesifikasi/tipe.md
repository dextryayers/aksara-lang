# Spesifikasi Tipe dan Operator

## Model tipe

Aksara memakai tipe dinamis dengan anotasi opsional. Anotasi tidak ditegakkan saat runtime. Pemeriksaan statis dijalankan oleh `aksara periksa --tipe` pada fase lanjutan.

Tipe bawaan:

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

## Kebenaran

Nilai berikut dianggap salah:

- `kosong`
- `salah`
- `0`, `0.0`, `0j`
- `""`, `b""`
- `[]`, `()`, `{}`, himpunan kosong
- `rentang(0)`

Nilai lain dianggap benar. `__bool__` dipanggil lebih dulu, lalu `__len__`. Jika keduanya tidak ada, objek dianggap benar.

## Kesamaan dan hash

- `==` memanggil `__eq__`.
- `!=` memanggil `__ne__`, dengan fallback negasi `__eq__`.
- `adalah` membandingkan identitas objek.
- `hash()` harus konsisten dengan `__eq__`.
- Objek yang tidak hashable tidak dapat menjadi kunci `kamus` atau anggota `himpunan`.

## Operator aritmetika

| Operator | Arti | Catatan |
|---|---|---|
| `+` | tambah, gabung | angka, teks, daftar, tupel |
| `-` | kurang, selisih | angka, himpunan |
| `*` | kali, ulang | angka, teks, daftar, tupel |
| `/` | bagi | selalu menghasilkan pecahan |
| `//` | bagi bulat | membulatkan ke negatif tak hingga |
| `%` | sisa bagi | mengikuti tanda pembagi |
| `**` | pangkat | asosiasi kanan |
| `@` | kali matriks | memanggil `__matmul__` |
| `+x` | positif | |
| `-x` | negatif | |
| `~x` | komplemen bit | setara `-x - 1` untuk bilangan bulat |

## Operator perbandingan

| Operator | Arti |
|---|---|
| `==` | sama nilai |
| `!=` | tidak sama nilai |
| `<`, `<=`, `>`, `>=` | urutan |
| `dalam` | keanggotaan |
| `bukan dalam` | bukan keanggotaan |
| `adalah` | identitas |
| `bukan adalah` | bukan identitas |
| `antara a dan b` | rentang tertutup `a <= x <= b` |
| `bukan antara a dan b` | di luar rentang tertutup |

`antara` mengevaluasi ketiga operand satu kali dan tidak dapat dirangkai. Jika `a > b`, hasilnya `salah`.

## Operator logika dan kosong

| Operator | Arti | Hasil |
|---|---|---|
| `dan` | DAN logika | operand kiri atau kanan |
| `atau` | ATAU logika | operand kiri atau kanan |
| `bukan` | negasi | `benar` atau `salah` |
| `??` | pengganti kosong | kiri jika bukan `kosong`, selain itu kanan |
| `??=` | penugasan pengganti kosong | mengisi jika belum ada atau `kosong` |
| `?.` | akses atribut aman | `kosong` jika penerima `kosong` |
| `?[]` | indeks aman | `kosong` jika penerima `kosong` |
| `?()` | panggilan aman | `kosong` jika penerima `kosong` |

`dan` dan `atau` melakukan short-circuit dan mengembalikan operand asli. `??` hanya memeriksa `kosong`. `?.`, `?[]`, dan `?()` tidak menelan atribut, kunci, atau indeks yang hilang.

## Operator bitwise dan himpunan

| Operator | Bilangan | `logika` | Himpunan |
|---|---|---|---|
| `&` | DAN bit | DAN | irisan |
| `\|` | ATAU bit | ATAU | gabungan |
| `^` | XOR bit | XOR eksklusif | selisih simetris |
| `~` | NOT bit | tidak berlaku | tidak berlaku |
| `<<` | geser kiri | tidak berlaku | tidak berlaku |
| `>>` | geser kanan | tidak berlaku | tidak berlaku |
| `-` | kurang | tidak berlaku | selisih |

Untuk `logika`, `&`, `|`, dan `^` selalu mengembalikan `benar` atau `salah`. `dan` dan `atau` tetap mengembalikan operand.

## Prioritas operator

Dari rendah ke tinggi:

1. `lambda`
2. `:=`
3. ternary `jika ... lainnya ...`
4. `??`
5. `atau`
6. `dan`
7. `bukan`
8. perbandingan, `dalam`, `adalah`, `antara`
9. `|`
10. `^`
11. `&`
12. `<<`, `>>`
13. `+`, `-`
14. `*`, `/`, `//`, `%`, `@`
15. `+x`, `-x`, `~x`
16. `**`
17. `tunggu`
18. `()`, `[]`, `.`, `?.`, `?[]`, `?()`

## Protokol objek

- `__init__`, `__str__`, `__repr__`, `__eq__`, `__hash__`, `__len__`, `__bool__`
- `__iter__`, `__next__`, `__getitem__`, `__setitem__`, `__contains__`
- `__enter__`, `__exit__`
- `__add__`, `__sub__`, `__mul__`, `__truediv__`, `__floordiv__`, `__mod__`, `__pow__`, `__matmul__`
- `__and__`, `__or__`, `__xor__`, `__lshift__`, `__rshift__`
- `__await__`, `__aiter__`, `__anext__`, `__aenter__`, `__aexit__`

Nama metode khusus tetap memakai bahasa Python agar jembatan CPython dan protokol objek tetap kompatibel.

## Konversi

| Fungsi | Hasil |
|---|---|
| `bilangan(x)` | bilangan bulat |
| `pecahan(x)` | pecahan |
| `kompleks(x)` | kompleks |
| `logika(x)` | logika |
| `teks(x)` | teks |
| `bytes(x)` | bytes |
| `daftar(x)` | daftar |
| `tupel(x)` | tupel |
| `kamus(x)` | kamus |
| `himpunan(x)` | himpunan |
| `susunan(x)` | array numerik |

Konversi yang gagal melempar `KesalahanNilai` atau `KesalahanTipe`.

## Aturan evaluasi

- Operand kiri dievaluasi sebelum operand kanan, kecuali operator short-circuit.
- Pada penugasan, sisi kanan dievaluasi sebelum target.
- `dan`, `atau`, dan `??` melakukan short-circuit.
- `antara` mengevaluasi nilai dan kedua batas satu kali.
- Target penugasan gabungan dievaluasi satu kali.
- Urutan evaluasi argumen fungsi adalah kiri ke kanan.
- Urutan evaluasi kunci dan nilai kamus adalah sesuai urutan penulisan.
