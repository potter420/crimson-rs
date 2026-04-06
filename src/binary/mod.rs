mod primitives;
mod types;
mod arrays;
pub(crate) mod trie;
pub(crate) mod papgt;
pub(crate) mod pamt;
pub(crate) mod paz;
pub(crate) mod paloc;

pub use types::*;

use std::io::{self, Write};

// ── Traits ──────────────────────────────────────────────────────────────────

pub trait BinaryRead<'a>: Sized {
    fn read_from(data: &'a [u8], offset: &mut usize) -> io::Result<Self>;
}

pub trait BinaryWrite {
    fn write_to(&self, writer: &mut dyn Write) -> io::Result<()>;
}

// ── Helpers ─────────────────────────────────────────────────────────────────

pub(crate) fn check_remaining(data: &[u8], offset: usize, need: usize) -> io::Result<()> {
    if offset + need > data.len() {
        Err(io::Error::new(io::ErrorKind::UnexpectedEof, "not enough data"))
    } else {
        Ok(())
    }
}

// ── Macro for simple structs (binary only, no Python conversion) ────────────

#[macro_export]
macro_rules! binary_struct {
    (
        $(#[$meta:meta])*
        pub struct $name:ident $(<$lt:lifetime>)? {
            $(pub $field:ident : $ty:ty),* $(,)?
        }
    ) => {
        $(#[$meta])*
        #[derive(Debug)]
        pub struct $name $(<$lt>)? {
            $(pub $field: $ty),*
        }

        impl<'a> $crate::binary::BinaryRead<'a> for $name $(<$lt>)? {
            fn read_from(data: &'a [u8], offset: &mut usize) -> std::io::Result<Self> {
                Ok($name {
                    $($field: $crate::binary::BinaryRead::read_from(data, offset)?),*
                })
            }
        }

        impl $(< $lt >)? $crate::binary::BinaryWrite for $name $(< $lt >)? {
            fn write_to(&self, w: &mut dyn std::io::Write) -> std::io::Result<()> {
                $($crate::binary::BinaryWrite::write_to(&self.$field, w)?;)*
                Ok(())
            }
        }
    };
}

// ── Macro for structs with binary + Python conversion ───────────────────────

#[macro_export]
macro_rules! py_binary_struct {
    (
        $(#[$meta:meta])*
        pub struct $name:ident $(<$lt:lifetime>)? {
            $(pub $field:ident : $ty:ty),* $(,)?
        }
    ) => {
        $(#[$meta])*
        #[derive(Debug)]
        pub struct $name $(<$lt>)? {
            $(pub $field: $ty),*
        }

        impl<'a> $crate::binary::BinaryRead<'a> for $name $(<$lt>)? {
            fn read_from(data: &'a [u8], offset: &mut usize) -> std::io::Result<Self> {
                Ok($name {
                    $($field: $crate::binary::BinaryRead::read_from(data, offset)?),*
                })
            }
        }

        impl $(< $lt >)? $crate::binary::BinaryWrite for $name $(< $lt >)? {
            fn write_to(&self, w: &mut dyn std::io::Write) -> std::io::Result<()> {
                $($crate::binary::BinaryWrite::write_to(&self.$field, w)?;)*
                Ok(())
            }
        }

        impl $(< $lt >)? $name $(< $lt >)? {
            pub fn to_py_dict<'py>(&self, py: pyo3::Python<'py>)
                -> pyo3::PyResult<pyo3::Bound<'py, pyo3::types::PyDict>>
            {
                use $crate::python_traits::ToPyValue;
                use pyo3::types::PyDictMethods;
                let d = pyo3::types::PyDict::new(py);
                $(d.set_item(stringify!($field), self.$field.to_py_value(py)?)?;)*
                Ok(d)
            }

            pub fn write_from_py_dict(
                w: &mut Vec<u8>,
                d: &pyo3::Bound<'_, pyo3::types::PyDict>,
            ) -> pyo3::PyResult<()> {
                use $crate::python_traits::{WritePyValue, get_field};
                $(<$ty as WritePyValue>::write_from_py(w, &get_field(d, stringify!($field))?)?;)*
                Ok(())
            }
        }

        impl $(< $lt >)? $crate::python_traits::ToPyValue for $name $(< $lt >)? {
            fn to_py_value(&self, py: pyo3::Python<'_>) -> pyo3::PyResult<pyo3::Py<pyo3::PyAny>> {
                Ok(self.to_py_dict(py)?.into_any().unbind())
            }
        }

        impl $(< $lt >)? $crate::python_traits::WritePyValue for $name $(< $lt >)? {
            fn write_from_py(w: &mut Vec<u8>, obj: &pyo3::Bound<'_, pyo3::PyAny>) -> pyo3::PyResult<()> {
                Self::write_from_py_dict(w, obj.cast::<pyo3::types::PyDict>()?)
            }
        }
    };
}
