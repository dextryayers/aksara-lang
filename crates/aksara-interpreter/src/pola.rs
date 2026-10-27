//! Pencocokan pola untuk pernyataan `cocokkan`.

use std::collections::HashMap;

use aksara_ast::{Expr, ExprKind, Pattern};

use crate::bawaan::jenis_dari_nilai;
use crate::interpreter::Interpreter;
use crate::nilai::*;

pub fn cocokkan(
    interpreter: &mut Interpreter,
    pola: &Pattern,
    nilai: &Nilai,
    ikatan: &mut HashMap<String, Nilai>,
    env: &Env,
) -> Hasil<bool> {
    match pola {
        Pattern::Wildcard => Ok(true),
        Pattern::Capture(nama) => {
            if nama != "_" {
                ikatan.insert(nama.clone(), nilai.clone());
            }
            Ok(true)
        }
        Pattern::Literal(expr) => {
            let pola_nilai = interpreter.eval(expr, env)?;
            if !literal_cocok(expr, &pola_nilai, nilai) {
                return Ok(false);
            }
            interpreter.sama(&pola_nilai, nilai)
        }
        Pattern::Star(_) => Ok(false),
        Pattern::Sequence(urutan) => cocok_urutan(interpreter, urutan, nilai, ikatan, env),
        Pattern::Mapping { items, rest } => {
            cocok_kamus(interpreter, items, rest.as_deref(), nilai, ikatan, env)
        }
        Pattern::Class {
            name,
            positional,
            keywords,
        } => cocok_kelas(interpreter, name, positional, keywords, nilai, ikatan, env),
        Pattern::Or(alternatif) => {
            for pola in alternatif {
                let mut sementara = ikatan.clone();
                if cocokkan(interpreter, pola, nilai, &mut sementara, env)? {
                    ikatan.extend(sementara);
                    return Ok(true);
                }
            }
            Ok(false)
        }
        Pattern::As { pattern, name } => {
            if cocokkan(interpreter, pattern, nilai, ikatan, env)? {
                ikatan.insert(name.clone(), nilai.clone());
                Ok(true)
            } else {
                Ok(false)
            }
        }
    }
}

/// Literal logika dan bilangan mengikuti pemisahan ketat agar `kasus benar`
/// tidak tertukar dengan `kasus 1`.
fn literal_cocok(expr: &Expr, pola: &Nilai, nilai: &Nilai) -> bool {
    match &expr.kind {
        ExprKind::Bool(_) => matches!(nilai, Nilai::Logika(_)),
        ExprKind::Int(_) | ExprKind::Imaginary(_) => {
            matches!(nilai, Nilai::Bilangan(_) | Nilai::Pecahan(_) | Nilai::Kompleks(_, _))
                && !matches!(nilai, Nilai::Logika(_))
        }
        ExprKind::Unary { operand, .. } => match &operand.kind {
            ExprKind::Int(_) | ExprKind::Imaginary(_) => {
                matches!(
                    nilai,
                    Nilai::Bilangan(_) | Nilai::Pecahan(_) | Nilai::Kompleks(_, _)
                ) && !matches!(nilai, Nilai::Logika(_))
            }
            ExprKind::Float(_) => {
                matches!(nilai, Nilai::Pecahan(_) | Nilai::Bilangan(_))
                    && !matches!(nilai, Nilai::Logika(_))
            }
            _ => true,
        },
        ExprKind::Float(_) => {
            matches!(nilai, Nilai::Pecahan(_) | Nilai::Bilangan(_))
                && !matches!(nilai, Nilai::Logika(_))
        }
        _ => {
            let _ = pola;
            true
        }
    }
}

fn cocok_urutan(
    interpreter: &mut Interpreter,
    pola: &[Pattern],
    nilai: &Nilai,
    ikatan: &mut HashMap<String, Nilai>,
    env: &Env,
) -> Hasil<bool> {
    let item = match nilai {
        Nilai::Daftar(item) => item.borrow().clone(),
        Nilai::Tupel(item) => item.as_ref().clone(),
        Nilai::Rentang(rentang) => rentang.nilai(),
        Nilai::Instance(instance) => {
            if instance.kelas.cari_atribut("__iter__").is_none() {
                return Ok(false);
            }
            interpreter.iter_nilai(nilai, interpreter.span_aktif)?
        }
        // Teks dan bytes sengaja tidak dianggap urutan pada pencocokan pola.
        _ => return Ok(false),
    };
    let bintang = pola
        .iter()
        .position(|pola| matches!(pola, Pattern::Star(_)));
    match bintang {
        None => {
            if item.len() != pola.len() {
                return Ok(false);
            }
            for (pola, nilai) in pola.iter().zip(item.iter()) {
                if !cocokkan(interpreter, pola, nilai, ikatan, env)? {
                    return Ok(false);
                }
            }
            Ok(true)
        }
        Some(posisi) => {
            let minimum = pola.len() - 1;
            if item.len() < minimum {
                return Ok(false);
            }
            let ekor = item.len() - minimum;
            for (indeks, pola_item) in pola.iter().enumerate() {
                if indeks == posisi {
                    if let Pattern::Star(Some(nama)) = pola_item {
                        ikatan.insert(
                            nama.clone(),
                            Nilai::daftar(item[posisi..posisi + ekor].to_vec()),
                        );
                    }
                } else {
                    let nilai = if indeks < posisi {
                        &item[indeks]
                    } else {
                        &item[item.len() - (pola.len() - indeks)]
                    };
                    if !cocokkan(interpreter, pola_item, nilai, ikatan, env)? {
                        return Ok(false);
                    }
                }
            }
            Ok(true)
        }
    }
}

fn cocok_kamus(
    interpreter: &mut Interpreter,
    items: &[(Expr, Pattern)],
    rest: Option<&str>,
    nilai: &Nilai,
    ikatan: &mut HashMap<String, Nilai>,
    env: &Env,
) -> Hasil<bool> {
    let pasangan = match nilai {
        Nilai::Kamus(pasangan) => pasangan.borrow().clone(),
        _ => return Ok(false),
    };
    let mut terpakai = Vec::new();
    for (kunci_expr, pola) in items {
        let kunci = interpreter.eval(kunci_expr, env)?;
        let mut ditemukan = None;
        for (indeks, (kandidat, nilai)) in pasangan.iter().enumerate() {
            if terpakai.contains(&indeks) {
                continue;
            }
            if interpreter.sama(kandidat, &kunci)? {
                ditemukan = Some((indeks, nilai.clone()));
                break;
            }
        }
        let Some((indeks, nilai)) = ditemukan else {
            return Ok(false);
        };
        if !cocokkan(interpreter, pola, &nilai, ikatan, env)? {
            return Ok(false);
        }
        terpakai.push(indeks);
    }
    if let Some(nama) = rest {
        let sisa = pasangan
            .into_iter()
            .enumerate()
            .filter(|(indeks, _)| !terpakai.contains(indeks))
            .map(|(_, pasangan)| pasangan)
            .collect();
        ikatan.insert(nama.to_string(), Nilai::kamus(sisa));
    }
    Ok(true)
}

fn cocok_kelas(
    interpreter: &mut Interpreter,
    nama: &str,
    positional: &[Pattern],
    keywords: &[(String, Pattern)],
    nilai: &Nilai,
    ikatan: &mut HashMap<String, Nilai>,
    env: &Env,
) -> Hasil<bool> {
    let kelas_nilai = cari_nama_bertitik(interpreter, env, nama)?;
    let Nilai::Kelas(kelas) = kelas_nilai else {
        return Err(interpreter.lempar_pesan(
            "KesalahanTipe",
            format!("`{nama}` pada pola kelas bukan kelas"),
            interpreter.span_aktif,
        ));
    };
    let jenis_nilai = match nilai {
        Nilai::Instance(instance) => instance.kelas.clone(),
        Nilai::Kesalahan(kesalahan) => kesalahan.tipe.clone(),
        _ => {
            let jenis = jenis_dari_nilai(nilai);
            if jenis == kelas.jenis {
                kelas.clone()
            } else if jenis == JenisKelas::Logika && kelas.jenis == JenisKelas::Bilangan {
                kelas.clone()
            } else {
                return Ok(false);
            }
        }
    };
    if !jenis_nilai.adalah_subkelas(&kelas) {
        return Ok(false);
    }
    if keywords.is_empty() && positional.len() == 1 && kelas.jenis != JenisKelas::Pengguna {
        return cocokkan(interpreter, &positional[0], nilai, ikatan, env);
    }
    let nama_posisional = match kelas.cari_atribut("__match_args__") {
        Some(Nilai::Tupel(item)) => item
            .iter()
            .filter_map(|nilai| match nilai {
                Nilai::Teks(teks) => Some(teks.to_string()),
                _ => None,
            })
            .collect::<Vec<_>>(),
        Some(Nilai::Daftar(item)) => item
            .borrow()
            .iter()
            .filter_map(|nilai| match nilai {
                Nilai::Teks(teks) => Some(teks.to_string()),
                _ => None,
            })
            .collect::<Vec<_>>(),
        _ => Vec::new(),
    };
    if !positional.is_empty() && nama_posisional.is_empty() {
        return Err(interpreter.lempar_pesan(
            "KesalahanTipe",
            format!(
                "kelas `{}` tidak mendefinisikan `__match_args__` untuk pola posisional",
                kelas.nama
            ),
            interpreter.span_aktif,
        ));
    }
    if positional.len() > nama_posisional.len() {
        return Ok(false);
    }
    for (pola, nama_atribut) in positional.iter().zip(nama_posisional.iter()) {
        let Some(atribut) = ambil_atribut_pola(interpreter, nilai, nama_atribut)? else {
            return Ok(false);
        };
        if !cocokkan(interpreter, pola, &atribut, ikatan, env)? {
            return Ok(false);
        }
    }
    for (nama_atribut, pola) in keywords {
        let Some(atribut) = ambil_atribut_pola(interpreter, nilai, nama_atribut)? else {
            return Ok(false);
        };
        if !cocokkan(interpreter, pola, &atribut, ikatan, env)? {
            return Ok(false);
        }
    }
    Ok(true)
}

fn ambil_atribut_pola(
    interpreter: &mut Interpreter,
    nilai: &Nilai,
    nama: &str,
) -> Hasil<Option<Nilai>> {
    match interpreter.ambil_atribut(nilai, nama, interpreter.span_aktif) {
        Ok(nilai) => Ok(Some(nilai)),
        Err(_) => Ok(None),
    }
}

fn cari_nama_bertitik(
    interpreter: &mut Interpreter,
    env: &Env,
    nama: &str,
) -> Hasil<Nilai> {
    let mut bagian = nama.split('.');
    let pertama = bagian.next().unwrap_or(nama);
    let mut nilai = interpreter.cari_nama(env, pertama, interpreter.span_aktif)?;
    for bagian in bagian {
        nilai = interpreter.ambil_atribut(&nilai, bagian, interpreter.span_aktif)?;
    }
    Ok(nilai)
}

/// Membantu modul membangun `__match_args__` dari daftar nama.
pub fn match_args(interpreter: &mut Interpreter, nama: &[String]) -> Nilai {
    let _ = interpreter;
    Nilai::tupel(nama.iter().map(|nama| Nilai::teks(nama.clone())).collect())
}
