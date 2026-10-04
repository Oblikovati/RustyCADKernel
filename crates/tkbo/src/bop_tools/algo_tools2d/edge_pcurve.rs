//! Stored pcurve lookup and projection fallback from BOPTools_AlgoTools2D.cxx.
use super::make_pcurve_on_face;
use crate::int_tools::{Context, Tools};
use rustycad_tkbrep::{
    brep::BRepTool,
    topods::{Edge, Face},
};
use rustycad_tkernel::{
    precision,
    standard::{Failure, Handle},
};
use rustycad_tkg2d::geom2d::Geom2dCurve;
/// Read native presence/range/tolerance outputs; `has_curve_on_surface(&edge, &face, &mut curve, &mut range, &mut tolerance)?`.
pub fn has_curve_on_surface(
    edge: &Edge,
    face: &Face,
    curve: &mut Handle<dyn Geom2dCurve>,
    range: &mut (f64, f64),
    tolerance: &mut f64,
) -> Result<bool, Failure> {
    *tolerance = BRepTool::edge_tolerance(edge)?;
    pcurve_presence(edge, face, curve, range)
}
/// Query presence without computing tolerance; `has_curve_on_surface_simple(&edge,&face)?`.
pub fn has_curve_on_surface_simple(edge: &Edge, face: &Face) -> Result<bool, Failure> {
    pcurve_presence(edge, face, &mut Handle::default(), &mut (0., 0.))
}
fn pcurve_presence(
    edge: &Edge,
    face: &Face,
    curve: &mut Handle<dyn Geom2dCurve>,
    range: &mut (f64, f64),
) -> Result<bool, Failure> {
    *range = BRepTool::range(edge)?;
    if range.1 - range.0 < precision::p_confusion() {
        return Ok(false);
    }
    let found = BRepTool::curve_on_face(edge, face)?;
    *curve = found.pcurve;
    *range = (found.first, found.last);
    Ok(!curve.is_null())
}
/// Return an existing pcurve or project the 3D edge without storing it; `curve_on_surface(&edge,&face,&mut curve,&mut range,&mut tolerance,None)?`.
pub fn curve_on_surface(
    edge: &Edge,
    face: &Face,
    curve: &mut Handle<dyn Geom2dCurve>,
    range: &mut (f64, f64),
    tolerance: &mut f64,
    context: Option<&Context>,
) -> Result<(), Failure> {
    let mut found = Handle::default();
    if has_curve_on_surface(edge, face, &mut found, range, tolerance)? {
        *curve = found;
        return Ok(());
    }
    make_2d(edge, face, &mut found, range, tolerance, context)?;
    *curve = found;
    Ok(())
}
/// Reuse support or project a located spatial curve; `make_2d(&edge,&face,&mut curve,&mut range,&mut tolerance,None)?`.
pub fn make_2d(
    edge: &Edge,
    face: &Face,
    curve: &mut Handle<dyn Geom2dCurve>,
    range: &mut (f64, f64),
    tolerance: &mut f64,
    context: Option<&Context>,
) -> Result<(), Failure> {
    let found = BRepTool::curve_on_face(edge, face)?;
    *range = (found.first, found.last);
    if !found.pcurve.is_null() {
        *curve = found.pcurve;
        return Ok(());
    }
    let (spatial, first, last) = BRepTool::curve(edge)?;
    if spatial.is_null() {
        return Err(Failure::with_message(
            "Standard_NullObject",
            "BOPTools_AlgoTools2D::Make2D curve=null; expected a spatial curve for projection",
        ));
    }
    *tolerance = BRepTool::edge_tolerance(edge)?;
    make_pcurve_on_face(face, &spatial, (first, last), curve, tolerance, context)?;
    *range = (first, last);
    Ok(())
}
/// Read a 3D edge's asymmetric interior parameter; `intermediate_point_on_edge(&edge)?`.
pub fn intermediate_point_on_edge(edge: &Edge) -> Result<f64, Failure> {
    let (curve, mut first, mut last) = BRepTool::curve(edge)?;
    if curve.is_null() {
        (first, last) = BRepTool::range(edge)?;
    }
    Ok(Tools::intermediate_point(first, last))
}
