use std::{fs, path::Path};

use concord::to_wgs_from_enu;
use serde_json::{Map, Value, json};

use crate::{Crs, Error, Feature, FeatureCollection, Geometry, Point3, Result};

pub fn write(collection: &FeatureCollection, path: impl AsRef<Path>, crs: Crs) -> Result<()> {
    let text = to_json_string(collection, crs)?;
    let path = path.as_ref();
    fs::write(path, format!("{text}\n")).map_err(|source| Error::Io {
        path: path.to_path_buf(),
        source,
    })
}

pub fn write_wgs(collection: &FeatureCollection, path: impl AsRef<Path>) -> Result<()> {
    write(collection, path, Crs::Wgs)
}

pub fn write_feature_collection(
    collection: &FeatureCollection,
    path: impl AsRef<Path>,
    crs: Crs,
) -> Result<()> {
    write(collection, path, crs)
}

pub fn to_json_string(collection: &FeatureCollection, crs: Crs) -> Result<String> {
    serde_json::to_string(&feature_collection_to_json(collection, crs))
        .map_err(|err| Error::InvalidGeoJson(format!("failed to serialize GeoJSON: {err}")))
}

pub(crate) fn feature_collection_to_json(collection: &FeatureCollection, crs: Crs) -> Value {
    let mut top_properties = Map::new();
    top_properties.insert("crs".into(), Value::String(crs.as_geojson_name().into()));
    top_properties.insert(
        "datum".into(),
        json!([
            collection.datum.longitude,
            collection.datum.latitude,
            collection.datum.altitude
        ]),
    );
    top_properties.insert("heading".into(), json!(collection.heading.yaw));
    for (key, value) in &collection.global_properties {
        top_properties.insert(key.clone(), Value::String(value.clone()));
    }

    let features: Vec<Value> = collection
        .features
        .iter()
        .map(|feature| feature_to_json(feature, collection.datum, crs))
        .collect();

    json!({
        "type": "FeatureCollection",
        "properties": Value::Object(top_properties),
        "features": features,
    })
}

fn feature_to_json(feature: &Feature, origin: concord::Geo, crs: Crs) -> Value {
    let mut properties = Map::new();
    for (key, value) in &feature.properties {
        properties.insert(key.clone(), Value::String(value.clone()));
    }

    json!({
        "type": "Feature",
        "properties": Value::Object(properties),
        "geometry": geometry_to_json(&feature.geometry, origin, crs),
    })
}

fn geometry_to_json(geometry: &Geometry, origin: concord::Geo, crs: Crs) -> Value {
    match geometry {
        Geometry::Point(point) => json!({
            "type": "Point",
            "coordinates": point_to_coords(point, origin, crs),
        }),
        Geometry::Segment(segment) => json!({
            "type": "LineString",
            "coordinates": [
                point_to_coords(&segment.start, origin, crs),
                point_to_coords(&segment.end, origin, crs)
            ],
        }),
        Geometry::Path(path) => json!({
            "type": "LineString",
            "coordinates": path.points.iter().map(|p| point_to_coords(p, origin, crs)).collect::<Vec<_>>(),
        }),
        Geometry::Polygon(poly) => json!({
            "type": "Polygon",
            "coordinates": [poly.vertices.iter().map(|p| point_to_coords(p, origin, crs)).collect::<Vec<_>>()],
        }),
    }
}

fn point_to_coords(point: &Point3, origin: concord::Geo, crs: Crs) -> Vec<f64> {
    match crs {
        Crs::Enu => vec![point.x, point.y, point.z],
        Crs::Wgs => {
            let wgs = to_wgs_from_enu(crate::io::parse::point_to_enu(point, origin));
            vec![wgs.longitude, wgs.latitude, wgs.altitude.round()]
        }
    }
}
