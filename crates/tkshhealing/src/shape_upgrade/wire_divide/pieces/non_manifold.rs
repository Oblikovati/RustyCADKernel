//! Context replacements occur even before deciding whether a non-manifold point belongs to a piece.
use super::*;
pub(super) fn attach(
    support: &WireSupport,
    source: &Edge,
    target: &mut Edge,
    cursor: &mut PieceCursor,
    last: &Vertex,
    bounds: (f64, f64),
) -> Result<(), Failure> {
    let mut index = 0;
    while index < cursor.vertices.non_manifold.len() {
        let (original, parameter) = &cursor.vertices.non_manifold[index];
        let copy = replace_vertex(support, original, target, source)?;
        if !place_vertex(
            support,
            target,
            &copy,
            *parameter,
            &cursor.vertices.first,
            last,
            bounds,
        )? {
            index += 1;
            continue;
        }
        cursor.vertices.non_manifold.remove(index);
    }
    Ok(())
}
fn place_vertex(
    support: &WireSupport,
    edge: &mut Edge,
    vertex: &Vertex,
    parameter: f64,
    first: &Vertex,
    last: &Vertex,
    bounds: (f64, f64),
) -> Result<bool, Failure> {
    if (parameter - bounds.0).abs() <= precision::p_confusion() {
        support.context.replace(vertex, first)?;
    } else if (parameter - bounds.1).abs() <= precision::p_confusion() {
        support.context.replace(vertex, last)?;
    } else if parameter > bounds.0 && parameter < bounds.1 {
        BRepBuilder::default().add(edge, vertex)?;
    } else {
        return Ok(false);
    }
    Ok(true)
}

fn replace_vertex(
    support: &WireSupport,
    original: &Vertex,
    target: &Edge,
    source: &Edge,
) -> Result<Vertex, Failure> {
    let copy = ShapeAnalysisTransferParametersProj::copy_nm_vertex_edge(original, target, source)?;
    support.context.replace(original, &copy)?;
    Ok(copy)
}
