//! Uji integrasi CLI `aksara periksa`.

use std::path::PathBuf;
use std::process::{Command, Stdio};

fn bin() -> &'static str {
    env!("CARGO_BIN_EXE_aksara")
}

fn contoh(nama: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../contoh")
        .join(nama)
}

fn jalankan(args: &[&str], stdin: Option<&str>) -> std::process::Output {
    let mut command = Command::new(bin());
    command
        .args(args)
        .stdin(if stdin.is_some() {
            Stdio::piped()
        } else {
            Stdio::null()
        })
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    let mut child = command.spawn().expect("CLI dapat dijalankan");
    if let Some(input) = stdin {
        use std::io::Write as _;
        child
            .stdin
            .as_mut()
            .expect("stdin pipa")
            .write_all(input.as_bytes())
            .expect("menulis stdin");
    }
    child.wait_with_output().expect("CLI selesai")
}

#[test]
fn berkas_valid_keluar_dengan_kode_nol() {
    let path = contoh("01_halo.aks");
    let output = jalankan(&["periksa", path.to_str().unwrap()], None);
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("OK"));
}

#[test]
fn kesalahan_sintaks_keluar_dengan_kode_satu_dan_bantuan() {
    let output = jalankan(
        &["periksa", "-", "--tanpa-warna"],
        Some("jika x = 1:\n    lewati\n"),
    );
    assert_eq!(output.status.code(), Some(1));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("E-PENUGASAN"));
    assert!(stderr.contains(":1:8"));
    assert!(stderr.contains("bantuan:"));
}

#[test]
fn berkas_tidak_ada_keluar_dengan_kode_dua() {
    let output = jalankan(
        &["periksa", "/tmp/aksara-berkas-yang-tidak-mungkin-ada.aks"],
        None,
    );
    assert_eq!(output.status.code(), Some(2));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(!stderr.is_empty());
}

#[test]
fn stdin_didukung() {
    let output = jalankan(&["periksa", "-", "--ringkas"], Some("nilai = 1\n"));
    assert!(output.status.success());
    assert!(String::from_utf8_lossy(&output.stdout).contains("OK <stdin>"));
}

#[test]
fn keluaran_json_valid() {
    let path = contoh("02_kendali.aks");
    let output = jalankan(&["periksa", "--json", path.to_str().unwrap()], None);
    assert!(output.status.success());
    let value: serde_json::Value = serde_json::from_slice(&output.stdout).expect("JSON valid");
    assert!(value.get("berkas").is_some());
}

#[test]
fn keluaran_ast_multi_berkas_valid() {
    let satu = contoh("01_halo.aks");
    let dua = contoh("02_kendali.aks");
    let output = jalankan(
        &[
            "periksa",
            "--ast",
            satu.to_str().unwrap(),
            dua.to_str().unwrap(),
        ],
        None,
    );
    assert!(output.status.success());
    let value: serde_json::Value = serde_json::from_slice(&output.stdout).expect("JSON AST valid");
    assert!(value.get("body").is_some() || value.as_object().is_some());
}

#[test]
fn warna_selalu_dan_tidak_didukung() {
    let source = "jika x = 1:\n    lewati\n";
    let selalu = jalankan(&["periksa", "-", "--warna", "selalu"], Some(source));
    assert_eq!(selalu.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&selalu.stderr).contains("\u{1b}["));

    let tidak = jalankan(&["periksa", "-", "--warna", "tidak"], Some(source));
    assert_eq!(tidak.status.code(), Some(1));
    assert!(!String::from_utf8_lossy(&tidak.stderr).contains("\u{1b}["));
}
