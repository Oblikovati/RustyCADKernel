// Rust translation of the static FC2D_make2d overloads of
// ModelingAlgorithms/TKBool/TopOpeBRepTool/TopOpeBRepTool_2d.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! A p-curve made for an edge on a face: the stored one, the projection of the 3d
//! curve, or the projection of the edge seen on its first ancestor face.
use super::{PCurveOnFace, fc2d_fancestor_e, fc2d_translate};
use crate::top_ope_brep_tool::curve_tool::{TopOpeBRepToolCurveTool, make_pcurve};
use crate::top_ope_brep_tool::tol::ftol_face_tolerances3d;
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::brep_adaptor::{BRepAdaptorCurve, BRepAdaptorSurface};
use rustycad_tkbrep::topods::{Edge, Face};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::adaptor3d::{Adaptor3dCurve, Adaptor3dSurface};
use rustycad_tkgeombase::proj_lib::ProjectedCurve;

/// The projection of the edge seen on its ancestor face `fe` onto `f`, on the edge
/// range, with the incoming tolerance.
fn projected_on_ancestor(e: &Edge, f: &Face, fe: &Face, tol: f64) -> Result<PCurveOnFace, Failure> {
    let bahs: Handle<dyn Adaptor3dSurface> =
        Handle::new(BRepAdaptorSurface::with_face(f, false)?).cast();
    let ahc: Handle<dyn Adaptor3dCurve> =
        Handle::new(BRepAdaptorCurve::with_edge_on_face(e, fe)?).cast();
    let tolin = ftol_face_tolerances3d(f, fe)?;
    let projcurv = ProjectedCurve::with_tolerance(&bahs, &ahc, tolin)?;
    let c2d = make_pcurve(&projcurv)?;
    let (first, last) = BRepTool::range(e)?;
    Ok((c2d, first, last, tol))
}

/// The p-curve of the edge: the stored one, else the projection of its 3d curve (on
/// its range when `trim3d`), else the projection on its ancestor face; null when it
/// has none of them; `translate` moves the made p-curves (not the stored one).
fn make2d_with(
    e: &Edge,
    f: &Face,
    tol: f64,
    trim3d: bool,
    translate: impl Fn(&Handle<dyn rustycad_tkg2d::geom2d::Geom2dCurve>) -> Result<(), Failure>,
) -> Result<PCurveOnFace, Failure> {
    let stored = BRepTool::curve_on_face(e, f)?;
    if !stored.pcurve.is_null() {
        return Ok((stored.pcurve, stored.first, stored.last, tol));
    }
    let (c2, f3d, l3d) = BRepTool::curve(e)?;
    if !c2.is_null() {
        let (first, last) = if trim3d { (f3d, l3d) } else { (0., 0.) };
        let (c2d, tol_reached) =
            TopOpeBRepToolCurveTool::make_pcurve_on_face(f.as_ref(), &c2, first, last)?;
        translate(&c2d)?;
        return Ok((c2d, f3d, l3d, tol_reached));
    }
    let fe = fc2d_fancestor_e(e)?;
    if fe.as_ref().is_null() {
        return Ok((Handle::default(), stored.first, stored.last, tol));
    }
    let made = projected_on_ancestor(e, f, &fe, tol)?;
    translate(&made.0)?;
    Ok(made)
}

/// `FC2D_make2d(E, F, f2d, l2d, tol, trim3d)`; `tol` is the incoming tolerance, kept
/// when no projection of the 3d curve sets it.
pub(super) fn fc2d_make2d(
    e: &Edge,
    f: &Face,
    tol: f64,
    trim3d: bool,
) -> Result<PCurveOnFace, Failure> {
    make2d_with(e, f, tol, trim3d, |_| Ok(()))
}

/// `FC2D_make2d(E, F, EF, f2d, l2d, tol, trim3d)`: the made p-curves translated onto
/// the iso of `ef` (FC2D_translate).
pub(super) fn fc2d_make2d_on_edge(
    e: &Edge,
    f: &Face,
    ef: &Edge,
    tol: f64,
    trim3d: bool,
) -> Result<PCurveOnFace, Failure> {
    make2d_with(e, f, tol, trim3d, |c2d| fc2d_translate(c2d, f, ef))
}
