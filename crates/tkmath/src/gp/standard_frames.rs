// Rust translation of FoundationClasses/TKMath/gp/gp.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The `gp` class statics: the resolution and the standard points, directions and frames.
//! Native returns references to function-local statics; these values are `Copy`.
use super::{
    Axis1, Axis2, Axis2d, Direction2, Direction3, Point2, Point3, StandardDirection2,
    StandardDirection3,
};
/// gp::Resolution, `RealSmall()` = `DBL_MIN`; `gp::resolution()`.
pub const fn resolution() -> f64 {
    f64::MIN_POSITIVE
}
/// gp::Origin; `gp::origin()`.
pub const fn origin() -> Point3 {
    Point3::new(0.0, 0.0, 0.0)
}
/// gp::DX; `gp::dx()`.
pub const fn dx() -> Direction3 {
    Direction3::from_standard(StandardDirection3::X)
}
/// gp::DY; `gp::dy()`.
pub const fn dy() -> Direction3 {
    Direction3::from_standard(StandardDirection3::Y)
}
/// gp::DZ; `gp::dz()`.
pub const fn dz() -> Direction3 {
    Direction3::from_standard(StandardDirection3::Z)
}
/// gp::OX; `gp::ox()`.
pub const fn ox() -> Axis1 {
    Axis1::with_standard(origin(), StandardDirection3::X)
}
/// gp::OY; `gp::oy()`.
pub const fn oy() -> Axis1 {
    Axis1::with_standard(origin(), StandardDirection3::Y)
}
/// gp::OZ; `gp::oz()`.
pub const fn oz() -> Axis1 {
    Axis1::with_standard(origin(), StandardDirection3::Z)
}
/// gp::XOY: main direction Z, X direction X; `gp::xoy()`.
pub const fn xoy() -> Axis2 {
    Axis2::with_standard(origin(), StandardDirection3::Z)
}
/// gp::ZOX: main direction Y, so the X direction is Z and the Y direction is X; `gp::zox()`.
pub const fn zox() -> Axis2 {
    Axis2::with_standard(origin(), StandardDirection3::Y)
}
/// gp::YOZ: main direction X, X direction Y and Y direction Z; `gp::yoz()`.
pub const fn yoz() -> Axis2 {
    Axis2::with_standard(origin(), StandardDirection3::X)
}
/// gp::Origin2d; `gp::origin2d()`.
pub const fn origin2d() -> Point2 {
    Point2::new(0.0, 0.0)
}
/// gp::DX2d; `gp::dx2d()`.
pub const fn dx2d() -> Direction2 {
    Direction2::from_standard(StandardDirection2::X)
}
/// gp::DY2d; `gp::dy2d()`.
pub const fn dy2d() -> Direction2 {
    Direction2::from_standard(StandardDirection2::Y)
}
/// gp::OX2d; `gp::ox2d()`.
pub const fn ox2d() -> Axis2d {
    Axis2d::with_standard(origin2d(), StandardDirection2::X)
}
/// gp::OY2d; `gp::oy2d()`.
pub const fn oy2d() -> Axis2d {
    Axis2d::with_standard(origin2d(), StandardDirection2::Y)
}
