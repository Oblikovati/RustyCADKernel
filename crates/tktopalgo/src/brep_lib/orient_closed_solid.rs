// Rust translation of BRepLib::OrientClosedSolid (BRepLib.cxx) at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `BRepLib::OrientClosedSolid`: the material of a closed solid set inside.
use crate::brep_class3d::BRepClass3dSolidClassifier;
use rustycad_tkbrep::topods::Solid;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::top_abs::State;

/// `BRepLib::OrientClosedSolid(solid)`: classifies the infinite point against the solid
/// at Precision::Confusion(); reverses the solid when the point is IN, and returns false
/// (the solid unchanged) when it is ON or UNKNOWN;
/// `brep_lib::orient_closed_solid(&mut solid)?`.
pub fn orient_closed_solid(solid: &mut Solid) -> Result<bool, Failure> {
    // Set material inside the solid
    let mut classifier = BRepClass3dSolidClassifier::with_shape(solid)?;
    classifier.perform_infinite_point(precision::confusion())?;
    match classifier.state() {
        State::In => solid.reverse(),
        State::On | State::Unknown => return Ok(false),
        State::Out => {}
    }
    Ok(true)
}
