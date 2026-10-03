//! Native wire intersection output and geometry evaluation conventions.
// ShapeAnalysis_Wire.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.
mod pair;
mod self_edge;
use super::*;
use rustycad_tkbrep::topods::Edge;
use rustycad_tkernel::ncollection::Sequence;
use rustycad_tkg2d::{
    adaptor2d::Adaptor2dCurve2d, geom2d::Geom2dCurve, geom2d_adaptor::Geom2dAdaptorCurve,
};
use rustycad_tkgeomalgo::{
    geom2d_int::GInter,
    int_res2d::{Domain, IntersectionPoint, Position},
};
use rustycad_tkmath::gp::Point3;

const INTERSECTION_TOLERANCE: f64 = 1e-10;

fn intersection_domain(
    curve: &Handle<dyn Geom2dCurve>,
    first: f64,
    last: f64,
) -> Result<Domain, Failure> {
    Ok(Domain::bounded(
        curve.value(first)?,
        first,
        INTERSECTION_TOLERANCE,
        curve.value(last)?,
        last,
        INTERSECTION_TOLERANCE,
    ))
}

fn has_middle_transition(point: &IntersectionPoint) -> bool {
    point.transition_of_first().position_on_curve() == Position::Middle
        || point.transition_of_second().position_on_curve() == Position::Middle
}

impl ShapeAnalysisWire {
    fn point_on_intersected_edge(
        &self,
        edge: &Edge,
        curve: &Geom2dAdaptorCurve,
        parameter: f64,
    ) -> Result<Point3, Failure> {
        // Native h0 (1999-05-29): prefer the located 3D curve only for SameParameter edges.
        if BRepTool::same_parameter(edge)? {
            let found = BRepTool::curve_with_location(edge)?;
            if !found.curve.is_null() {
                return Ok(found
                    .curve
                    .value(parameter)?
                    .transformed(found.location.transformation()));
            }
        }
        let uv = curve.value(parameter)?;
        use rustycad_tkg3d::adaptor3d::Adaptor3dSurface;
        self.surface().adaptor3d().value(uv.x(), uv.y())
    }
}

fn intersection_candidate(
    intersections: &GInter,
    count: i32,
    index: i32,
) -> Result<Option<IntersectionPoint>, Failure> {
    let point = if index <= count {
        intersections.point(index)?
    } else {
        let segment = intersections.segment(index - count)?;
        if !segment.has_first_point() || !segment.has_last_point() {
            return Ok(None);
        }
        let first = segment.first_point()?;
        if has_middle_transition(&first) {
            segment.last_point()?
        } else {
            first
        }
    };
    Ok(has_middle_transition(&point).then_some(point))
}

fn intersection_midpoint(first: Point3, second: Point3) -> Point3 {
    let midpoint = (first.xyz() + second.xyz()) * 0.5;
    Point3::new(midpoint.x(), midpoint.y(), midpoint.z())
}
