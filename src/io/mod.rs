mod parse;
mod write;

pub use parse::{parse_geojson, read, read_feature_collection};
pub use write::{to_json_string, write, write_feature_collection, write_wgs};

pub fn read_json_str(text: &str) -> crate::Result<crate::FeatureCollection> {
    let value: serde_json::Value = serde_json::from_str(text)
        .map_err(|err| crate::Error::InvalidGeoJson(format!("failed to parse JSON: {err}")))?;
    parse_geojson(value)
}

#[allow(non_snake_case)]
pub fn ReadFeatureCollection(
    path: impl AsRef<std::path::Path>,
) -> crate::Result<crate::FeatureCollection> {
    read(path)
}

#[allow(non_snake_case)]
pub fn WriteFeatureCollection(
    collection: &crate::FeatureCollection,
    path: impl AsRef<std::path::Path>,
    crs: crate::Crs,
) -> crate::Result<()> {
    write(collection, path, crs)
}

#[allow(non_snake_case)]
pub fn WriteFeatureCollectionWgs(
    collection: &crate::FeatureCollection,
    path: impl AsRef<std::path::Path>,
) -> crate::Result<()> {
    write_wgs(collection, path)
}
