# Baseline Benchmark

## CPython

Baseline diukur dengan `uji/benchmark/cpython_baseline.py` pada mesin referensi. Skrip mencetak JSON dengan versi Python, platform, dan waktu eksekusi untuk 10 program kecil.

```bash
python3 uji/benchmark/cpython_baseline.py > uji/benchmark/cpython_baseline.json
```

Baseline yang tercatat pada Fase 0 memakai Python yang tersedia di mesin pengembangan. Target rilis tetap CPython 3.12. Jika mesin hanya memiliki 3.13, catat versinya dan jalankan ulang pada CI 3.12.

## Parser

```bash
cargo run --release -p aksara-parser --example bench_parse
```

Target Fase 1: berkas 1 MB terparse kurang dari 100 ms pada mesin referensi. Angka aktual dicatat di `uji/benchmark/parser_baseline.json`.
