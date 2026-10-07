// Rust translation of BlendFunc.hxx/.cxx and BlendFunc_SectionShape.hxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The `BlendFunc` package functions: the shape of a blend section and the normal of a
//! surface at a singular point.
use rustycad_tkernel::ncollection::{Array1, Array2};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::arrays::{at, put};
use rustycad_tkg2d::geom2d::native::std_max;
use rustycad_tkg3d::adaptor3d::Adaptor3dSurface;
use rustycad_tkg3d::geom::{GeomCircle, GeomCurve, GeomTrimmedCurve};
use rustycad_tkgeombase::geom_convert::curve_to_bspline_curve;
use rustycad_tkmath::convert::ParameterisationType;
use rustycad_tkmath::cslib::{self, NormalStatus, ParametricPosition};
use rustycad_tkmath::geom_abs::Shape;
use rustycad_tkmath::gp::{Axis2, Circle3, Direction3, Point2, Point3, Vector3};
use std::f64::consts::PI;

/// The shape of a blend section (`BlendFunc_SectionShape`); `SectionShape::Linear as i32 == 3`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(i32)]
pub enum SectionShape {
    /// A rational circular section.
    Rational = 0,
    /// A quasi-angular circular section.
    QuasiAngular = 1,
    /// A polynomial section.
    Polynomial = 2,
    /// A straight section.
    Linear = 3,
}

/// The pole, knot and degree counts of a section (`GetShape` outputs).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct SectionCounts {
    /// `NbPoles`.
    pub nb_poles: i32,
    /// `NbKnots`.
    pub nb_knots: i32,
    /// `Degree`.
    pub degree: i32,
}

/// `(int)` of a double as x86-64 `cvttsd2si` computes it: out-of-range values and NaN give
/// `INT_MIN` (undefined behaviour in C++); `x86_int(2.5) == 2`.
pub(crate) fn x86_int(value: f64) -> i32 {
    if value.is_nan() || value >= 2_147_483_648.0 || value < -2_147_483_648.0 {
        return i32::MIN;
    }
    value as i32
}

/// `BlendFunc::GetShape(SectShape, MaxAng, NbPoles, NbKnots, Degree, TypeConv)`: the
/// counts of a section; `conv` is set except for a linear section;
/// `get_shape(SectionShape::Polynomial, 1.0, &mut conv)`.
pub fn get_shape(
    shape: SectionShape,
    max_ang: f64,
    conv: &mut ParameterisationType,
) -> SectionCounts {
    let counts = |nb_poles, nb_knots, degree| SectionCounts {
        nb_poles,
        nb_knots,
        degree,
    };
    match shape {
        SectionShape::Rational => {
            let nb_span = x86_int((3. * max_ang.abs() / 2. / PI).ceil());
            if nb_span == 1 {
                *conv = ParameterisationType::TgtThetaOver2_1;
                return counts(2 * nb_span + 1, nb_span + 1, 2);
            }
            // QuasiAngular, to be C1 (and much more).
            *conv = ParameterisationType::QuasiAngular;
            counts(7, 2, 6)
        }
        SectionShape::QuasiAngular => {
            *conv = ParameterisationType::QuasiAngular;
            counts(7, 2, 6)
        }
        SectionShape::Polynomial => {
            *conv = ParameterisationType::Polynomial;
            counts(8, 2, 7)
        }
        SectionShape::Linear => counts(2, 2, 1),
    }
}

/// `BlendFunc::GetMinimalWeights(SectShape, TConv, AngleMin, AngleMax, Weigths)`: 1 for
/// a polynomial or linear section, else the smaller weights of the B-spline arcs of the
/// unit circle over `[0, max_ang]` and `[0, max(PConfusion, min_ang)]`;
/// `get_minimal_weights(shape, conv, 0.1, 1.0, &mut weights)?`.
pub fn get_minimal_weights(
    shape: SectionShape,
    conv: ParameterisationType,
    min_ang: f64,
    max_ang: f64,
    weights: &mut Array1<'_, f64>,
) -> Result<(), Failure> {
    if matches!(shape, SectionShape::Polynomial | SectionShape::Linear) {
        weights.init(1.0);
        return Ok(());
    }
    let first = unit_arc_weights(conv, max_ang)?;
    weights.assign(&first);
    let angle_min = std_max(precision::p_confusion(), min_ang);
    let poids = unit_arc_weights(conv, angle_min)?;
    for ii in weights.lower()..=weights.upper() {
        let candidate = at(&poids, ii)?;
        if candidate < at(weights, ii)? {
            put(weights, ii, candidate)?;
        }
    }
    Ok(())
}

/// The weights of the B-spline conversion of the unit circle trimmed to `[0, angle]`.
fn unit_arc_weights(
    conv: ParameterisationType,
    angle: f64,
) -> Result<Array1<'static, f64>, Failure> {
    let frame = Axis2::new(Point3::default(), Direction3::new(0., 0., 1.)?)?;
    let circle: Handle<dyn GeomCurve> =
        Handle::new(GeomCircle::from_circ(Circle3::new(frame, 1.)?)).cast();
    let arc: Handle<dyn GeomCurve> =
        Handle::new(GeomTrimmedCurve::new(&circle, 0., angle, true, true)?).cast();
    let bspline = curve_to_bspline_curve(&arc, conv)?;
    Ok(bspline.weights_array(Clone::clone))
}

/// `BlendFunc::NextShape(S)`: the next continuity, CN after C2; `next_shape(Shape::C0) == Shape::C1`.
pub fn next_shape(shape: Shape) -> Shape {
    match shape {
        Shape::C0 => Shape::C1,
        Shape::C1 => Shape::C2,
        Shape::C2 => Shape::C3,
        _ => Shape::CN,
    }
}

/// The derivatives of `N = D1U ^ D1V` up to order 3 at `p2d` (the `DerNUV` table of the
/// normal functions) and the parametric position for `CSLib::Normal`.
fn normal_derivatives(
    surface: &dyn Adaptor3dSurface,
    p2d: Point2,
) -> Result<(Array2<'static, Vector3>, ParametricPosition), Failure> {
    const MAX_ORDER: i32 = 3;
    let (u, v) = (p2d.x(), p2d.y());
    let mut der_surf = array2(0, MAX_ORDER + 1)?;
    for i in 1..=MAX_ORDER + 1 {
        set2(&mut der_surf, i, 0, surface.dn(u, v, i, 0)?)?;
    }
    for i in 0..=MAX_ORDER + 1 {
        for j in 1..=MAX_ORDER + 1 {
            set2(&mut der_surf, i, j, surface.dn(u, v, i, j)?)?;
        }
    }
    let mut der_nuv = array2(0, MAX_ORDER)?;
    for i in 0..=MAX_ORDER {
        for j in 0..=MAX_ORDER {
            set2(&mut der_nuv, i, j, cslib::dnnuv(i, j, &der_surf)?)?;
        }
    }
    let position = ParametricPosition {
        u,
        v,
        u_min: surface.first_u_parameter()?,
        u_max: surface.last_u_parameter()?,
        v_min: surface.first_v_parameter()?,
        // szv: was FirstVParameter!
        v_max: surface.last_v_parameter()?,
    };
    Ok((der_nuv, position))
}

fn array2(low: i32, up: i32) -> Result<Array2<'static, Vector3>, Failure> {
    Array2::with_bounds(low, up, low, up)
        .map_err(|error| Failure::with_message("Standard_RangeError", &format!("{error:?}")))
}

fn set2(
    array: &mut Array2<'_, Vector3>,
    row: i32,
    col: i32,
    value: Vector3,
) -> Result<(), Failure> {
    array.set_value(row, col, value).map_err(|error| {
        Failure::out_of_range(&format!("BlendFunc normal table ({row}, {col}): {error:?}"))
    })
}

/// `BlendFunc::ComputeNormal(Surf, p2d, Normal)`: the normal at a point where the first
/// derivatives degenerate, from higher derivatives; `None` when it is not defined;
/// `compute_normal(&surface, p2d)?`.
pub fn compute_normal(
    surface: &Handle<dyn Adaptor3dSurface>,
    p2d: Point2,
) -> Result<Option<Vector3>, Failure> {
    let surface = live_surface(surface)?;
    let (der_nuv, position) = normal_derivatives(surface, p2d)?;
    let singular = cslib::normal_at_singularity(3, &der_nuv, 1.0e-9, position)?;
    if singular.result.status != NormalStatus::Defined {
        return Ok(None);
    }
    let normal = singular.result.normal.unwrap_or_default();
    Ok(Some(Vector3::from(normal.xyz())))
}

/// `BlendFunc::ComputeDNormal(Surf, p2d, Normal, DNu, DNv)`: the normal and its
/// derivatives at a singular point; `None` when the normal is not defined;
/// `compute_d_normal(&surface, p2d)?`.
pub fn compute_d_normal(
    surface: &Handle<dyn Adaptor3dSurface>,
    p2d: Point2,
) -> Result<Option<[Vector3; 3]>, Failure> {
    let surface = live_surface(surface)?;
    let (der_nuv, position) = normal_derivatives(surface, p2d)?;
    let singular = cslib::normal_at_singularity(3, &der_nuv, 1.0e-9, position)?;
    if singular.result.status != NormalStatus::Defined {
        return Ok(None);
    }
    let normal = Vector3::from(singular.result.normal.unwrap_or_default().xyz());
    let dnu = cslib::dn_normal(1, 0, &der_nuv, singular.order_u, singular.order_v)?;
    let dnv = cslib::dn_normal(0, 1, &der_nuv, singular.order_u, singular.order_v)?;
    Ok(Some([normal, dnu, dnv]))
}

/// The surface behind a handle; a null handle is dereferenced natively.
pub(crate) fn live_surface(
    surface: &Handle<dyn Adaptor3dSurface>,
) -> Result<&dyn Adaptor3dSurface, Failure> {
    surface.get().ok_or_else(|| {
        Failure::with_message("Standard_NullObject", "BlendFunc: null surface adaptor")
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rational_shape_uses_one_span_up_to_two_thirds_of_pi() {
        let mut conv = ParameterisationType::TgtThetaOver2;
        let counts = get_shape(SectionShape::Rational, 2.0, &mut conv);
        assert_eq!(
            (counts.nb_poles, counts.degree, conv),
            (3, 2, ParameterisationType::TgtThetaOver2_1)
        );
        let counts = get_shape(SectionShape::Rational, 2.5, &mut conv);
        assert_eq!(
            (counts.nb_poles, counts.nb_knots, conv),
            (7, 2, ParameterisationType::QuasiAngular)
        );
        let counts = get_shape(SectionShape::Linear, 1.0, &mut conv);
        assert_eq!(
            (counts.nb_poles, conv),
            (2, ParameterisationType::QuasiAngular)
        );
        assert_eq!(get_shape(SectionShape::Polynomial, 0., &mut conv).degree, 7);
    }

    #[test]
    fn x86_int_gives_int_min_out_of_range() {
        assert_eq!(x86_int(f64::INFINITY), i32::MIN);
        assert_eq!(x86_int(f64::NAN), i32::MIN);
        assert_eq!(x86_int(-3.7), -3);
    }

    #[test]
    fn next_shape_saturates_at_cn() {
        assert_eq!(next_shape(Shape::C2), Shape::C3);
        assert_eq!(next_shape(Shape::G1), Shape::CN);
    }

    #[test]
    fn linear_weights_are_one() {
        let mut buffer = [0.; 3];
        let mut weights = Array1::from_slice(&mut buffer);
        get_minimal_weights(
            SectionShape::Linear,
            ParameterisationType::QuasiAngular,
            0.,
            1.,
            &mut weights,
        )
        .unwrap();
        assert_eq!(buffer, [1.; 3]);
    }
}
