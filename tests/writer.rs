use std::collections::HashMap;

use concord::{Geo, Wgs, to_enu};
use vectory::{Crs, Feature, FeatureCollection, Geometry, Heading, Point3, Segment3, read, write};

#[test]
fn writer_round_trips_wgs_and_enu() {
    let datum = Geo::new(52.0, 5.0, 0.0);
    let heading = Heading::new(0.0, 0.0, 2.0);

    let enu_point = to_enu(datum, Wgs::new(52.1, 5.1, 10.0));
    let point = Point3::new(enu_point.east(), enu_point.north(), enu_point.up());
    let mut point_props = HashMap::new();
    point_props.insert("name".into(), "test_point".into());

    let enu_start = to_enu(datum, Wgs::new(52.1, 5.1, 0.0));
    let enu_end = to_enu(datum, Wgs::new(52.2, 5.2, 0.0));
    let line = Segment3::new(
        Point3::new(enu_start.east(), enu_start.north(), enu_start.up()),
        Point3::new(enu_end.east(), enu_end.north(), enu_end.up()),
    );
    let mut line_props = HashMap::new();
    line_props.insert("name".into(), "test_line".into());

    let mut collection = FeatureCollection::new(datum, heading);
    collection.features.push(Feature {
        geometry: Geometry::Point(point),
        properties: point_props,
    });
    collection.features.push(Feature {
        geometry: Geometry::Segment(line),
        properties: line_props,
    });

    let wgs_path = std::env::temp_dir().join("vectory_writer_wgs.geojson");
    write(&collection, &wgs_path, Crs::Wgs).unwrap();
    let loaded = read(&wgs_path).unwrap();
    assert_eq!(loaded.features.len(), 2);
    assert!(matches!(loaded.features[0].geometry, Geometry::Point(_)));
    assert!(matches!(loaded.features[1].geometry, Geometry::Segment(_)));

    let enu_path = std::env::temp_dir().join("vectory_writer_enu.geojson");
    write(&collection, &enu_path, Crs::Enu).unwrap();
    let loaded = read(&enu_path).unwrap();
    assert_eq!(loaded.features.len(), 2);
}
