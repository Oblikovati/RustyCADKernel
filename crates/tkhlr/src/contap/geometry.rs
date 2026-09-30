// Rust support for the gp conversions the Contap sources use implicitly at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! Small conversions between the gp types that the native code writes as implicit
//! constructors (`gp_Pnt::SetXYZ`, `gp_Vec(gp_Dir)`, `gp_Vec(P1, P2)`).
use rustycad_tkmath::gp::{Point3, Xyz};

/// `gp_Pnt(XYZ)`: the point of the coordinates `xyz`; `point_of(Xyz::new(1.0, 2.0, 3.0))`.
pub(crate) fn point_of(xyz: Xyz) -> Point3 {
    Point3::new(xyz.x(), xyz.y(), xyz.z())
}
