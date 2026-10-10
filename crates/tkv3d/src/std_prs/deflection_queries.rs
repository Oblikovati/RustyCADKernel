//! `StdPrs_DeflectionCurve::Match` overloads preserve automatic/ranged bound rules.
use super::{
    StdPrsDeflectionCurve,
    curve_limits::{explicit_limits, find_limits_bounded},
    deflection_match::matches_deflection,
    deflection_sampling::drawer_deflection,
};
use crate::prs3d::Prs3dDrawer;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::adaptor3d::Adaptor3dCurve;
use rustycad_tkmath::gp::Point3;
impl StdPrsDeflectionCurve {
    /// Match automatic bounds and drawer deviation; `StdPrsDeflectionCurve::matches(&p,0.1,&curve,&drawer)?`.
    pub fn matches(
        point: &Point3,
        distance: f64,
        curve: &dyn Adaptor3dCurve,
        drawer: &Handle<Prs3dDrawer>,
    ) -> Result<bool, Failure> {
        let Some(bounds) =
            find_limits_bounded(curve, drawer.maximal_parameter_value(), Some(100000))?
        else {
            return Ok(false);
        };
        matches_deflection(
            *point,
            distance,
            curve,
            bounds,
            drawer_deflection(curve, bounds, drawer)?,
            drawer.deviation_angle(),
        )
    }
    /// Match clamped explicit bounds; `StdPrsDeflectionCurve::matches_range(&p,0.1,&curve,0.,1.,&drawer)?`.
    pub fn matches_range(
        point: &Point3,
        distance: f64,
        curve: &dyn Adaptor3dCurve,
        first: f64,
        last: f64,
        drawer: &Handle<Prs3dDrawer>,
    ) -> Result<bool, Failure> {
        let bounds = explicit_limits([first, last], drawer.maximal_parameter_value());
        matches_deflection(
            *point,
            distance,
            curve,
            bounds,
            drawer_deflection(curve, bounds, drawer)?,
            drawer.deviation_angle(),
        )
    }
    /// Match explicit deviation with automatic limits; `StdPrsDeflectionCurve::matches_with_deflection(&p,0.1,&curve,0.01,100.,0.2)?`.
    pub fn matches_with_deflection(
        point: &Point3,
        distance: f64,
        curve: &dyn Adaptor3dCurve,
        deflection: f64,
        limit: f64,
        angle: f64,
    ) -> Result<bool, Failure> {
        let Some(bounds) = find_limits_bounded(curve, limit, Some(100000))? else {
            return Ok(false);
        };
        matches_deflection(*point, distance, curve, bounds, deflection, angle)
    }
    /// Match raw bounds and explicit deviation; `StdPrsDeflectionCurve::matches_range_with_deflection(&p,0.1,&curve,0.,1.,0.01,0.2)?`.
    pub fn matches_range_with_deflection(
        point: &Point3,
        distance: f64,
        curve: &dyn Adaptor3dCurve,
        first: f64,
        last: f64,
        deflection: f64,
        angle: f64,
    ) -> Result<bool, Failure> {
        matches_deflection(*point, distance, curve, [first, last], deflection, angle)
    }
}
