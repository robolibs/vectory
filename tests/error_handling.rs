use vectory::read;

fn write_temp(name: &str, contents: &str) -> std::path::PathBuf {
    let path = std::env::temp_dir().join(name);
    std::fs::write(&path, contents).unwrap();
    path
}

#[test]
fn malformed_json_errors() {
    let path = write_temp("vectory_invalid_json.geojson", "{ invalid json content }");
    assert!(read(path).is_err());
}

#[test]
fn missing_or_invalid_top_level_type_errors() {
    let missing = write_temp("vectory_missing_type.geojson", r#"{"features": []}"#);
    let invalid = write_temp(
        "vectory_invalid_type.geojson",
        r#"{"type": 123, "features": []}"#,
    );

    let missing_err = read(missing).unwrap_err().to_string();
    let invalid_err = read(invalid).unwrap_err().to_string();

    assert!(missing_err.contains("top-level object has no string 'type' field"));
    assert!(invalid_err.contains("top-level object has no string 'type' field"));
}

#[test]
fn invalid_required_properties_error() {
    let path = write_temp(
        "vectory_invalid_props.geojson",
        r#"{
            "type": "FeatureCollection",
            "properties": {
                "crs": 123,
                "datum": [5.0, 52.0],
                "heading": "invalid"
            },
            "features": []
        }"#,
    );

    let err = read(path).unwrap_err().to_string();
    assert!(err.contains("missing string 'crs'"));
}

#[test]
fn invalid_point_coordinates_error() {
    let path = write_temp(
        "vectory_invalid_point.geojson",
        r#"{
            "type": "FeatureCollection",
            "properties": {
                "crs": "EPSG:4326",
                "datum": [5.0, 52.0, 0.0],
                "heading": 0.0
            },
            "features": [{
                "type": "Feature",
                "geometry": {
                    "type": "Point",
                    "coordinates": [5.1]
                },
                "properties": {}
            }]
        }"#,
    );

    assert!(
        read(path)
            .unwrap_err()
            .to_string()
            .contains("Invalid point coordinates")
    );
}

#[test]
fn invalid_geometry_type_is_skipped_gracefully() {
    let path = write_temp(
        "vectory_invalid_geometry_type.geojson",
        r#"{
            "type": "FeatureCollection",
            "properties": {
                "crs": "EPSG:4326",
                "datum": [5.0, 52.0, 0.0],
                "heading": 0.0
            },
            "features": [{
                "type": "Feature",
                "geometry": {
                    "type": "InvalidType",
                    "coordinates": [5.1, 52.1]
                },
                "properties": {}
            }]
        }"#,
    );

    let collection = read(path).unwrap();
    assert!(collection.features.is_empty());
}
