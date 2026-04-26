use pyo3::prelude::*;
use pyo3::types::{PyAny, PyModule, PyTuple, PyType};
use pyo3::exceptions::{PyKeyError, PyTypeError, PyValueError};
use pyo3::IntoPyObjectExt;
use shakmaty::Square;

const BB_ALL: u64 = 0xffff_ffff_ffff_ffff;

fn extract_mask(value: &Bound<'_, PyAny>) -> PyResult<u64> {
    if let Ok(ss) = value.extract::<SquareSet>() {
        return Ok(ss.mask);
    }
    
    if let Ok(val) = value.call_method0("__int__") {
        if let Ok(masked) = val.call_method1("__and__", (BB_ALL,)) {
            if let Ok(mask) = masked.extract::<u64>() {
                return Ok(mask);
            }
        }
    }

    let mut mask = 0u64;
    if let Ok(iter) = value.try_iter() {
        for item in iter {
            let item = item?;
            let square: u8 = item.extract()?;
            if square >= 64 {
                return Err(PyValueError::new_err("Square out of bounds"));
            }
            mask |= 1u64 << square;
        }
        return Ok(mask);
    }

    Err(PyTypeError::new_err("Expected SquareSet, int, or iterable of squares"))
}

#[pyclass(module = "rust_chess", from_py_object)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct SquareSet {
    #[pyo3(get, set)]
    pub mask: u64,
}

#[pymethods]
impl SquareSet {
    #[new]
    #[pyo3(signature = (squares=None))]
    fn py_new(squares: Option<&Bound<'_, PyAny>>) -> PyResult<Self> {
        let mask = match squares {
            Some(obj) => extract_mask(obj)?,
            None => 0,
        };
        Ok(SquareSet { mask })
    }

    fn __contains__(&self, square: u8) -> bool {
        if square >= 64 {
            false
        } else {
            (self.mask & (1u64 << square)) != 0
        }
    }

    fn __iter__(&self) -> SquareSetIter {
        SquareSetIter { mask: self.mask }
    }

    fn __reversed__(&self) -> SquareSetRevIter {
        SquareSetRevIter { mask: self.mask }
    }

    fn __len__(&self) -> usize {
        self.mask.count_ones() as usize
    }

    fn add(&mut self, square: u8) -> PyResult<()> {
        if square >= 64 {
            return Err(PyValueError::new_err("Square out of bounds"));
        }
        self.mask |= 1u64 << square;
        Ok(())
    }

    fn discard(&mut self, square: u8) {
        if square < 64 {
            self.mask &= !(1u64 << square);
        }
    }

    fn isdisjoint(&self, other: &Bound<'_, PyAny>) -> PyResult<bool> {
        let other_mask = extract_mask(other)?;
        Ok((self.mask & other_mask) == 0)
    }

    fn issubset(&self, other: &Bound<'_, PyAny>) -> PyResult<bool> {
        let other_mask = extract_mask(other)?;
        Ok((self.mask & !other_mask) == 0)
    }

    fn issuperset(&self, other: &Bound<'_, PyAny>) -> PyResult<bool> {
        let other_mask = extract_mask(other)?;
        Ok((!self.mask & other_mask) == 0)
    }

    fn union(&self, other: &Bound<'_, PyAny>) -> PyResult<SquareSet> {
        let other_mask = extract_mask(other)?;
        Ok(SquareSet { mask: self.mask | other_mask })
    }

    fn __or__(&self, other: &Bound<'_, PyAny>) -> PyResult<SquareSet> {
        self.union(other)
    }

    fn intersection(&self, other: &Bound<'_, PyAny>) -> PyResult<SquareSet> {
        let other_mask = extract_mask(other)?;
        Ok(SquareSet { mask: self.mask & other_mask })
    }

    fn __and__(&self, other: &Bound<'_, PyAny>) -> PyResult<SquareSet> {
        self.intersection(other)
    }

    fn difference(&self, other: &Bound<'_, PyAny>) -> PyResult<SquareSet> {
        let other_mask = extract_mask(other)?;
        Ok(SquareSet { mask: self.mask & !other_mask })
    }

    fn __sub__(&self, other: &Bound<'_, PyAny>) -> PyResult<SquareSet> {
        self.difference(other)
    }

    fn symmetric_difference(&self, other: &Bound<'_, PyAny>) -> PyResult<SquareSet> {
        let other_mask = extract_mask(other)?;
        Ok(SquareSet { mask: self.mask ^ other_mask })
    }

    fn __xor__(&self, other: &Bound<'_, PyAny>) -> PyResult<SquareSet> {
        self.symmetric_difference(other)
    }

    fn copy(&self) -> SquareSet {
        *self
    }

    #[pyo3(signature = (*others))]
    fn update(&mut self, others: &Bound<'_, PyTuple>) -> PyResult<()> {
        for other in others.into_iter() {
            self.mask |= extract_mask(&other)?;
        }
        Ok(())
    }

    fn __ior__(&mut self, other: &Bound<'_, PyAny>) -> PyResult<()> {
        self.mask |= extract_mask(other)?;
        Ok(())
    }

    #[pyo3(signature = (*others))]
    fn intersection_update(&mut self, others: &Bound<'_, PyTuple>) -> PyResult<()> {
        for other in others.into_iter() {
            self.mask &= extract_mask(&other)?;
        }
        Ok(())
    }

    fn __iand__(&mut self, other: &Bound<'_, PyAny>) -> PyResult<()> {
        self.mask &= extract_mask(other)?;
        Ok(())
    }

    fn difference_update(&mut self, other: &Bound<'_, PyAny>) -> PyResult<()> {
        self.mask &= !extract_mask(other)?;
        Ok(())
    }

    fn __isub__(&mut self, other: &Bound<'_, PyAny>) -> PyResult<()> {
        self.mask &= !extract_mask(other)?;
        Ok(())
    }

    fn symmetric_difference_update(&mut self, other: &Bound<'_, PyAny>) -> PyResult<()> {
        self.mask ^= extract_mask(other)?;
        Ok(())
    }

    fn __ixor__(&mut self, other: &Bound<'_, PyAny>) -> PyResult<()> {
        self.mask ^= extract_mask(other)?;
        Ok(())
    }

    fn remove(&mut self, square: u8) -> PyResult<()> {
        if square >= 64 {
            return Err(PyKeyError::new_err(square));
        }
        let mask = 1u64 << square;
        if (self.mask & mask) != 0 {
            self.mask ^= mask;
            Ok(())
        } else {
            Err(PyKeyError::new_err(square))
        }
    }

    fn pop(&mut self) -> PyResult<u8> {
        if self.mask == 0 {
            return Err(PyKeyError::new_err("pop from empty SquareSet"));
        }
        let square = self.mask.trailing_zeros() as u8;
        self.mask &= self.mask - 1;
        Ok(square)
    }

    fn clear(&mut self) {
        self.mask = 0;
    }

    fn carry_rippler(&self) -> CarryRipplerIter {
        CarryRipplerIter {
            mask: self.mask,
            subset: 0,
            first: true,
        }
    }

    fn mirror(&self) -> SquareSet {
        SquareSet { mask: self.mask.swap_bytes() }
    }

    fn tolist(&self) -> Vec<bool> {
        let mut result = vec![false; 64];
        for i in 0..64 {
            if (self.mask & (1u64 << i)) != 0 {
                result[i] = true;
            }
        }
        result
    }

    fn __bool__(&self) -> bool {
        self.mask != 0
    }

    fn __eq__(&self, py: Python<'_>, other: &Bound<'_, PyAny>) -> PyResult<Py<PyAny>> {
        match extract_mask(other) {
            Ok(other_mask) => {
                let res = self.mask == other_mask;
                res.into_py_any(py)
            }
            Err(_) => py.NotImplemented().into_py_any(py),
        }
    }

    fn __lshift__(&self, shift: u32) -> SquareSet {
        let mask = if shift >= 64 { 0 } else { (self.mask << shift) & BB_ALL };
        SquareSet { mask }
    }

    fn __rshift__(&self, shift: u32) -> SquareSet {
        let mask = if shift >= 64 { 0 } else { self.mask >> shift };
        SquareSet { mask }
    }

    fn __ilshift__(&mut self, shift: u32) -> () {
        self.mask = if shift >= 64 { 0 } else { (self.mask << shift) & BB_ALL };
    }

    fn __irshift__(&mut self, shift: u32) -> () {
        self.mask = if shift >= 64 { 0 } else { self.mask >> shift };
    }

    fn __invert__(&self) -> SquareSet {
        SquareSet { mask: !self.mask & BB_ALL }
    }

    fn __int__(&self) -> u64 {
        self.mask
    }

    fn __index__(&self) -> u64 {
        self.mask
    }

    fn __repr__(&self) -> String {
        format!("SquareSet({:#021x})", self.mask)
    }

    fn __str__(&self) -> String {
        let mut builder = String::with_capacity(128);
        for rank in (0..8).rev() {
            for file in 0..8 {
                let square = rank * 8 + file;
                if (self.mask & (1u64 << square)) != 0 {
                    builder.push('1');
                } else {
                    builder.push('.');
                }
                
                if file != 7 {
                    builder.push(' ');
                }
            }
            if rank != 0 {
                builder.push('\n');
            }
        }
        builder
    }

    #[classmethod]
    fn ray(_cls: &Bound<'_, PyType>, a: u8, b: u8) -> PyResult<SquareSet> {
        if a >= 64 || b >= 64 {
            return Err(PyValueError::new_err("Square out of bounds"));
        }
        let sq_a = Square::new(a as u32);
        let sq_b = Square::new(b as u32);
        let ray = shakmaty::attacks::ray(sq_a, sq_b).0;
        Ok(SquareSet { mask: ray })
    }

    #[classmethod]
    fn between(_cls: &Bound<'_, PyType>, a: u8, b: u8) -> PyResult<SquareSet> {
        if a >= 64 || b >= 64 {
            return Err(PyValueError::new_err("Square out of bounds"));
        }
        let sq_a = Square::new(a as u32);
        let sq_b = Square::new(b as u32);
        let between = shakmaty::attacks::between(sq_a, sq_b).0;
        Ok(SquareSet { mask: between })
    }

    #[classmethod]
    fn from_square(_cls: &Bound<'_, PyType>, square: u8) -> PyResult<SquareSet> {
        if square >= 64 {
            return Err(PyValueError::new_err("Square out of bounds"));
        }
        Ok(SquareSet { mask: 1u64 << square })
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

#[pymodule]
fn rust_chess(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<SquareSet>()?;
    m.add_class::<SquareSetIter>()?;
    m.add_class::<SquareSetRevIter>()?;
    m.add_class::<CarryRipplerIter>()?;
    Ok(())
}
