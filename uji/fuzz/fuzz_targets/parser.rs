#![no_main]

use std::panic::{catch_unwind, AssertUnwindSafe};

use aksara_parser::parse;
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    // Parser hanya menerima teks UTF-8. Konversi lossy memastikan setiap byte
    // tetap menjadi masukan yang bermakna tanpa memicu panic konversi.
    let source = String::from_utf8_lossy(data);
    let hasil = catch_unwind(AssertUnwindSafe(|| parse(&source)));
    if hasil.is_err() {
        panic!("parser panic untuk masukan {} byte", data.len());
    }
    if let Ok(result) = hasil {
        let panjang = source.len() as u32;
        for diagnostic in &result.diagnostics {
            assert!(diagnostic.span.start <= diagnostic.span.end);
            assert!(diagnostic.span.end <= panjang);
        }
        assert!(result.module.span.end <= panjang);
    }
});
