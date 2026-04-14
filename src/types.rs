use std::collections::HashMap;
use std::fmt;

pub use datapod::{
    Euler as Heading, Geo, Linestring as Path3, Point as Point3, Polygon as Polygon3,
    Segment as Segment3,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Crs {
    Wgs,
    Enu,
}

impl Crs {
    pub fn parse(value: &str) -> crate::Result<Self> {
        match value {
            "EPSG:4326" | "WGS84" | "WGS" => Ok(Self::Wgs),
            "ENU" | "ECEF" => Ok(Self::Enu),
            other => Err(crate::Error::InvalidGeoJson(format!(
                "Unknown CRS string: {other}"
            ))),
        }
    }

    pub const fn as_geojson_name(self) -> &'static str {
        match self {
            Self::Wgs => "EPSG:4326",
            Self::Enu => "ENU",
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Geometry {
    Point(Point3),
    Segment(Segment3),
    Path(Path3),
    Polygon(Polygon3),
}

impl From<Point3> for Geometry {
    fn from(value: Point3) -> Self {
        Self::Point(value)
    }
}

impl From<Segment3> for Geometry {
    fn from(value: Segment3) -> Self {
        Self::Segment(value)
    }
}

impl From<Path3> for Geometry {
    fn from(value: Path3) -> Self {
        Self::Path(value)
    }
}

impl From<Polygon3> for Geometry {
    fn from(value: Polygon3) -> Self {
        Self::Polygon(value)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Feature {
    pub geometry: Geometry,
    pub properties: HashMap<String, String>,
}

impl Feature {
    pub fn new(geometry: impl Into<Geometry>, properties: HashMap<String, String>) -> Self {
        Self {
            geometry: geometry.into(),
            properties,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FeatureCollection {
    pub datum: Geo,
    pub heading: Heading,
    pub features: Vec<Feature>,
    pub global_properties: HashMap<String, String>,
}

impl FeatureCollection {
    pub fn new(datum: Geo, heading: Heading) -> Self {
        Self {
            datum,
            heading,
            features: Vec::new(),
            global_properties: HashMap::new(),
        }
    }
}

impl fmt::Display for FeatureCollection {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(
            f,
            "DATUM: {}, {}, {}",
            self.datum.latitude, self.datum.longitude, self.datum.altitude
        )?;
        writeln!(f, "HEADING: {}", self.heading.yaw)?;
        writeln!(f, "FEATURES: {}", self.features.len())?;

        for feature in &self.features {
            match &feature.geometry {
                Geometry::Polygon(_) => writeln!(f, "  POLYGON")?,
                Geometry::Segment(_) => writeln!(f, "  LINE")?,
                Geometry::Path(_) => writeln!(f, "  PATH")?,
                Geometry::Point(_) => writeln!(f, "   POINT")?,
            }
            if !feature.properties.is_empty() {
                writeln!(f, "    PROPS:{}", feature.properties.len())?;
            }
        }

        Ok(())
    }
}
