use std::collections::HashMap;

use concord::{Geo, Wgs, to_enu, to_wgs_from_enu};
use datapod::{Point, Polygon, Segment, Vector as PodVector};
use vectory::{Feature, FeatureCollection, Geometry, Heading};

#[test]
fn geometry_accepts_core_datapod_types() {
    let datum = Geo::new(52.0, 5.0, 0.0);

    let enu = to_enu(datum, Wgs::new(52.1, 5.1, 10.0));
    let point = Point::new(enu.east(), enu.north(), enu.up());
    let geom = Geometry::from(point);
    match geom {
        Geometry::Point(value) => {
            let back = to_wgs_from_enu(concord::Enu::new(value.x, value.y, value.z, datum));
            assert!((back.latitude - 52.1).abs() < 1e-6);
            assert!((back.longitude - 5.1).abs() < 1e-6);
            assert!((back.altitude - 10.0).abs() < 1e-6);
        }
        _ => panic!("expected point geometry"),
    }

    let segment = Segment::new(Point::new(1.0, 2.0, 3.0), Point::new(4.0, 5.0, 6.0));
    assert!(matches!(Geometry::from(segment), Geometry::Segment(_)));

    let path = datapod::Linestring {
        points: PodVector::from(vec![Point::new(1.0, 2.0, 0.0), Point::new(2.0, 3.0, 0.0)]),
    };
    assert!(matches!(Geometry::from(path), Geometry::Path(_)));

    let polygon = Polygon {
        vertices: PodVector::from(vec![
            Point::new(0.0, 0.0, 0.0),
            Point::new(1.0, 0.0, 0.0),
            Point::new(1.0, 1.0, 0.0),
        ]),
    };
    assert!(matches!(Geometry::from(polygon), Geometry::Polygon(_)));
}

#[test]
fn feature_and_collection_hold_expected_data() {
    let mut point_props = HashMap::new();
    point_props.insert("name".into(), "test_point".into());
    let feature = Feature::new(Point::new(1.0, 2.0, 3.0), point_props.clone());
    assert!(matches!(feature.geometry, Geometry::Point(_)));
    assert_eq!(feature.properties["name"], "test_point");

    let mut collection = FeatureCollection::new(Geo::new(52.0, 5.0, 0.0), Heading::new(0.0, 0.0, 2.0));
    collection.features.push(feature);
    collection.features.push(Feature::new(
        Segment::new(Point::new(0.0, 0.0, 0.0), Point::new(1.0, 1.0, 1.0)),
        HashMap::from([(String::from("name"), String::from("test_line"))]),
    ));

    assert_eq!(collection.datum.latitude, 52.0);
    assert_eq!(collection.heading.yaw, 2.0);
    assert_eq!(collection.features.len(), 2);
    assert!(matches!(collection.features[0].geometry, Geometry::Point(_)));
    assert!(matches!(collection.features[1].geometry, Geometry::Segment(_)));
}

#[test]
fn feature_collection_display_matches_expected_shape() {
    let mut collection = FeatureCollection::new(Geo::new(52.0, 5.0, 0.0), Heading::new(0.0, 0.0, 2.0));
    collection.features.push(Feature::new(
        Point::new(1.0, 2.0, 3.0),
        HashMap::from([(String::from("name"), String::from("test"))]),
    ));

    let rendered = collection.to_string();
    assert!(rendered.contains("DATUM: 52"));
    assert!(rendered.contains("HEADING: 2"));
    assert!(rendered.contains("FEATURES: 1"));
    assert!(rendered.contains("POINT"));
    assert!(rendered.contains("PROPS:1"));
}
