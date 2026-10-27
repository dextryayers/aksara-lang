# Baseline Benchmark

## CPython

Baseline diukur dengan `uji/benchmark/cpython_baseline.py` pada mesin referensi. Skrip mencetak JSON dengan versi Python, platform, dan waktu eksekusi untuk 10 program kecil.

```bash
python3 uji/benchmark/cpython_baseline.py > uji/benchmark/cpython_baseline.json
```

Baseline utama ada di `cpython_3.12_baseline.json` dan diukur dengan CPython 3.12.16, sesuai target utama proyek. Baseline tambahan `cpython_3.13_baseline.json` diukur dengan CPython 3.13.5 untuk memantau versi yang lebih baru. Jalankan ulang skrip pada mesin referensi sebelum menetapkan angka pembanding final.

## Parser

```bash
cargo run --release -p aksara-parser --example bench_parse -- --bytes 1000000 --iterations 20
```

Target Fase 1: berkas 1 MB terparse kurang dari 100 ms pada mesin referensi. Angka aktual dicatat di `uji/benchmark/parser_baseline.json`.

Baseline saat ini pada mesin pengembangan:

- 1.000.000 byte.
- Median 76,66 ms.
- Minimum 75,81 ms.
- P95 83,25 ms.
- Lolos target 100 ms.

Angka dapat berubah antar mesin. Yang penting adalah median di bawah 100 ms dengan sumber 1 MB dan jumlah iterasi yang tercatat.
