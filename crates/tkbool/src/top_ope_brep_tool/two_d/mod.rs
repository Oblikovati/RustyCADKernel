// Rust translation of ModelingAlgorithms/TKBool/TopOpeBRepTool/TopOpeBRepTool_2d.hxx/.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The `FC2D_*` functions: the p-curves of edges on faces, stored ones or new ones kept
//! in a per-thread cache (`thread_local` statics natively) that `FC2D_Prepare` resets
//! for a pair of shapes.
mod make2d;

use super::c2df::TopOpeBRepToolC2DF;
use super::tool_iso::uviso;
use make2d::{fc2d_make2d, fc2d_make2d_on_edge};
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::topexp;
use rustycad_tkbrep::topods::{self, Edge, Face, Shape};
use rustycad_tkbrep::toptools::{IndexedDataMapOfShapeListOfShape, ShapeMapHasher};
use rustycad_tkernel::ncollection::DataMap;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::Geom2dCurve;
use rustycad_tkg3d::top_abs::{Orientation, ShapeEnum};
use rustycad_tkmath::gp::{Direction2, Point2, Vector2};
use std::cell::RefCell;

/// A p-curve with its range and tolerance: `(C2D, f, l, tol)`.
pub type PCurveOnFace = (Handle<dyn Geom2dCurve>, f64, f64, f64);

/// The per-thread state of the FC2D functions (`GLOBAL_*` natively, null until
/// `FC2D_Prepare` or the first new p-curve).
#[derive(Default)]
struct Fc2dState {
    /// `GLOBAL_pmosloc2df`: the new p-curves of the edges.
    mosloc2df: Option<DataMap<Shape, Vec<TopOpeBRepToolC2DF>, ShapeMapHasher>>,
    /// `GLOBAL_pidmoslosc2df`: the faces of the edges of both shapes.
    idmoslosc2df: Option<IndexedDataMapOfShapeListOfShape>,
    /// `GLOBAL_pFc2df`: the null face returned for an edge without ancestor.
    fc2df: Option<Face>,
    /// `GLOBAL_pS1c2df`, `GLOBAL_pS2c2df`.
    shapes: Option<(Shape, Shape)>,
}

thread_local! {
    static FC2D_STATE: RefCell<Fc2dState> = RefCell::new(Fc2dState::default());
}

/// Standard_NullObject of the state used before `FC2D_Prepare` (a null pointer
/// dereference natively).
fn not_prepared(member: &str) -> Failure {
    Failure::with_type(
        "Standard_NullObject",
        &format!("{member}: FC2D_Prepare was not called; expected prepared shapes"),
        "TopOpeBRepTool",
    )
}

/// `FC2D_Prepare(S1, S2)`: clears the new p-curves and the ancestor faces, nullifies
/// the null face and keeps the shapes; `fc2d_prepare(&s1, &s2)`.
pub fn fc2d_prepare(s1: &Shape, s2: &Shape) -> i32 {
    FC2D_STATE.with(|state| {
        let mut st = state.borrow_mut();
        st.mosloc2df = Some(DataMap::default());
        st.idmoslosc2df = Some(IndexedDataMapOfShapeListOfShape::default());
        st.fc2df = Some(Face::default());
        st.shapes = Some((s1.clone(), s2.clone()));
    });
    0
}

/// `FC2D_FancestorE(E)` (static): the first face of the edge in the prepared shapes
/// (their edge/face ancestors mapped on the first call), or the null face.
fn fc2d_fancestor_e(e: &Edge) -> Result<Face, Failure> {
    FC2D_STATE.with(|state| {
        let mut st = state.borrow_mut();
        st.mosloc2df.get_or_insert_with(DataMap::default);
        let (s1, s2) = st
            .shapes
            .clone()
            .ok_or_else(|| not_prepared("FC2D_FancestorE"))?;
        let null_face = st
            .fc2df
            .clone()
            .ok_or_else(|| not_prepared("FC2D_FancestorE"))?;
        let map = st
            .idmoslosc2df
            .as_mut()
            .ok_or_else(|| not_prepared("FC2D_FancestorE"))?;
        if map.is_empty() {
            topexp::map_shapes_and_ancestors(&s1, ShapeEnum::Edge, ShapeEnum::Face, map)?;
            topexp::map_shapes_and_ancestors(&s2, ShapeEnum::Edge, ShapeEnum::Face, map)?;
        }
        let Some(lf) = map.seek(e.as_ref()) else {
            return Ok(null_face);
        };
        match lf.first() {
            Some(f) => topods::face(f),
            None => Ok(null_face),
        }
    })
}

/// `FC2D_HasC3D(E)`: whether the edge has a 3d curve; `fc2d_has_c3d(&edge)?`.
pub fn fc2d_has_c3d(e: &Edge) -> Result<bool, Failure> {
    Ok(!BRepTool::curve_with_location(e)?.curve.is_null())
}

/// `FC2D_HasCurveOnSurface(E, F)`: a stored or a new p-curve; `fc2d_has_curve_on_surface(&e, &f)?`.
pub fn fc2d_has_curve_on_surface(e: &Edge, f: &Face) -> Result<bool, Failure> {
    let hasold = fc2d_has_old_curve_on_surface(e, f)?.is_some();
    let hasnew = fc2d_has_new_curve_on_surface(e, f).is_some();
    Ok(hasold || hasnew)
}

/// `FC2D_HasOldCurveOnSurface(E, F, C2D, f2d, l2d, tol)`: the p-curve of
/// BRep_Tool::CurveOnSurface (the projection on a plane included) with the edge
/// tolerance; `None` when it is null; `fc2d_has_old_curve_on_surface(&e, &f)?`.
pub fn fc2d_has_old_curve_on_surface(e: &Edge, f: &Face) -> Result<Option<PCurveOnFace>, Failure> {
    let tol = BRepTool::edge_tolerance(e)?;
    let pc = BRepTool::curve_on_face(e, f)?;
    if pc.pcurve.is_null() {
        return Ok(None);
    }
    Ok(Some((pc.pcurve, pc.first, pc.last, tol)))
}

/// `FC2D_PNewCurveOnSurface(E, F)` (static): applies `f` to the new p-curve of the
/// edge on the equal face.
fn with_new_curve_on_surface<R>(
    e: &Edge,
    face: &Face,
    f: impl FnOnce(&mut TopOpeBRepToolC2DF) -> R,
) -> Option<R> {
    FC2D_STATE.with(|state| {
        let mut st = state.borrow_mut();
        let list = st.mosloc2df.as_mut()?.change_seek(e.as_ref())?;
        list.iter_mut().find(|c2df| c2df.is_face(face)).map(f)
    })
}

/// `FC2D_HasNewCurveOnSurface(E, F, C2D, f2d, l2d, tol)`: the new p-curve of the edge
/// on the face; `fc2d_has_new_curve_on_surface(&e, &f)`.
pub fn fc2d_has_new_curve_on_surface(e: &Edge, f: &Face) -> Option<PCurveOnFace> {
    with_new_curve_on_surface(e, f, |c2df| c2df.pc())
}

/// `FC2D_AddNewCurveOnSurface(PC, E, F, f, l, tol)`: the new p-curves of the edge
/// replaced by this one (1 for a null p-curve or before any preparation, else 0);
/// `fc2d_add_new_curve_on_surface(&pc, &e, &f, 0., 1., 1e-7)`.
pub fn fc2d_add_new_curve_on_surface(
    c2d: &Handle<dyn Geom2dCurve>,
    e: &Edge,
    f: &Face,
    f2d: f64,
    l2d: f64,
    tol: f64,
) -> i32 {
    if c2d.is_null() {
        return 1;
    }
    let c2df = TopOpeBRepToolC2DF::new(c2d, f2d, l2d, tol, f);
    FC2D_STATE.with(|state| {
        let mut st = state.borrow_mut();
        let Some(map) = st.mosloc2df.as_mut() else {
            return 1;
        };
        // The native Bind replaces the p-curves of the edge on the other faces.
        map.bind(e.as_ref().clone(), vec![c2df]);
        0
    })
}

/// `FC2D_MakeCurveOnSurface(E, F, f, l, tol, trim3d)`: a made p-curve, kept as new;
/// `tol` is the incoming value of the native in-out tolerance, returned when no
/// projection of a 3d curve sets it; `fc2d_make_curve_on_surface(&e, &f, 1e-7, false)?`.
pub fn fc2d_make_curve_on_surface(
    e: &Edge,
    f: &Face,
    tol: f64,
    trim3d: bool,
) -> Result<PCurveOnFace, Failure> {
    let made = fc2d_make2d(e, f, tol, trim3d)?;
    fc2d_add_new_curve_on_surface(&made.0, e, f, made.1, made.2, made.3);
    Ok(made)
}

/// `FC2D_CurveOnSurface(E, F, f, l, tol, trim3d)` (trim3d natively defaulted to
/// false): the stored, else the new, else a made p-curve;
/// `fc2d_curve_on_surface(&e, &f, false)?`.
pub fn fc2d_curve_on_surface(e: &Edge, f: &Face, trim3d: bool) -> Result<PCurveOnFace, Failure> {
    if let Some(old) = fc2d_has_old_curve_on_surface(e, f)? {
        return Ok(old);
    }
    if let Some(new) = fc2d_has_new_curve_on_surface(e, f) {
        return Ok(new);
    }
    // The tolerance is the edge one, set by FC2D_HasOldCurveOnSurface.
    fc2d_make_curve_on_surface(e, f, BRepTool::edge_tolerance(e)?, trim3d)
}

/// `FC2D_EditableCurveOnSurface(E, F, f, l, tol, trim3d)`: a copy of the stored
/// p-curve, else the new one, else a made one; `fc2d_editable_curve_on_surface(&e, &f, false)?`.
pub fn fc2d_editable_curve_on_surface(
    e: &Edge,
    f: &Face,
    trim3d: bool,
) -> Result<PCurveOnFace, Failure> {
    if let Some((c2d, f2d, l2d, tol)) = fc2d_has_old_curve_on_surface(e, f)? {
        let curve = c2d
            .get()
            .ok_or_else(|| not_prepared("FC2D_EditableCurveOnSurface"))?;
        return Ok((curve.copy()?.cast(), f2d, l2d, tol));
    }
    if let Some(new) = fc2d_has_new_curve_on_surface(e, f) {
        return Ok(new);
    }
    fc2d_make_curve_on_surface(e, f, BRepTool::edge_tolerance(e)?, trim3d)
}

/// `FC2D_translate(C2D, E, F, EF)` (static): on a periodic surface, an iso p-curve
/// translated in place onto the iso of the first UV point of `ef` (more than 1e-6
/// away).
fn fc2d_translate(c2d: &Handle<dyn Geom2dCurve>, f: &Face, ef: &Edge) -> Result<(), Failure> {
    let (s1, _) = BRepTool::surface_with_location(f)?;
    let surface = s1.get().ok_or_else(|| not_prepared("FC2D_translate"))?;
    let isperio = surface.is_u_periodic() || surface.is_v_periodic();
    let (mut d2d, mut o2d) = (Direction2::X, Point2::default());
    let (uviso_ok, isuiso, isviso) = uviso(c2d, &mut d2d, &mut o2d)?;
    if !(isperio && uviso_ok && !ef.as_ref().is_null()) {
        return Ok(());
    }
    let ffor = f.oriented(Orientation::Forward);
    let (p1, _) = BRepTool::uv_points_on_face(ef, &topods::face(&ffor)?)?;
    let (p_ef, p_c2d) = if isuiso {
        (p1.x(), o2d.x())
    } else {
        (p1.y(), o2d.y())
    };
    let factor = p_ef - p_c2d;
    if factor.abs() <= 1.0e-6 {
        return Ok(());
    }
    let transl = if isviso {
        Vector2::new(0., 1.)
    } else {
        Vector2::new(1., 0.)
    };
    let transl = Vector2::new(transl.x() * factor, transl.y() * factor);
    c2d.get()
        .ok_or_else(|| not_prepared("FC2D_translate"))?
        .translate(transl)
}

/// `FC2D_CurveOnSurface(E, F, EF, f, l, tol, trim3d)`: the stored p-curve, else the new
/// one translated onto `ef` (and kept), else a made one translated onto `ef` (kept as
/// new); `fc2d_curve_on_surface_on_edge(&e, &f, &ef, false)?`.
pub fn fc2d_curve_on_surface_on_edge(
    e: &Edge,
    f: &Face,
    ef: &Edge,
    trim3d: bool,
) -> Result<PCurveOnFace, Failure> {
    if let Some(old) = fc2d_has_old_curve_on_surface(e, f)? {
        return Ok(old);
    }
    if let Some(new) = fc2d_has_new_curve_on_surface(e, f) {
        fc2d_translate(&new.0, f, ef)?;
        with_new_curve_on_surface(e, f, |c2df| c2df.set_pc(&new.0, new.1, new.2, new.3));
        return Ok(new);
    }
    let made = fc2d_make2d_on_edge(e, f, ef, BRepTool::edge_tolerance(e)?, trim3d)?;
    fc2d_add_new_curve_on_surface(&made.0, e, f, made.1, made.2, made.3);
    Ok(made)
}
