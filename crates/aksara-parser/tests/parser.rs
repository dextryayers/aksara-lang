//! Uji perilaku parser: prioritas, bentuk pernyataan, pola, dan diagnostik.

use aksara_ast::{
    Arg, BinOp, CmpOp, CompClause, Expr, ExprKind, FStringPart, ForIter, ParamKind, Pattern,
    StmtKind, UnaryOp,
};

use aksara_parser::parse;

fn parse_ok(source: &str) -> aksara_ast::Module {
    let result = parse(source);
    assert!(
        result.diagnostics.is_empty(),
        "diagnostik tak terduga: {:#?}",
        result.diagnostics
    );
    result.module
}

fn value_of(source: &str) -> Expr {
    let module = parse_ok(source);
    match module
        .body
        .into_iter()
        .next()
        .expect("satu pernyataan")
        .kind
    {
        StmtKind::Assign { value, .. } => value,
        other => panic!("bukan penugasan: {other:?}"),
    }
}

#[test]
fn modul_kosong_tanpa_diagnostik() {
    let result = parse("");
    assert!(result.diagnostics.is_empty());
    assert!(result.module.body.is_empty());
}

#[test]
fn prioritas_aritmetika() {
    let value = value_of("x = 1 + 2 * 3\n");
    match value.kind {
        ExprKind::Binary { op, right, .. } => {
            assert_eq!(op, BinOp::Add);
            assert!(matches!(
                right.kind,
                ExprKind::Binary { op: BinOp::Mul, .. }
            ));
        }
        other => panic!("bukan penjumlahan: {other:?}"),
    }
}

#[test]
fn pangkat_bersifat_kanan() {
    let value = value_of("x = 2 ** 3 ** 2\n");
    match value.kind {
        ExprKind::Binary {
            op: BinOp::Pow,
            right,
            ..
        } => assert!(matches!(
            right.kind,
            ExprKind::Binary { op: BinOp::Pow, .. }
        )),
        other => panic!("bukan pangkat: {other:?}"),
    }
}

#[test]
fn uner_negatif_mengikat_lebih_lemah_daripada_pangkat() {
    let value = value_of("x = -2 ** 2\n");
    match value.kind {
        ExprKind::Unary {
            op: UnaryOp::Neg,
            operand,
        } => assert!(matches!(
            operand.kind,
            ExprKind::Binary { op: BinOp::Pow, .. }
        )),
        other => panic!("bukan negasi: {other:?}"),
    }
}

#[test]
fn perbandingan_berantai() {
    let value = value_of("x = 0 < a <= 10\n");
    match value.kind {
        ExprKind::Compare {
            ops, comparators, ..
        } => {
            assert_eq!(ops, vec![CmpOp::Lt, CmpOp::Le]);
            assert_eq!(comparators.len(), 2);
        }
        other => panic!("bukan perbandingan: {other:?}"),
    }
}

#[test]
fn antara_dan_bukan_antara() {
    let value = value_of("x = a antara 1 dan 10\n");
    match value.kind {
        ExprKind::Between { negated, .. } => assert!(!negated),
        other => panic!("bukan antara: {other:?}"),
    }

    let value = value_of("x = a bukan antara 1 dan 10\n");
    match value.kind {
        ExprKind::Between { negated, .. } => assert!(negated),
        other => panic!("bukan antara: {other:?}"),
    }
}

#[test]
fn bukan_adalah_membentuk_perbandingan_identitas_negatif() {
    let value = value_of("x = a bukan adalah b\n");
    match value.kind {
        ExprKind::Compare {
            ops, comparators, ..
        } => {
            assert_eq!(ops, vec![CmpOp::IsNot]);
            assert_eq!(comparators.len(), 1);
        }
        other => panic!("bukan perbandingan: {other:?}"),
    }
}

#[test]
fn urutan_adalah_bukan_memberi_bantuan() {
    let result = parse("x = a adalah bukan b\n");
    let diagnostic = result
        .diagnostics
        .iter()
        .find(|diagnostic| diagnostic.message.contains("adalah bukan"))
        .expect("ada diagnostik urutan operator");
    assert!(diagnostic
        .help
        .as_deref()
        .unwrap_or("")
        .contains("bukan adalah"));
}

#[test]
fn null_coalesce_mengikat_lebih_lemah_daripada_atau() {
    let value = value_of("x = a ?? b atau c\n");
    match value.kind {
        ExprKind::Binary {
            op: BinOp::NullCoalesce,
            right,
            ..
        } => assert!(matches!(right.kind, ExprKind::BoolOp { .. })),
        other => panic!("bukan pengganti kosong: {other:?}"),
    }
}

#[test]
fn ternary_bersifat_kanan() {
    let value = value_of("x = a jika b lainnya c jika d lainnya e\n");
    match value.kind {
        ExprKind::IfExpr { orelse, .. } => {
            assert!(matches!(orelse.kind, ExprKind::IfExpr { .. }));
        }
        other => panic!("bukan ternary: {other:?}"),
    }
}

#[test]
fn walrus_dan_target_harus_nama() {
    let value = value_of("x = (n := 10)\n");
    assert!(matches!(value.kind, ExprKind::NamedExpr { .. }));

    let result = parse("x = (a + b := 10)\n");
    assert!(result
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic.code == "E-SINTAKS"));
}

#[test]
fn akses_aman_berantai() {
    let value = value_of("x = a?.b?[0]?()\n");
    fn cek(expr: &Expr) -> bool {
        match &expr.kind {
            ExprKind::Call { safe, .. } => *safe,
            ExprKind::Subscript { safe, value, .. } => *safe && cek(value),
            ExprKind::Attribute { safe, value, .. } => *safe && cek(value),
            _ => true,
        }
    }
    assert!(cek(&value));
}

#[test]
fn slice_dan_subskrip_ganda() {
    let value = value_of("x = data[1:5:2]\n");
    assert!(matches!(value.kind, ExprKind::Subscript { .. }));

    let value = value_of("x = data[1:2, 3]\n");
    match value.kind {
        ExprKind::Subscript { index, .. } => {
            assert!(matches!(index.kind, ExprKind::Tuple(_)));
        }
        other => panic!("bukan subskrip: {other:?}"),
    }
}

#[test]
fn comprehension_dan_generator() {
    let value = value_of("x = [n * n untuk n dalam data jika n > 0]\n");
    match value.kind {
        ExprKind::ListComp { clauses, .. } => {
            assert_eq!(clauses.len(), 2);
            assert!(matches!(clauses[0], CompClause::For { .. }));
            assert!(matches!(clauses[1], CompClause::If(_)));
        }
        other => panic!("bukan comprehension: {other:?}"),
    }

    let value = value_of("x = (n untuk n dalam data)\n");
    assert!(matches!(value.kind, ExprKind::GeneratorExp { .. }));
}

#[test]
fn fstring_memecah_teks_ekspresi_dan_format() {
    let value = value_of("x = f\"Halo {nama!r:>{lebar}}\"\n");
    match value.kind {
        ExprKind::FString(parts) => {
            assert!(matches!(&parts[0], FStringPart::Text(text) if text == "Halo "));
            match &parts[1] {
                FStringPart::Expr {
                    conversion,
                    format_spec,
                    ..
                } => {
                    assert_eq!(*conversion, Some('r'));
                    let spec = format_spec.as_ref().expect("ada format spec");
                    assert!(matches!(&spec[0], FStringPart::Text(text) if text == ">"));
                    assert!(matches!(&spec[1], FStringPart::Expr { .. }));
                }
                other => panic!("bagian f-string salah: {other:?}"),
            }
        }
        other => panic!("bukan f-string: {other:?}"),
    }
}

#[test]
fn fstring_bersarang_dan_kurung_literal() {
    parse_ok("x = f\"{f'{nama}'} dan {{literal}}\"\n");
    parse_ok("x = f\"{ {'k': 1}['k'] }\"\n");
}

#[test]
fn empat_bentuk_perulangan() {
    let module = parse_ok(
        "untuk setiap x dalam data:\n    lewati\n\
         untuk i sebanyak 3:\n    lewati\n\
         untuk i dari 1 sampai 5:\n    lewati\n\
         untuk i dari 5 sampai 1 langkah -1:\n    lewati\n",
    );
    assert_eq!(module.body.len(), 4);

    match &module.body[0].kind {
        StmtKind::For { iter, .. } => assert!(matches!(iter, ForIter::Each(_))),
        other => panic!("bukan perulangan: {other:?}"),
    }
    match &module.body[1].kind {
        StmtKind::For { iter, .. } => assert!(matches!(iter, ForIter::Times(_))),
        other => panic!("bukan perulangan: {other:?}"),
    }
    match &module.body[2].kind {
        StmtKind::For { iter, .. } => assert!(matches!(iter, ForIter::Range { step: None, .. })),
        other => panic!("bukan perulangan: {other:?}"),
    }
    match &module.body[3].kind {
        StmtKind::For { iter, .. } => assert!(matches!(iter, ForIter::Range { step: Some(_), .. })),
        other => panic!("bukan perulangan: {other:?}"),
    }
}

#[test]
fn impor_python_ditandai() {
    let module = parse_ok("impor python \"os.path\" sebagai jalur\n");
    match &module.body[0].kind {
        StmtKind::Import { modules } => {
            assert_eq!(modules[0].name, "os.path");
            assert_eq!(modules[0].asname.as_deref(), Some("jalur"));
            assert!(modules[0].python);
        }
        other => panic!("bukan impor: {other:?}"),
    }

    let module = parse_ok("dari python \"os.path\" impor join sebagai gabung\n");
    match &module.body[0].kind {
        StmtKind::FromImport {
            module,
            names,
            python,
            ..
        } => {
            assert_eq!(module, "os.path");
            assert!(python);
            assert_eq!(names[0].name, "join");
            assert_eq!(names[0].asname.as_deref(), Some("gabung"));
        }
        other => panic!("bukan dari impor: {other:?}"),
    }
}

#[test]
fn kelas_dengan_kata_kunci_metakelas() {
    let module = parse_ok("kelas Anak(Induk, metaclass=Meta):\n    lewati\n");
    match &module.body[0].kind {
        StmtKind::Class(class) => {
            assert_eq!(class.name, "Anak");
            assert_eq!(class.bases.len(), 1);
            assert_eq!(class.keywords.len(), 1);
            assert!(matches!(&class.keywords[0], Arg::Keyword { name, .. } if name == "metaclass"));
        }
        other => panic!("bukan kelas: {other:?}"),
    }
}

#[test]
fn parameter_posonly_kwonly_dan_vararg() {
    let module = parse_ok("fungsi f(a, b=1, /, c=2, *args, d, e=3, **kwargs):\n    lewati\n");
    match &module.body[0].kind {
        StmtKind::Function(function) => {
            assert_eq!(function.params.posonly.len(), 2);
            assert_eq!(function.params.args.len(), 1);
            assert!(function.params.vararg.is_some());
            assert_eq!(function.params.kwonly.len(), 2);
            assert!(function.params.kwarg.is_some());
            assert_eq!(function.params.posonly[0].kind, ParamKind::PosOnly);
            assert_eq!(function.params.kwonly[0].kind, ParamKind::KwOnly);
        }
        other => panic!("bukan fungsi: {other:?}"),
    }
}

#[test]
fn pola_pengikat_sebagai() {
    let module = parse_ok("cocokkan x:\n    kasus str() sebagai teks:\n        lewati\n");
    match &module.body[0].kind {
        StmtKind::Match { cases, .. } => match &cases[0].pattern {
            Pattern::As { pattern, name } => {
                assert_eq!(name, "teks");
                assert!(matches!(pattern.as_ref(), Pattern::Class { .. }));
            }
            other => panic!("bukan pola sebagai: {other:?}"),
        },
        other => panic!("bukan cocokkan: {other:?}"),
    }
}

#[test]
fn pola_urutan_kamus_dan_penjaga() {
    parse_ok(
        "cocokkan x:\n\
         \x20   kasus [kepala, *ekor]:\n        lewati\n\
         \x20   kasus {\"k\": v, **sisa}:\n        lewati\n\
         \x20   kasus Titik(x, y) jika x > 0:\n        lewati\n",
    );
}

#[test]
fn diagnostik_penugasan_di_kondisi() {
    let result = parse("jika x = 1:\n    lewati\n");
    let diagnostic = result
        .diagnostics
        .iter()
        .find(|diagnostic| diagnostic.code == "E-PENUGASAN")
        .expect("ada diagnostik penugasan");
    assert!(diagnostic.help.as_deref().unwrap_or("").contains("=="));
}

#[test]
fn diagnostik_parameter_tanpa_default() {
    let result = parse("fungsi f(a=1, b):\n    lewati\n");
    let diagnostic = result
        .diagnostics
        .iter()
        .find(|diagnostic| diagnostic.code == "E-PARAMETER")
        .expect("ada diagnostik parameter");
    assert!(diagnostic.help.is_some());
}

#[test]
fn pemulihan_menghasilkan_beberapa_diagnostik() {
    let result = parse(
        "jika :\n    lewati\n\
         selama :\n    lewati\n\
         untuk i dalam :\n    lewati\n",
    );
    assert!(result.diagnostics.len() >= 3);
    assert!(result
        .diagnostics
        .iter()
        .all(|diagnostic| diagnostic.code.starts_with("E-")));
}

#[test]
fn semua_span_berada_di_dalam_sumber() {
    let source = "nilai = 42\ncetak(f\"{nilai}\")\n";
    let result = parse(source);
    let length = source.len() as u32;
    fn cek_span(span: aksara_ast::Span, length: u32) {
        assert!(span.start <= span.end);
        assert!(span.end <= length);
    }
    cek_span(result.module.span, length);
    for statement in &result.module.body {
        cek_span(statement.span, length);
    }
    for diagnostic in &result.diagnostics {
        cek_span(diagnostic.span, length);
    }
}

#[test]
fn sarang_dalam_menghasilkan_diagnostik_bukan_panic() {
    let source = format!("x = {}1{}\n", "(".repeat(2000), ")".repeat(2000));
    let result = parse(&source);
    assert!(result
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic.message.contains("terlalu dalam")));
}

#[test]
fn dekorator_pada_fungsi_asinkron() {
    let module = parse_ok("@dekorator\nasinkron fungsi f():\n    lewati\n");
    match &module.body[0].kind {
        StmtKind::Function(function) => {
            assert!(function.is_async);
            assert_eq!(function.decorators.len(), 1);
        }
        other => panic!("bukan fungsi: {other:?}"),
    }
}

#[test]
fn asinkron_untuk_dan_dengan() {
    let module = parse_ok(
        "asinkron fungsi f(stream):\n\
         \x20   asinkron untuk item dalam stream:\n        lewati\n\
         \x20   asinkron dengan stream sebagai nilai:\n        lewati\n",
    );
    match &module.body[0].kind {
        StmtKind::Function(function) => {
            assert!(matches!(
                function.body[0].kind,
                StmtKind::For { is_async: true, .. }
            ));
            assert!(matches!(
                function.body[1].kind,
                StmtKind::With { is_async: true, .. }
            ));
        }
        other => panic!("bukan fungsi: {other:?}"),
    }
}

#[test]
fn teks_bersebelahan_menjadi_string_concat() {
    let value = value_of("x = \"a\" f\"{b}\" 'c'\n");
    match value.kind {
        ExprKind::StringConcat(parts) => assert_eq!(parts.len(), 3),
        other => panic!("bukan gabungan teks: {other:?}"),
    }
}

#[test]
fn pernyataan_inline_dan_titik_koma() {
    parse_ok("jika benar: cetak(1); cetak(2)\n");
    parse_ok("a = 1; b = 2; c = 3\n");
}

#[test]
fn label_diagnostik_memiliki_lokasi() {
    let source = "jika x = 1:\n    lewati\n";
    let result = parse(source);
    let diagnostic = result
        .diagnostics
        .iter()
        .find(|diagnostic| diagnostic.code == "E-PENUGASAN")
        .expect("ada diagnostik");
    assert!(diagnostic.span.start > 0);
    assert!(diagnostic.span.end > diagnostic.span.start);
}
