use std::fs;
use std::io::{self, Read};
use std::path::PathBuf;
use std::process::ExitCode;

use aksara_diagnostik::{
    render_all, render_json, render_short, ColorMode, Severity, SourceFile,
};
use aksara_lexer::Lexer;
use aksara_parser::parse_tokens;
use serde_json::json;

const VERSI: &str = env!("CARGO_PKG_VERSION");

fn main() -> ExitCode {
    match run() {
        Ok(code) => code,
        Err(message) => {
            eprintln!("aksara: {message}");
            ExitCode::from(2)
        }
    }
}

fn run() -> Result<ExitCode, String> {
    let mut args = std::env::args().skip(1).collect::<Vec<_>>();
    if args.is_empty() {
        print_bantuan();
        return Ok(ExitCode::from(2));
    }

    match args.remove(0).as_str() {
        "--versi" | "-V" | "versi" => {
            println!("aksara {VERSI}");
            Ok(ExitCode::SUCCESS)
        }
        "--bantuan" | "-h" | "bantuan" => {
            print_bantuan();
            Ok(ExitCode::SUCCESS)
        }
        "periksa" => periksa(args),
        perintah => Err(format!("perintah `{perintah}` tidak dikenal")),
    }
}

#[derive(Default)]
struct OpsiPeriksa {
    json: bool,
    ringkas: bool,
    ast: bool,
    warna: ColorMode,
    berkas: Vec<String>,
}

fn periksa(args: Vec<String>) -> Result<ExitCode, String> {
    let mut opsi = OpsiPeriksa {
        warna: ColorMode::Auto,
        ..OpsiPeriksa::default()
    };
    let mut index = 0;
    let mut setelah_pemisah = false;

    while index < args.len() {
        let arg = &args[index];
        if setelah_pemisah {
            opsi.berkas.push(arg.clone());
            index += 1;
            continue;
        }
        match arg.as_str() {
            "--" => setelah_pemisah = true,
            "--json" => opsi.json = true,
            "--ringkas" => opsi.ringkas = true,
            "--ast" => opsi.ast = true,
            "--tanpa-warna" => opsi.warna = ColorMode::Never,
            "--warna" => {
                index += 1;
                let mode = args
                    .get(index)
                    .ok_or_else(|| "`--warna` membutuhkan nilai auto, selalu, atau tidak".to_string())?;
                opsi.warna = parse_warna(mode)?;
            }
            value if value.starts_with("--warna=") => {
                opsi.warna = parse_warna(&value["--warna=".len()..])?;
            }
            "--bantuan" | "-h" => {
                print_bantuan_periksa();
                return Ok(ExitCode::SUCCESS);
            }
            value if value.starts_with('-') && value != "-" => {
                return Err(format!("opsi `{value}` tidak dikenal"));
            }
            _ => opsi.berkas.push(arg.clone()),
        }
        index += 1;
    }

    if opsi.json && opsi.ringkas {
        return Err("`--json` dan `--ringkas` tidak dapat dipakai bersamaan".to_string());
    }
    if opsi.berkas.is_empty() {
        return Err("tidak ada berkas yang diperiksa".to_string());
    }

    let warna = opsi.warna.enabled();
    let mut hasil_berkas = Vec::new();
    let mut ast_berkas: Vec<(String, serde_json::Value)> = Vec::new();
    let mut total_error = 0usize;
    let mut total_peringatan = 0usize;
    let mut total_berkas = 0usize;
    let mut gagal = false;

    for path in &opsi.berkas {
        let (nama, isi) = baca_berkas(path)?;
        total_berkas += 1;
        let source = SourceFile::new(nama.clone(), isi);
        let lexed = Lexer::new(&source.text).tokenize();
        let parsed = parse_tokens(lexed.tokens, lexed.diagnostics);
        let errors = parsed
            .diagnostics
            .iter()
            .filter(|diagnostic| diagnostic.is_error())
            .count();
        let warnings = parsed
            .diagnostics
            .iter()
            .filter(|diagnostic| diagnostic.severity == Severity::Warning)
            .count();
        total_error += errors;
        total_peringatan += warnings;
        if errors > 0 {
            gagal = true;
        }

        if opsi.json {
            let mut item = json!({
                "nama": source.name,
                "diagnostik": parsed
                    .diagnostics
                    .iter()
                    .map(|diagnostic| render_json(diagnostic, &source))
                    .collect::<Vec<_>>(),
                "jumlah_error": errors,
                "jumlah_peringatan": warnings,
            });
            if opsi.ast {
                item["ast"] = serde_json::to_value(&parsed.module)
                    .map_err(|error| format!("gagal menserialkan AST: {error}"))?;
            }
            hasil_berkas.push(item);
        } else {
            if opsi.ast {
                let value = serde_json::to_value(&parsed.module)
                    .map_err(|error| format!("gagal menserialkan AST: {error}"))?;
                ast_berkas.push((source.name.clone(), value));
            }
            if !parsed.diagnostics.is_empty() {
                let rendered = if opsi.ringkas {
                    let mut out = String::new();
                    for diagnostic in &parsed.diagnostics {
                        out.push_str(&render_short(diagnostic, &source));
                        out.push('\n');
                    }
                    out
                } else {
                    render_all(&parsed.diagnostics, &source, warna)
                };
                eprint!("{rendered}");
            } else if !opsi.ast {
                println!("OK {}", source.name);
            }
        }
    }

    if opsi.json {
        let value = json!({
            "versi": VERSI,
            "berkas": hasil_berkas,
            "ringkasan": {
                "berkas": total_berkas,
                "error": total_error,
                "peringatan": total_peringatan,
            }
        });
        println!(
            "{}",
            serde_json::to_string_pretty(&value)
                .map_err(|error| format!("gagal menulis JSON: {error}"))?
        );
    } else if total_berkas > 1 {
        eprintln!(
            "{total_berkas} berkas diperiksa, {total_error} error, {total_peringatan} peringatan"
        );
    }

    if !opsi.json && opsi.ast {
        cetak_ast(&ast_berkas)?;
    }

    Ok(if gagal {
        ExitCode::from(1)
    } else {
        ExitCode::SUCCESS
    })
}

fn parse_warna(value: &str) -> Result<ColorMode, String> {
    match value {
        "auto" => Ok(ColorMode::Auto),
        "selalu" | "always" => Ok(ColorMode::Always),
        "tidak" | "never" => Ok(ColorMode::Never),
        _ => Err(format!(
            "mode warna `{value}` tidak dikenal, gunakan auto, selalu, atau tidak"
        )),
    }
}

fn baca_berkas(path: &str) -> Result<(String, String), String> {
    if path == "-" {
        let mut isi = String::new();
        io::stdin()
            .read_to_string(&mut isi)
            .map_err(|error| format!("gagal membaca stdin: {error}"))?;
        return Ok(("<stdin>".to_string(), isi));
    }
    let path_buf = PathBuf::from(path);
    let isi = fs::read_to_string(&path_buf)
        .map_err(|error| format!("gagal membaca `{path}`: {error}"))?;
    Ok((path.to_string(), isi))
}

fn cetak_ast(modules: &[(String, serde_json::Value)]) -> Result<(), String> {
    let value = if modules.len() == 1 {
        modules[0].1.clone()
    } else {
        let mut map = serde_json::Map::new();
        for (name, module) in modules {
            map.insert(name.clone(), module.clone());
        }
        serde_json::Value::Object(map)
    };
    println!(
        "{}",
        serde_json::to_string_pretty(&value)
            .map_err(|error| format!("gagal menulis AST: {error}"))?
    );
    Ok(())
}

fn print_bantuan() {
    println!(
        "aksara {VERSI}\n\
         \n\
         Pemakaian:\n\
           aksara periksa [opsi] <berkas>...\n\
           aksara --versi\n\
           aksara --bantuan\n\
         \n\
         Jalankan `aksara periksa --bantuan` untuk melihat opsi pemeriksaan."
    );
}

fn print_bantuan_periksa() {
    println!(
        "aksara periksa {VERSI}\n\
         \n\
         Memeriksa sintaks berkas Aksara tanpa menjalankannya.\n\
         \n\
         Pemakaian:\n\
           aksara periksa [opsi] <berkas>...\n\
         \n\
         Opsi:\n\
           --json             keluarkan diagnostik dan AST dalam JSON\n\
           --ringkas          satu baris per diagnostik\n\
           --ast              sertakan AST dalam keluaran JSON atau cetak AST\n\
           --warna <mode>     auto, selalu, atau tidak\n\
           --tanpa-warna      sama dengan --warna tidak\n\
           -h, --bantuan      tampilkan bantuan ini\n\
         \n\
         Gunakan `-` sebagai nama berkas untuk membaca stdin.\n\
         Kode keluar: 0 bila tidak ada error, 1 bila ada error sintaks, 2 bila\n\
         terjadi kesalahan pemakaian atau berkas tidak dapat dibaca."
    );
}
