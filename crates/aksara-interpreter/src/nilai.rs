use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::fmt;
use std::fs::File;
use std::rc::Rc;

use aksara_ast::Span;

pub type Env = Rc<RefCell<Lingkungan>>;
pub type Hasil<T> = Result<T, Lemparan>;

#[derive(Clone)]
pub struct Lingkungan {
    pub variabel: HashMap<String, Nilai>,
    pub induk: Option<Env>,
    /// Nama yang dideklarasikan `global` di fungsi ini.
    pub global: HashSet<String>,
    /// Nama yang dideklarasikan `nonlokal` di fungsi ini.
    pub nonlokal: HashSet<String>,
    /// Benar bila lingkungan ini adalah pemanggilan fungsi.
    pub adalah_fungsi: bool,
}

impl Lingkungan {
    pub fn baru() -> Env {
        Rc::new(RefCell::new(Self {
            variabel: HashMap::new(),
            induk: None,
            global: HashSet::new(),
            nonlokal: HashSet::new(),
            adalah_fungsi: false,
        }))
    }

    pub fn anak(induk: Env) -> Env {
        Rc::new(RefCell::new(Self {
            variabel: HashMap::new(),
            induk: Some(induk),
            global: HashSet::new(),
            nonlokal: HashSet::new(),
            adalah_fungsi: false,
        }))
    }

    pub fn anak_fungsi(induk: Env) -> Env {
        Rc::new(RefCell::new(Self {
            variabel: HashMap::new(),
            induk: Some(induk),
            global: HashSet::new(),
            nonlokal: HashSet::new(),
            adalah_fungsi: true,
        }))
    }

    pub fn cari_lokal(&self, nama: &str) -> Option<Nilai> {
        self.variabel.get(nama).cloned()
    }

    pub fn cari(&self, nama: &str) -> Option<Nilai> {
        if let Some(nilai) = self.variabel.get(nama) {
            return Some(nilai.clone());
        }
        self.induk.as_ref().and_then(|induk| induk.borrow().cari(nama))
    }

    pub fn atur_lokal(&mut self, nama: impl Into<String>, nilai: Nilai) {
        self.variabel.insert(nama.into(), nilai);
    }

    pub fn hapus_lokal(&mut self, nama: &str) -> Option<Nilai> {
        self.variabel.remove(nama)
    }

    pub fn cari_env_lokal(env: &Env, nama: &str) -> Option<Nilai> {
        env.borrow().cari_lokal(nama)
    }

    pub fn atur(env: &Env, nama: impl Into<String>, nilai: Nilai) {
        env.borrow_mut().atur_lokal(nama, nilai);
    }
}

#[derive(Clone)]
pub enum Nilai {
    Kosong,
    Logika(bool),
    Bilangan(i64),
    Pecahan(f64),
    Kompleks(f64, f64),
    Teks(Rc<str>),
    Bytes(Rc<Vec<u8>>),
    Daftar(Rc<RefCell<Vec<Nilai>>>),
    Tupel(Rc<Vec<Nilai>>),
    Himpunan(Rc<RefCell<Vec<Nilai>>>),
    Kamus(Rc<RefCell<Vec<(Nilai, Nilai)>>>),
    Rentang(Rentang),
    Slice(Rc<Slice>),
    Fungsi(Rc<Fungsi>),
    Bawaan(Rc<Bawaan>),
    BawaanKhusus(Rc<BawaanKhusus>),
    Kelas(Rc<Kelas>),
    Instance(Rc<Instance>),
    Metode(Rc<Metode>),
    MetodeBawaan(Rc<MetodeBawaan>),
    Properti(Rc<Properti>),
    MetodeStatis(Rc<Fungsi>),
    MetodeKelas(Rc<MetodeKelas>),
    Modul(Rc<Modul>),
    Generator(Rc<RefCell<Generator>>),
    Kesalahan(Rc<Kesalahan>),
    Induk(Rc<Induk>),
    Berkas(Rc<RefCell<Berkas>>),
    /// Objek Python asli yang disimpan pada registri jembatan.
    Python(usize),
    Ellipsis,
}

impl Nilai {
    pub fn teks(nilai: impl Into<String>) -> Self {
        Nilai::Teks(Rc::from(nilai.into()))
    }

    pub fn daftar(nilai: Vec<Nilai>) -> Self {
        Nilai::Daftar(Rc::new(RefCell::new(nilai)))
    }

    pub fn tupel(nilai: Vec<Nilai>) -> Self {
        Nilai::Tupel(Rc::new(nilai))
    }

    pub fn himpunan(nilai: Vec<Nilai>) -> Self {
        Nilai::Himpunan(Rc::new(RefCell::new(nilai)))
    }

    pub fn kamus(nilai: Vec<(Nilai, Nilai)>) -> Self {
        Nilai::Kamus(Rc::new(RefCell::new(nilai)))
    }

    pub fn nama_tipe(&self) -> &str {
        match self {
            Nilai::Kosong => "kosong",
            Nilai::Logika(_) => "logika",
            Nilai::Bilangan(_) => "bilangan",
            Nilai::Pecahan(_) => "pecahan",
            Nilai::Kompleks(_, _) => "kompleks",
            Nilai::Teks(_) => "teks",
            Nilai::Bytes(_) => "bytes",
            Nilai::Daftar(_) => "daftar",
            Nilai::Tupel(_) => "tupel",
            Nilai::Himpunan(_) => "himpunan",
            Nilai::Kamus(_) => "kamus",
            Nilai::Rentang(_) => "rentang",
            Nilai::Slice(_) => "slice",
            Nilai::Fungsi(_) | Nilai::Bawaan(_) | Nilai::BawaanKhusus(_) | Nilai::Metode(_)
            | Nilai::MetodeBawaan(_) => "fungsi",
            Nilai::Kelas(_) => "kelas",
            Nilai::Instance(instance) => return instance.kelas.nama.as_str(),
            Nilai::Properti(_) => "properti",
            Nilai::MetodeStatis(_) | Nilai::MetodeKelas(_) => "fungsi",
            Nilai::Modul(_) => "modul",
            Nilai::Generator(_) => "generator",
            Nilai::Kesalahan(kesalahan) => return kesalahan.tipe.nama.as_str(),
            Nilai::Induk(_) => "induk",
            Nilai::Berkas(_) => "berkas",
            Nilai::Python(_) => "objek_python",
            Nilai::Ellipsis => "ellipsis",
        }
    }

    pub fn adalah_angka(&self) -> bool {
        matches!(
            self,
            Nilai::Logika(_) | Nilai::Bilangan(_) | Nilai::Pecahan(_) | Nilai::Kompleks(_, _)
        )
    }

    pub fn sebagai_i64(&self) -> Option<i64> {
        match self {
            Nilai::Logika(nilai) => Some(i64::from(*nilai)),
            Nilai::Bilangan(nilai) => Some(*nilai),
            _ => None,
        }
    }

    pub fn sebagai_f64(&self) -> Option<f64> {
        match self {
            Nilai::Logika(nilai) => Some(if *nilai { 1.0 } else { 0.0 }),
            Nilai::Bilangan(nilai) => Some(*nilai as f64),
            Nilai::Pecahan(nilai) => Some(*nilai),
            _ => None,
        }
    }

    pub fn sebagai_kompleks(&self) -> Option<(f64, f64)> {
        match self {
            Nilai::Logika(nilai) => Some((if *nilai { 1.0 } else { 0.0 }, 0.0)),
            Nilai::Bilangan(nilai) => Some((*nilai as f64, 0.0)),
            Nilai::Pecahan(nilai) => Some((*nilai, 0.0)),
            Nilai::Kompleks(nyata, imajiner) => Some((*nyata, *imajiner)),
            _ => None,
        }
    }

    pub fn adalah_kosong(&self) -> bool {
        matches!(self, Nilai::Kosong)
    }

    pub fn iter_daftar(&self) -> Option<Vec<Nilai>> {
        match self {
            Nilai::Daftar(nilai) => Some(nilai.borrow().clone()),
            Nilai::Tupel(nilai) => Some(nilai.as_ref().clone()),
            _ => None,
        }
    }
}

impl fmt::Debug for Nilai {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Nilai::Kosong => write!(f, "Kosong"),
            Nilai::Logika(nilai) => write!(f, "Logika({nilai})"),
            Nilai::Bilangan(nilai) => write!(f, "Bilangan({nilai})"),
            Nilai::Pecahan(nilai) => write!(f, "Pecahan({nilai})"),
            Nilai::Kompleks(nyata, imajiner) => write!(f, "Kompleks({nyata}, {imajiner})"),
            Nilai::Teks(nilai) => write!(f, "Teks({nilai:?})"),
            Nilai::Bytes(nilai) => write!(f, "Bytes({:?})", nilai.as_slice()),
            Nilai::Daftar(nilai) => write!(f, "Daftar({:?})", nilai.borrow()),
            Nilai::Tupel(nilai) => write!(f, "Tupel({nilai:?})"),
            Nilai::Himpunan(nilai) => write!(f, "Himpunan({:?})", nilai.borrow()),
            Nilai::Kamus(nilai) => write!(f, "Kamus({:?})", nilai.borrow()),
            Nilai::Rentang(nilai) => write!(f, "Rentang({nilai:?})"),
            Nilai::Slice(nilai) => write!(f, "Slice({nilai:?})"),
            Nilai::Fungsi(fungsi) => write!(f, "Fungsi({})", fungsi.nama),
            Nilai::Bawaan(bawaan) => write!(f, "Bawaan({})", bawaan.nama),
            Nilai::BawaanKhusus(bawaan) => write!(f, "BawaanKhusus({})", bawaan.nama),
            Nilai::Kelas(kelas) => write!(f, "Kelas({})", kelas.nama),
            Nilai::Instance(instance) => write!(f, "Instance({})", instance.kelas.nama),
            Nilai::Metode(metode) => write!(f, "Metode({})", metode.fungsi.nama),
            Nilai::MetodeBawaan(metode) => write!(f, "MetodeBawaan({})", metode.nama),
            Nilai::Properti(_) => write!(f, "Properti"),
            Nilai::MetodeStatis(fungsi) => write!(f, "MetodeStatis({})", fungsi.nama),
            Nilai::MetodeKelas(metode) => write!(f, "MetodeKelas({})", metode.fungsi.nama),
            Nilai::Modul(modul) => write!(f, "Modul({})", modul.nama),
            Nilai::Generator(_) => write!(f, "Generator"),
            Nilai::Kesalahan(kesalahan) => write!(f, "Kesalahan({})", kesalahan.tipe.nama),
            Nilai::Induk(_) => write!(f, "Induk"),
            Nilai::Berkas(berkas) => write!(f, "Berkas({})", berkas.borrow().nama),
            Nilai::Python(id) => write!(f, "Python({id})"),
            Nilai::Ellipsis => write!(f, "Ellipsis"),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Rentang {
    pub mulai: i64,
    pub akhir: i64,
    pub langkah: i64,
}

impl Rentang {
    pub fn baru(mulai: i64, akhir: i64, langkah: i64) -> Option<Self> {
        if langkah == 0 {
            return None;
        }
        Some(Self {
            mulai,
            akhir,
            langkah,
        })
    }

    pub fn panjang(&self) -> usize {
        if self.langkah > 0 {
            if self.mulai >= self.akhir {
                0
            } else {
                ((self.akhir - self.mulai - 1) / self.langkah + 1) as usize
            }
        } else if self.mulai <= self.akhir {
            0
        } else {
            ((self.mulai - self.akhir - 1) / (-self.langkah) + 1) as usize
        }
    }

    pub fn nilai(&self) -> Vec<Nilai> {
        let mut hasil = Vec::with_capacity(self.panjang());
        let mut sekarang = self.mulai;
        if self.langkah > 0 {
            while sekarang < self.akhir {
                hasil.push(Nilai::Bilangan(sekarang));
                sekarang += self.langkah;
            }
        } else {
            while sekarang > self.akhir {
                hasil.push(Nilai::Bilangan(sekarang));
                sekarang += self.langkah;
            }
        }
        hasil
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Slice {
    pub mulai: Option<i64>,
    pub akhir: Option<i64>,
    pub langkah: Option<i64>,
}

impl Slice {
    pub fn indeks(&self, panjang: usize) -> Option<Vec<usize>> {
        let panjang_i = panjang as i64;
        let langkah = self.langkah.unwrap_or(1);
        if langkah == 0 {
            return None;
        }
        let (mulai, akhir) = if langkah > 0 {
            let mulai = self
                .mulai
                .map(|nilai| if nilai < 0 { nilai + panjang_i } else { nilai })
                .unwrap_or(0)
                .clamp(0, panjang_i);
            let akhir = self
                .akhir
                .map(|nilai| if nilai < 0 { nilai + panjang_i } else { nilai })
                .unwrap_or(panjang_i)
                .clamp(0, panjang_i);
            (mulai, akhir)
        } else {
            let mulai = self
                .mulai
                .map(|nilai| if nilai < 0 { nilai + panjang_i } else { nilai })
                .unwrap_or(panjang_i - 1)
                .clamp(-1, panjang_i - 1);
            let akhir = self
                .akhir
                .map(|nilai| if nilai < 0 { nilai + panjang_i } else { nilai })
                .unwrap_or(-1)
                .clamp(-1, panjang_i - 1);
            (mulai, akhir)
        };
        let mut hasil = Vec::new();
        let mut indeks = mulai;
        if langkah > 0 {
            while indeks < akhir {
                hasil.push(indeks as usize);
                indeks += langkah;
            }
        } else {
            while indeks > akhir {
                hasil.push(indeks as usize);
                indeks += langkah;
            }
        }
        Some(hasil)
    }
}

#[derive(Clone)]
pub struct Parameter {
    pub nama: String,
    pub jenis: aksara_ast::ParamKind,
    pub bawaan: Option<Nilai>,
    pub anotasi: Option<Nilai>,
}

#[derive(Clone)]
pub enum BadanFungsi {
    Blok(Vec<aksara_ast::Stmt>),
    Ekspresi(aksara_ast::Expr),
}

#[derive(Clone)]
pub struct Fungsi {
    pub nama: String,
    pub posonly: Vec<Parameter>,
    pub args: Vec<Parameter>,
    pub vararg: Option<String>,
    pub kwonly: Vec<Parameter>,
    pub kwarg: Option<String>,
    pub badan: BadanFungsi,
    pub penutup: Env,
    pub global: Env,
    pub doc: Option<String>,
    pub is_async: bool,
    pub is_generator: bool,
    pub dekorator: Vec<Nilai>,
    pub deklarasi_global: HashSet<String>,
    pub deklarasi_nonlokal: HashSet<String>,
    pub anotasi_kembali: Option<Nilai>,
    pub berkas: String,
}

impl Fungsi {
    pub fn parameter_semua(&self) -> impl Iterator<Item = &Parameter> {
        self.posonly
            .iter()
            .chain(self.args.iter())
            .chain(self.kwonly.iter())
    }
}

pub type BawaanFn = fn(
    &mut crate::interpreter::Interpreter,
    &Env,
    Vec<Nilai>,
    Vec<(String, Nilai)>,
) -> Hasil<Nilai>;

pub struct Bawaan {
    pub nama: String,
    pub fungsi: BawaanFn,
    pub doc: Option<String>,
}

impl fmt::Debug for Bawaan {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Bawaan({})", self.nama)
    }
}

pub type BawaanKhususFn = Box<
    dyn Fn(
        &mut crate::interpreter::Interpreter,
        Vec<Nilai>,
        Vec<(String, Nilai)>,
    ) -> Hasil<Nilai>,
>;

pub struct BawaanKhusus {
    pub nama: String,
    pub fungsi: BawaanKhususFn,
}

impl fmt::Debug for BawaanKhusus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "BawaanKhusus({})", self.nama)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum JenisKelas {
    Pengguna,
    Kosong,
    Logika,
    Bilangan,
    Pecahan,
    Kompleks,
    Teks,
    Bytes,
    Daftar,
    Tupel,
    Himpunan,
    Kamus,
    Rentang,
    Fungsi,
    Modul,
    Berkas,
    Generator,
    Kesalahan,
    Tipe,
}

pub struct Kelas {
    pub nama: String,
    pub basis: Vec<Rc<Kelas>>,
    pub mro: Vec<Rc<Kelas>>,
    pub atribut: RefCell<HashMap<String, Nilai>>,
    pub jenis: JenisKelas,
    pub doc: Option<String>,
    pub modul: String,
}

impl fmt::Debug for Kelas {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Kelas({})", self.nama)
    }
}

impl Kelas {
    pub fn baru(nama: impl Into<String>, basis: Vec<Rc<Kelas>>) -> Rc<Self> {
        Self::baru_jenis(nama, basis, JenisKelas::Pengguna)
    }

    pub fn baru_jenis(
        nama: impl Into<String>,
        basis: Vec<Rc<Kelas>>,
        jenis: JenisKelas,
    ) -> Rc<Self> {
        let nama = nama.into();
        let mro = linearisasi_mro(&nama, &basis);
        Rc::new(Self {
            nama,
            basis,
            mro,
            atribut: RefCell::new(HashMap::new()),
            jenis,
            doc: None,
            modul: "__utama__".to_string(),
        })
    }

    pub fn cari_atribut(&self, nama: &str) -> Option<Nilai> {
        if let Some(nilai) = self.atribut.borrow().get(nama) {
            return Some(nilai.clone());
        }
        for basis in &self.mro {
            if let Some(nilai) = basis.atribut.borrow().get(nama) {
                return Some(nilai.clone());
            }
        }
        None
    }

    pub fn adalah_subkelas(&self, lain: &Rc<Kelas>) -> bool {
        if std::ptr::eq(self, lain.as_ref()) {
            return true;
        }
        self.mro.iter().any(|basis| Rc::ptr_eq(basis, lain))
    }
}

/// Urutan MRO lengkap termasuk kelas itu sendiri.
pub fn mro_lengkap(kelas: &Rc<Kelas>) -> Vec<Rc<Kelas>> {
    let mut hasil = Vec::with_capacity(kelas.mro.len() + 1);
    hasil.push(kelas.clone());
    hasil.extend(kelas.mro.iter().cloned());
    hasil
}

fn linearisasi_mro(nama: &str, basis: &[Rc<Kelas>]) -> Vec<Rc<Kelas>> {
    if basis.is_empty() {
        return Vec::new();
    }
    let mut hasil = Vec::new();
    let mut urutan: Vec<Vec<Rc<Kelas>>> = basis
        .iter()
        .map(|kelas| {
            let mut daftar = vec![kelas.clone()];
            daftar.extend(kelas.mro.iter().cloned());
            daftar
        })
        .collect();
    urutan.push(basis.to_vec());
    loop {
        urutan.retain(|daftar| !daftar.is_empty());
        if urutan.is_empty() {
            break;
        }
        let mut kandidat = None;
        for daftar in &urutan {
            let kepala = &daftar[0];
            let dipakai_di_ekor = urutan
                .iter()
                .any(|lain| lain.iter().skip(1).any(|item| Rc::ptr_eq(item, kepala)));
            if !dipakai_di_ekor {
                kandidat = Some(kepala.clone());
                break;
            }
        }
        let Some(kandidat) = kandidat else {
            // MRO tidak konsisten. Pertahankan urutan basis agar runtime tetap
            // dapat melanjutkan dan laporkan melalui pemeriksaan terpisah.
            for daftar in &mut urutan {
                if let Some(item) = daftar.first().cloned() {
                    hasil.push(item);
                }
            }
            break;
        };
        hasil.push(kandidat.clone());
        for daftar in &mut urutan {
            if daftar.first().is_some_and(|item| Rc::ptr_eq(item, &kandidat)) {
                daftar.remove(0);
            }
        }
    }
    let _ = nama;
    hasil
}

pub struct Instance {
    pub kelas: Rc<Kelas>,
    pub atribut: RefCell<HashMap<String, Nilai>>,
}

impl fmt::Debug for Instance {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Instance({})", self.kelas.nama)
    }
}

pub struct Metode {
    pub fungsi: Rc<Fungsi>,
    pub penerima: Nilai,
}

pub struct MetodeKelas {
    pub fungsi: Rc<Fungsi>,
    pub kelas: Rc<Kelas>,
}

pub struct MetodeBawaan {
    pub penerima: Nilai,
    pub nama: String,
}

#[derive(Clone)]
pub struct Properti {
    pub getter: Nilai,
    pub setter: Option<Nilai>,
}

pub struct Induk {
    pub instance: Option<Rc<Instance>>,
    pub kelas: Rc<Kelas>,
    pub mulai: usize,
}

pub struct Modul {
    pub nama: String,
    pub atribut: RefCell<HashMap<String, Nilai>>,
    pub doc: Option<String>,
    pub berkas: Option<String>,
}

impl fmt::Debug for Modul {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Modul({})", self.nama)
    }
}

pub struct Generator {
    pub nama: String,
    pub nilai: Vec<Nilai>,
    pub indeks: usize,
}

impl fmt::Debug for Generator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Generator({})", self.nama)
    }
}

pub struct Berkas {
    pub berkas: Option<File>,
    pub nama: String,
    pub mode: String,
    pub biner: bool,
}

impl fmt::Debug for Berkas {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Berkas({})", self.nama)
    }
}

pub struct Kesalahan {
    pub tipe: Rc<Kelas>,
    pub pesan: String,
    pub argumen: Vec<Nilai>,
    pub penyebab: Option<Rc<Kesalahan>>,
    pub atribut: RefCell<HashMap<String, Nilai>>,
    pub jejak: RefCell<Vec<Jejak>>,
}

impl fmt::Debug for Kesalahan {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Kesalahan({}: {})", self.tipe.nama, self.pesan)
    }
}

impl Kesalahan {
    pub fn baru(tipe: Rc<Kelas>, pesan: impl Into<String>, argumen: Vec<Nilai>) -> Rc<Self> {
        Rc::new(Self {
            tipe,
            pesan: pesan.into(),
            argumen,
            penyebab: None,
            atribut: RefCell::new(HashMap::new()),
            jejak: RefCell::new(Vec::new()),
        })
    }

    pub fn dari_pesan(tipe: Rc<Kelas>, pesan: impl Into<String>) -> Rc<Self> {
        let pesan = pesan.into();
        Self::baru(tipe, pesan.clone(), vec![Nilai::teks(pesan)])
    }
}

#[derive(Clone, Debug)]
pub struct Jejak {
    pub berkas: String,
    pub baris: usize,
    pub kolom: usize,
    pub fungsi: String,
    pub teks: String,
}

#[derive(Clone, Debug)]
pub struct Lemparan {
    pub kesalahan: Rc<Kesalahan>,
    pub span: Span,
}

impl Lemparan {
    pub fn baru(kesalahan: Rc<Kesalahan>, span: Span) -> Self {
        Self { kesalahan, span }
    }
}

impl From<Rc<Kesalahan>> for Lemparan {
    fn from(kesalahan: Rc<Kesalahan>) -> Self {
        Self {
            kesalahan,
            span: Span::empty(0),
        }
    }
}
