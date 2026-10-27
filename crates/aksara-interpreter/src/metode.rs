use std::io::{Read, Seek, SeekFrom, Write};
use std::rc::Rc;

use aksara_ast::Span;

use crate::format::{format_nilai, nilai_ke_repr, nilai_ke_teks};
use crate::interpreter::Interpreter;
use crate::nilai::*;

/// Menentukan apakah tipe memiliki metode bawaan dengan nama tertentu.
pub fn ada_metode(tipe: &str, nama: &str) -> bool {
    let daftar: &[&str] = match tipe {
        "teks" => &[
            "huruf_besar", "huruf_kecil", "kapital", "judul", "balik_kapital", "gabung",
            "pisah", "pisah_baris", "ganti", "mulai_dengan", "berakhir_dengan", "cari",
            "cari_kanan", "indeks", "hitung", "jalur", "jalur_kiri", "jalur_kanan",
            "tengah", "kiri", "kanan", "isi_nol", "format", "encode", "decode",
            "adalah_alfanumerik", "adalah_alfabet", "adalah_angka", "adalah_spasi",
            "adalah_huruf_kecil", "adalah_huruf_besar", "adalah_judul", "adalah_desimal",
            "adalah_numerik", "adalah_identifier", "adalah_cetak", "partisi",
            "partisi_kanan", "hapus_prefix", "hapus_suffix", "perluas_tab", "kapital_kecil",
            "upper", "lower", "strip", "lstrip", "rstrip", "split", "rsplit", "join",
            "replace", "startswith", "endswith", "find", "rfind", "index", "count",
            "title", "capitalize", "casefold", "center", "ljust", "rjust", "zfill",
            "partition", "rpartition", "splitlines", "removeprefix", "removesuffix",
            "expandtabs", "swapcase", "islower", "isupper", "istitle", "isdecimal",
            "isnumeric", "isidentifier", "isprintable",
        ],
        "bytes" => &[
            "decode", "dekode", "hex", "upper", "lower", "strip", "lstrip", "rstrip",
            "split", "join", "replace", "startswith", "endswith", "find", "count",
            "huruf_besar", "huruf_kecil", "jalur", "pisah", "gabung", "ganti",
        ],
        "daftar" => &[
            "tambah", "sisip", "hapus", "ambil", "perluas", "urutkan", "balik", "indeks",
            "hitung", "salin", "bersihkan", "append", "insert", "remove", "pop", "extend",
            "sort", "reverse", "index", "count", "copy", "clear",
        ],
        "tupel" => &["indeks", "hitung", "index", "count"],
        "himpunan" => &[
            "tambah", "hapus", "buang", "ambil", "bersihkan", "gabung", "perbarui",
            "irisan", "irisan_perbarui", "selisih", "selisih_perbarui", "selisih_simetris",
            "selisih_simetris_perbarui", "salin", "adalah_subhimpunan",
            "adalah_superhimpunan", "terpisah", "add", "remove", "discard", "pop",
            "clear", "union", "update", "intersection", "intersection_update",
            "difference", "difference_update", "symmetric_difference",
            "symmetric_difference_update", "copy", "issubset", "issuperset",
            "isdisjoint",
        ],
        "kamus" => &[
            "kunci", "nilai", "item", "ambil", "perbarui", "hapus", "salin", "bersihkan",
            "set_default", "popitem", "keys", "values", "items", "get", "update", "pop",
            "copy", "clear", "setdefault",
        ],
        "berkas" => &[
            "baca", "baca_baris", "baca_semua_baris", "tulis", "tulis_baris", "tutup",
            "cari", "posisi", "siram", "bisa_baca", "bisa_tulis", "bisa_cari", "read",
            "readline", "readlines", "write", "writelines", "close", "seek", "tell",
            "flush", "readable", "writable", "seekable",
        ],
        "generator" => &["__iter__", "__next__", "kirim", "tutup", "lempar", "send", "close", "throw"],
        "kesalahan" => &["__str__", "__repr__"],
        "rentang" => &["hitung", "indeks", "count", "index"],
        "slice" => &["indices"],
        _ => &[],
    };
    daftar.contains(&nama)
}

impl Interpreter {
    pub fn panggil_metode(
        &mut self,
        penerima: &Nilai,
        nama: &str,
        args: Vec<Nilai>,
        kwargs: Vec<(String, Nilai)>,
        span: Span,
    ) -> Hasil<Nilai> {
        match penerima {
            Nilai::Teks(teks) => self.metode_teks(teks, nama, args, kwargs, span),
            Nilai::Bytes(bytes) => self.metode_bytes(bytes, nama, args, kwargs, span),
            Nilai::Daftar(item) => self.metode_daftar(item, nama, args, kwargs, span),
            Nilai::Tupel(item) => self.metode_tupel(item, nama, args, kwargs, span),
            Nilai::Himpunan(item) => self.metode_himpunan(item, nama, args, kwargs, span),
            Nilai::Kamus(item) => self.metode_kamus(item, nama, args, kwargs, span),
            Nilai::Berkas(berkas) => self.metode_berkas(berkas, nama, args, kwargs, span),
            Nilai::Generator(generator) => {
                self.metode_generator(generator, nama, args, kwargs, span)
            }
            Nilai::Kesalahan(_) => match nama {
                "__str__" => Ok(Nilai::teks(nilai_ke_teks(penerima))),
                "__repr__" => Ok(Nilai::teks(nilai_ke_repr(penerima))),
                _ => Err(self.metode_tidak_ada(nama, "kesalahan", span)),
            },
            Nilai::Rentang(rentang) => self.metode_rentang(rentang, nama, args, span),
            Nilai::Slice(slice) => self.metode_slice(slice, nama, args, span),
            Nilai::Instance(instance) => {
                if let Some(fungsi) = instance.kelas.cari_atribut(nama) {
                    let metode = self.bungkus_instance(instance, fungsi);
                    return self.panggil(&metode, args, kwargs, span);
                }
                Err(self.lempar_pesan(
                    "KesalahanAtribut",
                    format!("`{}` tidak memiliki metode `{nama}`", instance.kelas.nama),
                    span,
                ))
            }
            Nilai::Python(_) => crate::python_panggil_metode(penerima, nama, args, kwargs, span),
            other => Err(self.metode_tidak_ada(nama, other.nama_tipe(), span)),
        }
    }

    fn bungkus_instance(&mut self, instance: &Rc<Instance>, nilai: Nilai) -> Nilai {
        match nilai {
            Nilai::Fungsi(fungsi) => Nilai::Metode(Rc::new(Metode {
                fungsi,
                penerima: Nilai::Instance(instance.clone()),
            })),
            Nilai::MetodeStatis(fungsi) => Nilai::Fungsi(fungsi),
            Nilai::MetodeKelas(metode) => Nilai::MetodeKelas(Rc::new(MetodeKelas {
                fungsi: metode.fungsi.clone(),
                kelas: instance.kelas.clone(),
            })),
            other => other,
        }
    }

    fn metode_tidak_ada(&mut self, nama: &str, tipe: &str, span: Span) -> Lemparan {
        self.lempar_pesan(
            "KesalahanAtribut",
            format!("tipe `{tipe}` tidak memiliki metode `{nama}`"),
            span,
        )
    }

    fn satu_arg(&mut self, args: &[Nilai], nama: &str, span: Span) -> Hasil<Nilai> {
        if args.len() != 1 {
            return Err(self.lempar_pesan(
                "KesalahanTipe",
                format!("`{nama}` membutuhkan tepat satu argumen"),
                span,
            ));
        }
        Ok(args[0].clone())
    }

    fn arg_teks(&mut self, args: &[Nilai], nama: &str, span: Span) -> Hasil<String> {
        let nilai = self.satu_arg(args, nama, span)?;
        match nilai {
            Nilai::Teks(teks) => Ok(teks.to_string()),
            other => Err(self.lempar_pesan(
                "KesalahanTipe",
                format!("`{nama}` membutuhkan teks, ditemukan {}", other.nama_tipe()),
                span,
            )),
        }
    }

    fn arg_i64(&mut self, args: &[Nilai], nama: &str, span: Span) -> Hasil<i64> {
        let nilai = self.satu_arg(args, nama, span)?;
        self.nilai_ke_i64(&nilai, span)
    }

    fn ambil_opsi(&self, kwargs: &[(String, Nilai)], nama: &str) -> Option<Nilai> {
        kwargs
            .iter()
            .find(|(kunci, _)| kunci == nama)
            .map(|(_, nilai)| nilai.clone())
    }

    // ------------------------------------------------------------------
    // Teks
    // ------------------------------------------------------------------

    fn metode_teks(
        &mut self,
        teks: &Rc<str>,
        nama: &str,
        args: Vec<Nilai>,
        kwargs: Vec<(String, Nilai)>,
        span: Span,
    ) -> Hasil<Nilai> {
        let teks = teks.to_string();
        match nama {
            "huruf_besar" | "upper" => Ok(Nilai::teks(teks.to_uppercase())),
            "huruf_kecil" | "lower" | "kapital_kecil" | "casefold" => {
                Ok(Nilai::teks(teks.to_lowercase()))
            }
            "kapital" | "capitalize" => {
                let mut chars = teks.chars();
                let hasil = match chars.next() {
                    Some(pertama) => {
                        pertama.to_uppercase().collect::<String>()
                            + &chars.as_str().to_lowercase()
                    }
                    None => String::new(),
                };
                Ok(Nilai::teks(hasil))
            }
            "judul" | "title" => {
                let mut hasil = String::with_capacity(teks.len());
                let mut awal_kata = true;
                for ch in teks.chars() {
                    if ch.is_alphanumeric() {
                        if awal_kata {
                            hasil.extend(ch.to_uppercase());
                        } else {
                            hasil.extend(ch.to_lowercase());
                        }
                        awal_kata = false;
                    } else {
                        hasil.push(ch);
                        awal_kata = true;
                    }
                }
                Ok(Nilai::teks(hasil))
            }
            "balik_kapital" | "swapcase" => {
                let mut hasil = String::with_capacity(teks.len());
                for ch in teks.chars() {
                    if ch.is_uppercase() {
                        hasil.extend(ch.to_lowercase());
                    } else if ch.is_lowercase() {
                        hasil.extend(ch.to_uppercase());
                    } else {
                        hasil.push(ch);
                    }
                }
                Ok(Nilai::teks(hasil))
            }
            "jalur" | "strip" => {
                let chars = self.ambil_opsi(&kwargs, "chars");
                let hasil = match chars {
                    Some(Nilai::Teks(chars)) => {
                        teks.trim_matches(|ch| chars.contains(ch)).to_string()
                    }
                    _ => teks.trim().to_string(),
                };
                Ok(Nilai::teks(hasil))
            }
            "jalur_kiri" | "lstrip" => {
                let chars = self.ambil_opsi(&kwargs, "chars");
                let hasil = match chars {
                    Some(Nilai::Teks(chars)) => {
                        teks.trim_start_matches(|ch| chars.contains(ch)).to_string()
                    }
                    _ => teks.trim_start().to_string(),
                };
                Ok(Nilai::teks(hasil))
            }
            "jalur_kanan" | "rstrip" => {
                let chars = self.ambil_opsi(&kwargs, "chars");
                let hasil = match chars {
                    Some(Nilai::Teks(chars)) => {
                        teks.trim_end_matches(|ch| chars.contains(ch)).to_string()
                    }
                    _ => teks.trim_end().to_string(),
                };
                Ok(Nilai::teks(hasil))
            }
            "gabung" | "join" => {
                let nilai = self.satu_arg(&args, nama, span)?;
                let item = self.iter_nilai(&nilai, span)?;
                let mut bagian = Vec::with_capacity(item.len());
                for nilai in item {
                    match nilai {
                        Nilai::Teks(teks) => bagian.push(teks.to_string()),
                        other => {
                            return Err(self.lempar_pesan(
                                "KesalahanTipe",
                                format!(
                                    "`gabung` hanya menerima teks, ditemukan {}",
                                    other.nama_tipe()
                                ),
                                span,
                            ))
                        }
                    }
                }
                Ok(Nilai::teks(bagian.join(&teks)))
            }
            "pisah" | "split" | "rsplit" => {
                let pemisah = match args.first() {
                    Some(Nilai::Teks(teks)) => Some(teks.to_string()),
                    Some(Nilai::Kosong) | None => None,
                    Some(other) => {
                        return Err(self.lempar_pesan(
                            "KesalahanTipe",
                            format!("pemisah harus teks atau kosong, ditemukan {}", other.nama_tipe()),
                            span,
                        ))
                    }
                };
                let maks = args
                    .get(1)
                    .map(|nilai| self.nilai_ke_i64(nilai, span))
                    .transpose()?
                    .unwrap_or(-1);
                let hasil: Vec<Nilai> = match pemisah {
                    Some(pemisah) if !pemisah.is_empty() => {
                        let bagian: Vec<&str> = if nama == "rsplit" {
                            if maks < 0 {
                                teks.rsplit(&pemisah).collect()
                            } else {
                                teks.rsplitn(maks as usize + 1, &pemisah)
                                    .collect::<Vec<_>>()
                                    .into_iter()
                                    .rev()
                                    .collect()
                            }
                        } else if maks < 0 {
                            teks.split(&pemisah).collect()
                        } else {
                            teks.splitn(maks as usize + 1, &pemisah).collect()
                        };
                        bagian
                            .into_iter()
                            .map(|bagian| Nilai::teks(bagian.to_string()))
                            .collect()
                    }
                    _ => teks
                        .split_whitespace()
                        .map(|bagian| Nilai::teks(bagian.to_string()))
                        .collect(),
                };
                Ok(Nilai::daftar(hasil))
            }
            "pisah_baris" | "splitlines" => {
                let keep = args
                    .first()
                    .map(|nilai| self.kebenaran(nilai))
                    .transpose()?
                    .unwrap_or(false);
                let hasil = if keep {
                    teks.split_inclusive('\n')
                        .map(|baris| Nilai::teks(baris.to_string()))
                        .collect()
                } else {
                    teks.lines()
                        .map(|baris| Nilai::teks(baris.to_string()))
                        .collect()
                };
                Ok(Nilai::daftar(hasil))
            }
            "ganti" | "replace" => {
                if args.len() < 2 {
                    return Err(self.lempar_pesan(
                        "KesalahanTipe",
                        "`ganti` membutuhkan teks lama dan teks baru",
                        span,
                    ));
                }
                let lama = match &args[0] {
                    Nilai::Teks(teks) => teks.to_string(),
                    other => {
                        return Err(self.lempar_pesan(
                            "KesalahanTipe",
                            format!("teks lama harus teks, ditemukan {}", other.nama_tipe()),
                            span,
                        ))
                    }
                };
                let baru = match &args[1] {
                    Nilai::Teks(teks) => teks.to_string(),
                    other => {
                        return Err(self.lempar_pesan(
                            "KesalahanTipe",
                            format!("teks baru harus teks, ditemukan {}", other.nama_tipe()),
                            span,
                        ))
                    }
                };
                let jumlah = args
                    .get(2)
                    .map(|nilai| self.nilai_ke_i64(nilai, span))
                    .transpose()?
                    .unwrap_or(-1);
                let hasil = if jumlah < 0 {
                    teks.replace(&lama, &baru)
                } else {
                    teks.replacen(&lama, &baru, jumlah as usize)
                };
                Ok(Nilai::teks(hasil))
            }
            "mulai_dengan" | "startswith" | "berakhir_dengan" | "endswith" => {
                let awalan = self.satu_arg(&args, nama, span)?;
                let mulai = args
                    .get(1)
                    .map(|nilai| self.nilai_ke_i64(nilai, span))
                    .transpose()?
                    .unwrap_or(0);
                let akhir = args
                    .get(2)
                    .map(|nilai| self.nilai_ke_i64(nilai, span))
                    .transpose()?
                    .unwrap_or(teks.chars().count() as i64);
                let potong: String = teks
                    .chars()
                    .skip(mulai.max(0) as usize)
                    .take((akhir - mulai).max(0) as usize)
                    .collect();
                let daftar = match &awalan {
                    Nilai::Teks(teks) => vec![teks.to_string()],
                    Nilai::Tupel(item) => {
                        let mut hasil = Vec::new();
                        for nilai in item.iter() {
                            match nilai {
                                Nilai::Teks(teks) => hasil.push(teks.to_string()),
                                other => {
                                    return Err(self.lempar_pesan(
                                        "KesalahanTipe",
                                        format!("awalan harus teks, ditemukan {}", other.nama_tipe()),
                                        span,
                                    ))
                                }
                            }
                        }
                        hasil
                    }
                    other => {
                        return Err(self.lempar_pesan(
                            "KesalahanTipe",
                            format!("awalan harus teks atau tupel, ditemukan {}", other.nama_tipe()),
                            span,
                        ))
                    }
                };
                let cocok = daftar.iter().any(|awalan| {
                    if nama == "mulai_dengan" || nama == "startswith" {
                        potong.starts_with(awalan)
                    } else {
                        potong.ends_with(awalan)
                    }
                });
                Ok(Nilai::Logika(cocok))
            }
            "cari" | "find" | "cari_kanan" | "rfind" | "indeks" | "index" => {
                let cari = self.arg_teks(&args, nama, span)?;
                let mulai = args
                    .get(1)
                    .map(|nilai| self.nilai_ke_i64(nilai, span))
                    .transpose()?
                    .unwrap_or(0)
                    .max(0) as usize;
                let akhir = args
                    .get(2)
                    .map(|nilai| self.nilai_ke_i64(nilai, span))
                    .transpose()?
                    .map(|n| n.max(0) as usize)
                    .unwrap_or(teks.len());
                let potong: String = teks.chars().skip(mulai).take(akhir.saturating_sub(mulai)).collect();
                let hasil = if nama == "cari_kanan" || nama == "rfind" {
                    potong.rfind(&cari).map(|i| i as i64)
                } else {
                    potong.find(&cari).map(|i| i as i64)
                };
                match hasil {
                    Some(indeks) => {
                        let offset: usize = teks.chars().take(mulai).map(|ch| ch.len_utf8()).sum();
                        let posisi = potong
                            .char_indices()
                            .nth(0)
                            .map(|_| 0)
                            .unwrap_or(0);
                        let _ = posisi;
                        Ok(Nilai::Bilangan(indeks + offset as i64))
                    }
                    None if nama == "indeks" || nama == "index" => Err(self.lempar_pesan(
                        "KesalahanNilai",
                        format!("subteks `{cari}` tidak ditemukan"),
                        span,
                    )),
                    None => Ok(Nilai::Bilangan(-1)),
                }
            }
            "hitung" | "count" => {
                let cari = self.arg_teks(&args, nama, span)?;
                if cari.is_empty() {
                    return Ok(Nilai::Bilangan(teks.chars().count() as i64 + 1));
                }
                Ok(Nilai::Bilangan(teks.matches(&cari).count() as i64))
            }
            "tengah" | "center" | "kiri" | "ljust" | "kanan" | "rjust" => {
                if args.is_empty() || args.len() > 2 {
                    return Err(self.lempar_pesan(
                        "KesalahanTipe",
                        format!("`{nama}` menerima satu atau dua argumen"),
                        span,
                    ));
                }
                let lebar = self.nilai_ke_i64(&args[0], span)?.max(0) as usize;
                let isi = match args.get(1) {
                    Some(Nilai::Teks(teks)) => teks.chars().next().unwrap_or(' '),
                    None => ' ',
                    Some(_) => {
                        return Err(self.lempar_pesan(
                            "KesalahanTipe",
                            format!("`{nama}` membutuhkan teks pengisi"),
                            span,
                        ))
                    }
                };
                let panjang = teks.chars().count();
                if panjang >= lebar {
                    return Ok(Nilai::teks(teks));
                }
                let selisih = lebar - panjang;
                let hasil = if nama == "kiri" || nama == "ljust" {
                    format!("{teks}{}", isi.to_string().repeat(selisih))
                } else if nama == "kanan" || nama == "rjust" {
                    format!("{}{teks}", isi.to_string().repeat(selisih))
                } else {
                    let kiri = selisih / 2;
                    let kanan = selisih - kiri;
                    format!(
                        "{}{teks}{}",
                        isi.to_string().repeat(kiri),
                        isi.to_string().repeat(kanan)
                    )
                };
                Ok(Nilai::teks(hasil))
            }
            "isi_nol" | "zfill" => {
                let lebar = self.arg_i64(&args, nama, span)?.max(0) as usize;
                let panjang = teks.chars().count();
                if panjang >= lebar {
                    return Ok(Nilai::teks(teks));
                }
                let selisih = lebar - panjang;
                let (tanda, isi) = if teks.starts_with('+') || teks.starts_with('-') {
                    (teks[..1].to_string(), teks[1..].to_string())
                } else {
                    (String::new(), teks.clone())
                };
                Ok(Nilai::teks(format!(
                    "{tanda}{}{isi}",
                    "0".repeat(selisih)
                )))
            }
            "format" => self.format_teks(&teks, &args, &kwargs, span),
            "encode" => {
                let encoding = args
                    .first()
                    .and_then(|nilai| match nilai {
                        Nilai::Teks(teks) => Some(teks.to_string()),
                        _ => None,
                    })
                    .unwrap_or_else(|| "utf-8".to_string());
                if !encoding.eq_ignore_ascii_case("utf-8")
                    && !encoding.eq_ignore_ascii_case("utf8")
                    && !encoding.eq_ignore_ascii_case("ascii")
                {
                    return Err(self.lempar_pesan(
                        "KesalahanNilai",
                        format!("encoding `{encoding}` belum didukung"),
                        span,
                    ));
                }
                if encoding.eq_ignore_ascii_case("ascii") && !teks.is_ascii() {
                    return Err(self.lempar_pesan(
                        "KesalahanNilai",
                        "teks memuat karakter non-ASCII",
                        span,
                    ));
                }
                Ok(Nilai::Bytes(Rc::new(teks.into_bytes())))
            }
            "adalah_alfanumerik" | "isalnum" => Ok(Nilai::Logika(!teks.is_empty() && teks.chars().all(char::is_alphanumeric))),
            "adalah_alfabet" | "isalpha" => Ok(Nilai::Logika(!teks.is_empty() && teks.chars().all(char::is_alphabetic))),
            "adalah_angka" | "isdigit" => Ok(Nilai::Logika(!teks.is_empty() && teks.chars().all(|ch| ch.is_ascii_digit()))),
            "adalah_desimal" | "isdecimal" => Ok(Nilai::Logika(!teks.is_empty() && teks.chars().all(|ch| ch.is_ascii_digit()))),
            "adalah_numerik" | "isnumeric" => Ok(Nilai::Logika(!teks.is_empty() && teks.chars().all(|ch| ch.is_numeric()))),
            "adalah_spasi" | "isspace" => Ok(Nilai::Logika(!teks.is_empty() && teks.chars().all(char::is_whitespace))),
            "adalah_huruf_kecil" | "islower" => Ok(Nilai::Logika(teks.chars().any(char::is_lowercase) && !teks.chars().any(char::is_uppercase))),
            "adalah_huruf_besar" | "isupper" => Ok(Nilai::Logika(teks.chars().any(char::is_uppercase) && !teks.chars().any(char::is_lowercase))),
            "adalah_judul" | "istitle" => {
                let mut awal = true;
                let mut ada = false;
                let mut benar = true;
                for ch in teks.chars() {
                    if ch.is_alphabetic() {
                        ada = true;
                        if awal && !ch.is_uppercase() {
                            benar = false;
                        }
                        if !awal && !ch.is_lowercase() {
                            benar = false;
                        }
                        awal = false;
                    } else {
                        awal = true;
                    }
                }
                Ok(Nilai::Logika(ada && benar))
            }
            "adalah_identifier" | "isidentifier" => Ok(Nilai::Logika(is_identifier(&teks))),
            "adalah_cetak" | "isprintable" => Ok(Nilai::Logika(teks.chars().all(|ch| !ch.is_control()))),
            "partisi" | "partition" | "partisi_kanan" | "rpartition" => {
                let pemisah = self.arg_teks(&args, nama, span)?;
                let hasil = if nama == "partisi" || nama == "partition" {
                    teks.find(&pemisah).map(|i| {
                        vec![
                            Nilai::teks(teks[..i].to_string()),
                            Nilai::teks(pemisah.clone()),
                            Nilai::teks(teks[i + pemisah.len()..].to_string()),
                        ]
                    })
                } else {
                    teks.rfind(&pemisah).map(|i| {
                        vec![
                            Nilai::teks(teks[..i].to_string()),
                            Nilai::teks(pemisah.clone()),
                            Nilai::teks(teks[i + pemisah.len()..].to_string()),
                        ]
                    })
                };
                Ok(Nilai::tupel(hasil.unwrap_or_else(|| {
                    vec![
                        Nilai::teks(teks),
                        Nilai::teks(""),
                        Nilai::teks(""),
                    ]
                })))
            }
            "hapus_prefix" | "removeprefix" => {
                let awalan = self.arg_teks(&args, nama, span)?;
                Ok(Nilai::teks(teks.strip_prefix(&awalan).unwrap_or(&teks).to_string()))
            }
            "hapus_suffix" | "removesuffix" => {
                let akhiran = self.arg_teks(&args, nama, span)?;
                Ok(Nilai::teks(teks.strip_suffix(&akhiran).unwrap_or(&teks).to_string()))
            }
            "perluas_tab" | "expandtabs" => {
                let ukuran = args
                    .first()
                    .map(|nilai| self.nilai_ke_i64(nilai, span))
                    .transpose()?
                    .unwrap_or(8)
                    .max(0) as usize;
                let mut hasil = String::new();
                let mut kolom = 0usize;
                for ch in teks.chars() {
                    if ch == '\t' {
                        let isi = if ukuran == 0 { 0 } else { ukuran - (kolom % ukuran) };
                        hasil.push_str(&" ".repeat(isi));
                        kolom += isi;
                    } else {
                        hasil.push(ch);
                        kolom += 1;
                    }
                }
                Ok(Nilai::teks(hasil))
            }
            _ => Err(self.metode_tidak_ada(nama, "teks", span)),
        }
    }

    fn format_teks(
        &mut self,
        format: &str,
        args: &[Nilai],
        kwargs: &[(String, Nilai)],
        span: Span,
    ) -> Hasil<Nilai> {
        let mut hasil = String::new();
        let mut chars = format.chars().peekable();
        let mut indeks = 0usize;
        while let Some(ch) = chars.next() {
            if ch == '{' {
                if chars.peek() == Some(&'{') {
                    chars.next();
                    hasil.push('{');
                    continue;
                }
                let mut isi = String::new();
                let mut dalam = 1;
                while let Some(ch) = chars.next() {
                    if ch == '{' {
                        dalam += 1;
                    } else if ch == '}' {
                        dalam -= 1;
                        if dalam == 0 {
                            break;
                        }
                    }
                    isi.push(ch);
                }
                let (nama, spec) = match isi.split_once(':') {
                    Some((nama, spec)) => (nama, spec),
                    None => (isi.as_str(), ""),
                };
                let (nama, konversi) = match nama.split_once('!') {
                    Some((nama, konversi)) => (nama, konversi.chars().next()),
                    None => (nama, None),
                };
                let nilai = if nama.is_empty() {
                    let nilai = args.get(indeks).cloned().unwrap_or(Nilai::Kosong);
                    indeks += 1;
                    nilai
                } else if let Ok(posisi) = nama.parse::<usize>() {
                    args.get(posisi).cloned().unwrap_or(Nilai::Kosong)
                } else {
                    self.ambil_opsi(kwargs, nama).unwrap_or(Nilai::Kosong)
                };
                let nilai = match konversi {
                    Some('r') => Nilai::teks(nilai_ke_repr(&nilai)),
                    Some('s') | None => nilai,
                    _ => nilai,
                };
                hasil.push_str(&format_nilai(&nilai, spec).map_err(|pesan| {
                    self.lempar_pesan("KesalahanNilai", pesan, span)
                })?);
            } else if ch == '}' {
                if chars.peek() == Some(&'}') {
                    chars.next();
                }
                hasil.push('}');
            } else {
                hasil.push(ch);
            }
        }
        Ok(Nilai::teks(hasil))
    }

    // ------------------------------------------------------------------
    // Bytes
    // ------------------------------------------------------------------

    fn metode_bytes(
        &mut self,
        bytes: &Rc<Vec<u8>>,
        nama: &str,
        args: Vec<Nilai>,
        _kwargs: Vec<(String, Nilai)>,
        span: Span,
    ) -> Hasil<Nilai> {
        match nama {
            "decode" | "dekode" => {
                let encoding = args
                    .first()
                    .and_then(|nilai| match nilai {
                        Nilai::Teks(teks) => Some(teks.to_string()),
                        _ => None,
                    })
                    .unwrap_or_else(|| "utf-8".to_string());
                if !encoding.eq_ignore_ascii_case("utf-8")
                    && !encoding.eq_ignore_ascii_case("utf8")
                    && !encoding.eq_ignore_ascii_case("ascii")
                {
                    return Err(self.lempar_pesan(
                        "KesalahanNilai",
                        format!("encoding `{encoding}` belum didukung"),
                        span,
                    ));
                }
                match String::from_utf8(bytes.as_ref().clone()) {
                    Ok(teks) => Ok(Nilai::teks(teks)),
                    Err(_) => Err(self.lempar_pesan(
                        "KesalahanNilai",
                        "bytes bukan UTF-8 valid",
                        span,
                    )),
                }
            }
            "hex" => Ok(Nilai::teks(
                bytes.iter().map(|b| format!("{b:02x}")).collect::<String>(),
            )),
            "upper" | "huruf_besar" => Ok(Nilai::Bytes(Rc::new(
                bytes.iter().map(|b| b.to_ascii_uppercase()).collect(),
            ))),
            "lower" | "huruf_kecil" => Ok(Nilai::Bytes(Rc::new(
                bytes.iter().map(|b| b.to_ascii_lowercase()).collect(),
            ))),
            "strip" | "jalur" => {
                let awal = bytes.iter().position(|b| !b.is_ascii_whitespace());
                let akhir = bytes.iter().rposition(|b| !b.is_ascii_whitespace());
                match (awal, akhir) {
                    (Some(awal), Some(akhir)) => Ok(Nilai::Bytes(Rc::new(
                        bytes[awal..=akhir].to_vec(),
                    ))),
                    _ => Ok(Nilai::Bytes(Rc::new(Vec::new()))),
                }
            }
            "split" | "pisah" => {
                let pemisah = match args.first() {
                    Some(Nilai::Bytes(pemisah)) => pemisah.as_ref().clone(),
                    _ => vec![b' '],
                };
                let mut hasil = Vec::new();
                let mut mulai = 0usize;
                for (i, jendela) in bytes.windows(pemisah.len()).enumerate() {
                    if jendela == pemisah.as_slice() {
                        hasil.push(Nilai::Bytes(Rc::new(bytes[mulai..i].to_vec())));
                        mulai = i + pemisah.len();
                    }
                }
                hasil.push(Nilai::Bytes(Rc::new(bytes[mulai..].to_vec())));
                Ok(Nilai::daftar(hasil))
            }
            "join" | "gabung" => {
                let nilai = self.satu_arg(&args, nama, span)?;
                let item = self.iter_nilai(&nilai, span)?;
                let mut hasil = Vec::new();
                for (i, nilai) in item.into_iter().enumerate() {
                    if i > 0 {
                        hasil.extend_from_slice(bytes);
                    }
                    match nilai {
                        Nilai::Bytes(bagian) => hasil.extend_from_slice(&bagian),
                        other => {
                            return Err(self.lempar_pesan(
                                "KesalahanTipe",
                                format!("`gabung` bytes hanya menerima bytes, ditemukan {}", other.nama_tipe()),
                                span,
                            ))
                        }
                    }
                }
                Ok(Nilai::Bytes(Rc::new(hasil)))
            }
            "replace" | "ganti" => {
                let lama = match args.first() {
                    Some(Nilai::Bytes(nilai)) => nilai.as_ref().clone(),
                    _ => return Err(self.lempar_pesan("KesalahanTipe", "`ganti` membutuhkan bytes lama", span)),
                };
                let baru = match args.get(1) {
                    Some(Nilai::Bytes(nilai)) => nilai.as_ref().clone(),
                    _ => return Err(self.lempar_pesan("KesalahanTipe", "`ganti` membutuhkan bytes baru", span)),
                };
                let mut hasil = Vec::new();
                let mut mulai = 0usize;
                while mulai < bytes.len() {
                    if bytes[mulai..].starts_with(&lama) {
                        hasil.extend_from_slice(&baru);
                        mulai += lama.len();
                    } else {
                        hasil.push(bytes[mulai]);
                        mulai += 1;
                    }
                }
                Ok(Nilai::Bytes(Rc::new(hasil)))
            }
            "startswith" | "mulai_dengan" | "endswith" | "berakhir_dengan" => {
                let awalan = match args.first() {
                    Some(Nilai::Bytes(nilai)) => nilai.as_ref().clone(),
                    _ => return Err(self.lempar_pesan("KesalahanTipe", "argumen harus bytes", span)),
                };
                let hasil = if nama.starts_with("start") || nama.starts_with("mulai") {
                    bytes.starts_with(&awalan)
                } else {
                    bytes.ends_with(&awalan)
                };
                Ok(Nilai::Logika(hasil))
            }
            "find" | "cari" => {
                let cari = match args.first() {
                    Some(Nilai::Bytes(nilai)) => nilai.as_ref().clone(),
                    _ => return Err(self.lempar_pesan("KesalahanTipe", "argumen harus bytes", span)),
                };
                match bytes.windows(cari.len()).position(|jendela| jendela == cari.as_slice()) {
                    Some(posisi) => Ok(Nilai::Bilangan(posisi as i64)),
                    None => Ok(Nilai::Bilangan(-1)),
                }
            }
            "count" | "hitung" => {
                let cari = match args.first() {
                    Some(Nilai::Bytes(nilai)) => nilai.as_ref().clone(),
                    _ => return Err(self.lempar_pesan("KesalahanTipe", "argumen harus bytes", span)),
                };
                Ok(Nilai::Bilangan(
                    bytes
                        .windows(cari.len())
                        .filter(|jendela| *jendela == cari.as_slice())
                        .count() as i64,
                ))
            }
            _ => Err(self.metode_tidak_ada(nama, "bytes", span)),
        }
    }

    // ------------------------------------------------------------------
    // Daftar dan tupel
    // ------------------------------------------------------------------

    fn metode_daftar(
        &mut self,
        item: &Rc<std::cell::RefCell<Vec<Nilai>>>,
        nama: &str,
        args: Vec<Nilai>,
        kwargs: Vec<(String, Nilai)>,
        span: Span,
    ) -> Hasil<Nilai> {
        match nama {
            "tambah" | "append" => {
                let nilai = self.satu_arg(&args, nama, span)?;
                item.borrow_mut().push(nilai);
                Ok(Nilai::Kosong)
            }
            "sisip" | "insert" => {
                if args.len() != 2 {
                    return Err(self.lempar_pesan("KesalahanTipe", "`sisip` membutuhkan indeks dan nilai", span));
                }
                let indeks = self.nilai_ke_i64(&args[0], span)?;
                let panjang = item.borrow().len() as i64;
                let posisi = if indeks < 0 {
                    (indeks + panjang).max(0) as usize
                } else {
                    indeks.min(panjang) as usize
                };
                item.borrow_mut().insert(posisi, args[1].clone());
                Ok(Nilai::Kosong)
            }
            "hapus" | "remove" => {
                let nilai = self.satu_arg(&args, nama, span)?;
                let posisi = {
                    let daftar = item.borrow();
                    let mut posisi = None;
                    for (i, kandidat) in daftar.iter().enumerate() {
                        if self.sama(kandidat, &nilai)? {
                            posisi = Some(i);
                            break;
                        }
                    }
                    posisi
                };
                match posisi {
                    Some(posisi) => {
                        item.borrow_mut().remove(posisi);
                        Ok(Nilai::Kosong)
                    }
                    None => Err(self.lempar_pesan("KesalahanNilai", "nilai tidak ditemukan di daftar", span)),
                }
            }
            "ambil" | "pop" => {
                let panjang = item.borrow().len();
                if panjang == 0 {
                    return Err(self.lempar_pesan("KesalahanIndeks", "tidak dapat mengambil dari daftar kosong", span));
                }
                let indeks = args
                    .first()
                    .map(|nilai| self.nilai_ke_i64(nilai, span))
                    .transpose()?
                    .unwrap_or(-1);
                let posisi = if indeks < 0 {
                    (indeks + panjang as i64) as usize
                } else {
                    indeks as usize
                };
                if posisi >= panjang {
                    return Err(self.lempar_pesan("KesalahanIndeks", format!("indeks {indeks} di luar rentang"), span));
                }
                Ok(item.borrow_mut().remove(posisi))
            }
            "perluas" | "extend" => {
                let nilai = self.satu_arg(&args, nama, span)?;
                let tambahan = self.iter_nilai(&nilai, span)?;
                item.borrow_mut().extend(tambahan);
                Ok(Nilai::Kosong)
            }
            "urutkan" | "sort" => {
                let kunci = self.ambil_opsi(&kwargs, "kunci").or_else(|| self.ambil_opsi(&kwargs, "key"));
                let terbalik = self
                    .ambil_opsi(&kwargs, "terbalik")
                    .or_else(|| self.ambil_opsi(&kwargs, "reverse"))
                    .map(|nilai| self.kebenaran(&nilai))
                    .transpose()?
                    .unwrap_or(false);
                let mut daftar = item.borrow().clone();
                // Urutan gelembung dipakai agar galat perbandingan dapat
                // dipropagasikan dan pengurutan tetap stabil.
                let panjang = daftar.len();
                for i in 0..panjang {
                    for j in 0..panjang.saturating_sub(1 + i) {
                        let a = match &kunci {
                            Some(kunci) => self.panggil(kunci, vec![daftar[j].clone()], Vec::new(), span)?,
                            None => daftar[j].clone(),
                        };
                        let b = match &kunci {
                            Some(kunci) => self.panggil(kunci, vec![daftar[j + 1].clone()], Vec::new(), span)?,
                            None => daftar[j + 1].clone(),
                        };
                        let lebih = if terbalik {
                            self.bandingkan(aksara_ast::CmpOp::Lt, &a, &b, span)?
                        } else {
                            self.bandingkan(aksara_ast::CmpOp::Gt, &a, &b, span)?
                        };
                        if lebih {
                            daftar.swap(j, j + 1);
                        }
                    }
                }
                *item.borrow_mut() = daftar;
                Ok(Nilai::Kosong)
            }
            "balik" | "reverse" => {
                item.borrow_mut().reverse();
                Ok(Nilai::Kosong)
            }
            "indeks" | "index" => {
                let nilai = self.satu_arg(&args, nama, span)?;
                let daftar = item.borrow().clone();
                for (i, kandidat) in daftar.iter().enumerate() {
                    if self.sama(kandidat, &nilai)? {
                        return Ok(Nilai::Bilangan(i as i64));
                    }
                }
                Err(self.lempar_pesan("KesalahanNilai", "nilai tidak ditemukan di daftar", span))
            }
            "hitung" | "count" => {
                let nilai = self.satu_arg(&args, nama, span)?;
                let daftar = item.borrow().clone();
                let mut jumlah = 0i64;
                for kandidat in &daftar {
                    if self.sama(kandidat, &nilai)? {
                        jumlah += 1;
                    }
                }
                Ok(Nilai::Bilangan(jumlah))
            }
            "salin" | "copy" => Ok(Nilai::daftar(item.borrow().clone())),
            "bersihkan" | "clear" => {
                item.borrow_mut().clear();
                Ok(Nilai::Kosong)
            }
            _ => Err(self.metode_tidak_ada(nama, "daftar", span)),
        }
    }

    fn metode_tupel(
        &mut self,
        item: &Rc<Vec<Nilai>>,
        nama: &str,
        args: Vec<Nilai>,
        _kwargs: Vec<(String, Nilai)>,
        span: Span,
    ) -> Hasil<Nilai> {
        match nama {
            "indeks" | "index" => {
                let nilai = self.satu_arg(&args, nama, span)?;
                for (i, kandidat) in item.iter().enumerate() {
                    if self.sama(kandidat, &nilai)? {
                        return Ok(Nilai::Bilangan(i as i64));
                    }
                }
                Err(self.lempar_pesan("KesalahanNilai", "nilai tidak ditemukan di tupel", span))
            }
            "hitung" | "count" => {
                let nilai = self.satu_arg(&args, nama, span)?;
                let mut jumlah = 0i64;
                for kandidat in item.iter() {
                    if self.sama(kandidat, &nilai)? {
                        jumlah += 1;
                    }
                }
                Ok(Nilai::Bilangan(jumlah))
            }
            _ => Err(self.metode_tidak_ada(nama, "tupel", span)),
        }
    }

    // ------------------------------------------------------------------
    // Himpunan
    // ------------------------------------------------------------------

    fn metode_himpunan(
        &mut self,
        item: &Rc<std::cell::RefCell<Vec<Nilai>>>,
        nama: &str,
        args: Vec<Nilai>,
        _kwargs: Vec<(String, Nilai)>,
        span: Span,
    ) -> Hasil<Nilai> {
        match nama {
            "tambah" | "add" => {
                let nilai = self.satu_arg(&args, nama, span)?;
                let mut daftar = item.borrow().clone();
                self.tambah_himpunan(&mut daftar, nilai);
                *item.borrow_mut() = daftar;
                Ok(Nilai::Kosong)
            }
            "hapus" | "remove" => {
                let nilai = self.satu_arg(&args, nama, span)?;
                let mut daftar = item.borrow().clone();
                let posisi = self.posisi_dalam(&daftar, &nilai)?;
                match posisi {
                    Some(posisi) => {
                        daftar.remove(posisi);
                        *item.borrow_mut() = daftar;
                        Ok(Nilai::Kosong)
                    }
                    None => Err(self.lempar_pesan("KesalahanKunci", "nilai tidak ditemukan di himpunan", span)),
                }
            }
            "buang" | "discard" => {
                let nilai = self.satu_arg(&args, nama, span)?;
                let mut daftar = item.borrow().clone();
                if let Some(posisi) = self.posisi_dalam(&daftar, &nilai)? {
                    daftar.remove(posisi);
                }
                *item.borrow_mut() = daftar;
                Ok(Nilai::Kosong)
            }
            "ambil" | "pop" => {
                let mut daftar = item.borrow_mut();
                if daftar.is_empty() {
                    return Err(self.lempar_pesan("KesalahanKunci", "himpunan kosong", span));
                }
                Ok(daftar.remove(0))
            }
            "bersihkan" | "clear" => {
                item.borrow_mut().clear();
                Ok(Nilai::Kosong)
            }
            "gabung" | "union" => {
                let mut hasil = item.borrow().clone();
                for nilai in args {
                    for anggota in self.iter_nilai(&nilai, span)? {
                        self.tambah_himpunan(&mut hasil, anggota);
                    }
                }
                Ok(Nilai::himpunan(hasil))
            }
            "perbarui" | "update" => {
                let mut daftar = item.borrow().clone();
                for nilai in args {
                    for anggota in self.iter_nilai(&nilai, span)? {
                        self.tambah_himpunan(&mut daftar, anggota);
                    }
                }
                *item.borrow_mut() = daftar;
                Ok(Nilai::Kosong)
            }
            "irisan" | "intersection" => {
                let mut hasil = item.borrow().clone();
                for nilai in args {
                    let lain = self.iter_nilai(&nilai, span)?;
                    let mut baru = Vec::new();
                    for anggota in &hasil {
                        if self.urutan_mengandung_publik(&lain, anggota)? {
                            baru.push(anggota.clone());
                        }
                    }
                    hasil = baru;
                }
                Ok(Nilai::himpunan(hasil))
            }
            "irisan_perbarui" | "intersection_update" => {
                let hasil = self.metode_himpunan(item, "irisan", args, Vec::new(), span)?;
                if let Nilai::Himpunan(baru) = &hasil {
                    *item.borrow_mut() = baru.borrow().clone();
                }
                Ok(Nilai::Kosong)
            }
            "selisih" | "difference" => {
                let mut hasil = item.borrow().clone();
                for nilai in args {
                    let lain = self.iter_nilai(&nilai, span)?;
                    let mut baru = Vec::new();
                    for anggota in &hasil {
                        if !self.urutan_mengandung_publik(&lain, anggota)? {
                            baru.push(anggota.clone());
                        }
                    }
                    hasil = baru;
                }
                Ok(Nilai::himpunan(hasil))
            }
            "selisih_perbarui" | "difference_update" => {
                let hasil = self.metode_himpunan(item, "selisih", args, Vec::new(), span)?;
                if let Nilai::Himpunan(baru) = &hasil {
                    *item.borrow_mut() = baru.borrow().clone();
                }
                Ok(Nilai::Kosong)
            }
            "selisih_simetris" | "symmetric_difference" => {
                let nilai = self.satu_arg(&args, nama, span)?;
                let lain = self.iter_nilai(&nilai, span)?;
                let asli = item.borrow().clone();
                let mut hasil = Vec::new();
                for anggota in &asli {
                    if !self.urutan_mengandung_publik(&lain, anggota)? {
                        hasil.push(anggota.clone());
                    }
                }
                for anggota in &lain {
                    if !self.urutan_mengandung_publik(&asli, anggota)? {
                        hasil.push(anggota.clone());
                    }
                }
                Ok(Nilai::himpunan(hasil))
            }
            "selisih_simetris_perbarui" | "symmetric_difference_update" => {
                let hasil = self.metode_himpunan(item, "selisih_simetris", args, Vec::new(), span)?;
                if let Nilai::Himpunan(baru) = &hasil {
                    *item.borrow_mut() = baru.borrow().clone();
                }
                Ok(Nilai::Kosong)
            }
            "salin" | "copy" => Ok(Nilai::himpunan(item.borrow().clone())),
            "adalah_subhimpunan" | "issubset" => {
                let nilai = self.satu_arg(&args, nama, span)?;
                let lain = self.iter_nilai(&nilai, span)?;
                for anggota in item.borrow().iter() {
                    if !self.urutan_mengandung_publik(&lain, anggota)? {
                        return Ok(Nilai::Logika(false));
                    }
                }
                Ok(Nilai::Logika(true))
            }
            "adalah_superhimpunan" | "issuperset" => {
                let nilai = self.satu_arg(&args, nama, span)?;
                let lain = self.iter_nilai(&nilai, span)?;
                for anggota in &lain {
                    if !self.urutan_mengandung_publik(&item.borrow(), anggota)? {
                        return Ok(Nilai::Logika(false));
                    }
                }
                Ok(Nilai::Logika(true))
            }
            "terpisah" | "isdisjoint" => {
                let nilai = self.satu_arg(&args, nama, span)?;
                let lain = self.iter_nilai(&nilai, span)?;
                for anggota in item.borrow().iter() {
                    if self.urutan_mengandung_publik(&lain, anggota)? {
                        return Ok(Nilai::Logika(false));
                    }
                }
                Ok(Nilai::Logika(true))
            }
            _ => Err(self.metode_tidak_ada(nama, "himpunan", span)),
        }
    }

    fn posisi_dalam(&mut self, daftar: &[Nilai], nilai: &Nilai) -> Hasil<Option<usize>> {
        for (i, kandidat) in daftar.iter().enumerate() {
            if self.sama(kandidat, nilai)? {
                return Ok(Some(i));
            }
        }
        Ok(None)
    }

    // ------------------------------------------------------------------
    // Kamus
    // ------------------------------------------------------------------

    fn metode_kamus(
        &mut self,
        item: &Rc<std::cell::RefCell<Vec<(Nilai, Nilai)>>>,
        nama: &str,
        args: Vec<Nilai>,
        kwargs: Vec<(String, Nilai)>,
        span: Span,
    ) -> Hasil<Nilai> {
        match nama {
            "kunci" | "keys" => Ok(Nilai::daftar(
                item.borrow().iter().map(|(k, _)| k.clone()).collect(),
            )),
            "nilai" | "values" => Ok(Nilai::daftar(
                item.borrow().iter().map(|(_, v)| v.clone()).collect(),
            )),
            "item" | "items" => Ok(Nilai::daftar(
                item.borrow()
                    .iter()
                    .map(|(k, v)| Nilai::tupel(vec![k.clone(), v.clone()]))
                    .collect(),
            )),
            "ambil" | "get" => {
                if args.is_empty() || args.len() > 2 {
                    return Err(self.lempar_pesan(
                        "KesalahanTipe",
                        format!("`{nama}` menerima satu atau dua argumen"),
                        span,
                    ));
                }
                let kunci = args[0].clone();
                let bawaan = args.get(1).cloned().unwrap_or(Nilai::Kosong);
                let pasangan = item.borrow().clone();
                for (k, v) in pasangan {
                    if self.sama(&k, &kunci)? {
                        return Ok(v);
                    }
                }
                Ok(bawaan)
            }
            "perbarui" | "update" => {
                if let Some(nilai) = args.first() {
                    for (kunci, nilai) in self.kamus_pasangan(nilai, span)? {
                        let mut pasangan = item.borrow().clone();
                        self.atur_kamus(&mut pasangan, kunci, nilai);
                        *item.borrow_mut() = pasangan;
                    }
                }
                for (kunci, nilai) in kwargs {
                    let mut pasangan = item.borrow().clone();
                    self.atur_kamus(&mut pasangan, Nilai::teks(kunci), nilai);
                    *item.borrow_mut() = pasangan;
                }
                Ok(Nilai::Kosong)
            }
            "hapus" | "pop" => {
                if args.is_empty() || args.len() > 2 {
                    return Err(self.lempar_pesan(
                        "KesalahanTipe",
                        format!("`{nama}` menerima satu atau dua argumen"),
                        span,
                    ));
                }
                let kunci = args[0].clone();
                let pasangan = item.borrow().clone();
                for (i, (k, v)) in pasangan.iter().enumerate() {
                    if self.sama(k, &kunci)? {
                        item.borrow_mut().remove(i);
                        return Ok(v.clone());
                    }
                }
                if let Some(bawaan) = args.get(1) {
                    Ok(bawaan.clone())
                } else {
                    Err(self.lempar_pesan("KesalahanKunci", "kunci tidak ditemukan", span))
                }
            }
            "salin" | "copy" => Ok(Nilai::kamus(item.borrow().clone())),
            "bersihkan" | "clear" => {
                item.borrow_mut().clear();
                Ok(Nilai::Kosong)
            }
            "set_default" | "setdefault" => {
                if args.is_empty() || args.len() > 2 {
                    return Err(self.lempar_pesan(
                        "KesalahanTipe",
                        format!("`{nama}` menerima satu atau dua argumen"),
                        span,
                    ));
                }
                let kunci = args[0].clone();
                let bawaan = args.get(1).cloned().unwrap_or(Nilai::Kosong);
                let pasangan = item.borrow().clone();
                for (k, v) in pasangan {
                    if self.sama(&k, &kunci)? {
                        return Ok(v);
                    }
                }
                item.borrow_mut().push((kunci, bawaan.clone()));
                Ok(bawaan)
            }
            "popitem" => {
                let mut pasangan = item.borrow_mut();
                match pasangan.pop() {
                    Some((k, v)) => Ok(Nilai::tupel(vec![k, v])),
                    None => Err(self.lempar_pesan("KesalahanKunci", "kamus kosong", span)),
                }
            }
            _ => Err(self.metode_tidak_ada(nama, "kamus", span)),
        }
    }

    // ------------------------------------------------------------------
    // Berkas
    // ------------------------------------------------------------------

    fn metode_berkas(
        &mut self,
        berkas: &Rc<std::cell::RefCell<Berkas>>,
        nama: &str,
        args: Vec<Nilai>,
        _kwargs: Vec<(String, Nilai)>,
        span: Span,
    ) -> Hasil<Nilai> {
        match nama {
            "tutup" | "close" => {
                let mut berkas = berkas.borrow_mut();
                berkas.berkas.take();
                Ok(Nilai::Kosong)
            }
            "baca" | "read" => {
                let ukuran = args
                    .first()
                    .map(|nilai| self.nilai_ke_i64(nilai, span))
                    .transpose()?
                    .unwrap_or(-1);
                let mut berkas = berkas.borrow_mut();
                let biner = berkas.biner;
                let Some(file) = berkas.berkas.as_mut() else {
                    return Err(self.lempar_pesan("KesalahanNilai", "berkas sudah ditutup", span));
                };
                if biner {
                    let mut isi = Vec::new();
                    if ukuran < 0 {
                        file.read_to_end(&mut isi)
                    } else {
                        let mut buf = vec![0u8; ukuran as usize];
                        file.read(&mut buf).map(|n| {
                            isi.extend_from_slice(&buf[..n]);
                            n
                        })
                    }
                    .map_err(|galat| {
                        self.lempar_pesan("KesalahanBerkas", format!("gagal membaca: {galat}"), span)
                    })?;
                    Ok(Nilai::Bytes(Rc::new(isi)))
                } else {
                    let mut isi = String::new();
                    if ukuran < 0 {
                        file.read_to_string(&mut isi)
                    } else {
                        let mut buf = vec![0u8; ukuran as usize];
                        file.read(&mut buf).map(|n| {
                            isi.push_str(&String::from_utf8_lossy(&buf[..n]));
                            n
                        })
                    }
                    .map_err(|galat| {
                        self.lempar_pesan("KesalahanBerkas", format!("gagal membaca: {galat}"), span)
                    })?;
                    Ok(Nilai::teks(isi))
                }
            }
            "baca_baris" | "readline" => {
                let mut berkas = berkas.borrow_mut();
                let Some(file) = berkas.berkas.as_mut() else {
                    return Err(self.lempar_pesan("KesalahanNilai", "berkas sudah ditutup", span));
                };
                let mut bytes = Vec::new();
                let mut buf = [0u8; 1];
                loop {
                    match file.read(&mut buf) {
                        Ok(0) => break,
                        Ok(_) => {
                            bytes.push(buf[0]);
                            if buf[0] == b'\n' {
                                break;
                            }
                        }
                        Err(galat) => {
                            return Err(self.lempar_pesan(
                                "KesalahanBerkas",
                                format!("gagal membaca: {galat}"),
                                span,
                            ))
                        }
                    }
                }
                Ok(Nilai::teks(String::from_utf8_lossy(&bytes).to_string()))
            }
            "baca_semua_baris" | "readlines" => {
                let mut berkas = berkas.borrow_mut();
                let Some(file) = berkas.berkas.as_mut() else {
                    return Err(self.lempar_pesan("KesalahanNilai", "berkas sudah ditutup", span));
                };
                let mut isi = String::new();
                file.read_to_string(&mut isi).map_err(|galat| {
                    self.lempar_pesan("KesalahanBerkas", format!("gagal membaca: {galat}"), span)
                })?;
                Ok(Nilai::daftar(
                    isi.lines()
                        .map(|baris| Nilai::teks(format!("{baris}\n")))
                        .collect(),
                ))
            }
            "tulis" | "write" => {
                let nilai = self.satu_arg(&args, nama, span)?;
                let mut berkas = berkas.borrow_mut();
                let Some(file) = berkas.berkas.as_mut() else {
                    return Err(self.lempar_pesan("KesalahanNilai", "berkas sudah ditutup", span));
                };
                let jumlah = match nilai {
                    Nilai::Teks(teks) => file.write(teks.as_bytes()).map_err(|galat| {
                        self.lempar_pesan("KesalahanBerkas", format!("gagal menulis: {galat}"), span)
                    })?,
                    Nilai::Bytes(bytes) => file.write(&bytes).map_err(|galat| {
                        self.lempar_pesan("KesalahanBerkas", format!("gagal menulis: {galat}"), span)
                    })?,
                    other => {
                        return Err(self.lempar_pesan(
                            "KesalahanTipe",
                            format!("`tulis` membutuhkan teks atau bytes, ditemukan {}", other.nama_tipe()),
                            span,
                        ))
                    }
                };
                Ok(Nilai::Bilangan(jumlah as i64))
            }
            "tulis_baris" | "writelines" => {
                let nilai = self.satu_arg(&args, nama, span)?;
                let item = self.iter_nilai(&nilai, span)?;
                let mut berkas = berkas.borrow_mut();
                let Some(file) = berkas.berkas.as_mut() else {
                    return Err(self.lempar_pesan("KesalahanNilai", "berkas sudah ditutup", span));
                };
                for nilai in item {
                    match nilai {
                        Nilai::Teks(teks) => {
                            file.write_all(teks.as_bytes()).map_err(|galat| {
                                self.lempar_pesan("KesalahanBerkas", format!("gagal menulis: {galat}"), span)
                            })?;
                        }
                        Nilai::Bytes(bytes) => {
                            file.write_all(&bytes).map_err(|galat| {
                                self.lempar_pesan("KesalahanBerkas", format!("gagal menulis: {galat}"), span)
                            })?;
                        }
                        other => {
                            return Err(self.lempar_pesan(
                                "KesalahanTipe",
                                format!("`tulis_baris` menerima teks atau bytes, ditemukan {}", other.nama_tipe()),
                                span,
                            ))
                        }
                    }
                }
                Ok(Nilai::Kosong)
            }
            "cari" | "seek" => {
                let offset = self.arg_i64(&args, nama, span)?;
                let whence = args
                    .get(1)
                    .map(|nilai| self.nilai_ke_i64(nilai, span))
                    .transpose()?
                    .unwrap_or(0);
                let mut berkas = berkas.borrow_mut();
                let Some(file) = berkas.berkas.as_mut() else {
                    return Err(self.lempar_pesan("KesalahanNilai", "berkas sudah ditutup", span));
                };
                let posisi = match whence {
                    0 => SeekFrom::Start(offset as u64),
                    1 => SeekFrom::Current(offset),
                    2 => SeekFrom::End(offset),
                    _ => {
                        return Err(self.lempar_pesan("KesalahanNilai", "whence harus 0, 1, atau 2", span))
                    }
                };
                file.seek(posisi).map_err(|galat| {
                    self.lempar_pesan("KesalahanBerkas", format!("gagal memindahkan posisi: {galat}"), span)
                })?;
                Ok(Nilai::Kosong)
            }
            "posisi" | "tell" => {
                let mut berkas = berkas.borrow_mut();
                let Some(file) = berkas.berkas.as_mut() else {
                    return Err(self.lempar_pesan("KesalahanNilai", "berkas sudah ditutup", span));
                };
                let posisi = file.stream_position().map_err(|galat| {
                    self.lempar_pesan("KesalahanBerkas", format!("gagal membaca posisi: {galat}"), span)
                })?;
                Ok(Nilai::Bilangan(posisi as i64))
            }
            "siram" | "flush" => {
                let mut berkas = berkas.borrow_mut();
                let Some(file) = berkas.berkas.as_mut() else {
                    return Err(self.lempar_pesan("KesalahanNilai", "berkas sudah ditutup", span));
                };
                file.flush().map_err(|galat| {
                    self.lempar_pesan("KesalahanBerkas", format!("gagal menyiram: {galat}"), span)
                })?;
                Ok(Nilai::Kosong)
            }
            "bisa_baca" | "readable" => Ok(Nilai::Logika(
                berkas.borrow().mode.contains('r') || berkas.borrow().mode.contains('+'),
            )),
            "bisa_tulis" | "writable" => Ok(Nilai::Logika(
                berkas.borrow().mode.contains('w')
                    || berkas.borrow().mode.contains('a')
                    || berkas.borrow().mode.contains('+'),
            )),
            "bisa_cari" | "seekable" => Ok(Nilai::Logika(true)),
            _ => Err(self.metode_tidak_ada(nama, "berkas", span)),
        }
    }

    // ------------------------------------------------------------------
    // Generator, rentang, slice
    // ------------------------------------------------------------------

    fn metode_generator(
        &mut self,
        generator: &Rc<std::cell::RefCell<Generator>>,
        nama: &str,
        _args: Vec<Nilai>,
        _kwargs: Vec<(String, Nilai)>,
        span: Span,
    ) -> Hasil<Nilai> {
        match nama {
            "__iter__" => Ok(Nilai::Generator(generator.clone())),
            "__next__" => {
                let mut generator = generator.borrow_mut();
                if generator.indeks < generator.nilai.len() {
                    let nilai = generator.nilai[generator.indeks].clone();
                    generator.indeks += 1;
                    Ok(nilai)
                } else {
                    Err(self.lempar_pesan(
                        "KesalahanHentikanIterasi",
                        "iterasi selesai",
                        span,
                    ))
                }
            }
            "kirim" | "send" => {
                let mut generator = generator.borrow_mut();
                if generator.indeks < generator.nilai.len() {
                    let nilai = generator.nilai[generator.indeks].clone();
                    generator.indeks += 1;
                    Ok(nilai)
                } else {
                    Err(self.lempar_pesan(
                        "KesalahanHentikanIterasi",
                        "iterasi selesai",
                        span,
                    ))
                }
            }
            "tutup" | "close" => {
                let mut generator = generator.borrow_mut();
                generator.indeks = generator.nilai.len();
                Ok(Nilai::Kosong)
            }
            "lempar" | "throw" => Err(self.lempar_pesan(
                "KesalahanRuntime",
                "`lempar` ke generator belum didukung pada interpreter referensi",
                span,
            )),
            _ => Err(self.metode_tidak_ada(nama, "generator", span)),
        }
    }

    fn metode_rentang(
        &mut self,
        rentang: &Rentang,
        nama: &str,
        args: Vec<Nilai>,
        span: Span,
    ) -> Hasil<Nilai> {
        match nama {
            "hitung" | "count" => {
                let nilai = self.satu_arg(&args, nama, span)?;
                let angka = self.nilai_ke_i64(&nilai, span)?;
                if rentang.langkah > 0 {
                    if angka < rentang.mulai || angka >= rentang.akhir {
                        return Ok(Nilai::Bilangan(0));
                    }
                    Ok(Nilai::Bilangan(
                        if (angka - rentang.mulai) % rentang.langkah == 0 { 1 } else { 0 },
                    ))
                } else {
                    if angka > rentang.mulai || angka <= rentang.akhir {
                        return Ok(Nilai::Bilangan(0));
                    }
                    Ok(Nilai::Bilangan(
                        if (rentang.mulai - angka) % (-rentang.langkah) == 0 { 1 } else { 0 },
                    ))
                }
            }
            "indeks" | "index" => {
                let nilai = self.satu_arg(&args, nama, span)?;
                let angka = self.nilai_ke_i64(&nilai, span)?;
                let ada = if rentang.langkah > 0 {
                    angka >= rentang.mulai
                        && angka < rentang.akhir
                        && (angka - rentang.mulai) % rentang.langkah == 0
                } else {
                    angka <= rentang.mulai
                        && angka > rentang.akhir
                        && (rentang.mulai - angka) % (-rentang.langkah) == 0
                };
                if ada {
                    Ok(Nilai::Bilangan(
                        (angka - rentang.mulai) / rentang.langkah,
                    ))
                } else {
                    Err(self.lempar_pesan(
                        "KesalahanNilai",
                        format!("{angka} tidak ada di rentang"),
                        span,
                    ))
                }
            }
            _ => Err(self.metode_tidak_ada(nama, "rentang", span)),
        }
    }

    fn metode_slice(
        &mut self,
        slice: &Rc<Slice>,
        nama: &str,
        args: Vec<Nilai>,
        span: Span,
    ) -> Hasil<Nilai> {
        match nama {
            "indices" => {
                let panjang = self.arg_i64(&args, nama, span)?;
                if panjang < 0 {
                    return Err(self.lempar_pesan("KesalahanNilai", "panjang tidak boleh negatif", span));
                }
                let langkah = slice.langkah.unwrap_or(1);
                if langkah == 0 {
                    return Err(self.lempar_pesan("KesalahanNilai", "langkah slice tidak boleh nol", span));
                }
                let panjang_i = panjang;
                let (mulai, akhir) = if langkah > 0 {
                    (
                        slice.mulai.unwrap_or(0).clamp(0, panjang_i),
                        slice.akhir.unwrap_or(panjang_i).clamp(0, panjang_i),
                    )
                } else {
                    (
                        slice.mulai.unwrap_or(panjang_i - 1).clamp(-1, panjang_i - 1),
                        slice.akhir.unwrap_or(-1).clamp(-1, panjang_i - 1),
                    )
                };
                Ok(Nilai::tupel(vec![
                    Nilai::Bilangan(mulai),
                    Nilai::Bilangan(akhir),
                    Nilai::Bilangan(langkah),
                ]))
            }
            _ => Err(self.metode_tidak_ada(nama, "slice", span)),
        }
    }

    pub fn format_persen(
        &mut self,
        format: &str,
        argumen: &Nilai,
        span: Span,
    ) -> Hasil<Nilai> {
        let args = match argumen {
            Nilai::Tupel(item) => item.as_ref().clone(),
            other => vec![other.clone()],
        };
        let mut hasil = String::new();
        let mut chars = format.chars().peekable();
        let mut indeks = 0usize;
        while let Some(ch) = chars.next() {
            if ch != '%' {
                hasil.push(ch);
                continue;
            }
            if chars.peek() == Some(&'%') {
                chars.next();
                hasil.push('%');
                continue;
            }
            let mut spec = String::new();
            while let Some(&ch) = chars.peek() {
                if ch.is_ascii_alphabetic() {
                    break;
                }
                spec.push(ch);
                chars.next();
            }
            let tipe = chars.next().unwrap_or('s');
            let nilai = args.get(indeks).cloned().unwrap_or(Nilai::Kosong);
            indeks += 1;
            let teks = match tipe {
                's' => nilai_ke_teks(&nilai),
                'r' => nilai_ke_repr(&nilai),
                'd' | 'i' => self.nilai_ke_i64(&nilai, span)?.to_string(),
                'f' => format!("{:.6}", self.nilai_ke_f64(&nilai, span)?),
                'x' => format!("{:x}", self.nilai_ke_i64(&nilai, span)?),
                'X' => format!("{:X}", self.nilai_ke_i64(&nilai, span)?),
                'o' => format!("{:o}", self.nilai_ke_i64(&nilai, span)?),
                'c' => match nilai {
                    Nilai::Teks(teks) => teks.to_string(),
                    other => {
                        let angka = self.nilai_ke_i64(&other, span)?;
                        char::from_u32(angka as u32)
                            .map(|ch| ch.to_string())
                            .unwrap_or_default()
                    }
                },
                _ => {
                    return Err(self.lempar_pesan(
                        "KesalahanNilai",
                        format!("format persen `%{tipe}` tidak dikenal"),
                        span,
                    ))
                }
            };
            hasil.push_str(&teks);
        }
        Ok(Nilai::teks(hasil))
    }
}

fn is_identifier(teks: &str) -> bool {
    let mut chars = teks.chars();
    match chars.next() {
        Some(ch) if ch == '_' || ch.is_alphabetic() => {}
        _ => return false,
    }
    chars.all(|ch| ch == '_' || ch.is_alphanumeric())
}
