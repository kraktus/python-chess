use pyo3::prelude::*;



/// A Python module implemented in Rust.
#[pymodule]
mod rust_chess {
    use pyo3::prelude::*;
    use shakmaty::Bitboard;

    /// Formats the sum of two numbers as string.
    #[pyfunction]
    fn sum_as_string(a: usize, b: usize) -> PyResult<String> {
        Ok((a + b).to_string())
    }

    // TODO FIXME probably not interesting perf-wise to put on rust-side
    // but interesting to start with it first to see hurdle with pyo3.
    #[pyclass]
    struct SquareSet {
        bb: Bitboard
    }

    #[pymethods]
    impl SquareSet {
        #[new]
        fn py_new(value: u64) -> PyResult<Self> {
            Ok(Self {
                bb: Bitboard(value)
            })
        }

        #[getter]
        fn get_mask(&self) -> PyResult<u64> {
            Ok(self.bb.0)
        }

        #[setter]
        fn set_mask(&mut self, bb: u64) {
            self.bb = Bitboard(bb);
        }
    }
}
