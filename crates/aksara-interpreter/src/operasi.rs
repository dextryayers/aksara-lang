use std::rc::Rc;

use aksara_ast::{AugOp, BinOp, CmpOp, Span, UnaryOp};

use crate::format::{nilai_ke_repr, nilai_ke_teks, repr_float};
use crate::interpreter::Interpreter;
use crate::nilai::*;

impl Interpreter {
    // ------------------------------------------------------------------
    // Kebenaran, kesamaan, dan identitas
    // ------------------------------------------------------------------

    pub fn kebenaran(&mut self, nilai: &Nilai) -> Hasil<bool> {
        match nilai {
            Nilai::Kosong => Ok(false),
            Nilai::Logika(nilai) => Ok(*nilai),
            Nilai::Bilangan(nilai) => Ok(*nilai != 0),
            Nilai::Pecahan(nilai) => Ok(*nilai != 0.0),
            Nilai::Kompleks(nyata, imajiner) => Ok(*nyata != 0.0 || *imajiner != 0.0),
            Nilai::Teks(teks) => Ok(!teks.is_empty()),
            Nilai::Bytes(bytes) => Ok(!bytes.is_empty()),
            Nilai::Daftar(item) => Ok(!item.borrow().is_empty()),
            Nilai::Tupel(item) => Ok(!item.is_empty()),
            Nilai::Himpunan(item) => Ok(!item.borrow().is_empty()),
            Nilai::Kamus(item) => Ok(!item.borrow().is_empty()),
            Nilai::Rentang(rentang) => Ok(rentang.panjang() > 0),
            Nilai::Slice(_) => Ok(true),
            Nilai::Fungsi(_)
            | Nilai::Bawaan(_)
            | Nilai::BawaanKhusus(_)
            | Nilai::Kelas(_)
            | Nilai::Metode(_)
            | Nilai::MetodeBawaan(_)
            | Nilai::MetodeStatis(_)
            | Nilai::MetodeKelas(_)
            | Nilai::Properti(_)
            | Nilai::Modul(_)
            | Nilai::Generator(_)
            | Nilai::Kesalahan(_)
            | Nilai::Induk(_)
            | Nilai::Berkas(_)
            | Nilai::Ellipsis => Ok(true),
            Nilai::Instance(instance) => {
                if let Some(fungsi) = instance.kelas.cari_atribut("__bool__") {
                    let hasil = self.panggil(
                        &fungsi,
                        vec![Nilai::Instance(instance.clone())],
                        Vec::new(),
                        Span::empty(0),
                    )?;
                    return self.kebenaran(&hasil);
                }
                if let Some(fungsi) = instance.kelas.cari_atribut("__len__") {
                    let hasil = self.panggil(
                        &fungsi,
                        vec![Nilai::Instance(instance.clone())],
                        Vec::new(),
                        Span::empty(0),
                    )?;
                    let panjang = self.nilai_ke_i64(&hasil, Span::empty(0))?;
                    return Ok(panjang != 0);
                }
                Ok(true)
            }
            Nilai::Python(_) => crate::python_kebenaran(nilai),
        }
    }

    pub fn sama(&mut self, kiri: &Nilai, kanan: &Nilai) -> Hasil<bool> {
        match (kiri, kanan) {
            (Nilai::Kosong, Nilai::Kosong) => Ok(true),
            (Nilai::Kosong, _) | (_, Nilai::Kosong) => Ok(false),
            (Nilai::Ellipsis, Nilai::Ellipsis) => Ok(true),
            (Nilai::Ellipsis, _) | (_, Nilai::Ellipsis) => Ok(false),
            (Nilai::Logika(a), Nilai::Logika(b)) => Ok(a == b),
            (Nilai::Bilangan(a), Nilai::Bilangan(b)) => Ok(a == b),
            (Nilai::Pecahan(a), Nilai::Pecahan(b)) => Ok(a == b),
            (Nilai::Kompleks(ar, ai), Nilai::Kompleks(br, bi)) => Ok(ar == br && ai == bi),
            (Nilai::Teks(a), Nilai::Teks(b)) => Ok(a == b),
            (Nilai::Bytes(a), Nilai::Bytes(b)) => Ok(a == b),
            (Nilai::Rentang(a), Nilai::Rentang(b)) => Ok(a == b),
            (Nilai::Slice(a), Nilai::Slice(b)) => Ok(a == b),
            (Nilai::Daftar(a), Nilai::Daftar(b)) => {
                let a = a.borrow().clone();
                let b = b.borrow().clone();
                self.sama_urutan(&a, &b)
            }
            (Nilai::Tupel(a), Nilai::Tupel(b)) => self.sama_urutan(a, b),
            (Nilai::Himpunan(a), Nilai::Himpunan(b)) => {
                let a = a.borrow().clone();
                let b = b.borrow().clone();
                self.sama_himpunan(&a, &b)
            }
            (Nilai::Kamus(a), Nilai::Kamus(b)) => {
                let a = a.borrow().clone();
                let b = b.borrow().clone();
                self.sama_kamus(&a, &b)
            }
            (Nilai::Kelas(a), Nilai::Kelas(b)) => Ok(Rc::ptr_eq(a, b)),
            (Nilai::Fungsi(a), Nilai::Fungsi(b)) => Ok(Rc::ptr_eq(a, b)),
            (Nilai::Bawaan(a), Nilai::Bawaan(b)) => Ok(Rc::ptr_eq(a, b)),
            (Nilai::Modul(a), Nilai::Modul(b)) => Ok(Rc::ptr_eq(a, b)),
            (Nilai::Kesalahan(a), Nilai::Kesalahan(b)) => Ok(Rc::ptr_eq(a, b)),
            (Nilai::Berkas(a), Nilai::Berkas(b)) => Ok(Rc::ptr_eq(a, b)),
            (Nilai::Generator(a), Nilai::Generator(b)) => Ok(Rc::ptr_eq(a, b)),
            (Nilai::Python(_), _) | (_, Nilai::Python(_)) => crate::python_sama(kiri, kanan),
            _ if kiri.adalah_angka() && kanan.adalah_angka() => {
                let a = kiri.sebagai_kompleks().unwrap_or((0.0, 0.0));
                let b = kanan.sebagai_kompleks().unwrap_or((0.0, 0.0));
                Ok(a == b)
            }
            (Nilai::Instance(instance), _) => {
                if let Some(fungsi) = instance.kelas.cari_atribut("__eq__") {
                    let hasil = self.panggil(
                        &fungsi,
                        vec![Nilai::Instance(instance.clone()), kanan.clone()],
                        Vec::new(),
                        Span::empty(0),
                    )?;
                    return self.kebenaran(&hasil);
                }
                Ok(false)
            }
            (_, Nilai::Instance(instance)) => {
                if let Some(fungsi) = instance.kelas.cari_atribut("__eq__") {
                    let hasil = self.panggil(
                        &fungsi,
                        vec![Nilai::Instance(instance.clone()), kiri.clone()],
                        Vec::new(),
                        Span::empty(0),
                    )?;
                    return self.kebenaran(&hasil);
                }
                Ok(false)
            }
            _ => Ok(false),
        }
    }

    fn sama_urutan(&mut self, kiri: &[Nilai], kanan: &[Nilai]) -> Hasil<bool> {
        if kiri.len() != kanan.len() {
            return Ok(false);
        }
        for (a, b) in kiri.iter().zip(kanan) {
            if !self.sama(a, b)? {
                return Ok(false);
            }
        }
        Ok(true)
    }

    fn sama_himpunan(&mut self, kiri: &[Nilai], kanan: &[Nilai]) -> Hasil<bool> {
        if kiri.len() != kanan.len() {
            return Ok(false);
        }
        for item in kiri {
            if !self.urutan_mengandung(kanan, item)? {
                return Ok(false);
            }
        }
        Ok(true)
    }

    fn sama_kamus(
        &mut self,
        kiri: &[(Nilai, Nilai)],
        kanan: &[(Nilai, Nilai)],
    ) -> Hasil<bool> {
        if kiri.len() != kanan.len() {
            return Ok(false);
        }
        for (kunci, nilai) in kiri {
            let Some((_, lain)) = kanan.iter().find(|(k, _)| self.sama(kunci, k).unwrap_or(false))
            else {
                return Ok(false);
            };
            if !self.sama(nilai, lain)? {
                return Ok(false);
            }
        }
        Ok(true)
    }

    pub fn identik(&mut self, kiri: &Nilai, kanan: &Nilai) -> Hasil<bool> {
        match (kiri, kanan) {
            (Nilai::Kosong, Nilai::Kosong) => Ok(true),
            (Nilai::Ellipsis, Nilai::Ellipsis) => Ok(true),
            (Nilai::Logika(a), Nilai::Logika(b)) => Ok(a == b),
            (Nilai::Bilangan(a), Nilai::Bilangan(b)) => Ok(a == b),
            (Nilai::Pecahan(a), Nilai::Pecahan(b)) => Ok(a.to_bits() == b.to_bits()),
            (Nilai::Teks(a), Nilai::Teks(b)) => Ok(Rc::ptr_eq(a, b) || a == b),
            (Nilai::Bytes(a), Nilai::Bytes(b)) => Ok(Rc::ptr_eq(a, b) || a == b),
            (Nilai::Daftar(a), Nilai::Daftar(b)) => Ok(Rc::ptr_eq(a, b)),
            (Nilai::Tupel(a), Nilai::Tupel(b)) => Ok(Rc::ptr_eq(a, b) || a == b),
            (Nilai::Himpunan(a), Nilai::Himpunan(b)) => Ok(Rc::ptr_eq(a, b)),
            (Nilai::Kamus(a), Nilai::Kamus(b)) => Ok(Rc::ptr_eq(a, b)),
            (Nilai::Rentang(a), Nilai::Rentang(b)) => Ok(a == b),
            (Nilai::Slice(a), Nilai::Slice(b)) => Ok(Rc::ptr_eq(a, b)),
            (Nilai::Kelas(a), Nilai::Kelas(b)) => Ok(Rc::ptr_eq(a, b)),
            (Nilai::Fungsi(a), Nilai::Fungsi(b)) => Ok(Rc::ptr_eq(a, b)),
            (Nilai::Bawaan(a), Nilai::Bawaan(b)) => Ok(Rc::ptr_eq(a, b)),
            (Nilai::Instance(a), Nilai::Instance(b)) => Ok(Rc::ptr_eq(a, b)),
            (Nilai::Modul(a), Nilai::Modul(b)) => Ok(Rc::ptr_eq(a, b)),
            (Nilai::Kesalahan(a), Nilai::Kesalahan(b)) => Ok(Rc::ptr_eq(a, b)),
            (Nilai::Berkas(a), Nilai::Berkas(b)) => Ok(Rc::ptr_eq(a, b)),
            (Nilai::Generator(a), Nilai::Generator(b)) => Ok(Rc::ptr_eq(a, b)),
            (Nilai::Python(_), _) | (_, Nilai::Python(_)) => crate::python_identik(kiri, kanan),
            _ => Ok(false),
        }
    }

    // ------------------------------------------------------------------
    // Perbandingan
    // ------------------------------------------------------------------

    pub fn bandingkan(
        &mut self,
        op: CmpOp,
        kiri: &Nilai,
        kanan: &Nilai,
        span: Span,
    ) -> Hasil<bool> {
        match op {
            CmpOp::Eq => self.sama(kiri, kanan),
            CmpOp::NotEq => Ok(!self.sama(kiri, kanan)?),
            CmpOp::Is => self.identik(kiri, kanan),
            CmpOp::IsNot => Ok(!self.identik(kiri, kanan)?),
            CmpOp::In => self.mengandung(kanan, kiri, span),
            CmpOp::NotIn => Ok(!self.mengandung(kanan, kiri, span)?),
            CmpOp::Lt | CmpOp::Le | CmpOp::Gt | CmpOp::Ge => {
                if kiri.adalah_angka() && kanan.adalah_angka() {
                    let a = kiri.sebagai_kompleks().unwrap_or((0.0, 0.0));
                    let b = kanan.sebagai_kompleks().unwrap_or((0.0, 0.0));
                    if a.1 != 0.0 || b.1 != 0.0 {
                        return Err(self.lempar_pesan(
                            "KesalahanTipe",
                            "bilangan kompleks tidak dapat diurutkan",
                            span,
                        ));
                    }
                    return Ok(match op {
                        CmpOp::Lt => a.0 < b.0,
                        CmpOp::Le => a.0 <= b.0,
                        CmpOp::Gt => a.0 > b.0,
                        CmpOp::Ge => a.0 >= b.0,
                        _ => unreachable!(),
                    });
                }
                match (kiri, kanan) {
                    (Nilai::Teks(a), Nilai::Teks(b)) => Ok(urut_teks(op, a, b)),
                    (Nilai::Bytes(a), Nilai::Bytes(b)) => Ok(urut_urutan(op, a, b)),
                    (Nilai::Daftar(a), Nilai::Daftar(b)) => {
                        let a = a.borrow().clone();
                        let b = b.borrow().clone();
                        self.urut_nilai(op, &a, &b, span)
                    }
                    (Nilai::Tupel(a), Nilai::Tupel(b)) => self.urut_nilai(op, a, b, span),
                    (Nilai::Himpunan(a), Nilai::Himpunan(b)) => {
                        let a = a.borrow().clone();
                        let b = b.borrow().clone();
                        match op {
                            CmpOp::Lt => self.himpunan_subset(&a, &b, false),
                            CmpOp::Le => self.himpunan_subset(&a, &b, true),
                            CmpOp::Gt => self.himpunan_subset(&b, &a, false),
                            CmpOp::Ge => self.himpunan_subset(&b, &a, true),
                            _ => unreachable!(),
                        }
                    }
                    (Nilai::Instance(instance), _) => {
                        let nama = match op {
                            CmpOp::Lt => "__lt__",
                            CmpOp::Le => "__le__",
                            CmpOp::Gt => "__gt__",
                            CmpOp::Ge => "__ge__",
                            _ => unreachable!(),
                        };
                        if let Some(fungsi) = instance.kelas.cari_atribut(nama) {
                            let hasil = self.panggil(
                                &fungsi,
                                vec![Nilai::Instance(instance.clone()), kanan.clone()],
                                Vec::new(),
                                span,
                            )?;
                            return self.kebenaran(&hasil);
                        }
                        Err(self.lempar_pesan(
                            "KesalahanTipe",
                            format!(
                                "`{}` tidak mendukung perbandingan {}",
                                instance.kelas.nama,
                                nama
                            ),
                            span,
                        ))
                    }
                    (Nilai::Python(_), _) | (_, Nilai::Python(_)) => {
                        crate::python_bandingkan(op, kiri, kanan)
                    }
                    _ => Err(self.lempar_pesan(
                        "KesalahanTipe",
                        format!(
                            "{} dan {} tidak dapat dibandingkan",
                            kiri.nama_tipe(),
                            kanan.nama_tipe()
                        ),
                        span,
                    )),
                }
            }
        }
    }

    fn urut_nilai(
        &mut self,
        op: CmpOp,
        kiri: &[Nilai],
        kanan: &[Nilai],
        span: Span,
    ) -> Hasil<bool> {
        for (a, b) in kiri.iter().zip(kanan) {
            if self.sama(a, b)? {
                continue;
            }
            return self.bandingkan(op, a, b, span);
        }
        let a = kiri.len();
        let b = kanan.len();
        Ok(match op {
            CmpOp::Lt => a < b,
            CmpOp::Le => a <= b,
            CmpOp::Gt => a > b,
            CmpOp::Ge => a >= b,
            _ => false,
        })
    }

    fn himpunan_subset(
        &mut self,
        kiri: &[Nilai],
        kanan: &[Nilai],
        sama: bool,
    ) -> Hasil<bool> {
        for item in kiri {
            if !self.urutan_mengandung(kanan, item)? {
                return Ok(false);
            }
        }
        if sama && kiri.len() != kanan.len() {
            return Ok(false);
        }
        Ok(true)
    }

    pub fn mengandung(&mut self, wadah: &Nilai, item: &Nilai, span: Span) -> Hasil<bool> {
        match wadah {
            Nilai::Teks(teks) => match item {
                Nilai::Teks(cari) => Ok(teks.contains(&**cari)),
                _ => Err(self.lempar_pesan(
                    "KesalahanTipe",
                    "`dalam` pada teks membutuhkan teks",
                    span,
                )),
            },
            Nilai::Bytes(bytes) => match item {
                Nilai::Bytes(cari) => Ok(bytes
                    .windows(cari.len())
                    .any(|jendela| jendela == cari.as_slice())),
                Nilai::Bilangan(angka) => Ok(bytes.contains(&(*angka as u8))),
                _ => Err(self.lempar_pesan(
                    "KesalahanTipe",
                    "`dalam` pada bytes membutuhkan bytes atau bilangan",
                    span,
                )),
            },
            Nilai::Daftar(item_daftar) => {
                let item_daftar = item_daftar.borrow().clone();
                self.urutan_mengandung(&item_daftar, item)
            }
            Nilai::Tupel(item_daftar) => self.urutan_mengandung(item_daftar, item),
            Nilai::Himpunan(item_himpunan) => {
                let item_himpunan = item_himpunan.borrow().clone();
                self.urutan_mengandung(&item_himpunan, item)
            }
            Nilai::Kamus(pasangan) => {
                let pasangan = pasangan.borrow().clone();
                for (kunci, _) in pasangan {
                    if self.sama(&kunci, item)? {
                        return Ok(true);
                    }
                }
                Ok(false)
            }
            Nilai::Rentang(rentang) => {
                let angka = self.nilai_ke_i64(item, span)?;
                if rentang.langkah > 0 {
                    Ok(angka >= rentang.mulai
                        && angka < rentang.akhir
                        && (angka - rentang.mulai) % rentang.langkah == 0)
                } else {
                    Ok(angka <= rentang.mulai
                        && angka > rentang.akhir
                        && (rentang.mulai - angka) % (-rentang.langkah) == 0)
                }
            }
            Nilai::Instance(instance) => {
                if let Some(fungsi) = instance.kelas.cari_atribut("__contains__") {
                    let hasil = self.panggil(
                        &fungsi,
                        vec![Nilai::Instance(instance.clone()), item.clone()],
                        Vec::new(),
                        span,
                    )?;
                    return self.kebenaran(&hasil);
                }
                let iter = self.iter_nilai(wadah, span)?;
                self.urutan_mengandung(&iter, item)
            }
            Nilai::Python(_) => crate::python_mengandung(wadah, item),
            _ => Err(self.lempar_pesan(
                "KesalahanTipe",
                format!("{} tidak mendukung operator `dalam`", wadah.nama_tipe()),
                span,
            )),
        }
    }

    fn urutan_mengandung(&mut self, daftar: &[Nilai], item: &Nilai) -> Hasil<bool> {
        for kandidat in daftar {
            if self.sama(kandidat, item)? {
                return Ok(true);
            }
        }
        Ok(false)
    }

    // ------------------------------------------------------------------
    // Iterasi dan panjang
    // ------------------------------------------------------------------

    pub fn iter_nilai(&mut self, nilai: &Nilai, span: Span) -> Hasil<Vec<Nilai>> {
        match nilai {
            Nilai::Daftar(item) => Ok(item.borrow().clone()),
            Nilai::Tupel(item) => Ok(item.as_ref().clone()),
            Nilai::Himpunan(item) => Ok(item.borrow().clone()),
            Nilai::Kamus(pasangan) => Ok(pasangan
                .borrow()
                .iter()
                .map(|(kunci, _)| kunci.clone())
                .collect()),
            Nilai::Teks(teks) => Ok(teks.chars().map(|ch| Nilai::teks(ch.to_string())).collect()),
            Nilai::Bytes(bytes) => Ok(bytes
                .iter()
                .map(|byte| Nilai::Bilangan(i64::from(*byte)))
                .collect()),
            Nilai::Rentang(rentang) => Ok(rentang.nilai()),
            Nilai::Generator(generator) => {
                let mut generator = generator.borrow_mut();
                let hasil = generator.nilai[generator.indeks..].to_vec();
                generator.indeks = generator.nilai.len();
                Ok(hasil)
            }
            Nilai::Berkas(berkas) => {
                let mut berkas = berkas.borrow_mut();
                let Some(file) = berkas.berkas.as_mut() else {
                    return Err(self.lempar_pesan(
                        "KesalahanNilai",
                        "berkas sudah ditutup",
                        span,
                    ));
                };
                use std::io::Read;
                let mut isi = String::new();
                if let Err(galat) = file.read_to_string(&mut isi) {
                    return Err(self.lempar_pesan(
                        "KesalahanBerkas",
                        format!("gagal membaca berkas: {galat}"),
                        span,
                    ));
                }
                Ok(isi
                    .lines()
                    .map(|baris| Nilai::teks(format!("{baris}\n")))
                    .collect())
            }
            Nilai::Instance(instance) => {
                if let Some(fungsi) = instance.kelas.cari_atribut("__iter__") {
                    let iterator = self.panggil(
                        &fungsi,
                        vec![Nilai::Instance(instance.clone())],
                        Vec::new(),
                        span,
                    )?;
                    if let Nilai::Instance(sama) = &iterator {
                        if Rc::ptr_eq(sama, instance) {
                            let mut hasil = Vec::new();
                            loop {
                                if hasil.len() > 1_000_000 {
                                    return Err(self.lempar_pesan(
                                        "KesalahanRuntime",
                                        "iterasi melebihi satu juta elemen",
                                        span,
                                    ));
                                }
                                match self.panggil(
                                    &self.ambil_atribut(&iterator, "__next__", span)?,
                                    Vec::new(),
                                    Vec::new(),
                                    span,
                                ) {
                                    Ok(nilai) => hasil.push(nilai),
                                    Err(lemparan)
                                        if lemparan.kesalahan.tipe.nama
                                            == "KesalahanHentikanIterasi" =>
                                    {
                                        break
                                    }
                                    Err(lemparan) => return Err(lemparan),
                                }
                            }
                            return Ok(hasil);
                        }
                    }
                    return self.iter_nilai(&iterator, span);
                }
                Err(self.lempar_pesan(
                    "KesalahanTipe",
                    format!("`{}` tidak dapat diiterasi", instance.kelas.nama),
                    span,
                ))
            }
            Nilai::Python(_) => crate::python_iter(nilai),
            _ => Err(self.lempar_pesan(
                "KesalahanTipe",
                format!("{} tidak dapat diiterasi", nilai.nama_tipe()),
                span,
            )),
        }
    }

    pub fn panjang_nilai(&mut self, nilai: &Nilai, span: Span) -> Hasil<usize> {
        match nilai {
            Nilai::Teks(teks) => Ok(teks.chars().count()),
            Nilai::Bytes(bytes) => Ok(bytes.len()),
            Nilai::Daftar(item) => Ok(item.borrow().len()),
            Nilai::Tupel(item) => Ok(item.len()),
            Nilai::Himpunan(item) => Ok(item.borrow().len()),
            Nilai::Kamus(item) => Ok(item.borrow().len()),
            Nilai::Rentang(rentang) => Ok(rentang.panjang()),
            Nilai::Instance(instance) => {
                if let Some(fungsi) = instance.kelas.cari_atribut("__len__") {
                    let hasil = self.panggil(
                        &fungsi,
                        vec![Nilai::Instance(instance.clone())],
                        Vec::new(),
                        span,
                    )?;
                    let panjang = self.nilai_ke_i64(&hasil, span)?;
                    if panjang < 0 {
                        return Err(self.lempar_pesan(
                            "KesalahanNilai",
                            "`__len__` tidak boleh mengembalikan nilai negatif",
                            span,
                        ));
                    }
                    return Ok(panjang as usize);
                }
                Err(self.lempar_pesan(
                    "KesalahanTipe",
                    format!("`{}` tidak memiliki panjang", instance.kelas.nama),
                    span,
                ))
            }
            Nilai::Python(_) => crate::python_panjang(nilai),
            _ => Err(self.lempar_pesan(
                "KesalahanTipe",
                format!("{} tidak memiliki panjang", nilai.nama_tipe()),
                span,
            )),
        }
    }
}

fn urut_teks(op: CmpOp, kiri: &str, kanan: &str) -> bool {
    match op {
        CmpOp::Lt => kiri < kanan,
        CmpOp::Le => kiri <= kanan,
        CmpOp::Gt => kiri > kanan,
        CmpOp::Ge => kiri >= kanan,
        _ => false,
    }
}

fn urut_urutan<T: Ord>(op: CmpOp, kiri: &[T], kanan: &[T]) -> bool {
    match op {
        CmpOp::Lt => kiri < kanan,
        CmpOp::Le => kiri <= kanan,
        CmpOp::Gt => kiri > kanan,
        CmpOp::Ge => kiri >= kanan,
        _ => false,
    }
}

impl Interpreter {
    // ------------------------------------------------------------------
    // Operator biner dan uner
    // ------------------------------------------------------------------

    pub fn operasi_biner(
        &mut self,
        op: BinOp,
        kiri: Nilai,
        kanan: Nilai,
        span: Span,
    ) -> Hasil<Nilai> {
        match op {
            BinOp::Add => self.tambah(kiri, kanan, span),
            BinOp::Sub => self.kurang(kiri, kanan, span),
            BinOp::Mul => self.kali(kiri, kanan, span),
            BinOp::Div => self.bagi(kiri, kanan, span),
            BinOp::FloorDiv => self.bagi_bulat(kiri, kanan, span),
            BinOp::Mod => self.modulo(kiri, kanan, span),
            BinOp::Pow => self.pangkat(kiri, kanan, span),
            BinOp::MatMul => self.kali_matriks(kiri, kanan, span),
            BinOp::LShift => self.geser(op, kiri, kanan, span),
            BinOp::RShift => self.geser(op, kiri, kanan, span),
            BinOp::BitAnd => self.bit(op, kiri, kanan, span),
            BinOp::BitOr => self.bit(op, kiri, kanan, span),
            BinOp::BitXor => self.bit(op, kiri, kanan, span),
            BinOp::NullCoalesce => {
                if kiri.adalah_kosong() {
                    Ok(kanan)
                } else {
                    Ok(kiri)
                }
            }
        }
    }

    fn tambah(&mut self, kiri: Nilai, kanan: Nilai, span: Span) -> Hasil<Nilai> {
        match (&kiri, &kanan) {
            (Nilai::Teks(a), Nilai::Teks(b)) => {
                let mut hasil = String::with_capacity(a.len() + b.len());
                hasil.push_str(a);
                hasil.push_str(b);
                Ok(Nilai::teks(hasil))
            }
            (Nilai::Bytes(a), Nilai::Bytes(b)) => {
                let mut hasil = a.as_ref().clone();
                hasil.extend_from_slice(b);
                Ok(Nilai::Bytes(Rc::new(hasil)))
            }
            (Nilai::Daftar(a), Nilai::Daftar(b)) => {
                let mut hasil = a.borrow().clone();
                hasil.extend(b.borrow().iter().cloned());
                Ok(Nilai::daftar(hasil))
            }
            (Nilai::Tupel(a), Nilai::Tupel(b)) => {
                let mut hasil = a.as_ref().clone();
                hasil.extend(b.iter().cloned());
                Ok(Nilai::tupel(hasil))
            }
            _ if kiri.adalah_angka() && kanan.adalah_angka() => {
                self.angka_biner(op_add(), &kiri, &kanan, span)
            }
            (Nilai::Instance(_), _) => {
                if let Some(hasil) = self.dunder_biner(&kiri, "__add__", &kanan, span)? {
                    return Ok(hasil);
                }
                self.dunder_biner_kanan(&kanan, "__radd__", &kiri, span)
            }
            (_, Nilai::Instance(_)) => {
                self.dunder_biner_kanan(&kanan, "__radd__", &kiri, span)
            }
            (Nilai::Python(_), _) | (_, Nilai::Python(_)) => {
                crate::python_operasi(op_add(), &kiri, &kanan)
            }
            _ => Err(self.tipe_operator("+", &kiri, &kanan, span)),
        }
    }

    fn kurang(&mut self, kiri: Nilai, kanan: Nilai, span: Span) -> Hasil<Nilai> {
        match (&kiri, &kanan) {
            (Nilai::Himpunan(a), Nilai::Himpunan(b)) => {
                let a = a.borrow().clone();
                let b = b.borrow().clone();
                let mut hasil = Vec::new();
                for item in a {
                    if !self.urutan_mengandung_publik(&b, &item)? {
                        hasil.push(item);
                    }
                }
                Ok(Nilai::himpunan(hasil))
            }
            _ if kiri.adalah_angka() && kanan.adalah_angka() => {
                self.angka_biner(op_sub(), &kiri, &kanan, span)
            }
            (Nilai::Instance(_), _) => {
                if let Some(hasil) = self.dunder_biner(&kiri, "__sub__", &kanan, span)? {
                    return Ok(hasil);
                }
                self.dunder_biner_kanan(&kanan, "__rsub__", &kiri, span)
            }
            (Nilai::Python(_), _) | (_, Nilai::Python(_)) => {
                crate::python_operasi(op_sub(), &kiri, &kanan)
            }
            _ => Err(self.tipe_operator("-", &kiri, &kanan, span)),
        }
    }

    fn kali(&mut self, kiri: Nilai, kanan: Nilai, span: Span) -> Hasil<Nilai> {
        match (&kiri, &kanan) {
            (Nilai::Teks(teks), Nilai::Bilangan(n))
            | (Nilai::Bilangan(n), Nilai::Teks(teks)) => {
                if *n <= 0 {
                    return Ok(Nilai::teks(""));
                }
                Ok(Nilai::teks(teks.repeat(*n as usize)))
            }
            (Nilai::Bytes(bytes), Nilai::Bilangan(n))
            | (Nilai::Bilangan(n), Nilai::Bytes(bytes)) => {
                if *n <= 0 {
                    return Ok(Nilai::Bytes(Rc::new(Vec::new())));
                }
                let mut hasil = Vec::with_capacity(bytes.len() * *n as usize);
                for _ in 0..*n {
                    hasil.extend_from_slice(bytes);
                }
                Ok(Nilai::Bytes(Rc::new(hasil)))
            }
            (Nilai::Daftar(item), Nilai::Bilangan(n))
            | (Nilai::Bilangan(n), Nilai::Daftar(item)) => {
                let asli = item.borrow().clone();
                let mut hasil = Vec::new();
                for _ in 0..(*n).max(0) {
                    hasil.extend(asli.iter().cloned());
                }
                Ok(Nilai::daftar(hasil))
            }
            (Nilai::Tupel(item), Nilai::Bilangan(n))
            | (Nilai::Bilangan(n), Nilai::Tupel(item)) => {
                let mut hasil = Vec::new();
                for _ in 0..(*n).max(0) {
                    hasil.extend(item.iter().cloned());
                }
                Ok(Nilai::tupel(hasil))
            }
            _ if kiri.adalah_angka() && kanan.adalah_angka() => {
                self.angka_biner(op_mul(), &kiri, &kanan, span)
            }
            (Nilai::Instance(_), _) => {
                if let Some(hasil) = self.dunder_biner(&kiri, "__mul__", &kanan, span)? {
                    return Ok(hasil);
                }
                self.dunder_biner_kanan(&kanan, "__rmul__", &kiri, span)
            }
            (Nilai::Python(_), _) | (_, Nilai::Python(_)) => {
                crate::python_operasi(op_mul(), &kiri, &kanan)
            }
            _ => Err(self.tipe_operator("*", &kiri, &kanan, span)),
        }
    }

    fn bagi(&mut self, kiri: Nilai, kanan: Nilai, span: Span) -> Hasil<Nilai> {
        if let (Some(a), Some(b)) = (kiri.sebagai_kompleks(), kanan.sebagai_kompleks()) {
            if a.1 != 0.0 || b.1 != 0.0 {
                if b.0 == 0.0 && b.1 == 0.0 {
                    return Err(self.lempar_pesan(
                        "KesalahanNolPembagian",
                        "pembagian dengan nol",
                        span,
                    ));
                }
                let penyebut = b.0 * b.0 + b.1 * b.1;
                return Ok(Nilai::Kompleks(
                    (a.0 * b.0 + a.1 * b.1) / penyebut,
                    (a.1 * b.0 - a.0 * b.1) / penyebut,
                ));
            }
        }
        let a = self.nilai_ke_f64(&kiri, span)?;
        let b = self.nilai_ke_f64(&kanan, span)?;
        if b == 0.0 {
            return Err(self.lempar_pesan(
                "KesalahanNolPembagian",
                "pembagian dengan nol",
                span,
            ));
        }
        Ok(Nilai::Pecahan(a / b))
    }

    fn bagi_bulat(&mut self, kiri: Nilai, kanan: Nilai, span: Span) -> Hasil<Nilai> {
        let a = self.nilai_ke_f64(&kiri, span)?;
        let b = self.nilai_ke_f64(&kanan, span)?;
        if b == 0.0 {
            return Err(self.lempar_pesan(
                "KesalahanNolPembagian",
                "pembagian bulat dengan nol",
                span,
            ));
        }
        let hasil = (a / b).floor();
        if matches!(kiri, Nilai::Pecahan(_)) || matches!(kanan, Nilai::Pecahan(_)) {
            Ok(Nilai::Pecahan(hasil))
        } else {
            Ok(Nilai::Bilangan(hasil as i64))
        }
    }

    fn modulo(&mut self, kiri: Nilai, kanan: Nilai, span: Span) -> Hasil<Nilai> {
        if let (Nilai::Teks(format), _) = (&kiri, &kanan) {
            return self.format_persen(format, &kanan, span);
        }
        if kiri.adalah_angka() && kanan.adalah_angka() {
            let a = self.nilai_ke_f64(&kiri, span)?;
            let b = self.nilai_ke_f64(&kanan, span)?;
            if b == 0.0 {
                return Err(self.lempar_pesan(
                    "KesalahanNolPembagian",
                    "modulo dengan nol",
                    span,
                ));
            }
            let hasil = a - (a / b).floor() * b;
            if matches!(kiri, Nilai::Pecahan(_)) || matches!(kanan, Nilai::Pecahan(_)) {
                return Ok(Nilai::Pecahan(hasil));
            }
            return Ok(Nilai::Bilangan(hasil as i64));
        }
        if let Some(hasil) = self.dunder_biner(&kiri, "__mod__", &kanan, span)? {
            return Ok(hasil);
        }
        if let (Nilai::Python(_), _) | (_, Nilai::Python(_)) = (&kiri, &kanan) {
            return crate::python_operasi(op_mod(), &kiri, &kanan);
        }
        Err(self.tipe_operator("%", &kiri, &kanan, span))
    }

    fn pangkat(&mut self, kiri: Nilai, kanan: Nilai, span: Span) -> Hasil<Nilai> {
        if let (Nilai::Bilangan(a), Nilai::Bilangan(b)) = (&kiri, &kanan) {
            if *b >= 0 {
                match a.checked_pow(*b as u32) {
                    Some(hasil) => return Ok(Nilai::Bilangan(hasil)),
                    None => {
                        return Err(self.lempar_pesan(
                            "KesalahanOverflow",
                            "hasil pangkat melebihi kapasitas bilangan bulat",
                            span,
                        ))
                    }
                }
            }
        }
        let a = self.nilai_ke_f64(&kiri, span)?;
        let b = self.nilai_ke_f64(&kanan, span)?;
        if a == 0.0 && b < 0.0 {
            return Err(self.lempar_pesan(
                "KesalahanNolPembagian",
                "nol tidak boleh dipangkatkan dengan eksponen negatif",
                span,
            ));
        }
        Ok(Nilai::Pecahan(a.powf(b)))
    }

    fn kali_matriks(&mut self, kiri: Nilai, kanan: Nilai, span: Span) -> Hasil<Nilai> {
        if let (Nilai::Daftar(a), Nilai::Daftar(b)) = (&kiri, &kanan) {
            let a = a.borrow().clone();
            let b = b.borrow().clone();
            if a.iter().all(|baris| matches!(baris, Nilai::Daftar(_)))
                && b.iter().all(|baris| matches!(baris, Nilai::Daftar(_)))
            {
                return self.matriks_kali(&a, &b, span);
            }
        }
        if let Some(hasil) = self.dunder_biner(&kiri, "__matmul__", &kanan, span)? {
            return Ok(hasil);
        }
        if let (Nilai::Python(_), _) | (_, Nilai::Python(_)) = (&kiri, &kanan) {
            return crate::python_operasi(op_matmul(), &kiri, &kanan);
        }
        Err(self.tipe_operator("@", &kiri, &kanan, span))
    }

    fn matriks_kali(
        &mut self,
        kiri: &[Nilai],
        kanan: &[Nilai],
        span: Span,
    ) -> Hasil<Nilai> {
        let baris_a = kiri.len();
        let kolom_a = match &kiri[0] {
            Nilai::Daftar(item) => item.borrow().len(),
            _ => 0,
        };
        let baris_b = kanan.len();
        let kolom_b = match &kanan[0] {
            Nilai::Daftar(item) => item.borrow().len(),
            _ => 0,
        };
        if kolom_a != baris_b {
            return Err(self.lempar_pesan(
                "KesalahanNilai",
                format!(
                    "dimensi matriks tidak cocok: {}x{} dikali {}x{}",
                    baris_a, kolom_a, baris_b, kolom_b
                ),
                span,
            ));
        }
        let mut hasil = Vec::with_capacity(baris_a);
        for i in 0..baris_a {
            let baris = match &kiri[i] {
                Nilai::Daftar(item) => item.borrow().clone(),
                _ => Vec::new(),
            };
            let mut baru = Vec::with_capacity(kolom_b);
            for j in 0..kolom_b {
                let mut total = Nilai::Bilangan(0);
                for k in 0..kolom_a {
                    let b = match &kanan[k] {
                        Nilai::Daftar(item) => item.borrow().get(j).cloned().unwrap_or(Nilai::Kosong),
                        _ => Nilai::Kosong,
                    };
                    let produk = self.kali(baris[k].clone(), b, span)?;
                    total = self.tambah(total, produk, span)?;
                }
                baru.push(total);
            }
            hasil.push(Nilai::daftar(baru));
        }
        Ok(Nilai::daftar(hasil))
    }

    fn geser(&mut self, op: BinOp, kiri: Nilai, kanan: Nilai, span: Span) -> Hasil<Nilai> {
        let a = self.nilai_ke_i64(&kiri, span)?;
        let b = self.nilai_ke_i64(&kanan, span)?;
        if b < 0 {
            return Err(self.lempar_pesan(
                "KesalahanNilai",
                "jumlah geser tidak boleh negatif",
                span,
            ));
        }
        let hasil = match op {
            BinOp::LShift => a.checked_shl(b as u32).ok_or_else(|| {
                self.lempar_pesan("KesalahanOverflow", "hasil geser kiri melebihi kapasitas", span)
            })?,
            BinOp::RShift => a >> b,
            _ => unreachable!(),
        };
        Ok(Nilai::Bilangan(hasil))
    }

    fn bit(&mut self, op: BinOp, kiri: Nilai, kanan: Nilai, span: Span) -> Hasil<Nilai> {
        if let (Nilai::Logika(a), Nilai::Logika(b)) = (&kiri, &kanan) {
            return Ok(Nilai::Logika(match op {
                BinOp::BitAnd => *a && *b,
                BinOp::BitOr => *a || *b,
                BinOp::BitXor => *a ^ *b,
                _ => unreachable!(),
            }));
        }
        match (&kiri, &kanan) {
            (Nilai::Himpunan(a), Nilai::Himpunan(b)) => {
                let a = a.borrow().clone();
                let b = b.borrow().clone();
                let mut hasil = Vec::new();
                match op {
                    BinOp::BitOr => {
                        hasil = a.clone();
                        for item in b {
                            self.tambah_himpunan(&mut hasil, item);
                        }
                    }
                    BinOp::BitAnd => {
                        for item in a {
                            if self.urutan_mengandung_publik(&b, &item)? {
                                hasil.push(item);
                            }
                        }
                    }
                    BinOp::BitXor => {
                        for item in &a {
                            if !self.urutan_mengandung_publik(&b, item)? {
                                hasil.push(item.clone());
                            }
                        }
                        for item in &b {
                            if !self.urutan_mengandung_publik(&a, item)? {
                                hasil.push(item.clone());
                            }
                        }
                    }
                    _ => unreachable!(),
                }
                return Ok(Nilai::himpunan(hasil));
            }
            (Nilai::Kamus(a), Nilai::Kamus(b)) if matches!(op, BinOp::BitOr) => {
                let mut hasil = a.borrow().clone();
                for (kunci, nilai) in b.borrow().iter() {
                    self.atur_kamus(&mut hasil, kunci.clone(), nilai.clone());
                }
                return Ok(Nilai::kamus(hasil));
            }
            _ if kiri.adalah_angka() && kanan.adalah_angka() => {
                let a = self.nilai_ke_i64(&kiri, span)?;
                let b = self.nilai_ke_i64(&kanan, span)?;
                let hasil = match op {
                    BinOp::BitAnd => a & b,
                    BinOp::BitOr => a | b,
                    BinOp::BitXor => a ^ b,
                    _ => unreachable!(),
                };
                return Ok(Nilai::Bilangan(hasil));
            }
            _ => {}
        }
        let nama = match op {
            BinOp::BitAnd => "__and__",
            BinOp::BitOr => "__or__",
            BinOp::BitXor => "__xor__",
            _ => unreachable!(),
        };
        if let Some(hasil) = self.dunder_biner(&kiri, nama, &kanan, span)? {
            return Ok(hasil);
        }
        if let (Nilai::Python(_), _) | (_, Nilai::Python(_)) = (&kiri, &kanan) {
            return crate::python_operasi(op, &kiri, &kanan);
        }
        Err(self.tipe_operator("operator bit", &kiri, &kanan, span))
    }

    fn angka_biner(
        &mut self,
        op: BinOp,
        kiri: &Nilai,
        kanan: &Nilai,
        span: Span,
    ) -> Hasil<Nilai> {
        if let (Some(a), Some(b)) = (kiri.sebagai_kompleks(), kanan.sebagai_kompleks()) {
            if a.1 != 0.0 || b.1 != 0.0 {
                return Ok(Nilai::Kompleks(
                    match op {
                        BinOp::Add => a.0 + b.0,
                        BinOp::Sub => a.0 - b.0,
                        BinOp::Mul => a.0 * b.0 - a.1 * b.1,
                        _ => {
                            return Err(self.tipe_operator(
                                "operator",
                                kiri,
                                kanan,
                                span,
                            ))
                        }
                    },
                    match op {
                        BinOp::Add => a.1 + b.1,
                        BinOp::Sub => a.1 - b.1,
                        BinOp::Mul => a.0 * b.1 + a.1 * b.0,
                        _ => 0.0,
                    },
                ));
            }
        }
        let a = self.nilai_ke_f64(kiri, span)?;
        let b = self.nilai_ke_f64(kanan, span)?;
        let hasil = match op {
            BinOp::Add => a + b,
            BinOp::Sub => a - b,
            BinOp::Mul => a * b,
            _ => return Err(self.tipe_operator("operator", kiri, kanan, span)),
        };
        let semua_bulat = !matches!(kiri, Nilai::Pecahan(_))
            && !matches!(kanan, Nilai::Pecahan(_));
        if semua_bulat && hasil.abs() <= i64::MAX as f64 {
            Ok(Nilai::Bilangan(hasil as i64))
        } else {
            Ok(Nilai::Pecahan(hasil))
        }
    }

    pub fn operasi_uner(
        &mut self,
        op: UnaryOp,
        nilai: Nilai,
        span: Span,
    ) -> Hasil<Nilai> {
        match op {
            UnaryOp::Not => Ok(Nilai::Logika(!self.kebenaran(&nilai)?)),
            UnaryOp::Pos => match nilai {
                Nilai::Logika(nilai) => Ok(Nilai::Bilangan(i64::from(nilai))),
                Nilai::Bilangan(_) | Nilai::Pecahan(_) | Nilai::Kompleks(_, _) => Ok(nilai),
                Nilai::Instance(instance) => {
                    if let Some(fungsi) = instance.kelas.cari_atribut("__pos__") {
                        return self.panggil(
                            &fungsi,
                            vec![Nilai::Instance(instance.clone())],
                            Vec::new(),
                            span,
                        );
                    }
                    Err(self.lempar_pesan(
                        "KesalahanTipe",
                        format!("`{}` tidak mendukung operator uner +", instance.kelas.nama),
                        span,
                    ))
                }
                _ => Err(self.lempar_pesan(
                    "KesalahanTipe",
                    format!("{} tidak mendukung operator uner +", nilai.nama_tipe()),
                    span,
                )),
            },
            UnaryOp::Neg => match nilai {
                Nilai::Logika(nilai) => Ok(Nilai::Bilangan(-i64::from(nilai))),
                Nilai::Bilangan(nilai) => nilai
                    .checked_neg()
                    .map(Nilai::Bilangan)
                    .ok_or_else(|| {
                        self.lempar_pesan(
                            "KesalahanOverflow",
                            "negasi melebihi kapasitas bilangan bulat",
                            span,
                        )
                    }),
                Nilai::Pecahan(nilai) => Ok(Nilai::Pecahan(-nilai)),
                Nilai::Kompleks(nyata, imajiner) => Ok(Nilai::Kompleks(-nyata, -imajiner)),
                Nilai::Instance(instance) => {
                    if let Some(fungsi) = instance.kelas.cari_atribut("__neg__") {
                        return self.panggil(
                            &fungsi,
                            vec![Nilai::Instance(instance.clone())],
                            Vec::new(),
                            span,
                        );
                    }
                    Err(self.lempar_pesan(
                        "KesalahanTipe",
                        format!("`{}` tidak mendukung operator uner -", instance.kelas.nama),
                        span,
                    ))
                }
                _ => Err(self.lempar_pesan(
                    "KesalahanTipe",
                    format!("{} tidak mendukung operator uner -", nilai.nama_tipe()),
                    span,
                )),
            },
            UnaryOp::Invert => match nilai {
                Nilai::Logika(nilai) => Ok(Nilai::Bilangan(!i64::from(nilai))),
                Nilai::Bilangan(nilai) => Ok(Nilai::Bilangan(!nilai)),
                Nilai::Instance(instance) => {
                    if let Some(fungsi) = instance.kelas.cari_atribut("__invert__") {
                        return self.panggil(
                            &fungsi,
                            vec![Nilai::Instance(instance.clone())],
                            Vec::new(),
                            span,
                        );
                    }
                    Err(self.lempar_pesan(
                        "KesalahanTipe",
                        format!("`{}` tidak mendukung operator ~", instance.kelas.nama),
                        span,
                    ))
                }
                _ => Err(self.lempar_pesan(
                    "KesalahanTipe",
                    format!("{} tidak mendukung operator ~", nilai.nama_tipe()),
                    span,
                )),
            },
        }
    }

    pub fn augment(
        &mut self,
        kiri: &Nilai,
        op: AugOp,
        kanan: Nilai,
        span: Span,
    ) -> Hasil<Nilai> {
        match (op, kiri) {
            (AugOp::NullCoalesce, Nilai::Kosong) => return Ok(kanan),
            (AugOp::NullCoalesce, _) => return Ok(kiri.clone()),
            (AugOp::Add, Nilai::Daftar(item)) => {
                let tambahan = self.iter_nilai(&kanan, span)?;
                item.borrow_mut().extend(tambahan);
                return Ok(kiri.clone());
            }
            (AugOp::Mul, Nilai::Daftar(item)) => {
                let n = self.nilai_ke_i64(&kanan, span)?;
                let asli = item.borrow().clone();
                let mut hasil = Vec::new();
                for _ in 0..n.max(0) {
                    hasil.extend(asli.iter().cloned());
                }
                *item.borrow_mut() = hasil;
                return Ok(kiri.clone());
            }
            (AugOp::BitOr, Nilai::Himpunan(item)) => {
                let tambahan = self.iter_nilai(&kanan, span)?;
                for nilai in tambahan {
                    let mut daftar = item.borrow().clone();
                    self.tambah_himpunan(&mut daftar, nilai);
                    *item.borrow_mut() = daftar;
                }
                return Ok(kiri.clone());
            }
            (AugOp::BitOr, Nilai::Kamus(item)) => {
                for (kunci, nilai) in self.kamus_pasangan(&kanan, span)? {
                    let mut daftar = item.borrow().clone();
                    self.atur_kamus(&mut daftar, kunci, nilai);
                    *item.borrow_mut() = daftar;
                }
                return Ok(kiri.clone());
            }
            (AugOp::BitAnd, Nilai::Himpunan(item))
            | (AugOp::BitXor, Nilai::Himpunan(item))
            | (AugOp::Sub, Nilai::Himpunan(item)) => {
                let bin = match op {
                    AugOp::BitAnd => BinOp::BitAnd,
                    AugOp::BitXor => BinOp::BitXor,
                    AugOp::Sub => BinOp::Sub,
                    _ => unreachable!(),
                };
                let hasil = self.operasi_biner(bin, kiri.clone(), kanan, span)?;
                if let Nilai::Himpunan(baru) = &hasil {
                    *item.borrow_mut() = baru.borrow().clone();
                }
                return Ok(kiri.clone());
            }
            _ => {}
        }
        if let Nilai::Instance(instance) = kiri {
            let nama = match op {
                AugOp::Add => "__iadd__",
                AugOp::Sub => "__isub__",
                AugOp::Mul => "__imul__",
                AugOp::Div => "__itruediv__",
                AugOp::FloorDiv => "__ifloordiv__",
                AugOp::Mod => "__imod__",
                AugOp::Pow => "__ipow__",
                AugOp::MatMul => "__imatmul__",
                AugOp::LShift => "__ilshift__",
                AugOp::RShift => "__irshift__",
                AugOp::BitAnd => "__iand__",
                AugOp::BitOr => "__ior__",
                AugOp::BitXor => "__ixor__",
                AugOp::NullCoalesce => unreachable!(),
            };
            if let Some(fungsi) = instance.kelas.cari_atribut(nama) {
                return self.panggil(
                    &fungsi,
                    vec![Nilai::Instance(instance.clone()), kanan],
                    Vec::new(),
                    span,
                );
            }
        }
        let bin = match op {
            AugOp::Add => BinOp::Add,
            AugOp::Sub => BinOp::Sub,
            AugOp::Mul => BinOp::Mul,
            AugOp::Div => BinOp::Div,
            AugOp::FloorDiv => BinOp::FloorDiv,
            AugOp::Mod => BinOp::Mod,
            AugOp::Pow => BinOp::Pow,
            AugOp::MatMul => BinOp::MatMul,
            AugOp::LShift => BinOp::LShift,
            AugOp::RShift => BinOp::RShift,
            AugOp::BitAnd => BinOp::BitAnd,
            AugOp::BitOr => BinOp::BitOr,
            AugOp::BitXor => BinOp::BitXor,
            AugOp::NullCoalesce => BinOp::NullCoalesce,
        };
        self.operasi_biner(bin, kiri.clone(), kanan, span)
    }

    fn dunder_biner(
        &mut self,
        nilai: &Nilai,
        nama: &str,
        lain: &Nilai,
        span: Span,
    ) -> Hasil<Option<Nilai>> {
        if let Nilai::Instance(instance) = nilai {
            if let Some(fungsi) = instance.kelas.cari_atribut(nama) {
                let hasil = self.panggil(
                    &fungsi,
                    vec![Nilai::Instance(instance.clone()), lain.clone()],
                    Vec::new(),
                    span,
                )?;
                return Ok(Some(hasil));
            }
        }
        Ok(None)
    }

    fn dunder_biner_kanan(
        &mut self,
        nilai: &Nilai,
        nama: &str,
        lain: &Nilai,
        span: Span,
    ) -> Hasil<Nilai> {
        if let Some(hasil) = self.dunder_biner(nilai, nama, lain, span)? {
            return Ok(hasil);
        }
        Err(self.lempar_pesan(
            "KesalahanTipe",
            format!(
                "{} dan {} tidak mendukung operator ini",
                lain.nama_tipe(),
                nilai.nama_tipe()
            ),
            span,
        ))
    }

    fn tipe_operator(&mut self, op: &str, kiri: &Nilai, kanan: &Nilai, span: Span) -> Lemparan {
        self.lempar_pesan(
            "KesalahanTipe",
            format!(
                "operator {op} tidak mendukung {} dan {}",
                kiri.nama_tipe(),
                kanan.nama_tipe()
            ),
            span,
        )
    }

    pub fn nilai_ke_f64(&mut self, nilai: &Nilai, span: Span) -> Hasil<f64> {
        match nilai {
            Nilai::Logika(nilai) => Ok(if *nilai { 1.0 } else { 0.0 }),
            Nilai::Bilangan(nilai) => Ok(*nilai as f64),
            Nilai::Pecahan(nilai) => Ok(*nilai),
            _ => Err(self.lempar_pesan(
                "KesalahanTipe",
                format!("{} bukan bilangan nyata", nilai.nama_tipe()),
                span,
            )),
        }
    }

    fn urutan_mengandung_publik(&mut self, daftar: &[Nilai], item: &Nilai) -> Hasil<bool> {
        for kandidat in daftar {
            if self.sama(kandidat, item)? {
                return Ok(true);
            }
        }
        Ok(false)
    }
}

fn op_add() -> BinOp {
    BinOp::Add
}
fn op_sub() -> BinOp {
    BinOp::Sub
}
fn op_mul() -> BinOp {
    BinOp::Mul
}
fn op_mod() -> BinOp {
    BinOp::Mod
}
fn op_matmul() -> BinOp {
    BinOp::MatMul
}

impl Interpreter {
    // ------------------------------------------------------------------
    // Indeks dan slice
    // ------------------------------------------------------------------

    pub fn ambil_indeks(&mut self, wadah: &Nilai, kunci: &Nilai, span: Span) -> Hasil<Nilai> {
        match wadah {
            Nilai::Daftar(item) => {
                let daftar = item.borrow().clone();
                match kunci {
                    Nilai::Slice(slice) => {
                        let indeks = slice.indeks(daftar.len()).ok_or_else(|| {
                            self.lempar_pesan(
                                "KesalahanNilai",
                                "langkah slice tidak boleh nol",
                                span,
                            )
                        })?;
                        Ok(Nilai::daftar(
                            indeks.into_iter().map(|i| daftar[i].clone()).collect(),
                        ))
                    }
                    _ => {
                        let i = self.indeks_ke_usize(kunci, daftar.len(), span)?;
                        Ok(daftar[i].clone())
                    }
                }
            }
            Nilai::Tupel(item) => match kunci {
                Nilai::Slice(slice) => {
                    let indeks = slice.indeks(item.len()).ok_or_else(|| {
                        self.lempar_pesan(
                            "KesalahanNilai",
                            "langkah slice tidak boleh nol",
                            span,
                        )
                    })?;
                    Ok(Nilai::tupel(
                        indeks.into_iter().map(|i| item[i].clone()).collect(),
                    ))
                }
                _ => {
                    let i = self.indeks_ke_usize(kunci, item.len(), span)?;
                    Ok(item[i].clone())
                }
            },
            Nilai::Teks(teks) => {
                let chars: Vec<char> = teks.chars().collect();
                match kunci {
                    Nilai::Slice(slice) => {
                        let indeks = slice.indeks(chars.len()).ok_or_else(|| {
                            self.lempar_pesan(
                                "KesalahanNilai",
                                "langkah slice tidak boleh nol",
                                span,
                            )
                        })?;
                        Ok(Nilai::teks(
                            indeks.into_iter().map(|i| chars[i]).collect::<String>(),
                        ))
                    }
                    _ => {
                        let i = self.indeks_ke_usize(kunci, chars.len(), span)?;
                        Ok(Nilai::teks(chars[i].to_string()))
                    }
                }
            }
            Nilai::Bytes(bytes) => match kunci {
                Nilai::Slice(slice) => {
                    let indeks = slice.indeks(bytes.len()).ok_or_else(|| {
                        self.lempar_pesan(
                            "KesalahanNilai",
                            "langkah slice tidak boleh nol",
                            span,
                        )
                    })?;
                    Ok(Nilai::Bytes(Rc::new(
                        indeks.into_iter().map(|i| bytes[i]).collect(),
                    )))
                }
                _ => {
                    let i = self.indeks_ke_usize(kunci, bytes.len(), span)?;
                    Ok(Nilai::Bilangan(i64::from(bytes[i])))
                }
            },
            Nilai::Kamus(pasangan) => {
                let pasangan = pasangan.borrow().clone();
                for (k, v) in pasangan {
                    if self.sama(&k, kunci)? {
                        return Ok(v);
                    }
                }
                Err(self.lempar_pesan(
                    "KesalahanKunci",
                    format!("kunci {} tidak ditemukan", nilai_ke_repr(kunci)),
                    span,
                ))
            }
            Nilai::Rentang(rentang) => {
                let nilai = rentang.nilai();
                match kunci {
                    Nilai::Slice(slice) => {
                        let indeks = slice.indeks(nilai.len()).ok_or_else(|| {
                            self.lempar_pesan(
                                "KesalahanNilai",
                                "langkah slice tidak boleh nol",
                                span,
                            )
                        })?;
                        let dipilih: Vec<i64> = indeks
                            .into_iter()
                            .map(|i| match &nilai[i] {
                                Nilai::Bilangan(n) => *n,
                                _ => 0,
                            })
                            .collect();
                        if dipilih.is_empty() {
                            return Ok(Nilai::Rentang(Rentang {
                                mulai: 0,
                                akhir: 0,
                                langkah: 1,
                            }));
                        }
                        let langkah = if dipilih.len() > 1 {
                            dipilih[1] - dipilih[0]
                        } else {
                            rentang.langkah
                        };
                        Ok(Nilai::Rentang(Rentang {
                            mulai: dipilih[0],
                            akhir: dipilih[dipilih.len() - 1] + langkah,
                            langkah,
                        }))
                    }
                    _ => {
                        let i = self.indeks_ke_usize(kunci, nilai.len(), span)?;
                        Ok(nilai[i].clone())
                    }
                }
            }
            Nilai::Instance(instance) => {
                if let Some(fungsi) = instance.kelas.cari_atribut("__getitem__") {
                    return self.panggil(
                        &fungsi,
                        vec![Nilai::Instance(instance.clone()), kunci.clone()],
                        Vec::new(),
                        span,
                    );
                }
                Err(self.lempar_pesan(
                    "KesalahanTipe",
                    format!("`{}` tidak mendukung indeks", instance.kelas.nama),
                    span,
                ))
            }
            Nilai::Python(_) => crate::python_ambil_indeks(wadah, kunci),
            _ => Err(self.lempar_pesan(
                "KesalahanTipe",
                format!("{} tidak mendukung indeks", wadah.nama_tipe()),
                span,
            )),
        }
    }

    pub fn atur_indeks(
        &mut self,
        wadah: &Nilai,
        kunci: &Nilai,
        nilai: Nilai,
        span: Span,
    ) -> Hasil<()> {
        match wadah {
            Nilai::Daftar(item) => {
                let panjang = item.borrow().len();
                match kunci {
                    Nilai::Slice(slice) => {
                        let indeks = slice.indeks(panjang).ok_or_else(|| {
                            self.lempar_pesan(
                                "KesalahanNilai",
                                "langkah slice tidak boleh nol",
                                span,
                            )
                        })?;
                        let pengganti = self.iter_nilai(&nilai, span)?;
                        if indeks.is_empty() {
                            if pengganti.is_empty() {
                                return Ok(());
                            }
                            let posisi = slice
                                .mulai
                                .map(|n| if n < 0 { n + panjang as i64 } else { n })
                                .unwrap_or(0)
                                .clamp(0, panjang as i64) as usize;
                            item.borrow_mut().splice(posisi..posisi, pengganti);
                            return Ok(());
                        }
                        if indeks.len() != pengganti.len() {
                            let berurutan = indeks
                                .windows(2)
                                .all(|jendela| jendela[1] == jendela[0] + 1);
                            if !berurutan {
                                return Err(self.lempar_pesan(
                                    "KesalahanNilai",
                                    "slice berlangkah membutuhkan jumlah nilai yang sama",
                                    span,
                                ));
                            }
                            let mulai = indeks[0];
                            let akhir = indeks[indeks.len() - 1] + 1;
                            item.borrow_mut().splice(mulai..akhir, pengganti);
                        } else {
                            let mut daftar = item.borrow_mut();
                            for (posisi, nilai) in indeks.into_iter().zip(pengganti) {
                                daftar[posisi] = nilai;
                            }
                        }
                        Ok(())
                    }
                    _ => {
                        let i = self.indeks_ke_usize(kunci, panjang, span)?;
                        item.borrow_mut()[i] = nilai;
                        Ok(())
                    }
                }
            }
            Nilai::Kamus(pasangan) => {
                let mut pasangan = pasangan.borrow_mut();
                for (k, v) in pasangan.iter_mut() {
                    if self.sama(k, kunci)? {
                        *v = nilai;
                        return Ok(());
                    }
                }
                pasangan.push((kunci.clone(), nilai));
                Ok(())
            }
            Nilai::Instance(instance) => {
                if let Some(fungsi) = instance.kelas.cari_atribut("__setitem__") {
                    self.panggil(
                        &fungsi,
                        vec![Nilai::Instance(instance.clone()), kunci.clone(), nilai],
                        Vec::new(),
                        span,
                    )?;
                    return Ok(());
                }
                Err(self.lempar_pesan(
                    "KesalahanTipe",
                    format!("`{}` tidak mendukung penugasan indeks", instance.kelas.nama),
                    span,
                ))
            }
            Nilai::Python(_) => crate::python_atur_indeks(wadah, kunci, nilai),
            _ => Err(self.lempar_pesan(
                "KesalahanTipe",
                format!("{} tidak mendukung penugasan indeks", wadah.nama_tipe()),
                span,
            )),
        }
    }

    pub fn hapus_indeks(&mut self, wadah: &Nilai, kunci: &Nilai, span: Span) -> Hasil<()> {
        match wadah {
            Nilai::Daftar(item) => {
                let panjang = item.borrow().len();
                match kunci {
                    Nilai::Slice(slice) => {
                        let indeks = slice.indeks(panjang).ok_or_else(|| {
                            self.lempar_pesan(
                                "KesalahanNilai",
                                "langkah slice tidak boleh nol",
                                span,
                            )
                        })?;
                        let mut daftar = item.borrow_mut();
                        for i in indeks.into_iter().rev() {
                            daftar.remove(i);
                        }
                        Ok(())
                    }
                    _ => {
                        let i = self.indeks_ke_usize(kunci, panjang, span)?;
                        item.borrow_mut().remove(i);
                        Ok(())
                    }
                }
            }
            Nilai::Kamus(pasangan) => {
                let mut pasangan = pasangan.borrow_mut();
                let posisi = pasangan
                    .iter()
                    .position(|(k, _)| self.sama(k, kunci).unwrap_or(false));
                match posisi {
                    Some(posisi) => {
                        pasangan.remove(posisi);
                        Ok(())
                    }
                    None => Err(self.lempar_pesan(
                        "KesalahanKunci",
                        format!("kunci {} tidak ditemukan", nilai_ke_repr(kunci)),
                        span,
                    )),
                }
            }
            Nilai::Himpunan(item) => {
                let mut item = item.borrow_mut();
                let posisi = item
                    .iter()
                    .position(|kandidat| self.sama(kandidat, kunci).unwrap_or(false));
                match posisi {
                    Some(posisi) => {
                        item.remove(posisi);
                        Ok(())
                    }
                    None => Err(self.lempar_pesan(
                        "KesalahanKunci",
                        format!("nilai {} tidak ditemukan", nilai_ke_repr(kunci)),
                        span,
                    )),
                }
            }
            Nilai::Instance(instance) => {
                if let Some(fungsi) = instance.kelas.cari_atribut("__delitem__") {
                    self.panggil(
                        &fungsi,
                        vec![Nilai::Instance(instance.clone()), kunci.clone()],
                        Vec::new(),
                        span,
                    )?;
                    return Ok(());
                }
                Err(self.lempar_pesan(
                    "KesalahanTipe",
                    format!("`{}` tidak mendukung penghapusan indeks", instance.kelas.nama),
                    span,
                ))
            }
            Nilai::Python(_) => crate::python_hapus_indeks(wadah, kunci),
            _ => Err(self.lempar_pesan(
                "KesalahanTipe",
                format!("{} tidak mendukung penghapusan indeks", wadah.nama_tipe()),
                span,
            )),
        }
    }

    fn indeks_ke_usize(&mut self, kunci: &Nilai, panjang: usize, span: Span) -> Hasil<usize> {
        let indeks = self.nilai_ke_i64(kunci, span)?;
        let panjang_i = panjang as i64;
        let disesuaikan = if indeks < 0 {
            indeks + panjang_i
        } else {
            indeks
        };
        if disesuaikan < 0 || disesuaikan >= panjang_i {
            return Err(self.lempar_pesan(
                "KesalahanIndeks",
                format!(
                    "indeks {indeks} di luar rentang untuk panjang {panjang}"
                ),
                span,
            ));
        }
        Ok(disesuaikan as usize)
    }

    // ------------------------------------------------------------------
    // Atribut
    // ------------------------------------------------------------------

    pub fn ambil_atribut(&mut self, objek: &Nilai, nama: &str, span: Span) -> Hasil<Nilai> {
        match objek {
            Nilai::Modul(modul) => {
                if let Some(nilai) = modul.atribut.borrow().get(nama) {
                    return Ok(nilai.clone());
                }
                Err(self.lempar_pesan(
                    "KesalahanAtribut",
                    format!("modul `{}` tidak memiliki atribut `{nama}`", modul.nama),
                    span,
                ))
            }
            Nilai::Kelas(kelas) => {
                if let Some(nilai) = kelas.cari_atribut(nama) {
                    return Ok(self.bungkus_atribut_kelas(kelas, nilai));
                }
                if nama == "__mro__" {
                    return Ok(Nilai::tupel(
                        crate::nilai::mro_lengkap(kelas)
                            .into_iter()
                            .map(Nilai::Kelas)
                            .collect(),
                    ));
                }
                if nama == "__nama__" {
                    return Ok(Nilai::teks(kelas.nama.clone()));
                }
                Err(self.lempar_pesan(
                    "KesalahanAtribut",
                    format!("kelas `{}` tidak memiliki atribut `{nama}`", kelas.nama),
                    span,
                ))
            }
            Nilai::Instance(instance) => {
                if let Some(nilai) = instance.atribut.borrow().get(nama) {
                    return Ok(nilai.clone());
                }
                if let Some(nilai) = instance.kelas.cari_atribut(nama) {
                    return self.bungkus_atribut_instance(instance, nilai, span);
                }
                if let Some(getattr) = instance.kelas.cari_atribut("__getattr__") {
                    let hasil = self.panggil(
                        &getattr,
                        vec![
                            Nilai::Instance(instance.clone()),
                            Nilai::teks(nama.to_string()),
                        ],
                        Vec::new(),
                        span,
                    )?;
                    return Ok(hasil);
                }
                Err(self.lempar_pesan(
                    "KesalahanAtribut",
                    format!("`{}` tidak memiliki atribut `{nama}`", instance.kelas.nama),
                    span,
                ))
            }
            Nilai::Induk(induk) => {
                let mro = crate::nilai::mro_lengkap(&induk.kelas);
                for kelas in mro.iter().skip(induk.mulai) {
                    if let Some(nilai) = kelas.atribut.borrow().get(nama) {
                        return match &induk.instance {
                            Some(instance) => {
                                self.bungkus_atribut_instance(instance, nilai.clone(), span)
                            }
                            None => Ok(self.bungkus_atribut_kelas(kelas, nilai.clone())),
                        };
                    }
                }
                Err(self.lempar_pesan(
                    "KesalahanAtribut",
                    format!("`induk()` tidak memiliki atribut `{nama}`"),
                    span,
                ))
            }
            Nilai::Properti(properti) => match nama {
                "getter" => Ok(properti.getter.clone()),
                "setter" => {
                    let properti = properti.clone();
                    Ok(Nilai::BawaanKhusus(Rc::new(BawaanKhusus {
                        nama: "setter".to_string(),
                        fungsi: Box::new(move |_interpreter, args, _kwargs| {
                            let setter = args.first().cloned().ok_or_else(|| {
                                Lemparan::baru(
                                    Kesalahan::dari_pesan(
                                        Kelas::baru_jenis(
                                            "KesalahanTipe",
                                            Vec::new(),
                                            JenisKelas::Kesalahan,
                                        ),
                                        "setter membutuhkan satu fungsi",
                                    ),
                                    Span::empty(0),
                                )
                            })?;
                            Ok(Nilai::Properti(Rc::new(Properti {
                                getter: properti.getter.clone(),
                                setter: Some(setter),
                            })))
                        }),
                    })))
                }
                "deleter" => Ok(Nilai::Kosong),
                _ => Err(self.lempar_pesan(
                    "KesalahanAtribut",
                    format!("properti tidak memiliki atribut `{nama}`"),
                    span,
                )),
            },
            Nilai::Rentang(rentang) => match nama {
                "mulai" => Ok(Nilai::Bilangan(rentang.mulai)),
                "akhir" => Ok(Nilai::Bilangan(rentang.akhir)),
                "langkah" => Ok(Nilai::Bilangan(rentang.langkah)),
                "start" => Ok(Nilai::Bilangan(rentang.mulai)),
                "stop" => Ok(Nilai::Bilangan(rentang.akhir)),
                "step" => Ok(Nilai::Bilangan(rentang.langkah)),
                _ => self.atribut_metode(objek, metode_tipe(objek), nama, span),
            },
            Nilai::Slice(slice) => match nama {
                "mulai" => Ok(slice.mulai.map(Nilai::Bilangan).unwrap_or(Nilai::Kosong)),
                "akhir" => Ok(slice.akhir.map(Nilai::Bilangan).unwrap_or(Nilai::Kosong)),
                "langkah" => Ok(slice.langkah.map(Nilai::Bilangan).unwrap_or(Nilai::Kosong)),
                "start" => Ok(slice.mulai.map(Nilai::Bilangan).unwrap_or(Nilai::Kosong)),
                "stop" => Ok(slice.akhir.map(Nilai::Bilangan).unwrap_or(Nilai::Kosong)),
                "step" => Ok(slice.langkah.map(Nilai::Bilangan).unwrap_or(Nilai::Kosong)),
                _ => self.atribut_metode(objek, "slice", nama, span),
            },
            Nilai::Berkas(berkas) => {
                let b = berkas.borrow();
                match nama {
                    "nama" => Ok(Nilai::teks(b.nama.clone())),
                    "mode" => Ok(Nilai::teks(b.mode.clone())),
                    "biner" => Ok(Nilai::Logika(b.biner)),
                    "tertutup" => Ok(Nilai::Logika(b.berkas.is_none())),
                    _ => {
                        drop(b);
                        self.atribut_metode(objek, "berkas", nama, span)
                    }
                }
            }
            Nilai::Kesalahan(kesalahan) => match nama {
                "pesan" => Ok(Nilai::teks(kesalahan.pesan.clone())),
                "argumen" => Ok(Nilai::tupel(kesalahan.argumen.clone())),
                "tipe" => Ok(Nilai::Kelas(kesalahan.tipe.clone())),
                "__nama__" => Ok(Nilai::teks(kesalahan.tipe.nama.clone())),
                "__penyebab__" => Ok(kesalahan
                    .penyebab
                    .as_ref()
                    .map(|p| Nilai::Kesalahan(p.clone()))
                    .unwrap_or(Nilai::Kosong)),
                _ => {
                    if let Some(nilai) = kesalahan.atribut.borrow().get(nama) {
                        Ok(nilai.clone())
                    } else {
                        self.atribut_metode(objek, "kesalahan", nama, span)
                    }
                }
            },
            Nilai::Fungsi(fungsi) => match nama {
                "__nama__" => Ok(Nilai::teks(fungsi.nama.clone())),
                "__dok__" => Ok(fungsi
                    .doc
                    .as_ref()
                    .map(|d| Nilai::teks(d.clone()))
                    .unwrap_or(Nilai::Kosong)),
                "__modul__" => Ok(Nilai::teks(fungsi.berkas.clone())),
                _ => Err(self.lempar_pesan(
                    "KesalahanAtribut",
                    format!("fungsi tidak memiliki atribut `{nama}`"),
                    span,
                )),
            },
            Nilai::Bawaan(bawaan) => match nama {
                "__nama__" => Ok(Nilai::teks(bawaan.nama.clone())),
                "__dok__" => Ok(bawaan
                    .doc
                    .as_ref()
                    .map(|d| Nilai::teks(d.clone()))
                    .unwrap_or(Nilai::Kosong)),
                _ => Err(self.lempar_pesan(
                    "KesalahanAtribut",
                    format!("fungsi bawaan tidak memiliki atribut `{nama}`"),
                    span,
                )),
            },
            Nilai::Metode(metode) => match nama {
                "__nama__" => Ok(Nilai::teks(metode.fungsi.nama.clone())),
                "__diri__" => Ok(metode.penerima.clone()),
                _ => Err(self.lempar_pesan(
                    "KesalahanAtribut",
                    format!("metode tidak memiliki atribut `{nama}`"),
                    span,
                )),
            },
            Nilai::Python(_) => crate::python_ambil_atribut(objek, nama),
            _ => self.atribut_metode(objek, metode_tipe(objek), nama, span),
        }
    }

    fn bungkus_atribut_kelas(&mut self, kelas: &Rc<Kelas>, nilai: Nilai) -> Nilai {
        match nilai {
            Nilai::MetodeKelas(metode) => Nilai::MetodeKelas(Rc::new(MetodeKelas {
                fungsi: metode.fungsi.clone(),
                kelas: kelas.clone(),
            })),
            other => other,
        }
    }

    fn bungkus_atribut_instance(
        &mut self,
        instance: &Rc<Instance>,
        nilai: Nilai,
        span: Span,
    ) -> Hasil<Nilai> {
        match nilai {
            Nilai::Fungsi(fungsi) => Ok(Nilai::Metode(Rc::new(Metode {
                fungsi,
                penerima: Nilai::Instance(instance.clone()),
            }))),
            Nilai::MetodeStatis(fungsi) => Ok(Nilai::Fungsi(fungsi)),
            Nilai::MetodeKelas(metode) => Ok(Nilai::MetodeKelas(Rc::new(MetodeKelas {
                fungsi: metode.fungsi.clone(),
                kelas: instance.kelas.clone(),
            }))),
            Nilai::Properti(properti) => self.panggil(
                &properti.getter,
                vec![Nilai::Instance(instance.clone())],
                Vec::new(),
                span,
            ),
            other => Ok(other),
        }
    }

    fn atribut_metode(
        &mut self,
        objek: &Nilai,
        tipe: &str,
        nama: &str,
        span: Span,
    ) -> Hasil<Nilai> {
        if crate::metode::ada_metode(tipe, nama) {
            Ok(Nilai::MetodeBawaan(Rc::new(MetodeBawaan {
                penerima: objek.clone(),
                nama: nama.to_string(),
            })))
        } else {
            Err(self.lempar_pesan(
                "KesalahanAtribut",
                format!("tipe `{tipe}` tidak memiliki atribut `{nama}`"),
                span,
            ))
        }
    }

    pub fn atur_atribut(
        &mut self,
        objek: &Nilai,
        nama: &str,
        nilai: Nilai,
        span: Span,
    ) -> Hasil<()> {
        match objek {
            Nilai::Instance(instance) => {
                if let Some(Nilai::Properti(properti)) = instance.kelas.cari_atribut(nama) {
                    if let Some(setter) = &properti.setter {
                        self.panggil(
                            setter,
                            vec![Nilai::Instance(instance.clone()), nilai],
                            Vec::new(),
                            span,
                        )?;
                        return Ok(());
                    }
                }
                instance
                    .atribut
                    .borrow_mut()
                    .insert(nama.to_string(), nilai);
                Ok(())
            }
            Nilai::Kelas(kelas) => {
                kelas
                    .atribut
                    .borrow_mut()
                    .insert(nama.to_string(), nilai);
                Ok(())
            }
            Nilai::Modul(modul) => {
                modul
                    .atribut
                    .borrow_mut()
                    .insert(nama.to_string(), nilai);
                Ok(())
            }
            Nilai::Kesalahan(kesalahan) => {
                kesalahan
                    .atribut
                    .borrow_mut()
                    .insert(nama.to_string(), nilai);
                Ok(())
            }
            Nilai::Python(_) => crate::python_atur_atribut(objek, nama, nilai),
            _ => Err(self.lempar_pesan(
                "KesalahanTipe",
                format!("{} tidak mendukung penugasan atribut", objek.nama_tipe()),
                span,
            )),
        }
    }

    pub fn hapus_atribut(&mut self, objek: &Nilai, nama: &str, span: Span) -> Hasil<()> {
        let dihapus = match objek {
            Nilai::Instance(instance) => instance.atribut.borrow_mut().remove(nama).is_some(),
            Nilai::Kelas(kelas) => kelas.atribut.borrow_mut().remove(nama).is_some(),
            Nilai::Modul(modul) => modul.atribut.borrow_mut().remove(nama).is_some(),
            Nilai::Kesalahan(kesalahan) => {
                kesalahan.atribut.borrow_mut().remove(nama).is_some()
            }
            Nilai::Python(_) => {
                crate::python_hapus_atribut(objek, nama)?;
                return Ok(());
            }
            _ => false,
        };
        if dihapus {
            Ok(())
        } else {
            Err(self.lempar_pesan(
                "KesalahanAtribut",
                format!("atribut `{nama}` tidak ditemukan"),
                span,
            ))
        }
    }

    // ------------------------------------------------------------------
    // Koleksi: himpunan dan kamus
    // ------------------------------------------------------------------

    pub fn tambah_himpunan(&mut self, daftar: &mut Vec<Nilai>, nilai: Nilai) {
        for item in daftar.iter() {
            if self.sama(item, &nilai).unwrap_or(false) {
                return;
            }
        }
        daftar.push(nilai);
    }

    pub fn atur_kamus(&mut self, daftar: &mut Vec<(Nilai, Nilai)>, kunci: Nilai, nilai: Nilai) {
        for (k, v) in daftar.iter_mut() {
            if self.sama(k, &kunci).unwrap_or(false) {
                *v = nilai;
                return;
            }
        }
        daftar.push((kunci, nilai));
    }

    pub fn kamus_pasangan(&mut self, nilai: &Nilai, span: Span) -> Hasil<Vec<(Nilai, Nilai)>> {
        match nilai {
            Nilai::Kamus(pasangan) => Ok(pasangan.borrow().clone()),
            Nilai::Daftar(item) => {
                let item = item.borrow().clone();
                self.pasangan_dari_urutan(&item, span)
            }
            Nilai::Tupel(item) => self.pasangan_dari_urutan(item, span),
            Nilai::Python(_) => crate::python_kamus_pasangan(nilai),
            _ => Err(self.lempar_pesan(
                "KesalahanTipe",
                format!("{} tidak dapat diubah menjadi kamus", nilai.nama_tipe()),
                span,
            )),
        }
    }

    fn pasangan_dari_urutan(
        &mut self,
        item: &[Nilai],
        span: Span,
    ) -> Hasil<Vec<(Nilai, Nilai)>> {
        let mut hasil = Vec::new();
        for pasangan in item {
            let isi = self.iter_nilai(pasangan, span)?;
            if isi.len() != 2 {
                return Err(self.lempar_pesan(
                    "KesalahanNilai",
                    "setiap elemen kamus harus berisi dua nilai",
                    span,
                ));
            }
            self.atur_kamus(&mut hasil, isi[0].clone(), isi[1].clone());
        }
        Ok(hasil)
    }
}

fn metode_tipe(nilai: &Nilai) -> &'static str {
    match nilai {
        Nilai::Teks(_) => "teks",
        Nilai::Bytes(_) => "bytes",
        Nilai::Daftar(_) => "daftar",
        Nilai::Tupel(_) => "tupel",
        Nilai::Himpunan(_) => "himpunan",
        Nilai::Kamus(_) => "kamus",
        Nilai::Rentang(_) => "rentang",
        Nilai::Generator(_) => "generator",
        Nilai::Berkas(_) => "berkas",
        Nilai::Kesalahan(_) => "kesalahan",
        Nilai::Slice(_) => "slice",
        _ => "nilai",
    }
}
