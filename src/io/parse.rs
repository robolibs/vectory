use std::{collections::HashMap, fs, path::Path};

use concord::{Enu, Geo, Wgs, to_enu};
use serde_json::{Map, Value};

use crate::{Crs, Error, Feature, FeatureCollection, Geometry, Heading, Point3, Result, Segment3};

pub fn read(path: impl AsRef<Path>) -> Result<FeatureCollection> {
    let path = path.as_ref();
    let text = fs::read_to_string(path).map_err(|source| Error::Io {
        path: path.to_path_buf(),
        source,
    })?;
    let value: Value = serde_json::from_str(&text)
        .map_err(|err| Error::InvalidGeoJson(format!("failed to parse JSON: {err}")))?;
    parse_geojson(value)
}

pub fn read_feature_collection(path: impl AsRef<Path>) -> Result<FeatureCollection> {
    read(path)
}

pub fn parse_geojson(value: Value) -> Result<FeatureCollection> {
    let object = value.as_object().ok_or_else(|| {
        Error::InvalidGeoJson("vectkit::ReadFeatureCollection(): top-level object has no string 'type' field".into())
    })?;

    let ty = object.get("type").and_then(Value::as_str).ok_or_else(|| {
        Error::InvalidGeoJson("vectkit::ReadFeatureCollection(): top-level object has no string 'type' field".into())
    })?;

    match ty {
        "FeatureCollection" => parse_feature_collection_object(object),
        _ => {
            let mut wrapped = Map::new();
            wrapped.insert("type".into(), Value::String("FeatureCollection".into()));
            wrapped.insert("features".into(), Value::Array(vec![Value::Object(object.clone())]));
            parse_feature_collection_object(&wrapped)
        }
    }
}

fn parse_feature_collection_object(object: &Map<String, Value>) -> Result<FeatureCollection> {
    let props = object
        .get("properties")
        .and_then(Value::as_object)
        .ok_or_else(|| Error::InvalidGeoJson("missing top-level 'properties'".into()))?;

    let crs = get_required_string(props, "crs", "'properties' missing string 'crs'")?;
    let datum = parse_datum(props.get("datum"))?;
    let heading = parse_heading(props.get("heading"))?;
    let input_crs = Crs::parse(crs)?;

    let mut output = FeatureCollection::new(datum, heading);
    for (key, value) in props {
        if key != "crs" && key != "datum" && key != "heading" {
            output
                .global_properties
                .insert(key.clone(), stringify_json(value)?);
        }
    }

    if let Some(features) = object.get("features").and_then(Value::as_array) {
        for feature in features {
            let Some(feature_object) = feature.as_object() else {
                continue;
            };

            let Some(geometry_value) = feature_object.get("geometry") else {
                continue;
            };
            if geometry_value.is_null() {
                continue;
            }

            let properties = parse_properties(
                feature_object.get("properties").and_then(Value::as_object),
            )?;
            for geometry in parse_geometry_value(geometry_value, datum, input_crs)? {
                output.features.push(Feature {
                    geometry,
                    properties: properties.clone(),
                });
            }
        }
    }

    Ok(output)
}

fn get_required_string<'a>(
    props: &'a Map<String, Value>,
    key: &str,
    message: &str,
) -> Result<&'a str> {
    props
        .get(key)
        .and_then(Value::as_str)
        .ok_or_else(|| Error::InvalidGeoJson(message.into()))
}

fn parse_datum(value: Option<&Value>) -> Result<Geo> {
    let Some(Value::Array(values)) = value else {
        return Err(Error::InvalidGeoJson(
            "'properties' missing array 'datum' of ≥3 numbers".into(),
        ));
    };
    if values.len() < 3 {
        return Err(Error::InvalidGeoJson(
            "'properties' missing array 'datum' of ≥3 numbers".into(),
        ));
    }

    let lon = as_f64(&values[0]).ok_or_else(|| {
        Error::InvalidGeoJson("'properties' missing array 'datum' of ≥3 numbers".into())
    })?;
    let lat = as_f64(&values[1]).ok_or_else(|| {
        Error::InvalidGeoJson("'properties' missing array 'datum' of ≥3 numbers".into())
    })?;
    let alt = as_f64(&values[2]).ok_or_else(|| {
        Error::InvalidGeoJson("'properties' missing array 'datum' of ≥3 numbers".into())
    })?;

    Ok(Geo::new(lat, lon, alt))
}

fn parse_heading(value: Option<&Value>) -> Result<Heading> {
    let Some(value) = value else {
        return Err(Error::InvalidGeoJson(
            "'properties' missing numeric 'heading'".into(),
        ));
    };
    let yaw = as_f64(value)
        .ok_or_else(|| Error::InvalidGeoJson("'properties' missing numeric 'heading'".into()))?;
    Ok(Heading::new(0.0, 0.0, yaw))
}

fn parse_properties(properties: Option<&Map<String, Value>>) -> Result<HashMap<String, String>> {
    let mut output = HashMap::new();
    let Some(properties) = properties else {
        return Ok(output);
    };

    for (key, value) in properties {
        output.insert(key.clone(), stringify_json(value)?);
    }
    Ok(output)
}

fn stringify_json(value: &Value) -> Result<String> {
    match value {
        Value::String(value) => Ok(value.clone()),
        other => serde_json::to_string(other)
            .map_err(|err| Error::InvalidGeoJson(format!("failed to serialize property value: {err}"))),
    }
}

fn parse_geometry_value(value: &Value, datum: Geo, crs: Crs) -> Result<Vec<Geometry>> {
    let Some(object) = value.as_object() else {
        return Ok(Vec::new());
    };
    let Some(ty) = object.get("type").and_then(Value::as_str) else {
        return Ok(Vec::new());
    };

    match ty {
        "Point" => {
            let Some(coords) = object.get("coordinates").and_then(Value::as_array) else {
                return Ok(Vec::new());
            };
            Ok(vec![Geometry::Point(parse_point(coords, datum, crs)?)])
        }
        "LineString" => {
            let Some(coords) = object.get("coordinates").and_then(Value::as_array) else {
                return Ok(Vec::new());
            };
            Ok(vec![parse_line_string(coords, datum, crs)?])
        }
        "Polygon" => {
            let Some(rings) = object.get("coordinates").and_then(Value::as_array) else {
                return Ok(Vec::new());
            };
            Ok(vec![Geometry::polygon(parse_polygon(rings, datum, crs)?)])
        }
        "MultiPoint" => {
            let Some(points) = object.get("coordinates").and_then(Value::as_array) else {
                return Ok(Vec::new());
            };
            points
                .iter()
                .map(|coords| {
                    coords
                        .as_array()
                        .ok_or_else(|| Error::InvalidGeoJson("Invalid point coordinates".into()))
                        .and_then(|coords| parse_point(coords, datum, crs))
                        .map(Geometry::Point)
                })
                .collect()
        }
        "MultiLineString" => {
            let Some(lines) = object.get("coordinates").and_then(Value::as_array) else {
                return Ok(Vec::new());
            };
            lines
                .iter()
                .map(|coords| {
                    coords
                        .as_array()
                        .ok_or_else(|| Error::InvalidGeoJson("Invalid line coordinates".into()))
                        .and_then(|coords| parse_line_string(coords, datum, crs))
                })
                .collect()
        }
        "MultiPolygon" => {
            let Some(polygons) = object.get("coordinates").and_then(Value::as_array) else {
                return Ok(Vec::new());
            };
            polygons
                .iter()
                .map(|rings| {
                    rings
                        .as_array()
                        .ok_or_else(|| Error::InvalidGeoJson("Invalid polygon coordinates".into()))
                        .and_then(|rings| parse_polygon(rings, datum, crs))
                        .map(Geometry::polygon)
                })
                .collect()
        }
        "GeometryCollection" => {
            let Some(geometries) = object.get("geometries").and_then(Value::as_array) else {
                return Ok(Vec::new());
            };
            let mut out = Vec::new();
            for geometry in geometries {
                out.extend(parse_geometry_value(geometry, datum, crs)?);
            }
            Ok(out)
        }
        _ => Ok(Vec::new()),
    }
}

fn parse_point(coords: &[Value], datum: Geo, crs: Crs) -> Result<Point3> {
    if coords.len() < 2 {
        return Err(Error::InvalidGeoJson("Invalid point coordinates".into()));
    }

    let x = as_f64(&coords[0]).ok_or_else(|| Error::InvalidGeoJson("Invalid point coordinates".into()))?;
    let y = as_f64(&coords[1]).ok_or_else(|| Error::InvalidGeoJson("Invalid point coordinates".into()))?;
    let has_z = coords.len() > 2;
    let z = coords.get(2).and_then(as_f64).unwrap_or(0.0);

    match crs {
        Crs::Enu => Ok(Point3::new(x, y, z)),
        Crs::Wgs => {
            let altitude = if has_z { z } else { datum.altitude };
            let enu = to_enu(datum, Wgs::new(y, x, altitude));
            let up = if has_z { enu.up() } else { z - datum.altitude };
            Ok(Point3::new(enu.east(), enu.north(), up))
        }
    }
}

fn parse_line_string(coords: &[Value], datum: Geo, crs: Crs) -> Result<Geometry> {
    let points = parse_points(coords, datum, crs)?;
    if points.len() == 2 {
        Ok(Geometry::Segment(Segment3::new(points[0], points[1])))
    } else {
        Ok(Geometry::path(points))
    }
}

fn parse_polygon(rings: &[Value], datum: Geo, crs: Crs) -> Result<Vec<Point3>> {
    let Some(exterior) = rings.first().and_then(Value::as_array) else {
        return Ok(Vec::new());
    };
    parse_points(exterior, datum, crs)
}

fn parse_points(coords: &[Value], datum: Geo, crs: Crs) -> Result<Vec<Point3>> {
    coords
        .iter()
        .map(|coord| {
            coord
                .as_array()
                .ok_or_else(|| Error::InvalidGeoJson("Invalid point coordinates".into()))
                .and_then(|coord| parse_point(coord, datum, crs))
        })
        .collect()
}

fn as_f64(value: &Value) -> Option<f64> {
    value.as_f64()
}

pub(crate) fn point_to_enu(point: &Point3, origin: Geo) -> Enu {
    Enu::new(point.x, point.y, point.z, origin)
}
