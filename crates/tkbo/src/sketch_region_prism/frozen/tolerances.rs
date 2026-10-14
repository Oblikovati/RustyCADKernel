// SPDX-License-Identifier: GPL-2.0-only
//! Preserve stored metadata separately: the compatibility accessor floors invalid/sub-floor values.
use super::types::PrismFreezeCause;
use rustycad_tkbrep::{
    brep::access::{edge_tshape, face_tshape, vertex_tshape},
    topods::Shape,
};
use rustycad_tkg3d::top_abs::ShapeEnum;

pub(super) fn capture_stored_tolerance(actual: &Shape) -> Result<Option<f64>, PrismFreezeCause> {
    let member = "Martensite frozen prism raw tolerance";
    let value = match actual.shape_type().map_err(PrismFreezeCause::Native)? {
        ShapeEnum::Vertex => vertex_tshape(actual, member).map(|v| v.tolerance()),
        ShapeEnum::Edge => edge_tshape(actual, member).map(|e| e.tolerance()),
        ShapeEnum::Face => face_tshape(actual, member).map(|f| f.tolerance()),
        _ => return Ok(None),
    };
    value.map(Some).map_err(PrismFreezeCause::Native)
}
