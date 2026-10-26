# Spesifikasi Leksikal Aksara

Dokumen ini mendefinisikan bagaimana teks `.aks` diubah menjadi token. Implementasi acuan ada di `crates/aksara-lexer`.

## Encoding dan normalisasi

- Berkas sumber wajib UTF-8.
- BOM UTF-8 di awal berkas diabaikan.
- `\r\n` dan `\r` dinormalisasi menjadi `\n` sebelum lexing.
- Posisi token disimpan sebagai offset byte terhadap sumber yang sudah dinormalisasi.
- Baris dan kolom dihitung 1-based. Kolom dihitung per karakter Unicode, bukan per byte.

## Indentasi

- Satu level indentasi adalah 4 spasi.
- Tab pada awal baris ditolak dengan `E-INDENTASI`.
- Tab di dalam baris juga ditolak karena dapat membuat tampilan sumber ambigu.
- Baris kosong dan baris yang hanya berisi komentar tidak mengubah tumpukan indentasi.
- Naik indentasi menghasilkan satu token `Indent`.
- Turun indentasi menghasilkan satu atau lebih token `Dedent`.
- Jika jumlah spasi tidak cocok dengan level mana pun, lexer menghasilkan `E-INDENTASI` dan memakai level terdekat yang lebih rendah.
- Di akhir berkas, lexer menutup semua `Indent` dengan `Dedent`.

Contoh:

```aksara
jika benar:
    cetak("ya")
cetak("selesai")
```

Token:

```
Jika Benar Colon Newline Indent Name("cetak") LParen Str RParen Newline Dedent Name("cetak") LParen Str RParen Newline Eof
```

## Baris baru dan lanjutan

- `Newline` dihasilkan pada akhir baris logis.
- Di dalam `()`, `[]`, atau `{}`, baris baru dianggap spasi. Tidak ada `Newline` yang dihasilkan.
- Garis miring terbalik `\` di akhir baris melanjutkan baris logis berikutnya.
- `;` adalah pemisah pernyataan dan menghasilkan token `Semi`.
- Komentar dimulai dengan `#` dan berakhir di akhir baris. Komentar tidak menghasilkan token.

## Identifier

- Karakter awal: huruf Unicode atau `_`.
- Karakter lanjutan: huruf Unicode, angka Unicode, atau `_`.
- Identifier bersifat case-sensitive.
- Identifier tidak boleh sama dengan kata kunci yang dicadangkan.
- Kata kunci kontekstual `setiap`, `sebanyak`, `sampai`, dan `langkah` tetap dapat dipakai sebagai identifier di luar posisi sintaksnya.

## Kata kunci

Kata kunci yang dicadangkan:

```
jika lainnya selama untuk dalam fungsi kelas coba tangkap akhirnya lempar
dengan sebagai impor dari global nonlokal hapus pastikan lambda asinkron
tunggu adalah bukan dan atau benar salah kosong cocokkan kasus antara
hasilkan kembalikan hentikan lanjutkan lewati
```

`diri` dan `induk` bukan kata kunci. Keduanya adalah nama biasa. `diri` adalah konvensi untuk instance, dan `induk` adalah fungsi bawaan untuk memanggil kelas induk.

`python` bukan kata kunci. Parser memberinya arti khusus hanya setelah `impor` atau `dari`.

## Literal bilangan

- Bilangan bulat desimal: `0`, `42`, `1_000_000`.
- Heksadesimal: `0xFF`, `0Xff`.
- Oktal: `0o755`.
- Biner: `0b1010`.
- Pecahan: `3.14`, `.5`, `1.`, `1e-9`, `2.5E+3`.
- Imajiner: `2j`, `3.5j`, `1e3j`.
- Garis bawah pemisah diizinkan di antara digit.
- Sufiks `j` atau `J` menandakan bilangan imajiner.
- Angka dengan awalan nol desimal seperti `0123` ditolak dengan `E-ANGKA`.

## Literal teks

Bentuk:

```
"teks"
'teks'
"""teks multi baris"""
'''teks multi baris'''
```

Prefiks:

| Prefiks | Arti |
|---|---|
| `r` | raw, escape tidak diproses |
| `f` | f-string, ekspresi di dalam `{}` dievaluasi |
| `b` | bytes |
| `fr`, `rf` | raw f-string |
| `br`, `rb` | raw bytes |

Aturan:

- Prefix tidak peka huruf besar-kecil.
- `f` dan `b` tidak boleh digabung.
- Escape standar: `\\`, `\'`, `\"`, `\n`, `\r`, `\t`, `\b`, `\f`, `\v`, `\0`, `\xNN`, `\uNNNN`, `\UNNNNNNNN`.
- `\` di akhir baris di dalam string melanjutkan baris.
- String satu baris tidak boleh berisi baris baru mentah.
- String triple boleh berisi baris baru.
- String yang tidak ditutup menghasilkan `E-STRING`.
- F-string dapat berisi `{{` dan `}}` untuk kurung kurawal literal.
- Ekspresi f-string dapat memakai conversion `!s`, `!r`, `!a` dan format spec setelah `:`.
- Format spec dapat berisi ekspresi bersarang di dalam `{}`.

Contoh f-string:

```aksara
nama = "Ani"
umur = 25
cetak(f"Halo, {nama}. Umur: {umur:03d}")
cetak(f"Nilai: {3.14159:.2f}")
cetak(f"Debug: {nama!r}")
```

## Operator dan tanda baca

| Token | Teks |
|---|---|
| `Plus` | `+` |
| `Minus` | `-` |
| `Star` | `*` |
| `Slash` | `/` |
| `SlashSlash` | `//` |
| `Percent` | `%` |
| `StarStar` | `**` |
| `At` | `@` |
| `LShift` | `<<` |
| `RShift` | `>>` |
| `Amp` | `&` |
| `Pipe` | `\|` |
| `Caret` | `^` |
| `Tilde` | `~` |
| `Lt` | `<` |
| `Le` | `<=` |
| `Gt` | `>` |
| `Ge` | `>=` |
| `EqEq` | `==` |
| `NotEq` | `!=` |
| `Eq` | `=` |
| `PlusEq` | `+=` |
| `MinusEq` | `-=` |
| `StarEq` | `*=` |
| `SlashEq` | `/=` |
| `SlashSlashEq` | `//=` |
| `PercentEq` | `%=` |
| `StarStarEq` | `**=` |
| `AtEq` | `@=` |
| `AmpEq` | `&=` |
| `PipeEq` | `\|=` |
| `CaretEq` | `^=` |
| `LShiftEq` | `<<=` |
| `RShiftEq` | `>>=` |
| `ColonEq` | `:=` |
| `Arrow` | `->` |
| `Dot` | `.` |
| `Comma` | `,` |
| `Colon` | `:` |
| `Semi` | `;` |
| `LParen` | `(` |
| `RParen` | `)` |
| `LBracket` | `[` |
| `RBracket` | `]` |
| `LBrace` | `{` |
| `RBrace` | `}` |
| `QuestionQuestion` | `??` |
| `QuestionQuestionEq` | `??=` |
| `QuestionDot` | `?.` |
| `QuestionLBracket` | `?[` |
| `QuestionLParen` | `?(` |
| `Ellipsis` | `...` |

## Kesalahan leksikal

| Kode | Penyebab | Saran |
|---|---|---|
| `E-INDENTASI` | Tab, indentasi tidak kelipatan 4, dedent tidak cocok | Gunakan 4 spasi per level |
| `E-LEKSIKAL` | Karakter tidak dikenal, `?` berdiri sendiri, `!` di luar f-string | Periksa operator atau gunakan `bukan` |
| `E-STRING` | String tidak ditutup, prefix tidak valid | Tutup string atau perbaiki prefix |
| `E-ANGKA` | Angka tidak valid, awalan nol desimal | Hapus awalan nol atau gunakan `0o` |
| `E-FSTRING` | Kurung kurawal tidak seimbang, ekspresi kosong | Perbaiki ekspresi f-string |

## Aturan implementasi

- Lexer tidak pernah panic pada input apa pun.
- Lexer selalu menghasilkan token `Eof` di akhir.
- Lexer menghasilkan diagnostik dan tetap melanjutkan bila memungkinkan.
- Token `Error` dipakai untuk karakter yang tidak dapat dipulihkan.
- F-string disimpan sebagai token string mentah. Parser memecah bagian ekspresinya agar lexer tidak bergantung pada parser.
