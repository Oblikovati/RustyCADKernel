// Rust translation of FoundationClasses/TKMath/Bnd/Bnd_Tools.hxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `Bnd_Tools`: conversion of Bnd boxes to BVH boxes.
use super::{BoundingBox2, BoundingBox3};
use crate::bvh::{BvhBox, BvhVec2d, BvhVec3d};
use rustycad_tkernel::standard::Failure;

/// `Bnd_Tools::Bnd2BVH(const Bnd_Box2d&)`: copies the gap-adjusted limits (±1e100 on open
/// sides) into BVH corners; a void box raises `Standard_ConstructionError` like `Get`.
/// Example: `bnd::box2_to_bvh(&bounds)?.corner_min()`.
pub fn box2_to_bvh(bounds: &BoundingBox2) -> Result<BvhBox<BvhVec2d>, Failure> {
    let [x_min, y_min, x_max, y_max] = bounds.get()?;
    Ok(BvhBox::from_corners(
        BvhVec2d::new(x_min, y_min),
        BvhVec2d::new(x_max, y_max),
    ))
}

/// `Bnd_Tools::Bnd2BVH(const Bnd_Box&)`: copies the gap-adjusted limits (±1e100 on open
/// sides) into BVH corners; a void box raises `Standard_ConstructionError` like `Get`.
/// Example: `bnd::box3_to_bvh(&bounds)?.corner_max()`.
pub fn box3_to_bvh(bounds: &BoundingBox3) -> Result<BvhBox<BvhVec3d>, Failure> {
    let [x_min, y_min, z_min, x_max, y_max, z_max] = bounds.get()?;
    Ok(BvhBox::from_corners(
        BvhVec3d::new(x_min, y_min, z_min),
        BvhVec3d::new(x_max, y_max, z_max),
    ))
}
