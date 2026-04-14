//! Rust port of the `vectkit` GeoJSON vector library.
//!
//! Current implementation focus:
//!
//! - GeoJSON parsing and writing
//! - WGS84 to ENU normalization through `concord`
//! - high-level `Vector` field API

#![deny(unsafe_op_in_unsafe_fn)]

pub mod core;
pub mod ffi;
pub mod io;
#[cfg(feature = "python")]
pub mod python;
pub mod types;
pub mod vector;

pub use core::{Error, Result};
pub use io::{
    ReadFeatureCollection, WriteFeatureCollection, WriteFeatureCollectionWgs, parse_geojson, read,
    read_feature_collection, read_json_str, to_json_string, write, write_feature_collection,
    write_wgs,
};
pub use types::{
    Crs, Feature, FeatureCollection, Geometry, Heading, Path3, Point3, Polygon3, Segment3,
};
pub use vector::{Element, Vector};

pub use crate as vk;
