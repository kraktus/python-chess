use pyo3::IntoPyObjectExt;
use pyo3::exceptions::PyKeyError;
use pyo3::prelude::*;
use pyo3::types::{PyAny, PyTuple, PyType};
use shakmaty::Bitboard;

use crate::util::{IntoSquareSet, PySquare, extract_mask};

#[pyclass(module = "rust_chess", from_py_object)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct SquareSet {
    pub bb: Bitboard,
}

#[pymethods]
impl SquareSet {
    #[new]
    #[pyo3(signature = (squares=None))]
    fn py_new(squares: Option<&Bound<'_, PyAny>>) -> PyResult<Self> {
        let mask = match squares {
            Some(obj) => extract_mask(obj)?,
            None => Bitboard::EMPTY,
        };
        Ok(SquareSet { bb: mask })
    }

    #[getter]
    fn get_mask(&self) -> u64 {
        self.bb.0
    }

    #[setter]
    fn set_mask(&mut self, value: u64) {
        self.bb = Bitboard(value);
    }

    fn __contains__(&self, square: PySquare) -> bool {
        self.bb.contains(square.0)
    }

    fn __iter__(&self) -> SquareSetIter {
        SquareSetIter { mask: self.bb.0 }
    }

    fn __reversed__(&self) -> SquareSetRevIter {
        SquareSetRevIter { mask: self.bb.0 }
    }

    fn __len__(&self) -> usize {
        self.bb.count()
    }

    fn add(&mut self, square: PySquare) {
        self.bb.add(square.0);
    }

    fn discard(&mut self, square: PySquare) {
        self.bb.discard(square.0);
    }

    fn remove(&mut self, square: PySquare) -> PyResult<()> {
        if self.bb.contains(square.0) {
            self.bb.discard(square.0);
            Ok(())
        } else {
            Err(PyKeyError::new_err(i32::from(square.0)))
        }
    }

    fn pop(&mut self) -> PyResult<u8> {
        if self.bb.is_empty() {
            return Err(PyKeyError::new_err("pop from empty SquareSet"));
        }
        let sq = self.bb.first().unwrap();
        self.bb.discard(sq);
        Ok(sq.into())
    }

    fn clear(&mut self) {
        self.bb = Bitboard(0);
    }

    fn isdisjoint(&self, other: IntoSquareSet) -> PyResult<bool> {
        Ok(self.bb.intersect(other.0).is_empty())
    }

    fn issubset(&self, other: IntoSquareSet) -> PyResult<bool> {
        Ok(self.bb.without(other.0).is_empty())
    }

    fn issuperset(&self, other: IntoSquareSet) -> PyResult<bool> {
        Ok(other.0.without(self.bb).is_empty())
    }

    fn union(&self, other: IntoSquareSet) -> PyResult<SquareSet> {
        Ok(SquareSet {
            bb: self.bb.with(other.0),
        })
    }

    fn __or__(&self, other: IntoSquareSet) -> PyResult<SquareSet> {
        self.union(other)
    }

    fn intersection(&self, other: IntoSquareSet) -> PyResult<SquareSet> {
        Ok(SquareSet {
            bb: self.bb.intersect(other.0),
        })
    }

    fn __and__(&self, other: IntoSquareSet) -> PyResult<SquareSet> {
        self.intersection(other)
    }

    fn difference(&self, other: IntoSquareSet) -> PyResult<SquareSet> {
        Ok(SquareSet {
            bb: self.bb.without(other.0),
        })
    }

    fn __sub__(&self, other: IntoSquareSet) -> PyResult<SquareSet> {
        self.difference(other)
    }

    fn symmetric_difference(&self, other: IntoSquareSet) -> PyResult<SquareSet> {
        Ok(SquareSet {
            bb: self.bb.toggled(other.0),
        })
    }

    fn __xor__(&self, other: IntoSquareSet) -> PyResult<SquareSet> {
        self.symmetric_difference(other)
    }

    fn copy(&self) -> SquareSet {
        *self
    }

    #[pyo3(signature = (*others))]
    fn update(&mut self, others: &Bound<'_, PyTuple>) -> PyResult<()> {
        for other in others {
            self.bb = self.bb.with(extract_mask(&other)?);
        }
        Ok(())
    }

    fn __ior__(&mut self, other: IntoSquareSet) -> PyResult<()> {
        self.bb = self.bb.with(other.0);
        Ok(())
    }

    #[pyo3(signature = (*others))]
    fn intersection_update(&mut self, others: &Bound<'_, PyTuple>) -> PyResult<()> {
        for other in others {
            self.bb = self.bb.intersect(extract_mask(&other)?);
        }
        Ok(())
    }

    fn __iand__(&mut self, other: IntoSquareSet) -> PyResult<()> {
        self.bb = self.bb.intersect(other.0);
        Ok(())
    }

    #[pyo3(signature = (*others))]
    fn difference_update(&mut self, others: &Bound<'_, PyTuple>) -> PyResult<()> {
        for other in others {
            self.bb = self.bb.without(extract_mask(&other)?);
        }
        Ok(())
    }

    fn __isub__(&mut self, other: IntoSquareSet) -> PyResult<()> {
        self.bb = self.bb.without(other.0);
        Ok(())
    }

    #[pyo3(signature = (*others))]
    fn symmetric_difference_update(&mut self, others: &Bound<'_, PyTuple>) -> PyResult<()> {
        for other in others {
            self.bb = self.bb.toggled(extract_mask(&other)?);
        }
        Ok(())
    }

    fn __ixor__(&mut self, other: IntoSquareSet) -> PyResult<()> {
        self.bb = self.bb.toggled(other.0);
        Ok(())
    }

    fn carry_rippler(&self) -> CarryRipplerIter {
        CarryRipplerIter {
            mask: self.bb.0,
            subset: 0,
            first: true,
        }
    }

    fn mirror(&self) -> SquareSet {
        SquareSet {
            bb: self.bb.flip_vertical(),
        }
    }

    fn tolist(&self) -> Vec<bool> {
        let mut result = vec![false; 64];
        for i in 0..64 {
            if (self.bb.0 & (1u64 << i)) != 0 {
                result[i] = true;
            }
        }
        result
    }

    fn __bool__(&self) -> bool {
        self.bb.0 != 0
    }

    fn __eq__(&self, py: Python<'_>, other: &Bound<'_, PyAny>) -> PyResult<Py<PyAny>> {
        match extract_mask(other) {
            Ok(other_mask) => {
                let res = self.bb == other_mask;
                res.into_py_any(py)
            }
            Err(_) => py.NotImplemented().into_py_any(py),
        }
    }

    fn __lshift__(&self, shift: u32) -> SquareSet {
        let mask = if shift >= 64 { 0 } else { self.bb.0 << shift };
        SquareSet { bb: Bitboard(mask) }
    }

    fn __rshift__(&self, shift: u32) -> SquareSet {
        let mask = if shift >= 64 { 0 } else { self.bb.0 >> shift };
        SquareSet { bb: Bitboard(mask) }
    }

    fn __ilshift__(&mut self, shift: u32) {
        self.bb.0 = if shift >= 64 { 0 } else { self.bb.0 << shift };
    }

    fn __irshift__(&mut self, shift: u32) {
        self.bb.0 = if shift >= 64 { 0 } else { self.bb.0 >> shift };
    }

    fn __invert__(&self) -> SquareSet {
        SquareSet {
            bb: self.bb.toggled(Bitboard::FULL),
        }
    }

    fn __int__(&self) -> u64 {
        self.bb.0
    }

    fn __index__(&self) -> u64 {
        self.bb.0
    }

    fn __repr__(&self) -> String {
        let hex = format!("{:016x}", self.bb.0);
        format!(
            "SquareSet(0x{}_{}_{}_{})",
            &hex[0..4],
            &hex[4..8],
            &hex[8..12],
            &hex[12..16]
        )
    }

    fn __str__(&self) -> String {
        let mut str_repr = format!("{:?}", self.bb);
        // shakmaty add a trailing \n because only used for debug
        str_repr.pop();
        str_repr
    }

    #[classmethod]
    fn ray(_cls: &Bound<'_, PyType>, a: PySquare, b: PySquare) -> SquareSet {
        SquareSet {
            bb: shakmaty::attacks::ray(a.0, b.0),
        }
    }

    #[classmethod]
    fn between(_cls: &Bound<'_, PyType>, a: PySquare, b: PySquare) -> SquareSet {
        SquareSet {
            bb: shakmaty::attacks::between(a.0, b.0),
        }
    }

    #[classmethod]
    fn from_square(_cls: &Bound<'_, PyType>, square: PySquare) -> SquareSet {
        SquareSet {
            bb: Bitboard::from_square(square.0),
        }
    }
}

#[pyclass(module = "rust_chess")]
pub struct SquareSetIter {
    pub mask: u64,
}

#[pymethods]
impl SquareSetIter {
    fn __iter__(slf: PyRef<'_, Self>) -> PyRef<'_, Self> {
        slf
    }

    fn __next__(mut slf: PyRefMut<'_, Self>) -> Option<u8> {
        if slf.mask == 0 {
            None
        } else {
            let lsb = slf.mask.trailing_zeros() as u8;
            slf.mask &= slf.mask - 1;
            Some(lsb)
        }
    }
}

#[pyclass(module = "rust_chess")]
pub struct SquareSetRevIter {
    pub mask: u64,
}

#[pymethods]
impl SquareSetRevIter {
    fn __iter__(slf: PyRef<'_, Self>) -> PyRef<'_, Self> {
        slf
    }

    fn __next__(mut slf: PyRefMut<'_, Self>) -> Option<u8> {
        if slf.mask == 0 {
            None
        } else {
            let msb = 63 - slf.mask.leading_zeros() as u8;
            slf.mask &= !(1u64 << msb);
            Some(msb)
        }
    }
}

#[pyclass(module = "rust_chess")]
pub struct CarryRipplerIter {
    mask: u64,
    subset: u64,
    first: bool,
}

#[pymethods]
impl CarryRipplerIter {
    fn __iter__(slf: PyRef<'_, Self>) -> PyRef<'_, Self> {
        slf
    }

    fn __next__(mut slf: PyRefMut<'_, Self>) -> Option<u64> {
        if slf.first {
            slf.first = false;
            Some(slf.subset)
        } else if slf.subset == slf.mask {
            None
        } else {
            slf.subset = slf.subset.wrapping_sub(slf.mask) & slf.mask;
            Some(slf.subset)
        }
    }
}
