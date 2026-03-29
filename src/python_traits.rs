use pyo3::prelude::*;
use pyo3::types::{PyDict, PyList};
use pyo3::exceptions::PyKeyError;

use crate::binary::{CArray, COptional, CString, LocalizableString};

// ── Traits ────────────────────────────────────────────────────────────────────

/// Convert a Rust value to a Python object.
pub trait ToPyValue {
    fn to_py_value(&self, py: Python<'_>) -> PyResult<Py<PyAny>>;
}

/// Read a Python value and write it as binary bytes.
pub trait WritePyValue {
    fn write_from_py(w: &mut Vec<u8>, obj: &Bound<'_, PyAny>) -> PyResult<()>;
}

// ── Dict helper (used by generated write_from_py_dict) ────────────────────────

pub fn get_field<'py>(d: &Bound<'py, PyDict>, key: &str) -> PyResult<Bound<'py, PyAny>> {
    d.get_item(key)?
        .ok_or_else(|| PyKeyError::new_err(key.to_string()))
}

// ── Primitives ────────────────────────────────────────────────────────────────

macro_rules! impl_primitive {
    ($t:ty) => {
        impl ToPyValue for $t {
            fn to_py_value(&self, py: Python<'_>) -> PyResult<Py<PyAny>> {
                Ok((*self).into_pyobject(py)?.into_any().unbind())
            }
        }

        impl WritePyValue for $t {
            fn write_from_py(w: &mut Vec<u8>, obj: &Bound<'_, PyAny>) -> PyResult<()> {
                let v: $t = obj.extract()?;
                w.extend_from_slice(&v.to_le_bytes());
                Ok(())
            }
        }
    };
}

impl_primitive!(u16);
impl_primitive!(u32);
impl_primitive!(u64);
impl_primitive!(i8);
impl_primitive!(i64);
impl_primitive!(f32);

// u8 is special: to_le_bytes returns [u8; 1], just push directly
impl ToPyValue for u8 {
    fn to_py_value(&self, py: Python<'_>) -> PyResult<Py<PyAny>> {
        Ok((*self).into_pyobject(py)?.into_any().unbind())
    }
}

impl WritePyValue for u8 {
    fn write_from_py(w: &mut Vec<u8>, obj: &Bound<'_, PyAny>) -> PyResult<()> {
        let v: u8 = obj.extract()?;
        w.push(v);
        Ok(())
    }
}

// ── Fixed-size arrays ─────────────────────────────────────────────────────────

impl ToPyValue for [f32; 3] {
    fn to_py_value(&self, py: Python<'_>) -> PyResult<Py<PyAny>> {
        Ok(self.to_vec().into_pyobject(py)?.into_any().unbind())
    }
}

impl WritePyValue for [f32; 3] {
    fn write_from_py(w: &mut Vec<u8>, obj: &Bound<'_, PyAny>) -> PyResult<()> {
        let list = obj.cast::<PyList>()?;
        for item in list.iter() {
            let v: f32 = item.extract()?;
            w.extend_from_slice(&v.to_le_bytes());
        }
        Ok(())
    }
}

impl ToPyValue for [u32; 4] {
    fn to_py_value(&self, py: Python<'_>) -> PyResult<Py<PyAny>> {
        Ok(self.to_vec().into_pyobject(py)?.into_any().unbind())
    }
}

impl WritePyValue for [u32; 4] {
    fn write_from_py(w: &mut Vec<u8>, obj: &Bound<'_, PyAny>) -> PyResult<()> {
        let list = obj.cast::<PyList>()?;
        for item in list.iter() {
            let v: u32 = item.extract()?;
            w.extend_from_slice(&v.to_le_bytes());
        }
        Ok(())
    }
}

// ── CString ───────────────────────────────────────────────────────────────────

impl ToPyValue for CString<'_> {
    fn to_py_value(&self, py: Python<'_>) -> PyResult<Py<PyAny>> {
        Ok(self.data.into_pyobject(py)?.into_any().unbind())
    }
}

impl WritePyValue for CString<'_> {
    fn write_from_py(w: &mut Vec<u8>, obj: &Bound<'_, PyAny>) -> PyResult<()> {
        let s: String = obj.extract()?;
        w.extend_from_slice(&(s.len() as u32).to_le_bytes());
        w.extend_from_slice(s.as_bytes());
        Ok(())
    }
}

// ── LocalizableString ─────────────────────────────────────────────────────────

impl ToPyValue for LocalizableString<'_> {
    fn to_py_value(&self, py: Python<'_>) -> PyResult<Py<PyAny>> {
        let d = PyDict::new(py);
        d.set_item("category", self.category)?;
        d.set_item("index", self.index)?;
        d.set_item("default", self.default.data)?;
        Ok(d.into_any().unbind())
    }
}

impl WritePyValue for LocalizableString<'_> {
    fn write_from_py(w: &mut Vec<u8>, obj: &Bound<'_, PyAny>) -> PyResult<()> {
        let d = obj.cast::<PyDict>()?;
        let category: u8 = get_field(d, "category")?.extract()?;
        let index: u64 = get_field(d, "index")?.extract()?;
        let default: String = get_field(d, "default")?.extract()?;
        w.push(category);
        w.extend_from_slice(&index.to_le_bytes());
        w.extend_from_slice(&(default.len() as u32).to_le_bytes());
        w.extend_from_slice(default.as_bytes());
        Ok(())
    }
}

// ── CArray ────────────────────────────────────────────────────────────────────

impl<T: ToPyValue> ToPyValue for CArray<T> {
    fn to_py_value(&self, py: Python<'_>) -> PyResult<Py<PyAny>> {
        let list = PyList::empty(py);
        for item in &self.items {
            list.append(item.to_py_value(py)?)?;
        }
        Ok(list.into_any().unbind())
    }
}

impl<T: WritePyValue> WritePyValue for CArray<T> {
    fn write_from_py(w: &mut Vec<u8>, obj: &Bound<'_, PyAny>) -> PyResult<()> {
        let list = obj.cast::<PyList>()?;
        w.extend_from_slice(&(list.len() as u32).to_le_bytes());
        for item in list.iter() {
            T::write_from_py(w, &item)?;
        }
        Ok(())
    }
}

// ── COptional ─────────────────────────────────────────────────────────────────

impl<T: ToPyValue> ToPyValue for COptional<T> {
    fn to_py_value(&self, py: Python<'_>) -> PyResult<Py<PyAny>> {
        match &self.value {
            Some(v) => v.to_py_value(py),
            None => Ok(py.None().into()),
        }
    }
}

impl<T: WritePyValue> WritePyValue for COptional<T> {
    fn write_from_py(w: &mut Vec<u8>, obj: &Bound<'_, PyAny>) -> PyResult<()> {
        if obj.is_none() {
            w.push(0);
        } else {
            w.push(1);
            T::write_from_py(w, obj)?;
        }
        Ok(())
    }
}
