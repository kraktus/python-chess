use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;
use pyo3::types::PyType;
use shakmaty::{Color, Piece, Role};
use std::convert::TryFrom;

#[pyclass(module = "rust_chess", from_py_object, eq, name = "Piece")]
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct PyPiece {
    pub inner: Piece,
}

#[pymethods]
impl PyPiece {
    #[new]
    #[pyo3(signature = (piece_type, color))]
    fn py_new(piece_type: u8, color: bool) -> PyResult<Self> {
        let role = Role::try_from(piece_type)
            .map_err(|_| PyValueError::new_err(format!("invalid piece type: {}", piece_type)))?;
        let color = if color { Color::White } else { Color::Black };
        Ok(PyPiece {
            inner: Piece { role, color },
        })
    }

    #[classattr]
    #[allow(non_upper_case_globals)]
    const __match_args__: (&'static str, &'static str) = ("piece_type", "color");

    #[getter]
    fn piece_type(&self) -> u8 {
        self.inner.role as u8
    }

    #[setter]
    fn set_piece_type(&mut self, piece_type: u8) -> PyResult<()> {
        let role = Role::try_from(piece_type)
            .map_err(|_| PyValueError::new_err(format!("invalid piece type: {}", piece_type)))?;
        self.inner.role = role;
        Ok(())
    }

    #[getter]
    fn color(&self) -> bool {
        self.inner.color.is_white()
    }

    #[setter]
    fn set_color(&mut self, color: bool) {
        self.inner.color = if color { Color::White } else { Color::Black };
    }

    fn symbol(&self) -> char {
        self.inner.char()
    }

    #[pyo3(signature = (*, invert_color=false))]
    const fn unicode_symbol(&self, invert_color: bool) -> char {
        // xor not const yet
        let color = if invert_color {
            self.inner.color.other()
        } else {
            self.inner.color
        };
        match (color, self.inner.role) {
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

    // do not implement for now
    // fn _repr_svg_(slf: PyRef<'_, Self>, py: Python<'_>) -> PyResult<String> {
    //     let chess_svg = py.import("chess.svg")?;
    //     let p = slf.into_py_any(py)?;
    //     let kwargs = pyo3::types::PyDict::new(py);
    //     kwargs.set_item("size", 45)?;
    //     let svg = chess_svg.call_method("piece", (p,), Some(&kwargs))?;
    //     svg.extract()
    // }

    #[classmethod]
    fn from_symbol(_cls: &Bound<'_, PyType>, ch: char) -> PyResult<Self> {
        match Piece::from_char(ch) {
            Some(inner) => Ok(PyPiece { inner }),
            None => Err(PyValueError::new_err(format!(
                "invalid piece symbol: '{ch}'"
            ))),
        }
    }
}
