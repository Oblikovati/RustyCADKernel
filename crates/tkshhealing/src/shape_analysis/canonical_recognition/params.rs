// Rust translation of the static GetNbPars, SetConicParameters, CompareConicParams,
// SetSurfParams, CompareSurfParams and DeviationSurfParams of
// ModelingAlgorithms/TKShHealing/ShapeAnalysis/ShapeAnalysis_CanonicalRecognition.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The parameters of the recognized entities (radii, angles) and their comparisons.
//! A parameter array is 1-based natively; here `params[0]` is `theParams(1)`.
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::adaptor3d::{Adaptor3dCurve, Adaptor3dSurface};
use rustycad_tkg3d::geom::{GeomCurve, GeomSurface};
use rustycad_tkg3d::geom_adaptor::{GeomAdaptorCurve, GeomAdaptorSurface};
use rustycad_tkmath::geom_abs::{CurveType, SurfaceType};
use rustycad_tkmath::gp::{Axis2, Axis3, Cone3};
use std::f64::consts::PI;

/// The static `GetNbPars(GeomAbs_CurveType)`: 1 for a circle, 2 for an ellipse, else 0.
pub(super) fn curve_nb_pars(target: CurveType) -> usize {
    match target {
        CurveType::Circle => 1,
        CurveType::Ellipse => 2,
        _ => 0,
    }
}

/// The static `GetNbPars(GeomAbs_SurfaceType)`: 1 for a cylinder or a sphere, 2 for a
/// cone, else 0.
pub(super) fn surface_nb_pars(target: SurfaceType) -> usize {
    match target {
        SurfaceType::Cylinder | SurfaceType::Sphere => 1,
        SurfaceType::Cone => 2,
        _ => 0,
    }
}

/// `theParams(i)` with the native bound check.
fn param(params: &[f64], i: usize) -> Result<f64, Failure> {
    params
        .get(i)
        .copied()
        .ok_or_else(|| out_of_range(i, params.len()))
}

/// `theParams(i) = value` with the native bound check.
fn set_param(params: &mut [f64], i: usize, value: f64) -> Result<(), Failure> {
    let len = params.len();
    *params.get_mut(i).ok_or_else(|| out_of_range(i, len))? = value;
    Ok(())
}

fn out_of_range(i: usize, len: usize) -> Failure {
    Failure::with_message(
        "Standard_OutOfRange",
        &format!(
            "ShapeAnalysis_CanonicalRecognition: parameter {}; expected 1..={len}",
            i + 1
        ),
    )
}

/// The static `SetConicParameters`: the frame and radii of the conic when the adaptor
/// sees it as `target`; false for a null curve or another type.
pub(super) fn set_conic_parameters(
    target: CurveType,
    conic: &Handle<dyn GeomCurve>,
    pos: &mut Axis2,
    params: &mut [f64],
) -> Result<bool, Failure> {
    if conic.is_null() {
        return Ok(false);
    }
    let adaptor = GeomAdaptorCurve::with_curve(conic)?;
    if adaptor.get_type()? != target {
        return Ok(false);
    }
    match target {
        CurveType::Line => pos.set_axis(adaptor.line()?.position())?,
        CurveType::Circle => {
            let circle = adaptor.circle()?;
            *pos = circle.position();
            set_param(params, 0, circle.radius())?;
        }
        CurveType::Ellipse => {
            let ellipse = adaptor.ellipse()?;
            *pos = ellipse.position();
            set_param(params, 0, ellipse.major_radius())?;
            set_param(params, 1, ellipse.minor_radius())?;
        }
        _ => return Ok(false),
    }
    Ok(true)
}

/// The static `CompareConicParams`: the radii agree within `tol` and the axes are
/// coaxial (either sense) within `tol / (2 PI)` and `tol` (any distance for lines).
pub(super) fn compare_conic_params(
    target: CurveType,
    tol: f64,
    ref_pos: &Axis2,
    ref_params: &[f64],
    pos: &Axis2,
    params: &[f64],
) -> Result<bool, Failure> {
    for i in 0..curve_nb_pars(target) {
        if (param(ref_params, i)? - param(params, i)?).abs() > tol {
            return Ok(false);
        }
    }
    let ang_tol = tol / (2.0 * PI);
    let lin_tol = if target == CurveType::Line {
        precision::infinite()
    } else {
        tol
    };
    let reference = ref_pos.axis();
    let axis = pos.axis();
    Ok(reference.is_coaxial(axis, ang_tol, lin_tol)
        || reference.is_coaxial(axis.reversed(), ang_tol, lin_tol))
}

/// The static `SetSurfParams`: the frame and parameters of the surface when the adaptor
/// sees it as `target`; false for a null surface, another type or too few parameters.
pub(super) fn set_surf_params(
    target: SurfaceType,
    surface: &Handle<dyn GeomSurface>,
    pos: &mut Axis3,
    params: &mut [f64],
) -> Result<bool, Failure> {
    if surface.is_null() {
        return Ok(false);
    }
    let adaptor = GeomAdaptorSurface::with_surface(surface)?;
    if adaptor.get_type()? != target || params.len() < surface_nb_pars(target) {
        return Ok(false);
    }
    match target {
        SurfaceType::Plane => *pos = adaptor.plane()?.position(),
        SurfaceType::Cylinder => {
            let cylinder = adaptor.cylinder()?;
            *pos = cylinder.position();
            set_param(params, 0, cylinder.radius())?;
        }
        SurfaceType::Cone => {
            let cone = adaptor.cone()?;
            *pos = cone.position();
            set_param(params, 0, cone.semi_angle())?;
            set_param(params, 1, cone.ref_radius())?;
        }
        SurfaceType::Sphere => {
            let sphere = adaptor.sphere()?;
            *pos = sphere.position();
            set_param(params, 0, sphere.radius())?;
        }
        _ => return Ok(false),
    }
    Ok(true)
}

/// The static `CompareSurfParams`: the first parameter agrees within `tol` (not for a
/// plane); spheres compare their centres; the others need coaxial axes (any distance
/// for cylinders and cones), and cones close apexes.
pub(super) fn compare_surf_params(
    target: SurfaceType,
    tol: f64,
    ref_pos: &Axis3,
    ref_params: &[f64],
    pos: &Axis3,
    params: &[f64],
) -> Result<bool, Failure> {
    if target != SurfaceType::Plane && (param(ref_params, 0)? - param(params, 0)?).abs() > tol {
        return Ok(false);
    }
    if target == SurfaceType::Sphere {
        return Ok(ref_pos.location().square_distance(&pos.location()) <= tol * tol);
    }
    let ang_tol = tol / (2.0 * PI);
    let lin_tol = if matches!(target, SurfaceType::Cylinder | SurfaceType::Cone) {
        precision::infinite()
    } else {
        tol
    };
    let reference = ref_pos.axis();
    let axis = pos.axis();
    if !(reference.is_coaxial(axis, ang_tol, lin_tol)
        || reference.is_coaxial(axis.reversed(), ang_tol, lin_tol))
    {
        return Ok(false);
    }
    if target != SurfaceType::Cone {
        return Ok(true);
    }
    let ref_cone = Cone3::new(*ref_pos, param(ref_params, 0)?, param(ref_params, 1)?)?;
    let cone = Cone3::new(*pos, param(params, 0)?, param(params, 1)?)?;
    Ok(ref_cone.apex().square_distance(&cone.apex()) <= tol * tol)
}

/// The static `DeviationSurfParams`: the difference of the first parameters (not for a
/// plane) plus the distance of the centres (spheres) or `1 - |cos|` of the axes.
pub(super) fn deviation_surf_params(
    target: SurfaceType,
    ref_pos: &Axis3,
    ref_params: &[f64],
    pos: &Axis3,
    params: &[f64],
) -> Result<f64, Failure> {
    let mut dev = 0.0;
    if target != SurfaceType::Plane {
        dev = (param(ref_params, 0)? - param(params, 0)?).abs();
    }
    if target == SurfaceType::Sphere {
        dev += ref_pos.location().distance(&pos.location());
    } else {
        dev += 1.0 - ref_pos.direction().dot(pos.direction()).abs();
    }
    Ok(dev)
}
