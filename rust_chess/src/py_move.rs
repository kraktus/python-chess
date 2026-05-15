use std::convert::TryFrom;
use std::hash::{DefaultHasher, Hash, Hasher};
use std::str::FromStr;

use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;
use pyo3::types::PyType;
use shakmaty::uci::UciMove;
use shakmaty::{Move, Role, Square};

use crate::util::{PyRole, PySquare};

#[pyclass(module = "rust_chess", from_py_object, eq, name = "Move")]
#[derive(Clone, PartialEq, Eq, Hash)]
pub struct PyMove {
    pub inner: UciMove,
}

impl From<Move> for PyMove {
    fn from(value: Move) -> Self {
        Self {
            inner: value.to_uci(shakmaty::CastlingMode::Standard),
        }
    }
}

impl From<&Move> for PyMove {
    fn from(value: &Move) -> Self {
        (*value).into()
    }
}

#[pymethods]
impl PyMove {
    #[new]
    #[pyo3(signature = (from_square, to_square, promotion=None, drop=None))]
    fn py_new(
        from_square: PySquare,
        to_square: PySquare,
        promotion: Option<PyRole>,
        drop: Option<PyRole>,
    ) -> PyResult<Self> {
        let inner = if from_square.0 == Square::A1
            && to_square.0 == Square::A1
            && promotion.is_none()
            && drop.is_none()
        {
            UciMove::Null
        } else if let Some(drop_piece) = drop {
            if from_square.0 != to_square.0 {
                return Err(PyValueError::new_err(
                    "drop must have from_square == to_square",
                ));
            }
            UciMove::Put {
                role: drop_piece.0,
                to: to_square.0,
            }
        } else {
            UciMove::Normal {
                from: from_square.0,
                to: to_square.0,
                promotion: promotion.map(|x| x.0),
            }
        };

        Ok(PyMove { inner })
    }

    #[getter]
    fn from_square(&self) -> u8 {
        match &self.inner {
            UciMove::Normal { from, .. } => (*from).into(),
            UciMove::Put { to, .. } => (*to).into(),
            UciMove::Null => 0,
        }
    }

    #[setter]
    fn set_from_square(&mut self, from_square: PySquare) {
        match &mut self.inner {
            UciMove::Normal { from, .. } => *from = from_square.0,
            UciMove::Put { .. } => {}
            UciMove::Null => {}
        }
    }

    #[getter]
    fn to_square(&self) -> u8 {
        match &self.inner {
            UciMove::Normal { to, .. } | UciMove::Put { to, .. } => (*to).into(),
            UciMove::Null => 0,
        }
    }

    #[setter]
    fn set_to_square(&mut self, to_square: PySquare) {
        match &mut self.inner {
            UciMove::Normal { to, .. } => *to = to_square.0,
            UciMove::Put { to, .. } => *to = to_square.0,
            UciMove::Null => {
                panic!("Attempting to set to square on Null move")
            }
        }
    }

    #[getter]
    fn promotion(&self) -> Option<u8> {
        match &self.inner {
            UciMove::Normal {
                promotion: Some(role),
                ..
            } => Some(*role as u8),
            _ => None,
        }
    }

    #[setter]
    fn set_promotion(&mut self, promotion: Option<u8>) -> PyResult<()> {
        match &mut self.inner {
            UciMove::Normal {
                promotion: promo, ..
            } => {
                *promo = match promotion {
                    Some(p) => Some(
                        Role::try_from(p)
                            .map_err(|_| PyValueError::new_err("invalid promotion piece type"))?,
                    ),
                    None => None,
                };
                Ok(())
            }
            _ => {
                if promotion.is_some() {
                    Err(PyValueError::new_err(
                        "cannot set promotion on drop or null move",
                    ))
                } else {
                    Ok(())
                }
            }
        }
    }

    #[getter]
    fn drop(&self) -> Option<u8> {
        match &self.inner {
            UciMove::Put { role, .. } => Some(*role as u8),
            _ => None,
        }
    }

    #[setter]
    fn set_drop(&mut self, drop: Option<PyRole>) -> PyResult<()> {
        match &mut self.inner {
            UciMove::Put { role, .. } => {
                if let Some(p) = drop {
                    *role = p.0;
                }
                Ok(())
            }
            _ => {
                if drop.is_some() {
                    Err(PyValueError::new_err(
                        "cannot set drop on normal or null move",
                    ))
                } else {
                    Ok(())
                }
            }
        }
    }

    fn uci(&self) -> String {
        self.inner.to_string()
    }

    #[must_use] 
    pub fn xboard(&self) -> String {
        match self.inner {
            // because shakmaty consider null move to be 0000
            UciMove::Null => "@@@@".to_string(),
            _ => self.inner.to_string(),
        }
    }

    // not possible to derive because pymove is mutable
    fn __hash__(&self) -> u64 {
        let mut s = DefaultHasher::new();
        self.hash(&mut s);
        s.finish()
    }

    fn __bool__(&self) -> bool {
        !matches!(self.inner, UciMove::Null)
    }

    fn __repr__(&self) -> String {
        format!("Move.from_uci('{}')", self.uci())
    }

    fn __str__(&self) -> String {
        self.uci()
    }

    #[classmethod]
    fn from_uci(cls: &Bound<'_, PyType>, uci: &str) -> PyResult<Self> {
        if let Ok(inner) = UciMove::from_str(uci) { Ok(PyMove { inner }) } else {
            let py = cls.py();
            let chess_module = py.import("chess")?;
            let invalid_move_error = chess_module.getattr("InvalidMoveError")?;
            Err(PyErr::from_value(
                invalid_move_error.call1((format!("invalid uci: {uci:?}"),))?,
            ))
        }
    }

    #[classmethod]
    fn null(_cls: &Bound<'_, PyType>) -> Self {
        PyMove {
            inner: UciMove::Null,
        }
    }

    fn __copy__(&self) -> Self {
        self.clone()
    }

    fn __deepcopy__(&self, _memo: &Bound<'_, PyAny>) -> Self {
        self.clone()
    }
}
