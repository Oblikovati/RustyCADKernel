// Rust translation of the SetTrsf and GetAverageLaw bodies that
// ModelingAlgorithms/TKGeomAlgo/GeomFill/GeomFill_CurveAndTrihedron.cxx,
// GeomFill_LocationDraft.cxx and GeomFill_LocationGuide.cxx repeat, at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The steps the location laws repeat natively, written once: whether a
//! transformation differs from the identity (`SetTrsf`), the mean placement of a law
//! (`GetAverageLaw`) and the matrix of a frame.
use super::location_law::LocationD0;
use super::trihedron_law::TrihedronD0;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::adaptor3d::Adaptor3dCurve;
use rustycad_tkmath::gp::{Matrix3, Vector3};

/// `WithTrans` of `SetTrsf(Transfo)`: some coefficient of `I - Transfo` exceeds
/// 1e-14 in absolute value.
pub(crate) fn differs_from_identity(trans: &Matrix3) -> bool {
    let mut aux = Matrix3::default();
    aux.set_identity();
    aux -= *trans;
    aux.coefficients()
        .iter()
        .flatten()
        .any(|c| c.abs() > 1.0e-14)
}

/// `M.SetCols(a, b, c)`; `columns(n, b, t)`.
pub(crate) fn columns(a: Vector3, b: Vector3, c: Vector3) -> Matrix3 {
    let mut m = Matrix3::default();
    m.set_columns(*a.xyz(), *b.xyz(), *c.xyz());
    m
}

/// `GetAverageLaw(AM, AV)`: the matrix of the mean frame of the trihedron law
/// (tangent, normal, binormal as columns) and the mean of eleven points of the
/// trimmed path.
pub(crate) fn average_placement(
    frame: TrihedronD0,
    trimmed: &dyn Adaptor3dCurve,
) -> Result<LocationD0, Failure> {
    let am = columns(frame.tangent, frame.normal, frame.binormal);
    let mut av = Vector3::new(0.0, 0.0, 0.0);
    let delta = (trimmed.last_parameter()? - trimmed.first_parameter()?) / 10.0;
    let mut u = trimmed.first_parameter()?;
    for _ in 0..=10 {
        av += Vector3::from(trimmed.value(u)?.xyz());
        u += delta;
    }
    av /= 11.0;
    Ok(LocationD0 { m: am, v: av })
}
