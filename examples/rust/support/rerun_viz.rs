#![allow(clippy::collapsible_if)]

use std::error::Error;

use concord::{Geo, to_wgs_from_enu};
use datapod::{Point, Polygon};
use rerun::{Color, GeoLineStrings, LineStrips3D, RecordingStream, RecordingStreamBuilder};

pub fn connect(app_id: &str) -> Result<RecordingStream, Box<dyn Error>> {
    let url = std::env::var("RERUN_URL")
        .unwrap_or_else(|_| "rerun+http://0.0.0.0:9876/proxy".to_string());
    let rec = RecordingStreamBuilder::new(app_id).connect_grpc_opts(url)?;
    Ok(rec)
}

pub fn log_polygon(
    rec: &RecordingStream,
    path: &str,
    polygon: &Polygon,
    color: Color,
) -> Result<(), Box<dyn Error>> {
    let strip = close_points(
        polygon
            .vertices
            .iter()
            .map(|point| [point.x as f32, point.y as f32, point.z as f32])
            .collect(),
    );
    rec.log(path, &LineStrips3D::new([strip]).with_colors([color]))?;
    Ok(())
}

pub fn log_polygon_geo(
    rec: &RecordingStream,
    path: &str,
    polygon: &Polygon,
    datum: Geo,
    color: Color,
) -> Result<(), Box<dyn Error>> {
    let strip = close_geo_points(
        polygon
            .vertices
            .iter()
            .map(|point| point_geo(*point, datum))
            .collect(),
    );
    rec.log(
        path,
        &GeoLineStrings::from_lat_lon([strip]).with_colors([color]),
    )?;
    Ok(())
}

fn point_geo(point: Point, datum: Geo) -> [f64; 2] {
    let wgs = to_wgs_from_enu(concord::Enu::new(point.x, point.y, point.z, datum));
    [wgs.latitude, wgs.longitude]
}

fn close_points(mut points: Vec<[f32; 3]>) -> Vec<[f32; 3]> {
    if let Some(first) = points.first().copied() {
        if points.last().copied() != Some(first) {
            points.push(first);
        }
    }
    points
}

fn close_geo_points(mut points: Vec<[f64; 2]>) -> Vec<[f64; 2]> {
    if let Some(first) = points.first().copied() {
        if points.last().copied() != Some(first) {
            points.push(first);
        }
    }
    points
}
