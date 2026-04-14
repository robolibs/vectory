#[path = "support/rerun_viz.rs"]
mod rerun_viz;

use rerun::Color;
use vectory::{Geometry, read};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let fc = read("/home/bresilla/data/code/robolibs/vectkit/misc/field4.geojson")?;
    let rec = rerun_viz::connect("vectory_wgs")?;

    for (index, feature) in fc.features.iter().enumerate() {
        if let Geometry::Polygon(polygon) = &feature.geometry {
            println!("Found polygon with {} vertices", polygon.vertices.len());
            for vertex in &polygon.vertices {
                println!("  Vertex: ({}, {}, {})", vertex.x, vertex.y, vertex.z);
            }
            rerun_viz::log_polygon(
                &rec,
                &format!("enu/field/{index}"),
                polygon,
                Color::from_rgb(70, 120, 70),
            )?;
            rerun_viz::log_polygon_geo(
                &rec,
                &format!("geo/field/{index}"),
                polygon,
                fc.datum,
                Color::from_rgb(70, 120, 70),
            )?;
        }
    }

    rec.flush_blocking()?;
    Ok(())
}
