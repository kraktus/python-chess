use pyo3::exceptions::{PyAssertionError, PyTypeError};
use pyo3::prelude::*;
use pyo3::types::PyAny;
use shakmaty::{Bitboard, Color, Role, Square};

use crate::square_set::SquareSet;

pub fn extract_mask(value: &Bound<'_, PyAny>) -> PyResult<Bitboard> {
    if let Ok(ss) = value.extract::<SquareSet>() {
        return Ok(ss.bb);
    }

    if let Ok(val) = value.call_method0("__int__")
        && let Ok(masked) = val.call_method1("__and__", (Bitboard::FULL.0,))
        && let Ok(mask) = masked.extract::<u64>()
    {
        return Ok(Bitboard(mask));
    }

    let mut mask = Bitboard::EMPTY;
    if let Ok(iter) = value.try_iter() {
        for item in iter {
            let item = item?;
            let square = item.extract::<PySquare>()?;
            mask.add(square.0);
        }
        return Ok(mask);
    }

    Err(PyTypeError::new_err(
        "Expected SquareSet, int, or iterable of squares",
    ))
}

pub struct IntoSquareSet(pub Bitboard);

impl FromPyObject<'_, '_> for IntoSquareSet {
    type Error = PyErr;

    fn extract(obj: Borrowed<'_, '_, PyAny>) -> Result<Self, Self::Error> {
        Ok(Self(extract_mask(&obj)?))
    }
}

#[derive(Clone, Copy)]
pub struct PySquare(pub Square);

impl FromPyObject<'_, '_> for PySquare {
    type Error = PyErr;

    fn extract(obj: Borrowed<'_, '_, PyAny>) -> Result<Self, Self::Error> {
        let int: i32 = obj.extract()?;
        Ok(Self(int.try_into().or_else(|_| {
            Err(PyTypeError::new_err(format!("Square out of bounds: {int}")))
        })?))
    }
}

pub struct PyRole(pub Role);

impl FromPyObject<'_, '_> for PyRole {
    type Error = PyErr;

    fn extract(obj: Borrowed<'_, '_, PyAny>) -> Result<Self, Self::Error> {
        let int: i32 = obj.extract()?;
        Ok(Self(int.try_into().or_else(|_| {
            Err(PyAssertionError::new_err(format!(
                "Role out of bounds: {int}"
            )))
        })?))
    }
}

pub struct PyColor(pub Color);

impl FromPyObject<'_, '_> for PyColor {
    type Error = PyErr;

    fn extract(obj: Borrowed<'_, '_, PyAny>) -> Result<Self, Self::Error> {
        let c = obj.extract().map(Color::from_white)?;
        Ok(Self(c))
    }
}
