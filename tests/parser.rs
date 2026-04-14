use vectory::{Geometry, read};

fn write_temp(name: &str, contents: &str) -> std::path::PathBuf {
    let path = std::env::temp_dir().join(name);
    std::fs::write(&path, contents).unwrap();
    path
}

#[test]
fn parser_preserves_feature_properties() {
    let path = write_temp(
        "vectory_parser_properties.geojson",
        r#"{
            "type": "FeatureCollection",
            "properties": {
                "crs": "EPSG:4326",
                "datum": [5.0, 52.0, 0.0],
                "heading": 2.0
            },
            "features": [{
                "type": "Feature",
                "geometry": { "type": "Point", "coordinates": [5.1, 52.1, 10.0] },
                "properties": { "name": "test_point", "number": 42, "boolean": true }
            }]
        }"#,
    );

    let collection = read(&path).unwrap();
    assert_eq!(collection.features.len(), 1);
    assert_eq!(collection.features[0].properties["name"], "test_point");
    assert_eq!(collection.features[0].properties["number"], "42");
    assert_eq!(collection.features[0].properties["boolean"], "true");
}

#[test]
fn parser_supports_core_geometry_types() {
    let point_path = write_temp(
        "vectory_parser_point.geojson",
        r#"{
            "type": "FeatureCollection",
            "properties": { "crs": "EPSG:4326", "datum": [5.0, 52.0, 0.0], "heading": 2.0 },
            "features": [{
                "type": "Feature",
                "geometry": { "type": "Point", "coordinates": [5.1, 52.1, 10.0] },
                "properties": {}
            }]
        }"#,
    );
    let line_path = write_temp(
        "vectory_parser_line.geojson",
        r#"{
            "type": "FeatureCollection",
            "properties": { "crs": "EPSG:4326", "datum": [5.0, 52.0, 0.0], "heading": 2.0 },
            "features": [{
                "type": "Feature",
                "geometry": { "type": "LineString", "coordinates": [[5.1, 52.1, 0.0], [5.2, 52.2, 0.0]] },
                "properties": {}
            }]
        }"#,
    );
    let polygon_path = write_temp(
        "vectory_parser_polygon.geojson",
        r#"{
            "type": "FeatureCollection",
            "properties": { "crs": "EPSG:4326", "datum": [5.0, 52.0, 0.0], "heading": 2.0 },
            "features": [{
                "type": "Feature",
                "geometry": { "type": "Polygon", "coordinates": [[[5.1, 52.1, 0.0], [5.2, 52.1, 0.0], [5.2, 52.2, 0.0], [5.1, 52.2, 0.0], [5.1, 52.1, 0.0]]] },
                "properties": {}
            }]
        }"#,
    );

    let point_collection = read(point_path).unwrap();
    let line_collection = read(line_path).unwrap();
    let polygon_collection = read(polygon_path).unwrap();

    assert!(matches!(point_collection.features[0].geometry, Geometry::Point(_)));
    assert!(matches!(line_collection.features[0].geometry, Geometry::Segment(_)));
    assert!(matches!(polygon_collection.features[0].geometry, Geometry::Polygon(_)));
}

#[test]
fn parser_errors_on_missing_properties() {
    let path = write_temp(
        "vectory_parser_missing_props.geojson",
        r#"{"type": "FeatureCollection", "features": []}"#,
    );

    let error = read(path).unwrap_err();
    assert!(error.to_string().contains("missing top-level 'properties'"));
}

#[test]
fn parser_top_level_feature_and_bare_geometry_error_without_properties() {
    let feature_path = write_temp(
        "vectory_top_level_feature.geojson",
        r#"{
            "type": "Feature",
            "geometry": { "type": "Point", "coordinates": [5.1, 52.1, 10.0] },
            "properties": {}
        }"#,
    );
    let geometry_path = write_temp(
        "vectory_bare_geometry.geojson",
        r#"{
            "type": "Point",
            "coordinates": [5.1, 52.1, 10.0]
        }"#,
    );

    assert!(read(feature_path)
        .unwrap_err()
        .to_string()
        .contains("missing top-level 'properties'"));
    assert!(read(geometry_path)
        .unwrap_err()
        .to_string()
        .contains("missing top-level 'properties'"));
}

#[test]
fn parser_flattens_multi_geometries() {
    let path = write_temp(
        "vectory_multigeom.geojson",
        r#"{
            "type": "FeatureCollection",
            "properties": {
                "crs": "ENU",
                "datum": [5.0, 52.0, 0.0],
                "heading": 0.0,
                "farm": "alpha"
            },
            "features": [{
                "type": "Feature",
                "geometry": {
                    "type": "GeometryCollection",
                    "geometries": [
                        { "type": "MultiPoint", "coordinates": [[1,2,3], [4,5,6]] },
                        { "type": "MultiLineString", "coordinates": [[[0,0,0], [1,1,1]], [[2,2,2], [3,3,3], [4,4,4]]] }
                    ]
                },
                "properties": { "kind": "mixed" }
            }]
        }"#,
    );

    let collection = read(path).unwrap();
    assert_eq!(collection.global_properties["farm"], "alpha");
    assert_eq!(collection.features.len(), 4);
    assert!(matches!(collection.features[0].geometry, Geometry::Point(_)));
    assert!(matches!(collection.features[1].geometry, Geometry::Point(_)));
    assert!(matches!(collection.features[2].geometry, Geometry::Segment(_)));
    assert!(matches!(collection.features[3].geometry, Geometry::Path(_)));
}

#[test]
fn parser_errors_on_missing_required_properties() {
    let missing_crs = write_temp(
        "vectory_missing_crs.geojson",
        r#"{
            "type": "FeatureCollection",
            "properties": { "datum": [5.0, 52.0, 0.0], "heading": 0.0 },
            "features": []
        }"#,
    );
    let missing_datum = write_temp(
        "vectory_missing_datum.geojson",
        r#"{
            "type": "FeatureCollection",
            "properties": { "crs": "ENU", "heading": 0.0 },
            "features": []
        }"#,
    );
    let missing_heading = write_temp(
        "vectory_missing_heading.geojson",
        r#"{
            "type": "FeatureCollection",
            "properties": { "crs": "ENU", "datum": [5.0, 52.0, 0.0] },
            "features": []
        }"#,
    );

    assert!(read(missing_crs).unwrap_err().to_string().contains("missing string 'crs'"));
    assert!(read(missing_datum).unwrap_err().to_string().contains("missing array 'datum'"));
    assert!(read(missing_heading).unwrap_err().to_string().contains("missing numeric 'heading'"));
}
