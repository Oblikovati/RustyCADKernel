// Rust translation of the iso computations of the BRepFill_MultiLine constructor and of the
// static isIsoU (ModelingAlgorithms/TKBool/BRepFill/BRepFill_MultiLine.cxx) at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The isos of a face of the multi-line: the iso of its edge and the transverse iso,
//! on their restriction domains, projected in planes.
use super::eps;
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::topexp::Explorer;
use rustycad_tkbrep::topods::{self, Edge, Face};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d_adaptor::Geom2dAdaptorCurve;
use rustycad_tkg3d::adaptor3d::{Adaptor3dCurve, Adaptor3dSurface};
use rustycad_tkg3d::geom::{GeomCurve, GeomPlane, GeomSurface};
use rustycad_tkg3d::geom_adaptor::{GeomAdaptorCurve, GeomAdaptorSurface};
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkgeombase::geom_proj_lib;
use rustycad_tkmath::elclib;
use rustycad_tkmath::geom_abs::{CurveType, SurfaceType};
use rustycad_tkmath::gp::{Axis3, Direction2, Direction3};

/// The isos of one face: on the edge (`u`, projected in the XY plane) and transverse
/// (`v`, projected in the plane normal to the first iso at its start), and the type of
/// the surface.
pub(super) struct FaceIsos {
    pub u: Geom2dAdaptorCurve,
    pub v: Geom2dAdaptorCurve,
    pub surface_type: SurfaceType,
}

/// `isIsoU(Face, Edge)`: whether the p-curve of the edge runs along V at its start;
/// Standard_ConstructionError for an edge without p-curve.
pub(super) fn is_iso_u(face: &Face, edge: &Edge) -> Result<bool, Failure> {
    let pc = BRepTool::curve_on_face(edge, face)?;
    let Some(c) = pc.pcurve.get() else {
        return Err(Failure::with_type(
            "Standard_ConstructionError",
            "BRepFill_MultiLine : Edge without PCurve",
            "BRepFill_MultiLine",
        ));
    };
    let d = Direction2::from_vector(c.dn(pc.first, 1)?)?;
    Ok(d.dot(Direction2::X).abs() < d.dot(Direction2::Y).abs())
}

/// The UV box of the ends of the edges of the face: (umin, umax, vmin, vmax).
fn uv_box(face: &Face) -> Result<(f64, f64, f64, f64), Failure> {
    let mut b: Option<(f64, f64, f64, f64)> = None;
    let mut exp = Explorer::new(face, ShapeEnum::Edge, ShapeEnum::Shape)?;
    while exp.more() {
        let (p1, p2) = BRepTool::uv_points_on_face(&topods::edge(exp.current())?, face)?;
        let e = (
            p1.x().min(p2.x()),
            p1.x().max(p2.x()),
            p1.y().min(p2.y()),
            p1.y().max(p2.y()),
        );
        b = Some(match b {
            None => e,
            Some((umin, umax, vmin, vmax)) => {
                (umin.min(e.0), umax.max(e.1), vmin.min(e.2), vmax.max(e.3))
            }
        });
        exp.next()?;
    }
    Ok(b.unwrap_or((0., 0., 0., 0.)))
}

/// The surface of the face, transformed by its location.
fn located_surface(face: &Face) -> Result<Handle<dyn GeomSurface>, Failure> {
    let (s, l) = BRepTool::surface_with_location(face)?;
    if l.is_identity() {
        return Ok(s);
    }
    let surface = s.get().ok_or_else(null_surface)?;
    Ok(surface.transformed(l.transformation())?.cast())
}

/// Whether a curve is a circle of a radius below `eps`.
fn tiny_circle(c: &Handle<dyn GeomCurve>) -> Result<bool, Failure> {
    let dummy = GeomAdaptorCurve::with_curve(c)?;
    Ok(dummy.get_type()? == CurveType::Circle && dummy.circle()?.radius() < eps())
}

/// The iso at `first` (`second` when it is a degenerated circle) of the surface along
/// U (`along_u`) or V.
fn iso_at(
    s: &dyn GeomSurface,
    along_u: bool,
    (first, second): (f64, f64),
) -> Result<Handle<dyn GeomCurve>, Failure> {
    let iso = |p: f64| if along_u { s.u_iso(p) } else { s.v_iso(p) };
    let c = iso(first)?;
    if tiny_circle(&c)? {
        return iso(second);
    }
    Ok(c)
}

/// The isos of a face: the iso of the edge at the minimal parameter (the maximal one
/// for an inverted or degenerated edge) and the transverse iso at the minimum, on
/// their domains (adjusted in the period), projected in planes.
pub(super) fn face_isos(
    face: &Face,
    edge: &Edge,
    inv: bool,
    is_iso_u: bool,
) -> Result<FaceIsos, Failure> {
    let (mut umin, mut umax, mut vmin, mut vmax) = uv_box(face)?;
    let s = located_surface(face)?;
    let surface = s.get().ok_or_else(null_surface)?;
    let at_min: bool = !BRepTool::degenerated(edge)? && !inv;
    let (uu, vv) = if is_iso_u {
        let range = if at_min { (umin, umax) } else { (umax, umin) };
        (iso_at(surface, true, range)?, surface.v_iso(vmin)?)
    } else {
        let range = if at_min { (vmin, vmax) } else { (vmax, vmin) };
        (iso_at(surface, false, range)?, surface.u_iso(umin)?)
    };
    if is_iso_u {
        (umin, umax, vmin, vmax) = (vmin, vmax, umin, umax);
    }
    let surface_type = GeomAdaptorSurface::with_surface(&s)?.get_type()?;
    (umin, umax) = adjusted(&uu, (umin, umax))?;
    (vmin, vmax) = adjusted(&vv, (vmin, vmax))?;
    projected(&uu, &vv, (umin, umax), (vmin, vmax), surface_type)
}

/// The range adjusted in the period of a periodic curve.
fn adjusted(c: &Handle<dyn GeomCurve>, (a, b): (f64, f64)) -> Result<(f64, f64), Failure> {
    let curve = c.get().ok_or_else(null_surface)?;
    if !curve.is_periodic() {
        return Ok((a, b));
    }
    Ok(elclib::adjust_periodic(
        curve.first_parameter(),
        curve.last_parameter(),
        precision::p_confusion(),
        a,
        b,
    ))
}

/// The isos projected: `uu` in the XY plane, `vv` in the plane normal to `uu` at its
/// start (X along Z).
fn projected(
    uu: &Handle<dyn GeomCurve>,
    vv: &Handle<dyn GeomCurve>,
    (umin, umax): (f64, f64),
    (vmin, vmax): (f64, f64),
    surface_type: SurfaceType,
) -> Result<FaceIsos, Failure> {
    let basis_plane: Handle<dyn GeomSurface> =
        Handle::new(GeomPlane::from_coefficients(0., 0., 1., 0.)?).cast();
    let u = Geom2dAdaptorCurve::with_range(&geom_proj_lib::curve2d(uu, &basis_plane)?, umin, umax)?;
    let (p, dz) = uu.get().ok_or_else(null_surface)?.d1(umin)?;
    let ref_plane: Handle<dyn GeomSurface> = Handle::new(GeomPlane::new(Axis3::new(
        p,
        Direction3::from_vector(dz)?,
        Direction3::Z,
    )?))
    .cast();
    let v = Geom2dAdaptorCurve::with_range(&geom_proj_lib::curve2d(vv, &ref_plane)?, vmin, vmax)?;
    Ok(FaceIsos { u, v, surface_type })
}

/// Standard_NullObject of a null surface or iso.
fn null_surface() -> Failure {
    Failure::with_type(
        "Standard_NullObject",
        "BRepFill_MultiLine: null surface or iso; expected a geometry",
        "BRepFill_MultiLine",
    )
}
