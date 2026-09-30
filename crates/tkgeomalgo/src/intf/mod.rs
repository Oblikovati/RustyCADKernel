// Rust translation of ModelingAlgorithms/TKGeomAlgo/Intf at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! OCCT package Intf: interference (intersection points, section lines and tangent
//! zones) between polygons and polyhedra, the triangle helpers of the `Intf` class,
//! and `Tool`, the boxes of infinite curves in a domain.
//!
//! The generic polygon-polyhedron interference (Intf_InterferencePolygonPolyhedron.gxx)
//! is `InterferencePolygonPolyhedron`, generic over the `Polygon3dTool` and
//! `PolyhedronTool` traits; docs/geomalgo-design.md section 2 records the mapping.
mod interference;
mod interference_insert;
mod interference_polygon2d;
mod interference_polygon_polyhedron;
mod polygon2d;
mod section_line;
mod section_point;
mod sequence_access;
mod tangent_zone;
mod tool;

pub use interference::Interference;
pub use interference_polygon_polyhedron::{
    InterferencePolygonPolyhedron, Polygon3dTool, PolyhedronTool,
};
pub use interference_polygon2d::InterferencePolygon2d;
pub use polygon2d::Polygon2d;
pub use section_line::SectionLine;
pub use section_point::{PIType, SectionPoint};
pub use tangent_zone::TangentZone;
pub use tool::Tool;

use rustycad_tkmath::gp::{self, Point3, Xyz};

/// Intf::PlaneEquation: the unit normal of the triangle `p1 p2 p3` and its polar
/// distance (the normal dotted with `p1`).
///
/// The normal is the sum of the three edge cross products, as native sums them; for a
/// degenerate triangle (normal shorter than `gp::Resolution`) native leaves the
/// unnormalized normal and returns a polar distance of 0.
/// ```
/// use rustycad_tkgeomalgo::intf::plane_equation;
/// use rustycad_tkmath::gp::Point3;
/// let (normal, distance) = plane_equation(
///     Point3::new(0.0, 0.0, 1.0), Point3::new(1.0, 0.0, 1.0), Point3::new(0.0, 1.0, 1.0));
/// assert_eq!((normal.z(), distance), (1.0, 1.0));
/// ```
pub fn plane_equation(p1: Point3, p2: Point3, p3: Point3) -> (Xyz, f64) {
    let v1: Xyz = p2.xyz() - p1.xyz();
    let v2: Xyz = p3.xyz() - p2.xyz();
    let v3: Xyz = p1.xyz() - p3.xyz();
    let mut normal: Xyz = (v1 ^ v2) + (v2 ^ v3) + (v3 ^ v1);
    let length: f64 = normal.modulus();
    if length < gp::resolution() {
        return (normal, 0.0);
    }
    normal /= length;
    (normal, normal * p1.xyz())
}

/// Intf::Contain: whether `point` lies in the triangle `p1 p2 p3`, boundary included.
///
/// The three edge-to-point cross products must pairwise have non-negative dot products;
/// native uses no tolerance band, so a point exactly on an edge is inside.
/// ```
/// use rustycad_tkgeomalgo::intf::contain;
/// use rustycad_tkmath::gp::Point3;
/// let (a, b, c) = (Point3::new(0.0, 0.0, 0.0), Point3::new(1.0, 0.0, 0.0), Point3::new(0.0, 1.0, 0.0));
/// assert!(contain(a, b, c, Point3::new(0.25, 0.25, 0.0)));
/// ```
pub fn contain(p1: Point3, p2: Point3, p3: Point3, point: Point3) -> bool {
    let v1: Xyz = (p2.xyz() - p1.xyz()) ^ (point.xyz() - p1.xyz());
    let v2: Xyz = (p3.xyz() - p2.xyz()) ^ (point.xyz() - p2.xyz());
    let v3: Xyz = (p1.xyz() - p3.xyz()) ^ (point.xyz() - p3.xyz());
    v1 * v2 >= 0.0 && v2 * v3 >= 0.0 && v3 * v1 >= 0.0
}

/// A null interference tolerance becomes `Epsilon(1000.)`, as in every native
/// interference constructor.
pub(crate) fn nonzero_tolerance(tolerance: f64) -> f64 {
    if tolerance == 0.0 {
        rustycad_tkernel::standard::real::epsilon(1000.0)
    } else {
        tolerance
    }
}

/// `std::min(a, b)`: `b` when `b < a`, else `a` (unlike `f64::min`, NaN operands are
/// not skipped).
pub(crate) fn std_min(a: f64, b: f64) -> f64 {
    if b < a { b } else { a }
}

/// `std::max(a, b)`: `b` when `a < b`, else `a`.
pub(crate) fn std_max(a: f64, b: f64) -> f64 {
    if a < b { b } else { a }
}

/// `static_cast<int>(double)` as compiled for x86-64 (`cvttsd2si`): truncation toward
/// zero, NaN and out-of-range values (undefined behavior in ISO C++) giving `INT_MIN`;
/// the same rule as the private helper of rustycad_tkmath::bnd::sort_box.
pub(crate) fn native_int_cast(value: f64) -> i32 {
    if value.is_nan() || !(-2_147_483_648.0..2_147_483_648.0).contains(&value) {
        return i32::MIN;
    }
    value as i32
}
