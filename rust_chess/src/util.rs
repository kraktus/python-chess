use pyo3::IntoPyObjectExt;
use pyo3::exceptions::{PyKeyError, PyTypeError};
use pyo3::prelude::*;
use pyo3::types::{PyAny, PyTuple, PyType};
use shakmaty::{Bitboard, Square, Color, Role};

use crate::square_set::SquareSet;

#[macro_export]
macro_rules! derive_deref_and_mut {
    ($wrapper:ident, $inner:ty, $field:tt) => {
        impl std::ops::Deref for $wrapper {
            type Target = $inner;

            #[inline]
            fn deref(&self) -> &Self::Target {
                &self.$field
            }
        }

        impl std::ops::DerefMut for $wrapper {
            #[inline]
            fn deref_mut(&mut self) -> &mut Self::Target {
                &mut self.$field
            }
        }
    };
    ($wrapper:ident, $inner:ty) => {
        derive_deref_and_mut!($wrapper, $inner, 0);
    };
}

use derive_deref_and_mut;

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


pub struct IntoSquareSet(Bitboard);
derive_deref_and_mut!(IntoSquareSet, Bitboard);

impl FromPyObject<'_, '_> for IntoSquareSet {
    type Error = PyErr;

    fn extract(obj: Borrowed<'_, '_, PyAny>) -> Result<Self, Self::Error> {
        Ok(Self(extract_mask(&obj)?))
    }
}

pub struct PySquare(pub Square);
derive_deref_and_mut!(PySquare, Square);

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
derive_deref_and_mut!(PyRole, Role);

impl FromPyObject<'_, '_> for PyRole {
    type Error = PyErr;

    fn extract(obj: Borrowed<'_, '_, PyAny>) -> Result<Self, Self::Error> {
        let int: i32 = obj.extract()?;
        Ok(Self(int.try_into().or_else(|_| {
            Err(PyTypeError::new_err(format!("Role out of bounds: {int}")))
        })?))
    }
}

pub struct PyColor(pub Color);
derive_deref_and_mut!(PyColor, Color);

impl FromPyObject<'_, '_> for PyColor {
    type Error = PyErr;

    fn extract(obj: Borrowed<'_, '_, PyAny>) -> Result<Self, Self::Error> {
        let c = obj.extract().map(Color::from_white)?;
        Ok(Self(c))
    }
}
