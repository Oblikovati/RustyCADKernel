// Rust translation of ModelingData/TKG3d/Geom/Geom_RevolutionUtils.pxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `Geom_RevolutionUtils`: the evaluation of a surface of revolution, the basis
//! curve point of parameter `v` rotated by `u` around the axis, shared by
//! Geom_SurfaceOfRevolution and the adaptors.
//!
//! The derivatives in U are computed in the frame of the basis curve (the axis
//! direction crossed with the vector from the axis), then everything is rotated
//! by the same gp_Trsf, as natively.
use super::basis_eval::BasisCurveEvaluator;
use super::surface::{GeomSurfaceD1, GeomSurfaceD2, GeomSurfaceD3};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::gp::{Axis1, Point3, Transform3, Vector3, Xyz};

// `gp_Trsf aRotation; aRotation.SetRotation(theAxis, theU)`.
fn rotation(axis: Axis1, u: f64) -> Result<Transform3, Failure> {
    let mut result = Transform3::default();
    result.set_rotation(axis, u)?;
    Ok(result)
}

fn rotated(mut vector: Vector3, rotation: Transform3) -> Vector3 {
    vector.transform(rotation);
    vector
}

// The first U derivative before rotation: `Dir ^ CQ`, reset to null below
// Precision::SquareConfusion() in square magnitude.
fn d1u_unrotated(dir: Xyz, cq: Xyz) -> Vector3 {
    let d1u = Vector3::from(dir.crossed(cq));
    if d1u.square_magnitude() < precision::square_confusion() {
        return Vector3::new(0.0, 0.0, 0.0);
    }
    d1u
}

// `(Dir . X) * Dir - X`: the second U derivative of a rotated vector X.
fn radial_second(dir: Xyz, x: Xyz) -> Xyz {
    dir.dot(x) * dir - x
}

/// `CalculateD0`: the curve point rotated by `u`; `calculate_d0(p, 0.5, axis)?`.
pub fn calculate_d0(curve_point: Point3, u: f64, axis: Axis1) -> Result<Point3, Failure> {
    let mut point: Point3 = curve_point;
    point.transform(rotation(axis, u)?);
    Ok(point)
}

/// `CalculateD1` from the curve point and first derivative;
/// `calculate_d1(p, d1, 0.5, axis)?`.
pub fn calculate_d1(
    curve_point: Point3,
    curve_d1: Vector3,
    u: f64,
    axis: Axis1,
) -> Result<GeomSurfaceD1, Failure> {
    let cq: Xyz = curve_point.xyz() - axis.location().xyz();
    let d1u: Vector3 = d1u_unrotated(axis.direction().xyz(), cq);
    let rotation: Transform3 = rotation(axis, u)?;
    let mut point: Point3 = curve_point;
    point.transform(rotation);
    Ok(GeomSurfaceD1 {
        point,
        d1u: rotated(d1u, rotation),
        d1v: rotated(curve_d1, rotation),
    })
}

/// `CalculateD2` from the curve point and two derivatives `[D1, D2]`;
/// `calculate_d2(p, [d1, d2], 0.5, axis)?`.
pub fn calculate_d2(
    curve_point: Point3,
    curve: [Vector3; 2],
    u: f64,
    axis: Axis1,
) -> Result<GeomSurfaceD2, Failure> {
    let cq: Xyz = curve_point.xyz() - axis.location().xyz();
    let dir: Xyz = axis.direction().xyz();
    let d1u: Vector3 = d1u_unrotated(dir, cq);
    let d2u = Vector3::from(radial_second(dir, cq));
    let d2uv = Vector3::from(dir.crossed(*curve[0].xyz()));
    let rotation: Transform3 = rotation(axis, u)?;
    let mut point: Point3 = curve_point;
    point.transform(rotation);
    Ok(GeomSurfaceD2 {
        point,
        d1u: rotated(d1u, rotation),
        d1v: rotated(curve[0], rotation),
        d2u: rotated(d2u, rotation),
        d2v: rotated(curve[1], rotation),
        d2uv: rotated(d2uv, rotation),
    })
}

/// `CalculateD3` from the curve point and three derivatives `[D1, D2, D3]`;
/// `calculate_d3(p, [d1, d2, d3], 0.5, axis)?`.
pub fn calculate_d3(
    curve_point: Point3,
    curve: [Vector3; 3],
    u: f64,
    axis: Axis1,
) -> Result<GeomSurfaceD3, Failure> {
    let second: GeomSurfaceD2 = calculate_d2(curve_point, [curve[0], curve[1]], u, axis)?;
    let cq: Xyz = curve_point.xyz() - axis.location().xyz();
    let dir: Xyz = axis.direction().xyz();
    let d3u: Vector3 = -d1u_unrotated(dir, cq);
    let d3uuv = Vector3::from(radial_second(dir, *curve[0].xyz()));
    let d3uvv = Vector3::from(dir.crossed(*curve[1].xyz()));
    let rotation: Transform3 = rotation(axis, u)?;
    Ok(GeomSurfaceD3 {
        point: second.point,
        d1u: second.d1u,
        d1v: second.d1v,
        d2u: second.d2u,
        d2v: second.d2v,
        d2uv: second.d2uv,
        d3u: rotated(d3u, rotation),
        d3v: rotated(curve[2], rotation),
        d3uuv: rotated(d3uuv, rotation),
        d3uvv: rotated(d3uvv, rotation),
    })
}

/// `CalculateDN`: the U derivative of order `der_u` of the rotated vector (the curve
/// derivative in V, or the vector from the axis origin for `der_v == 0`), cycling
/// with period 4; `calculate_dn(x, 0.5, axis, 2, 1)?`.
pub fn calculate_dn(
    curve_point_or_dn: Vector3,
    u: f64,
    axis: Axis1,
    der_u: i32,
    _der_v: i32,
) -> Result<Vector3, Failure> {
    let rotation: Transform3 = rotation(axis, u)?;
    let x: Xyz = *curve_point_or_dn.xyz();
    let dir: Xyz = axis.direction().xyz();
    let result: Vector3 = match (der_u, der_u % 4) {
        (0, _) => curve_point_or_dn,
        (_, 1) => Vector3::from(dir.crossed(x)),
        (_, 2) => Vector3::from(radial_second(dir, x)),
        (_, 3) => Vector3::from(dir.crossed(x)) * (-1.0),
        _ => Vector3::from(x - dir.dot(x) * dir),
    };
    Ok(rotated(result, rotation))
}

/// `D0<CurveType>`: the basis point of parameter `v` rotated by `u`;
/// `d0(u, v, &*basis, axis)?`.
pub fn d0<C: BasisCurveEvaluator + ?Sized>(
    u: f64,
    v: f64,
    basis: &C,
    axis: Axis1,
) -> Result<Point3, Failure> {
    calculate_d0(basis.basis_d0(v)?, u, axis)
}

/// `D1<CurveType>`; `d1(u, v, &*basis, axis)?`.
pub fn d1<C: BasisCurveEvaluator + ?Sized>(
    u: f64,
    v: f64,
    basis: &C,
    axis: Axis1,
) -> Result<GeomSurfaceD1, Failure> {
    let curve = basis.basis_d1(v)?;
    calculate_d1(curve.point, curve.d1, u, axis)
}

/// `D2<CurveType>`; `d2(u, v, &*basis, axis)?`.
pub fn d2<C: BasisCurveEvaluator + ?Sized>(
    u: f64,
    v: f64,
    basis: &C,
    axis: Axis1,
) -> Result<GeomSurfaceD2, Failure> {
    let curve = basis.basis_d2(v)?;
    calculate_d2(curve.point, [curve.d1, curve.d2], u, axis)
}

/// `D3<CurveType>`; `d3(u, v, &*basis, axis)?`.
pub fn d3<C: BasisCurveEvaluator + ?Sized>(
    u: f64,
    v: f64,
    basis: &C,
    axis: Axis1,
) -> Result<GeomSurfaceD3, Failure> {
    let curve = basis.basis_d3(v)?;
    calculate_d3(curve.point, [curve.d1, curve.d2, curve.d3], u, axis)
}

/// `DN<CurveType>`: the basis derivative of order `der_v`, or for `der_v == 0` the
/// vector from the axis origin to the basis point, then CalculateDN;
/// `dn(u, v, &*basis, axis, 1, 1)?`.
pub fn dn<C: BasisCurveEvaluator + ?Sized>(
    u: f64,
    v: f64,
    basis: &C,
    axis: Axis1,
    der: (i32, i32),
) -> Result<Vector3, Failure> {
    let (der_u, der_v) = der;
    let curve_point_or_dn: Vector3 = if der_u != 0 && der_v == 0 {
        let point: Point3 = basis.basis_d0(v)?;
        Vector3::from(point.xyz() - axis.location().xyz())
    } else {
        basis.basis_dn(v, der_v)?
    };
    calculate_dn(curve_point_or_dn, u, axis, der_u, der_v)
}
