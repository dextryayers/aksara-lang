use crate::nilai::{Kesalahan, Nilai};

pub fn repr_teks(teks: &str) -> String {
    // Python memilih kutip tunggal, kecuali teks memuat kutip tunggal tetapi
    // tidak memuat kutip ganda. Aturan ini menjaga `repr` tetap familiar.
    let kutip = if teks.contains('\'') && !teks.contains('"') {
        '"'
    } else {
        '\''
    };
    let mut hasil = String::with_capacity(teks.len() + 2);
    hasil.push(kutip);
    for ch in teks.chars() {
        match ch {
            '\\' => hasil.push_str("\\\\"),
            '\n' => hasil.push_str("\\n"),
            '\r' => hasil.push_str("\\r"),
            '\t' => hasil.push_str("\\t"),
            '\u{0008}' => hasil.push_str("\\b"),
            '\u{000c}' => hasil.push_str("\\f"),
            '\u{000b}' => hasil.push_str("\\v"),
            ch if ch == kutip => {
                hasil.push('\\');
                hasil.push(ch);
            }
            ch if ch.is_control() => {
                use std::fmt::Write as _;
                let _ = write!(hasil, "\\u{:04x}", ch as u32);
            }
            ch => hasil.push(ch),
        }
    }
    hasil.push(kutip);
    hasil
}

pub fn repr_bytes(bytes: &[u8]) -> String {
    let mut hasil = String::with_capacity(bytes.len() + 3);
    hasil.push_str("b'");
    for byte in bytes {
        match byte {
            b'\\' => hasil.push_str("\\\\"),
            b'\'' => hasil.push_str("\\'"),
            b'\n' => hasil.push_str("\\n"),
            b'\r' => hasil.push_str("\\r"),
            b'\t' => hasil.push_str("\\t"),
            0x20..=0x7e => hasil.push(*byte as char),
            byte => {
                use std::fmt::Write as _;
                let _ = write!(hasil, "\\x{byte:02x}");
            }
        }
    }
    hasil.push('\'');
    hasil
}

pub fn repr_float(nilai: f64) -> String {
    if nilai.is_nan() {
        return "nan".to_string();
    }
    if nilai.is_infinite() {
        return if nilai.is_sign_negative() {
            "-inf".to_string()
        } else {
            "inf".to_string()
        };
    }
    let teks = format!("{nilai}");
    if teks.contains('.') || teks.contains('e') || teks.contains('E') {
        teks
    } else {
        format!("{teks}.0")
    }
}

pub fn repr_kompleks(nyata: f64, imajiner: f64) -> String {
    if nyata == 0.0 {
        return format!("{}j", repr_float(imajiner));
    }
    let tanda = if imajiner.is_sign_negative() { "-" } else { "+" };
    format!(
        "({}{}{}j)",
        repr_float(nyata),
        tanda,
        repr_float(imajiner.abs())
    )
}

pub fn teks_kesalahan(kesalahan: &Kesalahan) -> String {
    if kesalahan.argumen.is_empty() {
        kesalahan.tipe.nama.clone()
    } else if kesalahan.argumen.len() == 1 {
        format!(
            "{}: {}",
            kesalahan.tipe.nama,
            crate::format::nilai_ke_teks(&kesalahan.argumen[0])
        )
    } else {
        let args = kesalahan
            .argumen
            .iter()
            .map(nilai_ke_teks)
            .collect::<Vec<_>>()
            .join(", ");
        format!("{}({})", kesalahan.tipe.nama, args)
    }
}

pub fn nilai_ke_teks(nilai: &Nilai) -> String {
    match nilai {
        Nilai::Kosong => "kosong".to_string(),
        Nilai::Logika(true) => "benar".to_string(),
        Nilai::Logika(false) => "salah".to_string(),
        Nilai::Bilangan(nilai) => nilai.to_string(),
        Nilai::Pecahan(nilai) => repr_float(*nilai),
        Nilai::Kompleks(nyata, imajiner) => repr_kompleks(*nyata, *imajiner),
        Nilai::Teks(teks) => teks.to_string(),
        Nilai::Bytes(bytes) => repr_bytes(bytes),
        Nilai::Daftar(nilai) => {
            let isi = nilai
                .borrow()
                .iter()
                .map(nilai_ke_repr)
                .collect::<Vec<_>>()
                .join(", ");
            format!("[{isi}]")
        }
        Nilai::Tupel(nilai) => {
            let isi = nilai
                .iter()
                .map(nilai_ke_repr)
                .collect::<Vec<_>>()
                .join(", ");
            if nilai.len() == 1 {
                format!("({isi},)")
            } else {
                format!("({isi})")
            }
        }
        Nilai::Himpunan(nilai) => {
            let isi = nilai
                .borrow()
                .iter()
                .map(nilai_ke_repr)
                .collect::<Vec<_>>()
                .join(", ");
            if isi.is_empty() {
                "himpunan()".to_string()
            } else {
                format!("{{{isi}}}")
            }
        }
        Nilai::Kamus(nilai) => {
            let isi = nilai
                .borrow()
                .iter()
                .map(|(kunci, nilai)| format!("{}: {}", nilai_ke_repr(kunci), nilai_ke_repr(nilai)))
                .collect::<Vec<_>>()
                .join(", ");
            format!("{{{isi}}}")
        }
        Nilai::Rentang(rentang) => format!(
            "rentang({}, {}, {})",
            rentang.mulai, rentang.akhir, rentang.langkah
        ),
        Nilai::Slice(slice) => format!(
            "slice({}, {}, {})",
            slice
                .mulai
                .map(|nilai| nilai.to_string())
                .unwrap_or_else(|| "kosong".to_string()),
            slice
                .akhir
                .map(|nilai| nilai.to_string())
                .unwrap_or_else(|| "kosong".to_string()),
            slice
                .langkah
                .map(|nilai| nilai.to_string())
                .unwrap_or_else(|| "kosong".to_string())
        ),
        Nilai::Fungsi(fungsi) => format!("<fungsi {}>", fungsi.nama),
        Nilai::Bawaan(bawaan) => format!("<fungsi bawaan {}>", bawaan.nama),
        Nilai::BawaanKhusus(bawaan) => format!("<fungsi bawaan {}>", bawaan.nama),
        Nilai::Kelas(kelas) => format!("<kelas '{}'>", kelas.nama),
        Nilai::Instance(instance) => {
            format!("<{} objek>", instance.kelas.nama)
        }
        Nilai::Metode(metode) => format!("<metode {}>", metode.fungsi.nama),
        Nilai::MetodeBawaan(metode) => format!("<metode {}>", metode.nama),
        Nilai::Properti(_) => "<properti>".to_string(),
        Nilai::MetodeStatis(fungsi) => format!("<fungsi {}>", fungsi.nama),
        Nilai::MetodeKelas(metode) => format!("<metode kelas {}>", metode.fungsi.nama),
        Nilai::Modul(modul) => format!("<modul '{}'>", modul.nama),
        Nilai::Generator(generator) => {
            format!("<generator {}>", generator.borrow().nama)
        }
        Nilai::Kesalahan(kesalahan) => teks_kesalahan(kesalahan),
        Nilai::Induk(_) => "<induk>".to_string(),
        Nilai::Berkas(berkas) => format!("<berkas '{}'>", berkas.borrow().nama),
        Nilai::Python(_) => crate::python_teks(nilai),
        Nilai::Ellipsis => "Ellipsis".to_string(),
    }
}

pub fn nilai_ke_repr(nilai: &Nilai) -> String {
    match nilai {
        Nilai::Teks(teks) => repr_teks(teks),
        Nilai::Bytes(bytes) => repr_bytes(bytes),
        Nilai::Pecahan(nilai) => repr_float(*nilai),
        Nilai::Kompleks(nyata, imajiner) => repr_kompleks(*nyata, *imajiner),
        Nilai::Python(_) => crate::python_repr(nilai),
        other => nilai_ke_teks(other),
    }
}

/// Format angka dengan spesifikasi gaya Python yang umum.
pub fn format_nilai(nilai: &Nilai, spec: &str) -> Result<String, String> {
    if spec.is_empty() {
        return Ok(nilai_ke_teks(nilai));
    }
    let mut sisa = spec;
    let mut kiri = false;
    let mut kanan = false;
    let mut tengah = false;
    let mut isi = ' ';
    let mut tanda = None;
    let mut alternatif = false;
    let mut nol = false;
    let mut lebar = None;
    let mut presisi = None;
    let mut tipe = None;

    let chars: Vec<char> = sisa.chars().collect();
    let mut index = 0;
    if index + 1 < chars.len() && matches!(chars[index + 1], '<' | '>' | '^' | '=') {
        isi = chars[index];
        index += 1;
    }
    if index < chars.len() {
        match chars[index] {
            '<' => {
                kiri = true;
                index += 1;
            }
            '>' => {
                kanan = true;
                index += 1;
            }
            '^' => {
                tengah = true;
                index += 1;
            }
            '=' => {
                index += 1;
            }
            _ => {}
        }
    }
    if index < chars.len() && matches!(chars[index], '+' | '-' | ' ') {
        tanda = Some(chars[index]);
        index += 1;
    }
    if index < chars.len() && chars[index] == '#' {
        alternatif = true;
        index += 1;
    }
    if index < chars.len() && chars[index] == '0' {
        nol = true;
        index += 1;
    }
    let mulai_lebar = index;
    while index < chars.len() && chars[index].is_ascii_digit() {
        index += 1;
    }
    if index > mulai_lebar {
        lebar = chars[mulai_lebar..index]
            .iter()
            .collect::<String>()
            .parse::<usize>()
            .ok();
    }
    if index < chars.len() && chars[index] == ',' {
        index += 1;
    } else if index < chars.len() && chars[index] == '_' {
        index += 1;
    }
    if index < chars.len() && chars[index] == '.' {
        index += 1;
        let mulai = index;
        while index < chars.len() && chars[index].is_ascii_digit() {
            index += 1;
        }
        if index > mulai {
            presisi = chars[mulai..index]
                .iter()
                .collect::<String>()
                .parse::<usize>()
                .ok();
        }
    }
    if index < chars.len() {
        tipe = Some(chars[index]);
        index += 1;
    }
    sisa = &spec[index..];
    if !sisa.is_empty() {
        return Err(format!("spesifikasi format tidak dikenal: `{spec}`"));
    }

    let mut hasil = match (nilai, tipe) {
        (Nilai::Teks(teks), Some('s') | None) => teks.to_string(),
        (Nilai::Teks(teks), Some('r')) => repr_teks(teks),
        (Nilai::Bytes(bytes), Some('s') | None) => String::from_utf8_lossy(bytes).into_owned(),
        (Nilai::Bilangan(angka), Some('d') | None) => {
            let mut teks = angka.to_string();
            if tanda == Some('+') && *angka >= 0 {
                teks.insert(0, '+');
            } else if tanda == Some(' ') && *angka >= 0 {
                teks.insert(0, ' ');
            }
            if alternatif && *angka != 0 {
                let _ = alternatif;
            }
            teks
        }
        (Nilai::Bilangan(angka), Some('x')) => {
            if alternatif {
                format!("{angka:#x}")
            } else {
                format!("{angka:x}")
            }
        }
        (Nilai::Bilangan(angka), Some('X')) => {
            if alternatif {
                format!("{angka:#X}")
            } else {
                format!("{angka:X}")
            }
        }
        (Nilai::Bilangan(angka), Some('o')) => {
            if alternatif {
                format!("{angka:#o}")
            } else {
                format!("{angka:o}")
            }
        }
        (Nilai::Bilangan(angka), Some('b')) => {
            if alternatif {
                format!("{angka:#b}")
            } else {
                format!("{angka:b}")
            }
        }
        (Nilai::Pecahan(angka), Some('f') | Some('F') | None) => {
            format!("{:.*}", presisi.unwrap_or(6), angka)
        }
        (Nilai::Pecahan(angka), Some('e')) => {
            format!("{:.*e}", presisi.unwrap_or(6), angka)
        }
        (Nilai::Pecahan(angka), Some('E')) => {
            format!("{:.*E}", presisi.unwrap_or(6), angka)
        }
        (Nilai::Pecahan(angka), Some('g')) => {
            format!("{:.*}", presisi.unwrap_or(6), angka)
        }
        (Nilai::Pecahan(angka), Some('%')) => {
            format!("{:.*}%", presisi.unwrap_or(6), angka * 100.0)
        }
        (Nilai::Bilangan(angka), Some('e')) => {
            format!("{:.*e}", presisi.unwrap_or(6), *angka as f64)
        }
        (Nilai::Bilangan(angka), Some('g')) => {
            format!("{:.*}", presisi.unwrap_or(6), *angka as f64)
        }
        (_, Some(tipe)) => {
            return Err(format!("tipe format `{tipe}` tidak cocok untuk nilai ini"));
        }
        (_, None) => nilai_ke_teks(nilai),
    };

    if let Some(lebar) = lebar {
        let panjang = hasil.chars().count();
        if panjang < lebar {
            let selisih = lebar - panjang;
            if nol && !kiri && !kanan && !tengah {
                let tanda_awal = hasil
                    .chars()
                    .next()
                    .filter(|ch| matches!(ch, '+' | '-' | ' '))
                    .map(|ch| ch.to_string())
                    .unwrap_or_default();
                if tanda_awal.is_empty() {
                    hasil = format!("{}{}", "0".repeat(selisih), hasil);
                } else {
                    hasil = format!(
                        "{}{}{}",
                        tanda_awal,
                        "0".repeat(selisih),
                        &hasil[tanda_awal.len()..]
                    );
                }
            } else if kiri {
                hasil.push_str(&isi.to_string().repeat(selisih));
            } else if tengah {
                let kiri_n = selisih / 2;
                let kanan_n = selisih - kiri_n;
                hasil = format!(
                    "{}{}{}",
                    isi.to_string().repeat(kiri_n),
                    hasil,
                    isi.to_string().repeat(kanan_n)
                );
            } else {
                hasil = format!("{}{}", isi.to_string().repeat(selisih), hasil);
            }
        }
    }
    Ok(hasil)
}

pub fn kesalahan_format(pesan: impl Into<String>) -> String {
    pesan.into()
}
