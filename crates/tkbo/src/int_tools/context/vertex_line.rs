use super::*;
use crate::int_tools::Curve;
use rustycad_tkbrep::topods::Vertex;
use rustycad_tkernel::precision;
use rustycad_tkg3d::{geom::GeomBoundedCurve, geom_adaptor::GeomAdaptorCurve};
use rustycad_tkmath::geom_abs::CurveType;
mod endpoint;
use endpoint::EndpointSearch;
impl Context {
    /// Uses the vertex's stored tolerance; `context.is_vertex_on_line(&v, &curve, 1e-7, &mut t)?`.
    pub fn is_vertex_on_line(
        &self,
        vertex: &Vertex,
        curve: &Curve,
        tolerance: f64,
        parameter: &mut f64,
    ) -> Result<bool, Failure> {
        self.is_vertex_on_line_tolerance(
            vertex,
            BRepTool::vertex_tolerance(vertex)?,
            curve,
            tolerance,
            parameter,
        )
    }
    /// Preserves endpoint preference and the native minimum tolerances; `context.is_vertex_on_line_tolerance(&v, 1e-7, &curve, 1e-7, &mut t)?`.
    pub fn is_vertex_on_line_tolerance(
        &self,
        vertex: &Vertex,
        vertex_tol: f64,
        curve: &Curve,
        curve_tol: f64,
        parameter: &mut f64,
    ) -> Result<bool, Failure> {
        let point = BRepTool::pnt(vertex)?;
        let handle = curve.curve();
        let adaptor = GeomAdaptorCurve::with_curve(handle)?;
        let limit = line_tolerance(curve.curve_type()?, vertex_tol, curve_tol);
        let search = EndpointSearch::new(point, &adaptor, vertex_tol, limit);
        if search.accept_bounds(handle, parameter)? {
            return Ok(true);
        }
        self.line_projection(point, handle, limit, parameter)
    }
    fn line_projection(
        &self,
        point: Point3,
        curve: &Handle<dyn GeomCurve>,
        limit: f64,
        parameter: &mut f64,
    ) -> Result<bool, Failure> {
        let service = self.proj_pt(curve)?;
        let mut projector = service.write();
        projector.perform(point)?;
        if projector.nb_points() == 0 {
            return bounded_end_projection(point, curve, limit, parameter);
        }
        if projector.lower_distance()? > limit {
            return Ok(false);
        }
        *parameter = projector.lower_distance_parameter()?;
        Ok(true)
    }
}
fn line_tolerance(kind: CurveType, vertex: f64, curve: f64) -> f64 {
    let sum = 2.0 * (vertex + curve);
    let minimum = if matches!(kind, CurveType::BSplineCurve | CurveType::BezierCurve) {
        1e-5
    } else {
        1e-6
    };
    if sum < minimum { minimum } else { sum }
}
fn bounded_end_projection(
    point: Point3,
    curve: &Handle<dyn GeomCurve>,
    limit: f64,
    parameter: &mut f64,
) -> Result<bool, Failure> {
    let bounded = curve.cast::<dyn GeomBoundedCurve>();
    if bounded.is_null() {
        return Ok(false);
    }
    let first = bounded.start_point()?;
    let last = bounded.end_point()?;
    for (bound_point, bound) in [
        (first, curve.first_parameter()),
        (last, curve.last_parameter()),
    ] {
        if point.distance(&bound_point) < limit {
            *parameter = bound;
            return Ok(true);
        }
    }
    Ok(false)
}
