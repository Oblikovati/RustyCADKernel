//! OCCT 3d097a0 matches the chordal polygon, with its distinct circle stepping rule.
use super::curve_match::{match_curve, near_point, near_segment};
use rustycad_tkernel::{precision, standard::Failure};
use rustycad_tkg3d::adaptor3d::Adaptor3dCurve;
use rustycad_tkgeombase::gcpnts::{TangentialCriteria, TangentialDeflection};
use rustycad_tkmath::{geom_abs::CurveType, gp::Point3};
pub(super) fn matches_deflection(
    query: Point3,
    distance: f64,
    curve: &dyn Adaptor3dCurve,
    bounds: [f64; 2],
    deflection: f64,
    angle: f64,
) -> Result<bool, Failure> {
    let kind = curve.get_type()?;
    if kind == CurveType::Circle && precision::is_infinite(curve.circle()?.radius()) {
        return Ok(false);
    }
    if matches!(kind, CurveType::Circle | CurveType::Line) {
        return match_curve(query, distance, curve, deflection, 2, bounds);
    }
    let distribution = TangentialDeflection::with_range(
        curve,
        bounds[0],
        bounds[1],
        TangentialCriteria::new(angle, deflection),
    )?;
    let mut previous = None;
    for index in 1..=distribution.nb_points() {
        let point = distribution.value(index)?;
        if near_point(query, point, distance)
            || previous.is_some_and(|p| near_segment(query, p, point, distance))
        {
            return Ok(true);
        }
        previous = Some(point);
    }
    Ok(false)
}
