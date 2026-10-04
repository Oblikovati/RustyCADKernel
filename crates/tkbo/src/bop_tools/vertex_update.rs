use super::d_tolerance;
use crate::int_tools::Curve;
use rustycad_tkbrep::{
    brep::{BRepBuilder, BRepTool},
    topods::{Edge, Vertex},
};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::{adaptor3d::Adaptor3dCurve, geom::GeomCurve, geom_adaptor::GeomAdaptorCurve};
/// Enlarge the first vertex to contain the second vertex sphere; `update_vertex_pair(&first,&second)?`.
pub fn update_vertex_pair(first: &Vertex, second: &Vertex) -> Result<(), Failure> {
    let distance = BRepTool::pnt(first)?.distance(&BRepTool::pnt(second)?);
    enlarge_vertex(first, distance + BRepTool::vertex_tolerance(second)?)
}
/// Enlarge a vertex to contain an edge evaluation; `update_vertex_edge(&edge,parameter,&vertex)?`.
pub fn update_vertex_edge(edge: &Edge, parameter: f64, vertex: &Vertex) -> Result<(), Failure> {
    let (curve, _, _) = BRepTool::curve(edge)?;
    update_vertex_curve_handle(&curve, parameter, vertex)
}
/// Enlarge a vertex to contain an intersection-curve evaluation; `update_vertex_curve(&curve,parameter,&vertex)?`.
pub fn update_vertex_curve(curve: &Curve, parameter: f64, vertex: &Vertex) -> Result<(), Failure> {
    update_vertex_curve_handle(curve.curve(), parameter, vertex)
}
fn update_vertex_curve_handle(
    curve: &Handle<dyn GeomCurve>,
    parameter: f64,
    vertex: &Vertex,
) -> Result<(), Failure> {
    let adaptor = GeomAdaptorCurve::with_curve(curve)?;
    let distance = BRepTool::pnt(vertex)?.distance(&adaptor.d0(parameter)?);
    enlarge_vertex(vertex, distance)
}
fn enlarge_vertex(vertex: &Vertex, needed: f64) -> Result<(), Failure> {
    if needed > BRepTool::vertex_tolerance(vertex)? {
        BRepBuilder::default().update_vertex_tolerance(vertex, needed + d_tolerance())?;
    }
    Ok(())
}
