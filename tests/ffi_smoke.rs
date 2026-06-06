use std::ffi::{CStr, CString};

use vectory::ffi::{
    VectoryCrs, VectoryEuler, VectoryGeo3, VectoryGeometryKind, VectoryPoint3,
    VectoryPointArrayView, vectory_feature_collection_feature_count,
    vectory_feature_collection_feature_geometry_kind, vectory_feature_collection_feature_point,
    vectory_feature_collection_free, vectory_feature_collection_from_json,
    vectory_feature_collection_to_json, vectory_last_error_message, vectory_string_free,
    vectory_vector_add_point, vectory_vector_element_count, vectory_vector_free,
    vectory_vector_new,
};

fn cstr(value: &str) -> CString {
    CString::new(value).unwrap()
}

#[test]
fn ffi_feature_collection_json_roundtrip() {
    let json = cstr(
        r#"{
            "type": "FeatureCollection",
            "properties": {
                "crs": "ENU",
                "datum": [5.0, 52.0, 0.0],
                "heading": 0.0
            },
            "features": [{
                "type": "Feature",
                "geometry": { "type": "Point", "coordinates": [1.0, 2.0, 3.0] },
                "properties": { "name": "demo" }
            }]
        }"#,
    );

    let handle = vectory_feature_collection_from_json(json.as_ptr());
    assert!(!handle.is_null(), "ffi read failed");
    assert_eq!(vectory_feature_collection_feature_count(handle), 1);
    assert_eq!(
        vectory_feature_collection_feature_geometry_kind(handle, 0),
        VectoryGeometryKind::Point
    );

    let mut point = VectoryPoint3 {
        x: 0.0,
        y: 0.0,
        z: 0.0,
    };
    assert!(vectory_feature_collection_feature_point(
        handle, 0, &mut point
    ));
    assert_eq!((point.x, point.y, point.z), (1.0, 2.0, 3.0));

    let encoded = vectory_feature_collection_to_json(handle, VectoryCrs::Enu);
    assert!(!encoded.is_null());
    let encoded_text = unsafe { CStr::from_ptr(encoded) }
        .to_str()
        .unwrap()
        .to_string();
    assert!(encoded_text.contains("\"FeatureCollection\""));
    vectory_string_free(encoded);
    vectory_feature_collection_free(handle);
}

#[test]
fn ffi_vector_basic_workflow() {
    let boundary = [
        VectoryPoint3 {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        },
        VectoryPoint3 {
            x: 10.0,
            y: 0.0,
            z: 0.0,
        },
        VectoryPoint3 {
            x: 10.0,
            y: 10.0,
            z: 0.0,
        },
        VectoryPoint3 {
            x: 0.0,
            y: 10.0,
            z: 0.0,
        },
    ];
    let handle = vectory_vector_new(
        VectoryPointArrayView {
            ptr: boundary.as_ptr(),
            len: boundary.len(),
        },
        VectoryGeo3 {
            latitude: 52.0,
            longitude: 5.0,
            altitude: 0.0,
        },
        VectoryEuler {
            roll: 0.0,
            pitch: 0.0,
            yaw: 0.0,
        },
        VectoryCrs::Enu,
    );
    assert!(!handle.is_null());
    assert_eq!(vectory_vector_element_count(handle), 0);

    let kind = cstr("marker");
    assert!(vectory_vector_add_point(
        handle,
        VectoryPoint3 {
            x: 1.0,
            y: 2.0,
            z: 3.0,
        },
        kind.as_ptr(),
    ));
    assert_eq!(vectory_vector_element_count(handle), 1);
    vectory_vector_free(handle);
}

#[test]
fn ffi_reports_errors() {
    let bad_json = cstr("{ nope");
    let handle = vectory_feature_collection_from_json(bad_json.as_ptr());
    assert!(handle.is_null());
    let err = unsafe { CStr::from_ptr(vectory_last_error_message()) }
        .to_str()
        .unwrap()
        .to_string();
    assert!(!err.is_empty());
    assert!(err.contains("failed to parse JSON"));
    let _ = VectoryCrs::Wgs;
}
