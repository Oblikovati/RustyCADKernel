//! A sampling helper over the public GCPnts_TangentialDeflection that the numeric
//! probe (examples/numeric_probe.rs) compares with OCCT.
use crate::gcpnts::{TCurveTypes, TangentialCriteria, TangentialDeflection};
use rustycad_tkernel::standard::Failure;
use rustycad_tkg2d::adaptor2d::Adaptor2dCurve2d;
use rustycad_tkg3d::adaptor3d::Adaptor3dCurve;
use rustycad_tkmath::gp::Point3;

/// The parameters and points of one GCPnts_TangentialDeflection run.
pub type DeflectionSampling = (Vec<f64>, Vec<Point3>);

/// `GCPnts_TangentialDeflection::Initialize` on a 2D adaptor, over `range` or the curve
/// bounds; `criteria` is `[angular, curvature, minimum_of_points, u_tol, min_len]`.
pub fn tangential_deflection_2d(
    curve: &dyn Adaptor2dCurve2d,
    range: Option<(f64, f64)>,
    criteria: [f64; 5],
) -> Result<DeflectionSampling, Failure> {
    sampling(curve, range, criteria)
}

/// `GCPnts_TangentialDeflection::Initialize` on a 3D adaptor; see
/// [`tangential_deflection_2d`].
pub fn tangential_deflection_3d(
    curve: &dyn Adaptor3dCurve,
    range: Option<(f64, f64)>,
    criteria: [f64; 5],
) -> Result<DeflectionSampling, Failure> {
    sampling(curve, range, criteria)
}

fn sampling<C: TCurveTypes + ?Sized>(
    curve: &C,
    range: Option<(f64, f64)>,
    [angular, curvature, points, u_tol, min_len]: [f64; 5],
) -> Result<DeflectionSampling, Failure> {
    let criteria = TangentialCriteria {
        angular_deflection: angular,
        curvature_deflection: curvature,
        minimum_of_points: points as i32,
        u_tol,
        min_len,
    };
    let result = match range {
        Some((first, last)) => TangentialDeflection::with_range(curve, first, last, criteria)?,
        None => TangentialDeflection::with_curve(curve, criteria)?,
    };
    let count: i32 = result.nb_points();
    let params = (1..=count)
        .map(|i| result.parameter(i))
        .collect::<Result<_, _>>()?;
    let points = (1..=count)
        .map(|i| result.value(i))
        .collect::<Result<_, _>>()?;
    Ok((params, points))
}
