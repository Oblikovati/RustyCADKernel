//! Radius estimation and curve parameters retain output values on unsuccessful returns.
use super::IntTools;
use rustycad_tkbrep::{brep::BRepTool, brep_adaptor::BRepAdaptorCurve, topods::Edge};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::{
    adaptor3d::Adaptor3dCurve,
    geom::{GeomBSplineCurve, GeomBezierCurve, GeomCurve, GeomOffsetCurve},
    geom_adaptor::GeomAdaptorCurve,
};
use rustycad_tkgeomalgo::geom_api::ProjectPointOnCurve;
use rustycad_tkgeombase::{
    gce::{ErrorType, MakeCirc, Root},
    gprop::GProps,
};
use rustycad_tkmath::{elclib, geom_abs::CurveType, gp::Point3};
use rustycad_tktopalgo::brep_gprop::BRepGProp;
impl IntTools {
    /// Measures only geometric, nondegenerate edges; `IntTools::length(&edge)?`.
    pub fn length(edge: &Edge) -> Result<f64, Failure> {
        if BRepTool::degenerated(edge)? || !BRepTool::is_geometric_edge(edge)? {
            return Ok(0.);
        }
        let mut properties = GProps::default();
        BRepGProp::linear_properties(edge, &mut properties, false, false)?;
        Ok(properties.mass())
    }
    /// Estimates a radius while retaining the output on failure; `IntTools::get_radius(&curve,0.,1.,&mut radius)?`.
    pub fn get_radius(
        curve: &BRepAdaptorCurve,
        first: f64,
        last: f64,
        radius: &mut f64,
    ) -> Result<i32, Failure> {
        if let Some(status) = Self::elementary_radius(curve, radius)? {
            return Ok(status);
        }
        let samples = [
            curve.d0(first)?,
            curve.d0(0.5 * (first + last))?,
            curve.d0(last)?,
        ];
        Self::estimate_sample_radius(curve, samples, radius)
    }
    /// Writes radius only for line/circle cases that finish natively.
    fn elementary_radius(
        curve: &BRepAdaptorCurve,
        radius: &mut f64,
    ) -> Result<Option<i32>, Failure> {
        match curve.get_type()? {
            CurveType::Line => Ok(Some(1)),
            CurveType::Circle => {
                *radius = curve.circle()?.radius();
                Ok(Some(0))
            }
            _ => Ok(None),
        }
    }
    /// Estimates through three samples, preserving the native output on all failures.
    fn estimate_sample_radius(
        curve: &BRepAdaptorCurve,
        samples: [Point3; 3],
        radius: &mut f64,
    ) -> Result<i32, Failure> {
        // OCCT 3d097a0 leaves gce_Root::TheError unset here; pinned spline offsets read -1.
        if is_spline_offset(curve) && coincident_radius_samples(samples) {
            return Ok(-1);
        }
        let circle = MakeCirc::through_points(samples[0], samples[1], samples[2])?;
        if !circle.is_done() {
            return Ok(radius_failure_status(circle.status()));
        }
        *radius = circle.value()?.radius();
        Ok(0)
    }
    /// Obtains an elementary parameter or a spline projection; `IntTools::parameter(point,&curve,&mut parameter)?`.
    pub fn parameter(
        point: Point3,
        curve: &Handle<dyn GeomCurve>,
        parameter: &mut f64,
    ) -> Result<i32, Failure> {
        let first = curve.first_parameter();
        let last = curve.last_parameter();
        let adaptor = GeomAdaptorCurve::with_range(curve, first, last)?;
        let kind = adaptor.get_type()?;
        if matches!(kind, CurveType::BezierCurve | CurveType::BSplineCurve) {
            return projected_parameter(point, curve, (first, last), parameter);
        }
        let Some(value) = elementary_parameter(point, &adaptor, kind)? else {
            return Ok(1);
        };
        *parameter = value;
        Ok(0)
    }
}

fn radius_failure_status(error: ErrorType) -> i32 {
    if matches!(
        error,
        ErrorType::ConfusedPoints | ErrorType::IntersectionError | ErrorType::ColinearPoints
    ) {
        2
    } else {
        -1
    }
}

fn is_spline_offset(curve: &BRepAdaptorCurve) -> bool {
    let geometry = curve.transformed_curve().geom_curve();
    let offset = geometry.cast::<GeomOffsetCurve>();
    let Some(offset) = offset.get() else {
        return false;
    };
    let basis = offset.basis_curve();
    basis.cast::<GeomBezierCurve>().get().is_some()
        || basis.cast::<GeomBSplineCurve>().get().is_some()
}

fn coincident_radius_samples([first, middle, last]: [Point3; 3]) -> bool {
    let resolution = rustycad_tkmath::gp::resolution();
    first.distance(&middle) < resolution
        && first.distance(&last) < resolution
        && middle.distance(&last) < resolution
}

fn elementary_parameter(
    point: Point3,
    curve: &GeomAdaptorCurve,
    kind: CurveType,
) -> Result<Option<f64>, Failure> {
    Ok(Some(match kind {
        CurveType::Line => elclib::line_parameter(curve.line()?.position(), point),
        CurveType::Circle => elclib::circle_parameter(curve.circle()?.position(), point),
        CurveType::Ellipse => {
            let ellipse = curve.ellipse()?;
            elclib::ellipse_parameter(
                ellipse.position(),
                ellipse.major_radius(),
                ellipse.minor_radius(),
                point,
            )?
        }
        CurveType::Hyperbola => {
            let hyperbola = curve.hyperbola()?;
            elclib::hyperbola_parameter(
                hyperbola.position(),
                hyperbola.major_radius(),
                hyperbola.minor_radius(),
                point,
            )
        }
        CurveType::Parabola => elclib::parabola_parameter(curve.parabola()?.position(), point),
        _ => return Ok(None),
    }))
}
fn projected_parameter(
    point: Point3,
    curve: &Handle<dyn GeomCurve>,
    range: (f64, f64),
    parameter: &mut f64,
) -> Result<i32, Failure> {
    let projector = ProjectPointOnCurve::with_range(point, curve, range.0, range.1)?;
    if projector.nb_points() == 0 {
        return Ok(2);
    }
    *parameter = projector.lower_distance_parameter()?;
    Ok(0)
}
