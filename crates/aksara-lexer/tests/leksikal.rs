//! Uji lexer untuk token, indentasi, literal, dan diagnostik.

use aksara_lexer::{lex, TokenKind};

fn kinds(source: &str) -> Vec<TokenKind> {
    lex(source)
        .tokens
        .into_iter()
        .map(|token| token.kind)
        .collect()
}

fn names(source: &str) -> Vec<String> {
    lex(source)
        .tokens
        .into_iter()
        .filter_map(|token| match token.kind {
            TokenKind::Name(name) => Some(name),
            _ => None,
        })
        .collect()
}

fn codes(source: &str) -> Vec<String> {
    lex(source)
        .diagnostics
        .into_iter()
        .map(|diagnostic| diagnostic.code)
        .collect()
}

#[test]
fn penugasan_dasar() {
    let tokens = kinds("nilai = 42\n");
    assert_eq!(
        tokens,
        vec![
            TokenKind::Name("nilai".into()),
            TokenKind::Eq,
            TokenKind::Int("42".into()),
            TokenKind::Newline,
            TokenKind::Eof,
        ]
    );
}

#[test]
fn indentasi_dan_dedentasi() {
    let tokens = kinds("jika benar:\n    lewati\ncetak(1)\n");
    assert!(tokens.contains(&TokenKind::Indent));
    assert!(tokens.contains(&TokenKind::Dedent));
    let indent = tokens
        .iter()
        .position(|kind| *kind == TokenKind::Indent)
        .expect("ada indentasi");
    let dedent = tokens
        .iter()
        .position(|kind| *kind == TokenKind::Dedent)
        .expect("ada dedentasi");
    assert!(indent < dedent);
}

#[test]
fn baris_kosong_dan_komentar_tidak_mengubah_indentasi() {
    let tokens = kinds("jika benar:\n\n    # komentar\n    lewati\ncetak(1)\n");
    let indents = tokens
        .iter()
        .filter(|kind| **kind == TokenKind::Indent)
        .count();
    let dedents = tokens
        .iter()
        .filter(|kind| **kind == TokenKind::Dedent)
        .count();
    assert_eq!(indents, 1);
    assert_eq!(dedents, 1);
}

#[test]
fn newline_di_dalam_kurung_diabaikan() {
    let tokens = kinds("data = [\n    1,\n    2,\n]\n");
    assert_eq!(
        tokens
            .iter()
            .filter(|kind| **kind == TokenKind::Newline)
            .count(),
        1
    );
    assert!(tokens.contains(&TokenKind::LBracket));
    assert!(tokens.contains(&TokenKind::RBracket));
}

#[test]
fn lanjutan_baris_eksplisit() {
    let tokens = kinds("total = 1 + \\\n    2\n");
    assert_eq!(
        tokens
            .iter()
            .filter(|kind| **kind == TokenKind::Newline)
            .count(),
        1
    );
    assert!(!codes("total = 1 + \\\n    2\n").contains(&"E-LEKSIKAL".to_string()));
}

#[test]
fn kata_kunci_dan_identifier_kontekstual() {
    let tokens = kinds("setiap sebanyak sampai langkah\n");
    assert_eq!(
        names("setiap sebanyak sampai langkah\n"),
        vec!["setiap", "sebanyak", "sampai", "langkah"]
    );
    assert_eq!(tokens.len(), 6);
    assert_eq!(
        kinds("untuk setiap i dalam data:\n    lewati\n")[0],
        TokenKind::Untuk
    );
}

#[test]
fn literal_bilangan() {
    let tokens = kinds("0 42 1_000_000 0xFF 0o755 0b1010 3.14 .5 1. 1e-9 2.5E+3 2j 3.5j 1e3j\n");
    assert!(tokens.contains(&TokenKind::Int("0".into())));
    assert!(tokens.contains(&TokenKind::Int("1_000_000".into())));
    assert!(tokens.contains(&TokenKind::Int("0xFF".into())));
    assert!(tokens.contains(&TokenKind::Int("0o755".into())));
    assert!(tokens.contains(&TokenKind::Int("0b1010".into())));
    assert!(tokens.contains(&TokenKind::Float("3.14".into())));
    assert!(tokens.contains(&TokenKind::Float(".5".into())));
    assert!(tokens.contains(&TokenKind::Float("1.".into())));
    assert!(tokens.contains(&TokenKind::Float("1e-9".into())));
    assert!(tokens.contains(&TokenKind::Float("2.5E+3".into())));
    assert!(tokens.contains(&TokenKind::Imaginary("2j".into())));
    assert!(tokens.contains(&TokenKind::Imaginary("3.5j".into())));
    assert!(tokens.contains(&TokenKind::Imaginary("1e3j".into())));
}

#[test]
fn literal_teks_dan_prefix() {
    let tokens = kinds(
        "'satu' \"dua\" '''tiga''' \"\"\"empat\"\"\" r\"x\" b\"y\" f\"{z}\" fr\"{z}\" rb\"y\"\n",
    );
    let strings = tokens
        .iter()
        .filter_map(|kind| match kind {
            TokenKind::Str(lit) => Some(lit),
            _ => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(strings.len(), 9);
    assert_eq!(strings[0].raw, "satu");
    assert_eq!(strings[2].raw, "tiga");
    assert_eq!(strings[4].prefix, "r");
    assert_eq!(strings[5].prefix, "b");
    assert_eq!(strings[6].prefix, "f");
    assert_eq!(strings[7].prefix, "fr");
    assert_eq!(strings[8].prefix, "rb");
}

#[test]
fn fstring_disimpan_mentah() {
    let tokens = kinds("f\"Halo {nama:.2f}\"\n");
    match &tokens[0] {
        TokenKind::Str(lit) => {
            assert_eq!(lit.prefix, "f");
            assert_eq!(lit.raw, "Halo {nama:.2f}");
        }
        other => panic!("token pertama bukan teks: {other:?}"),
    }
}

#[test]
fn identifier_unicode() {
    assert_eq!(names("π = 3.14\nnama_ä = \"Ani\"\n"), vec!["π", "nama_ä"]);
}

#[test]
fn operator_tambahan() {
    let tokens = kinds("a ?? b ??= c?.d?[0]?() @ e\n");
    assert!(tokens.contains(&TokenKind::QuestionQuestion));
    assert!(tokens.contains(&TokenKind::QuestionQuestionEq));
    assert!(tokens.contains(&TokenKind::QuestionDot));
    assert!(tokens.contains(&TokenKind::QuestionLBracket));
    assert!(tokens.contains(&TokenKind::QuestionLParen));
    assert!(tokens.contains(&TokenKind::At));
}

#[test]
fn prefiks_tidak_valid_menghasilkan_diagnostik() {
    assert!(codes("a = rr\"x\"\n").contains(&"E-STRING".to_string()));
    assert!(codes("a = fb\"x\"\n").contains(&"E-STRING".to_string()));
}

#[test]
fn angka_tidak_valid_menghasilkan_diagnostik() {
    assert!(codes("a = 1__000\n").contains(&"E-ANGKA".to_string()));
    assert!(codes("a = 1_\n").contains(&"E-ANGKA".to_string()));
    assert!(codes("a = 0x_FF\n").contains(&"E-ANGKA".to_string()));
    assert!(codes("a = 0123\n").contains(&"E-ANGKA".to_string()));
}

#[test]
fn tab_ditolak() {
    assert!(codes("\ta = 1\n").contains(&"E-INDENTASI".to_string()));
    assert!(codes("a\t= 1\n").contains(&"E-INDENTASI".to_string()));
}

#[test]
fn karakter_tidak_dikenal() {
    assert!(codes("a = 1 $\n").contains(&"E-LEKSIKAL".to_string()));
    assert!(codes("a = !b\n").contains(&"E-LEKSIKAL".to_string()));
    assert!(codes("a = b ? c\n").contains(&"E-LEKSIKAL".to_string()));
}

#[test]
fn lexer_selalu_menghasilkan_eof() {
    for source in ["", "\n", "# komentar", "jika benar:\n", "\"tidak ditutup"] {
        let tokens = lex(source).tokens;
        assert_eq!(
            tokens.last().map(|token| &token.kind),
            Some(&TokenKind::Eof)
        );
    }
}
