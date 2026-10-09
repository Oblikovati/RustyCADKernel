// Rust translation of TopOpeBRepTool_TOOL::UVISO (the three overloads in
// ModelingAlgorithms/TKBool/TopOpeBRepTool/TopOpeBRepTool_TOOL.cxx) at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! Whether a p-curve is an iso-U or iso-V line.
use super::c2df::TopOpeBRepToolC2DF;
use super::geometry::basis_curve2d;
use super::two_d::{
    fc2d_add_new_curve_on_surface, fc2d_editable_curve_on_surface, fc2d_has_old_curve_on_surface,
};
use rustycad_tkbrep::topods::{Edge, Face};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::{Geom2dCurve, Geom2dLine};
use rustycad_tkmath::gp::{Direction2, Point2};

/// `(UVISO, isoU, isoV)`.
pub type UvIsoFlags = (bool, bool, bool);

/// `TopOpeBRepTool_TOOL::UVISO(PC, isoU, isoV, d2d, o2d)`: whether the basis of the
/// p-curve is a line along V (iso-U) or U (iso-V); `d2d` is written for any line and
/// `o2d` for an iso, as natively (both untouched otherwise);
/// `let (iso, isou, isov) = uviso(&pc, &mut d2d, &mut o2d)?;`.
pub fn uviso(
    pc: &Handle<dyn Geom2dCurve>,
    d2d: &mut Direction2,
    o2d: &mut Point2,
) -> Result<UvIsoFlags, Failure> {
    if pc.is_null() {
        return Ok((false, false, false));
    }
    let lll = basis_curve2d(pc)?;
    let line: Handle<Geom2dLine> = lll.cast();
    let Some(l) = line.get() else {
        return Ok((false, false, false));
    };
    *d2d = l.direction();
    let tol: f64 = precision::parametric(precision::confusion());
    let (iso_u, iso_v) = (d2d.x().abs() < tol, d2d.y().abs() < tol);
    if !(iso_u || iso_v) {
        return Ok((false, iso_u, iso_v));
    }
    *o2d = l.location();
    Ok((true, iso_u, iso_v))
}

/// `TopOpeBRepTool_TOOL::UVISO(E, F, isoU, isoV, d2d, o2d)`: on the editable p-curve of
/// the edge on the face (kept as new when it was not stored);
/// `uviso_edge(&e, &f, &mut d2d, &mut o2d)?`.
pub fn uviso_edge(
    e: &Edge,
    f: &Face,
    d2d: &mut Direction2,
    o2d: &mut Point2,
) -> Result<UvIsoFlags, Failure> {
    let hasold = fc2d_has_old_curve_on_surface(e, f)?.is_some();
    let (pc, first, last, tol) = fc2d_editable_curve_on_surface(e, f, false)?;
    if !hasold {
        fc2d_add_new_curve_on_surface(&pc, e, f, first, last, tol);
    }
    uviso(&pc, d2d, o2d)
}

/// `TopOpeBRepTool_TOOL::UVISO(C2DF, isoU, isoV, d2d, o2d)`: on the p-curve of the
/// C2DF; `uviso_c2df(&c2df, &mut d2d, &mut o2d)?`.
pub fn uviso_c2df(
    c2df: &TopOpeBRepToolC2DF,
    d2d: &mut Direction2,
    o2d: &mut Point2,
) -> Result<UvIsoFlags, Failure> {
    let (pc, _, _, _) = c2df.pc();
    uviso(&pc, d2d, o2d)
}
