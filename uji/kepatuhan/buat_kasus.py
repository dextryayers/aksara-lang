#!/usr/bin/env python3
"""Membangun suite kepatuhan Aksara Fase 2.

Setiap kasus adalah program `.aks` mandiri yang memakai `pastikan` untuk
memeriksa semantik dan mencetak `OK` bila seluruh pemeriksaan lolos. Berkas
`.keluaran` menyimpan keluaran standar yang diharapkan. Kasus galat memiliki
`.kode` dan `.galat` untuk kode keluar serta potongan pesan kesalahan.

Jalankan ulang skrip ini setelah menambah kasus:

    python3 uji/kepatuhan/buat_kasus.py
"""

from pathlib import Path
import shutil

ROOT = Path(__file__).resolve().parent
KASUS = ROOT / "kasus"


def bersihkan() -> None:
    if KASUS.exists():
        shutil.rmtree(KASUS)
    for bagian in ("bahasa", "pustaka", "jembatan", "galat"):
        (KASUS / bagian).mkdir(parents=True)


def tulis(bagian: str, nama: str, isi: str, keluaran: str = "OK\n") -> None:
    jalur = KASUS / bagian / f"{nama}.aks"
    jalur.parent.mkdir(parents=True, exist_ok=True)
    sumber = isi.strip("\n") + "\n"
    if "cetak(\"OK\")" not in sumber:
        sumber += '\ncetak("OK")\n'
    jalur.write_text(sumber, encoding="utf-8")
    jalur.with_suffix(".keluaran").write_text(keluaran, encoding="utf-8")


def galat(nama: str, isi: str, kode: int, potongan: str) -> None:
    jalur = KASUS / "galat" / f"{nama}.aks"
    jalur.write_text(isi.strip("\n") + "\n", encoding="utf-8")
    jalur.with_suffix(".kode").write_text(f"{kode}\n", encoding="utf-8")
    jalur.with_suffix(".galat").write_text(potongan + "\n", encoding="utf-8")


def _pesan(teks: str) -> str:
    return (
        teks.replace("\\", "\\\\")
        .replace('"', '\\"')
        .replace("\n", "\\n")
        .replace("\t", "\\t")
    )


def cek(ekspr: str, harap: str, pesan: str | None = None) -> str:
    pesan = _pesan(pesan or ekspr)
    return f'pastikan ({ekspr}) == ({harap}), "{pesan}"'


def cek_benar(ekspr: str, pesan: str | None = None) -> str:
    pesan = _pesan(pesan or ekspr)
    return f'pastikan ({ekspr}), "{pesan}"'


def cek_dekat(ekspr: str, harap: str, pesan: str | None = None) -> str:
    pesan = _pesan(pesan or ekspr)
    return f'pastikan abs(({ekspr}) - ({harap})) < 0.000001, "{pesan}"'


def kumpulan(judul: str, baris: list[str]) -> str:
    return f'# {judul}\n\n' + "\n".join(baris) + "\n"


# ------------------------------------------------------------------
# Bahasa: literal dan tipe
# ------------------------------------------------------------------

def literal() -> None:
    tulis("bahasa", "001_literal_bilangan", kumpulan("Literal bilangan", [
        cek("42", "42"),
        cek("-17", "-17"),
        cek("0xFF", "255"),
        cek("0b1010", "10"),
        cek("0o755", "493"),
        cek("1_000_000", "1000000"),
        cek("jenis(42).__nama__", '"Bilangan"'),
    ]))
    tulis("bahasa", "002_literal_pecahan", kumpulan("Literal pecahan", [
        cek("3.14", "3.14"),
        cek("1e3", "1000.0"),
        cek("2.5e-2", "0.025"),
        cek("jenis(1.0).__nama__", '"Pecahan"'),
        cek_benar("1.0 == 1"),
    ]))
    tulis("bahasa", "003_literal_kompleks", kumpulan("Literal kompleks", [
        cek("2j", "2j"),
        cek("kompleks(1, 2)", "1 + 2j"),
        cek_benar("abs(3 + 4j) == 5.0"),
    ]))
    tulis("bahasa", "004_literal_logika_kosong", kumpulan("Logika dan kosong", [
        cek("benar", "benar"),
        cek("salah", "salah"),
        cek("kosong", "kosong"),
        cek("jenis(benar).__nama__", '"Logika"'),
        cek("jenis(kosong).__nama__", '"Kosong"'),
    ]))
    tulis("bahasa", "005_literal_teks", kumpulan("Literal teks", [
        cek('"halo"', '"halo"'),
        cek("'halo'", '"halo"'),
        cek('"a" + "b"', '"ab"'),
        cek('"ha" * 3', '"hahaha"'),
        cek('panjang("halo")', "4"),
        cek('"halo"[0]', '"h"'),
        cek('"halo"[-1]', '"o"'),
    ]))
    tulis("bahasa", "006_teks_escape", kumpulan("Escape teks", [
        cek('"a\\nb"', '"a\\nb"'),
        cek('"a\\tb"', '"a\\tb"'),
        cek('"a\\\\b"', '"a\\\\b"'),
        cek('"\\u00e9"', '"é"'),
        cek('panjang("\\u00e9")', "1"),
    ]))
    tulis("bahasa", "007_teks_mentah", kumpulan("Teks mentah", [
        cek('r"folder\\berkas"', '"folder\\\\berkas"'),
        cek('r"a\\nb"', '"a\\\\nb"'),
        cek('panjang(r"\\n")', "2"),
    ]))
    tulis("bahasa", "008_teks_multibaris", kumpulan("Teks multibaris", [
        'teks = """baris satu\nbaris dua"""',
        cek("panjang(teks.pisah_baris())", "2"),
        cek('teks.pisah_baris()[1]', '"baris dua"'),
    ]))
    tulis("bahasa", "009_bytes", kumpulan("Bytes", [
        cek('b"abc"', "b'abc'"),
        cek('panjang(b"abc")', "3"),
        cek('b"abc"[1]', "98"),
        cek('b"abc".decode()', '"abc"'),
        cek('panjang("hé".encode())', "3"),
        cek('"hé".encode().decode()', '"hé"'),
    ]))
    tulis("bahasa", "010_konversi_tipe", kumpulan("Konversi tipe", [
        cek('bilangan("42")', "42"),
        cek('bilangan("ff", 16)', "255"),
        cek('bilangan(3.9)', "3"),
        cek('pecahan("2.5")', "2.5"),
        cek('teks(42)', '"42"'),
        cek('logika(0)', "salah"),
        cek('logika(1)', "benar"),
        cek('logika("")', "salah"),
        cek('logika("x")', "benar"),
    ]))
    tulis("bahasa", "011_jenis_nilai", kumpulan("Jenis nilai", [
        cek("jenis([]).__nama__", '"Daftar"'),
        cek("jenis(()).__nama__", '"Tupel"'),
        cek("jenis({}).__nama__", '"Kamus"'),
        cek("jenis(susunan()).__nama__", '"Himpunan"'),
        cek("jenis(rentang(3)).__nama__", '"Rentang"'),
        cek("jenis(lambda x: x).__nama__", '"Fungsi"'),
    ]))
    tulis("bahasa", "012_identitas_dan_kesamaan", kumpulan("Identitas dan kesamaan", [
        cek_benar("1 == 1"),
        cek_benar("1 == 1.0"),
        cek_benar("1 != 2"),
        cek_benar('"a" == "a"'),
        cek_benar("[1, 2] == [1, 2]"),
        cek_benar("{1: 2} == {1: 2}"),
        cek_benar("(1, 2) == (1, 2)"),
    ]))
    tulis("bahasa", "013_span_dan_posisi", kumpulan("Posisi sumber tersedia", [
        "nilai = 10",
        cek("nilai", "10"),
    ]))


# ------------------------------------------------------------------
# Bahasa: aritmetika dan operator
# ------------------------------------------------------------------

def aritmetika() -> None:
    tulis("bahasa", "020_aritmetika_dasar", kumpulan("Aritmetika dasar", [
        cek("2 + 3", "5"),
        cek("7 - 10", "-3"),
        cek("4 * 6", "24"),
        cek("7 / 2", "3.5"),
        cek("7 // 2", "3"),
        cek("7 % 2", "1"),
        cek("2 ** 8", "256"),
        cek("-2 ** 2", "-4"),
        cek("(-2) ** 2", "4"),
    ]))
    tulis("bahasa", "021_aritmetika_negatif", kumpulan("Aritmetika negatif", [
        cek("-7 // 2", "-4"),
        cek("7 // -2", "-4"),
        cek("-7 % 3", "2"),
        cek("7 % -3", "-2"),
        cek("-7 / 2", "-3.5"),
    ]))
    tulis("bahasa", "022_prioritas", kumpulan("Prioritas operator", [
        cek("2 + 3 * 4", "14"),
        cek("(2 + 3) * 4", "20"),
        cek("2 ** 3 ** 2", "512"),
        cek("10 - 2 - 3", "5"),
        cek("100 / 10 / 2", "5.0"),
        cek("1 + 2 * 3 ** 2", "19"),
    ]))
    tulis("bahasa", "023_campuran_numerik", kumpulan("Campuran numerik", [
        cek("1 + 2.5", "3.5"),
        cek("2 * 3.0", "6.0"),
        cek("7 // 2.0", "3.0"),
        cek("7 % 2.5", "2.0"),
        cek("2 ** 0.5", "1.4142135623730951"),
        cek("3 + 2j", "3 + 2j"),
    ]))
    tulis("bahasa", "024_bitwise", kumpulan("Operator bitwise", [
        cek("6 & 3", "2"),
        cek("6 | 3", "7"),
        cek("6 ^ 3", "5"),
        cek("~5", "-6"),
        cek("1 << 4", "16"),
        cek("32 >> 3", "4"),
        cek("benar & salah", "salah"),
        cek("benar | salah", "benar"),
        cek("benar ^ benar", "salah"),
    ]))
    tulis("bahasa", "025_pembagian_nol", kumpulan("Pembagian nol", [
        "coba:",
        "    x = 1 / 0",
        "tangkap KesalahanNolPembagian:",
        '    pastikan benar, "pembagian bulat"',
        "coba:",
        "    x = 1 // 0",
        "tangkap KesalahanNolPembagian:",
        '    pastikan benar, "pembagian lantai"',
        "coba:",
        "    x = 1 % 0",
        "tangkap KesalahanNolPembagian:",
        '    pastikan benar, "modulo"',
    ]))
    tulis("bahasa", "026_aritmetika_besar", kumpulan("Bilangan besar", [
        cek("2 ** 62", "4611686018427387904"),
        cek("(2 ** 40) // 1024", "1073741824"),
    ]))
    tulis("bahasa", "027_operator_augmented", kumpulan("Operator augmented", [
        "x = 5",
        "x += 3",
        "x -= 1",
        "x *= 2",
        "x //= 3",
        "x %= 4",
        "x **= 2",
        cek("x", "0"),
        "s = \"a\"",
        "s += \"b\"",
        cek("s", '"ab"'),
    ]))
    tulis("bahasa", "028_operator_walrus", kumpulan("Operator walrus", [
        "jika (n := 10) > 5:",
        "    hasil = n * 2",
        cek("hasil", "20"),
        cek("n", "10"),
    ]))
    tulis("bahasa", "029_pembulatan", kumpulan("Pembulatan", [
        cek("bulat(2.4)", "2"),
        cek("bulat(2.6)", "3"),
        cek("bulat(2.5)", "2"),
        cek("bulat(3.5)", "4"),
        cek("bulat(-2.5)", "-2"),
        cek("bulat(2.675, 2)", "2.68"),
        cek("abs(-7)", "7"),
        cek("abs(-2.5)", "2.5"),
    ]))
    tulis("bahasa", "030_divmod_dan_pow", kumpulan("divmod dan pow", [
        cek("divmod(7, 3)", "(2, 1)"),
        cek("divmod(-7, 3)", "(-3, 2)"),
        cek("pow(2, 10)", "1024"),
        cek("pow(2, 10, 1000)", "24"),
        cek("pow(2, -1)", "0.5"),
    ]))


# ------------------------------------------------------------------
# Bahasa: perbandingan, logika, operator kosong
# ------------------------------------------------------------------

def perbandingan() -> None:
    tulis("bahasa", "040_perbandingan", kumpulan("Perbandingan", [
        cek_benar("1 < 2"),
        cek_benar("2 <= 2"),
        cek_benar("3 > 2"),
        cek_benar("bukan (3 >= 4)"),
        cek_benar('"a" < "b"'),
        cek_benar("[1, 2] < [1, 3]"),
        cek_benar("(1, 2) < (1, 3)"),
    ]))
    tulis("bahasa", "041_perbandingan_berantai", kumpulan("Perbandingan berantai", [
        cek_benar("1 < 2 < 3"),
        cek_benar("1 < 2 > 0"),
        cek_benar("bukan (1 < 3 < 2)"),
        cek_benar("0 <= 0 <= 0"),
    ]))
    tulis("bahasa", "042_keanggotaan", kumpulan("Keanggotaan", [
        cek_benar("2 dalam [1, 2, 3]"),
        cek_benar("4 bukan dalam [1, 2, 3]"),
        cek_benar('"a" dalam "halo"'),
        cek_benar('"z" bukan dalam "halo"'),
        cek_benar('"a" dalam {"a": 1}'),
        cek_benar("2 dalam {1, 2, 3}"),
        cek_benar("3 dalam rentang(5)"),
    ]))
    tulis("bahasa", "043_identitas", kumpulan("Operator identitas", [
        cek_benar("kosong adalah kosong"),
        cek_benar("1 bukan adalah 2"),
        "a = [1]",
        "b = a",
        "c = [1]",
        cek_benar("a adalah b"),
        cek_benar("a bukan adalah c"),
        cek_benar("a == c"),
    ]))
    tulis("bahasa", "044_logika_dan_atau", kumpulan("Logika dan atau", [
        cek('0 atau "nilai"', '"nilai"'),
        cek('"nilai" atau 0', '"nilai"'),
        cek('1 dan 2', "2"),
        cek('0 dan 2', "0"),
        cek('kosong atau 5', "5"),
        cek('[] atau [1]', "[1]"),
        cek_benar("bukan salah"),
        cek_benar("bukan (1 == 2)"),
    ]))
    tulis("bahasa", "045_short_circuit", kumpulan("Short circuit", [
        "dipanggil = []",
        "fungsi catat(n):",
        "    dipanggil.tambah(n)",
        "    kembalikan benar",
        "hasil = salah dan catat(1)",
        cek("hasil", "salah"),
        cek("dipanggil", "[]"),
        "hasil = benar atau catat(2)",
        cek("hasil", "benar"),
        cek("dipanggil", "[]"),
    ]))
    tulis("bahasa", "046_truthiness", kumpulan("Truthiness", [
        cek("logika(kosong)", "salah"),
        cek("logika(0)", "salah"),
        cek("logika(0.0)", "salah"),
        cek('logika("")', "salah"),
        cek("logika([])", "salah"),
        cek("logika(())", "salah"),
        cek("logika({})", "salah"),
        cek("logika(susunan())", "salah"),
        cek("logika(rentang(0))", "salah"),
        cek("logika([0])", "benar"),
        cek("logika(rentang(1))", "benar"),
    ]))
    tulis("bahasa", "047_operator_kosong", kumpulan("Operator kosong", [
        cek("kosong ?? 10", "10"),
        cek("0 ?? 10", "0"),
        cek("salah ?? 10", "salah"),
        cek('"x" ?? "y"', '"x"'),
        "nilai = kosong",
        "nilai ??= 7",
        cek("nilai", "7"),
    ]))
    tulis("bahasa", "048_operator_aman", kumpulan("Operator aman", [
        "objek = kosong",
        cek("objek?.nama", "kosong"),
        cek("objek?[0]", "kosong"),
        cek("objek?()", "kosong"),
        cek('objek?.nama ?? "Tamu"', '"Tamu"'),
        "kelas Orang:",
        "    fungsi __init__(diri):",
        "        diri.nama = \"Ani\"",
        "orang = Orang()",
        cek('orang?.nama', '"Ani"'),
    ]))
    tulis("bahasa", "049_antara", kumpulan("Operator antara", [
        cek_benar("5 antara 1 dan 10"),
        cek_benar("1 antara 1 dan 10"),
        cek_benar("10 antara 1 dan 10"),
        cek_benar("bukan (0 antara 1 dan 10)"),
        cek_benar("0 bukan antara 1 dan 10"),
        cek_benar('"b" antara "a" dan "c"'),
    ]))
    tulis("bahasa", "050_ternary", kumpulan("Ternary", [
        cek('"ya" jika benar lainnya "tidak"', '"ya"'),
        cek('"ya" jika salah lainnya "tidak"', '"tidak"'),
        cek("1 jika 0 lainnya 2", "2"),
        cek("(1 jika salah lainnya 2) + 3", "5"),
    ]))


# ------------------------------------------------------------------
# Bahasa: teks dan format
# ------------------------------------------------------------------

def teks() -> None:
    tulis("bahasa", "060_metode_teks_dasar", kumpulan("Metode teks dasar", [
        cek('"  halo  ".strip()', '"halo"'),
        cek('"  halo  ".lstrip()', '"halo  "'),
        cek('"  halo  ".rstrip()', '"  halo"'),
        cek('"halo".huruf_besar()', '"HALO"'),
        cek('"HALO".huruf_kecil()', '"halo"'),
        cek('"halo dunia".judul()', '"Halo Dunia"'),
        cek('"halo dunia".kapital()', '"Halo dunia"'),
        cek('"AbC".swapcase()', '"aBc"'),
        cek('"AbC".casefold()', '"abc"'),
    ]))
    tulis("bahasa", "061_metode_teks_pencarian", kumpulan("Pencarian teks", [
        cek('"halo".find("l")', "2"),
        cek('"halo".rfind("l")', "2"),
        cek('"halo".count("l")', "1"),
        cek('"halo".startswith("ha")', "benar"),
        cek('"halo".endswith("lo")', "benar"),
        cek('"halo".startswith("lo")', "salah"),
        cek('"halo".indeks("a")', "1"),
        cek('"halo".cari("z")', "-1"),
    ]))
    tulis("bahasa", "062_metode_teks_pisah", kumpulan("Pemisahan teks", [
        cek('"a,b,c".split(",")', "['a', 'b', 'c']"),
        cek('"a b  c".split()', "['a', 'b', 'c']"),
        cek('"a,b,c".rsplit(",", 1)', "['a,b', 'c']"),
        cek('"-".join(["a", "b"])', '"a-b"'),
        cek('"a=b".partition("=")', "('a', '=', 'b')"),
        cek('"a=b=c".rpartition("=")', "('a=b', '=', 'c')"),
        cek('"a\\nb".splitlines()', "['a', 'b']"),
    ]))
    tulis("bahasa", "063_metode_teks_ubah", kumpulan("Perubahan teks", [
        cek('"aaa".replace("a", "b")', '"bbb"'),
        cek('"aaa".replace("a", "b", 2)', '"bba"'),
        cek('"abcdef".removeprefix("abc")', '"def"'),
        cek('"abcdef".removesuffix("def")', '"abc"'),
        cek('"a\\tb".expandtabs(4)', '"a   b"'),
        cek('"x".center(5)', '"  x  "'),
        cek('"x".ljust(3)', '"x  "'),
        cek('"x".rjust(3)', '"  x"'),
        cek('"7".zfill(3)', '"007"'),
    ]))
    tulis("bahasa", "064_metode_teks_predikat", kumpulan("Predikat teks", [
        cek('"abc".adalah_alfabet()', "benar"),
        cek('"abc1".adalah_alfanumerik()', "benar"),
        cek('"123".adalah_angka()', "benar"),
        cek('"   ".adalah_spasi()', "benar"),
        cek('"abc".adalah_huruf_kecil()', "benar"),
        cek('"ABC".adalah_huruf_besar()', "benar"),
        cek('"Halo Dunia".adalah_judul()', "benar"),
        cek('"123".adalah_desimal()', "benar"),
        cek('"abc".adalah_identifier()', "benar"),
        cek('"a b".adalah_identifier()', "salah"),
        cek('"halo".adalah_cetak()', "benar"),
    ]))
    tulis("bahasa", "065_fstring", kumpulan("F-string", [
        'nama = "Ani"',
        "umur = 25",
        cek('f"Halo, {nama}!"', '"Halo, Ani!"'),
        cek('f"{umur + 1}"', '"26"'),
        cek('f"{umur:03d}"', '"025"'),
        cek('f"{3.14159:.2f}"', '"3.14"'),
        cek('f"{255:x}"', '"ff"'),
        cek('f"{255:#x}"', '"0xff"'),
        cek('f"{nama!r}"', '"\'Ani\'"'),
        cek('f"{{literal}}"', '"{literal}"'),
    ]))
    tulis("bahasa", "066_format", kumpulan("Fungsi format", [
        cek('format(42, "05d")', '"00042"'),
        cek('format(3.14159, ".2f")', '"3.14"'),
        cek('format("x", ">5")', '"    x"'),
        cek('format("x", "<5")', '"x    "'),
        cek('format("x", "^5")', '"  x  "'),
        cek('format(255, "x")', '"ff"'),
        cek('format(255, "b")', '"11111111"'),
        cek('format(255, "o")', '"377"'),
    ]))
    tulis("bahasa", "067_unicode", kumpulan("Unicode", [
        cek('ord("A")', "65"),
        cek('chr(65)', '"A"'),
        cek('ord("é")', "233"),
        cek('chr(233)', '"é"'),
        cek('panjang("日本語")', "3"),
        cek('"日本語"[1]', '"本"'),
        cek('"aé日".encode().decode()', '"aé日"'),
    ]))
    tulis("bahasa", "068_representasi_teks", kumpulan("Representasi teks", [
        cek('repr("a")', '"\'a\'"'),
        cek('repr("a\\nb")', '"\'a\\\\nb\'"'),
        cek('repr("it\'s")', '"\\"it\'s\\""'),
        cek('teks(42)', '"42"'),
        cek('teks(3.5)', '"3.5"'),
        cek('teks(benar)', '"benar"'),
        cek('teks(kosong)', '"kosong"'),
    ]))
    tulis("bahasa", "069_teks_gabung_implisit", kumpulan("Penggabungan teks", [
        'nilai = "a" "b" "c"',
        cek("nilai", '"abc"'),
        cek('"x" + "y" + "z"', '"xyz"'),
    ]))
    tulis("bahasa", "070_slice_teks", kumpulan("Slice teks", [
        cek('"abcdef"[1:4]', '"bcd"'),
        cek('"abcdef"[:3]', '"abc"'),
        cek('"abcdef"[3:]', '"def"'),
        cek('"abcdef"[::2]', '"ace"'),
        cek('"abcdef"[::-1]', '"fedcba"'),
        cek('"abcdef"[-2:]', '"ef"'),
        cek('"abcdef"[10:]', '""'),
    ]))
    tulis("bahasa", "071_teks_bilangan", kumpulan("Konversi teks bilangan", [
        cek('hex(255)', '"0xff"'),
        cek('oct(8)', '"0o10"'),
        cek('bin(5)', '"0b101"'),
        cek('bilangan("101", 2)', "5"),
        cek('bilangan("0xff", 0)', "255"),
        cek('str(3.0)', '"3.0"'),
    ]))
    tulis("bahasa", "072_teks_kesalahan_indeks", kumpulan("Indeks teks", [
        "coba:",
        '    x = "abc"[10]',
        "tangkap KesalahanIndeks:",
        '    pastikan benar, "indeks teks"',
    ]))


# ------------------------------------------------------------------
# Bahasa: koleksi
# ------------------------------------------------------------------

def koleksi() -> None:
    tulis("bahasa", "080_daftar_dasar", kumpulan("Daftar dasar", [
        cek("[1, 2, 3]", "[1, 2, 3]"),
        cek("[]", "[]"),
        cek("panjang([1, 2, 3])", "3"),
        cek("[10, 20, 30][0]", "10"),
        cek("[10, 20, 30][-1]", "30"),
        cek("[10, 20, 30][1:3]", "[20, 30]"),
        cek("[10, 20, 30][::-1]", "[30, 20, 10]"),
    ]))
    tulis("bahasa", "081_daftar_metode", kumpulan("Metode daftar", [
        "d = [3, 1, 2]",
        "d.tambah(4)",
        cek("d", "[3, 1, 2, 4]"),
        "d.sisip(0, 0)",
        cek("d", "[0, 3, 1, 2, 4]"),
        "d.remove(3)",
        cek("d", "[0, 1, 2, 4]"),
        cek("d.ambil()", "4"),
        cek("d", "[0, 1, 2]"),
        "d.perluas([5, 6])",
        cek("d", "[0, 1, 2, 5, 6]"),
        "d.urutkan()",
        cek("d", "[0, 1, 2, 5, 6]"),
        "d.balik()",
        cek("d", "[6, 5, 2, 1, 0]"),
        cek("d.hitung(2)", "1"),
        cek("d.indeks(5)", "1"),
        "s = d.salin()",
        "s.tambah(99)",
        cek("d", "[6, 5, 2, 1, 0]"),
        "d.bersihkan()",
        cek("d", "[]"),
    ]))
    tulis("bahasa", "082_daftar_urut", kumpulan("Pengurutan daftar", [
        "d = [3, 1, 2]",
        "d.urutkan(reverse=benar)",
        cek("d", "[3, 2, 1]"),
        "d.urutkan()",
        cek("d", "[1, 2, 3]"),
        cek("sorted([3, 1, 2])", "[1, 2, 3]"),
        cek("sorted([3, 1, 2], reverse=benar)", "[3, 2, 1]"),
        cek('sorted(["bb", "a", "ccc"], key=lambda x: panjang(x))', "['a', 'bb', 'ccc']"),
        cek("daftar(reversed([1, 2, 3]))", "[3, 2, 1]"),
    ]))
    tulis("bahasa", "083_tupel", kumpulan("Tupel", [
        cek("(1, 2, 3)", "(1, 2, 3)"),
        cek("(1,)", "(1,)"),
        cek("()", "()"),
        cek("panjang((1, 2))", "2"),
        cek("(1, 2)[1]", "2"),
        cek("(1, 2) + (3,)", "(1, 2, 3)"),
        cek("(1, 2) * 2", "(1, 2, 1, 2)"),
        cek("2 dalam (1, 2)", "benar"),
        cek("tupel([1, 2])", "(1, 2)"),
    ]))
    tulis("bahasa", "084_unpacking", kumpulan("Unpacking", [
        "a, b = 1, 2",
        cek("(a, b)", "(1, 2)"),
        "a, b = b, a",
        cek("(a, b)", "(2, 1)"),
        "kepala, *ekor = [1, 2, 3, 4]",
        cek("kepala", "1"),
        cek("ekor", "[2, 3, 4]"),
        "*awal, akhir = [1, 2, 3]",
        cek("awal", "[1, 2]"),
        cek("akhir", "3"),
        "gabung = [0, *[1, 2], 3]",
        cek("gabung", "[0, 1, 2, 3]"),
        "kamus_gabung = {**{\"a\": 1}, **{\"b\": 2}}",
        cek('kamus_gabung["b"]', "2"),
    ]))
    tulis("bahasa", "085_kamus", kumpulan("Kamus", [
        "d = {\"a\": 1, \"b\": 2}",
        cek("d", "{'a': 1, 'b': 2}"),
        cek('d["a"]', "1"),
        cek("panjang(d)", "2"),
        cek('"a" dalam d', "benar"),
        "d[\"c\"] = 3",
        cek('d["c"]', "3"),
        cek('d.get("a")', "1"),
        cek('d.get("z", 99)', "99"),
        cek('d.setdefault("d", 4)', "4"),
        cek('d["d"]', "4"),
        cek("d.keys()", "['a', 'b', 'c', 'd']"),
        cek("d.values()", "[1, 2, 3, 4]"),
        cek("d.items()", "[('a', 1), ('b', 2), ('c', 3), ('d', 4)]"),
    ]))
    tulis("bahasa", "086_kamus_metode", kumpulan("Metode kamus", [
        "d = {\"a\": 1, \"b\": 2}",
        cek('d.pop("a")', "1"),
        cek("d", "{'b': 2}"),
        "d.update({\"c\": 3})",
        cek('d["c"]', "3"),
        "d.perbarui({\"d\": 4})",
        cek('d["d"]', "4"),
        "s = d.salin()",
        "s[\"e\"] = 5",
        cek('"e" dalam d', "salah"),
        cek("d.popitem()", "('d', 4)"),
        "d.bersihkan()",
        cek("d", "{}"),
    ]))
    tulis("bahasa", "087_himpunan", kumpulan("Himpunan", [
        "h = {1, 2, 3}",
        cek("panjang(h)", "3"),
        cek("2 dalam h", "benar"),
        "h.tambah(4)",
        cek("4 dalam h", "benar"),
        "h.buang(1)",
        cek("1 dalam h", "salah"),
        cek("h | {5}", "{2, 3, 4, 5}"),
        cek("h & {2, 9}", "{2}"),
        cek("h - {2}", "{3, 4}"),
        cek("h ^ {3, 9}", "{2, 4, 9}"),
        cek("{1, 2}.adalah_subhimpunan({1, 2, 3})", "benar"),
        cek("{1, 2, 3}.adalah_superhimpunan({1, 2})", "benar"),
        cek("{1}.terpisah({2, 3})", "benar"),
    ]))
    tulis("bahasa", "088_himpunan_metode", kumpulan("Metode himpunan", [
        "h = {1, 2, 3}",
        cek("h.gabung({4})", "{1, 2, 3, 4}"),
        cek("h.irisan({2, 3, 4})", "{2, 3}"),
        cek("h.selisih({1})", "{2, 3}"),
        cek("h.selisih_simetris({3, 4})", "{1, 2, 4}"),
        "s = h.salin()",
        "s.tambah(9)",
        cek("9 dalam h", "salah"),
        "h.perbarui({5, 6})",
        cek("panjang(h)", "5"),
        cek("h.ambil() dalam {1, 2, 3, 4, 5, 6}", "benar"),
    ]))
    tulis("bahasa", "089_rentang", kumpulan("Rentang", [
        cek("daftar(rentang(5))", "[0, 1, 2, 3, 4]"),
        cek("daftar(rentang(2, 5))", "[2, 3, 4]"),
        cek("daftar(rentang(0, 10, 3))", "[0, 3, 6, 9]"),
        cek("daftar(rentang(5, 0, -1))", "[5, 4, 3, 2, 1]"),
        cek("panjang(rentang(5))", "5"),
        cek("3 dalam rentang(5)", "benar"),
        cek("5 dalam rentang(5)", "salah"),
        cek("rentang(3)[1]", "1"),
        cek("rentang(5)[1:3]", "rentang(1, 3, 1)"),
    ]))
    tulis("bahasa", "090_slice_objek", kumpulan("Objek slice", [
        "s = slice(1, 5, 2)",
        cek("s.mulai", "1"),
        cek("s.akhir", "5"),
        cek("s.langkah", "2"),
        cek("[0, 1, 2, 3, 4, 5][s]", "[1, 3]"),
        cek("daftar(s.indices(10))", "[1, 5, 2]"),
    ]))
    tulis("bahasa", "091_koleksi_bersarang", kumpulan("Koleksi bersarang", [
        "m = [[1, 2], [3, 4]]",
        cek("m[1][0]", "3"),
        cek("m[0]", "[1, 2]"),
        "m[0][0] = 9",
        cek("m[0][0]", "9"),
        cek("m", "[[9, 2], [3, 4]]"),
    ]))
    tulis("bahasa", "092_alias_dan_salinan", kumpulan("Alias dan salinan", [
        "a = [1, 2]",
        "b = a",
        "b.tambah(3)",
        cek("a", "[1, 2, 3]"),
        "c = a[:]",
        "c.tambah(4)",
        cek("a", "[1, 2, 3]"),
        cek("c", "[1, 2, 3, 4]"),
        "d = {\"x\": [1]}",
        "e = d.salin()",
        'e["x"].tambah(2)',
        cek('d["x"]', "[1, 2]"),
    ]))
    tulis("bahasa", "093_koleksi_bawaan", kumpulan("Konstruktor koleksi", [
        cek("daftar()", "[]"),
        cek("daftar((1, 2))", "[1, 2]"),
        cek("daftar(\"abc\")", "['a', 'b', 'c']"),
        cek("tupel([1, 2])", "(1, 2)"),
        cek("susunan([1, 1, 2])", "{1, 2}"),
        cek("kamus()", "{}"),
        cek('kamus(a=1, b=2)', "{'a': 1, 'b': 2}"),
    ]))
    tulis("bahasa", "094_agregat", kumpulan("Agregat koleksi", [
        cek("jumlah([1, 2, 3])", "6"),
        cek("jumlah([1, 2, 3], 10)", "16"),
        cek("maks([1, 3, 2])", "3"),
        cek("min([1, 3, 2])", "1"),
        cek("maks(1, 5, 3)", "5"),
        cek("min(1, 5, 3)", "1"),
        cek("maks([], default=7)", "7"),
        cek("semua([benar, benar])", "benar"),
        cek("semua([benar, salah])", "salah"),
        cek("apa_saja([salah, benar])", "benar"),
        cek("apa_saja([salah, salah])", "salah"),
    ]))
    tulis("bahasa", "095_koleksi_error", kumpulan("Kesalahan koleksi", [
        "coba:",
        "    x = [1, 2][5]",
        "tangkap KesalahanIndeks:",
        "    pastikan benar, \"indeks daftar\"",
        "coba:",
        "    x = {\"a\": 1}[\"b\"]",
        "tangkap KesalahanKunci:",
        "    pastikan benar, \"kunci kamus\"",
        "coba:",
        "    x = [].ambil()",
        "tangkap KesalahanIndeks:",
        "    pastikan benar, \"pop kosong\"",
    ]))


# ------------------------------------------------------------------
# Bahasa: kendali alur
# ------------------------------------------------------------------

def kendali() -> None:
    tulis("bahasa", "100_jika", kumpulan("Percabangan jika", [
        "nilai = 85",
        'status = "A" jika nilai >= 90 lainnya ("B" jika nilai >= 80 lainnya "C")',
        cek("status", '"B"'),
        "hasil = []",
        "jika nilai > 100:",
        '    hasil.tambah("besar")',
        "lainnya jika nilai >= 80:",
        '    hasil.tambah("sedang")',
        "lainnya:",
        '    hasil.tambah("kecil")',
        cek("hasil", "['sedang']"),
    ]))
    tulis("bahasa", "101_selama", kumpulan("Perulangan selama", [
        "i = 0",
        "total = 0",
        "selama i < 5:",
        "    total += i",
        "    i += 1",
        cek("total", "10"),
        cek("i", "5"),
    ]))
    tulis("bahasa", "102_selama_lainnya", kumpulan("Selama dengan lainnya", [
        "i = 0",
        "selama i < 3:",
        "    i += 1",
        "lainnya:",
        "    selesai = benar",
        cek("selesai", "benar"),
        "selama benar:",
        "    hentikan",
        "lainnya:",
        "    tidak_jalan = benar",
        cek('"tidak_jalan" dalam globals()', "salah"),
    ]))
    tulis("bahasa", "103_untuk_dalam", kumpulan("Untuk dalam", [
        "hasil = []",
        "untuk x dalam [1, 2, 3]:",
        "    hasil.tambah(x * 2)",
        cek("hasil", "[2, 4, 6]"),
    ]))
    tulis("bahasa", "104_untuk_setiap", kumpulan("Untuk setiap", [
        "hasil = []",
        'untuk setiap nama dalam ["Ani", "Budi"]:',
        "    hasil.tambah(nama.huruf_besar())",
        cek("hasil", "['ANI', 'BUDI']"),
    ]))
    tulis("bahasa", "105_untuk_sebanyak", kumpulan("Untuk sebanyak", [
        "hasil = []",
        "untuk i sebanyak 4:",
        "    hasil.tambah(i)",
        cek("hasil", "[0, 1, 2, 3]"),
    ]))
    tulis("bahasa", "106_untuk_dari_sampai", kumpulan("Untuk dari sampai", [
        "naik = []",
        "untuk i dari 1 sampai 4:",
        "    naik.tambah(i)",
        cek("naik", "[1, 2, 3, 4]"),
        "turun = []",
        "untuk i dari 4 sampai 1:",
        "    turun.tambah(i)",
        cek("turun", "[4, 3, 2, 1]"),
        "langkah = []",
        "untuk i dari 1 sampai 7 langkah 2:",
        "    langkah.tambah(i)",
        cek("langkah", "[1, 3, 5, 7]"),
    ]))
    tulis("bahasa", "107_hentikan_lanjutkan", kumpulan("Hentikan dan lanjutkan", [
        "hasil = []",
        "untuk i dalam rentang(10):",
        "    jika i == 3:",
        "        lanjutkan",
        "    jika i == 6:",
        "        hentikan",
        "    hasil.tambah(i)",
        cek("hasil", "[0, 1, 2, 4, 5]"),
    ]))
    tulis("bahasa", "108_untuk_lainnya", kumpulan("Untuk dengan lainnya", [
        "selesai = salah",
        "untuk i dalam rentang(3):",
        "    lewati",
        "lainnya:",
        "    selesai = benar",
        cek("selesai", "benar"),
        "selesai = salah",
        "untuk i dalam rentang(3):",
        "    jika i == 1:",
        "        hentikan",
        "lainnya:",
        "    selesai = benar",
        cek("selesai", "salah"),
    ]))
    tulis("bahasa", "109_bersarang", kumpulan("Perulangan bersarang", [
        "hasil = []",
        "untuk i dalam rentang(3):",
        "    untuk j dalam rentang(2):",
        "        hasil.tambah((i, j))",
        cek("panjang(hasil)", "6"),
        cek("hasil[0]", "(0, 0)"),
        cek("hasil[-1]", "(2, 1)"),
    ]))
    tulis("bahasa", "110_lewati", kumpulan("Pernyataan lewati", [
        "jika benar:",
        "    lewati",
        "selama salah:",
        "    lewati",
        "cetak(\"OK\")",
    ]))
    tulis("bahasa", "111_pastikan", kumpulan("Pernyataan pastikan", [
        "nilai = 10",
        'pastikan nilai > 0, "positif"',
        "coba:",
        '    pastikan nilai < 0, "harus gagal"',
        "tangkap KesalahanKlaim:",
        "    pastikan benar, \"klaim gagal tertangkap\"",
    ]))
    tulis("bahasa", "112_hapus", kumpulan("Pernyataan hapus", [
        "nilai = 10",
        "hapus nilai",
        cek('"nilai" dalam globals()', "salah"),
        "d = [1, 2, 3]",
        "hapus d[1]",
        cek("d", "[1, 3]"),
        "k = {\"a\": 1, \"b\": 2}",
        'hapus k["a"]',
        cek("k", "{'b': 2}"),
    ]))
    tulis("bahasa", "113_lingkup_blok", kumpulan("Lingkup blok", [
        "jika benar:",
        "    di_dalam = 5",
        cek("di_dalam", "5"),
        "untuk i dalam rentang(2):",
        "    terakhir = i",
        cek("terakhir", "1"),
    ]))
    tulis("bahasa", "114_ekspresi_bersyarat", kumpulan("Ekspresi bersyarat", [
        "nilai = 7",
        'jenis_nilai = "ganjil" jika nilai % 2 == 1 lainnya "genap"',
        cek("jenis_nilai", '"ganjil"'),
        cek('("besar" jika nilai > 5 lainnya "kecil")', '"besar"'),
    ]))
    tulis("bahasa", "115_while_break_continue", kumpulan("While break continue", [
        "i = 0",
        "hasil = []",
        "selama benar:",
        "    i += 1",
        "    jika i % 2 == 0:",
        "        lanjutkan",
        "    jika i > 7:",
        "        hentikan",
        "    hasil.tambah(i)",
        cek("hasil", "[1, 3, 5, 7]"),
    ]))


# ------------------------------------------------------------------
# Bahasa: fungsi
# ------------------------------------------------------------------

def fungsi() -> None:
    tulis("bahasa", "120_fungsi_dasar", kumpulan("Fungsi dasar", [
        "fungsi tambah(a, b):",
        "    kembalikan a + b",
        cek("tambah(2, 3)", "5"),
        cek("tambah(b=3, a=2)", "5"),
    ]))
    tulis("bahasa", "121_fungsi_default", kumpulan("Nilai default", [
        "fungsi sapa(nama, sapaan=\"Halo\"):",
        "    kembalikan f\"{sapaan}, {nama}\"",
        cek('sapa("Ani")', '"Halo, Ani"'),
        cek('sapa("Ani", "Hai")', '"Hai, Ani"'),
        cek('sapa("Ani", sapaan="Selamat")', '"Selamat, Ani"'),
    ]))
    tulis("bahasa", "122_fungsi_varargs", kumpulan("Varargs", [
        "fungsi jumlahkan(*angka):",
        "    total = 0",
        "    untuk n dalam angka:",
        "        total += n",
        "    kembalikan total",
        cek("jumlahkan()", "0"),
        cek("jumlahkan(1, 2, 3)", "6"),
        cek("jumlahkan(*[4, 5])", "9"),
    ]))
    tulis("bahasa", "123_fungsi_kwargs", kumpulan("Kwargs", [
        "fungsi info(**data):",
        "    kembalikan data",
        cek('info(nama="Ani", umur=25)', "{'nama': 'Ani', 'umur': 25}"),
        cek('info(**{"a": 1})', "{'a': 1}"),
    ]))
    tulis("bahasa", "124_fungsi_campuran", kumpulan("Parameter campuran", [
        "fungsi f(a, b=2, *sisa, c=3, **opsi):",
        "    kembalikan (a, b, sisa, c, opsi)",
        cek("f(1)", "(1, 2, (), 3, {})"),
        cek("f(1, 5, 6, 7, c=9)", "(1, 5, (6, 7), 9, {})"),
        cek("f(1, c=8, d=10)", "(1, 2, (), 8, {'d': 10})"),
    ]))
    tulis("bahasa", "125_fungsi_positional_only", kumpulan("Positional only", [
        "fungsi bagi(a, b, /, *, bulatkan=salah):",
        "    hasil = a / b",
        "    jika bulatkan:",
        "        kembalikan bulat(hasil)",
        "    kembalikan hasil",
        cek("bagi(10, 4)", "2.5"),
        cek("bagi(10, 4, bulatkan=benar)", "2"),
        "coba:",
        "    bagi(a=10, b=2)",
        "tangkap KesalahanTipe:",
        "    pastikan benar, \"positional only\"",
    ]))
    tulis("bahasa", "126_rekursi", kumpulan("Rekursi", [
        "fungsi faktorial(n):",
        "    jika n <= 1:",
        "        kembalikan 1",
        "    kembalikan n * faktorial(n - 1)",
        cek("faktorial(5)", "120"),
        cek("faktorial(10)", "3628800"),
        "fungsi fib(n):",
        "    jika n < 2:",
        "        kembalikan n",
        "    kembalikan fib(n - 1) + fib(n - 2)",
        cek("fib(10)", "55"),
    ]))
    tulis("bahasa", "127_lambda", kumpulan("Lambda", [
        "kali = lambda x, y=2: x * y",
        cek("kali(3)", "6"),
        cek("kali(3, 4)", "12"),
        "identitas = lambda x: x",
        cek("identitas(5)", "5"),
        cek("(lambda: 42)()", "42"),
    ]))
    tulis("bahasa", "128_closure", kumpulan("Closure", [
        "fungsi buat_penambah(n):",
        "    fungsi tambah(x):",
        "        kembalikan x + n",
        "    kembalikan tambah",
        "tambah5 = buat_penambah(5)",
        cek("tambah5(10)", "15"),
        "tambah10 = buat_penambah(10)",
        cek("tambah10(1)", "11"),
    ]))
    tulis("bahasa", "129_nonlokal", kumpulan("Nonlokal", [
        "fungsi penghitung():",
        "    nilai = 0",
        "    fungsi naik():",
        "        nonlokal nilai",
        "        nilai += 1",
        "        kembalikan nilai",
        "    kembalikan naik",
        "naik = penghitung()",
        cek("naik()", "1"),
        cek("naik()", "2"),
        cek("naik()", "3"),
    ]))
    tulis("bahasa", "130_global", kumpulan("Global", [
        "nilai = 1",
        "fungsi ubah():",
        "    global nilai",
        "    nilai = 10",
        "ubah()",
        cek("nilai", "10"),
    ]))
    tulis("bahasa", "131_dekorator", kumpulan("Dekorator", [
        "fungsi dua_kali(f):",
        "    fungsi pembungkus(x):",
        "        kembalikan f(f(x))",
        "    kembalikan pembungkus",
        "@dua_kali",
        "fungsi tambah_satu(x):",
        "    kembalikan x + 1",
        cek("tambah_satu(5)", "7"),
    ]))
    tulis("bahasa", "132_dekorator_argumen", kumpulan("Dekorator berargumen", [
        "fungsi ulang(n):",
        "    fungsi dekorator(f):",
        "        fungsi pembungkus(*args):",
        "            hasil = kosong",
        "            untuk _ dalam rentang(n):",
        "                hasil = f(*args)",
        "            kembalikan hasil",
        "        kembalikan pembungkus",
        "    kembalikan dekorator",
        "@ulang(3)",
        "fungsi ambil():",
        "    kembalikan 7",
        cek("ambil()", "7"),
    ]))
    tulis("bahasa", "133_metadata_fungsi", kumpulan("Metadata fungsi", [
        "fungsi contoh(a, b=2):",
        '    """Dokumentasi contoh."""',
        "    kembalikan a + b",
        cek("contoh.__nama__", '"contoh"'),
        cek("contoh.__dok__", '"Dokumentasi contoh."'),
        cek("daftar(contoh.__parameter__)", "['a', 'b']"),
        cek("contoh.__bawaan__['b']", "2"),
    ]))
    tulis("bahasa", "134_annotations", kumpulan("Anotasi", [
        "fungsi tambah(a: Bilangan, b: Bilangan = 0) -> Bilangan:",
        "    kembalikan a + b",
        cek("tambah(1, 2)", "3"),
        cek("tambah.__anotasi__['a'].__nama__", '"Bilangan"'),
        cek("tambah.__anotasi__['kembali'].__nama__", '"Bilangan"'),
    ]))
    tulis("bahasa", "135_fungsi_sebagai_nilai", kumpulan("Fungsi sebagai nilai", [
        "fungsi kuadrat(x):",
        "    kembalikan x * x",
        "operasi = kuadrat",
        cek("operasi(4)", "16"),
        "daftar_fungsi = [kuadrat, lambda x: x + 1]",
        cek("daftar_fungsi[0](3)", "9"),
        cek("daftar_fungsi[1](3)", "4"),
    ]))
    tulis("bahasa", "136_rekursi_bersarang", kumpulan("Rekursi bersarang", [
        "fungsi jumlah_daftar(items):",
        "    jika bukan items:",
        "        kembalikan 0",
        "    kembalikan items[0] + jumlah_daftar(items[1:])",
        cek("jumlah_daftar([1, 2, 3, 4])", "10"),
    ]))
    tulis("bahasa", "137_fungsi_kesalahan_argumen", kumpulan("Kesalahan argumen", [
        "fungsi f(a, b):",
        "    kembalikan a + b",
        "coba:",
        "    f(1)",
        "tangkap KesalahanTipe:",
        "    pastikan benar, \"argumen kurang\"",
        "coba:",
        "    f(1, 2, 3)",
        "tangkap KesalahanTipe:",
        "    pastikan benar, \"argumen lebih\"",
        "coba:",
        "    f(1, 2, c=3)",
        "tangkap KesalahanTipe:",
        "    pastikan benar, \"kwarg tak dikenal\"",
    ]))
    tulis("bahasa", "138_batas_rekursi", kumpulan("Batas rekursi", [
        "fungsi turun(n):",
        "    jika n == 0:",
        "        kembalikan 0",
        "    kembalikan turun(n - 1)",
        cek("turun(100)", "0"),
    ]))
    tulis("bahasa", "139_fungsi_generator_sederhana", kumpulan("Fungsi generator", [
        "fungsi urutan(n):",
        "    untuk i dalam rentang(n):",
        "        hasilkan i",
        cek("daftar(urutan(4))", "[0, 1, 2, 3]"),
        "g = urutan(3)",
        cek("next(g)", "0"),
        cek("next(g)", "1"),
        cek("next(g)", "2"),
        "coba:",
        "    next(g)",
        "tangkap HentikanIterasi:",
        "    pastikan benar, \"generator habis\"",
    ]))


if __name__ == "__main__":
    bersihkan()
    literal()
    aritmetika()
    perbandingan()
    teks()
    koleksi()
    kendali()
    fungsi()
    print("Bagian pertama selesai")
