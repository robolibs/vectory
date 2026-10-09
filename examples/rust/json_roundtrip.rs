use vectory::{Crs, read_json_str, to_json_string};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let input = r#"{
        "type": "FeatureCollection",
        "properties": {
            "crs": "ENU",
            "datum": [5.0, 52.0, 0.0],
            "heading": 0.0
        },
        "features": [{
            "type": "Feature",
            "geometry": { "type": "Point", "coordinates": [1.0, 2.0, 3.0] },
            "properties": { "name": "demo" }
        }]
    }"#;

    let fc = read_json_str(input)?;
    let output = to_json_string(&fc, Crs::Enu)?;
    println!("{output}");
    Ok(())
}
