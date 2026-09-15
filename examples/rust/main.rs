use vectory::{read, write_wgs};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut fc = read("/home/bresilla/data/code/robolibs/vectkit/misc/field4.geojson")?;
    println!("{fc}");

    fc.datum.latitude += 5.1;
    println!(
        "After tweak, new datum is: {}, {}, {}",
        fc.datum.latitude, fc.datum.longitude, fc.datum.altitude
    );

    write_wgs(&fc, "/tmp/field4_modified.geojson")?;
    println!("Saved modified GeoJSON to /tmp/field4_modified.geojson");
    Ok(())
}
