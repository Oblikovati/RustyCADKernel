// Rust translation of the static CanUseEdges and FindExactUVBounds of BRepBndLib.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The file-static `CanUseEdges` and `FindExactUVBounds` of BRepBndLib.cxx, which
//! `AddOptimal` uses to choose and bound the surface patch of a face.
use crate::brep_top_adaptor::errors::null_object;
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::brep_adaptor::BRepAdaptorSurface;
use rustycad_tkbrep::topexp::Explorer;
use rustycad_tkbrep::topods::{self, Edge, Face};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::adaptor3d::Adaptor3dSurface;
use rustycad_tkg3d::top_abs::{Orientation, ShapeEnum};
use rustycad_tkgeombase::bnd_lib::Add2dCurve;
use rustycad_tkmath::bnd::BoundingBox2;
use rustycad_tkmath::geom_abs::{CurveType, SurfaceType};
use rustycad_tkmath::gp::{self, Vector2};

/// The UV patch `(umin, umax, vmin, vmax)` of a face.
pub(super) type UvPatch = [f64; 4];

/// `std::max(a, b)`: `a` unless `a < b`.
pub(super) fn std_max(a: f64, b: f64) -> f64 {
    if a < b { b } else { a }
}
/// `std::min(a, b)`: `a` unless `b < a`.
fn std_min(a: f64, b: f64) -> f64 {
    if b < a { b } else { a }
}

/// `CanUseEdges(BS)`: whether the boxes of the edges alone bound the face (planes,
/// cylinders, cones, extrusions, revolutions of a line, offsets of such a surface,
/// splines of degree 1 without inner knot in one direction).
pub(super) fn can_use_edges(surface: &dyn Adaptor3dSurface) -> Result<bool, Failure> {
    Ok(match surface.get_type()? {
        SurfaceType::Plane
        | SurfaceType::Cylinder
        | SurfaceType::Cone
        | SurfaceType::SurfaceOfExtrusion => true,
        SurfaceType::SurfaceOfRevolution => surface.basis_curve()?.get_type()? == CurveType::Line,
        SurfaceType::OffsetSurface => can_use_edges(&*surface.basis_surface()?)?,
        SurfaceType::BSplineSurface => {
            let spline = surface.bspline()?;
            (spline.u_degree() == 1 && spline.nb_u_knots() == 2)
                || (spline.v_degree() == 1 && spline.nb_v_knots() == 2)
        }
        SurfaceType::BezierSurface => {
            let patch = surface.bezier()?;
            patch.u_degree() == 1 || patch.v_degree() == 1
        }
        _ => false,
    })
}

/// `FindExactUVBounds(FF, umin, umax, vmin, vmax, Tol, isNaturalRestriction)`: the
/// UV patch of the face and whether it is its natural restriction (the flag of the
/// face, or at most four pcurves lying two on U and two on V boundaries of the
/// surface); else the optimal 2d box of its pcurves, clipped to the surface bounds (or
/// shrunk to one period on a periodic direction).
pub(super) fn find_exact_uv_bounds(face_in: &Face, tol: f64) -> Result<(UvPatch, bool), Failure> {
    let face = topods::face(&face_in.oriented(Orientation::Forward))?;
    let flagged: bool = BRepTool::natural_restriction(&face)?; // "Can we trust this flag?"
    let surface = BRepAdaptorSurface::with_face(&face, false)?;
    let natural = flagged || pcurves_on_bounds(&face, &surface, tol)?;
    let surface_patch: UvPatch = [
        surface.first_u_parameter()?,
        surface.last_u_parameter()?,
        surface.first_v_parameter()?,
        surface.last_v_parameter()?,
    ];
    if natural {
        return Ok((surface_patch, true));
    }
    let patch = pcurves_box(&face, &surface, tol, surface_patch)?;
    Ok((clip_to_surface(face_in, patch)?, false))
}

/// The resolutions `(TolU, TolV)` of `tol`, at least Precision::PConfusion().
pub(super) fn uv_tolerances(
    surface: &dyn Adaptor3dSurface,
    tol: f64,
) -> Result<(f64, f64), Failure> {
    let pconfusion = precision::p_confusion();
    Ok((
        std_max(surface.u_resolution(tol)?, pconfusion),
        std_max(surface.v_resolution(tol)?, pconfusion),
    ))
}

/// The natural restriction check of an unflagged face: its first edges (up to four)
/// have a pcurve whose middle lies on a U (or V) boundary of the surface, tangent to
/// it; exactly two on U boundaries and two on V boundaries.
fn pcurves_on_bounds(face: &Face, surface: &BRepAdaptorSurface, tol: f64) -> Result<bool, Failure> {
    let patch = [
        surface.first_u_parameter()?,
        surface.last_u_parameter()?,
        surface.first_v_parameter()?,
        surface.last_v_parameter()?,
    ];
    let tols = uv_tolerances(surface, tol)?;
    let (mut nu, mut nv) = (0, 0);
    for (count, edge) in Explorer::new(face, ShapeEnum::Edge, ShapeEnum::Shape)?.enumerate() {
        if count >= 4 {
            break;
        }
        let Some(side) = boundary_side(&topods::edge(&edge)?, face, patch, tols)? else {
            break;
        };
        let counter = if side { &mut nu } else { &mut nv };
        *counter += 1;
        if *counter > 2 {
            break;
        }
    }
    Ok(nu == 2 && nv == 2)
}

/// Whether the middle of the edge's pcurve lies on a U boundary (true) or a V boundary
/// (false) of `patch`, along it within Precision::PConfusion(); `None` stops the check
/// (no pcurve, a null derivative, off the boundaries or across them).
fn boundary_side(
    edge: &Edge,
    face: &Face,
    [umin, umax, vmin, vmax]: UvPatch,
    (tol_u, tol_v): (f64, f64),
) -> Result<Option<bool>, Failure> {
    let on_face = BRepTool::curve_on_face(edge, face)?;
    let Some(pcurve) = on_face.pcurve.get() else {
        return Ok(None);
    };
    let (point, mut tangent) = pcurve.d1((on_face.first + on_face.last) / 2.)?;
    let magn: f64 = tangent.square_magnitude();
    if magn < gp::resolution() {
        return Ok(None);
    }
    tangent /= magn.sqrt();
    // On a periodic surface the native code calls ElCLib::InPeriod(u, umin, umax) (and
    // on v) but discards the result, so the middle point is never moved into the period.
    let (u, v) = (point.x(), point.y());
    let side = if (u - umin).abs() <= tol_u || (u - umax).abs() <= tol_u {
        (true, Vector2::new(0., 1.))
    } else if (v - vmin).abs() <= tol_v || (v - vmax).abs() <= tol_v {
        (false, Vector2::new(1., 0.))
    } else {
        return Ok(None);
    };
    let d: f64 = side.1.dot(tangent);
    Ok((1. - d.abs() <= precision::p_confusion()).then_some(side.0))
}

/// The optimal 2d box of the pcurves (tolerance max(TolU, TolV)), or `patch` when no
/// edge has a pcurve.
fn pcurves_box(
    face: &Face,
    surface: &BRepAdaptorSurface,
    tol: f64,
    patch: UvPatch,
) -> Result<UvPatch, Failure> {
    let (tol_u, tol_v) = uv_tolerances(surface, tol)?;
    let tol_uv = std_max(tol_u, tol_v);
    let mut uv_box = BoundingBox2::default();
    for edge in Explorer::new(face, ShapeEnum::Edge, ShapeEnum::Shape)? {
        let on_face = BRepTool::curve_on_face(&topods::edge(&edge)?, face)?;
        if !on_face.pcurve.is_null() {
            Add2dCurve::add_optimal(
                &on_face.pcurve,
                on_face.first,
                on_face.last,
                tol_uv,
                &mut uv_box,
            )?;
        }
    }
    if uv_box.is_void() {
        return Ok(patch);
    }
    let [umin, vmin, umax, vmax] = uv_box.get()?;
    Ok([umin, umax, vmin, vmax])
}

/// Clips the patch to the bounds of the stored surface in a non-periodic direction,
/// and shrinks it symmetrically to one period in a periodic direction.
fn clip_to_surface(face: &Face, [umin, umax, vmin, vmax]: UvPatch) -> Result<UvPatch, Failure> {
    let (stored, _) = BRepTool::surface_with_location(face)?;
    let surface = stored
        .get()
        .ok_or_else(|| null_object("BRepBndLib: Bounds() of the null surface of a face"))?;
    let (su1, su2, sv1, sv2) = surface.bounds();
    let (umin, umax) = clip_range((umin, umax), (su1, su2), surface.is_u_periodic(), || {
        surface.u_period()
    })?;
    let (vmin, vmax) = clip_range((vmin, vmax), (sv1, sv2), surface.is_v_periodic(), || {
        surface.v_period()
    })?;
    Ok([umin, umax, vmin, vmax])
}

/// One direction of [`clip_to_surface`].
fn clip_range(
    (low, high): (f64, f64),
    (first, last): (f64, f64),
    periodic: bool,
    period: impl FnOnce() -> Result<f64, Failure>,
) -> Result<(f64, f64), Failure> {
    if !periodic {
        return Ok((std_max(first, low), std_min(last, high)));
    }
    let period: f64 = period()?;
    if high - low > period {
        let delta: f64 = high - low - period;
        return Ok((low + delta / 2., high - delta / 2.));
    }
    Ok((low, high))
}
