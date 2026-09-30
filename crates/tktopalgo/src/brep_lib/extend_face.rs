// Rust translation of BRepLib::ExtendFace (BRepLib.cxx) at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `BRepLib::ExtendFace`: a face enlarged by a length in the chosen parameter
//! directions.
use super::make_face::BRepLibMakeFace;
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::brep_adaptor::BRepAdaptorSurface;
use rustycad_tkbrep::topods::{self, Face};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::native::{std_max, std_min};
use rustycad_tkg3d::adaptor3d::Adaptor3dSurface;
use rustycad_tkg3d::geom::{GeomBoundedSurface, GeomSurface, SurfaceBounds};
use rustycad_tkg3d::top_abs::Orientation;
use rustycad_tkgeombase::geom_lib;
use rustycad_tkmath::geom_abs::SurfaceType;

/// The directions to extend: UMin, UMax, VMin, VMax.
pub type ExtendDirections = [bool; 4];

/// The surface of the extended face and its bounds.
type BoundedFace = (Handle<dyn GeomSurface>, SurfaceBounds);

/// `BRepLib::ExtendFace(theF, theExtVal, theExtUMin, theExtUMax, theExtVMin,
/// theExtVMax, theFExtended)`: an analytic face widened by the parameter resolution of
/// `ext_val` within its surface bounds (the natural bounds when a periodic direction
/// closes), any other face on a bounded surface extended by length (GeomLib) in the
/// open directions, and the face itself otherwise; the face is rebuilt with
/// BRepLib_MakeFace at the face tolerance and has the input orientation;
/// `brep_lib::extend_face(&face, 1.0, [true, true, false, false])?`.
pub fn extend_face(
    face: &Face,
    ext_val: f64,
    directions: ExtendDirections,
) -> Result<Face, Failure> {
    // Get face bounds
    let bas = BRepAdaptorSurface::with_face(face, true)?;
    let bounds: SurfaceBounds = (
        bas.first_u_parameter()?,
        bas.last_u_parameter()?,
        bas.first_v_parameter()?,
        bas.last_v_parameter()?,
    );
    let tol: f64 = BRepTool::face_tolerance(face)?;
    let ty = bas.get_type()?;
    // treat analytical surfaces first
    let analytic = matches!(
        ty,
        SurfaceType::Plane
            | SurfaceType::Sphere
            | SurfaceType::Cylinder
            | SurfaceType::Torus
            | SurfaceType::Cone
    );
    let (surface, bounds) = if analytic {
        analytic_extension(&bas, bounds, ext_val, directions)?
    } else {
        let Some(extended) = general_extension(face, bounds, ext_val, directions)? else {
            return Ok(face.clone());
        };
        extended
    };
    let maker = BRepLibMakeFace::from_surface_bounds(&surface, bounds, tol)?;
    let mut extended = topods::face(maker.shape()?)?;
    if face.orientation() == Orientation::Reversed {
        extended.reverse();
    }
    Ok(extended)
}

/// The transformed basis surface of an analytic face and its bounds: the face bounds
/// moved to the first period, widened by the parameter resolutions of `ext_val`
/// within the surface bounds (one period at most), the surface bounds when a periodic
/// direction then covers its period.
fn analytic_extension(
    bas: &BRepAdaptorSurface,
    (mut fu_min, mut fu_max, mut fv_min, mut fv_max): SurfaceBounds,
    ext_val: f64,
    [ext_umin, ext_umax, ext_vmin, ext_vmax]: ExtendDirections,
) -> Result<BoundedFace, Failure> {
    // Get basis transformed basis surface
    let surf = bas.transformed_surface().geom_surface_transformed()?;
    let (su_min, su_max, sv_min, sv_max) = surf.get().expect("a surface").bounds();
    let u_period: Option<f64> = if bas.is_u_periodic()? {
        Some(bas.u_period()?)
    } else {
        None
    };
    let v_period: Option<f64> = if bas.is_v_periodic()? {
        Some(bas.v_period()?)
    } else {
        None
    };
    (fu_min, fu_max) = first_period((fu_min, fu_max), su_min, u_period);
    (fv_min, fv_max) = first_period((fv_min, fv_max), sv_min, v_period);
    // Enlarge the face
    let u_res: f64 = if ext_umin || ext_umax {
        bas.u_resolution(ext_val)?
    } else {
        0.
    };
    let v_res: f64 = if ext_vmin || ext_vmax {
        bas.v_resolution(ext_val)?
    } else {
        0.
    };
    (fu_min, fu_max) = enlarged(
        (fu_min, fu_max),
        (su_min, su_max),
        u_res,
        u_period,
        (ext_umin, ext_umax),
    );
    (fv_min, fv_max) = enlarged(
        (fv_min, fv_max),
        (sv_min, sv_max),
        v_res,
        v_period,
        (ext_vmin, ext_vmax),
    );
    // Check if the periodic surface should become closed.
    // In this case, use the basis surface with basis bounds.
    (fu_min, fu_max) = closed_period((fu_min, fu_max), (su_min, su_max), u_period);
    (fv_min, fv_max) = closed_period((fv_min, fv_max), (sv_min, sv_max), v_period);
    Ok((surf, (fu_min, fu_max, fv_min, fv_max)))
}

/// "Adjust face bounds to first period".
fn first_period((min, max): (f64, f64), s_min: f64, period: Option<f64>) -> (f64, f64) {
    let Some(period) = period else {
        return (min, max);
    };
    let delta: f64 = max - min;
    let min: f64 = std_max(s_min, min + period * ((s_min - min) / period).ceil());
    (min, min + delta)
}

/// One direction of "Enlarge the face".
fn enlarged(
    (mut min, mut max): (f64, f64),
    (s_min, s_max): (f64, f64),
    res: f64,
    period: Option<f64>,
    (ext_min, ext_max): (bool, bool),
) -> (f64, f64) {
    if ext_min {
        min = std_max(s_min, min - res);
    }
    if ext_max {
        let limit: f64 = match period {
            Some(period) => min + period,
            None => s_max,
        };
        max = std_min(limit, max + res);
    }
    (min, max)
}

/// The surface bounds of a periodic direction that covers its period.
fn closed_period(
    (min, max): (f64, f64),
    (s_min, s_max): (f64, f64),
    period: Option<f64>,
) -> (f64, f64) {
    match period {
        Some(period) if (max - min - period).abs() < precision::p_confusion() => (s_min, s_max),
        _ => (min, max),
    }
}

/// "General case": the bounded surface of the face extended by length (GeomLib) in
/// each requested direction that is open and finite (the VMin test reads the VMax
/// bound, as natively), the face bounds replaced by the new surface bounds there;
/// None when the surface is not bounded.
fn general_extension(
    face: &Face,
    (mut fu_min, mut fu_max, mut fv_min, mut fv_max): SurfaceBounds,
    ext_val: f64,
    [ext_umin, ext_umax, ext_vmin, ext_vmax]: ExtendDirections,
) -> Result<Option<BoundedFace>, Failure> {
    let mut sb = BRepTool::surface(face)?.cast::<dyn GeomBoundedSurface>();
    let Some(bounded) = sb.get() else {
        return Ok(None);
    };
    // Get surfaces bounds
    let (su_min, su_max, _, sv_max) = bounded.bounds();
    let (u_closed, v_closed) = (bounded.is_u_closed()?, bounded.is_v_closed()?);
    // Check if the extension in necessary directions is done
    let done = [
        ext_umin && !u_closed && !precision::is_infinite(su_min),
        ext_umax && !u_closed && !precision::is_infinite(su_max),
        ext_vmin && !v_closed && !precision::is_infinite(sv_max),
        ext_vmax && !v_closed && !precision::is_infinite(sv_max),
    ];
    for (extend, (in_u, after)) in
        done.iter()
            .zip([(true, false), (true, true), (false, false), (false, true)])
    {
        if *extend {
            geom_lib::extend_surf_by_length(&mut sb, ext_val, 1, in_u, after)?;
        }
    }
    let surface: Handle<dyn GeomSurface> = sb.cast();
    // Get new bounds
    let (su_min, su_max, sv_min, sv_max) = surface.get().expect("a surface").bounds();
    let new = [su_min, su_max, sv_min, sv_max];
    let mut bounds = [&mut fu_min, &mut fu_max, &mut fv_min, &mut fv_max];
    for (i, bound) in bounds.iter_mut().enumerate() {
        if done[i] {
            **bound = new[i];
        }
    }
    Ok(Some((surface, (fu_min, fu_max, fv_min, fv_max))))
}
