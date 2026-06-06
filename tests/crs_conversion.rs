use std::collections::HashMap;

use concord::{Geo, Wgs, to_enu};
use datapod::Point;
use vectory::{Crs, Feature, FeatureCollection, Geometry, Heading, read, write};

fn write_temp(name: &str, contents: &str) -> std::path::PathBuf {
    let path = std::env::temp_dir().join(name);
    std::fs::write(&path, contents).unwrap();
    path
}

#[test]
fn wgs_input_is_normalized_to_internal_enu() {
    let path = write_temp(
        "vectory_crs_input.geojson",
        r#"{
            "type": "FeatureCollection",
            "properties": {
                "crs": "EPSG:4326",
                "datum": [5.0, 52.0, 100.0],
                "heading": 45.0
            },
            "features": [{
                "type": "Feature",
                "geometry": { "type": "Point", "coordinates": [5.1, 52.1, 105.0] },
                "properties": { "name": "test_point" }
            }]
        }"#,
    );

    let collection = read(&path).unwrap();
    match &collection.features[0].geometry {
        Geometry::Point(point) => {
            assert_ne!(point.x, 5.1);
            assert_ne!(point.y, 52.1);
        }
        _ => panic!("expected point geometry"),
    }
}

#[test]
fn wgs_and_enu_output_round_trip_to_same_internal_point() {
    let input = write_temp(
        "vectory_crs_roundtrip_input.geojson",
        r#"{
            "type": "FeatureCollection",
            "properties": {
                "crs": "EPSG:4326",
                "datum": [5.0, 52.0, 100.0],
                "heading": 45.0
            },
            "features": [{
                "type": "Feature",
                "geometry": { "type": "Point", "coordinates": [5.1, 52.1, 105.0] },
                "properties": { "name": "test_point" }
            }]
        }"#,
    );
    let original = read(&input).unwrap();

    let wgs_path = std::env::temp_dir().join("vectory_output_wgs.geojson");
    let enu_path = std::env::temp_dir().join("vectory_output_enu.geojson");
    write(&original, &wgs_path, Crs::Wgs).unwrap();
    write(&original, &enu_path, Crs::Enu).unwrap();

    let wgs_collection = read(&wgs_path).unwrap();
    let enu_collection = read(&enu_path).unwrap();

    match (
        &wgs_collection.features[0].geometry,
        &enu_collection.features[0].geometry,
    ) {
        (Geometry::Point(a), Geometry::Point(b)) => {
            assert!((a.x - b.x).abs() < 1e-6);
            assert!((a.y - b.y).abs() < 1e-6);
            assert!((a.z - b.z).abs() < 1e-6);
        }
        _ => panic!("expected point geometries"),
    }
}

#[test]
fn wgs_and_enu_flavors_round_trip() {
    let datum = Geo::new(52.0, 5.0, 0.0);
    let heading = Heading::new(0.0, 0.0, 1.5);

    let enu = to_enu(datum, Wgs::new(52.1, 5.1, 10.0));
    let point = Point::new(enu.east(), enu.north(), enu.up());
    let mut properties = HashMap::new();
    properties.insert("name".into(), "test_point".into());

    let mut collection = FeatureCollection::new(datum, heading);
    collection.features.push(Feature::new(point, properties));

    let wgs_path = std::env::temp_dir().join("vectory_wgs_flavor.geojson");
    write(&collection, &wgs_path, Crs::Wgs).unwrap();
    let loaded_wgs = read(&wgs_path).unwrap();
    assert_eq!(loaded_wgs.features.len(), 1);
    assert_eq!(loaded_wgs.features[0].properties["name"], "test_point");

    let enu_path = std::env::temp_dir().join("vectory_enu_flavor.geojson");
    write(&collection, &enu_path, Crs::Enu).unwrap();
    let loaded_enu = read(&enu_path).unwrap();
    assert_eq!(loaded_enu.features.len(), 1);
    assert_eq!(loaded_enu.features[0].properties["name"], "test_point");
}
