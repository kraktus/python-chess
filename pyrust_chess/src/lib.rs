#![warn(clippy::pedantic)]

pub mod base_board;
pub mod board;
pub mod board_status;
pub mod epd_ops;
pub mod outcome;
pub mod piece;
pub mod py_move;
pub mod square_set;
pub mod util;

use base_board::{BaseBoard, OccupiedCo};
use board_status::Status;
use outcome::{PyOutcome, PyTermination};
use piece::PyPiece;
use py_move::PyMove;
use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;

use square_set::{CarryRipplerIter, SquareSet, SquareSetIter, SquareSetRevIter};

pyo3::create_exception!(pyrust_chess, InvalidMoveError, PyValueError);
pyo3::create_exception!(pyrust_chess, IllegalMoveError, PyValueError);
pyo3::create_exception!(pyrust_chess, AmbiguousMoveError, PyValueError);

#[pymodule]
fn pyrust_chess(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<SquareSet>()?;
    m.add_class::<Status>()?;
    m.add_class::<SquareSetIter>()?;
    m.add_class::<SquareSetRevIter>()?;
    m.add_class::<CarryRipplerIter>()?;
    m.add_class::<PyPiece>()?;
    m.add_class::<PyMove>()?;
    m.add_class::<BaseBoard>()?;
    m.add_class::<OccupiedCo>()?;
    m.add_class::<board::Board>()?;
    m.add_class::<PyTermination>()?;
    m.add_class::<PyOutcome>()?;
    m.add_class::<board::LegalMoveGenerator>()?;
    m.add_class::<board::PseudoLegalMoveGenerator>()?;
    m.add("IllegalMoveError", m.py().get_type::<IllegalMoveError>())?;
    m.add(
        "AmbiguousMoveError",
        m.py().get_type::<AmbiguousMoveError>(),
    )?;
    m.add("InvalidMoveError", m.py().get_type::<InvalidMoveError>())?;

    Ok(())
}
