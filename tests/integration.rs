use std::collections::HashMap;

use concord::{Geo, Wgs, to_enu};
use datapod::{Point, Polygon, Segment};
use vectory::{Crs, Feature, FeatureCollection, Geometry, Heading, read, write};

#[test]
fn round_trip_conversion_keeps_collection_shape() {
    let datum = Geo::new(52.0, 5.0, 0.0);
    let heading = Heading::new(0.0, 0.0, 2.0);

    let point = {
        let enu = to_enu(datum, Wgs::new(52.1, 5.1, 10.0));
        Point::new(enu.east(), enu.north(), enu.up())
    };
    let line = {
        let start = to_enu(datum, Wgs::new(52.1, 5.1, 0.0));
        let end = to_enu(datum, Wgs::new(52.2, 5.2, 0.0));
        Segment::new(
            Point::new(start.east(), start.north(), start.up()),
            Point::new(end.east(), end.north(), end.up()),
        )
    };
    let path = datapod::Linestring::new(
        [Wgs::new(52.1, 5.1, 0.0), Wgs::new(52.2, 5.2, 0.0), Wgs::new(52.3, 5.3, 0.0)]
            .into_iter()
            .map(|wgs| {
                let enu = to_enu(datum, wgs);
                Point::new(enu.east(), enu.north(), enu.up())
            })
            .collect::<Vec<_>>(),
    );
    let polygon = Polygon::new(
        [
            Wgs::new(52.1, 5.1, 0.0),
            Wgs::new(52.2, 5.1, 0.0),
            Wgs::new(52.2, 5.2, 0.0),
            Wgs::new(52.1, 5.2, 0.0),
            Wgs::new(52.1, 5.1, 0.0),
        ]
        .into_iter()
        .map(|wgs| {
            let enu = to_enu(datum, wgs);
            Point::new(enu.east(), enu.north(), enu.up())
        })
        .collect::<Vec<_>>(),
    );

    let mut collection = FeatureCollection::new(datum, heading);
    collection
        .features
        .push(Feature::new(point, HashMap::from([(String::from("name"), String::from("test_point"))])));
    collection
        .features
        .push(Feature::new(line, HashMap::from([(String::from("name"), String::from("test_line"))])));
    collection
        .features
        .push(Feature::new(path, HashMap::from([(String::from("name"), String::from("test_path"))])));
    collection.features.push(Feature::new(
        polygon,
        HashMap::from([(String::from("name"), String::from("test_polygon"))]),
    ));

    let path = std::env::temp_dir().join("vectory_round_trip_test.geojson");
    write(&collection, &path, Crs::Wgs).unwrap();
    let loaded = read(&path).unwrap();

    assert_eq!(loaded.datum.latitude, collection.datum.latitude);
    assert_eq!(loaded.datum.longitude, collection.datum.longitude);
    assert_eq!(loaded.datum.altitude, collection.datum.altitude);
    assert_eq!(loaded.heading.yaw, collection.heading.yaw);
    assert_eq!(loaded.features.len(), collection.features.len());
}

#[test]
fn read_existing_vectkit_fixture() {
    let path = "/home/bresilla/data/code/robolibs/vectkit/misc/field4.geojson";
    let collection = read(path).unwrap();

    assert!((collection.datum.latitude - 51.9877).abs() < 1e-6);
    assert!((collection.datum.longitude - 5.65).abs() < 1e-6);
    assert_eq!(collection.datum.altitude, 0.0);
    assert_eq!(collection.heading.yaw, 0.0);
    assert_eq!(collection.features.len(), 1);
    assert!(matches!(collection.features[0].geometry, Geometry::Polygon(_)));
}

#[test]
fn modify_and_save_fixture() {
    let path = "/home/bresilla/data/code/robolibs/vectkit/misc/field4.geojson";
    let mut collection = read(path).unwrap();
    collection.datum.latitude += 5.1;

    let output = std::env::temp_dir().join("vectory_modified_test.geojson");
    write(&collection, &output, Crs::Wgs).unwrap();
    let modified = read(&output).unwrap();

    assert!((modified.datum.latitude - 57.0877).abs() < 1e-6);
}
