# Perintah pengembangan Aksara.
# Jalankan `just` untuk melihat daftar perintah.

default:
    @just --list

build:
    cargo build --workspace

test:
    cargo test --workspace

fmt:
    cargo fmt --all

fmt-check:
    cargo fmt --all -- --check

lint:
    cargo clippy --workspace --all-targets -- -D warnings

check: fmt-check lint test

runtime:
    cmake -S . -B build
    cmake --build build

runtime-test: runtime
    ctest --test-dir build --output-on-failure

bench:
    cargo run --release -p aksara-parser --example bench_parse

contoh:
    cargo run -p aksara-cli -- periksa contoh/*.aks

golden:
    cargo test -p aksara-parser --test golden

korpus:
    cargo test -p aksara-parser --test korpus

fuzz-smoke:
    cargo test -p aksara-parser --test fuzz_smoke

benchmark:
    cargo run --release -p aksara-parser --example bench_parse -- --bytes 1000000 --iterations 10

bersih:
    cargo clean
    rm -rf build
