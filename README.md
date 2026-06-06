# vectory

`vectory` is the Rust port of `vectkit`, a GeoJSON vector library for robotics
workflows. It reads and writes GeoJSON feature collections, normalizes geometry
into a local ENU frame, and provides a higher-level `Vector` API for field
boundaries and typed vector elements.

The crate uses:

- `concord` for WGS84 and ENU conversions
- `datapod` for shared geometry and spatial data types
- `serde_json` for GeoJSON parsing and serialization

## Scope

The current implementation covers the core `vectkit` library behavior:

- `Feature`, `FeatureCollection`, `Geometry`, and `Crs`
- GeoJSON parsing and writing
- WGS to ENU normalization on ingest
- WGS and ENU output
- the higher-level `Vector` abstraction
- a C ABI
- optional Python bindings

The crate is self-contained. It does not depend on a local `../vectkit`
checkout to build, test, or run.

## Installation

Add the crate to `Cargo.toml`:

```toml
[dependencies]
vectory = { path = "../vectory" }
```

Python bindings are optional:

```toml
[dependencies]
vectory = { path = "../vectory", features = ["python"] }
```

## Core Types

`vectory` re-exports the main public types from the crate root:

- `Crs`
- `Geometry`
- `Feature`
- `FeatureCollection`
- `Vector`
- `Element`
- `Heading`
- `Geo`
- `Point3`
- `Segment3`
- `Path3`
- `Polygon3`

Geometry storage is based on `datapod` types. Parsed geometries are stored
internally in local ENU-style coordinates regardless of input CRS.

## GeoJSON API

Read and write a feature collection:

```rust
use vectory::{Crs, read, write};

let collection = read("field.geojson")?;
write(&collection, "field_out.geojson", Crs::Wgs)?;
# Ok::<(), vectory::Error>(())
```

Read from an in-memory JSON string:

```rust
use vectory::read_json_str;

let json = r#"{
    "type": "FeatureCollection",
    "properties": {
        "crs": "ENU",
        "datum": [5.0, 52.0, 0.0],
        "heading": 0.0
    },
    "features": []
}"#;

let collection = read_json_str(json)?;
# Ok::<(), vectory::Error>(())
```

Serialize a collection to a JSON string:

```rust
use concord::Geo;
use vectory::{FeatureCollection, Heading, Crs, to_json_string};

let collection = FeatureCollection::new(Geo::new(52.0, 5.0, 0.0), Heading::default());
let json = to_json_string(&collection, Crs::Wgs)?;
# let _ = json;
# Ok::<(), vectory::Error>(())
```

## Vector API

`Vector` is the higher-level field-oriented abstraction. It stores:

- one field boundary polygon
- field properties
- a list of typed elements
- datum, heading, and CRS metadata
- global properties

Example:

```rust
use std::collections::HashMap;

use concord::Geo;
use datapod::{Point, Polygon, Vector as PodVector};
use vectory::{Crs, Heading, Vector};

let field = Polygon {
    vertices: PodVector::from(vec![
        Point::new(0.0, 0.0, 0.0),
        Point::new(10.0, 0.0, 0.0),
        Point::new(10.0, 10.0, 0.0),
        Point::new(0.0, 10.0, 0.0),
    ]),
};

let mut vector = Vector::new(field, Geo::new(52.0, 5.0, 0.0), Heading::default(), Crs::Enu);
vector.set_field_property("name", "demo");
vector.add_point(Point::new(1.0, 2.0, 0.0), "marker", HashMap::new());

vector.to_file("vector.geojson", Crs::Wgs)?;
let loaded = Vector::from_file("vector.geojson")?;

assert_eq!(loaded.points().len(), 1);
# Ok::<(), vectory::Error>(())
```

## Parsing Semantics

Important behavior inherited from `vectkit`:

- top-level GeoJSON must include `properties`
- required top-level properties are:
  - `crs`
  - `datum`
  - `heading`
- accepted CRS strings include `EPSG:4326`, `WGS84`, `WGS`, `ENU`, and `ECEF`
- `LineString` with 2 points becomes `Segment`
- longer `LineString` becomes `Path`
- `Polygon` uses the outer ring
- `MultiPoint`, `MultiLineString`, `MultiPolygon`, and `GeometryCollection`
  are flattened into multiple features
- unknown geometry types are skipped gracefully
- WGS input is converted to ENU internally

## C ABI

The crate exposes a C ABI and installs the public header:

- header: [include/vectory.h](include/vectory.h)
- implementation: [src/ffi.rs](src/ffi.rs)

The ABI covers:

- feature collection create/free/read/write/JSON conversion
- collection metadata and global properties
- vector create/free/read/write
- field boundary access
- element insertion and inspection
- last-error retrieval

A small C example is included in [examples/c_abi/demo.c](examples/c_abi/demo.c).

## Python Bindings

Python bindings are implemented with `pyo3` behind the `python` feature.

Relevant files:

- module implementation: [src/python/mod.rs](src/python/mod.rs)
- packaging config: [pyproject.toml](pyproject.toml)
- example scripts:
  - [examples/python_binding/basic.py](examples/python_binding/basic.py)
  - [examples/python_binding/main.py](examples/python_binding/main.py)

Build-time check:

```sh
cargo check --features python
```

## Examples

Rust examples:

- `cargo run --example main`
- `cargo run --example read_write`
- `cargo run --example vector_workflow`
- `cargo run --example json_roundtrip`
- `cargo run --example crs_conversion_example`
- `cargo run --example rerun_vis_from_wgs`
- `cargo run --example rerun_vis_from_enu`

C ABI example:

```sh
make c-demo
```

## Testing

Run the normal Rust test suite:

```sh
cargo test
```

Or through the project `Makefile`:

```sh
make test
```

Python binding compilation check:

```sh
make test-python
```

The Python feature currently compiles cleanly, but full `cargo test --features python`
can still depend on having the appropriate Python development libraries available
on the machine.

## Project Layout

- [src/lib.rs](src/lib.rs): crate exports
- [src/types.rs](src/types.rs): core data model
- [src/io/parse.rs](src/io/parse.rs): GeoJSON parser
- [src/io/write.rs](src/io/write.rs): GeoJSON writer
- [src/vector.rs](src/vector.rs): field-oriented API
- [src/ffi.rs](src/ffi.rs): C ABI
- [src/python/mod.rs](src/python/mod.rs): Python bindings
- [PLAN.md](PLAN.md): original porting plan

## Status

`vectory` is implemented as the Rust port of `vectkit` for the core library
surface. The crate includes Rust tests, fixture-based integration tests, C ABI
smoke tests, examples, and optional Python bindings.
