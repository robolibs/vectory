use vectory::{read, write_wgs};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut fc = read("/home/bresilla/data/code/robolibs/vectkit/misc/field4.geojson")?;
    println!("loaded features: {}", fc.features.len());
    println!(
        "datum before: {:?}",
        (fc.datum.latitude, fc.datum.longitude, fc.datum.altitude)
    );

    fc.datum.latitude += 0.1;
    write_wgs(&fc, "/tmp/vectory_read_write.geojson")?;

    println!("wrote /tmp/vectory_read_write.geojson");
    Ok(())
}
