//! Complete `StdPrs_DeflectionCurve` drawing family from OCCT 3d097a0.
use super::{
    curve_limits::{explicit_limits, find_limits_bounded},
    deflection_sampling::{draw_deflection, drawer_deflection},
};
use crate::prs3d::{Prs3dArrow, Prs3dDrawer};
use rustycad_tkernel::{
    ncollection::Sequence,
    standard::{Failure, Handle},
};
use rustycad_tkg3d::adaptor3d::Adaptor3dCurve;
use rustycad_tkmath::gp::{Direction3, Point3};
use rustycad_tkservice::graphic3d::*;
/// Adaptive chordal-deviation presentations; `StdPrsDeflectionCurve::add(&prs,&curve,&drawer,true)?`.
pub struct StdPrsDeflectionCurve;
impl StdPrsDeflectionCurve {
    /// Draw automatic curve bounds with drawer aspects; `StdPrsDeflectionCurve::add(&prs,&curve,&drawer,true)?`.
    pub fn add(
        prs: &Handle<dyn Graphic3dStructureSource>,
        curve: &dyn Adaptor3dCurve,
        drawer: &Handle<Prs3dDrawer>,
        draw: bool,
    ) -> Result<(), Failure> {
        let group = styled_group(prs, drawer, draw)?;
        let Some(bounds) =
            find_limits_bounded(curve, drawer.maximal_parameter_value(), Some(100000))?
        else {
            return Ok(());
        };
        draw_styled(curve, &group, bounds, drawer)
    }
    /// Draw explicit bounds with infinite-sentinel clamping; `StdPrsDeflectionCurve::add_range(&prs,&curve,0.,1.,&drawer,true)?`.
    pub fn add_range(
        prs: &Handle<dyn Graphic3dStructureSource>,
        curve: &dyn Adaptor3dCurve,
        first: f64,
        last: f64,
        drawer: &Handle<Prs3dDrawer>,
        draw: bool,
    ) -> Result<(), Failure> {
        let group = styled_group(prs, drawer, draw)?;
        let bounds = explicit_limits([first, last], drawer.maximal_parameter_value());
        draw_styled(curve, &group, bounds, drawer)
    }
    /// Draw with explicit deviation and automatic limits, preserving aspects; `StdPrsDeflectionCurve::add_with_deflection(&prs,&curve,0.01,100.,0.2,true)?`.
    pub fn add_with_deflection(
        prs: &Handle<dyn Graphic3dStructureSource>,
        curve: &dyn Adaptor3dCurve,
        deflection: f64,
        limit: f64,
        angle: f64,
        draw: bool,
    ) -> Result<(), Failure> {
        let Some(bounds) = find_limits_bounded(curve, limit, Some(100000))? else {
            return Ok(());
        };
        draw_deflection(
            curve,
            &optional_group(prs, draw)?,
            bounds,
            deflection,
            angle,
            &mut Sequence::default(),
        )
    }
    /// Append points with the drawer's limit and angle; `StdPrsDeflectionCurve::add_points(&prs,&curve,0.01,&drawer,&mut points,false)?`.
    pub fn add_points(
        prs: &Handle<dyn Graphic3dStructureSource>,
        curve: &dyn Adaptor3dCurve,
        deflection: f64,
        drawer: &Handle<Prs3dDrawer>,
        points: &mut Sequence<Point3>,
        draw: bool,
    ) -> Result<(), Failure> {
        let Some(bounds) =
            find_limits_bounded(curve, drawer.maximal_parameter_value(), Some(100000))?
        else {
            return Ok(());
        };
        draw_deflection(
            curve,
            &optional_group(prs, draw)?,
            bounds,
            deflection,
            drawer.deviation_angle(),
            points,
        )
    }
    /// Append raw-range samples, preserving current aspects; `StdPrsDeflectionCurve::add_range_points(&prs,&curve,0.,1.,0.01,&mut points,0.2,true)?`.
    #[allow(clippy::too_many_arguments)]
    pub fn add_range_points(
        prs: &Handle<dyn Graphic3dStructureSource>,
        curve: &dyn Adaptor3dCurve,
        first: f64,
        last: f64,
        deflection: f64,
        points: &mut Sequence<Point3>,
        angle: f64,
        draw: bool,
    ) -> Result<(), Failure> {
        draw_deflection(
            curve,
            &optional_group(prs, draw)?,
            [first, last],
            deflection,
            angle,
            points,
        )
    }
}
fn optional_group(
    prs: &Handle<dyn Graphic3dStructureSource>,
    draw: bool,
) -> Result<Handle<dyn Graphic3dGroup>, Failure> {
    if draw {
        prs.current_group()
    } else {
        Ok(Handle::default())
    }
}
fn styled_group(
    prs: &Handle<dyn Graphic3dStructureSource>,
    drawer: &Prs3dDrawer,
    draw: bool,
) -> Result<Handle<dyn Graphic3dGroup>, Failure> {
    let group = optional_group(prs, draw)?;
    if !group.is_null() {
        group.set_primitives_aspect(&drawer.line_aspect().aspect().cast());
    }
    Ok(group)
}
fn draw_styled(
    curve: &dyn Adaptor3dCurve,
    group: &Handle<dyn Graphic3dGroup>,
    bounds: [f64; 2],
    drawer: &Prs3dDrawer,
) -> Result<(), Failure> {
    draw_deflection(
        curve,
        group,
        bounds,
        drawer_deflection(curve, bounds, drawer)?,
        drawer.deviation_angle(),
        &mut Sequence::default(),
    )?;
    if !drawer.line_arrow_draw() || group.is_null() {
        return Ok(());
    }
    // Unlike StdPrs_Curve, native DeflectionCurve places its arrow at the trimmed endpoint.
    let (tip, tangent) = curve.d1(bounds[1])?;
    let aspect = drawer.arrow_aspect();
    Prs3dArrow::draw(
        group,
        &tip,
        &Direction3::from_vector(tangent)?,
        aspect.angle(),
        aspect.length(),
    )
}
