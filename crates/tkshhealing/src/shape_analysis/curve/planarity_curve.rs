//! Native primitive normals, wrapper recursion and complex-curve control polygons.
use super::{ShapeAnalysisCurve, planarity_points::line_normal, sampling_3d::wrapper_basis};
use crate::shape_extend::ShapeExtendComplexCurve;
use rustycad_tkernel::{
    ncollection::Array1,
    precision,
    standard::{Failure, Handle},
};
use rustycad_tkg3d::geom::GeomGeometry;
use rustycad_tkg3d::geom::{
    GeomBSplineCurve, GeomBezierCurve, GeomConic, GeomCurve, GeomLine, GeomTrimmedCurve,
};
use rustycad_tkmath::gp::{Point3, Xyz};

fn conic_normal(axis: Xyz, normal: &mut Xyz) -> bool {
    if normal.square_modulus() == 0.0 {
        *normal = axis;
        return true;
    }
    axis.crossed(*normal).square_modulus() < precision::square_confusion()
}
fn curve_poles(curve: &Handle<dyn GeomCurve>) -> Option<Array1<'static, Point3>> {
    if let Some(spline) = curve.cast::<GeomBSplineCurve>().get() {
        return Some(spline.poles());
    }
    curve
        .cast::<GeomBezierCurve>()
        .get()
        .map(GeomBezierCurve::poles)
}
fn trim_control_polygon(
    curve: &Handle<dyn GeomCurve>,
    bounds: (f64, f64),
    points: &mut Vec<Point3>,
) -> Result<bool, Failure> {
    if let Some(spline) = curve.cast::<GeomBSplineCurve>().get() {
        let copy = spline.copy()?.cast::<GeomBSplineCurve>();
        copy.segment(bounds.0, bounds.1, precision::p_confusion())?;
        append_control_poles(&copy.cast(), points)?;
        return Ok(true);
    }
    if let Some(bezier) = curve.cast::<GeomBezierCurve>().get() {
        let copy = bezier.copy()?.cast::<GeomBezierCurve>();
        copy.segment(bounds.0, bounds.1)?;
        append_control_poles(&copy.cast(), points)?;
        return Ok(true);
    }
    Ok(false)
}
fn append_values(
    curve: &dyn GeomCurve,
    parameters: &[f64],
    points: &mut Vec<Point3>,
) -> Result<(), Failure> {
    for parameter in parameters {
        points.push(curve.value(*parameter)?);
    }
    Ok(())
}
fn append_wrapper_poles(
    curve: &Handle<dyn GeomCurve>,
    points: &mut Vec<Point3>,
) -> Result<(), Failure> {
    let bounds = (curve.first_parameter(), curve.last_parameter());
    if let Some(trimmed) = curve.cast::<GeomTrimmedCurve>().get()
        && trim_control_polygon(&trimmed.basis_curve(), bounds, points).unwrap_or(false)
    {
        return Ok(());
    }
    append_values(
        &**curve,
        &[bounds.0, (bounds.0 + bounds.1) / 2.0, bounds.1],
        points,
    )
}
fn append_control_poles(
    curve: &Handle<dyn GeomCurve>,
    points: &mut Vec<Point3>,
) -> Result<(), Failure> {
    if !curve.cast::<GeomLine>().is_null() {
        return append_values(&**curve, &[0.0, 1.0], points);
    }
    if !curve.cast::<dyn GeomConic>().is_null() {
        return append_values(
            &**curve,
            &[0.0, std::f64::consts::PI / 2.0, std::f64::consts::PI],
            points,
        );
    }
    if wrapper_basis(curve).is_some() {
        return append_wrapper_poles(curve, points);
    }
    if let Some(poles) = curve_poles(curve) {
        points.extend(poles.iter().copied());
    }
    Ok(())
}
fn complex_planarity(
    curve: &dyn ShapeExtendComplexCurve,
    normal: &mut Xyz,
    tolerance: f64,
) -> Result<bool, Failure> {
    let mut points = Vec::new();
    for i in 1..=curve.nb_curves() {
        append_control_poles(&curve.curve(i)?, &mut points)?;
    }
    ShapeAnalysisCurve::is_planar_points(&Array1::from_shared(&points, 1), normal, tolerance)
}
impl ShapeAnalysisCurve {
    /// Test native curve planarity, recursing through wrappers; `ShapeAnalysisCurve::is_planar(&curve,&mut normal,1e-7)?`.
    pub fn is_planar(
        curve: &Handle<dyn GeomCurve>,
        normal: &mut Xyz,
        tolerance: f64,
    ) -> Result<bool, Failure> {
        if curve.is_null() {
            return Err(Failure::with_message(
                "Standard_NullObject",
                "ShapeAnalysis_Curve: null curve; expected live Geom_Curve",
            ));
        }
        let tolerance = if tolerance > 0.0 {
            tolerance
        } else {
            precision::confusion()
        };
        if let Some(line) = curve.cast::<GeomLine>().get() {
            return Ok(line_normal(line.position().direction().xyz(), normal));
        }
        if let Some(conic) = curve.cast::<dyn GeomConic>().get() {
            return Ok(conic_normal(conic.axis().direction().xyz(), normal));
        }
        if let Some(basis) = wrapper_basis(curve) {
            return Self::is_planar(&basis, normal, tolerance);
        }
        if let Some(poles) = curve_poles(curve) {
            return Self::is_planar_points(&poles, normal, tolerance);
        }
        if let Some(complex) = curve.cast::<dyn ShapeExtendComplexCurve>().get() {
            return complex_planarity(complex, normal, tolerance);
        }
        Ok(false)
    }
}
