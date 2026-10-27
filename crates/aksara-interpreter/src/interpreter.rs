use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::rc::Rc;

use aksara_ast::*;
use aksara_diagnostik::SourceFile;
use aksara_lexer::Lexer;
use aksara_parser::parse_tokens;

use crate::bawaan;
use crate::format::{format_nilai, nilai_ke_teks};
use crate::nilai::*;
use crate::pola;

/// Hasil eksekusi satu program.
#[derive(Clone, Debug)]
pub struct HasilJalan {
    pub nilai: Nilai,
    pub keluaran: String,
    pub kesalahan: Option<Rc<Kesalahan>>,
    pub jejak: String,
}

impl HasilJalan {
    pub fn sukses(&self) -> bool {
        self.kesalahan.is_none()
    }
}

/// Alur kendali non-lokal di dalam satu blok.
pub enum Alur {
    Normal,
    Hentikan,
    Lanjutkan,
    Kembalikan(Nilai),
}

impl Alur {
    pub fn normal(&self) -> bool {
        matches!(self, Alur::Normal)
    }
}

/// Satu bingkai panggilan untuk jejak kesalahan.
#[derive(Clone, Debug)]
pub struct Bingkai {
    pub nama: String,
    pub berkas: String,
    pub baris: usize,
    pub kolom: usize,
    pub diri: Option<Nilai>,
    pub kelas: Option<Rc<Kelas>>,
}

/// Interpreter referensi Aksara.
pub struct Interpreter {
    pub global: Env,
    /// Lingkungan global modul yang sedang dieksekusi.
    pub global_aktif: Env,
    pub bawaan: Env,
    pub kelas_bawaan: HashMap<String, Rc<Kelas>>,
    pub modul_cache: HashMap<String, Rc<Modul>>,
    pub jalur: Vec<PathBuf>,
    pub keluaran: String,
    pub nama_berkas: String,
    pub nama_modul: String,
    pub nama_paket: String,
    pub tumpukan: Vec<Bingkai>,
    pub kedalaman: usize,
    pub batas_kedalaman: usize,
    pub kesalahan_aktif: Option<Rc<Kesalahan>>,
    pub buffer_yield: Vec<Vec<Nilai>>,
    pub acak: u64,
    pub argumen: Vec<String>,
    pub sumber: Option<SourceFile>,
    pub modul_utama: Option<Rc<Modul>>,
    pub impor_aktif: Vec<String>,
    /// Direktori modul yang sedang dijalankan, dipakai impor relatif.
    pub modul_dir: Option<PathBuf>,
    /// Span pemanggilan terakhir, dipakai bawaan untuk diagnostik.
    pub span_aktif: Span,
}

impl Interpreter {
    pub fn baru() -> Self {
        let bawaan = Lingkungan::baru();
        let global = Lingkungan::anak(bawaan.clone());
        let mut this = Self {
            global_aktif: global.clone(),
            global,
            bawaan,
            kelas_bawaan: HashMap::new(),
            modul_cache: HashMap::new(),
            jalur: vec![PathBuf::from(".")],
            keluaran: String::new(),
            nama_berkas: "<stdin>".to_string(),
            nama_modul: "__utama__".to_string(),
            nama_paket: String::new(),
            tumpukan: Vec::new(),
            kedalaman: 0,
            batas_kedalaman: 1000,
            kesalahan_aktif: None,
            buffer_yield: Vec::new(),
            acak: 0x9E37_79B9_7F4A_7C15,
            argumen: Vec::new(),
            sumber: None,
            modul_utama: None,
            impor_aktif: Vec::new(),
            modul_dir: None,
            span_aktif: Span::empty(0),
        };
        let env_bawaan = this.bawaan.clone();
        bawaan::pasang(&mut this, &env_bawaan);
        this.siapkan_utama();
        this
    }

    fn siapkan_utama(&mut self) {
        let modul = Rc::new(Modul {
            nama: "__utama__".to_string(),
            atribut: std::cell::RefCell::new(HashMap::new()),
            doc: None,
            berkas: None,
        });
        self.modul_utama = Some(modul);
        let env = self.global.clone();
        let mut e = env.borrow_mut();
        e.atur_lokal("__nama__", Nilai::teks("__utama__"));
        e.atur_lokal("__berkas__", Nilai::teks("<stdin>"));
        e.atur_lokal("__paket__", Nilai::teks(""));
        e.atur_lokal("__dok__", Nilai::Kosong);
        e.atur_lokal("__semua__", Nilai::daftar(Vec::new()));
    }

    /// Menjalankan sumber sebagai program utama.
    pub fn jalankan_sumber(&mut self, nama: &str, sumber: &str) -> HasilJalan {
        self.keluaran.clear();
        self.nama_berkas = nama.to_string();
        self.nama_modul = "__utama__".to_string();
        self.nama_paket = String::new();
        self.sumber = Some(SourceFile::new(nama, sumber));
        self.siapkan_metadata_utama(nama, sumber);
        let env = self.global.clone();
        let hasil = self.jalankan_teks(nama, sumber, env);
        self.hasil_ke_jalan(hasil)
    }

    fn siapkan_metadata_utama(&mut self, nama: &str, sumber: &str) {
        let doc = dokumen_modul_sumber(sumber);
        let env = self.global.clone();
        let mut e = env.borrow_mut();
        e.atur_lokal("__nama__", Nilai::teks("__utama__"));
        e.atur_lokal("__berkas__", Nilai::teks(nama));
        e.atur_lokal("__paket__", Nilai::teks(""));
        e.atur_lokal("__dok__", doc.map(Nilai::teks).unwrap_or(Nilai::Kosong));
        e.atur_lokal("__semua__", Nilai::Kosong);
    }

    /// Menjalankan sumber di lingkungan tertentu.
    pub fn jalankan_teks(&mut self, nama: &str, sumber: &str, env: Env) -> Hasil<Nilai> {
        let source = SourceFile::new(nama, sumber);
        let lexed = Lexer::new(&source.text).tokenize();
        let parsed = parse_tokens(lexed.tokens, lexed.diagnostics);
        let galat = parsed
            .diagnostics
            .iter()
            .filter(|diagnostik| diagnostik.is_error())
            .map(|diagnostik| {
                let (baris, kolom) = source.line_col(diagnostik.span.start as usize);
                format!(
                    "{}:{}:{}: {} [{}]",
                    nama, baris, kolom, diagnostik.message, diagnostik.code
                )
            })
            .collect::<Vec<_>>();
        if !galat.is_empty() {
            let kesalahan = self.kesalahan_baru("KesalahanSintaks", galat.join("\n"));
            return Err(Lemparan::baru(kesalahan, Span::empty(0)));
        }
        let sumber_lama = self.sumber.replace(source);
        #[cfg(feature = "python")]
        let _aktif = crate::python::Aktif::baru(self);
        let hasil = self.exec_block(&parsed.module.body, &env);
        self.sumber = sumber_lama;
        match hasil {
            Ok(Alur::Kembalikan(nilai)) => Ok(nilai),
            Ok(_) => Ok(Nilai::Kosong),
            Err(lemparan) => Err(lemparan),
        }
    }

    fn hasil_ke_jalan(&mut self, hasil: Hasil<Nilai>) -> HasilJalan {
        match hasil {
            Ok(nilai) => HasilJalan {
                nilai,
                keluaran: self.keluaran.clone(),
                kesalahan: None,
                jejak: String::new(),
            },
            Err(lemparan) => {
                self.catat_jejak(&lemparan.kesalahan, lemparan.span);
                let jejak = self.format_jejak(&lemparan.kesalahan);
                HasilJalan {
                    nilai: Nilai::Kosong,
                    keluaran: self.keluaran.clone(),
                    kesalahan: Some(lemparan.kesalahan),
                    jejak,
                }
            }
        }
    }

    /// Menjalankan berkas `.aks`.
    pub fn jalankan_berkas(&mut self, jalur: &Path) -> HasilJalan {
        match std::fs::read_to_string(jalur) {
            Ok(sumber) => {
                let direktori = jalur.parent().unwrap_or(Path::new(".")).to_path_buf();
                self.modul_dir = Some(direktori.clone());
                self.jalur = vec![direktori, PathBuf::from(".")];
                if let Ok(ekstra) = std::env::var("AKSARA_PATH") {
                    for bagian in ekstra.split(':').filter(|bagian| !bagian.is_empty()) {
                        self.jalur.push(PathBuf::from(bagian));
                    }
                }
                self.jalankan_sumber(&jalur.display().to_string(), &sumber)
            }
            Err(galat) => {
                let kesalahan = self.kesalahan_baru(
                    "KesalahanBerkas",
                    format!("tidak dapat membaca `{}`: {galat}", jalur.display()),
                );
                HasilJalan {
                    nilai: Nilai::Kosong,
                    keluaran: self.keluaran.clone(),
                    kesalahan: Some(kesalahan),
                    jejak: String::new(),
                }
            }
        }
    }

    pub fn kesalahan_baru(&mut self, tipe: &str, pesan: impl Into<String>) -> Rc<Kesalahan> {
        let kelas = self
            .kelas_bawaan
            .get(tipe)
            .cloned()
            .or_else(|| self.kelas_bawaan.get("Kesalahan").cloned())
            .unwrap_or_else(|| Kelas::baru_jenis(tipe, Vec::new(), JenisKelas::Kesalahan));
        Kesalahan::dari_pesan(kelas, pesan)
    }

    pub fn lempar_pesan(
        &mut self,
        tipe: &str,
        pesan: impl Into<String>,
        span: Span,
    ) -> Lemparan {
        let kesalahan = self.kesalahan_baru(tipe, pesan);
        Lemparan::baru(kesalahan, span)
    }

    pub fn posisi(&self, span: Span) -> (usize, usize, String) {
        match &self.sumber {
            Some(sumber) => {
                let (baris, kolom) = sumber.line_col(span.start as usize);
                (baris, kolom, sumber.line_text(baris).to_string())
            }
            None => (0, 0, String::new()),
        }
    }

    pub fn catat_jejak(&self, kesalahan: &Rc<Kesalahan>, span: Span) {
        let (baris, kolom, teks) = self.posisi(span);
        let fungsi = self
            .tumpukan
            .last()
            .map(|bingkai| bingkai.nama.clone())
            .unwrap_or_else(|| "<modul>".to_string());
        kesalahan.jejak.borrow_mut().push(Jejak {
            berkas: self.nama_berkas.clone(),
            baris,
            kolom,
            fungsi,
            teks,
        });
    }

    pub fn format_jejak(&self, kesalahan: &Kesalahan) -> String {
        let mut hasil = String::new();
        hasil.push_str(&format!("{}: {}\n", kesalahan.tipe.nama, kesalahan.pesan));
        let jejak = kesalahan.jejak.borrow();
        if !jejak.is_empty() {
            hasil.push_str("Jejak panggilan (paling baru di akhir):\n");
        }
        for bingkai in jejak.iter().rev() {
            hasil.push_str(&format!(
                "  Berkas \"{}\", baris {}, kolom {}, di {}\n",
                bingkai.berkas, bingkai.baris, bingkai.kolom, bingkai.fungsi
            ));
            if !bingkai.teks.is_empty() {
                hasil.push_str(&format!("    {}\n", bingkai.teks.trim()));
            }
        }
        hasil
    }

    pub fn tulis(&mut self, teks: impl AsRef<str>) {
        self.keluaran.push_str(teks.as_ref());
    }

    pub fn tulis_baris(&mut self, teks: impl AsRef<str>) {
        self.keluaran.push_str(teks.as_ref());
        self.keluaran.push('\n');
    }

    // ------------------------------------------------------------------
    // Lingkungan dan penugasan
    // ------------------------------------------------------------------

    pub fn cari_nama(&mut self, env: &Env, nama: &str, span: Span) -> Hasil<Nilai> {
        if let Some(nilai) = env.borrow().cari(nama) {
            return Ok(nilai);
        }
        Err(self.lempar_pesan(
            "KesalahanNama",
            format!("nama `{nama}` belum didefinisikan"),
            span,
        ))
    }

    pub fn atur_nama(&mut self, env: &Env, nama: impl Into<String>, nilai: Nilai) {
        let nama = nama.into();
        let (global, nonlokal) = {
            let e = env.borrow();
            (
                e.global.contains(&nama),
                e.nonlokal.contains(&nama),
            )
        };
        if global {
            self.global_aktif.borrow_mut().atur_lokal(nama, nilai);
            return;
        }
        if nonlokal {
            let mut kandidat = env.borrow().induk.clone();
            while let Some(calon) = kandidat {
                let adalah_fungsi = calon.borrow().adalah_fungsi;
                if adalah_fungsi {
                    calon.borrow_mut().atur_lokal(nama, nilai);
                    return;
                }
                kandidat = calon.borrow().induk.clone();
            }
        }
        env.borrow_mut().atur_lokal(nama, nilai);
    }

    pub fn hapus_nama(&mut self, env: &Env, nama: &str, span: Span) -> Hasil<()> {
        let dihapus = {
            let mut e = env.borrow_mut();
            if e.global.contains(nama) {
                self.global.borrow_mut().hapus_lokal(nama)
            } else {
                e.hapus_lokal(nama)
            }
        };
        if dihapus.is_some() {
            Ok(())
        } else {
            Err(self.lempar_pesan(
                "KesalahanNama",
                format!("nama `{nama}` tidak dapat dihapus karena belum didefinisikan"),
                span,
            ))
        }
    }

    fn atur_target(&mut self, target: &Expr, nilai: Nilai, env: &Env) -> Hasil<()> {
        match &target.kind {
            ExprKind::Name(nama) => {
                self.atur_nama(env, nama.clone(), nilai);
                Ok(())
            }
            ExprKind::Tuple(elemen) | ExprKind::List(elemen) => {
                self.atur_unpack(elemen, nilai, env, target.span)
            }
            ExprKind::Attribute { value, attr, safe } => {
                let objek = self.eval(value, env)?;
                if *safe && objek.adalah_kosong() {
                    return Ok(());
                }
                self.atur_atribut(&objek, attr, nilai, target.span)
            }
            ExprKind::Subscript { value, index, safe } => {
                let objek = self.eval(value, env)?;
                if *safe && objek.adalah_kosong() {
                    return Ok(());
                }
                let kunci = self.eval(index, env)?;
                self.atur_indeks(&objek, &kunci, nilai, target.span)
            }
            ExprKind::Starred(inner) => self.atur_target(inner, nilai, env),
            _ => Err(self.lempar_pesan(
                "KesalahanSintaks",
                "target penugasan tidak valid",
                target.span,
            )),
        }
    }

    fn atur_unpack(
        &mut self,
        target: &[Expr],
        nilai: Nilai,
        env: &Env,
        span: Span,
    ) -> Hasil<()> {
        let item = self.iter_nilai(&nilai, span)?;
        let bintang = target
            .iter()
            .position(|expr| matches!(expr.kind, ExprKind::Starred(_)));
        match bintang {
            None => {
                if item.len() != target.len() {
                    return Err(self.lempar_pesan(
                        "KesalahanNilai",
                        format!(
                            "tidak dapat membongkar {} nilai ke {} target",
                            item.len(),
                            target.len()
                        ),
                        span,
                    ));
                }
                for (target, nilai) in target.iter().zip(item) {
                    self.atur_target(target, nilai, env)?;
                }
                Ok(())
            }
            Some(posisi) => {
                let minimum = target.len() - 1;
                if item.len() < minimum {
                    return Err(self.lempar_pesan(
                        "KesalahanNilai",
                        format!(
                            "tidak cukup nilai untuk membongkar: butuh minimal {minimum}, ditemukan {}",
                            item.len()
                        ),
                        span,
                    ));
                }
                let ekor = item.len() - minimum;
                let jumlah_target = target.len();
                for (indeks, target_item) in target.iter().enumerate() {
                    if indeks == posisi {
                        if let ExprKind::Starred(inner) = &target_item.kind {
                            let isi = item[posisi..posisi + ekor].to_vec();
                            self.atur_target(inner, Nilai::daftar(isi), env)?;
                        }
                    } else if indeks < posisi {
                        self.atur_target(target_item, item[indeks].clone(), env)?;
                    } else {
                        let dari_akhir = jumlah_target - indeks;
                        self.atur_target(target_item, item[item.len() - dari_akhir].clone(), env)?;
                    }
                }
                Ok(())
            }
        }
    }

    fn baca_target(&mut self, target: &Expr, env: &Env) -> Hasil<Nilai> {
        match &target.kind {
            ExprKind::Name(nama) => self.cari_nama(env, nama, target.span),
            ExprKind::Attribute { value, attr, .. } => {
                let objek = self.eval(value, env)?;
                self.ambil_atribut(&objek, attr, target.span)
            }
            ExprKind::Subscript { value, index, .. } => {
                let objek = self.eval(value, env)?;
                let kunci = self.eval(index, env)?;
                self.ambil_indeks(&objek, &kunci, target.span)
            }
            _ => Err(self.lempar_pesan(
                "KesalahanSintaks",
                "target tidak dapat dibaca",
                target.span,
            )),
        }
    }

    // ------------------------------------------------------------------
    // Eksekusi pernyataan
    // ------------------------------------------------------------------

    pub fn exec_block(&mut self, body: &[Stmt], env: &Env) -> Hasil<Alur> {
        for stmt in body {
            let alur = self.exec_stmt(stmt, env)?;
            if !alur.normal() {
                return Ok(alur);
            }
        }
        Ok(Alur::Normal)
    }

    pub fn exec_stmt(&mut self, stmt: &Stmt, env: &Env) -> Hasil<Alur> {
        match &stmt.kind {
            StmtKind::Expr(expr) => {
                self.eval(expr, env)?;
                Ok(Alur::Normal)
            }
            StmtKind::Assign { targets, value } => {
                let nilai = self.eval(value, env)?;
                for target in targets {
                    self.atur_target(target, nilai.clone(), env)?;
                }
                Ok(Alur::Normal)
            }
            StmtKind::AnnAssign {
                target,
                annotation,
                value,
            } => {
                let _ = self.eval(annotation, env)?;
                if let Some(value) = value {
                    let nilai = self.eval(value, env)?;
                    self.atur_target(target, nilai, env)?;
                }
                Ok(Alur::Normal)
            }
            StmtKind::AugAssign { target, op, value } => {
                let kiri = self.baca_target(target, env)?;
                let kanan = self.eval(value, env)?;
                let hasil = self.augment(&kiri, *op, kanan, stmt.span)?;
                self.atur_target(target, hasil, env)?;
                Ok(Alur::Normal)
            }
            StmtKind::If {
                condition,
                body,
                orelse,
            } => {
                let syarat = self.eval(condition, env)?;
                if self.kebenaran(&syarat)? {
                    self.exec_block(body, env)
                } else {
                    self.exec_block(orelse, env)
                }
            }
            StmtKind::While {
                condition,
                body,
                orelse,
            } => {
                let mut henti = false;
                loop {
                    let syarat = self.eval(condition, env)?;
                    if !self.kebenaran(&syarat)? {
                        break;
                    }
                    match self.exec_block(body, env)? {
                        Alur::Normal | Alur::Lanjutkan => {}
                        Alur::Hentikan => {
                            henti = true;
                            break;
                        }
                        alur @ Alur::Kembalikan(_) => return Ok(alur),
                    }
                }
                if !henti && !orelse.is_empty() {
                    return self.exec_block(orelse, env);
                }
                Ok(Alur::Normal)
            }
            StmtKind::For {
                target,
                iter,
                body,
                orelse,
                ..
            } => {
                let daftar = self.iter_untuk(iter, env)?;
                let mut henti = false;
                for nilai in daftar {
                    self.atur_target(target, nilai, env)?;
                    match self.exec_block(body, env)? {
                        Alur::Normal | Alur::Lanjutkan => {}
                        Alur::Hentikan => {
                            henti = true;
                            break;
                        }
                        alur @ Alur::Kembalikan(_) => return Ok(alur),
                    }
                }
                if !henti && !orelse.is_empty() {
                    return self.exec_block(orelse, env);
                }
                Ok(Alur::Normal)
            }
            StmtKind::Function(def) => {
                let fungsi = self.buat_fungsi(def, env)?;
                self.atur_nama(env, def.name.clone(), fungsi);
                Ok(Alur::Normal)
            }
            StmtKind::Class(def) => {
                let kelas = self.buat_kelas(def, env)?;
                self.atur_nama(env, def.name.clone(), kelas);
                Ok(Alur::Normal)
            }
            StmtKind::Return(value) => {
                let nilai = match value {
                    Some(expr) => self.eval(expr, env)?,
                    None => Nilai::Kosong,
                };
                Ok(Alur::Kembalikan(nilai))
            }
            StmtKind::Yield { value, from } => {
                let nilai = match value {
                    Some(expr) => self.eval(expr, env)?,
                    None => Nilai::Kosong,
                };
                if *from {
                    let item = self.iter_nilai(&nilai, stmt.span)?;
                    if let Some(buffer) = self.buffer_yield.last_mut() {
                        buffer.extend(item);
                    } else {
                        return Err(self.lempar_pesan(
                            "KesalahanRuntime",
                            "`hasilkan dari` hanya boleh dipakai di dalam fungsi generator",
                            stmt.span,
                        ));
                    }
                } else if let Some(buffer) = self.buffer_yield.last_mut() {
                    buffer.push(nilai);
                } else {
                    return Err(self.lempar_pesan(
                        "KesalahanRuntime",
                        "`hasilkan` hanya boleh dipakai di dalam fungsi generator",
                        stmt.span,
                    ));
                }
                Ok(Alur::Normal)
            }
            StmtKind::Break => Ok(Alur::Hentikan),
            StmtKind::Continue => Ok(Alur::Lanjutkan),
            StmtKind::Pass => Ok(Alur::Normal),
            StmtKind::Try {
                body,
                handlers,
                orelse,
                finalbody,
            } => self.exec_try(body, handlers, orelse, finalbody, env, stmt.span),
            StmtKind::Raise { exc, cause } => {
                let kesalahan = match exc {
                    Some(expr) => {
                        let nilai = self.eval(expr, env)?;
                        self.nilai_ke_kesalahan(nilai, stmt.span)?
                    }
                    None => self.kesalahan_aktif.clone().ok_or_else(|| {
                        self.lempar_pesan(
                            "KesalahanRuntime",
                            "`lempar` tanpa nilai hanya boleh di dalam `tangkap`",
                            stmt.span,
                        )
                    })?,
                };
                if let Some(cause) = cause {
                    let nilai = self.eval(cause, env)?;
                    if !nilai.adalah_kosong() {
                        let penyebab = self.nilai_ke_kesalahan(nilai, stmt.span)?;
                        let kesalahan_baru = Rc::new(Kesalahan {
                            tipe: kesalahan.tipe.clone(),
                            pesan: kesalahan.pesan.clone(),
                            argumen: kesalahan.argumen.clone(),
                            penyebab: Some(penyebab),
                            atribut: std::cell::RefCell::new(
                                kesalahan.atribut.borrow().clone(),
                            ),
                            jejak: std::cell::RefCell::new(
                                kesalahan.jejak.borrow().clone(),
                            ),
                        });
                        return Err(Lemparan::baru(kesalahan_baru, stmt.span));
                    }
                }
                Err(Lemparan::baru(kesalahan, stmt.span))
            }
            StmtKind::With { items, body, .. } => self.exec_with(items, body, env, stmt.span),
            StmtKind::Import { modules } => {
                for alias in modules {
                    self.impor_alias(alias, env, stmt.span)?;
                }
                Ok(Alur::Normal)
            }
            StmtKind::FromImport {
                module,
                level,
                names,
                python,
            } => {
                self.impor_dari(module, *level, names, *python, env, stmt.span)?;
                Ok(Alur::Normal)
            }
            StmtKind::Global(names) => {
                let mut e = env.borrow_mut();
                for nama in names {
                    e.global.insert(nama.clone());
                }
                Ok(Alur::Normal)
            }
            StmtKind::Nonlocal(names) => {
                let mut e = env.borrow_mut();
                for nama in names {
                    e.nonlokal.insert(nama.clone());
                }
                Ok(Alur::Normal)
            }
            StmtKind::Assert { test, msg } => {
                let nilai = self.eval(test, env)?;
                if self.kebenaran(&nilai)? {
                    Ok(Alur::Normal)
                } else {
                    let pesan = match msg {
                        Some(expr) => nilai_ke_teks(&self.eval(expr, env)?),
                        None => "klaim gagal".to_string(),
                    };
                    Err(self.lempar_pesan("KesalahanKlaim", pesan, stmt.span))
                }
            }
            StmtKind::Delete(targets) => {
                for target in targets {
                    self.hapus_target(target, env)?;
                }
                Ok(Alur::Normal)
            }
            StmtKind::Match { subject, cases } => {
                let subjek = self.eval(subject, env)?;
                for kasus in cases {
                    let mut ikatan = HashMap::new();
                    if pola::cocokkan(self, &kasus.pattern, &subjek, &mut ikatan, env)? {
                        let anak = Lingkungan::anak(env.clone());
                        for (nama, nilai) in ikatan {
                            anak.borrow_mut().atur_lokal(nama, nilai);
                        }
                        if let Some(guard) = &kasus.guard {
                            let nilai_guard = self.eval(guard, &anak)?;
                            if !self.kebenaran(&nilai_guard)? {
                                continue;
                            }
                        }
                        return self.exec_block(&kasus.body, &anak);
                    }
                }
                Ok(Alur::Normal)
            }
        }
    }

    fn exec_try(
        &mut self,
        body: &[Stmt],
        handlers: &[ExceptHandler],
        orelse: &[Stmt],
        finalbody: &[Stmt],
        env: &Env,
        _span: Span,
    ) -> Hasil<Alur> {
        let hasil = match self.exec_block(body, env) {
            Ok(Alur::Normal) if !orelse.is_empty() => self.exec_block(orelse, env),
            Ok(alur) => Ok(alur),
            Err(lemparan) => {
                let lama = self.kesalahan_aktif.replace(lemparan.kesalahan.clone());
                let mut ditangani = None;
                for handler in handlers {
                    if self.handler_cocok(handler, &lemparan.kesalahan, env)? {
                        if let Some(nama) = &handler.name {
                            self.atur_nama(env, nama.clone(), Nilai::Kesalahan(lemparan.kesalahan.clone()));
                        }
                        ditangani = Some(self.exec_block(&handler.body, env));
                        break;
                    }
                }
                self.kesalahan_aktif = lama;
                match ditangani {
                    Some(hasil) => hasil,
                    None => Err(lemparan),
                }
            }
        };

        if finalbody.is_empty() {
            return hasil;
        }
        let akhir = self.exec_block(finalbody, env);
        match akhir {
            Err(lemparan) => Err(lemparan),
            Ok(alur) if !alur.normal() => Ok(alur),
            Ok(_) => hasil,
        }
    }

    fn handler_cocok(
        &mut self,
        handler: &ExceptHandler,
        kesalahan: &Rc<Kesalahan>,
        env: &Env,
    ) -> Hasil<bool> {
        let Some(tipe) = &handler.type_ else {
            return Ok(true);
        };
        let nilai = self.eval(tipe, env)?;
        self.kesalahan_cocok(kesalahan, &nilai, handler.span)
    }

    fn kesalahan_cocok(
        &mut self,
        kesalahan: &Rc<Kesalahan>,
        tipe: &Nilai,
        span: Span,
    ) -> Hasil<bool> {
        match tipe {
            Nilai::Kelas(kelas) => Ok(kesalahan.tipe.adalah_subkelas(kelas)),
            Nilai::Tupel(item) => {
                for calon in item.iter() {
                    if self.kesalahan_cocok(kesalahan, calon, span)? {
                        return Ok(true);
                    }
                }
                Ok(false)
            }
            _ => Err(self.lempar_pesan(
                "KesalahanTipe",
                "tipe pada `tangkap` harus kelas kesalahan atau tupel kelas",
                span,
            )),
        }
    }

    fn nilai_ke_kesalahan(&mut self, nilai: Nilai, span: Span) -> Hasil<Rc<Kesalahan>> {
        match nilai {
            Nilai::Kesalahan(kesalahan) => Ok(kesalahan),
            Nilai::Kelas(kelas) => {
                if !matches!(kelas.jenis, JenisKelas::Kesalahan) {
                    return Err(self.lempar_pesan(
                        "KesalahanTipe",
                        format!("`{}` bukan kelas kesalahan", kelas.nama),
                        span,
                    ));
                }
                Ok(Kesalahan::dari_pesan(kelas, String::new()))
            }
            Nilai::Instance(instance) => {
                let subkelas = instance
                    .kelas
                    .mro
                    .iter()
                    .any(|kelas| kelas.nama == "Kesalahan");
                if !subkelas && instance.kelas.nama != "Kesalahan" {
                    return Err(self.lempar_pesan(
                        "KesalahanTipe",
                        format!("`{}` bukan kelas kesalahan", instance.kelas.nama),
                        span,
                    ));
                }
                let atribut = instance.atribut.borrow().clone();
                let argumen = match atribut.get("argumen") {
                    Some(Nilai::Tupel(item)) => item.as_ref().clone(),
                    _ => Vec::new(),
                };
                let pesan = match atribut.get("pesan") {
                    Some(nilai) => nilai_ke_teks(nilai),
                    None if !argumen.is_empty() => {
                        let mut bagian = Vec::with_capacity(argumen.len());
                        for nilai in &argumen {
                            bagian.push(self.teks_nilai(nilai, span)?);
                        }
                        bagian.join(", ")
                    }
                    None => String::new(),
                };
                let kesalahan = Kesalahan::baru(instance.kelas.clone(), pesan, argumen);
                for (nama, nilai) in atribut {
                    if nama == "argumen" || nama == "pesan" {
                        continue;
                    }
                    kesalahan
                        .atribut
                        .borrow_mut()
                        .insert(nama, nilai);
                }
                Ok(kesalahan)
            }
            other => Err(self.lempar_pesan(
                "KesalahanTipe",
                format!("nilai {} tidak dapat dilempar", other.nama_tipe()),
                span,
            )),
        }
    }

    fn exec_with(
        &mut self,
        items: &[WithItem],
        body: &[Stmt],
        env: &Env,
        span: Span,
    ) -> Hasil<Alur> {
        if items.is_empty() {
            return self.exec_block(body, env);
        }
        let item = &items[0];
        let konteks = self.eval(&item.context, env)?;
        let masuk = self.masuk_konteks(&konteks, span)?;
        if let Some(target) = &item.target {
            self.atur_target(target, masuk, env)?;
        }
        let hasil = if items.len() > 1 {
            self.exec_with(&items[1..], body, env, span)
        } else {
            self.exec_block(body, env)
        };
        match hasil {
            Ok(alur) => {
                self.keluar_konteks(&konteks, Nilai::Kosong, Nilai::Kosong, Nilai::Kosong, span)?;
                Ok(alur)
            }
            Err(lemparan) => {
                let ditekan = self.keluar_konteks(
                    &konteks,
                    Nilai::Kelas(lemparan.kesalahan.tipe.clone()),
                    Nilai::Kesalahan(lemparan.kesalahan.clone()),
                    Nilai::Kosong,
                    span,
                )?;
                if self.kebenaran(&ditekan)? {
                    Ok(Alur::Normal)
                } else {
                    Err(lemparan)
                }
            }
        }
    }

    fn masuk_konteks(&mut self, nilai: &Nilai, span: Span) -> Hasil<Nilai> {
        match nilai {
            Nilai::Berkas(berkas) => {
                let _ = berkas;
                Ok(nilai.clone())
            }
            Nilai::Python(_) => crate::python_masuk(nilai, span),
            _ => match self.ambil_atribut(nilai, "__enter__", span) {
                Ok(fungsi) => self.panggil(&fungsi, Vec::new(), Vec::new(), span),
                Err(_) => match self.ambil_atribut(nilai, "__masuk__", span) {
                    Ok(fungsi) => self.panggil(&fungsi, Vec::new(), Vec::new(), span),
                    Err(_) => Ok(nilai.clone()),
                },
            },
        }
    }

    fn keluar_konteks(
        &mut self,
        nilai: &Nilai,
        tipe: Nilai,
        galat: Nilai,
        jejak: Nilai,
        span: Span,
    ) -> Hasil<Nilai> {
        match nilai {
            Nilai::Berkas(berkas) => {
                if let Some(file) = berkas.borrow_mut().berkas.take() {
                    let _ = file;
                }
                Ok(Nilai::Logika(false))
            }
            Nilai::Python(_) => crate::python_keluar(nilai, tipe, galat, jejak, span),
            _ => match self.ambil_atribut(nilai, "__exit__", span) {
                Ok(fungsi) => {
                    self.panggil(&fungsi, vec![tipe, galat, jejak], Vec::new(), span)
                }
                Err(_) => match self.ambil_atribut(nilai, "__keluar__", span) {
                    Ok(fungsi) => {
                        self.panggil(&fungsi, vec![tipe, galat, jejak], Vec::new(), span)
                    }
                    Err(_) => Ok(Nilai::Logika(false)),
                },
            },
        }
    }

    fn hapus_target(&mut self, target: &Expr, env: &Env) -> Hasil<()> {
        match &target.kind {
            ExprKind::Name(nama) => self.hapus_nama(env, nama, target.span),
            ExprKind::Attribute { value, attr, .. } => {
                let objek = self.eval(value, env)?;
                self.hapus_atribut(&objek, attr, target.span)
            }
            ExprKind::Subscript { value, index, .. } => {
                let objek = self.eval(value, env)?;
                let kunci = self.eval(index, env)?;
                self.hapus_indeks(&objek, &kunci, target.span)
            }
            ExprKind::Tuple(elemen) | ExprKind::List(elemen) => {
                for item in elemen {
                    self.hapus_target(item, env)?;
                }
                Ok(())
            }
            _ => Err(self.lempar_pesan(
                "KesalahanSintaks",
                "target `hapus` tidak valid",
                target.span,
            )),
        }
    }

    fn iter_untuk(&mut self, iter: &ForIter, env: &Env) -> Hasil<Vec<Nilai>> {
        match iter {
            ForIter::In(expr) | ForIter::Each(expr) => {
                let nilai = self.eval(expr, env)?;
                self.iter_nilai(&nilai, expr.span)
            }
            ForIter::Times(expr) => {
                let nilai = self.eval(expr, env)?;
                let jumlah = self.nilai_ke_i64(&nilai, expr.span)?;
                if jumlah < 0 {
                    return Ok(Vec::new());
                }
                Ok((0..jumlah).map(Nilai::Bilangan).collect())
            }
            ForIter::Range { start, end, step } => {
                let nilai_mulai = self.eval(start, env)?;
                let mulai = self.nilai_ke_i64(&nilai_mulai, start.span)?;
                let nilai_akhir = self.eval(end, env)?;
                let akhir = self.nilai_ke_i64(&nilai_akhir, end.span)?;
                let langkah = match step {
                    Some(expr) => {
                        let nilai_langkah = self.eval(expr, env)?;
                        self.nilai_ke_i64(&nilai_langkah, expr.span)?
                    }
                    None => {
                        if mulai <= akhir {
                            1
                        } else {
                            -1
                        }
                    }
                };
                if langkah == 0 {
                    return Err(self.lempar_pesan(
                        "KesalahanNilai",
                        "langkah perulangan tidak boleh nol",
                        step.as_ref().map(|expr| expr.span).unwrap_or(end.span),
                    ));
                }
                let mut hasil = Vec::new();
                let mut nilai = mulai;
                if langkah > 0 {
                    while nilai <= akhir {
                        hasil.push(Nilai::Bilangan(nilai));
                        nilai += langkah;
                    }
                } else {
                    while nilai >= akhir {
                        hasil.push(Nilai::Bilangan(nilai));
                        nilai += langkah;
                    }
                }
                Ok(hasil)
            }
        }
    }

    // ------------------------------------------------------------------
    // Ekspresi
    // ------------------------------------------------------------------

    pub fn eval(&mut self, expr: &Expr, env: &Env) -> Hasil<Nilai> {
        match &expr.kind {
            ExprKind::Name(nama) => self.cari_nama(env, nama, expr.span),
            ExprKind::Int(teks) => self.parse_int(teks, expr.span),
            ExprKind::Float(teks) => self.parse_float(teks, expr.span),
            ExprKind::Imaginary(teks) => self.parse_imaginary(teks, expr.span),
            ExprKind::Str(literal) => match literal.decoded() {
                Some(teks) => Ok(Nilai::teks(teks)),
                None => Err(self.lempar_pesan(
                    "KesalahanSintaks",
                    "literal teks tidak dapat didekode",
                    expr.span,
                )),
            },
            ExprKind::Bytes(literal) => match literal.decoded_bytes() {
                Some(bytes) => Ok(Nilai::Bytes(Rc::new(bytes))),
                None => Err(self.lempar_pesan(
                    "KesalahanSintaks",
                    "literal bytes tidak dapat didekode",
                    expr.span,
                )),
            },
            ExprKind::Bool(nilai) => Ok(Nilai::Logika(*nilai)),
            ExprKind::None => Ok(Nilai::Kosong),
            ExprKind::Ellipsis => Ok(Nilai::Ellipsis),
            ExprKind::List(item) => {
                let mut hasil = Vec::new();
                for expr in item {
                    if let ExprKind::Starred(inner) = &expr.kind {
                        let nilai = self.eval(inner, env)?;
                        hasil.extend(self.iter_nilai(&nilai, expr.span)?);
                    } else {
                        hasil.push(self.eval(expr, env)?);
                    }
                }
                Ok(Nilai::daftar(hasil))
            }
            ExprKind::Tuple(item) => {
                let mut hasil = Vec::new();
                for expr in item {
                    if let ExprKind::Starred(inner) = &expr.kind {
                        let nilai = self.eval(inner, env)?;
                        hasil.extend(self.iter_nilai(&nilai, expr.span)?);
                    } else {
                        hasil.push(self.eval(expr, env)?);
                    }
                }
                Ok(Nilai::tupel(hasil))
            }
            ExprKind::Set(item) => {
                let mut hasil = Vec::new();
                for expr in item {
                    if let ExprKind::Starred(inner) = &expr.kind {
                        let nilai = self.eval(inner, env)?;
                        for item in self.iter_nilai(&nilai, expr.span)? {
                            self.tambah_himpunan(&mut hasil, item);
                        }
                    } else {
                        let nilai = self.eval(expr, env)?;
                        self.tambah_himpunan(&mut hasil, nilai);
                    }
                }
                Ok(Nilai::himpunan(hasil))
            }
            ExprKind::Dict(item) => {
                let mut hasil: Vec<(Nilai, Nilai)> = Vec::new();
                for item in item {
                    match item {
                        DictItem::Pair(kunci, nilai) => {
                            let kunci = self.eval(kunci, env)?;
                            let nilai = self.eval(nilai, env)?;
                            self.atur_kamus(&mut hasil, kunci, nilai);
                        }
                        DictItem::Unpack(expr) => {
                            let nilai = self.eval(expr, env)?;
                            let pasangan = self.kamus_pasangan(&nilai, expr.span)?;
                            for (kunci, nilai) in pasangan {
                                self.atur_kamus(&mut hasil, kunci, nilai);
                            }
                        }
                    }
                }
                Ok(Nilai::kamus(hasil))
            }
            ExprKind::Attribute { value, attr, safe } => {
                let objek = self.eval(value, env)?;
                if *safe && objek.adalah_kosong() {
                    return Ok(Nilai::Kosong);
                }
                self.ambil_atribut(&objek, attr, expr.span)
            }
            ExprKind::Subscript { value, index, safe } => {
                let objek = self.eval(value, env)?;
                if *safe && objek.adalah_kosong() {
                    return Ok(Nilai::Kosong);
                }
                let kunci = self.eval(index, env)?;
                self.ambil_indeks(&objek, &kunci, expr.span)
            }
            ExprKind::Slice { start, stop, step } => {
                let mulai = match start {
                    Some(expr) => {
                        let nilai = self.eval(expr, env)?;
                        Some(self.nilai_ke_i64(&nilai, expr.span)?)
                    }
                    None => None,
                };
                let akhir = match stop {
                    Some(expr) => {
                        let nilai = self.eval(expr, env)?;
                        Some(self.nilai_ke_i64(&nilai, expr.span)?)
                    }
                    None => None,
                };
                let langkah = match step {
                    Some(expr) => {
                        let nilai = self.eval(expr, env)?;
                        Some(self.nilai_ke_i64(&nilai, expr.span)?)
                    }
                    None => None,
                };
                Ok(Nilai::Slice(Rc::new(Slice {
                    mulai,
                    akhir,
                    langkah,
                })))
            }
            ExprKind::Call { func, args, safe } => {
                let fungsi = self.eval(func, env)?;
                if *safe && fungsi.adalah_kosong() {
                    return Ok(Nilai::Kosong);
                }
                let (posisional, kata_kunci) = self.eval_argumen(args, env)?;
                self.panggil(&fungsi, posisional, kata_kunci, expr.span)
            }
            ExprKind::Unary { op, operand } => {
                let nilai = self.eval(operand, env)?;
                self.operasi_uner(*op, nilai, expr.span)
            }
            ExprKind::Binary { left, op, right } => {
                let kiri = self.eval(left, env)?;
                if matches!(op, BinOp::NullCoalesce) {
                    return if kiri.adalah_kosong() {
                        self.eval(right, env)
                    } else {
                        Ok(kiri)
                    };
                }
                let kanan = self.eval(right, env)?;
                self.operasi_biner(*op, kiri, kanan, expr.span)
            }
            ExprKind::BoolOp { op, values } => {
                let mut terakhir = Nilai::Kosong;
                for (indeks, expr) in values.iter().enumerate() {
                    terakhir = self.eval(expr, env)?;
                    let benar = self.kebenaran(&terakhir)?;
                    match op {
                        BoolOp::And if !benar => return Ok(terakhir),
                        BoolOp::Or if benar => return Ok(terakhir),
                        _ => {}
                    }
                    if indeks + 1 == values.len() {
                        return Ok(terakhir);
                    }
                }
                Ok(terakhir)
            }
            ExprKind::Compare {
                left,
                ops,
                comparators,
            } => {
                let mut kiri = self.eval(left, env)?;
                for (op, expr) in ops.iter().zip(comparators) {
                    let kanan = self.eval(expr, env)?;
                    if !self.bandingkan(*op, &kiri, &kanan, expr.span)? {
                        return Ok(Nilai::Logika(false));
                    }
                    kiri = kanan;
                }
                Ok(Nilai::Logika(true))
            }
            ExprKind::Between {
                value,
                low,
                high,
                negated,
            } => {
                let nilai = self.eval(value, env)?;
                let rendah = self.eval(low, env)?;
                let tinggi = self.eval(high, env)?;
                let di_dalam = self.bandingkan(CmpOp::Ge, &nilai, &rendah, expr.span)?
                    && self.bandingkan(CmpOp::Le, &nilai, &tinggi, expr.span)?;
                Ok(Nilai::Logika(if *negated { !di_dalam } else { di_dalam }))
            }
            ExprKind::IfExpr {
                body,
                condition,
                orelse,
            } => {
                let syarat = self.eval(condition, env)?;
                if self.kebenaran(&syarat)? {
                    self.eval(body, env)
                } else {
                    self.eval(orelse, env)
                }
            }
            ExprKind::Lambda { params, body } => {
                let parameter = self.siapkan_parameter(params, env)?;
                let fungsi = Rc::new(Fungsi {
                    nama: "<lambda>".to_string(),
                    posonly: parameter.0,
                    args: parameter.1,
                    vararg: parameter.2,
                    kwonly: parameter.3,
                    kwarg: parameter.4,
                    badan: BadanFungsi::Ekspresi((**body).clone()),
                    penutup: env.clone(),
                    global: self.global.clone(),
                    doc: None,
                    is_async: false,
                    is_generator: false,
                    dekorator: Vec::new(),
                    deklarasi_global: HashSet::new(),
                    deklarasi_nonlokal: HashSet::new(),
                    anotasi_kembali: None,
                    berkas: self.nama_berkas.clone(),
                });
                Ok(Nilai::Fungsi(fungsi))
            }
            ExprKind::NamedExpr { target, value } => {
                let nilai = self.eval(value, env)?;
                self.atur_target(target, nilai.clone(), env)?;
                Ok(nilai)
            }
            ExprKind::Await(inner) => {
                let nilai = self.eval(inner, env)?;
                self.tunggu_nilai(nilai, expr.span)
            }
            ExprKind::Yield { value, from } => {
                let nilai = match value {
                    Some(expr) => self.eval(expr, env)?,
                    None => Nilai::Kosong,
                };
                if *from {
                    let item = self.iter_nilai(&nilai, expr.span)?;
                    if let Some(buffer) = self.buffer_yield.last_mut() {
                        buffer.extend(item);
                    }
                } else if let Some(buffer) = self.buffer_yield.last_mut() {
                    buffer.push(nilai);
                }
                Ok(Nilai::Kosong)
            }
            ExprKind::FString(parts) => {
                let mut hasil = String::new();
                for part in parts {
                    match part {
                        FStringPart::Text(teks) => hasil.push_str(teks),
                        FStringPart::Expr {
                            value,
                            conversion,
                            format_spec,
                        } => {
                            let mut nilai = self.eval(value, env)?;
                            if let Some(konversi) = conversion {
                                nilai = match konversi {
                                    'r' => Nilai::teks(self.repr_nilai(&nilai, expr.span)?),
                                    's' => Nilai::teks(self.teks_nilai(&nilai, expr.span)?),
                                    'a' => Nilai::teks(self.repr_nilai(&nilai, expr.span)?),
                                    _ => nilai,
                                };
                            }
                            let spec = match format_spec {
                                Some(parts) => self.eval_fstring_parts(parts, env)?,
                                None => String::new(),
                            };
                            if spec.is_empty() {
                                hasil.push_str(&self.teks_nilai(&nilai, expr.span)?);
                            } else {
                                match format_nilai(&nilai, &spec) {
                                    Ok(teks) => hasil.push_str(&teks),
                                    Err(pesan) => {
                                        return Err(self.lempar_pesan(
                                            "KesalahanNilai",
                                            pesan,
                                            expr.span,
                                        ))
                                    }
                                }
                            }
                        }
                    }
                }
                Ok(Nilai::teks(hasil))
            }
            ExprKind::StringConcat(parts) => {
                let mut hasil = String::new();
                for part in parts {
                    let nilai = self.eval(part, env)?;
                    match nilai {
                        Nilai::Teks(teks) => hasil.push_str(&teks),
                        Nilai::Bytes(bytes) => {
                            hasil.push_str(&String::from_utf8_lossy(&bytes))
                        }
                        other => {
                            return Err(self.lempar_pesan(
                                "KesalahanTipe",
                                format!(
                                    "gabungan literal hanya menerima teks atau bytes, ditemukan {}",
                                    other.nama_tipe()
                                ),
                                part.span,
                            ))
                        }
                    }
                }
                Ok(Nilai::teks(hasil))
            }
            ExprKind::Starred(inner) => self.eval(inner, env),
            ExprKind::ListComp { elt, clauses } => {
                let hasil = self.eval_comprehension(elt, clauses, env, expr.span)?;
                Ok(Nilai::daftar(hasil))
            }
            ExprKind::SetComp { elt, clauses } => {
                let hasil = self.eval_comprehension(elt, clauses, env, expr.span)?;
                let mut himpunan = Vec::new();
                for item in hasil {
                    self.tambah_himpunan(&mut himpunan, item);
                }
                Ok(Nilai::himpunan(himpunan))
            }
            ExprKind::DictComp {
                key,
                value,
                clauses,
            } => {
                let hasil = self.eval_comprehension_pair(key, value, clauses, env, expr.span)?;
                Ok(Nilai::kamus(hasil))
            }
            ExprKind::GeneratorExp { elt, clauses } => {
                let hasil = self.eval_comprehension(elt, clauses, env, expr.span)?;
                Ok(Nilai::Generator(Rc::new(std::cell::RefCell::new(
                    Generator {
                        nama: "<genexpr>".to_string(),
                        nilai: hasil,
                        indeks: 0,
                    },
                ))))
            }
        }
    }

    fn eval_fstring_parts(&mut self, parts: &[FStringPart], env: &Env) -> Hasil<String> {
        let mut hasil = String::new();
        for part in parts {
            match part {
                FStringPart::Text(teks) => hasil.push_str(teks),
                FStringPart::Expr { value, .. } => {
                    let nilai = self.eval(value, env)?;
                    hasil.push_str(&self.teks_nilai(&nilai, value.span)?);
                }
            }
        }
        Ok(hasil)
    }

    fn eval_argumen(
        &mut self,
        args: &[Arg],
        env: &Env,
    ) -> Hasil<(Vec<Nilai>, Vec<(String, Nilai)>)> {
        let mut posisional = Vec::new();
        let mut kata_kunci = Vec::new();
        for arg in args {
            match arg {
                Arg::Positional(expr) => posisional.push(self.eval(expr, env)?),
                Arg::Keyword { name, value } => {
                    kata_kunci.push((name.clone(), self.eval(value, env)?))
                }
                Arg::Star(expr) => {
                    let nilai = self.eval(expr, env)?;
                    posisional.extend(self.iter_nilai(&nilai, expr.span)?);
                }
                Arg::KwStar(expr) => {
                    let nilai = self.eval(expr, env)?;
                    for (kunci, nilai) in self.kamus_pasangan(&nilai, expr.span)? {
                        match kunci {
                            Nilai::Teks(teks) => kata_kunci.push((teks.to_string(), nilai)),
                            other => {
                                return Err(self.lempar_pesan(
                                    "KesalahanTipe",
                                    format!(
                                        "kunci `**` harus teks, ditemukan {}",
                                        other.nama_tipe()
                                    ),
                                    expr.span,
                                ))
                            }
                        }
                    }
                }
            }
        }
        Ok((posisional, kata_kunci))
    }

    fn eval_comprehension(
        &mut self,
        elt: &Expr,
        clauses: &[CompClause],
        env: &Env,
        span: Span,
    ) -> Hasil<Vec<Nilai>> {
        let anak = Lingkungan::anak(env.clone());
        let mut hasil = Vec::new();
        self.eval_comp_clause(elt, clauses, 0, &anak, &mut hasil, span)?;
        Ok(hasil)
    }

    fn eval_comprehension_pair(
        &mut self,
        key: &Expr,
        value: &Expr,
        clauses: &[CompClause],
        env: &Env,
        span: Span,
    ) -> Hasil<Vec<(Nilai, Nilai)>> {
        let anak = Lingkungan::anak(env.clone());
        let mut hasil = Vec::new();
        self.eval_comp_pair_clause(key, value, clauses, 0, &anak, &mut hasil, span)?;
        Ok(hasil)
    }

    fn eval_comp_clause(
        &mut self,
        elt: &Expr,
        clauses: &[CompClause],
        indeks: usize,
        env: &Env,
        hasil: &mut Vec<Nilai>,
        span: Span,
    ) -> Hasil<()> {
        if indeks >= clauses.len() {
            hasil.push(self.eval(elt, env)?);
            return Ok(());
        }
        match &clauses[indeks] {
            CompClause::If(syarat) => {
                let nilai_syarat = self.eval(syarat, env)?;
                if self.kebenaran(&nilai_syarat)? {
                    self.eval_comp_clause(elt, clauses, indeks + 1, env, hasil, span)?;
                }
            }
            CompClause::For { target, iter, .. } => {
                let nilai = self.eval(iter, env)?;
                for item in self.iter_nilai(&nilai, span)? {
                    self.atur_target(target, item, env)?;
                    self.eval_comp_clause(elt, clauses, indeks + 1, env, hasil, span)?;
                }
            }
        }
        Ok(())
    }

    fn eval_comp_pair_clause(
        &mut self,
        key: &Expr,
        value: &Expr,
        clauses: &[CompClause],
        indeks: usize,
        env: &Env,
        hasil: &mut Vec<(Nilai, Nilai)>,
        span: Span,
    ) -> Hasil<()> {
        if indeks >= clauses.len() {
            let kunci = self.eval(key, env)?;
            let nilai = self.eval(value, env)?;
            self.atur_kamus(hasil, kunci, nilai);
            return Ok(());
        }
        match &clauses[indeks] {
            CompClause::If(syarat) => {
                let nilai_syarat = self.eval(syarat, env)?;
                if self.kebenaran(&nilai_syarat)? {
                    self.eval_comp_pair_clause(
                        key,
                        value,
                        clauses,
                        indeks + 1,
                        env,
                        hasil,
                        span,
                    )?;
                }
            }
            CompClause::For { target, iter, .. } => {
                let nilai = self.eval(iter, env)?;
                for item in self.iter_nilai(&nilai, span)? {
                    self.atur_target(target, item, env)?;
                    self.eval_comp_pair_clause(
                        key,
                        value,
                        clauses,
                        indeks + 1,
                        env,
                        hasil,
                        span,
                    )?;
                }
            }
        }
        Ok(())
    }

    // ------------------------------------------------------------------
    // Fungsi dan kelas
    // ------------------------------------------------------------------

    fn siapkan_parameter(
        &mut self,
        params: &Parameters,
        env: &Env,
    ) -> Hasil<(
        Vec<Parameter>,
        Vec<Parameter>,
        Option<String>,
        Vec<Parameter>,
        Option<String>,
    )> {
        let ubah = |daftar: &[Param], ini: &mut Self, env: &Env| -> Hasil<Vec<Parameter>> {
            let mut hasil = Vec::new();
            for param in daftar {
                let bawaan = match &param.default {
                    Some(expr) => Some(ini.eval(expr, env)?),
                    None => None,
                };
                // Anotasi bersifat dokumentasi pada runtime. Kesalahan
                // evaluasi anotasi tidak boleh menggagalkan definisi fungsi.
                let anotasi = match &param.annotation {
                    Some(expr) => ini.eval(expr, env).ok(),
                    None => None,
                };
                hasil.push(Parameter {
                    nama: param.name.clone(),
                    jenis: param.kind,
                    bawaan,
                    anotasi,
                });
            }
            Ok(hasil)
        };
        let posonly = ubah(&params.posonly, self, env)?;
        let args = ubah(&params.args, self, env)?;
        let vararg = params.vararg.as_ref().map(|param| param.name.clone());
        let kwonly = ubah(&params.kwonly, self, env)?;
        let kwarg = params.kwarg.as_ref().map(|param| param.name.clone());
        Ok((posonly, args, vararg, kwonly, kwarg))
    }

    fn buat_fungsi(&mut self, def: &FunctionDef, env: &Env) -> Hasil<Nilai> {
        let (posonly, args, vararg, kwonly, kwarg) = self.siapkan_parameter(&def.params, env)?;
        let (deklarasi_global, deklarasi_nonlokal) = pindai_deklarasi(&def.body);
        let is_generator = mengandung_hasil(&def.body);
        let doc = ambil_dokumen(&def.body);
        let anotasi_kembali = match &def.returns {
            Some(expr) => self.eval(expr, env).ok(),
            None => None,
        };
        let fungsi = Rc::new(Fungsi {
            nama: def.name.clone(),
            posonly,
            args,
            vararg,
            kwonly,
            kwarg,
            badan: BadanFungsi::Blok(def.body.clone()),
            penutup: env.clone(),
            global: self.global_aktif.clone(),
            doc,
            is_async: def.is_async,
            is_generator,
            dekorator: Vec::new(),
            deklarasi_global,
            deklarasi_nonlokal,
            anotasi_kembali,
            berkas: self.nama_berkas.clone(),
        });
        let mut nilai = Nilai::Fungsi(fungsi);
        for dekorator in def.decorators.iter().rev() {
            let fungsi_dekorator = self.eval(dekorator, env)?;
            nilai = self.panggil(&fungsi_dekorator, vec![nilai], Vec::new(), dekorator.span)?;
        }
        Ok(nilai)
    }

    fn buat_kelas(&mut self, def: &ClassDef, env: &Env) -> Hasil<Nilai> {
        let mut basis = Vec::new();
        for expr in &def.bases {
            let nilai = self.eval(expr, env)?;
            match nilai {
                Nilai::Kelas(kelas) => basis.push(kelas),
                other => {
                    return Err(self.lempar_pesan(
                        "KesalahanTipe",
                        format!("basis kelas harus kelas, ditemukan {}", other.nama_tipe()),
                        expr.span,
                    ))
                }
            }
        }
        let kelas = Kelas::baru(def.name.clone(), basis);
        kelas.atribut.borrow_mut().insert(
            "__nama__".to_string(),
            Nilai::teks(def.name.clone()),
        );
        kelas.atribut.borrow_mut().insert(
            "__modul__".to_string(),
            Nilai::teks(self.nama_modul.clone()),
        );
        if let Some(dok) = ambil_dokumen(&def.body) {
            kelas
                .atribut
                .borrow_mut()
                .insert("__dok__".to_string(), Nilai::teks(dok));
        }
        let badan_env = Lingkungan::anak(env.clone());
        let mut anotasi = Vec::new();
        let mut urutan_anotasi = Vec::new();
        for stmt in &def.body {
            if let StmtKind::AnnAssign {
                target: Expr { kind: ExprKind::Name(nama), .. },
                annotation,
                ..
            } = &stmt.kind
            {
                let nilai = self.eval(annotation, &badan_env).ok();
                anotasi.push((
                    Nilai::teks(nama.clone()),
                    nilai.unwrap_or(Nilai::Kosong),
                ));
                urutan_anotasi.push(Nilai::teks(nama.clone()));
            }
        }
        let hasil = self.exec_block(&def.body, &badan_env);
        if let Err(lemparan) = hasil {
            return Err(lemparan);
        }
        {
            let variabel = badan_env.borrow().variabel.clone();
            let mut atribut = kelas.atribut.borrow_mut();
            for (nama, nilai) in variabel {
                atribut.insert(nama, nilai);
            }
            if !anotasi.is_empty() {
                atribut.insert("__anotasi__".to_string(), Nilai::kamus(anotasi));
                atribut.insert(
                    "__urutan_anotasi__".to_string(),
                    Nilai::daftar(urutan_anotasi),
                );
            }
            // Pola kelas posisional mengikuti urutan parameter `__init__`.
            // Ini membuat `kasus Titik(x, y)` bekerja tanpa mengharuskan
            // penulis pemula menulis `__match_args__` secara manual.
            if !atribut.contains_key("__match_args__") {
                if let Some(Nilai::Fungsi(init)) = atribut.get("__init__") {
                    let nama: Vec<Nilai> = init
                        .posonly
                        .iter()
                        .chain(init.args.iter())
                        .skip(1)
                        .map(|parameter| Nilai::teks(parameter.nama.clone()))
                        .collect();
                    if !nama.is_empty() {
                        atribut.insert("__match_args__".to_string(), Nilai::tupel(nama));
                    }
                }
            }
        }
        let mut nilai = Nilai::Kelas(kelas);
        for dekorator in def.decorators.iter().rev() {
            let fungsi_dekorator = self.eval(dekorator, env)?;
            nilai = self.panggil(&fungsi_dekorator, vec![nilai], Vec::new(), dekorator.span)?;
        }
        Ok(nilai)
    }

    pub fn panggil(
        &mut self,
        fungsi: &Nilai,
        args: Vec<Nilai>,
        kwargs: Vec<(String, Nilai)>,
        span: Span,
    ) -> Hasil<Nilai> {
        let span_lama = self.span_aktif;
        self.span_aktif = span;
        let global_lama = self.global_aktif.clone();
        if let Nilai::Fungsi(fungsi) = fungsi {
            self.global_aktif = fungsi.global.clone();
        }
        #[cfg(feature = "python")]
        let _aktif = crate::python::Aktif::baru(self);
        let hasil = self.panggil_dalam(fungsi, args, kwargs, span);
        self.global_aktif = global_lama;
        self.span_aktif = span_lama;
        hasil
    }

    fn panggil_dalam(
        &mut self,
        fungsi: &Nilai,
        args: Vec<Nilai>,
        kwargs: Vec<(String, Nilai)>,
        span: Span,
    ) -> Hasil<Nilai> {
        match fungsi {
            Nilai::Bawaan(bawaan) => (bawaan.fungsi)(self, &self.global_aktif.clone(), args, kwargs),
            Nilai::BawaanKhusus(bawaan) => (bawaan.fungsi)(self, args, kwargs),
            Nilai::Fungsi(fungsi) => self.panggil_fungsi(fungsi, None, None, args, kwargs, span),
            Nilai::Metode(metode) => {
                let kelas = match &metode.penerima {
                    Nilai::Instance(instance) => Some(kelas_penentu(instance, &metode.fungsi)),
                    Nilai::Kesalahan(kesalahan) => Some(kesalahan.tipe.clone()),
                    _ => None,
                };
                self.panggil_fungsi(
                    &metode.fungsi,
                    Some(metode.penerima.clone()),
                    kelas,
                    args,
                    kwargs,
                    span,
                )
            }
            Nilai::MetodeBawaan(metode) => {
                self.panggil_metode(&metode.penerima, &metode.nama, args, kwargs, span)
            }
            Nilai::MetodeStatis(fungsi) => {
                self.panggil_fungsi(fungsi, None, None, args, kwargs, span)
            }
            Nilai::MetodeKelas(metode) => self.panggil_fungsi(
                &metode.fungsi,
                Some(Nilai::Kelas(metode.kelas.clone())),
                Some(metode.kelas.clone()),
                args,
                kwargs,
                span,
            ),
            Nilai::Kelas(kelas) => self.panggil_kelas(kelas, args, kwargs, span),
            Nilai::Instance(instance) => {
                let fungsi = self.ambil_atribut(&Nilai::Instance(instance.clone()), "__call__", span)?;
                self.panggil(&fungsi, args, kwargs, span)
            }
            Nilai::Python(_) => crate::python_panggil(fungsi, args, kwargs, span),
            Nilai::Properti(properti) => {
                self.panggil(&properti.getter, args, kwargs, span)
            }
            other => Err(self.lempar_pesan(
                "KesalahanTipe",
                format!("nilai {} tidak dapat dipanggil", other.nama_tipe()),
                span,
            )),
        }
    }

    pub fn panggil_fungsi(
        &mut self,
        fungsi: &Rc<Fungsi>,
        penerima: Option<Nilai>,
        kelas: Option<Rc<Kelas>>,
        args: Vec<Nilai>,
        kwargs: Vec<(String, Nilai)>,
        span: Span,
    ) -> Hasil<Nilai> {
        if self.kedalaman >= self.batas_kedalaman {
            return Err(self.lempar_pesan(
                "KesalahanRekursi",
                "batas rekursi terlampaui",
                span,
            ));
        }
        self.kedalaman += 1;
        let hasil = self.panggil_fungsi_dalam(fungsi, penerima, kelas, args, kwargs, span);
        self.kedalaman -= 1;
        if let Err(lemparan) = &hasil {
            self.catat_jejak(&lemparan.kesalahan, span);
        }
        hasil
    }

    fn panggil_fungsi_dalam(
        &mut self,
        fungsi: &Rc<Fungsi>,
        penerima: Option<Nilai>,
        kelas: Option<Rc<Kelas>>,
        args: Vec<Nilai>,
        kwargs: Vec<(String, Nilai)>,
        span: Span,
    ) -> Hasil<Nilai> {
        let env = Lingkungan::anak_fungsi(fungsi.penutup.clone());
        {
            let mut e = env.borrow_mut();
            e.global = fungsi.deklarasi_global.clone();
            e.nonlokal = fungsi.deklarasi_nonlokal.clone();
        }
        let mut posisi = 0usize;
        let mut kwargs = kwargs;
        let mut lewati_penerima = penerima.is_some();
        if let Some(nilai) = penerima.clone() {
            let pertama = fungsi
                .posonly
                .first()
                .or_else(|| fungsi.args.first());
            match pertama {
                Some(parameter) => {
                    env.borrow_mut().atur_lokal(parameter.nama.clone(), nilai);
                }
                None => {
                    return Err(self.lempar_pesan(
                        "KesalahanTipe",
                        "metode membutuhkan parameter penerima",
                        span,
                    ))
                }
            }
        }
        for parameter in &fungsi.posonly {
            if lewati_penerima {
                lewati_penerima = false;
                continue;
            }
            let nilai = if posisi < args.len() {
                let nilai = args[posisi].clone();
                posisi += 1;
                nilai
            } else if let Some(bawaan) = &parameter.bawaan {
                bawaan.clone()
            } else {
                return Err(self.lempar_pesan(
                    "KesalahanTipe",
                    format!(
                        "parameter `{}` pada `{}` belum diberi nilai",
                        parameter.nama, fungsi.nama
                    ),
                    span,
                ));
            };
            env.borrow_mut().atur_lokal(parameter.nama.clone(), nilai);
        }
        for parameter in &fungsi.args {
            if lewati_penerima {
                lewati_penerima = false;
                continue;
            }
            let nilai = if posisi < args.len() {
                let nilai = args[posisi].clone();
                posisi += 1;
                nilai
            } else if let Some(posisi_kw) =
                kwargs.iter().position(|(nama, _)| nama == &parameter.nama)
            {
                kwargs.remove(posisi_kw).1
            } else if let Some(bawaan) = &parameter.bawaan {
                bawaan.clone()
            } else {
                return Err(self.lempar_pesan(
                    "KesalahanTipe",
                    format!(
                        "parameter `{}` pada `{}` belum diberi nilai",
                        parameter.nama, fungsi.nama
                    ),
                    span,
                ));
            };
            env.borrow_mut().atur_lokal(parameter.nama.clone(), nilai);
        }
        if let Some(vararg) = &fungsi.vararg {
            let sisa = args[posisi..].to_vec();
            env.borrow_mut()
                .atur_lokal(vararg.clone(), Nilai::tupel(sisa));
        } else if posisi < args.len() {
            return Err(self.lempar_pesan(
                "KesalahanTipe",
                format!(
                    "`{}` menerima terlalu banyak argumen posisional",
                    fungsi.nama
                ),
                span,
            ));
        }
        for parameter in &fungsi.kwonly {
            let nilai = if let Some(posisi_kw) =
                kwargs.iter().position(|(nama, _)| nama == &parameter.nama)
            {
                kwargs.remove(posisi_kw).1
            } else if let Some(bawaan) = &parameter.bawaan {
                bawaan.clone()
            } else {
                return Err(self.lempar_pesan(
                    "KesalahanTipe",
                    format!(
                        "parameter kata kunci `{}` pada `{}` belum diberi nilai",
                        parameter.nama, fungsi.nama
                    ),
                    span,
                ));
            };
            env.borrow_mut().atur_lokal(parameter.nama.clone(), nilai);
        }
        if let Some(kwarg) = &fungsi.kwarg {
            let sisa = kwargs
                .iter()
                .map(|(nama, nilai)| (Nilai::teks(nama.clone()), nilai.clone()))
                .collect();
            env.borrow_mut().atur_lokal(kwarg.clone(), Nilai::kamus(sisa));
        } else if let Some((nama, _)) = kwargs.first() {
            return Err(self.lempar_pesan(
                "KesalahanTipe",
                format!(
                    "`{}` menerima argumen kata kunci tak dikenal `{}`",
                    fungsi.nama, nama
                ),
                span,
            ));
        }
        let (baris, kolom, _) = self.posisi(span);
        self.tumpukan.push(Bingkai {
            nama: fungsi.nama.clone(),
            berkas: fungsi.berkas.clone(),
            baris,
            kolom,
            diri: penerima.clone(),
            kelas: kelas.clone(),
        });
        let hasil = if fungsi.is_generator {
            self.buffer_yield.push(Vec::new());
            let hasil_badan = match &fungsi.badan {
                BadanFungsi::Blok(body) => self.exec_block(body, &env),
                BadanFungsi::Ekspresi(expr) => self
                    .eval(expr, &env)
                    .map(|nilai| Alur::Kembalikan(nilai)),
            };
            let nilai = self.buffer_yield.pop().unwrap_or_default();
            match hasil_badan {
                Ok(Alur::Kembalikan(_)) | Ok(_) => Ok(Nilai::Generator(Rc::new(
                    std::cell::RefCell::new(Generator {
                        nama: fungsi.nama.clone(),
                        nilai,
                        indeks: 0,
                    }),
                ))),
                Err(lemparan) => Err(lemparan),
            }
        } else {
            match &fungsi.badan {
                BadanFungsi::Blok(body) => match self.exec_block(body, &env) {
                    Ok(Alur::Kembalikan(nilai)) => Ok(nilai),
                    Ok(Alur::Hentikan) | Ok(Alur::Lanjutkan) => Err(self.lempar_pesan(
                        "KesalahanSintaks",
                        "`hentikan` atau `lanjutkan` di luar perulangan",
                        span,
                    )),
                    Ok(Alur::Normal) => Ok(Nilai::Kosong),
                    Err(lemparan) => Err(lemparan),
                },
                BadanFungsi::Ekspresi(expr) => self.eval(expr, &env),
            }
        };
        self.tumpukan.pop();
        hasil
    }

    fn panggil_kelas(
        &mut self,
        kelas: &Rc<Kelas>,
        args: Vec<Nilai>,
        kwargs: Vec<(String, Nilai)>,
        span: Span,
    ) -> Hasil<Nilai> {
        if kelas.jenis != JenisKelas::Pengguna {
            return self.bangun_tipe_bawaan(kelas, args, kwargs, span);
        }
        let instance = Rc::new(Instance {
            kelas: kelas.clone(),
            atribut: std::cell::RefCell::new(HashMap::new()),
        });
        let nilai = Nilai::Instance(instance.clone());
        if let Some(init) = kelas.cari_atribut("__init__") {
            match init {
                Nilai::Fungsi(fungsi) => {
                    self.panggil_fungsi(
                        &fungsi,
                        Some(nilai.clone()),
                        Some(kelas.clone()),
                        args,
                        kwargs,
                        span,
                    )?;
                }
                other => {
                    self.panggil(&other, args, kwargs, span)?;
                }
            }
        } else if !args.is_empty() || !kwargs.is_empty() {
            return Err(self.lempar_pesan(
                "KesalahanTipe",
                format!("`{}` tidak menerima argumen", kelas.nama),
                span,
            ));
        }
        if let Some(pasca) = kelas.cari_atribut("__pasca_init__") {
            self.panggil(&pasca, vec![nilai.clone()], Vec::new(), span)?;
        }
        Ok(nilai)
    }

    fn bangun_tipe_bawaan(
        &mut self,
        kelas: &Rc<Kelas>,
        args: Vec<Nilai>,
        kwargs: Vec<(String, Nilai)>,
        span: Span,
    ) -> Hasil<Nilai> {
        match kelas.jenis {
            JenisKelas::Kosong => Ok(Nilai::Kosong),
            JenisKelas::Logika => {
                let nilai = args.first().cloned().unwrap_or(Nilai::Logika(false));
                Ok(Nilai::Logika(self.kebenaran(&nilai)?))
            }
            JenisKelas::Bilangan => {
                let nilai = args.first().cloned().unwrap_or(Nilai::Bilangan(0));
                self.ke_bilangan(&nilai, span)
            }
            JenisKelas::Pecahan => {
                let nilai = args.first().cloned().unwrap_or(Nilai::Pecahan(0.0));
                self.ke_pecahan(&nilai, span)
            }
            JenisKelas::Kompleks => {
                let nilai = args.first().cloned().unwrap_or(Nilai::Kompleks(0.0, 0.0));
                self.ke_kompleks(&nilai, span)
            }
            JenisKelas::Teks => {
                let nilai = args.first().cloned().unwrap_or(Nilai::teks(""));
                self.ke_teks(&nilai, span)
            }
            JenisKelas::Bytes => {
                let nilai = args.first().cloned().unwrap_or(Nilai::Bytes(Rc::new(Vec::new())));
                self.ke_bytes(&nilai, span)
            }
            JenisKelas::Daftar => {
                if let Some(nilai) = args.first() {
                    Ok(Nilai::daftar(self.iter_nilai(nilai, span)?))
                } else {
                    Ok(Nilai::daftar(Vec::new()))
                }
            }
            JenisKelas::Tupel => {
                if let Some(nilai) = args.first() {
                    Ok(Nilai::tupel(self.iter_nilai(nilai, span)?))
                } else {
                    Ok(Nilai::tupel(Vec::new()))
                }
            }
            JenisKelas::Himpunan => {
                let mut hasil = Vec::new();
                if let Some(nilai) = args.first() {
                    for item in self.iter_nilai(nilai, span)? {
                        self.tambah_himpunan(&mut hasil, item);
                    }
                }
                Ok(Nilai::himpunan(hasil))
            }
            JenisKelas::Kamus => {
                let mut hasil = Vec::new();
                if let Some(nilai) = args.first() {
                    for (kunci, nilai) in self.kamus_pasangan(nilai, span)? {
                        self.atur_kamus(&mut hasil, kunci, nilai);
                    }
                }
                for (nama, nilai) in kwargs {
                    self.atur_kamus(&mut hasil, Nilai::teks(nama), nilai);
                }
                Ok(Nilai::kamus(hasil))
            }
            JenisKelas::Rentang => {
                let (mulai, akhir) = if args.len() == 1 {
                    let akhir = self.nilai_ke_i64(&args[0], span)?;
                    (0, akhir)
                } else {
                    let mulai = args
                        .first()
                        .map(|nilai| self.nilai_ke_i64(nilai, span))
                        .transpose()?
                        .unwrap_or(0);
                    let akhir = args
                        .get(1)
                        .map(|nilai| self.nilai_ke_i64(nilai, span))
                        .transpose()?
                        .unwrap_or(mulai);
                    (mulai, akhir)
                };
                let langkah = args
                    .get(2)
                    .map(|nilai| self.nilai_ke_i64(nilai, span))
                    .transpose()?
                    .unwrap_or(1);
                match Rentang::baru(mulai, akhir, langkah) {
                    Some(rentang) => Ok(Nilai::Rentang(rentang)),
                    None => Err(self.lempar_pesan(
                        "KesalahanNilai",
                        "langkah `rentang` tidak boleh nol",
                        span,
                    )),
                }
            }
            JenisKelas::Berkas => Err(self.lempar_pesan(
                "KesalahanTipe",
                "gunakan `buka()` untuk membuat berkas",
                span,
            )),
            JenisKelas::Kesalahan => {
                let pesan = match args.len() {
                    0 => String::new(),
                    1 => self.teks_nilai(&args[0], span)?,
                    _ => {
                        let mut bagian = Vec::with_capacity(args.len());
                        for nilai in &args {
                            bagian.push(self.teks_nilai(nilai, span)?);
                        }
                        bagian.join(", ")
                    }
                };
                Ok(Nilai::Kesalahan(Kesalahan::baru(
                    kelas.clone(),
                    pesan,
                    args,
                )))
            }
            _ => Err(self.lempar_pesan(
                "KesalahanTipe",
                format!("tipe `{}` tidak dapat dibangun", kelas.nama),
                span,
            )),
        }
    }

    fn tunggu_nilai(&mut self, nilai: Nilai, span: Span) -> Hasil<Nilai> {
        match nilai {
            Nilai::Python(_) => crate::python_tunggu(&nilai, span),
            other => Ok(other),
        }
    }

    // ------------------------------------------------------------------
    // Utilitas kecil
    // ------------------------------------------------------------------

    fn parse_int(&mut self, teks: &str, span: Span) -> Hasil<Nilai> {
        let bersih = teks.replace('_', "");
        let (radix, isi) = if let Some(sisa) = bersih.strip_prefix("0x").or_else(|| bersih.strip_prefix("0X")) {
            (16, sisa)
        } else if let Some(sisa) = bersih.strip_prefix("0b").or_else(|| bersih.strip_prefix("0B")) {
            (2, sisa)
        } else if let Some(sisa) = bersih.strip_prefix("0o").or_else(|| bersih.strip_prefix("0O")) {
            (8, sisa)
        } else {
            (10, bersih.as_str())
        };
        match i64::from_str_radix(isi, radix) {
            Ok(nilai) => Ok(Nilai::Bilangan(nilai)),
            Err(_) => Err(self.lempar_pesan(
                "KesalahanNilai",
                format!("literal bilangan tidak valid: `{teks}`"),
                span,
            )),
        }
    }

    fn parse_float(&mut self, teks: &str, span: Span) -> Hasil<Nilai> {
        match teks.replace('_', "").parse::<f64>() {
            Ok(nilai) => Ok(Nilai::Pecahan(nilai)),
            Err(_) => Err(self.lempar_pesan(
                "KesalahanNilai",
                format!("literal pecahan tidak valid: `{teks}`"),
                span,
            )),
        }
    }

    fn parse_imaginary(&mut self, teks: &str, span: Span) -> Hasil<Nilai> {
        let bersih = teks.replace('_', "");
        let isi = bersih.trim_end_matches('j').trim_end_matches('J');
        match isi.parse::<f64>() {
            Ok(nilai) => Ok(Nilai::Kompleks(0.0, nilai)),
            Err(_) => Err(self.lempar_pesan(
                "KesalahanNilai",
                format!("literal imajiner tidak valid: `{teks}`"),
                span,
            )),
        }
    }

    pub fn nilai_ke_i64(&mut self, nilai: &Nilai, span: Span) -> Hasil<i64> {
        match nilai {
            Nilai::Logika(nilai) => Ok(i64::from(*nilai)),
            Nilai::Bilangan(nilai) => Ok(*nilai),
            Nilai::Pecahan(nilai) => Ok(*nilai as i64),
            Nilai::Teks(teks) => teks.trim().parse::<i64>().map_err(|_| {
                self.lempar_pesan(
                    "KesalahanNilai",
                    format!("`{teks}` tidak dapat diubah menjadi bilangan"),
                    span,
                )
            }),
            _ => Err(self.lempar_pesan(
                "KesalahanTipe",
                format!("nilai {} bukan bilangan", nilai.nama_tipe()),
                span,
            )),
        }
    }

    pub fn ke_bilangan(&mut self, nilai: &Nilai, span: Span) -> Hasil<Nilai> {
        match nilai {
            Nilai::Bilangan(_) => Ok(nilai.clone()),
            Nilai::Logika(nilai) => Ok(Nilai::Bilangan(i64::from(*nilai))),
            Nilai::Pecahan(nilai) => Ok(Nilai::Bilangan(*nilai as i64)),
            Nilai::Teks(teks) => {
                let bersih = teks.trim().replace('_', "");
                if let Some(sisa) = bersih.strip_prefix("0x").or_else(|| bersih.strip_prefix("0X")) {
                    i64::from_str_radix(sisa, 16)
                        .map(Nilai::Bilangan)
                        .map_err(|_| self.lempar_pesan("KesalahanNilai", format!("`{teks}` bukan bilangan valid"), span))
                } else if let Some(sisa) = bersih.strip_prefix("0b").or_else(|| bersih.strip_prefix("0B")) {
                    i64::from_str_radix(sisa, 2)
                        .map(Nilai::Bilangan)
                        .map_err(|_| self.lempar_pesan("KesalahanNilai", format!("`{teks}` bukan bilangan valid"), span))
                } else if let Some(sisa) = bersih.strip_prefix("0o").or_else(|| bersih.strip_prefix("0O")) {
                    i64::from_str_radix(sisa, 8)
                        .map(Nilai::Bilangan)
                        .map_err(|_| self.lempar_pesan("KesalahanNilai", format!("`{teks}` bukan bilangan valid"), span))
                } else {
                    bersih
                        .parse::<i64>()
                        .map(Nilai::Bilangan)
                        .map_err(|_| self.lempar_pesan("KesalahanNilai", format!("`{teks}` bukan bilangan valid"), span))
                }
            }
            _ => Err(self.lempar_pesan(
                "KesalahanTipe",
                format!("{} tidak dapat diubah menjadi bilangan", nilai.nama_tipe()),
                span,
            )),
        }
    }

    pub fn ke_pecahan(&mut self, nilai: &Nilai, span: Span) -> Hasil<Nilai> {
        match nilai {
            Nilai::Pecahan(_) => Ok(nilai.clone()),
            Nilai::Bilangan(nilai) => Ok(Nilai::Pecahan(*nilai as f64)),
            Nilai::Logika(nilai) => Ok(Nilai::Pecahan(if *nilai { 1.0 } else { 0.0 })),
            Nilai::Teks(teks) => teks
                .trim()
                .parse::<f64>()
                .map(Nilai::Pecahan)
                .map_err(|_| self.lempar_pesan("KesalahanNilai", format!("`{teks}` bukan pecahan valid"), span)),
            _ => Err(self.lempar_pesan(
                "KesalahanTipe",
                format!("{} tidak dapat diubah menjadi pecahan", nilai.nama_tipe()),
                span,
            )),
        }
    }

    pub fn ke_kompleks(&mut self, nilai: &Nilai, span: Span) -> Hasil<Nilai> {
        match nilai {
            Nilai::Kompleks(_, _) => Ok(nilai.clone()),
            Nilai::Bilangan(nilai) => Ok(Nilai::Kompleks(*nilai as f64, 0.0)),
            Nilai::Pecahan(nilai) => Ok(Nilai::Kompleks(*nilai, 0.0)),
            Nilai::Logika(nilai) => Ok(Nilai::Kompleks(if *nilai { 1.0 } else { 0.0 }, 0.0)),
            Nilai::Teks(teks) => {
                let bersih = teks.trim().trim_end_matches('j');
                bersih
                    .parse::<f64>()
                    .map(|nilai| Nilai::Kompleks(0.0, nilai))
                    .map_err(|_| self.lempar_pesan("KesalahanNilai", format!("`{teks}` bukan kompleks valid"), span))
            }
            _ => Err(self.lempar_pesan(
                "KesalahanTipe",
                format!("{} tidak dapat diubah menjadi kompleks", nilai.nama_tipe()),
                span,
            )),
        }
    }

    pub fn ke_teks(&mut self, nilai: &Nilai, span: Span) -> Hasil<Nilai> {
        Ok(Nilai::teks(self.teks_nilai(nilai, span)?))
    }

    pub fn ke_bytes(&mut self, nilai: &Nilai, span: Span) -> Hasil<Nilai> {
        match nilai {
            Nilai::Bytes(_) => Ok(nilai.clone()),
            Nilai::Teks(teks) => Ok(Nilai::Bytes(Rc::new(teks.as_bytes().to_vec()))),
            Nilai::Daftar(item) => {
                let daftar = item.borrow().clone();
                let mut bytes = Vec::with_capacity(daftar.len());
                for item in daftar {
                    let angka = self.nilai_ke_i64(&item, span)?;
                    if !(0..=255).contains(&angka) {
                        return Err(self.lempar_pesan(
                            "KesalahanNilai",
                            "nilai bytes harus berada pada rentang 0 sampai 255",
                            span,
                        ));
                    }
                    bytes.push(angka as u8);
                }
                Ok(Nilai::Bytes(Rc::new(bytes)))
            }
            Nilai::Tupel(item) => {
                let mut bytes = Vec::with_capacity(item.len());
                for item in item.iter() {
                    let angka = self.nilai_ke_i64(item, span)?;
                    if !(0..=255).contains(&angka) {
                        return Err(self.lempar_pesan(
                            "KesalahanNilai",
                            "nilai bytes harus berada pada rentang 0 sampai 255",
                            span,
                        ));
                    }
                    bytes.push(angka as u8);
                }
                Ok(Nilai::Bytes(Rc::new(bytes)))
            }
            _ => Err(self.lempar_pesan(
                "KesalahanTipe",
                format!("{} tidak dapat diubah menjadi bytes", nilai.nama_tipe()),
                span,
            )),
        }
    }
}

/// Menemukan kelas tempat sebuah fungsi didefinisikan pada MRO instance.
pub fn kelas_penentu(instance: &Rc<Instance>, fungsi: &Rc<Fungsi>) -> Rc<Kelas> {
    for kelas in mro_lengkap(&instance.kelas) {
        for nilai in kelas.atribut.borrow().values() {
            if let Nilai::Fungsi(kandidat) = nilai {
                if Rc::ptr_eq(kandidat, fungsi) {
                    return kelas.clone();
                }
            }
        }
    }
    instance.kelas.clone()
}

fn pindai_deklarasi(body: &[Stmt]) -> (HashSet<String>, HashSet<String>) {
    let mut global = HashSet::new();
    let mut nonlokal = HashSet::new();
    for stmt in body {
        match &stmt.kind {
            StmtKind::Global(names) => global.extend(names.iter().cloned()),
            StmtKind::Nonlocal(names) => nonlokal.extend(names.iter().cloned()),
            _ => {}
        }
    }
    (global, nonlokal)
}

fn mengandung_hasil(body: &[Stmt]) -> bool {
    body.iter().any(|stmt| match &stmt.kind {
        StmtKind::Yield { .. } => true,
        StmtKind::If { body, orelse, .. } => {
            mengandung_hasil(body) || mengandung_hasil(orelse)
        }
        StmtKind::While { body, orelse, .. } => {
            mengandung_hasil(body) || mengandung_hasil(orelse)
        }
        StmtKind::For { body, orelse, .. } => {
            mengandung_hasil(body) || mengandung_hasil(orelse)
        }
        StmtKind::Try {
            body,
            handlers,
            orelse,
            finalbody,
        } => {
            mengandung_hasil(body)
                || handlers.iter().any(|h| mengandung_hasil(&h.body))
                || mengandung_hasil(orelse)
                || mengandung_hasil(finalbody)
        }
        StmtKind::With { body, .. } => mengandung_hasil(body),
        StmtKind::Match { cases, .. } => cases.iter().any(|k| mengandung_hasil(&k.body)),
        _ => false,
    })
}

fn ambil_dokumen(body: &[Stmt]) -> Option<String> {
    match body.first().map(|stmt| &stmt.kind) {
        Some(StmtKind::Expr(Expr {
            kind: ExprKind::Str(literal),
            ..
        })) => literal.decoded(),
        _ => None,
    }
}

pub fn dokumen_modul_publik(sumber: &str) -> Option<String> {
    dokumen_modul_sumber(sumber)
}

fn dokumen_modul_sumber(sumber: &str) -> Option<String> {
    let lexed = Lexer::new(sumber).tokenize();
    let parsed = parse_tokens(lexed.tokens, lexed.diagnostics);
    ambil_dokumen(&parsed.module.body)
}
