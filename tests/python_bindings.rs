#![cfg(feature = "python")]

use pyo3::prelude::*;
use pyo3::types::{PyDict, PyModule};

#[test]
fn python_module_registers_and_basic_workflows_run() {
    Python::with_gil(|py| {
        let module = PyModule::new(py, "vectory").unwrap();
        vectory::python::register_python_module(&module).unwrap();

        let locals = PyDict::new(py);
        locals.set_item("vectory", &module).unwrap();

        py.run(
            pyo3::ffi::c_str!(
                r#"
fc = vectory.FeatureCollection(datum=(52.0, 5.0, 0.0), heading=(0.0, 0.0, 1.0))
fc.set_global_property("field_id", "F42")
fc.add_point_feature((1.0, 2.0, 3.0), {"name": "demo"})
assert fc.feature_count() == 1
assert fc.get_global_property("field_id", "") == "F42"
assert fc.feature(0)["geometry"]["kind"] == "point"

vec = vectory.Vector(
    field_boundary=[(0.0, 0.0, 0.0), (10.0, 0.0, 0.0), (10.0, 10.0, 0.0), (0.0, 10.0, 0.0)],
    datum=(52.0, 5.0, 0.0),
    heading=(0.0, 0.0, 0.0),
    crs="ENU",
)
vec.add_point((2.0, 3.0, 0.0), "marker", {"label": "A"})
assert vec.element_count() == 1
assert vec.points()[0]["kind"] == "marker"
json_text = fc.to_json("ENU")
assert "FeatureCollection" in json_text
"#
            ),
            None,
            Some(&locals),
        )
        .unwrap();
    });
}
