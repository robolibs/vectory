use concord::Geo;
use vectory::{Crs, Heading, Point3, Vector};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let boundary = vec![
        Point3::new(0.0, 0.0, 0.0),
        Point3::new(100.0, 0.0, 0.0),
        Point3::new(100.0, 100.0, 0.0),
        Point3::new(0.0, 100.0, 0.0),
    ];

    let mut vector = Vector::new(
        boundary,
        Geo::new(52.0, 5.0, 0.0),
        Heading::new(0.0, 0.0, 0.0),
        Crs::Enu,
    );
    vector.set_field_property("name", "Demo Field");
    vector.add_point(Point3::new(25.0, 25.0, 0.0), "marker", Default::default());
    vector.add_point(Point3::new(50.0, 50.0, 0.0), "waypoint", Default::default());

    println!("elements: {}", vector.element_count());
    println!("points: {}", vector.points().len());
    println!(
        "field name: {}",
        vector.getGlobalProperty("missing", "unset")
    );

    vector.to_file("/tmp/vectory_vector_workflow.geojson", Crs::Wgs)?;
    println!("wrote /tmp/vectory_vector_workflow.geojson");
    Ok(())
}
