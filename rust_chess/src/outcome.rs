use pyo3::prelude::*;

#[pyclass(module = "rust_chess", from_py_object, eq, name = "Termination")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[allow(non_camel_case_types)]
pub enum PyTermination {
    CHECKMATE = 1,
    STALEMATE = 2,
    INSUFFICIENT_MATERIAL = 3,
    SEVENTYFIVE_MOVES = 4,
    FIVEFOLD_REPETITION = 5,
    FIFTY_MOVES = 6,
    THREEFOLD_REPETITION = 7,
    VARIANT_WIN = 8,
    VARIANT_LOSS = 9,
    VARIANT_DRAW = 10,
}

#[pymethods]
impl PyTermination {
    #[getter]
    pub fn name(&self) -> &'static str {
        match self {
            Self::CHECKMATE => "CHECKMATE",
            Self::STALEMATE => "STALEMATE",
            Self::INSUFFICIENT_MATERIAL => "INSUFFICIENT_MATERIAL",
            Self::SEVENTYFIVE_MOVES => "SEVENTYFIVE_MOVES",
            Self::FIVEFOLD_REPETITION => "FIVEFOLD_REPETITION",
            Self::FIFTY_MOVES => "FIFTY_MOVES",
            Self::THREEFOLD_REPETITION => "THREEFOLD_REPETITION",
            Self::VARIANT_WIN => "VARIANT_WIN",
            Self::VARIANT_LOSS => "VARIANT_LOSS",
            Self::VARIANT_DRAW => "VARIANT_DRAW",
        }
    }

    #[getter]
    pub fn value(&self) -> u8 {
        *self as u8
    }

    fn __repr__(&self) -> String {
        format!("<Termination.{}: {}>", self.name(), self.value())
    }

    fn __str__(&self) -> String {
        format!("Termination.{}", self.name())
    }
}

#[pyclass(module = "rust_chess", from_py_object, name = "Outcome")]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PyOutcome {
    #[pyo3(get, set)]
    pub termination: PyTermination,
    #[pyo3(get, set)]
    pub winner: Option<bool>,
}

#[pymethods]
impl PyOutcome {
    #[new]
    #[pyo3(signature = (termination, winner))]
    pub fn new(termination: PyTermination, winner: Option<bool>) -> Self {
        Self {
            termination,
            winner,
        }
    }

    pub fn result(&self) -> &'static str {
        match self.winner {
            None => "1/2-1/2",
            Some(true) => "1-0",
            Some(false) => "0-1",
        }
    }

    fn __repr__(&self) -> String {
        let winner_str = match self.winner {
            Some(true) => "True",
            Some(false) => "False",
            None => "None",
        };
        format!(
            "Outcome(termination={:?}, winner={winner_str})",
            self.termination
        )
    }

    fn __copy__(&self) -> Self {
        self.clone()
    }

    fn __deepcopy__(&self, _memo: &Bound<'_, PyAny>) -> Self {
        self.clone()
    }
}
