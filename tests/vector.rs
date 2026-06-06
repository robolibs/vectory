use std::collections::HashMap;

use concord::Geo;
use vectory::{Crs, Heading, Point3, Segment3, Vector};

fn square(size: f64) -> Vec<Point3> {
    vec![
        Point3::new(0.0, 0.0, 0.0),
        Point3::new(size, 0.0, 0.0),
        Point3::new(size, size, 0.0),
        Point3::new(0.0, size, 0.0),
    ]
}

#[test]
fn vector_basic_construction_and_queries() {
    let mut vector = Vector::new(
        square(100.0),
        Geo::new(52.0, 5.0, 0.0),
        Heading::default(),
        Crs::Enu,
    );
    assert_eq!(vector.element_count(), 0);
    assert!(!vector.has_elements());

    let mut props = HashMap::new();
    props.insert("id".into(), "wp1".into());
    vector.add_point(Point3::new(50.0, 50.0, 0.0), "waypoint", props);
    vector.add_line(
        Segment3::new(Point3::new(10.0, 10.0, 0.0), Point3::new(90.0, 90.0, 0.0)),
        "boundary",
        HashMap::new(),
    );

    assert_eq!(vector.element_count(), 2);
    assert_eq!(vector.points().len(), 1);
    assert_eq!(vector.lines().len(), 1);
    assert_eq!(vector.elements_by_type("waypoint").len(), 1);
}

#[test]
fn vector_round_trips_to_file() {
    let mut vector = Vector::new(
        square(100.0),
        Geo::new(52.0, 5.0, 0.0),
        Heading::default(),
        Crs::Enu,
    );
    vector.set_field_property("name", "Test Field");
    let mut props = HashMap::new();
    props.insert("important".into(), "true".into());
    vector.add_point(Point3::new(50.0, 50.0, 0.0), "center", props);

    let path = std::env::temp_dir().join("vectory_vector.geojson");
    vector.to_file(&path, Crs::Wgs).unwrap();

    let loaded = Vector::from_file(&path).unwrap();
    assert_eq!(loaded.element_count(), 1);
    assert_eq!(loaded.field_properties()["name"], "Test Field");
    assert_eq!(loaded.points().len(), 1);
}

#[test]
fn vector_uses_first_polygon_as_boundary_when_field_type_is_missing() {
    let geojson = r#"{
        "type": "FeatureCollection",
        "properties": { "crs": "ENU", "datum": [5.0, 52.0, 0.0], "heading": 0.0 },
        "features": [
            {
                "type": "Feature",
                "geometry": { "type": "Polygon", "coordinates": [[[0,0,0], [10,0,0], [10,10,0], [0,10,0]]] },
                "properties": { "name": "field-ish" }
            },
            {
                "type": "Feature",
                "geometry": { "type": "Point", "coordinates": [1,1,0] },
                "properties": { "type": "marker" }
            }
        ]
    }"#;
    let path = std::env::temp_dir().join("vectory_vector_first_polygon.geojson");
    std::fs::write(&path, geojson).unwrap();

    let loaded = Vector::from_file(&path).unwrap();
    assert_eq!(loaded.element_count(), 2);
    assert_eq!(loaded.field_properties()["name"], "field-ish");
    assert_eq!(loaded.polygons().len(), 1);
    assert_eq!(loaded.points().len(), 1);
}

#[test]
fn vector_errors_when_file_has_no_features() {
    let geojson = r#"{
        "type": "FeatureCollection",
        "properties": { "crs": "ENU", "datum": [5.0, 52.0, 0.0], "heading": 0.0 },
        "features": []
    }"#;
    let path = std::env::temp_dir().join("vectory_vector_empty.geojson");
    std::fs::write(&path, geojson).unwrap();

    let err = Vector::from_file(&path).unwrap_err().to_string();
    assert!(err.contains("Vector::fromFile: No features found in file"));
}

#[test]
fn vector_skips_all_explicit_field_features_from_elements() {
    let geojson = r#"{
        "type": "FeatureCollection",
        "properties": { "crs": "ENU", "datum": [5.0, 52.0, 0.0], "heading": 0.0 },
        "features": [
            {
                "type": "Feature",
                "geometry": { "type": "Polygon", "coordinates": [[[0,0,0], [10,0,0], [10,10,0], [0,10,0]]] },
                "properties": { "type": "field", "name": "primary" }
            },
            {
                "type": "Feature",
                "geometry": { "type": "Polygon", "coordinates": [[[20,20,0], [30,20,0], [30,30,0], [20,30,0]]] },
                "properties": { "type": "field", "name": "secondary" }
            },
            {
                "type": "Feature",
                "geometry": { "type": "Point", "coordinates": [1,1,0] },
                "properties": { "type": "marker" }
            }
        ]
    }"#;
    let path = std::env::temp_dir().join("vectory_vector_multi_field.geojson");
    std::fs::write(&path, geojson).unwrap();

    let loaded = Vector::from_file(&path).unwrap();
    assert_eq!(loaded.field_properties()["name"], "primary");
    assert_eq!(loaded.element_count(), 1);
    assert_eq!(loaded.points().len(), 1);
}
