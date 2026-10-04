use crate::int_tools::{Context, ShrunkRange};
use rustycad_tkbrep::{brep::BRepTool, topexp, topods::Edge};
use rustycad_tkernel::standard::{Failure, Handle};
/// Classify an edge using its endpoint tolerance spheres and optional splitting check; `is_micro_edge(&edge,&context,true)?`.
pub fn is_micro_edge(
    edge: &Edge,
    context: &Handle<Context>,
    check_splittable: bool,
) -> Result<bool, Failure> {
    if BRepTool::degenerated(edge)? || !BRepTool::is_geometric_edge(edge)? {
        return Ok(true);
    }
    let _ = BRepTool::curve(edge)?;
    let (first_vertex, last_vertex) = topexp::vertices(edge, false);
    let mut first = BRepTool::parameter(&first_vertex, edge)?;
    let mut last = BRepTool::parameter(&last_vertex, edge)?;
    if last < first {
        std::mem::swap(&mut first, &mut last);
    }
    let mut shrunk = ShrunkRange::new();
    shrunk.set_context(context);
    shrunk.set_data(edge, first, last, &first_vertex, &last_vertex);
    shrunk.perform()?;
    Ok(!shrunk.is_done() || (check_splittable && !shrunk.is_splittable()))
}
