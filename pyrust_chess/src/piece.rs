use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;
use pyo3::types::PyType;
use shakmaty::{Color, Piece, Role};
use std::convert::TryFrom;

#[pyclass(module = "pyrust_chess", from_py_object, eq, name = "Piece")]
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct PyPiece(pub Piece);

impl From<Piece> for PyPiece {
    #[inline]
    fn from(inner: Piece) -> Self {
        Self(inner)
    }
}

#[pymethods]
impl PyPiece {
    #[new]
    #[pyo3(signature = (piece_type, color))]
    pub fn py_new(piece_type: u8, color: bool) -> PyResult<Self> {
        let role = Role::try_from(piece_type)
            .map_err(|_| PyValueError::new_err(format!("invalid piece type: {piece_type}")))?;
        let color = if color { Color::White } else { Color::Black };
        Ok(PyPiece(Piece { role, color }))
    }

    #[classattr]
    #[allow(non_upper_case_globals)]
    const __match_args__: (&'static str, &'static str) = ("piece_type", "color");

    #[getter]
    fn piece_type(&self) -> u8 {
        self.0.role as u8
    }

    #[setter]
    fn set_piece_type(&mut self, piece_type: u8) -> PyResult<()> {
        let role = Role::try_from(piece_type)
            .map_err(|_| PyValueError::new_err(format!("invalid piece type: {piece_type}")))?;
        self.0.role = role;
        Ok(())
    }

    #[getter]
    fn color(&self) -> bool {
        self.0.color.is_white()
    }

    #[setter]
    fn set_color(&mut self, color: bool) {
        self.0.color = if color { Color::White } else { Color::Black };
    }

    fn symbol(&self) -> char {
        self.0.char()
    }

    #[pyo3(signature = (*, invert_color=false))]
    const fn unicode_symbol(&self, invert_color: bool) -> char {
        // xor not const yet
        let color = if invert_color {
            self.0.color.other()
        } else {
            self.0.color
        };
        match (color, self.0.role) {
            (Color::White, Role::Rook) => '♖',
            (Color::Black, Role::Rook) => '♜',
            (Color::White, Role::Knight) => '♘',
            (Color::Black, Role::Knight) => '♞',
            (Color::White, Role::Bishop) => '♗',
            (Color::Black, Role::Bishop) => '♝',
            (Color::White, Role::Queen) => '♕',
            (Color::Black, Role::Queen) => '♛',
            (Color::White, Role::King) => '♔',
            (Color::Black, Role::King) => '♚',
            (Color::White, Role::Pawn) => '♙',
            (Color::Black, Role::Pawn) => '♟',
        }
    }

    fn __hash__(&self) -> isize {
        self.piece_type() as isize + if self.color() { -1 } else { 5 }
    }

    fn __repr__(&self) -> String {
        format!("Piece.from_symbol('{}')", self.symbol())
    }

    fn __str__(&self) -> char {
        self.symbol()
    }

    #[classmethod]
    fn from_symbol(_cls: &Bound<'_, PyType>, ch: char) -> PyResult<Self> {
        match Piece::from_char(ch) {
            Some(inner) => Ok(PyPiece(inner)),
            None => Err(PyValueError::new_err(format!(
                "invalid piece symbol: '{ch}'"
            ))),
        }
    }
}
