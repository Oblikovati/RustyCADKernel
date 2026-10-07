// Rust translation of ChFi3d_ApproxByC2 and ChFi3d_IsSmooth (ChFi3d_Builder_0.cxx) at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The smoothness test of a guide line and its C2 approximation.
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::adaptor3d::Adaptor3dCurve;
use rustycad_tkg3d::geom::{GeomBSplineCurve, GeomCurve};
use rustycad_tkg3d::geom_adaptor::GeomAdaptorCurve;
use rustycad_tkgeomalgo::geom_api::PointsToBSpline;
use rustycad_tkgeombase::approx::ParametrizationType;
use rustycad_tkgeombase::geom_lprop::CLProps;
use rustycad_tkmath::geom_abs::Shape as Continuity;
use rustycad_tkmath::gp::{self, Point3, Vector3};
use std::f64::consts::PI;

/// `ChFi3d_ApproxByC2(C)`: the cubic to degree 8 C2 B-spline through 101 points of the
/// curve (chord-length parametrization, tolerance 1.000001e-3); `approx_by_c2(&c)?`.
pub fn approx_by_c2(c: &Handle<dyn GeomCurve>) -> Result<Handle<GeomBSplineCurve>, Failure> {
    let (first, last) = (c.first_parameter(), c.last_parameter());
    let nb_points = 101;
    let delta = (last - first) / f64::from(nb_points - 1);
    let mut points = Vec::with_capacity(nb_points as usize);
    for i in 1..nb_points {
        points.push(c.value(first + f64::from(i - 1) * delta)?);
    }
    points.push(c.value(last)?);
    let approx = PointsToBSpline::with_parametrization(
        &Array1::from_slice(&mut points),
        ParametrizationType::ChordLength,
        3,
        8,
        Continuity::C2,
        1.000001e-3,
    )?;
    approx.curve()
}

/// One sample of `ChFi3d_IsSmooth`.
enum CentreSample {
    /// The tangent is not defined.
    Undefined,
    /// The curvature is not above the resolution.
    Flat,
    /// The vector from the point to its centre of curvature.
    Centre(Vector3),
}

/// The sample of the curve at `t`.
fn to_centre(
    c: &Handle<dyn GeomCurve>,
    lprop: &mut CLProps,
    t: f64,
) -> Result<CentreSample, Failure> {
    lprop.set_parameter(t)?;
    if !lprop.is_tangent_defined()? {
        return Ok(CentreSample::Undefined);
    }
    // Not `Curvature > Resolution` (a NaN curvature is flat), as natively.
    let curvature = lprop.curvature()?.abs();
    if curvature.partial_cmp(&gp::resolution()) != Some(std::cmp::Ordering::Greater) {
        return Ok(CentreSample::Flat);
    }
    let p1: Point3 = c.value(t)?;
    let p2 = lprop.centre_of_curvature()?;
    Ok(CentreSample::Centre(Vector3::between(p1, p2)))
}

/// The samples of the interval `intrv` (1-based) of the bounds: 30 parameters from its
/// start.
fn samples(ti: &[f64], intrv: usize) -> impl Iterator<Item = (usize, f64)> + '_ {
    let step = (ti[intrv] - ti[intrv - 1]) / 30.0;
    (1..=30).scan(ti[intrv - 1], move |t, ii| {
        let current = *t;
        *t += step;
        Some((ii, current))
    })
}

/// The first vector to a centre of curvature and its interval, `Undefined` when a
/// tangent is undefined first, `Flat` when there is none.
fn first_centre(
    c: &Handle<dyn GeomCurve>,
    lprop: &mut CLProps,
    ti: &[f64],
) -> Result<(CentreSample, usize), Failure> {
    for intrv in 1..ti.len() {
        for (_, t) in samples(ti, intrv) {
            match to_centre(c, lprop, t)? {
                CentreSample::Flat => {}
                found => return Ok((found, intrv)),
            }
        }
    }
    Ok((CentreSample::Flat, 0))
}

/// `ChFi3d_IsSmooth(C)`: whether the tangent is defined along the curve and the vector to
/// the centre of curvature turns by at most 60 degrees and changes its length by at most
/// a factor 2 between successive samples (the last sample aside);
/// `is_smooth(&c)?`.
pub fn is_smooth(c: &Handle<dyn GeomCurve>) -> Result<bool, Failure> {
    let gac = GeomAdaptorCurve::with_curve(c)?;
    let nbintv = gac.nb_intervals(Continuity::CN)?;
    let mut bounds = Array1::with_bounds(1, nbintv + 1)
        .map_err(|e| Failure::with_message("Standard_RangeError", &format!("{e:?}")))?;
    gac.intervals(&mut bounds, Continuity::CN)?;
    let ti: Vec<f64> = bounds.iter().copied().collect();
    let mut lprop = CLProps::new(c.clone(), 2, gp::resolution())?;
    let (intrv_found, mut prev_vec) = match first_centre(c, &mut lprop, &ti)? {
        (CentreSample::Undefined, _) => return Ok(false),
        (CentreSample::Flat, _) => return Ok(true),
        (CentreSample::Centre(v), intrv) => (intrv, v),
    };
    let nbintv = ti.len() - 1;
    for intrv in intrv_found..=nbintv {
        for (ii, t) in samples(&ti, intrv) {
            let vec = match to_centre(c, &mut lprop, t)? {
                CentreSample::Undefined => return Ok(false),
                CentreSample::Flat => continue,
                CentreSample::Centre(vec) => vec,
            };
            if prev_vec.angle(vec)? > PI / 3.0 {
                return Ok(false);
            }
            let ratio = vec.magnitude() / prev_vec.magnitude();
            let ratio = if ratio < 1.0 { 1.0 / ratio } else { ratio };
            if ratio > 2.0 && (intrv != nbintv || ii != 30) {
                return Ok(false);
            }
            prev_vec = vec;
        }
    }
    Ok(true)
}
