use std::collections::HashMap;

use concord::Geo;
use datapod::{Point, Segment};
use vectory::{Crs, FeatureCollection, Geometry, Heading, Vector};

fn square(size: f64) -> Vec<Point> {
    vec![
        Point::new(0.0, 0.0, 0.0),
        Point::new(size, 0.0, 0.0),
        Point::new(size, size, 0.0),
        Point::new(0.0, size, 0.0),
    ]
}

#[test]
fn cpp_style_vector_aliases_work() {
    let mut vector = Vector::new(
        square(10.0),
        Geo::new(52.0, 5.0, 0.0),
        Heading::default(),
        Crs::Enu,
    );
    vector.add_point(Point::new(1.0, 2.0, 3.0), "waypoint", HashMap::new());
    vector.add_line(
        Segment::new(Point::new(0.0, 0.0, 0.0), Point::new(1.0, 1.0, 1.0)),
        "line",
        HashMap::new(),
    );

    assert_eq!(vector.elementCount(), 2);
    assert!(vector.hasElements());
    assert_eq!(vector.getPoints().len(), 1);
    assert_eq!(vector.getLines().len(), 1);
    assert!(matches!(
        vector.getElement(0).unwrap().geometry,
        Geometry::Point(_)
    ));
    assert_eq!(vector.getElementsByType("waypoint").len(), 1);

    vector.setGlobalProperty("field_id", "F42");
    assert_eq!(vector.getGlobalProperty("field_id", ""), "F42");
    assert_eq!(vector.getCRS(), Crs::Enu);
}

#[test]
fn cpp_style_io_aliases_work() {
    let collection = FeatureCollection::new(Geo::new(52.0, 5.0, 0.0), Heading::default());
    let path = std::env::temp_dir().join("vectory_cpp_aliases.geojson");
    vectory::WriteFeatureCollectionWgs(&collection, &path).unwrap();
    let loaded = vectory::ReadFeatureCollection(&path).unwrap();
    assert_eq!(loaded.features.len(), 0);
}
