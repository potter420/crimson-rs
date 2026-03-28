mod binary;
mod item_info;
mod python;

use pyo3::prelude::*;

#[pymodule]
fn crimson_rs(m: &Bound<'_, PyModule>) -> PyResult<()> {
    python::register(m)
}
