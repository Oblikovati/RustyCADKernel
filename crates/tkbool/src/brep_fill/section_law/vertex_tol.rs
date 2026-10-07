// Rust translation of the body shared by BRepFill_ShapeLaw::VertexTol and
// BRepFill_NSections::VertexTol (ModelingAlgorithms/TKBool/BRepFill) at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The gap between the end of one section law and the start of the next.
use super::index_error;
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::arrays::{at, one_based};
use rustycad_tkg3d::geom::{GeomBSplineCurve, GeomCurve};
use rustycad_tkgeomalgo::geom_fill::GeomFillSectionLaw;
use rustycad_tkmath::gp::Point3;

/// The section of a law at `param` as a B-spline curve and its knots.
fn section_curve(
    law: &Handle<dyn GeomFillSectionLaw>,
    param: f64,
) -> Result<(GeomBSplineCurve, Array1<'static, f64>), Failure> {
    let Some(loi) = law.get() else {
        return Err(Failure::with_type(
            "Standard_NullObject",
            "BRepFill VertexTol: null section law; expected a law",
            "BRepFill_SectionLaw",
        ));
    };
    let (nb_poles, nb_knots, degree) = loi.section_shape()?;
    let mut poles = one_based::<Point3>(nb_poles)?;
    let mut weights = one_based::<f64>(nb_poles)?;
    loi.d0(param, &mut poles, &mut weights)?;
    let mut knots = one_based::<f64>(nb_knots)?;
    loi.knots(&mut knots)?;
    let mut mults = one_based::<i32>(nb_knots)?;
    loi.mults(&mut mults)?;
    let curve = GeomBSplineCurve::with_weights(
        &poles,
        &weights,
        &knots,
        &mults,
        degree,
        loi.is_u_periodic()?,
    )?;
    Ok((curve, knots))
}

/// `VertexTol` between the laws `i1` and `i2` (1-based): Precision::Confusion() plus
/// the distance from the end of the section of `i1` to the start of the section of
/// `i2`, both at `param`.
pub(super) fn vertex_gap(
    laws: &[Handle<dyn GeomFillSectionLaw>],
    (i1, i2): (i32, i32),
    param: f64,
) -> Result<f64, Failure> {
    let law = |i: i32| -> Result<&Handle<dyn GeomFillSectionLaw>, Failure> {
        usize::try_from(i - 1)
            .ok()
            .and_then(|k| laws.get(k))
            .ok_or_else(|| index_error("BRepFill VertexTol", i, laws.len()))
    };
    let mut tol: f64 = precision::confusion();
    let (first, first_knots) = section_curve(law(i1)?, param)?;
    let p_first: Point3 = first.value(at(&first_knots, first_knots.upper())?)?;
    let (second, second_knots) = section_curve(law(i2)?, param)?;
    tol += p_first.distance(&second.value(at(&second_knots, 1)?)?);
    Ok(tol)
}
