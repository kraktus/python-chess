pub mod base_board;
pub mod piece;
pub mod py_move;
pub mod util;
pub mod square_set;

use base_board::{OccupiedCo, BaseBoard};
use piece::PyPiece;
use py_move::PyMove;
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
    m.add_class::<BaseBoard>()?;
    m.add_class::<OccupiedCo>()?;

    Ok(())
}
