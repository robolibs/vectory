use std::cell::RefCell;
use std::collections::HashMap;
use std::ffi::{CStr, CString, c_char};
use std::ptr;

use datapod::{Euler, Point, Segment};

use crate::{Crs, Feature, FeatureCollection, Geometry, Vector};

thread_local! {
    static LAST_ERROR: RefCell<Option<CString>> = const { RefCell::new(None) };
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct VectoryGeo3 {
    pub latitude: f64,
    pub longitude: f64,
    pub altitude: f64,
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct VectoryEuler {
    pub roll: f64,
    pub pitch: f64,
    pub yaw: f64,
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct VectoryPoint3 {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct VectorySegment3 {
    pub start: VectoryPoint3,
    pub end: VectoryPoint3,
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct VectoryPointArrayView {
    pub ptr: *const VectoryPoint3,
    pub len: usize,
}

#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VectoryCrs {
    Wgs = 0,
    Enu = 1,
}

#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VectoryGeometryKind {
    None = 0,
    Point = 1,
    Segment = 2,
    Path = 3,
    Polygon = 4,
}

pub struct VectoryFeatureCollectionHandle {
    inner: FeatureCollection,
}

pub struct VectoryVectorHandle {
    inner: Vector,
}

impl From<VectoryGeo3> for datapod::Geo {
    fn from(value: VectoryGeo3) -> Self {
        Self::new(value.latitude, value.longitude, value.altitude)
    }
}

impl From<datapod::Geo> for VectoryGeo3 {
    fn from(value: datapod::Geo) -> Self {
        Self {
            latitude: value.latitude,
            longitude: value.longitude,
            altitude: value.altitude,
        }
    }
}

impl From<VectoryEuler> for Euler {
    fn from(value: VectoryEuler) -> Self {
        Self::new(value.roll, value.pitch, value.yaw)
    }
}

impl From<Euler> for VectoryEuler {
    fn from(value: Euler) -> Self {
        Self {
            roll: value.roll,
            pitch: value.pitch,
            yaw: value.yaw,
        }
    }
}

impl From<VectoryPoint3> for Point {
    fn from(value: VectoryPoint3) -> Self {
        Self::new(value.x, value.y, value.z)
    }
}

impl From<Point> for VectoryPoint3 {
    fn from(value: Point) -> Self {
        Self {
            x: value.x,
            y: value.y,
            z: value.z,
        }
    }
}

impl From<VectorySegment3> for Segment {
    fn from(value: VectorySegment3) -> Self {
        Self::new(value.start.into(), value.end.into())
    }
}

impl From<Segment> for VectorySegment3 {
    fn from(value: Segment) -> Self {
        Self {
            start: value.start.into(),
            end: value.end.into(),
        }
    }
}

fn clear_last_error() {
    LAST_ERROR.with(|slot| {
        *slot.borrow_mut() = None;
    });
}

fn set_last_error(message: impl Into<String>) {
    let message = message.into().replace('\0', " ");
    LAST_ERROR.with(|slot| {
        *slot.borrow_mut() = Some(
            CString::new(message).unwrap_or_else(|_| CString::new("vectory ffi error").unwrap()),
        );
    });
}

fn ok() -> bool {
    clear_last_error();
    true
}

fn fail(message: impl Into<String>) -> bool {
    set_last_error(message);
    false
}

fn cstr_to_str<'a>(value: *const c_char, label: &str) -> crate::Result<&'a str> {
    if value.is_null() {
        return Err(crate::Error::InvalidGeoJson(format!("null {label} pointer")));
    }
    let cstr = unsafe { CStr::from_ptr(value) };
    cstr.to_str()
        .map_err(|_| crate::Error::InvalidGeoJson(format!("{label} must be valid UTF-8")))
}

fn string_to_ptr(value: String) -> *mut c_char {
    CString::new(value)
        .unwrap_or_else(|_| CString::new("").unwrap())
        .into_raw()
}

fn crs_from_c(value: VectoryCrs) -> Crs {
    match value {
        VectoryCrs::Wgs => Crs::Wgs,
        VectoryCrs::Enu => Crs::Enu,
    }
}

fn geometry_kind(geometry: &Geometry) -> VectoryGeometryKind {
    match geometry {
        Geometry::Point(_) => VectoryGeometryKind::Point,
        Geometry::Segment(_) => VectoryGeometryKind::Segment,
        Geometry::Path(_) => VectoryGeometryKind::Path,
        Geometry::Polygon(_) => VectoryGeometryKind::Polygon,
    }
}

fn read_points(view: VectoryPointArrayView) -> crate::Result<Vec<Point>> {
    if view.ptr.is_null() && view.len != 0 {
        return Err(crate::Error::InvalidGeoJson("null points pointer".into()));
    }
    let slice = unsafe { std::slice::from_raw_parts(view.ptr, view.len) };
    Ok(slice.iter().copied().map(Point::from).collect())
}

fn fc_from_ptr_mut<'a>(
    handle: *mut VectoryFeatureCollectionHandle,
) -> crate::Result<&'a mut VectoryFeatureCollectionHandle> {
    if handle.is_null() {
        return Err(crate::Error::InvalidGeoJson(
            "null feature collection handle".into(),
        ));
    }
    Ok(unsafe { &mut *handle })
}

fn fc_from_ptr<'a>(
    handle: *const VectoryFeatureCollectionHandle,
) -> crate::Result<&'a VectoryFeatureCollectionHandle> {
    if handle.is_null() {
        return Err(crate::Error::InvalidGeoJson(
            "null feature collection handle".into(),
        ));
    }
    Ok(unsafe { &*handle })
}

fn vector_from_ptr_mut<'a>(
    handle: *mut VectoryVectorHandle,
) -> crate::Result<&'a mut VectoryVectorHandle> {
    if handle.is_null() {
        return Err(crate::Error::InvalidGeoJson("null vector handle".into()));
    }
    Ok(unsafe { &mut *handle })
}

fn vector_from_ptr<'a>(handle: *const VectoryVectorHandle) -> crate::Result<&'a VectoryVectorHandle> {
    if handle.is_null() {
        return Err(crate::Error::InvalidGeoJson("null vector handle".into()));
    }
    Ok(unsafe { &*handle })
}

fn write_out<T>(out: *mut T, value: T) -> bool {
    if out.is_null() {
        return fail("null output pointer");
    }
    unsafe { ptr::write(out, value) };
    ok()
}

#[unsafe(no_mangle)]
pub extern "C" fn vectory_last_error_message() -> *const c_char {
    LAST_ERROR.with(|slot| {
        slot.borrow()
            .as_ref()
            .map_or(ptr::null(), |message| message.as_ptr())
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn vectory_string_free(ptr: *mut c_char) {
    if !ptr.is_null() {
        unsafe {
            drop(CString::from_raw(ptr));
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn vectory_feature_collection_new(
    datum: VectoryGeo3,
    heading: VectoryEuler,
) -> *mut VectoryFeatureCollectionHandle {
    clear_last_error();
    Box::into_raw(Box::new(VectoryFeatureCollectionHandle {
        inner: FeatureCollection::new(datum.into(), heading.into()),
    }))
}

#[unsafe(no_mangle)]
pub extern "C" fn vectory_feature_collection_free(handle: *mut VectoryFeatureCollectionHandle) {
    if !handle.is_null() {
        unsafe {
            drop(Box::from_raw(handle));
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn vectory_feature_collection_read(
    path: *const c_char,
) -> *mut VectoryFeatureCollectionHandle {
    clear_last_error();
    match cstr_to_str(path, "path").and_then(crate::read) {
        Ok(inner) => Box::into_raw(Box::new(VectoryFeatureCollectionHandle { inner })),
        Err(err) => {
            set_last_error(err.to_string());
            ptr::null_mut()
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn vectory_feature_collection_from_json(
    text: *const c_char,
) -> *mut VectoryFeatureCollectionHandle {
    clear_last_error();
    match cstr_to_str(text, "json").and_then(crate::read_json_str) {
        Ok(inner) => Box::into_raw(Box::new(VectoryFeatureCollectionHandle { inner })),
        Err(err) => {
            set_last_error(err.to_string());
            ptr::null_mut()
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn vectory_feature_collection_write(
    handle: *const VectoryFeatureCollectionHandle,
    path: *const c_char,
    crs: VectoryCrs,
) -> bool {
    match fc_from_ptr(handle)
        .and_then(|handle| cstr_to_str(path, "path").and_then(|path| crate::write(&handle.inner, path, crs_from_c(crs))))
    {
        Ok(()) => ok(),
        Err(err) => fail(err.to_string()),
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn vectory_feature_collection_to_json(
    handle: *const VectoryFeatureCollectionHandle,
    crs: VectoryCrs,
) -> *mut c_char {
    clear_last_error();
    match fc_from_ptr(handle).and_then(|handle| crate::to_json_string(&handle.inner, crs_from_c(crs))) {
        Ok(json) => string_to_ptr(json),
        Err(err) => {
            set_last_error(err.to_string());
            ptr::null_mut()
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn vectory_feature_collection_feature_count(
    handle: *const VectoryFeatureCollectionHandle,
) -> usize {
    fc_from_ptr(handle)
        .map(|handle| handle.inner.features.len())
        .unwrap_or(0)
}

#[unsafe(no_mangle)]
pub extern "C" fn vectory_feature_collection_clear_features(
    handle: *mut VectoryFeatureCollectionHandle,
) -> bool {
    match fc_from_ptr_mut(handle) {
        Ok(handle) => {
            handle.inner.features.clear();
            ok()
        }
        Err(err) => fail(err.to_string()),
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn vectory_feature_collection_get_datum(
    handle: *const VectoryFeatureCollectionHandle,
    out: *mut VectoryGeo3,
) -> bool {
    match fc_from_ptr(handle) {
        Ok(handle) => write_out(out, handle.inner.datum.into()),
        Err(err) => fail(err.to_string()),
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn vectory_feature_collection_set_datum(
    handle: *mut VectoryFeatureCollectionHandle,
    datum: VectoryGeo3,
) -> bool {
    match fc_from_ptr_mut(handle) {
        Ok(handle) => {
            handle.inner.datum = datum.into();
            ok()
        }
        Err(err) => fail(err.to_string()),
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn vectory_feature_collection_get_heading(
    handle: *const VectoryFeatureCollectionHandle,
    out: *mut VectoryEuler,
) -> bool {
    match fc_from_ptr(handle) {
        Ok(handle) => write_out(out, handle.inner.heading.into()),
        Err(err) => fail(err.to_string()),
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn vectory_feature_collection_set_heading(
    handle: *mut VectoryFeatureCollectionHandle,
    heading: VectoryEuler,
) -> bool {
    match fc_from_ptr_mut(handle) {
        Ok(handle) => {
            handle.inner.heading = heading.into();
            ok()
        }
        Err(err) => fail(err.to_string()),
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn vectory_feature_collection_set_global_property(
    handle: *mut VectoryFeatureCollectionHandle,
    key: *const c_char,
    value: *const c_char,
) -> bool {
    match fc_from_ptr_mut(handle).and_then(|handle| {
        Ok((
            handle,
            cstr_to_str(key, "key")?.to_string(),
            cstr_to_str(value, "value")?.to_string(),
        ))
    }) {
        Ok((handle, key, value)) => {
            handle.inner.global_properties.insert(key, value);
            ok()
        }
        Err(err) => fail(err.to_string()),
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn vectory_feature_collection_get_global_property(
    handle: *const VectoryFeatureCollectionHandle,
    key: *const c_char,
) -> *mut c_char {
    clear_last_error();
    match fc_from_ptr(handle).and_then(|handle| {
        let key = cstr_to_str(key, "key")?;
        Ok(handle.inner.global_properties.get(key).cloned())
    }) {
        Ok(Some(value)) => string_to_ptr(value),
        Ok(None) => ptr::null_mut(),
        Err(err) => {
            set_last_error(err.to_string());
            ptr::null_mut()
        }
    }
}

fn feature_properties_from_c(
    kind: Option<&str>,
    property_key: *const c_char,
    property_value: *const c_char,
) -> crate::Result<HashMap<String, String>> {
    let mut props = HashMap::new();
    if let Some(kind) = kind {
        props.insert("type".into(), kind.into());
    }
    if !property_key.is_null() {
        props.insert(
            cstr_to_str(property_key, "property key")?.into(),
            cstr_to_str(property_value, "property value")?.into(),
        );
    }
    Ok(props)
}

#[unsafe(no_mangle)]
pub extern "C" fn vectory_feature_collection_add_point_feature(
    handle: *mut VectoryFeatureCollectionHandle,
    point: VectoryPoint3,
    property_key: *const c_char,
    property_value: *const c_char,
) -> bool {
    match fc_from_ptr_mut(handle).and_then(|handle| {
        let properties = feature_properties_from_c(None, property_key, property_value)?;
        handle
            .inner
            .features
            .push(Feature::new(Point::from(point), properties));
        Ok(())
    }) {
        Ok(()) => ok(),
        Err(err) => fail(err.to_string()),
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn vectory_feature_collection_add_segment_feature(
    handle: *mut VectoryFeatureCollectionHandle,
    segment: VectorySegment3,
    property_key: *const c_char,
    property_value: *const c_char,
) -> bool {
    match fc_from_ptr_mut(handle).and_then(|handle| {
        let properties = feature_properties_from_c(None, property_key, property_value)?;
        handle
            .inner
            .features
            .push(Feature::new(Segment::from(segment), properties));
        Ok(())
    }) {
        Ok(()) => ok(),
        Err(err) => fail(err.to_string()),
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn vectory_feature_collection_add_path_feature(
    handle: *mut VectoryFeatureCollectionHandle,
    points: VectoryPointArrayView,
    property_key: *const c_char,
    property_value: *const c_char,
) -> bool {
    match fc_from_ptr_mut(handle).and_then(|handle| {
        let points = read_points(points)?;
        let properties = feature_properties_from_c(None, property_key, property_value)?;
        handle.inner.features.push(Feature::new(
            crate::Geometry::path(points),
            properties,
        ));
        Ok(())
    }) {
        Ok(()) => ok(),
        Err(err) => fail(err.to_string()),
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn vectory_feature_collection_add_polygon_feature(
    handle: *mut VectoryFeatureCollectionHandle,
    points: VectoryPointArrayView,
    property_key: *const c_char,
    property_value: *const c_char,
) -> bool {
    match fc_from_ptr_mut(handle).and_then(|handle| {
        let points = read_points(points)?;
        let properties = feature_properties_from_c(None, property_key, property_value)?;
        handle.inner.features.push(Feature::new(
            crate::Geometry::polygon(points),
            properties,
        ));
        Ok(())
    }) {
        Ok(()) => ok(),
        Err(err) => fail(err.to_string()),
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn vectory_feature_collection_feature_geometry_kind(
    handle: *const VectoryFeatureCollectionHandle,
    index: usize,
) -> VectoryGeometryKind {
    match fc_from_ptr(handle)
        .ok()
        .and_then(|handle| handle.inner.features.get(index))
        .map(|feature| geometry_kind(&feature.geometry))
    {
        Some(kind) => kind,
        None => VectoryGeometryKind::None,
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn vectory_feature_collection_feature_property(
    handle: *const VectoryFeatureCollectionHandle,
    index: usize,
    key: *const c_char,
) -> *mut c_char {
    clear_last_error();
    match fc_from_ptr(handle).and_then(|handle| {
        let key = cstr_to_str(key, "key")?;
        let feature = handle
            .inner
            .features
            .get(index)
            .ok_or_else(|| crate::Error::InvalidGeoJson("feature index out of range".into()))?;
        Ok(feature.properties.get(key).cloned())
    }) {
        Ok(Some(value)) => string_to_ptr(value),
        Ok(None) => ptr::null_mut(),
        Err(err) => {
            set_last_error(err.to_string());
            ptr::null_mut()
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn vectory_feature_collection_feature_point(
    handle: *const VectoryFeatureCollectionHandle,
    index: usize,
    out: *mut VectoryPoint3,
) -> bool {
    match fc_from_ptr(handle).and_then(|handle| {
        let feature = handle
            .inner
            .features
            .get(index)
            .ok_or_else(|| crate::Error::InvalidGeoJson("feature index out of range".into()))?;
        if let Geometry::Point(point) = feature.geometry {
            Ok(point)
        } else {
            Err(crate::Error::InvalidGeoJson("feature is not a point".into()))
        }
    }) {
        Ok(point) => write_out(out, point.into()),
        Err(err) => fail(err.to_string()),
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn vectory_feature_collection_feature_segment(
    handle: *const VectoryFeatureCollectionHandle,
    index: usize,
    out: *mut VectorySegment3,
) -> bool {
    match fc_from_ptr(handle).and_then(|handle| {
        let feature = handle
            .inner
            .features
            .get(index)
            .ok_or_else(|| crate::Error::InvalidGeoJson("feature index out of range".into()))?;
        if let Geometry::Segment(segment) = feature.geometry {
            Ok(segment)
        } else {
            Err(crate::Error::InvalidGeoJson("feature is not a segment".into()))
        }
    }) {
        Ok(segment) => write_out(out, segment.into()),
        Err(err) => fail(err.to_string()),
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn vectory_feature_collection_feature_vertex_count(
    handle: *const VectoryFeatureCollectionHandle,
    index: usize,
) -> usize {
    fc_from_ptr(handle)
        .ok()
        .and_then(|handle| handle.inner.features.get(index))
        .map(|feature| match &feature.geometry {
            Geometry::Path(path) => path.points.len(),
            Geometry::Polygon(poly) => poly.vertices.len(),
            _ => 0,
        })
        .unwrap_or(0)
}

#[unsafe(no_mangle)]
pub extern "C" fn vectory_feature_collection_feature_vertex_at(
    handle: *const VectoryFeatureCollectionHandle,
    index: usize,
    vertex_index: usize,
    out: *mut VectoryPoint3,
) -> bool {
    match fc_from_ptr(handle).and_then(|handle| {
        let feature = handle
            .inner
            .features
            .get(index)
            .ok_or_else(|| crate::Error::InvalidGeoJson("feature index out of range".into()))?;
        match &feature.geometry {
            Geometry::Path(path) => path
                .points
                .get(vertex_index)
                .copied()
                .ok_or_else(|| crate::Error::InvalidGeoJson("vertex index out of range".into())),
            Geometry::Polygon(poly) => poly
                .vertices
                .get(vertex_index)
                .copied()
                .ok_or_else(|| crate::Error::InvalidGeoJson("vertex index out of range".into())),
            _ => Err(crate::Error::InvalidGeoJson(
                "feature does not expose vertices".into(),
            )),
        }
    }) {
        Ok(point) => write_out(out, point.into()),
        Err(err) => fail(err.to_string()),
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn vectory_vector_new(
    boundary: VectoryPointArrayView,
    datum: VectoryGeo3,
    heading: VectoryEuler,
    crs: VectoryCrs,
) -> *mut VectoryVectorHandle {
    clear_last_error();
    match read_points(boundary) {
        Ok(points) => {
            let inner = Vector::new(
                points,
                datum.into(),
                heading.into(),
                crs_from_c(crs),
            );
            Box::into_raw(Box::new(VectoryVectorHandle { inner }))
        }
        Err(err) => {
            set_last_error(err.to_string());
            ptr::null_mut()
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn vectory_vector_from_file(path: *const c_char) -> *mut VectoryVectorHandle {
    clear_last_error();
    match cstr_to_str(path, "path").and_then(Vector::from_file) {
        Ok(inner) => Box::into_raw(Box::new(VectoryVectorHandle { inner })),
        Err(err) => {
            set_last_error(err.to_string());
            ptr::null_mut()
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn vectory_vector_free(handle: *mut VectoryVectorHandle) {
    if !handle.is_null() {
        unsafe {
            drop(Box::from_raw(handle));
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn vectory_vector_to_file(
    handle: *const VectoryVectorHandle,
    path: *const c_char,
    crs: VectoryCrs,
) -> bool {
    match vector_from_ptr(handle)
        .and_then(|handle| cstr_to_str(path, "path").and_then(|path| handle.inner.to_file(path, crs_from_c(crs))))
    {
        Ok(()) => ok(),
        Err(err) => fail(err.to_string()),
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn vectory_vector_element_count(handle: *const VectoryVectorHandle) -> usize {
    vector_from_ptr(handle)
        .map(|handle| handle.inner.element_count())
        .unwrap_or(0)
}

#[unsafe(no_mangle)]
pub extern "C" fn vectory_vector_has_elements(handle: *const VectoryVectorHandle) -> bool {
    vector_from_ptr(handle)
        .map(|handle| handle.inner.has_elements())
        .unwrap_or(false)
}

#[unsafe(no_mangle)]
pub extern "C" fn vectory_vector_clear_elements(handle: *mut VectoryVectorHandle) -> bool {
    match vector_from_ptr_mut(handle) {
        Ok(handle) => {
            handle.inner.clear_elements();
            ok()
        }
        Err(err) => fail(err.to_string()),
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn vectory_vector_get_datum(
    handle: *const VectoryVectorHandle,
    out: *mut VectoryGeo3,
) -> bool {
    match vector_from_ptr(handle) {
        Ok(handle) => write_out(out, handle.inner.datum().into()),
        Err(err) => fail(err.to_string()),
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn vectory_vector_set_datum(
    handle: *mut VectoryVectorHandle,
    datum: VectoryGeo3,
) -> bool {
    match vector_from_ptr_mut(handle) {
        Ok(handle) => {
            handle.inner.set_datum(datum.into());
            ok()
        }
        Err(err) => fail(err.to_string()),
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn vectory_vector_get_heading(
    handle: *const VectoryVectorHandle,
    out: *mut VectoryEuler,
) -> bool {
    match vector_from_ptr(handle) {
        Ok(handle) => write_out(out, handle.inner.heading().into()),
        Err(err) => fail(err.to_string()),
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn vectory_vector_set_heading(
    handle: *mut VectoryVectorHandle,
    heading: VectoryEuler,
) -> bool {
    match vector_from_ptr_mut(handle) {
        Ok(handle) => {
            handle.inner.set_heading(heading.into());
            ok()
        }
        Err(err) => fail(err.to_string()),
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn vectory_vector_get_crs(
    handle: *const VectoryVectorHandle,
    out: *mut VectoryCrs,
) -> bool {
    match vector_from_ptr(handle) {
        Ok(handle) => write_out(
            out,
            match handle.inner.crs() {
                Crs::Wgs => VectoryCrs::Wgs,
                Crs::Enu => VectoryCrs::Enu,
            },
        ),
        Err(err) => fail(err.to_string()),
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn vectory_vector_set_crs(
    handle: *mut VectoryVectorHandle,
    crs: VectoryCrs,
) -> bool {
    match vector_from_ptr_mut(handle) {
        Ok(handle) => {
            handle.inner.set_crs(crs_from_c(crs));
            ok()
        }
        Err(err) => fail(err.to_string()),
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn vectory_vector_field_boundary_vertex_count(handle: *const VectoryVectorHandle) -> usize {
    vector_from_ptr(handle)
        .map(|handle| handle.inner.field_boundary().len())
        .unwrap_or(0)
}

#[unsafe(no_mangle)]
pub extern "C" fn vectory_vector_field_boundary_vertex_at(
    handle: *const VectoryVectorHandle,
    index: usize,
    out: *mut VectoryPoint3,
) -> bool {
    match vector_from_ptr(handle).and_then(|handle| {
        handle
            .inner
            .field_boundary()
            .get(index)
            .copied()
            .ok_or_else(|| crate::Error::InvalidGeoJson("field boundary index out of range".into()))
    }) {
        Ok(point) => write_out(out, point.into()),
        Err(err) => fail(err.to_string()),
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn vectory_vector_set_global_property(
    handle: *mut VectoryVectorHandle,
    key: *const c_char,
    value: *const c_char,
) -> bool {
    match vector_from_ptr_mut(handle).and_then(|handle| {
        let key = cstr_to_str(key, "key")?;
        let value = cstr_to_str(value, "value")?;
        handle.inner.set_global_property(key, value);
        Ok(())
    }) {
        Ok(()) => ok(),
        Err(err) => fail(err.to_string()),
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn vectory_vector_get_global_property(
    handle: *const VectoryVectorHandle,
    key: *const c_char,
    default_value: *const c_char,
) -> *mut c_char {
    clear_last_error();
    match vector_from_ptr(handle).and_then(|handle| {
        let key = cstr_to_str(key, "key")?;
        let default_value = if default_value.is_null() {
            ""
        } else {
            cstr_to_str(default_value, "default value")?
        };
        Ok(handle.inner.getGlobalProperty(key, default_value))
    }) {
        Ok(value) => string_to_ptr(value),
        Err(err) => {
            set_last_error(err.to_string());
            ptr::null_mut()
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn vectory_vector_add_point(
    handle: *mut VectoryVectorHandle,
    point: VectoryPoint3,
    kind: *const c_char,
) -> bool {
    match vector_from_ptr_mut(handle).and_then(|handle| {
        let kind = if kind.is_null() { "" } else { cstr_to_str(kind, "kind")? };
        handle.inner.add_point(point.into(), kind, HashMap::new());
        Ok(())
    }) {
        Ok(()) => ok(),
        Err(err) => fail(err.to_string()),
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn vectory_vector_add_segment(
    handle: *mut VectoryVectorHandle,
    segment: VectorySegment3,
    kind: *const c_char,
) -> bool {
    match vector_from_ptr_mut(handle).and_then(|handle| {
        let kind = if kind.is_null() { "" } else { cstr_to_str(kind, "kind")? };
        handle.inner.add_line(segment.into(), kind, HashMap::new());
        Ok(())
    }) {
        Ok(()) => ok(),
        Err(err) => fail(err.to_string()),
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn vectory_vector_add_path(
    handle: *mut VectoryVectorHandle,
    points: VectoryPointArrayView,
    kind: *const c_char,
) -> bool {
    match vector_from_ptr_mut(handle).and_then(|handle| {
        let kind = if kind.is_null() { "" } else { cstr_to_str(kind, "kind")? };
        handle.inner.add_path(
            read_points(points)?,
            kind,
            HashMap::new(),
        );
        Ok(())
    }) {
        Ok(()) => ok(),
        Err(err) => fail(err.to_string()),
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn vectory_vector_add_polygon(
    handle: *mut VectoryVectorHandle,
    points: VectoryPointArrayView,
    kind: *const c_char,
) -> bool {
    match vector_from_ptr_mut(handle).and_then(|handle| {
        let kind = if kind.is_null() { "" } else { cstr_to_str(kind, "kind")? };
        handle.inner.add_polygon(
            read_points(points)?,
            kind,
            HashMap::new(),
        );
        Ok(())
    }) {
        Ok(()) => ok(),
        Err(err) => fail(err.to_string()),
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn vectory_vector_element_kind(
    handle: *const VectoryVectorHandle,
    index: usize,
) -> *mut c_char {
    clear_last_error();
    match vector_from_ptr(handle).and_then(|handle| {
        let element = handle
            .inner
            .element(index)
            .ok_or_else(|| crate::Error::InvalidGeoJson("element index out of range".into()))?;
        Ok(element.kind.clone())
    }) {
        Ok(kind) => string_to_ptr(kind),
        Err(err) => {
            set_last_error(err.to_string());
            ptr::null_mut()
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn vectory_vector_element_geometry_kind(
    handle: *const VectoryVectorHandle,
    index: usize,
) -> VectoryGeometryKind {
    vector_from_ptr(handle)
        .ok()
        .and_then(|handle| handle.inner.element(index))
        .map(|element| geometry_kind(&element.geometry))
        .unwrap_or(VectoryGeometryKind::None)
}

#[unsafe(no_mangle)]
pub extern "C" fn vectory_vector_element_point(
    handle: *const VectoryVectorHandle,
    index: usize,
    out: *mut VectoryPoint3,
) -> bool {
    match vector_from_ptr(handle).and_then(|handle| {
        let element = handle
            .inner
            .element(index)
            .ok_or_else(|| crate::Error::InvalidGeoJson("element index out of range".into()))?;
        if let Geometry::Point(point) = element.geometry {
            Ok(point)
        } else {
            Err(crate::Error::InvalidGeoJson("element is not a point".into()))
        }
    }) {
        Ok(point) => write_out(out, point.into()),
        Err(err) => fail(err.to_string()),
    }
}
