//! HLRBRep_BCurveTool.hxx/.lxx/.cxx, OCCT 3d097a0328e71b826377d4814ab05ec3c3d23871.
use crate::hlr_algo::c_int_truncation;
use rustycad_tkbrep::brep_adaptor::BRepAdaptorCurve;
use rustycad_tkernel::{
    ncollection::Array1,
    standard::{Failure, Handle},
};
use rustycad_tkg2d::geom2d::arrays::{at, put};
use rustycad_tkg3d::{
    adaptor3d::{Adaptor3dCurve, Adaptor3dD3},
    geom::{GeomBSplineCurve, GeomBezierCurve},
};
use rustycad_tkmath::{
    geom_abs::{CurveType, Shape},
    gp::{Circle3, Ellipse3, Hyperbola3, Line3, Parabola3, Point3, Vector3},
};
/// Exact-HLR access to an edge's 3D curve; `BCurveTool::value(&curve, 0.5)?`.
pub struct BCurveTool;
impl BCurveTool {
    /// Native adaptor query; `BCurveTool::first_parameter(&curve)?`.
    pub fn first_parameter(curve: &BRepAdaptorCurve) -> Result<f64, Failure> {
        curve.first_parameter()
    }
    /// Native adaptor query; `BCurveTool::last_parameter(&curve)?`.
    pub fn last_parameter(curve: &BRepAdaptorCurve) -> Result<f64, Failure> {
        curve.last_parameter()
    }
    /// Native adaptor query; `BCurveTool::continuity(&curve)?`.
    pub fn continuity(curve: &BRepAdaptorCurve) -> Result<Shape, Failure> {
        curve.continuity()
    }
    /// Native adaptor query; `BCurveTool::is_closed(&curve)?`.
    pub fn is_closed(curve: &BRepAdaptorCurve) -> Result<bool, Failure> {
        curve.is_closed()
    }
    /// Native adaptor query; `BCurveTool::is_periodic(&curve)?`.
    pub fn is_periodic(curve: &BRepAdaptorCurve) -> Result<bool, Failure> {
        curve.is_periodic()
    }
    /// Native adaptor query; `BCurveTool::period(&curve)?`.
    pub fn period(curve: &BRepAdaptorCurve) -> Result<f64, Failure> {
        curve.period()
    }
    /// Native adaptor query; `BCurveTool::get_type(&curve)?`.
    pub fn get_type(curve: &BRepAdaptorCurve) -> Result<CurveType, Failure> {
        curve.get_type()
    }
    /// Native adaptor query; `BCurveTool::line(&curve)?`.
    pub fn line(curve: &BRepAdaptorCurve) -> Result<Line3, Failure> {
        curve.line()
    }
    /// Native adaptor query; `BCurveTool::circle(&curve)?`.
    pub fn circle(curve: &BRepAdaptorCurve) -> Result<Circle3, Failure> {
        curve.circle()
    }
    /// Native adaptor query; `BCurveTool::ellipse(&curve)?`.
    pub fn ellipse(curve: &BRepAdaptorCurve) -> Result<Ellipse3, Failure> {
        curve.ellipse()
    }
    /// Native adaptor query; `BCurveTool::hyperbola(&curve)?`.
    pub fn hyperbola(curve: &BRepAdaptorCurve) -> Result<Hyperbola3, Failure> {
        curve.hyperbola()
    }
    /// Native adaptor query; `BCurveTool::parabola(&curve)?`.
    pub fn parabola(curve: &BRepAdaptorCurve) -> Result<Parabola3, Failure> {
        curve.parabola()
    }
    /// Native adaptor query; `BCurveTool::bezier(&curve)?`.
    pub fn bezier(curve: &BRepAdaptorCurve) -> Result<Handle<GeomBezierCurve>, Failure> {
        curve.bezier()
    }
    /// Native adaptor query; `BCurveTool::bspline(&curve)?`.
    pub fn bspline(curve: &BRepAdaptorCurve) -> Result<Handle<GeomBSplineCurve>, Failure> {
        curve.bspline()
    }
    /// Native adaptor query; `BCurveTool::degree(&curve)?`.
    pub fn degree(curve: &BRepAdaptorCurve) -> Result<i32, Failure> {
        curve.degree()
    }
    /// Native adaptor query; `BCurveTool::is_rational(&curve)?`.
    pub fn is_rational(curve: &BRepAdaptorCurve) -> Result<bool, Failure> {
        curve.is_rational()
    }
    /// Native adaptor query; `BCurveTool::nb_poles(&curve)?`.
    pub fn nb_poles(curve: &BRepAdaptorCurve) -> Result<i32, Failure> {
        curve.nb_poles()
    }
    /// Native adaptor query; `BCurveTool::nb_knots(&curve)?`.
    pub fn nb_knots(curve: &BRepAdaptorCurve) -> Result<i32, Failure> {
        curve.nb_knots()
    }
    /// Evaluates the native curve at its parameter; `BCurveTool::value(&curve,0.5)?`.
    pub fn value(curve: &BRepAdaptorCurve, parameter: f64) -> Result<Point3, Failure> {
        curve.value(parameter)
    }
    /// Evaluates the native curve at its parameter; `BCurveTool::d0(&curve,0.5)?`.
    pub fn d0(curve: &BRepAdaptorCurve, parameter: f64) -> Result<Point3, Failure> {
        curve.d0(parameter)
    }
    /// Evaluates the native curve at its parameter; `BCurveTool::d1(&curve,0.5)?`.
    pub fn d1(curve: &BRepAdaptorCurve, parameter: f64) -> Result<(Point3, Vector3), Failure> {
        curve.d1(parameter)
    }
    /// Evaluates the native curve at its parameter; `BCurveTool::d2(&curve,0.5)?`.
    pub fn d2(
        curve: &BRepAdaptorCurve,
        parameter: f64,
    ) -> Result<(Point3, Vector3, Vector3), Failure> {
        curve.d2(parameter)
    }
    /// Evaluates the native curve at its parameter; `BCurveTool::d3(&curve,0.5)?`.
    pub fn d3(curve: &BRepAdaptorCurve, parameter: f64) -> Result<Adaptor3dD3, Failure> {
        curve.d3(parameter)
    }
    /// Evaluates the requested derivative; `BCurveTool::dn(&curve,0.5,2)?`.
    pub fn dn(curve: &BRepAdaptorCurve, parameter: f64, order: i32) -> Result<Vector3, Failure> {
        curve.dn(parameter, order)
    }
    /// Converts spatial tolerance to parameter tolerance; `BCurveTool::resolution(&curve,1e-7)?`.
    pub fn resolution(curve: &BRepAdaptorCurve, tolerance: f64) -> Result<f64, Failure> {
        curve.resolution(tolerance)
    }
    /// Counts continuity intervals; `BCurveTool::nb_intervals(&curve,Shape::C2)?`.
    pub fn nb_intervals(curve: &BRepAdaptorCurve, shape: Shape) -> Result<i32, Failure> {
        curve.nb_intervals(shape)
    }
    /// Fills caller-owned interval bounds; `BCurveTool::intervals(&curve,&mut bounds,Shape::C2)?`.
    pub fn intervals(
        curve: &BRepAdaptorCurve,
        bounds: &mut Array1<'_, f64>,
        shape: Shape,
    ) -> Result<(), Failure> {
        curve.intervals(bounds, shape)
    }
    /// Copies only the requested native pole indices; other curve kinds leave output intact. `BCurveTool::poles(&curve,&mut poles)?`.
    pub fn poles(curve: &BRepAdaptorCurve, poles: &mut Array1<'_, Point3>) -> Result<(), Failure> {
        match curve.get_type()? {
            CurveType::BezierCurve => copy_indices(&curve.bezier()?.poles(), poles),
            CurveType::BSplineCurve => copy_indices(&curve.bspline()?.poles(), poles),
            _ => Ok(()),
        }
    }
    /// Copies poles before weights, preserving partial writes on failure; `BCurveTool::poles_and_weights(&curve,&mut poles,&mut weights)?`.
    pub fn poles_and_weights(
        curve: &BRepAdaptorCurve,
        poles: &mut Array1<'_, Point3>,
        weights: &mut Array1<'_, f64>,
    ) -> Result<(), Failure> {
        match curve.get_type()? {
            CurveType::BezierCurve => {
                let spline = curve.bezier()?;
                copy_indices(&spline.poles(), poles)?;
                spline.weights_array(|source| copy_indices(source, weights))
            }
            CurveType::BSplineCurve => {
                let spline = curve.bspline()?;
                copy_indices(&spline.poles(), poles)?;
                spline.weights_array(|source| copy_indices(source, weights))
            }
            _ => Ok(()),
        }
    }
    /// Native inverse-span sampling rule, capped at 50; `BCurveTool::nb_samples(&curve,0.,1.)?`.
    pub fn nb_samples(curve: &BRepAdaptorCurve, first: f64, last: f64) -> Result<i32, Failure> {
        let mut count = match curve.get_type()? {
            CurveType::Line => 2.,
            CurveType::BezierCurve => f64::from(3 + curve.nb_poles()?),
            CurveType::BSplineCurve => spline_samples(curve, first, last)?,
            _ => 10.,
        };
        if count > 50. {
            count = 50.;
        }
        Ok(c_int_truncation(count))
    }
}
fn copy_indices<T: Copy>(
    source: &Array1<'_, T>,
    target: &mut Array1<'_, T>,
) -> Result<(), Failure> {
    for index in target.lower()..=target.upper() {
        put(target, index, at(source, index)?)?;
    }
    Ok(())
}
fn spline_samples(curve: &BRepAdaptorCurve, first: f64, last: f64) -> Result<f64, Failure> {
    let mut count = f64::from(curve.nb_knots()?);
    count *= f64::from(curve.degree()?);
    count *= curve.last_parameter()? - curve.first_parameter()?;
    count /= last - first;
    if count < 2. {
        count = 2.;
    }
    Ok(count)
}
