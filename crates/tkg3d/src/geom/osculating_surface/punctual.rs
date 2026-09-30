// Rust translation of ModelingData/TKG3d/Geom/Geom_OsculatingSurface.cxx (isQPunctual) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `isQPunctual`: whether a boundary iso of the basis is quasi-punctual, sampled
//! through a `GeomAdaptor_Surface` as natively (its Bezier and B-spline caches, and
//! the local evaluators at the bounds of a B-spline).
use crate::adaptor3d::Adaptor3dSurface;
use crate::geom_adaptor::GeomAdaptorSurface;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg2d::geom2d::native::std_max;

/// `isQPunctual`: whether the largest tangent along the iso (11 samples) lies in
/// `[tol_min, tol_max]`; `is_q_punctual(&adaptor, v1, true, 0.0, 1e-7)?`.
pub(super) fn is_q_punctual(
    adaptor: &GeomAdaptorSurface,
    parameter: f64,
    iso_v: bool,
    tol_min: f64,
    tol_max: f64,
) -> Result<bool, Failure> {
    const THE_NB_SUBDIVISIONS: i32 = 10;
    let (p_min, p_max) = if iso_v {
        (adaptor.first_u_parameter()?, adaptor.last_u_parameter()?)
    } else {
        (adaptor.first_v_parameter()?, adaptor.last_v_parameter()?)
    };
    let range: f64 = p_max - p_min;
    let mut d1_norm_max: f64 = 0.0;
    // Keep the historical sampling density while avoiding floating-point loop progress.
    for index in 0..=THE_NB_SUBDIVISIONS {
        let sample: f64 = if index == THE_NB_SUBDIVISIONS {
            p_max
        } else {
            p_min + range * f64::from(index) / f64::from(THE_NB_SUBDIVISIONS)
        };
        let norm: f64 = if iso_v {
            adaptor.eval_d1(sample, parameter)?.d1u.magnitude()
        } else {
            adaptor.eval_d1(parameter, sample)?.d1v.magnitude()
        };
        d1_norm_max = std_max(d1_norm_max, norm);
    }
    Ok(!(d1_norm_max > tol_max || d1_norm_max < tol_min))
}
