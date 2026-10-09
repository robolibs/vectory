use vectory::{Crs, read, write, write_wgs};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let fc = read("/home/bresilla/data/code/robolibs/vectkit/misc/field4.geojson")?;

    println!("Original file information:\n{fc}");
    println!("\nInternal representation is always ENU/local coordinates.");

    write_wgs(&fc, "/tmp/output_wgs84.geojson")?;
    write(&fc, "/tmp/output_enu.geojson", Crs::Enu)?;
    write_wgs(&fc, "/tmp/output_original.geojson")?;

    println!("Created:");
    println!("- /tmp/output_wgs84.geojson");
    println!("- /tmp/output_enu.geojson");
    println!("- /tmp/output_original.geojson");
    Ok(())
}
