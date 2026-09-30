// Rust translation of the static FDeriv and DDeriv helpers that GeomFill_Frenet.cxx,
// GeomFill_ConstantBiNormal.cxx, GeomFill_DraftTrihedron.cxx and GeomFill_Darboux.cxx
// each repeat, at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The derivatives of a normalized vector field that several trihedron laws repeat
//! natively (one copy here), and the `gp_Pnt(gp_XYZ)` conversion.
use rustycad_tkmath::gp::{Point3, Vector3, Xyz};

/// `FDeriv(F, DF)`: the derivative of `F / |F|`;
/// `(DF - F * (F * DF) / (|F| * |F|)) / |F|`.
pub(crate) fn fderiv(f: Vector3, df: Vector3) -> Vector3 {
    let norma = f.magnitude();
    (df - f * (f * df) / (norma * norma)) / norma
}

/// `DDeriv(F, DF, D2F)`: the second derivative of `F / |F|`, in the native
/// operation order.
pub(crate) fn dderiv(f: Vector3, df: Vector3, d2f: Vector3) -> Vector3 {
    let norma = f.magnitude();
    (d2f - 2.0 * df * (f * df) / (norma * norma)) / norma
        - f * ((df.square_magnitude() + f * d2f - 3.0 * (f * df) * (f * df) / (norma * norma))
            / (norma * norma * norma))
}

/// `gp_Pnt(XYZ)`; `point_from_xyz(Xyz::new(1.0, 2.0, 3.0))`.
pub(crate) fn point_from_xyz(xyz: Xyz) -> Point3 {
    Point3::new(xyz.x(), xyz.y(), xyz.z())
}

/// `std::min(a, b)`: `b < a ? b : a` (the first argument on ties and NaN).
pub(crate) fn std_min(a: f64, b: f64) -> f64 {
    if b < a { b } else { a }
}

/// `std::max(a, b)`: `a < b ? b : a` (the first argument on ties and NaN).
pub(crate) fn std_max(a: f64, b: f64) -> f64 {
    if a < b { b } else { a }
}
