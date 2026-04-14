# vectory

Rust port of `../vectkit`, following the translation style already used in
`../graphix_rs` and `../concord_rs`.

Current status: template crate aligned and migration plan written in `PLAN.md`.

Planned core dependencies:

- `concord` from `../concord_rs` for CRS and frame conversions
- `graphix` from `../graphix_rs` for graph/vector-oriented support where useful

The initial implementation target is GeoJSON parsing, writing, and the high-level
`Vector` API from `vectkit`.
