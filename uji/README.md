# Harness Uji

Direktori ini berisi harness yang tidak bergantung pada satu crate.

- `parser/`: korpus, golden test, dan catatan uji parser.
- `benchmark/`: baseline CPython dan benchmark parser.
- `fuzz/`: target fuzz dan prosedur menjalankannya.
- `kepatuhan/`: suite kepatuhan bahasa yang akan diisi pada Fase 2.

## Prinsip

- Setiap uji harus deterministik.
- Uji yang membutuhkan jaringan atau paket Python berada di suite ekosistem, bukan suite inti.
- Setiap kegagalan harus dapat direproduksi dengan satu perintah.
- Golden file disimpan sebagai teks yang dapat dibaca manusia.
