//! Impor modul native, berkas `.aks`, paket, dan modul Python.
//!
//! Resolusi impor sengaja dibuat eksplisit agar pesan kesalahan menyebut
//! lokasi yang dicari. Modul yang sedang dimuat dicatat lebih dahulu di cache
//! sehingga impor melingkar tidak menyebabkan rekursi tanpa batas.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::rc::Rc;

use aksara_ast::{ImportAlias, Span};

use crate::format::nilai_ke_teks;
use crate::interpreter::Interpreter;
use crate::modul_bawaan;
use crate::nilai::*;

impl Interpreter {
    /// Menjalankan satu pernyataan `impor`.
    pub fn impor_alias(
        &mut self,
        alias: &ImportAlias,
        env: &Env,
        span: Span,
    ) -> Hasil<Nilai> {
        if alias.python {
            let nilai = self.impor_python(&alias.name, span)?;
            let nama = alias
                .asname
                .clone()
                .unwrap_or_else(|| alias.name.rsplit('.').next().unwrap_or(&alias.name).to_string());
            self.atur_nama(env, nama, nilai.clone());
            return Ok(nilai);
        }
        let (akar, daun) = self.impor_rantai(&alias.name, span)?;
        let nilai = match &alias.asname {
            Some(_) => Nilai::Modul(daun),
            None => Nilai::Modul(akar),
        };
        let nama = alias
            .asname
            .clone()
            .unwrap_or_else(|| alias.name.split('.').next().unwrap_or(&alias.name).to_string());
        self.atur_nama(env, nama, nilai.clone());
        Ok(nilai)
    }

    /// Menjalankan `dari ... impor ...`.
    pub fn impor_dari(
        &mut self,
        module: &str,
        level: usize,
        names: &[ImportAlias],
        python: bool,
        env: &Env,
        span: Span,
    ) -> Hasil<()> {
        if python {
            let modul = self.impor_python(module, span)?;
            for alias in names {
                if alias.name == "*" {
                    let nama = crate::python_semua(&modul, span)?;
                    for nama in nama {
                        if nama.starts_with('_') {
                            continue;
                        }
                        let nilai = crate::python_ambil_atribut(&modul, &nama)?;
                        self.atur_nama(env, nama, nilai);
                    }
                } else {
                    let nilai = crate::python_ambil_atribut(&modul, &alias.name)?;
                    let nama = alias.asname.clone().unwrap_or_else(|| alias.name.clone());
                    self.atur_nama(env, nama, nilai);
                }
            }
            return Ok(());
        }

        let modul = if level > 0 {
            self.impor_relatif(module, level, span)?
        } else {
            self.impor_satu(module, span)?
        };
        for alias in names {
            if alias.name == "*" {
                for nama in modul_bawaan::nama_publik(&modul) {
                    if let Some(nilai) = modul.atribut.borrow().get(&nama).cloned() {
                        self.atur_nama(env, nama, nilai);
                    }
                }
                continue;
            }
            let nilai = match modul.atribut.borrow().get(&alias.name).cloned() {
                Some(nilai) => nilai,
                None => {
                    // `dari paket impor submodul` juga harus bekerja.
                    let lengkap = if module.is_empty() {
                        alias.name.clone()
                    } else {
                        format!("{module}.{}", alias.name)
                    };
                    match self.impor_satu(&lengkap, span) {
                        Ok(anak) => {
                            let anak_nilai = Nilai::Modul(anak.clone());
                            modul
                                .atribut
                                .borrow_mut()
                                .insert(alias.name.clone(), anak_nilai.clone());
                            anak_nilai
                        }
                        Err(_) => {
                            return Err(self.lempar_pesan(
                                "KesalahanImpor",
                                format!(
                                    "modul `{}` tidak memiliki nama `{}`",
                                    modul.nama, alias.name
                                ),
                                span,
                            ))
                        }
                    }
                }
            };
            let nama = alias.asname.clone().unwrap_or_else(|| alias.name.clone());
            self.atur_nama(env, nama, nilai);
        }
        Ok(())
    }

    /// Mengimpor modul Python melalui jembatan.
    pub fn impor_python(&mut self, nama: &str, span: Span) -> Hasil<Nilai> {
        crate::python_impor(nama, span)
    }

    /// Mengimpor nama bertitik dan mengembalikan (akar, daun).
    fn impor_rantai(&mut self, nama: &str, span: Span) -> Hasil<(Rc<Modul>, Rc<Modul>)> {
        let bagian: Vec<&str> = nama.split('.').filter(|bagian| !bagian.is_empty()).collect();
        if bagian.is_empty() {
            return Err(self.lempar_pesan(
                "KesalahanImpor",
                "nama modul tidak boleh kosong",
                span,
            ));
        }
        let mut akar = self.impor_satu(bagian[0], span)?;
        let mut daun = akar.clone();
        for i in 1..bagian.len() {
            let lengkap = bagian[..=i].join(".");
            let anak = self.impor_satu(&lengkap, span)?;
            daun.atribut
                .borrow_mut()
                .insert(bagian[i].to_string(), Nilai::Modul(anak.clone()));
            daun = anak;
        }
        if bagian.len() == 1 {
            akar = daun.clone();
        }
        Ok((akar, daun))
    }

    /// Mengimpor satu modul berdasarkan nama.
    fn impor_satu(&mut self, nama: &str, span: Span) -> Hasil<Rc<Modul>> {
        if let Some(modul) = self.modul_cache.get(nama) {
            return Ok(modul.clone());
        }
        if let Some(modul) = modul_bawaan::ambil(self, nama) {
            self.modul_cache.insert(nama.to_string(), modul.clone());
            return Ok(modul);
        }
        let jalur = self.cari_berkas_modul(nama)?;
        let paket = self.paket_dari_berkas(&jalur, nama);
        self.muat_berkas_modul(nama, &jalur, &paket, span)
    }

    /// Mengimpor modul relatif terhadap direktori modul aktif.
    fn impor_relatif(&mut self, module: &str, level: usize, span: Span) -> Hasil<Rc<Modul>> {
        let Some(mut dasar) = self.modul_dir.clone() else {
            return Err(self.lempar_pesan(
                "KesalahanImpor",
                "impor relatif hanya dapat dipakai di dalam berkas modul",
                span,
            ));
        };
        for _ in 1..level {
            dasar = dasar.parent().map(Path::to_path_buf).unwrap_or(dasar);
        }
        let (jalur, nama) = if module.is_empty() {
            let inisialisasi = dasar.join("__init__.aks");
            if !inisialisasi.is_file() {
                return Err(self.lempar_pesan(
                    "KesalahanImpor",
                    format!(
                        "paket `{}` tidak memiliki `__init__.aks`",
                        dasar.display()
                    ),
                    span,
                ));
            }
            let nama = dasar
                .file_name()
                .map(|nama| nama.to_string_lossy().to_string())
                .unwrap_or_else(|| "__init__".to_string());
            (inisialisasi, nama)
        } else {
            let komponen: Vec<&str> = module.split('.').filter(|bagian| !bagian.is_empty()).collect();
            let relatif = komponen.join("/");
            let kandidat = [
                dasar.join(format!("{relatif}.aks")),
                dasar.join(&relatif).join("__init__.aks"),
            ];
            match kandidat.iter().find(|jalur| jalur.is_file()) {
                Some(jalur) => (jalur.clone(), module.to_string()),
                None => {
                    return Err(self.lempar_pesan(
                        "KesalahanImpor",
                        format!(
                            "modul relatif `{module}` tidak ditemukan di `{}`",
                            dasar.display()
                        ),
                        span,
                    ))
                }
            }
        };
        let paket = self.paket_dari_berkas(&jalur, &nama);
        self.muat_berkas_modul(&nama, &jalur, &paket, span)
    }

    /// Mencari berkas modul pada jalur pencarian.
    fn cari_berkas_modul(&mut self, nama: &str) -> Hasil<PathBuf> {
        let komponen: Vec<&str> = nama.split('.').filter(|bagian| !bagian.is_empty()).collect();
        let relatif = komponen.join("/");
        let mut dasar: Vec<PathBuf> = self.jalur.clone();
        if let Ok(direktori) = std::env::current_dir() {
            if !dasar.contains(&direktori) {
                dasar.push(direktori);
            }
        }
        dasar.push(PathBuf::from("pustaka"));
        let mut dicoba = Vec::new();
        for basis in &dasar {
            let kandidat = [
                basis.join(format!("{relatif}.aks")),
                basis.join(&relatif).join("__init__.aks"),
            ];
            for jalur in kandidat {
                if jalur.is_file() {
                    return Ok(jalur);
                }
                dicoba.push(jalur);
            }
        }
        let daftar = dicoba
            .iter()
            .take(4)
            .map(|jalur| jalur.display().to_string())
            .collect::<Vec<_>>()
            .join(", ");
        Err(self.lempar_pesan(
            "KesalahanImpor",
            format!(
                "modul `{nama}` tidak ditemukan. Jalur yang dicoba: {daftar}. Gunakan `AKSARA_PATH` untuk menambah direktori pencarian."
            ),
            Span::empty(0),
        ))
    }

    /// Menentukan nama paket dari letak berkas.
    fn paket_dari_berkas(&self, jalur: &Path, nama: &str) -> String {
        if jalur.file_name().is_some_and(|berkas| berkas == "__init__.aks") {
            return nama.to_string();
        }
        if let Some((paket, _)) = nama.rsplit_once('.') {
            return paket.to_string();
        }
        let mut bagian = Vec::new();
        let mut direktori = jalur.parent();
        while let Some(ini) = direktori {
            if ini.join("__init__.aks").is_file() {
                if let Some(nama) = ini.file_name() {
                    bagian.push(nama.to_string_lossy().to_string());
                }
                direktori = ini.parent();
            } else {
                break;
            }
        }
        bagian.reverse();
        bagian.join(".")
    }

    /// Memuat dan menjalankan berkas modul, lalu menyimpan atributnya.
    fn muat_berkas_modul(
        &mut self,
        nama: &str,
        jalur: &Path,
        paket: &str,
        span: Span,
    ) -> Hasil<Rc<Modul>> {
        let kunci = std::fs::canonicalize(jalur)
            .unwrap_or_else(|_| jalur.to_path_buf())
            .to_string_lossy()
            .to_string();
        if let Some(modul) = self.modul_cache.get(&kunci) {
            return Ok(modul.clone());
        }
        let sumber = match std::fs::read_to_string(jalur) {
            Ok(sumber) => sumber,
            Err(galat) => {
                return Err(self.lempar_pesan(
                    "KesalahanBerkas",
                    format!("tidak dapat membaca `{}`: {galat}", jalur.display()),
                    span,
                ))
            }
        };
        let modul = Rc::new(Modul {
            nama: nama.to_string(),
            atribut: std::cell::RefCell::new(HashMap::new()),
            doc: None,
            berkas: Some(jalur.display().to_string()),
        });
        self.modul_cache.insert(kunci.clone(), modul.clone());
        self.modul_cache.insert(nama.to_string(), modul.clone());

        let env = Lingkungan::anak(self.bawaan.clone());
        let doc = crate::interpreter::dokumen_modul_publik(&sumber);
        {
            let mut e = env.borrow_mut();
            e.atur_lokal("__nama__", Nilai::teks(nama));
            e.atur_lokal(
                "__berkas__",
                Nilai::teks(jalur.display().to_string()),
            );
            e.atur_lokal("__paket__", Nilai::teks(paket));
            e.atur_lokal(
                "__dok__",
                doc.clone().map(Nilai::teks).unwrap_or(Nilai::Kosong),
            );
            e.atur_lokal("__semua__", Nilai::Kosong);
        }

        let nama_berkas_lama = self.nama_berkas.clone();
        let nama_modul_lama = self.nama_modul.clone();
        let nama_paket_lama = self.nama_paket.clone();
        let modul_dir_lama = self.modul_dir.clone();
        let jalur_lama = self.jalur.clone();
        let global_lama = self.global_aktif.clone();

        self.nama_berkas = jalur.display().to_string();
        self.nama_modul = nama.to_string();
        self.nama_paket = paket.to_string();
        self.modul_dir = jalur.parent().map(Path::to_path_buf);
        if let Some(direktori) = &self.modul_dir {
            if !self.jalur.contains(direktori) {
                self.jalur.insert(0, direktori.clone());
            }
        }
        self.global_aktif = env.clone();
        self.impor_aktif.push(nama.to_string());

        let hasil = self.jalankan_teks(&jalur.display().to_string(), &sumber, env.clone());

        self.impor_aktif.pop();
        self.global_aktif = global_lama;
        self.jalur = jalur_lama;
        self.modul_dir = modul_dir_lama;
        self.nama_paket = nama_paket_lama;
        self.nama_modul = nama_modul_lama;
        self.nama_berkas = nama_berkas_lama;

        if let Err(lemparan) = hasil {
            self.modul_cache.remove(&kunci);
            self.modul_cache.remove(nama);
            return Err(lemparan);
        }

        {
            let variabel = env.borrow().variabel.clone();
            let mut atribut = modul.atribut.borrow_mut();
            for (nama, nilai) in variabel {
                atribut.insert(nama, nilai);
            }
            atribut.insert("__nama__".to_string(), Nilai::teks(nama));
            atribut.insert(
                "__berkas__".to_string(),
                Nilai::teks(jalur.display().to_string()),
            );
            atribut.insert("__paket__".to_string(), Nilai::teks(paket));
            atribut.insert(
                "__dok__".to_string(),
                doc.clone().map(Nilai::teks).unwrap_or(Nilai::Kosong),
            );
            if !atribut.contains_key("__semua__") {
                atribut.insert("__semua__".to_string(), Nilai::Kosong);
            }
        }
        let _ = doc;
        Ok(modul)
    }

    /// Mengembalikan daftar nama modul yang sudah dimuat.
    pub fn modul_dimuat(&self) -> Vec<String> {
        let mut nama: Vec<String> = self.modul_cache.keys().cloned().collect();
        nama.sort();
        nama
    }

    /// Menulis modul ke cache dengan nama tertentu.
    pub fn daftarkan_modul(&mut self, nama: &str, modul: Rc<Modul>) {
        self.modul_cache.insert(nama.to_string(), modul);
    }

    /// Mencari nilai atribut modul tanpa melempar kesalahan.
    pub fn atribut_modul(&self, modul: &Modul, nama: &str) -> Option<Nilai> {
        modul.atribut.borrow().get(nama).cloned()
    }

    /// Menambahkan atribut ke modul.
    pub fn atur_atribut_modul(&mut self, modul: &Modul, nama: &str, nilai: Nilai) {
        modul
            .atribut
            .borrow_mut()
            .insert(nama.to_string(), nilai);
    }

    /// Menyalin seluruh nama publik dari modul ke lingkungan.
    pub fn impor_bintang(&mut self, modul: &Rc<Modul>, env: &Env) {
        for nama in modul_bawaan::nama_publik(modul) {
            if let Some(nilai) = modul.atribut.borrow().get(&nama).cloned() {
                self.atur_nama(env, nama, nilai);
            }
        }
    }

    /// Memeriksa apakah nama adalah modul native.
    pub fn adalah_modul_native(nama: &str) -> bool {
        modul_bawaan::daftar_modul().contains(&nama)
    }

    /// Mengembalikan nama modul sebagai teks untuk diagnostik.
    pub fn nama_modul_aktif(&self) -> &str {
        &self.nama_modul
    }

    /// Mengembalikan paket modul aktif.
    pub fn paket_aktif(&self) -> &str {
        &self.nama_paket
    }

    /// Menyusun jalur pencarian tambahan.
    pub fn tambah_jalur(&mut self, jalur: impl Into<PathBuf>) {
        let jalur = jalur.into();
        if !self.jalur.contains(&jalur) {
            self.jalur.push(jalur);
        }
    }

    /// Menghapus modul dari cache berdasarkan nama.
    pub fn lupakan_modul(&mut self, nama: &str) {
        self.modul_cache.remove(nama);
    }

    /// Mengosongkan cache modul.
    pub fn bersihkan_modul(&mut self) {
        self.modul_cache.clear();
    }

    /// Mengubah nilai menjadi nama modul bila memungkinkan.
    pub fn nama_dari_nilai(&self, nilai: &Nilai) -> Option<String> {
        match nilai {
            Nilai::Modul(modul) => Some(modul.nama.clone()),
            Nilai::Teks(teks) => Some(teks.to_string()),
            _ => None,
        }
    }

    /// Mengambil modul dari nilai atau nama.
    pub fn modul_dari_nilai(&self, nilai: &Nilai) -> Option<Rc<Modul>> {
        match nilai {
            Nilai::Modul(modul) => Some(modul.clone()),
            Nilai::Teks(teks) => self.modul_cache.get(teks.as_ref()).cloned(),
            _ => None,
        }
    }

    /// Menambahkan metadata modul ke nilai modul.
    pub fn lengkapi_metadata(&mut self, modul: &Rc<Modul>, paket: &str) {
        let mut atribut = modul.atribut.borrow_mut();
        atribut
            .entry("__nama__".to_string())
            .or_insert_with(|| Nilai::teks(modul.nama.clone()));
        atribut
            .entry("__paket__".to_string())
            .or_insert_with(|| Nilai::teks(paket));
        if let Some(berkas) = &modul.berkas {
            atribut
                .entry("__berkas__".to_string())
                .or_insert_with(|| Nilai::teks(berkas.clone()));
        }
    }

    /// Menyalin nilai modul ke kamus.
    pub fn modul_ke_kamus(&mut self, modul: &Rc<Modul>) -> Nilai {
        let atribut = modul.atribut.borrow().clone();
        Nilai::kamus(
            atribut
                .into_iter()
                .map(|(nama, nilai)| (Nilai::teks(nama), nilai))
                .collect(),
        )
    }

    /// Mengambil nama publik sebagai teks.
    pub fn nama_publik_modul(&self, modul: &Rc<Modul>) -> Vec<String> {
        modul_bawaan::nama_publik(modul)
    }

    /// Mengambil dokumentasi modul.
    pub fn dokumen_modul(&self, modul: &Rc<Modul>) -> Option<String> {
        modul.doc.clone()
    }

    /// Mengambil jalur berkas modul.
    pub fn berkas_modul(&self, modul: &Rc<Modul>) -> Option<String> {
        modul.berkas.clone()
    }

    /// Menentukan apakah modul adalah paket.
    pub fn adalah_paket(&self, modul: &Rc<Modul>) -> bool {
        modul
            .berkas
            .as_deref()
            .is_some_and(|berkas| berkas.ends_with("__init__.aks"))
    }

    /// Menyusun nama atribut submodul.
    pub fn nama_submodul(modul: &Rc<Modul>, nama: &str) -> String {
        format!("{}.{}", modul.nama, nama)
    }

    /// Menggabungkan jalur pencarian modul.
    pub fn jalur_pencarian(&self) -> Vec<PathBuf> {
        self.jalur.clone()
    }

    /// Menormalkan nama modul.
    pub fn normal_modul(nama: &str) -> String {
        nama.trim().trim_matches('.').to_string()
    }

    /// Menampilkan nama modul dalam bentuk teks.
    pub fn teks_modul(nilai: &Nilai) -> Option<String> {
        match nilai {
            Nilai::Modul(modul) => Some(modul.nama.clone()),
            _ => None,
        }
    }

    /// Menambahkan atribut __semua__.
    pub fn atur_semua(&mut self, modul: &Rc<Modul>, nama: Vec<String>) {
        modul.atribut.borrow_mut().insert(
            "__semua__".to_string(),
            Nilai::daftar(nama.into_iter().map(Nilai::teks).collect()),
        );
    }

    /// Mengambil atribut __semua__.
    pub fn semua_modul(&self, modul: &Rc<Modul>) -> Vec<String> {
        match modul.atribut.borrow().get("__semua__") {
            Some(Nilai::Daftar(item)) => item.borrow().iter().map(nilai_ke_teks).collect(),
            _ => Vec::new(),
        }
    }
}
