//! `Graphic3d_BndBox3d`, `Graphic3d_BndBox4d` and `Graphic3d_BndBox4f`: the BVH boxes the
//! presentation structures and groups keep their bounds in, and the corner transform the
//! flipper and the transform persistence apply to them.
use super::Graphic3dCameraScalar;
use rustycad_tkernel::ncollection::{Mat4, Vec3, Vec4};
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::bnd::BoundingBox3;
use rustycad_tkmath::bvh::{BvhBox, BvhVec3d, BvhVec4d, BvhVec4f, BvhVector};
use std::ops::Div;

/// `Graphic3d_BndBox3d` (`BVH_Box<double, 3>`); `Graphic3dBndBox3d::default().is_valid() == false`.
pub type Graphic3dBndBox3d = BvhBox<BvhVec3d>;
/// `Graphic3d_BndBox4d` (`BVH_Box<double, 4>`); `Graphic3dBndBox4d::default()`.
pub type Graphic3dBndBox4d = BvhBox<BvhVec4d>;
/// `Graphic3d_BndBox4f` (`BVH_Box<float, 4>`); `Graphic3dBndBox4f::default()`.
pub type Graphic3dBndBox4f = BvhBox<BvhVec4f>;

/// Replaces `bounds` by the box of its eight corners transformed by `matrix` (each divided
/// by its w), in the native corner order.
pub(crate) fn transform_box_corners<T: Graphic3dCameraScalar>(
    matrix: &Mat4<T>,
    bounds: &mut BvhBox<Vec3<T>>,
) where
    Vec4<T>: Div<T, Output = Vec4<T>>,
    Vec3<T>: BvhVector,
{
    let (min, max) = (*bounds.corner_min(), *bounds.corner_max());
    let one = T::from_f64(1.0);
    let corners = [
        (min.x, min.y, min.z),
        (min.x, min.y, max.z),
        (min.x, max.y, min.z),
        (min.x, max.y, max.z),
        (max.x, min.y, min.z),
        (max.x, min.y, max.z),
        (max.x, max.y, min.z),
        (max.x, max.y, max.z),
    ];
    bounds.clear();
    for (x, y, z) in corners {
        let corner = *matrix * Vec4::new(x, y, z, one);
        let corner = corner / corner.w;
        bounds.add(&Vec3::new(corner.x, corner.y, corner.z));
    }
}

/// Runs `apply` on the BVH box of a non-void `Bnd_Box`, then stores the result back (a fresh
/// box updated with the corners), as the native `Apply(..., Bnd_Box&)` members do.
pub(crate) fn apply_on_bnd_box(
    bounds: &mut BoundingBox3,
    apply: impl FnOnce(&mut BvhBox<Vec3<f64>>) -> Result<(), Failure>,
) -> Result<(), Failure> {
    if bounds.is_void() {
        return Ok(());
    }
    let [x0, y0, z0, x1, y1, z1] = bounds.get()?;
    let mut bvh = BvhBox::from_corners(Vec3::new(x0, y0, z0), Vec3::new(x1, y1, z1));
    apply(&mut bvh)?;
    let (min, max) = (bvh.corner_min(), bvh.corner_max());
    *bounds = BoundingBox3::default();
    bounds.update_bounds([min.x, min.y, min.z], [max.x, max.y, max.z]);
    Ok(())
}
