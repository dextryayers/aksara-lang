#!/usr/bin/env python3
"""Baseline CPython untuk 10 program kecil.

Skrip ini sengaja tidak memakai dependensi eksternal. Hasilnya JSON agar dapat
dibandingkan antar mesin dan antar versi Python.
"""

from __future__ import annotations

import json
import platform
import statistics
import sys
import time
from typing import Callable

PROGRAM: list[tuple[str, Callable[[], None]]] = []


def program(nama: str):
    def dekorator(fn: Callable[[], None]) -> Callable[[], None]:
        PROGRAM.append((nama, fn))
        return fn

    return dekorator


@program("loop_1000")
def loop_1000() -> None:
    total = 0
    for i in range(1000):
        total += i


@program("fib_20")
def fib_20() -> None:
    def fib(n: int) -> int:
        if n < 2:
            return n
        return fib(n - 1) + fib(n - 2)

    fib(20)


@program("list_sort")
def list_sort() -> None:
    data = list(range(1000, 0, -1))
    data.sort()


@program("dict_build")
def dict_build() -> None:
    data = {i: i * i for i in range(1000)}


@program("string_join")
def string_join() -> None:
    "".join(str(i) for i in range(1000))


@program("json_roundtrip")
def json_roundtrip() -> None:
    data = {"nilai": list(range(100))}
    json.loads(json.dumps(data))


@program("class_create")
def class_create() -> None:
    class Titik:
        def __init__(self, x: int, y: int) -> None:
            self.x = x
            self.y = y

    for i in range(1000):
        Titik(i, i + 1)


@program("exception")
def exception() -> None:
    for _ in range(1000):
        try:
            raise ValueError("uji")
        except ValueError:
            pass


@program("generator")
def generator() -> None:
    def angka():
        for i in range(1000):
            yield i

    sum(angka())


@program("regex")
def regex() -> None:
    import re

    pola = re.compile(r"[a-z]+")
    for _ in range(1000):
        pola.findall("halo dunia aksara")


def main() -> int:
    hasil = {
        "python": sys.version.split()[0],
        "implementasi": platform.python_implementation(),
        "platform": platform.platform(),
        "program": {},
    }
    for nama, fn in PROGRAM:
        sampel = []
        for _ in range(5):
            mulai = time.perf_counter()
            fn()
            sampel.append((time.perf_counter() - mulai) * 1000.0)
        hasil["program"][nama] = {
            "median_ms": round(statistics.median(sampel), 4),
            "min_ms": round(min(sampel), 4),
        }
    json.dump(hasil, sys.stdout, indent=2)
    sys.stdout.write("\n")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
