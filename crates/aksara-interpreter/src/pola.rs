use std::collections::HashMap;

use aksara_ast::Pattern;

use crate::interpreter::Interpreter;
use crate::nilai::{Env, Hasil, Nilai};

pub fn cocokkan(
    _interpreter: &mut Interpreter,
    _pola: &Pattern,
    _nilai: &Nilai,
    _ikatan: &mut HashMap<String, Nilai>,
    _env: &Env,
) -> Hasil<bool> {
    Ok(false)
}
