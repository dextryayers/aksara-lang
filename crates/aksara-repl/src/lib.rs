//! REPL Aksara dengan dukungan input multiline.

use std::io::{self, BufRead, Write};

use aksara_interpreter::{format::nilai_ke_repr, HasilJalan, Interpreter};
use aksara_lexer::Lexer;
use aksara_parser::parse_tokens;

pub struct Repl {
    pub interpreter: Interpreter,
    pub prompt: String,
    pub prompt_lanjutan: String,
}

impl Repl {
    pub fn baru() -> Self {
        Self {
            interpreter: Interpreter::baru(),
            prompt: ">>> ".to_string(),
            prompt_lanjutan: "... ".to_string(),
        }
    }

    /// Menjalankan REPL sampai EOF atau perintah keluar.
    pub fn jalankan(&mut self) -> io::Result<()> {
        let stdin = io::stdin();
        let mut input = stdin.lock();
        let stdout = io::stdout();
        let mut output = stdout.lock();
        let mut buffer = String::new();
        loop {
            let prompt = if buffer.is_empty() {
                &self.prompt
            } else {
                &self.prompt_lanjutan
            };
            write!(output, "{prompt}")?;
            output.flush()?;
            let mut baris = String::new();
            let dibaca = input.read_line(&mut baris)?;
            if dibaca == 0 {
                break;
            }
            let perintah = baris.trim();
            if buffer.is_empty() {
                match perintah {
                    ":keluar" | ":q" | "keluar" => break,
                    ":bantuan" | ":h" | "bantuan" => {
                        self.cetak_bantuan(&mut output)?;
                        continue;
                    }
                    ":bersihkan" | ":c" => {
                        self.interpreter = Interpreter::baru();
                        writeln!(output, "lingkungan direset")?;
                        continue;
                    }
                    _ => {}
                }
                if let Some(jalur) = perintah.strip_prefix(":muat ") {
                    self.muat(jalur.trim(), &mut output)?;
                    continue;
                }
            }
            buffer.push_str(&baris);
            if butuh_lanjutan(&buffer) {
                continue;
            }
            let sumber = std::mem::take(&mut buffer);
            self.jalankan_chunk(&sumber, &mut output)?;
        }
        Ok(())
    }

    fn cetak_bantuan(&self, output: &mut impl Write) -> io::Result<()> {
        writeln!(output, "Perintah REPL:")?;
        writeln!(output, "  :bantuan    tampilkan bantuan ini")?;
        writeln!(output, "  :muat FILE  jalankan berkas .aks")?;
        writeln!(output, "  :bersihkan  reset lingkungan")?;
        writeln!(output, "  :keluar     keluar dari REPL")?;
        Ok(())
    }

    fn muat(&mut self, jalur: &str, output: &mut impl Write) -> io::Result<()> {
        let hasil = self.interpreter.jalankan_berkas(std::path::Path::new(jalur));
        tampilkan_hasil(&hasil, output)?;
        Ok(())
    }

    fn jalankan_chunk(&mut self, sumber: &str, output: &mut impl Write) -> io::Result<()> {
        let lexed = Lexer::new(sumber).tokenize();
        let parsed = parse_tokens(lexed.tokens, lexed.diagnostics);
        let galat = parsed
            .diagnostics
            .iter()
            .filter(|diagnostik| diagnostik.is_error())
            .count();
        if galat > 0 {
            for diagnostik in parsed.diagnostics.iter().filter(|d| d.is_error()) {
                writeln!(output, "error[{}]: {}", diagnostik.code, diagnostik.message)?;
                if let Some(help) = &diagnostik.help {
                    writeln!(output, "  bantuan: {help}")?;
                }
            }
            return Ok(());
        }
        let sebelum = self.interpreter.keluaran.len();
        let hasil = self.interpreter.jalankan_teks("<repl>", sumber, self.interpreter.global.clone());
        let keluaran = self.interpreter.keluaran[sebelum..].to_string();
        if !keluaran.is_empty() {
            write!(output, "{keluaran}")?;
            if !keluaran.ends_with('\n') {
                writeln!(output)?;
            }
        }
        match hasil {
            Ok(nilai) => {
                if !nilai.adalah_kosong() {
                    writeln!(output, "{}", nilai_ke_repr(&nilai))?;
                }
            }
            Err(lemparan) => {
                self.interpreter.catat_jejak(&lemparan.kesalahan, lemparan.span);
                write!(output, "{}", self.interpreter.format_jejak(&lemparan.kesalahan))?;
            }
        }
        Ok(())
    }
}

fn tampilkan_hasil(hasil: &HasilJalan, output: &mut impl Write) -> io::Result<()> {
    if !hasil.keluaran.is_empty() {
        write!(output, "{}", hasil.keluaran)?;
        if !hasil.keluaran.ends_with('\n') {
            writeln!(output)?;
        }
    }
    if let Some(kesalahan) = &hasil.kesalahan {
        writeln!(output, "{}: {}", kesalahan.tipe.nama, kesalahan.pesan)?;
    }
    Ok(())
}

/// Menentukan apakah input masih membutuhkan baris lanjutan.
pub fn butuh_lanjutan(sumber: &str) -> bool {
    let lexed = Lexer::new(sumber).tokenize();
    if lexed
        .diagnostics
        .iter()
        .any(|diagnostik| diagnostik.code.contains("KURUNG") || diagnostik.code.contains("TEKS"))
    {
        return true;
    }
    let mut kurung = 0i32;
    let mut dalam_teks: Option<char> = None;
    let mut escape = false;
    for ch in sumber.chars() {
        if let Some(kutip) = dalam_teks {
            if escape {
                escape = false;
            } else if ch == '\\' {
                escape = true;
            } else if ch == kutip {
                dalam_teks = None;
            }
            continue;
        }
        match ch {
            '\'' | '"' => dalam_teks = Some(ch),
            '(' | '[' | '{' => kurung += 1,
            ')' | ']' | '}' => kurung -= 1,
            _ => {}
        }
    }
    if kurung > 0 || dalam_teks.is_some() {
        return true;
    }
    let tanpa_komentar = sumber
        .lines()
        .last()
        .unwrap_or("")
        .split('#')
        .next()
        .unwrap_or("")
        .trim_end();
    tanpa_komentar.ends_with(':') || tanpa_komentar.ends_with('\\')
}
