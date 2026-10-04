use super::{adjust_pcurve_on_face, curve_on_surface, has_curve_on_surface};
use crate::int_tools::{Context, Tools};
use rustycad_tkbrep::{
    brep::{BRepBuilder, BRepTool},
    topods::{Edge, Face},
};
use rustycad_tkernel::{
    precision,
    standard::{Failure, Handle},
};
use rustycad_tkg2d::geom2d::Geom2dCurve;
use rustycad_tkg3d::geom::GeomCurve;
use rustycad_tkmath::gp::{self, Point2, Vector2};
/// Persist a projected pcurve only when native presence lookup fails; `build_pcurve_for_edge_on_face(&edge,&face,None)?`.
pub fn build_pcurve_for_edge_on_face(
    edge: &Edge,
    face: &Face,
    context: Option<&Context>,
) -> Result<(), Failure> {
    let mut curve = Handle::default();
    let mut range = (0., 0.);
    let mut tolerance = 0.;
    if has_curve_on_surface(edge, face, &mut curve, &mut range, &mut tolerance)? {
        return Ok(());
    }
    curve_on_surface_simple(edge, face, &mut curve, &mut tolerance, context)?;
    let edge_tolerance = BRepTool::edge_tolerance(edge)?;
    let tolerance = if edge_tolerance < tolerance {
        tolerance
    } else {
        edge_tolerance
    };
    BRepBuilder::default().update_edge_pcurve_on_face(edge, &curve, face, tolerance)
}
/// Return or project a pcurve while discarding its range outputs; `curve_on_surface_simple(&edge,&face,&mut curve,&mut tolerance,None)?`.
pub fn curve_on_surface_simple(
    edge: &Edge,
    face: &Face,
    curve: &mut Handle<dyn Geom2dCurve>,
    tolerance: &mut f64,
    context: Option<&Context>,
) -> Result<(), Failure> {
    curve_on_surface(edge, face, curve, &mut (0., 0.), tolerance, context)
}
/// Evaluate the edge's existing or projected pcurve; `point_on_surface(&edge,&face,0.5,&mut uv,None)?`.
pub fn point_on_surface(
    edge: &Edge,
    face: &Face,
    parameter: f64,
    point: &mut Point2,
    context: Option<&Context>,
) -> Result<(), Failure> {
    let mut curve = Handle::default();
    let mut tolerance = 0.;
    curve_on_surface_simple(edge, face, &mut curve, &mut tolerance, context)?;
    *point = curve.value(parameter)?;
    Ok(())
}
/// Adjust periodic placement over the spatial curve's complete range; `adjust_pcurve_for_curve(&face,&spatial,&pcurve,None)?`.
pub fn adjust_pcurve_for_curve(
    face: &Face,
    spatial: &Handle<dyn GeomCurve>,
    curve: &Handle<dyn Geom2dCurve>,
    context: Option<&Context>,
) -> Result<Handle<dyn Geom2dCurve>, Failure> {
    adjust_pcurve_on_face(
        face,
        spatial.first_parameter(),
        spatial.last_parameter(),
        curve,
        context,
    )
}
/// Use OCCT's asymmetric division constant; `intermediate_point(0.,1.)`.
pub fn intermediate_point(first: f64, last: f64) -> f64 {
    Tools::intermediate_point(first, last)
}
/// Set native cross-product magnitude isoline flags from the midpoint derivative; `is_edge_isoline(&edge,&face,&mut u_iso,&mut v_iso)?`.
pub fn is_edge_isoline(
    edge: &Edge,
    face: &Face,
    u_iso: &mut bool,
    v_iso: &mut bool,
) -> Result<(), Failure> {
    *u_iso = false;
    *v_iso = false;
    let curve = BRepTool::curve_on_face(edge, face)?;
    let (_, mut tangent) = curve.pcurve.d1(0.5 * (curve.first + curve.last))?;
    let squared = tangent.square_magnitude();
    if squared <= gp::resolution() {
        return Ok(());
    }
    tangent /= squared.sqrt();
    *u_iso = tangent.cross_magnitude(Vector2::new(0., 1.)) <= precision::angular();
    *v_iso = tangent.cross_magnitude(Vector2::new(1., 0.)) <= precision::angular();
    Ok(())
}
