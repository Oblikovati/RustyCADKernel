//! Complete `StdPrs_Curve` overload family at OCCT 3d097a0.
use super::{
    curve_draw::{draw_arrow, draw_curve},
    curve_limits::{explicit_limits, find_limits},
    curve_match::match_curve,
};
use crate::prs3d::Prs3dDrawer;
use rustycad_tkernel::{
    ncollection::Sequence,
    standard::{Failure, Handle},
};
use rustycad_tkg3d::adaptor3d::Adaptor3dCurve;
use rustycad_tkmath::gp::Point3;
use rustycad_tkservice::graphic3d::{Graphic3dStructureMembers, Graphic3dStructureSource};
/// Native fixed-discretisation curve presentations; `StdPrsCurve::add(&prs, &curve, &drawer, true)?`.
pub struct StdPrsCurve;
impl StdPrsCurve {
    /// Draw with the drawer aspect and optional arrow; `StdPrsCurve::add(&prs, &curve, &drawer, true)?`.
    pub fn add(
        prs: &Handle<dyn Graphic3dStructureSource>,
        curve: &dyn Adaptor3dCurve,
        drawer: &Handle<Prs3dDrawer>,
        draw: bool,
    ) -> Result<(), Failure> {
        let group = prs.current_group()?;
        group.set_primitives_aspect(&drawer.line_aspect().aspect().cast());
        let bounds = find_limits(curve, drawer.maximal_parameter_value())?;
        draw_curve(
            curve,
            &group,
            drawer.discretisation(),
            bounds,
            &mut Sequence::default(),
            draw,
        )?;
        draw_arrow(curve, &group, drawer)
    }
    /// Draw explicit bounds, clamping infinite sentinels; `StdPrsCurve::add_range(&prs, &curve, 0., 1., &drawer, true)?`.
    pub fn add_range(
        prs: &Handle<dyn Graphic3dStructureSource>,
        curve: &dyn Adaptor3dCurve,
        first: f64,
        last: f64,
        drawer: &Handle<Prs3dDrawer>,
        draw: bool,
    ) -> Result<(), Failure> {
        let group = prs.current_group()?;
        group.set_primitives_aspect(&drawer.line_aspect().aspect().cast());
        let bounds = explicit_limits([first, last], drawer.maximal_parameter_value());
        draw_curve(
            curve,
            &group,
            drawer.discretisation(),
            bounds,
            &mut Sequence::default(),
            draw,
        )?;
        draw_arrow(curve, &group, drawer)
    }
    /// Append sampled points without changing aspect or drawing arrows; `StdPrsCurve::add_points(&prs, &curve, &drawer, &mut points, false)?`.
    pub fn add_points(
        prs: &Handle<dyn Graphic3dStructureSource>,
        curve: &dyn Adaptor3dCurve,
        drawer: &Handle<Prs3dDrawer>,
        points: &mut Sequence<Point3>,
        draw: bool,
    ) -> Result<(), Failure> {
        let bounds = find_limits(curve, drawer.maximal_parameter_value())?;
        draw_curve(
            curve,
            &prs.current_group()?,
            drawer.discretisation(),
            bounds,
            points,
            draw,
        )
    }
    /// Append points on raw bounds using explicit sampling; `StdPrsCurve::add_range_points(&prs, &curve, 0., 1., &mut points, 30, true)?`.
    pub fn add_range_points(
        prs: &Handle<dyn Graphic3dStructureSource>,
        curve: &dyn Adaptor3dCurve,
        first: f64,
        last: f64,
        points: &mut Sequence<Point3>,
        count: i32,
        draw: bool,
    ) -> Result<(), Failure> {
        draw_curve(
            curve,
            &prs.current_group()?,
            count,
            [first, last],
            points,
            draw,
        )
    }
    /// Match using drawer sampling and automatic limits; `StdPrsCurve::matches(&point, 0.1, &curve, &drawer)?`.
    pub fn matches(
        point: &Point3,
        distance: f64,
        curve: &dyn Adaptor3dCurve,
        drawer: &Handle<Prs3dDrawer>,
    ) -> Result<bool, Failure> {
        Self::matches_with_sampling(
            point,
            distance,
            curve,
            drawer.maximal_chordial_deviation(),
            drawer.maximal_parameter_value(),
            drawer.discretisation(),
        )
    }
    /// Match with explicit sampling and automatic limits; `StdPrsCurve::matches_with_sampling(&p, 0.1, &curve, 0.01, 100., 30)?`.
    pub fn matches_with_sampling(
        point: &Point3,
        distance: f64,
        curve: &dyn Adaptor3dCurve,
        deflection: f64,
        limit: f64,
        count: i32,
    ) -> Result<bool, Failure> {
        match_curve(
            *point,
            distance,
            curve,
            deflection,
            count,
            find_limits(curve, limit)?,
        )
    }
    /// Match explicit bounds after sentinel clamping; `StdPrsCurve::matches_range(&p, 0.1, &curve, 0., 1., &drawer)?`.
    pub fn matches_range(
        point: &Point3,
        distance: f64,
        curve: &dyn Adaptor3dCurve,
        first: f64,
        last: f64,
        drawer: &Handle<Prs3dDrawer>,
    ) -> Result<bool, Failure> {
        match_curve(
            *point,
            distance,
            curve,
            drawer.maximal_chordial_deviation(),
            drawer.discretisation(),
            explicit_limits([first, last], drawer.maximal_parameter_value()),
        )
    }
    /// Match raw bounds with explicit sampling; `StdPrsCurve::matches_range_with_sampling(&p, 0.1, &curve, 0., 1., 0.01, 30)?`.
    pub fn matches_range_with_sampling(
        point: &Point3,
        distance: f64,
        curve: &dyn Adaptor3dCurve,
        first: f64,
        last: f64,
        deflection: f64,
        count: i32,
    ) -> Result<bool, Failure> {
        match_curve(*point, distance, curve, deflection, count, [first, last])
    }
}
