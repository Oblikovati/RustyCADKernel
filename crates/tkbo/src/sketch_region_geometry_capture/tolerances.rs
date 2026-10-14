// SPDX-License-Identifier: GPL-2.0-only
//! Preserve stored metadata separately: the compatibility accessor floors invalid/sub-floor values.
use rustycad_tkbrep::{
    brep::access::{edge_tshape, face_tshape, vertex_tshape},
    topods::Shape,
};
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::top_abs::ShapeEnum;

/// Read stored metadata without the compatibility floor; example: `capture_raw_entity_tolerance(shape, "inspection")`.
pub(crate) fn capture_raw_entity_tolerance(
    actual: &Shape,
    member: &str,
) -> Result<Option<f64>, Failure> {
    let value = match actual.shape_type()? {
        ShapeEnum::Vertex => vertex_tshape(actual, member).map(|v| v.tolerance()),
        ShapeEnum::Edge => edge_tshape(actual, member).map(|e| e.tolerance()),
        ShapeEnum::Face => face_tshape(actual, member).map(|f| f.tolerance()),
        _ => return Ok(None),
    };
    value.map(Some)
}
