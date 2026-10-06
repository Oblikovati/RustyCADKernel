// Rust translation of the shape and tolerance members of
// ModelingAlgorithms/TKGeomAlgo/GeomFill/GeomFill.hxx/.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The static members of the `GeomFill` class that describe the circular sections
//! of a sweep: their B-spline shape, knots, multiplicities, minimal weights and the
//! approximation tolerance. They are functions of the `geom_fill` module, as the
//! class functions of the other packages (`GeomFill::GetShape` is `get_shape`).
use super::frame_math::std_max;
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::arrays::{at, put};
use rustycad_tkg3d::geom::{GeomBSplineCurve, GeomCircle, GeomCurve, GeomTrimmedCurve};
use rustycad_tkgeombase::geom_convert::curve_to_bspline_curve;
use rustycad_tkmath::convert::ParameterisationType;
use rustycad_tkmath::gp::{Axis2, Circle3, Direction3, Point3, StandardDirection3};
use std::f64::consts::PI;

/// `GeomFill::GetShape(MaxAng, NbPoles, NbKnots, Degree, TypeConv)`: the
/// `(NbPoles, NbKnots, Degree)` of a circular section of angle up to `max_ang`;
/// for the rational quadratic kinds, `conv` becomes the one of the needed number of
/// spans (up to three). `get_shape(PI, &mut conv)`.
pub fn get_shape(max_ang: f64, conv: &mut ParameterisationType) -> (i32, i32, i32) {
    match *conv {
        ParameterisationType::QuasiAngular => (7, 2, 6),
        ParameterisationType::Polynomial => (8, 2, 7),
        _ => {
            let nb_span = (3.0 * max_ang.abs() / 2.0 / PI).ceil() as i32;
            if nb_span == 1 {
                *conv = ParameterisationType::TgtThetaOver2_1;
            } else if nb_span == 2 {
                *conv = ParameterisationType::TgtThetaOver2_2;
            } else if nb_span == 3 {
                *conv = ParameterisationType::TgtThetaOver2_3;
            }
            (2 * nb_span + 1, nb_span + 1, 2)
        }
    }
}

/// `GeomFill::Knots(TypeConv, TKnots)`: `0, 1, 2...` from the lower bound, or
/// `TKnots(1) = 0`, `TKnots(2) = 1` for the quasi-angular and polynomial kinds;
/// `knots(conv, &mut knots)?`.
pub fn knots(conv: ParameterisationType, t_knots: &mut Array1<'_, f64>) -> Result<(), Failure> {
    if !matches!(
        conv,
        ParameterisationType::QuasiAngular | ParameterisationType::Polynomial
    ) {
        let mut val = 0.0;
        for i in t_knots.lower()..=t_knots.upper() {
            put(t_knots, i, val)?;
            val += 1.0;
        }
        return Ok(());
    }
    put(t_knots, 1, 0.0)?;
    put(t_knots, 2, 1.0)
}

/// `GeomFill::Mults(TypeConv, TMults)`: `7, 7` (quasi-angular), `8, 8` (polynomial)
/// at indices 1 and 2, or `3, 2, ..., 2, 3` over the bounds; `mults(conv, &mut m)?`.
pub fn mults(conv: ParameterisationType, t_mults: &mut Array1<'_, i32>) -> Result<(), Failure> {
    match conv {
        ParameterisationType::QuasiAngular => {
            put(t_mults, 1, 7)?;
            put(t_mults, 2, 7)
        }
        ParameterisationType::Polynomial => {
            put(t_mults, 1, 8)?;
            put(t_mults, 2, 8)
        }
        _ => {
            let (lower, upper) = (t_mults.lower(), t_mults.upper());
            put(t_mults, lower, 3)?;
            for i in lower + 1..=upper - 1 {
                put(t_mults, i, 2)?;
            }
            put(t_mults, upper, 3)
        }
    }
}

/// The unit circle (or of `radius`) of the XY plane, trimmed to `[0, angle]` and
/// converted with `conv`.
fn arc_bspline(
    radius: f64,
    angle: f64,
    conv: ParameterisationType,
) -> Result<Handle<GeomBSplineCurve>, Failure> {
    let z = Direction3::from_standard(StandardDirection3::Z);
    let circle = Circle3::new(Axis2::new(Point3::new(0.0, 0.0, 0.0), z)?, radius)?;
    let basis: Handle<dyn GeomCurve> = Handle::new(GeomCircle::from_circ(circle)).cast();
    let sect: Handle<dyn GeomCurve> =
        Handle::new(GeomTrimmedCurve::new(&basis, 0.0, angle, true, true)?).cast();
    curve_to_bspline_curve(&sect, conv)
}

/// `GeomFill::GetMinimalWeights(TConv, AngleMin, AngleMax, Weigths)`: all 1 for the
/// polynomial kind; otherwise the smaller weights of the arcs of `max_ang` and
/// `max(PConfusion, min_ang)`. `weights` takes the length and bounds of the first
/// arc's weights (`Assign`); `get_minimal_weights(conv, 0.1, 1.0, &mut w)?`.
pub fn get_minimal_weights(
    conv: ParameterisationType,
    min_ang: f64,
    max_ang: f64,
    weights: &mut Array1<'_, f64>,
) -> Result<(), Failure> {
    if conv == ParameterisationType::Polynomial {
        weights.init(1.0);
        return Ok(());
    }
    let first = arc_bspline(1.0, max_ang, conv)?;
    first.weights_array(|w| {
        weights.assign(w);
    });
    let angle_min = std_max(precision::p_confusion(), min_ang);
    let second = arc_bspline(1.0, angle_min, conv)?;
    let poids: Array1<'static, f64> = second.weights_array(Clone::clone);
    for ii in weights.lower()..=weights.upper() {
        let poid = at(&poids, ii)?;
        if poid < at(weights, ii)? {
            put(weights, ii, poid)?;
        }
    }
    Ok(())
}

/// `GeomFill::GetTolerance(TConv, AngleMin, Radius, AngularTol, SpatialTol)`: the
/// approximation tolerance of a section, from the distance of the first two poles
/// of an arc of `max(AngleMin, 0.02)`; `get_tolerance(conv, 0.1, 2.0, 1e-4, 1e-7)?`.
pub fn get_tolerance(
    conv: ParameterisationType,
    angle_min: f64,
    radius: f64,
    angular_tol: f64,
    spatial_tol: f64,
) -> Result<f64, Failure> {
    let ctobspl = arc_bspline(radius, std_max(angle_min, 0.02), conv)?;
    let dist = ctobspl.pole(1)?.distance(&ctobspl.pole(2)?) + spatial_tol;
    Ok(dist * angular_tol / 2.0)
}
