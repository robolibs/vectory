use std::path::PathBuf;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("io error for {path}: {source}")]
    Io {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("failed to parse GeoJSON: {0}")]
    GeoJson(Box<geojson::Error>),
    #[error("{0}")]
    InvalidGeoJson(String),
}

impl From<geojson::Error> for Error {
    fn from(value: geojson::Error) -> Self {
        Self::GeoJson(Box::new(value))
    }
}

pub type Result<T> = std::result::Result<T, Error>;
