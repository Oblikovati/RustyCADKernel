// Rust translation of the NewCurve bodies that ShapeCustom_DirectModification.cxx and
// ShapeCustom_ConvertToRevolution.cxx share at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The `NewCurve` of the ShapeCustom modifications that change surfaces only: an edge
//! with a p-curve on a surface that changes is copied (docs/shhealing-design.md
//! section 7).
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::brep::access::edge_tshape;
use rustycad_tkbrep::brep::update_curves::as_gcurve;
use rustycad_tkbrep::brep_tools::NewCurve;
use rustycad_tkbrep::topods::Edge;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::geom::{GeomCurve, GeomSurface};
use rustycad_tkmath::toploc::Location;

/// `NewCurve(E, C, L, Tol)` of both modifications (":p5 abv 26 Feb 99: force copying of
/// edge if any its pcurve will be replaced"): at the first curve on surface whose
/// surface and location satisfy `changes`, the 3D curve of the edge (a copy, null
/// without one), its location and the edge tolerance, and true; false (the outputs left
/// unchanged) when no p-curve lies on a changing surface.
pub(super) fn copy_edge_curve(
    edge: &Edge,
    result: &mut NewCurve,
    member: &str,
    changes: impl Fn(&Handle<dyn GeomSurface>, &Location) -> Result<bool, Failure>,
) -> Result<bool, Failure> {
    for representation in edge_tshape(edge, member)?.curves() {
        let gcurve = as_gcurve(&representation);
        let Some(gc) = gcurve.get() else { continue };
        if !gc.is_curve_on_surface() {
            continue;
        }
        if !changes(&gc.surface()?, &gc.location())? {
            continue;
        }
        let found = BRepTool::curve_with_location(edge)?;
        result.curve = copied_curve(&found.curve)?;
        result.location = found.location;
        result.tolerance = BRepTool::edge_tolerance(edge)?;
        return Ok(true);
    }
    Ok(false)
}

/// `occ::down_cast<Geom_Curve>(C->Copy())`, a null curve staying null.
fn copied_curve(curve: &Handle<dyn GeomCurve>) -> Result<Handle<dyn GeomCurve>, Failure> {
    match curve.get() {
        Some(stored) => Ok(stored.copy()?.cast::<dyn GeomCurve>()),
        None => Ok(Handle::default()),
    }
}
