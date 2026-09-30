// Rust translation of BRepLib::CheckSameRange and BRepLib::SameRange (BRepLib.cxx) at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `BRepLib::CheckSameRange` and `BRepLib::SameRange`: the parameter ranges of the
//! curve representations of an edge.
use rustycad_tkbrep::brep::access::edge_tshape;
use rustycad_tkbrep::brep::update_curves::as_gcurve;
use rustycad_tkbrep::brep::{BRepBuilder, BRepGCurve, BRepTool};
use rustycad_tkbrep::topods::Edge;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::Geom2dCurve;
use rustycad_tkgeombase::geom_lib;

/// The parametric representations of the edge, in list order.
pub(crate) fn gcurves(edge: &Edge, member: &str) -> Result<Vec<Handle<dyn BRepGCurve>>, Failure> {
    let curves = edge_tshape(edge, member)?.curves();
    Ok(curves
        .iter()
        .map(as_gcurve)
        .filter(|g| g.is_some())
        .collect())
}

/// `BRepLib::CheckSameRange(E, Confusion)`: whether every parametric representation of
/// the edge has the range of the first one within `confusion`, ignoring the SameRange
/// flag; `brep_lib::check_same_range(&edge, 1e-12)?`.
pub fn check_same_range(edge: &Edge, confusion: f64) -> Result<bool, Failure> {
    let mut current: Option<(f64, f64)> = None;
    for gcurve in gcurves(edge, "BRepLib::CheckSameRange")? {
        let (first, last) = gcurve.get().map(|g| g.range()).unwrap_or_default();
        let Some((current_first, current_last)) = current else {
            current = Some((first, last));
            continue;
        };
        if !((current_first - first).abs() <= confusion && (current_last - last).abs() <= confusion)
        {
            return Ok(false);
        }
    }
    Ok(true)
}

/// `BRepLib::SameRange(E, Tolerance)`: reparametrizes the p-curves of the edge
/// (GeomLib::SameRange at `tolerance`) to the range of its 3d curve, or of its first
/// curve on surface, then sets that range and the SameRange flag, ignoring the flag
/// value; `brep_lib::same_range(&edge, 1e-5)?`.
pub fn same_range(edge: &Edge, tolerance: f64) -> Result<(), Failure> {
    let c3d = BRepTool::curve_with_location(edge)?;
    let mut current: Option<(f64, f64)> = c3d.curve.is_some().then_some((c3d.first, c3d.last));
    // The native output handles live across the loop: a SameRange that leaves its
    // output unassigned passes on the previous curve.
    let mut new_curve: Handle<dyn Geom2dCurve> = Handle::default();
    let mut new_curve2: Handle<dyn Geom2dCurve> = Handle::default();
    for gcurve in gcurves(edge, "BRepLib::SameRange")? {
        let g: &dyn BRepGCurve = gcurve
            .get()
            .expect("parametric representations are non-null");
        let (has_curve, has_closed_curve) =
            (g.is_curve_on_surface(), g.is_curve_on_closed_surface());
        if !has_curve && !has_closed_curve {
            continue;
        }
        let (first, last) = g.range();
        let (current_first, current_last) = *current.get_or_insert((first, last));
        let confusion: f64 = precision::confusion();
        if !((first - current_first).abs() > confusion || (last - current_last).abs() > confusion) {
            continue;
        }
        let requested = (current_first, current_last);
        if has_curve {
            geom_lib::same_range(
                tolerance,
                &g.pcurve()?,
                (first, last),
                requested,
                &mut new_curve,
            )?;
            g.set_pcurve(&new_curve)?;
        }
        if has_closed_curve {
            geom_lib::same_range(
                tolerance,
                &g.pcurve2()?,
                (first, last),
                requested,
                &mut new_curve2,
            )?;
            g.set_pcurve2(&new_curve2)?;
        }
    }
    let (current_first, current_last) = current.unwrap_or((c3d.first, c3d.last));
    let builder = BRepBuilder::default();
    builder.range(edge, current_first, current_last, false)?;
    builder.same_range(edge, true)
}
