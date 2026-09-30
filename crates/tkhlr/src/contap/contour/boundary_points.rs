//! Contap_Contour boundary attachment and exclusion at OCCT 3d097a0.
use super::math_helpers::{LineMatch, find_line};
use crate::contap::{
    ContapHContTool, ContapHCurve2dTool, ContapLine, ContapPoint, ContapSurfFunction,
    ContapTheSearch, ContapTheSearchInside,
};
use rustycad_tkernel::{precision, standard::Failure};
use rustycad_tkg3d::adaptor3d::h_surface_tool::{self as surface_tool, Adaptor3dSurfaceHandle};
use rustycad_tkgeomalgo::int_surf::{IntSurfInteriorPoint, IntSurfTransition, make_transition};
use rustycad_tkmath::gp::{Direction3, Point2, Vector2, Vector3};
struct BoundaryAttachment {
    matched: LineMatch,
    line_transition: IntSurfTransition,
    arc_transition: IntSurfTransition,
}
pub(super) fn put_points_on_lines(
    search: &ContapTheSearch,
    surface: &Adaptor3dSurfaceHandle,
    lines: &[ContapLine],
) -> Result<(), Failure> {
    let count = search.nb_points()?;
    let mut attachment = BoundaryAttachment {
        matched: LineMatch::default(),
        line_transition: IntSurfTransition::new(),
        arc_transition: IntSurfTransition::new(),
    };
    for line in lines {
        for index in 1..=count {
            attachment.attach(search, index, surface, line)?;
        }
    }
    Ok(())
}
impl BoundaryAttachment {
    fn attach(
        &mut self,
        search: &ContapTheSearch,
        index: i32,
        surface: &Adaptor3dSurfaceHandle,
        line: &ContapLine,
    ) -> Result<(), Failure> {
        let start = search.point(index)?;
        let arc = start.arc();
        let (uv, tangent) = ContapHCurve2dTool::d1(&arc, start.parameter())?;
        if !find_line(line, surface, uv, &mut self.matched)? {
            return Ok(());
        }
        self.transitions(surface, uv, tangent)?;
        let mut point = ContapPoint::with_value(self.matched.point, uv.x(), uv.y());
        point.set_arc(
            &arc,
            start.parameter(),
            self.line_transition,
            self.arc_transition,
        );
        point.set_parameter(self.matched.parameter);
        if !start.is_new() {
            point.set_vertex(&start.vertex()?);
        }
        line.add_vertex(&point);
        Ok(())
    }
    fn transitions(
        &mut self,
        surface: &Adaptor3dSurfaceHandle,
        uv: Point2,
        tangent: Vector2,
    ) -> Result<(), Failure> {
        let derivative = surface_tool::d1(surface, uv.x(), uv.y())?;
        let normal = self.matched.normal;
        if normal.magnitude() < f64::EPSILON {
            self.line_transition.set_undecided();
            self.arc_transition.set_undecided();
            return Ok(());
        }
        let boundary = restriction_tangent(tangent, derivative.d1u, derivative.d1v, normal);
        let direction = Direction3::new(normal.x(), normal.y(), normal.z())?;
        make_transition(
            self.matched.tangent,
            boundary,
            direction,
            &mut self.line_transition,
            &mut self.arc_transition,
        );
        Ok(())
    }
}
fn restriction_tangent(tangent: Vector2, du: Vector3, dv: Vector3, normal: Vector3) -> Vector3 {
    if tangent.y().abs() <= precision::confusion() {
        let mut boundary = dv.crossed(normal);
        if tangent.x() < 0. {
            boundary.reverse();
        }
        return boundary;
    }
    let mut boundary = Vector3::default();
    boundary.set_linear_form(tangent.x(), du, tangent.y(), dv);
    boundary
}
pub(super) fn keep_inside_points(
    inside: &ContapTheSearchInside,
    boundary: &ContapTheSearch,
    function: &ContapSurfFunction,
    output: &mut Vec<IntSurfInteriorPoint>,
) -> Result<(), Failure> {
    let segments = boundary.nb_segments()?;
    for index in 1..=inside.nb_points()? {
        let point = inside.value(index)?;
        if keep_point(point, boundary, segments, &function.surface())? {
            output.push(*point);
        }
    }
    Ok(())
}
fn keep_point(
    point: &IntSurfInteriorPoint,
    boundary: &ContapTheSearch,
    segments: i32,
    surface: &Adaptor3dSurfaceHandle,
) -> Result<bool, Failure> {
    let uv = Point2::new(point.u_parameter(), point.v_parameter());
    for index in 1..=segments {
        let arc = boundary.segment(index)?.curve();
        let Some((_, projected)) = ContapHContTool::project(&arc, uv)? else {
            continue;
        };
        let projection = surface_tool::value(surface, projected.x(), projected.y())?;
        if point.value().distance(&projection) <= precision::confusion() {
            return Ok(false);
        }
    }
    Ok(true)
}
