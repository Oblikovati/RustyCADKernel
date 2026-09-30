// Rust translation of GeomFill_Gordon.cxx (fallbackSampleCount, unitParameters,
// approximateGrid, profileSampleGrid, guideSampleGrid, approximatePreparedNetwork and
// the deviation measures) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The sampled fallback of GeomFill_Gordon and the curve-to-surface deviations.
use super::gordon_contacts::adaptor_d0;
use super::gordon_params::null_curve;
use super::network_surface_skin::point_of;
use super::network_surface_skin::{array, grid};
use crate::geom_api::PointsToBSplineSurface;
use rustycad_tkernel::ncollection::{Array1, Array2};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::geom::{GeomBSplineCurve, GeomBSplineSurface, GeomSurface};
use rustycad_tkg3d::geom_grid_eval::{GeomGridEvalCurve, GeomGridEvalSurface};
use rustycad_tkgeombase::approx::ParametrizationType;
use rustycad_tkmath::geom_abs::Shape;
use rustycad_tkmath::gp::Point3;

/// `fallbackSampleCount`: the curve count (at least 2), the largest pole count, at
/// least half the maximal B-spline degree.
fn fallback_sample_count(the_curves: &[Handle<GeomBSplineCurve>]) -> usize {
    let mut a_nb_samples: usize = the_curves.len().max(2);
    for curve in the_curves.iter().filter_map(|c| c.get()) {
        a_nb_samples = a_nb_samples.max(curve.nb_poles() as usize);
    }
    a_nb_samples.max(GeomBSplineCurve::max_degree() as usize / 2)
}

/// `unitParameters`: `n` regular parameters on `[0, 1]`.
fn unit_parameters(the_nb_samples: usize) -> Result<Array1<'static, f64>, Failure> {
    let mut a_parameters: Array1<'static, f64> = array(the_nb_samples)?;
    for i in 0..the_nb_samples {
        a_parameters[i as i32 + 1] = i as f64 / (the_nb_samples - 1) as f64;
    }
    Ok(a_parameters)
}

/// `approximateGrid`: GeomAPI_PointsToBSplineSurface on the grid (isoparametric,
/// degrees 1 to the maximum, C1, the tolerance at least Precision::Confusion()); None
/// when it is not done.
fn approximate_grid(
    the_points: &Array2<'_, Point3>,
    the_tolerance: f64,
) -> Result<Option<Handle<GeomBSplineSurface>>, Failure> {
    let mut an_approx = PointsToBSplineSurface::new();
    let settings = (
        1,
        GeomBSplineSurface::max_degree(),
        Shape::C1,
        the_tolerance.max(precision::confusion()),
    );
    an_approx.init_parametrization(
        the_points,
        ParametrizationType::IsoParametric,
        settings,
        false,
    )?;
    if !an_approx.is_done() {
        return Ok(None);
    }
    Ok(Some(an_approx.surface()?))
}

/// `profileSampleGrid` and `guideSampleGrid`: each curve sampled at `n` regular
/// parameters, a curve per column (`along_rows` false) or per row.
fn sample_grid(
    the_curves: &[Handle<GeomBSplineCurve>],
    the_nb_samples: usize,
    along_rows: bool,
) -> Result<Array2<'static, Point3>, Failure> {
    let params: Array1<'static, f64> = unit_parameters(the_nb_samples)?;
    let (n, m) = (the_nb_samples as i32, the_curves.len() as i32);
    let mut a_points: Array2<'static, Point3> = if along_rows { grid(m, n)? } else { grid(n, m)? };
    for (ci, curve) in the_curves.iter().enumerate() {
        let points = GeomGridEvalCurve::from_curve(&curve.cast()).evaluate_grid(&params)?;
        for k in 1..=n {
            let cell = if along_rows {
                (ci as i32 + 1, k)
            } else {
                (k, ci as i32 + 1)
            };
            a_points[cell] = points[points.lower() + k - 1];
        }
    }
    Ok(a_points)
}

/// `approximatePreparedNetwork`: the approximate profile skin, guide skin and
/// reference surface combined on a regular grid (`profile + guide - reference`) and
/// approximated again; None when an approximation fails.
pub(crate) fn approximate_prepared_network(
    (the_profiles, the_guides): (&[Handle<GeomBSplineCurve>], &[Handle<GeomBSplineCurve>]),
    the_intersections: &Array2<'_, Point3>,
    the_tolerance: f64,
) -> Result<Option<Handle<GeomBSplineSurface>>, Failure> {
    let (nu, nv) = (
        fallback_sample_count(the_profiles),
        fallback_sample_count(the_guides),
    );
    let profile_surface = approximate_grid(&sample_grid(the_profiles, nu, false)?, the_tolerance)?;
    let guide_surface = approximate_grid(&sample_grid(the_guides, nv, true)?, the_tolerance)?;
    let reference_surface = approximate_grid(the_intersections, the_tolerance)?;
    let (Some(p), Some(g), Some(r)) = (profile_surface, guide_surface, reference_surface) else {
        return Ok(None);
    };
    let (u_params, v_params) = (unit_parameters(nu)?, unit_parameters(nv)?);
    let evaluate = |s: &Handle<GeomBSplineSurface>| {
        GeomGridEvalSurface::from_surface(&s.cast::<dyn GeomSurface>())
            .evaluate_grid(&u_params, &v_params)
    };
    let (pp, gp, rp) = (evaluate(&p)?, evaluate(&g)?, evaluate(&r)?);
    let mut a_points: Array2<'static, Point3> = grid(nu as i32, nv as i32)?;
    for (i, cell) in a_points.iter_mut().enumerate() {
        let (a, b, c) = (pp.as_slice()[i], gp.as_slice()[i], rp.as_slice()[i]);
        *cell = point_of(a.xyz() + b.xyz() - c.xyz());
    }
    approximate_grid(&a_points, the_tolerance)
}

// The first and last knots of a surface in U (`the_is_u`) or V, and the parameter at a
// unit ratio of that range (`surfaceParameter`).
fn surface_parameter(
    the_surface: &GeomBSplineSurface,
    the_is_u: bool,
    the_unit_parameter: f64,
) -> Result<f64, Failure> {
    let (a_first, a_last) = if the_is_u {
        (
            the_surface.u_knot(1)?,
            the_surface.u_knot(the_surface.nb_u_knots())?,
        )
    } else {
        (
            the_surface.v_knot(1)?,
            the_surface.v_knot(the_surface.nb_v_knots())?,
        )
    };
    Ok(a_first + the_unit_parameter * (a_last - a_first))
}

/// `maxCurveDeviation`: the largest distance between the curve and its isoparameter of
/// the surface, sampled with degree(curve) + degree(surface) intervals per non-empty
/// knot span of the curve.
fn max_curve_deviation(
    the_surface: &GeomBSplineSurface,
    the_curve: &Handle<GeomBSplineCurve>,
    the_fixed_parameter: f64,
    the_is_profile: bool,
) -> Result<f64, Failure> {
    let curve = the_curve.get().ok_or_else(null_curve)?;
    let mut a_max_deviation: f64 = 0.0;
    let per_span: usize = curve.degree() as usize
        + if the_is_profile {
            the_surface.u_degree()
        } else {
            the_surface.v_degree()
        } as usize;
    for k in 1..curve.nb_knots() {
        let (a_first, a_last) = (curve.knot(k)?, curve.knot(k + 1)?);
        if a_last - a_first <= precision::p_confusion() {
            continue;
        }
        // The sampled difference combines the curve and surface polynomial orders.
        for i in 0..=per_span {
            let the_parameter: f64 = a_first + (i as f64 / per_span as f64) * (a_last - a_first);
            let (unit_u, unit_v) = if the_is_profile {
                (the_parameter, the_fixed_parameter)
            } else {
                (the_fixed_parameter, the_parameter)
            };
            let a_u: f64 = surface_parameter(the_surface, true, unit_u)?;
            let a_v: f64 = surface_parameter(the_surface, false, unit_v)?;
            let distance =
                adaptor_d0(the_curve, the_parameter)?.distance(&the_surface.value(a_u, a_v)?);
            a_max_deviation = a_max_deviation.max(distance);
        }
    }
    Ok(a_max_deviation)
}

/// `maxProfileDeviation` (`the_is_profile`) and `maxGuideDeviation`: the largest
/// deviation over the curves of one family, each on its isoparameter.
pub(crate) fn max_family_deviation(
    the_surface: &Handle<GeomBSplineSurface>,
    the_curves: &[Handle<GeomBSplineCurve>],
    the_params: &[f64],
    the_is_profile: bool,
) -> Result<f64, Failure> {
    let surface = the_surface.get().ok_or_else(|| {
        Failure::with_type("Standard_NullObject", "GeomFill_Gordon - null surface", "")
    })?;
    let mut a_max_deviation: f64 = 0.0;
    for (curve, param) in the_curves.iter().zip(the_params) {
        let fixed: f64 = surface_parameter(surface, !the_is_profile, *param)?;
        a_max_deviation =
            a_max_deviation.max(max_curve_deviation(surface, curve, fixed, the_is_profile)?);
    }
    Ok(a_max_deviation)
}
