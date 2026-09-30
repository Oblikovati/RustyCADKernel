// Rust translation of ModelingData/TKGeomBase/GeomLib/GeomLib.cxx (AdjustExtremity)
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `GeomLib::AdjustExtremity`: a cubic deformation that moves the ends of a curve
//! and turns its end tangents.
use super::live;
use crate::array_access::matrix;
use crate::geom_convert;
use crate::native_failure::failure;
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::arrays::one_based;
use rustycad_tkg3d::geom::{GeomBSplineCurve, GeomBoundedCurve, GeomCurve, GeomCurveD1};
use rustycad_tkmath::convert::ParameterisationType;
use rustycad_tkmath::gp::{Point3, Vector3, Xyz};
use rustycad_tkmath::math::Matrix;
use rustycad_tkmath::plib;

/// `GeomLib::AdjustExtremity(Curve, P1, P2, T1, T2)`: replaces `curve` by its
/// B-spline (quasi-angular conversion) plus the cubic Hermite deformation that
/// moves its start to `p1` and its end to `p2` and makes its end derivatives
/// parallel to `t1` and `t2` (their components along the tangents are kept).
///
/// The deformation is raised to the curve's degree (or the curve to 3) and gets its
/// inner knots. Standard_ConstructionError when the Hermite system fails or the pole
/// counts differ ("Inconsistent poles's number"); gp_VectorWithNullMagnitude for a
/// null tangent; `geom_lib::adjust_extremity(&mut curve, p1, p2, t1, t2)?`.
pub fn adjust_extremity(
    curve: &mut Handle<dyn GeomBoundedCurve>,
    p1: Point3,
    p2: Point3,
    t1: Vector3,
    t2: Vector3,
) -> Result<(), Failure> {
    // Convert the input (preserving the parameterization if possible).
    let a_in: Handle<GeomBSplineCurve> = geom_convert::curve_to_bspline_curve(
        &curve.cast::<dyn GeomCurve>(),
        ParameterisationType::QuasiAngular,
    )?;
    let input: &GeomBSplineCurve = live(&a_in, "GeomLib::AdjustExtremity")?;
    let (first, last): (f64, f64) = (input.first_parameter(), input.last_parameter());
    // Calculation of deformation constraints.
    let mut poles_def: Array1<'static, Point3> = one_based(4)?;
    let start: GeomCurveD1 = input.eval_d1(first)?;
    poles_def[1].set_xyz(p1.xyz() - start.point.xyz());
    poles_def[2].set_xyz(tangent_correction(t1, start.d1, last - first)?);
    let end: GeomCurveD1 = input.eval_d1(last)?;
    poles_def[3].set_xyz(p2.xyz() - end.point.xyz());
    poles_def[4].set_xyz(tangent_correction(t2, end.d1, last - first)?);
    let poles_def: Array1<'static, Point3> = hermite_poles(&poles_def)?;
    // Add the deformation.
    let a_def: GeomBSplineCurve = deformation(&poles_def, first, last)?;
    if input.degree() < 3 {
        input.increase_degree(3)?;
    } else {
        a_def.increase_degree(input.degree())?;
    }
    for ii in 2..input.nb_knots() {
        a_def.insert_knot(input.knot(ii)?, input.multiplicity(ii)?, 0.0, true)?;
    }
    if a_def.nb_poles() != input.nb_poles() {
        return Err(failure(
            "Standard_ConstructionError",
            "Inconsistent poles's number",
        ));
    }
    for ii in 1..=a_def.nb_poles() {
        let mut a_pole: Point3 = input.pole(ii)?;
        a_pole.set_xyz(a_pole.xyz() + a_def.pole(ii)?.xyz());
        input.set_pole(ii, a_pole)?;
    }
    *curve = a_in.cast();
    Ok(())
}

// (Last - First) (u (u . D) - D), u the unit tangent: the change that makes the
// derivative D parallel to the tangent.
fn tangent_correction(tangent: Vector3, derivative: Vector3, span: f64) -> Result<Xyz, Failure> {
    let unit: Vector3 = tangent.normalized()?;
    let correction: Vector3 = unit * (unit * derivative) - derivative;
    Ok(span * *correction.xyz())
}

// Interpolation of constraints: the Hermite coefficients on [0, 1] applied to the
// constraints, then converted to Bernstein poles.
fn hermite_poles(poles_def: &Array1<'static, Point3>) -> Result<Array1<'static, Point3>, Failure> {
    let mut mat: Matrix<'static> = matrix(1, 4, 1, 4, 0.0)?;
    if !plib::hermite_coefficients(0.0, 1.0, 1, 1, &mut mat)? {
        return Err(failure(
            "Standard_ConstructionError",
            "GeomLib::AdjustExtremity: PLib::HermiteCoefficients failed",
        ));
    }
    let mut coeffs: Array1<'static, Point3> = one_based(4)?;
    for coeff in 1..=4 {
        let mut aux = Xyz::new(0.0, 0.0, 0.0);
        for constraint in 1..=4 {
            aux.set_scaled_sum(
                *mat.value(constraint, coeff)?,
                poles_def[constraint].xyz(),
                aux,
            );
        }
        coeffs[coeff].set_xyz(aux);
    }
    let mut poles: Array1<'static, Point3> = one_based(4)?;
    plib::coefficients_poles(&coeffs, None, &mut poles, None)?;
    Ok(poles)
}

// The cubic Bezier-like B-spline of the deformation on [First, Last].
fn deformation(
    poles_def: &Array1<'static, Point3>,
    first: f64,
    last: f64,
) -> Result<GeomBSplineCurve, Failure> {
    let mut k: Array1<'static, f64> = one_based(2)?;
    (k[1], k[2]) = (first, last);
    let mut m: Array1<'static, i32> = one_based(2)?;
    m.init(4);
    GeomBSplineCurve::new(poles_def, &k, &m, 3, false)
}
