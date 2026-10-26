# ADR 0001: Pembagian Front End Rust dan Runtime C

## Status

Diterima.

## Konteks

Aksara membutuhkan parser yang cepat, diagnostik yang kaya, tooling yang mudah dikembangkan, dan runtime yang dapat mengontrol memori serta berinteraksi dengan CPython C API. Tidak ada satu bahasa yang optimal untuk semua bagian.

## Keputusan

- Front end (lexer, parser, AST, semantic checker, bytecode compiler, reference interpreter, CLI, REPL, formatter, linter, LSP, debugger, package manager) ditulis dalam Rust.
- Runtime (bytecode VM, object model, GC, builtins, C ABI, CPython bridge) ditulis dalam C11.
- Batas FFI berupa C ABI yang sempit dan versioned.
- Front end tidak menyentuh pointer runtime secara langsung.

## Konsekuensi

- Parser dan tooling mendapat keamanan memori dan pattern matching Rust.
- Runtime mendapat kontrol layout objek, interop C, dan integrasi CPython yang langsung.
- Dibutuhkan marshal bytecode dan disiplin ABI.
- Uji diferensial antara interpreter referensi Rust dan VM C menjadi wajib.

## Alternatif yang ditolak

- Seluruhnya Rust: interop CPython C API lebih rumit dan layout objek kurang terkontrol.
- Seluruhnya C: tooling dan parser lebih rawan bug serta lebih lambat dikembangkan.
- Seluruhnya Python: tidak memenuhi tujuan bahasa dan runtime sendiri.
