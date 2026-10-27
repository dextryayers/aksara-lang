//! Dukungan `kelas_data` untuk interpreter referensi.
//!
//! Implementasi ini membangun `__init__`, `__repr__`, dan `__eq__` secara
//! sintetis dari anotasi kelas. Tujuannya adalah semantik yang dapat diprediksi
//! pada Fase 2, bukan menggantikan dataclass CPython sepenuhnya.

use std::cell::RefCell;
use std::collections::HashSet;
use std::rc::Rc;

use aksara_ast::{
    Arg, BinOp, BoolOp, CmpOp, Expr, ExprKind, NodeId, ParamKind, Quote, Span, Stmt, StmtKind,
    StringLit,
};

use crate::format::nilai_ke_teks;
use crate::interpreter::Interpreter;
use crate::nilai::*;

struct Pembangun {
    berikut: u32,
}

impl Pembangun {
    fn baru() -> Self {
        Self { berikut: 1 }
    }

    fn id(&mut self) -> NodeId {
        let id = NodeId::new(self.berikut);
        self.berikut += 1;
        id
    }

    fn expr(&mut self, kind: ExprKind) -> Expr {
        Expr {
            id: self.id(),
            span: Span::empty(0),
            kind,
        }
    }

    fn stmt(&mut self, kind: StmtKind) -> Stmt {
        Stmt {
            id: self.id(),
            span: Span::empty(0),
            kind,
        }
    }

    fn nama(&mut self, nama: &str) -> Expr {
        self.expr(ExprKind::Name(nama.to_string()))
    }

    fn teks(&mut self, teks: &str) -> Expr {
        self.expr(ExprKind::Str(StringLit {
            prefix: String::new(),
            quote: Quote::Double,
            raw: teks.to_string(),
        }))
    }

    fn atribut(&mut self, dasar: Expr, nama: &str) -> Expr {
        self.expr(ExprKind::Attribute {
            value: Box::new(dasar),
            attr: nama.to_string(),
            safe: false,
        })
    }

    fn panggil(&mut self, fungsi: Expr, args: Vec<Expr>) -> Expr {
        self.expr(ExprKind::Call {
            func: Box::new(fungsi),
            args: args.into_iter().map(Arg::Positional).collect(),
            safe: false,
        })
    }

    fn biner(&mut self, kiri: Expr, op: BinOp, kanan: Expr) -> Expr {
        self.expr(ExprKind::Binary {
            left: Box::new(kiri),
            op,
            right: Box::new(kanan),
        })
    }
}

/// Melengkapi kelas dengan metode kelas data.
pub fn lengkapi_kelas_data(interpreter: &mut Interpreter, kelas: &Rc<Kelas>) -> Hasil<()> {
    if kelas.atribut.borrow().contains_key("__kelas_data__") {
        return Ok(());
    }
    let anotasi = match kelas.atribut.borrow().get("__anotasi__").cloned() {
        Some(Nilai::Kamus(pasangan)) => pasangan.borrow().clone(),
        _ => Vec::new(),
    };
    let urutan: Vec<String> = match kelas.atribut.borrow().get("__urutan_anotasi__").cloned() {
        Some(Nilai::Daftar(item)) => item.borrow().iter().map(nilai_ke_teks).collect(),
        _ => anotasi
            .iter()
            .map(|(kunci, _)| nilai_ke_teks(kunci))
            .collect(),
    };
    if urutan.is_empty() {
        kelas
            .atribut
            .borrow_mut()
            .insert("__kelas_data__".to_string(), Nilai::Logika(true));
        return Ok(());
    }
    let mut pembangun = Pembangun::baru();
    let env = Lingkungan::anak(interpreter.global.clone());
    env.borrow_mut()
        .atur_lokal("__kelas__", Nilai::Kelas(kelas.clone()));

    // __init__(diri, medan1, medan2=...)
    let mut parameter = Vec::new();
    parameter.push(Parameter {
        nama: "diri".to_string(),
        jenis: ParamKind::PosOrKw,
        bawaan: None,
        anotasi: None,
    });
    for nama in &urutan {
        let bawaan = kelas.atribut.borrow().get(nama).cloned();
        parameter.push(Parameter {
            nama: nama.clone(),
            jenis: ParamKind::PosOrKw,
            bawaan,
            anotasi: None,
        });
    }
    let mut badan_init = Vec::new();
    for nama in &urutan {
        let diri = pembangun.nama("diri");
        let target = pembangun.atribut(diri, nama);
        let nilai = pembangun.nama(nama);
        badan_init.push(pembangun.stmt(StmtKind::Assign {
            targets: vec![target],
            value: nilai,
        }));
    }
    let init = Fungsi {
        nama: "__init__".to_string(),
        posonly: Vec::new(),
        args: parameter,
        vararg: None,
        kwonly: Vec::new(),
        kwarg: None,
        badan: BadanFungsi::Blok(badan_init),
        penutup: env.clone(),
        global: interpreter.global.clone(),
        doc: Some("Menginisialisasi kelas data.".to_string()),
        is_async: false,
        is_generator: false,
        dekorator: Vec::new(),
        deklarasi_global: HashSet::new(),
        deklarasi_nonlokal: HashSet::new(),
        anotasi_kembali: None,
        berkas: interpreter.nama_berkas.clone(),
    };
    kelas
        .atribut
        .borrow_mut()
        .insert("__init__".to_string(), Nilai::Fungsi(Rc::new(init)));

    // __repr__(diri) -> "Nama(medan=repr(...), ...)"
    let mut repr_badan = Vec::new();
    let mut ekspresi = pembangun.teks(&format!("{}(", kelas.nama));
    for (indeks, nama) in urutan.iter().enumerate() {
        if indeks > 0 {
            let pemisah = pembangun.teks(", ");
            ekspresi = pembangun.biner(ekspresi, BinOp::Add, pemisah);
        }
        let label = pembangun.teks(&format!("{nama}="));
        ekspresi = pembangun.biner(ekspresi, BinOp::Add, label);
        let diri = pembangun.nama("diri");
        let atribut = pembangun.atribut(diri, nama);
        let repr_nama = pembangun.nama("repr");
        let repr = pembangun.panggil(repr_nama, vec![atribut]);
        ekspresi = pembangun.biner(ekspresi, BinOp::Add, repr);
    }
    let tutup = pembangun.teks(")");
    ekspresi = pembangun.biner(ekspresi, BinOp::Add, tutup);
    repr_badan.push(pembangun.stmt(StmtKind::Return(Some(ekspresi))));
    let repr = Fungsi {
        nama: "__repr__".to_string(),
        posonly: Vec::new(),
        args: vec![Parameter {
            nama: "diri".to_string(),
            jenis: ParamKind::PosOrKw,
            bawaan: None,
            anotasi: None,
        }],
        vararg: None,
        kwonly: Vec::new(),
        kwarg: None,
        badan: BadanFungsi::Blok(repr_badan),
        penutup: env.clone(),
        global: interpreter.global.clone(),
        doc: Some("Representasi kelas data.".to_string()),
        is_async: false,
        is_generator: false,
        dekorator: Vec::new(),
        deklarasi_global: HashSet::new(),
        deklarasi_nonlokal: HashSet::new(),
        anotasi_kembali: None,
        berkas: interpreter.nama_berkas.clone(),
    };
    kelas
        .atribut
        .borrow_mut()
        .insert("__repr__".to_string(), Nilai::Fungsi(Rc::new(repr)));

    // __eq__(diri, lain): bandingkan seluruh medan.
    let mut eq_badan = Vec::new();
    let isinstance = pembangun.nama("isinstance");
    let lain = pembangun.nama("lain");
    let kelas_nama = pembangun.nama("__kelas__");
    let panggil = pembangun.panggil(isinstance, vec![lain, kelas_nama]);
    let bukan_cocok = pembangun.expr(ExprKind::Unary {
        op: aksara_ast::UnaryOp::Not,
        operand: Box::new(panggil),
    });
    let salah = pembangun.expr(ExprKind::Bool(false));
    let kembali_salah = pembangun.stmt(StmtKind::Return(Some(salah)));
    let cabang = pembangun.stmt(StmtKind::If {
        condition: bukan_cocok,
        body: vec![kembali_salah],
        orelse: Vec::new(),
    });
    eq_badan.push(cabang);
    let mut perbandingan = Vec::new();
    for nama in &urutan {
        let diri = pembangun.nama("diri");
        let kiri = pembangun.atribut(diri, nama);
        let lain = pembangun.nama("lain");
        let kanan = pembangun.atribut(lain, nama);
        perbandingan.push(pembangun.expr(ExprKind::Compare {
            left: Box::new(kiri),
            ops: vec![CmpOp::Eq],
            comparators: vec![kanan],
        }));
    }
    let hasil = if perbandingan.len() == 1 {
        perbandingan.remove(0)
    } else {
        pembangun.expr(ExprKind::BoolOp {
            op: BoolOp::And,
            values: perbandingan,
        })
    };
    eq_badan.push(pembangun.stmt(StmtKind::Return(Some(hasil))));
    let eq = Fungsi {
        nama: "__eq__".to_string(),
        posonly: Vec::new(),
        args: vec![
            Parameter {
                nama: "diri".to_string(),
                jenis: ParamKind::PosOrKw,
                bawaan: None,
                anotasi: None,
            },
            Parameter {
                nama: "lain".to_string(),
                jenis: ParamKind::PosOrKw,
                bawaan: None,
                anotasi: None,
            },
        ],
        vararg: None,
        kwonly: Vec::new(),
        kwarg: None,
        badan: BadanFungsi::Blok(eq_badan),
        penutup: env,
        global: interpreter.global.clone(),
        doc: Some("Perbandingan kelas data.".to_string()),
        is_async: false,
        is_generator: false,
        dekorator: Vec::new(),
        deklarasi_global: HashSet::new(),
        deklarasi_nonlokal: HashSet::new(),
        anotasi_kembali: None,
        berkas: interpreter.nama_berkas.clone(),
    };
    kelas
        .atribut
        .borrow_mut()
        .insert("__eq__".to_string(), Nilai::Fungsi(Rc::new(eq)));
    kelas
        .atribut
        .borrow_mut()
        .insert("__kelas_data__".to_string(), Nilai::Logika(true));
    let _ = RefCell::new(());
    Ok(())
}
