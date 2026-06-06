use std::collections::HashMap;

use datapod::{Point, Segment};
use pyo3::exceptions::{PyIndexError, PyRuntimeError, PyValueError};
use pyo3::prelude::*;
use pyo3::types::{PyDict, PyModule};
use pyo3::wrap_pyfunction;

use crate::{
    Crs, Feature, FeatureCollection, Geometry, Heading, Vector, read, read_json_str,
    to_json_string, write,
};

fn py_runtime_error(err: crate::Error) -> PyErr {
    PyRuntimeError::new_err(err.to_string())
}

fn parse_crs(value: &str) -> PyResult<Crs> {
    match value {
        "WGS" | "WGS84" | "EPSG:4326" => Ok(Crs::Wgs),
        "ENU" => Ok(Crs::Enu),
        _ => Err(PyValueError::new_err(format!("unknown CRS: {value}"))),
    }
}

fn geo_tuple(value: datapod::Geo) -> (f64, f64, f64) {
    (value.latitude, value.longitude, value.altitude)
}

fn heading_tuple(value: Heading) -> (f64, f64, f64) {
    (value.roll, value.pitch, value.yaw)
}

fn point_tuple(value: Point) -> (f64, f64, f64) {
    (value.x, value.y, value.z)
}

fn dict_from_properties<'py>(
    py: Python<'py>,
    properties: &HashMap<String, String>,
) -> PyResult<Bound<'py, PyDict>> {
    let dict = PyDict::new(py);
    for (key, value) in properties {
        dict.set_item(key, value)?;
    }
    Ok(dict)
}

fn geometry_dict<'py>(py: Python<'py>, geometry: &Geometry) -> PyResult<Bound<'py, PyDict>> {
    let dict = PyDict::new(py);
    match geometry {
        Geometry::Point(point) => {
            dict.set_item("kind", "point")?;
            dict.set_item("point", point_tuple(*point))?;
        }
        Geometry::Segment(segment) => {
            dict.set_item("kind", "segment")?;
            dict.set_item("start", point_tuple(segment.start))?;
            dict.set_item("end", point_tuple(segment.end))?;
        }
        Geometry::Path(path) => {
            dict.set_item("kind", "path")?;
            let points = path
                .points
                .iter()
                .copied()
                .map(point_tuple)
                .collect::<Vec<_>>();
            dict.set_item("points", points)?;
        }
        Geometry::Polygon(polygon) => {
            dict.set_item("kind", "polygon")?;
            let vertices = polygon
                .vertices
                .iter()
                .copied()
                .map(point_tuple)
                .collect::<Vec<_>>();
            dict.set_item("vertices", vertices)?;
        }
    }
    Ok(dict)
}

fn feature_dict<'py>(py: Python<'py>, feature: &Feature) -> PyResult<Bound<'py, PyDict>> {
    let dict = PyDict::new(py);
    dict.set_item("geometry", geometry_dict(py, &feature.geometry)?)?;
    dict.set_item("properties", dict_from_properties(py, &feature.properties)?)?;
    Ok(dict)
}

fn parse_point(value: (f64, f64, f64)) -> Point {
    Point::new(value.0, value.1, value.2)
}

fn parse_points(values: Vec<(f64, f64, f64)>) -> Vec<Point> {
    values.into_iter().map(parse_point).collect()
}

#[pyfunction]
fn read_json(py: Python<'_>, text: &str) -> PyResult<Py<PyFeatureCollection>> {
    let inner = read_json_str(text).map_err(py_runtime_error)?;
    Py::new(py, PyFeatureCollection { inner })
}

#[pyfunction]
fn read_file(py: Python<'_>, path: &str) -> PyResult<Py<PyFeatureCollection>> {
    let inner = read(path).map_err(py_runtime_error)?;
    Py::new(py, PyFeatureCollection { inner })
}

#[pyclass(name = "FeatureCollection")]
pub struct PyFeatureCollection {
    inner: FeatureCollection,
}

#[pymethods]
impl PyFeatureCollection {
    #[new]
    #[pyo3(signature = (datum=(0.0, 0.0, 0.0), heading=(0.0, 0.0, 0.0)))]
    fn new(datum: (f64, f64, f64), heading: (f64, f64, f64)) -> Self {
        Self {
            inner: FeatureCollection::new(
                datapod::Geo::new(datum.0, datum.1, datum.2),
                Heading::new(heading.0, heading.1, heading.2),
            ),
        }
    }

    #[staticmethod]
    fn from_file(path: &str) -> PyResult<Self> {
        Ok(Self {
            inner: read(path).map_err(py_runtime_error)?,
        })
    }

    #[staticmethod]
    fn from_json(text: &str) -> PyResult<Self> {
        Ok(Self {
            inner: read_json_str(text).map_err(py_runtime_error)?,
        })
    }

    #[pyo3(signature = (crs="WGS"))]
    fn to_json(&self, crs: &str) -> PyResult<String> {
        to_json_string(&self.inner, parse_crs(crs)?).map_err(py_runtime_error)
    }

    #[pyo3(signature = (path, crs="WGS"))]
    fn write(&self, path: &str, crs: &str) -> PyResult<()> {
        write(&self.inner, path, parse_crs(crs)?).map_err(py_runtime_error)
    }

    fn feature_count(&self) -> usize {
        self.inner.features.len()
    }

    fn clear_features(&mut self) {
        self.inner.features.clear();
    }

    fn datum(&self) -> (f64, f64, f64) {
        geo_tuple(self.inner.datum)
    }

    fn set_datum(&mut self, datum: (f64, f64, f64)) {
        self.inner.datum = datapod::Geo::new(datum.0, datum.1, datum.2);
    }

    fn heading(&self) -> (f64, f64, f64) {
        heading_tuple(self.inner.heading)
    }

    fn set_heading(&mut self, heading: (f64, f64, f64)) {
        self.inner.heading = Heading::new(heading.0, heading.1, heading.2);
    }

    fn set_global_property(&mut self, key: &str, value: &str) {
        self.inner
            .global_properties
            .insert(key.to_string(), value.to_string());
    }

    #[pyo3(signature = (key, default_value=""))]
    fn get_global_property(&self, key: &str, default_value: &str) -> String {
        self.inner
            .global_properties
            .get(key)
            .cloned()
            .unwrap_or_else(|| default_value.to_string())
    }

    fn global_properties<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyDict>> {
        dict_from_properties(py, &self.inner.global_properties)
    }

    #[pyo3(signature = (point, properties=None))]
    fn add_point_feature(
        &mut self,
        point: (f64, f64, f64),
        properties: Option<HashMap<String, String>>,
    ) {
        self.inner.features.push(Feature::new(
            parse_point(point),
            properties.unwrap_or_default(),
        ));
    }

    #[pyo3(signature = (start, end, properties=None))]
    fn add_segment_feature(
        &mut self,
        start: (f64, f64, f64),
        end: (f64, f64, f64),
        properties: Option<HashMap<String, String>>,
    ) {
        self.inner.features.push(Feature::new(
            Segment::new(parse_point(start), parse_point(end)),
            properties.unwrap_or_default(),
        ));
    }

    #[pyo3(signature = (points, properties=None))]
    fn add_path_feature(
        &mut self,
        points: Vec<(f64, f64, f64)>,
        properties: Option<HashMap<String, String>>,
    ) {
        self.inner.features.push(Feature::new(
            Geometry::path(parse_points(points)),
            properties.unwrap_or_default(),
        ));
    }

    #[pyo3(signature = (points, properties=None))]
    fn add_polygon_feature(
        &mut self,
        points: Vec<(f64, f64, f64)>,
        properties: Option<HashMap<String, String>>,
    ) {
        self.inner.features.push(Feature::new(
            Geometry::polygon(parse_points(points)),
            properties.unwrap_or_default(),
        ));
    }

    fn feature<'py>(&self, py: Python<'py>, index: usize) -> PyResult<Bound<'py, PyDict>> {
        let feature = self
            .inner
            .features
            .get(index)
            .ok_or_else(|| PyIndexError::new_err("feature index out of range"))?;
        feature_dict(py, feature)
    }

    fn features<'py>(&self, py: Python<'py>) -> PyResult<Vec<Py<PyAny>>> {
        self.inner
            .features
            .iter()
            .map(|feature| Ok(feature_dict(py, feature)?.into_any().unbind()))
            .collect()
    }
}

#[pyclass(name = "Vector")]
pub struct PyVector {
    inner: Vector,
}

#[pymethods]
impl PyVector {
    #[new]
    #[pyo3(signature = (field_boundary, datum=(0.001, 0.001, 1.0), heading=(0.0, 0.0, 0.0), crs="ENU"))]
    fn new(
        field_boundary: Vec<(f64, f64, f64)>,
        datum: (f64, f64, f64),
        heading: (f64, f64, f64),
        crs: &str,
    ) -> PyResult<Self> {
        Ok(Self {
            inner: Vector::new(
                parse_points(field_boundary),
                datapod::Geo::new(datum.0, datum.1, datum.2),
                Heading::new(heading.0, heading.1, heading.2),
                parse_crs(crs)?,
            ),
        })
    }

    #[staticmethod]
    fn from_file(path: &str) -> PyResult<Self> {
        Ok(Self {
            inner: Vector::from_file(path).map_err(py_runtime_error)?,
        })
    }

    #[pyo3(signature = (path, crs="WGS"))]
    fn to_file(&self, path: &str, crs: &str) -> PyResult<()> {
        self.inner
            .to_file(path, parse_crs(crs)?)
            .map_err(py_runtime_error)
    }

    fn field_boundary(&self) -> Vec<(f64, f64, f64)> {
        self.inner
            .field_boundary()
            .iter()
            .copied()
            .map(point_tuple)
            .collect()
    }

    fn field_properties<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyDict>> {
        dict_from_properties(py, self.inner.field_properties())
    }

    fn set_field_property(&mut self, key: &str, value: &str) {
        self.inner.set_field_property(key, value);
    }

    fn remove_field_property(&mut self, key: &str) {
        self.inner.remove_field_property(key);
    }

    fn element_count(&self) -> usize {
        self.inner.element_count()
    }

    fn has_elements(&self) -> bool {
        self.inner.has_elements()
    }

    fn clear_elements(&mut self) {
        self.inner.clear_elements();
    }

    fn datum(&self) -> (f64, f64, f64) {
        geo_tuple(self.inner.datum())
    }

    fn set_datum(&mut self, datum: (f64, f64, f64)) {
        self.inner
            .set_datum(datapod::Geo::new(datum.0, datum.1, datum.2));
    }

    fn heading(&self) -> (f64, f64, f64) {
        heading_tuple(self.inner.heading())
    }

    fn set_heading(&mut self, heading: (f64, f64, f64)) {
        self.inner
            .set_heading(Heading::new(heading.0, heading.1, heading.2));
    }

    fn crs(&self) -> &'static str {
        match self.inner.crs() {
            Crs::Wgs => "WGS",
            Crs::Enu => "ENU",
        }
    }

    fn set_crs(&mut self, crs: &str) -> PyResult<()> {
        self.inner.set_crs(parse_crs(crs)?);
        Ok(())
    }

    fn set_global_property(&mut self, key: &str, value: &str) {
        self.inner.set_global_property(key, value);
    }

    #[pyo3(signature = (key, default_value=""))]
    fn get_global_property(&self, key: &str, default_value: &str) -> String {
        self.inner.getGlobalProperty(key, default_value)
    }

    fn global_properties<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyDict>> {
        dict_from_properties(py, self.inner.global_properties())
    }

    #[pyo3(signature = (point, kind=None, properties=None))]
    fn add_point(
        &mut self,
        point: (f64, f64, f64),
        kind: Option<&str>,
        properties: Option<HashMap<String, String>>,
    ) {
        self.inner.add_point(
            parse_point(point),
            kind.unwrap_or("point"),
            properties.unwrap_or_default(),
        );
    }

    #[pyo3(signature = (start, end, kind=None, properties=None))]
    fn add_line(
        &mut self,
        start: (f64, f64, f64),
        end: (f64, f64, f64),
        kind: Option<&str>,
        properties: Option<HashMap<String, String>>,
    ) {
        self.inner.add_line(
            Segment::new(parse_point(start), parse_point(end)),
            kind.unwrap_or("line"),
            properties.unwrap_or_default(),
        );
    }

    #[pyo3(signature = (points, kind=None, properties=None))]
    fn add_path(
        &mut self,
        points: Vec<(f64, f64, f64)>,
        kind: Option<&str>,
        properties: Option<HashMap<String, String>>,
    ) {
        self.inner.add_path(
            parse_points(points),
            kind.unwrap_or("path"),
            properties.unwrap_or_default(),
        );
    }

    #[pyo3(signature = (points, kind=None, properties=None))]
    fn add_polygon(
        &mut self,
        points: Vec<(f64, f64, f64)>,
        kind: Option<&str>,
        properties: Option<HashMap<String, String>>,
    ) {
        self.inner.add_polygon(
            parse_points(points),
            kind.unwrap_or("polygon"),
            properties.unwrap_or_default(),
        );
    }

    fn element<'py>(&self, py: Python<'py>, index: usize) -> PyResult<Bound<'py, PyDict>> {
        let element = self
            .inner
            .element(index)
            .ok_or_else(|| PyIndexError::new_err("element index out of range"))?;
        let dict = PyDict::new(py);
        dict.set_item("kind", &element.kind)?;
        dict.set_item("geometry", geometry_dict(py, &element.geometry)?)?;
        dict.set_item("properties", dict_from_properties(py, &element.properties)?)?;
        Ok(dict)
    }

    fn elements<'py>(&self, py: Python<'py>) -> PyResult<Vec<Py<PyAny>>> {
        self.inner
            .iter()
            .map(|element| {
                let dict = PyDict::new(py);
                dict.set_item("kind", &element.kind)?;
                dict.set_item("geometry", geometry_dict(py, &element.geometry)?)?;
                dict.set_item("properties", dict_from_properties(py, &element.properties)?)?;
                Ok(dict.into_any().unbind())
            })
            .collect()
    }

    fn points<'py>(&self, py: Python<'py>) -> PyResult<Vec<Py<PyAny>>> {
        self.inner
            .points()
            .into_iter()
            .map(|element| {
                let dict = PyDict::new(py);
                dict.set_item("kind", &element.kind)?;
                dict.set_item("geometry", geometry_dict(py, &element.geometry)?)?;
                dict.set_item("properties", dict_from_properties(py, &element.properties)?)?;
                Ok(dict.into_any().unbind())
            })
            .collect()
    }
}

pub fn register_python_module(module: &Bound<'_, PyModule>) -> PyResult<()> {
    module.add_function(wrap_pyfunction!(read_json, module)?)?;
    module.add_function(wrap_pyfunction!(read_file, module)?)?;
    module.add_class::<PyFeatureCollection>()?;
    module.add_class::<PyVector>()?;
    Ok(())
}

#[pymodule]
fn vectory(module: &Bound<'_, PyModule>) -> PyResult<()> {
    register_python_module(module)
}
