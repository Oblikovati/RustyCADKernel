// Rust translation of AdjustToSeam and SeamPosition (NIZNHY-PKV, 15 Sep 2011) of
// ModelingAlgorithms/TKGeomAlgo/IntPatch/IntPatch_ImpImpIntersection.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The frames of the circles of the result, turned so that their origin is on the
//! seam of the quadric.
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::gp::{Axis2, Axis3, Circle3, Point3, Sphere3};

/// `SeamPosition(aPLoc, aPos, aSeamPos)`: the frame at `loc` with the main and X
/// directions of the quadric.
fn seam_position(loc: Point3, pos: &Axis3) -> Result<Axis2, Failure> {
    Axis2::with_x_direction(loc, pos.direction(), pos.x_direction())
}

/// `AdjustToSeam(aQuad, aCirc)` of a cylinder, cone or torus: the circle takes the seam
/// frame at its center.
pub(super) fn adjust_to_seam(position: &Axis3, circ: &mut Circle3) -> Result<(), Failure> {
    circ.set_position(seam_position(circ.location(), position)?);
    Ok(())
}

/// `AdjustToSeam(aQuad, aCirc, aTolAng)` of a sphere: only a circle whose axis is
/// parallel to the sphere axis.
pub(super) fn adjust_to_sphere_seam(
    quad: &Sphere3,
    circ: &mut Circle3,
    tol_ang: f64,
) -> Result<(), Failure> {
    let pos: Axis3 = quad.position();
    if circ
        .axis()
        .direction()
        .is_parallel(pos.axis().direction(), tol_ang)
    {
        circ.set_position(seam_position(circ.location(), &pos)?);
    }
    Ok(())
}
