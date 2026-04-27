pub mod py_move;
pub mod piece;
pub mod square_set;

use py_move::PyMove;
use piece::PyPiece;
use pyo3::prelude::*;
use square_set::{CarryRipplerIter, SquareSet, SquareSetIter, SquareSetRevIter};

#[pymodule]
fn rust_chess(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<SquareSet>()?;
    m.add_class::<SquareSetIter>()?;
    m.add_class::<SquareSetRevIter>()?;
    m.add_class::<CarryRipplerIter>()?;
    m.add_class::<PyPiece>()?;
    m.add_class::<PyMove>()?;

    let py = m.py();
    let python_code = std::ffi::CString::new(include_str!("patch.py")).unwrap();

    let funcs = pyo3::types::PyModule::from_code(py, &python_code, c"patch.py", c"rust_chess_py")?;
    m.add("patch_supported", funcs.getattr("patch_supported")?)?;

    Ok(())
}
