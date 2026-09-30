//! Copies own endpoint vertices and pcurves before replacement geometry is installed.
use super::*;
pub(super) fn new_edge(
    source: &Edge,
    cursor: &mut PieceCursor,
    last: &mut Vertex,
    piece: &PieceGeometry,
) -> Result<Edge, Failure> {
    orient_vertices(cursor, last);
    let edge = if cursor.vertices.forward {
        ShapeBuildEdge.copy_replace_vertices(source, &cursor.vertices.first, last)?
    } else {
        ShapeBuildEdge.copy_replace_vertices(source, last, &cursor.vertices.first)?
    };
    ShapeBuildEdge.copy_pcurves(&edge, source)?;
    if !piece.spatial.is_null() {
        BRepBuilder::default().update_edge_curve(&edge, &piece.spatial, 0.)?;
    } else if cursor.vertices.degenerate {
        BRepBuilder::default().degenerated(&edge, true)?;
    }
    Ok(edge)
}
fn orient_vertices(cursor: &mut PieceCursor, last: &mut Vertex) {
    let first_orientation = if cursor.vertices.forward {
        Orientation::Forward
    } else {
        Orientation::Reversed
    };
    cursor.vertices.first.set_orientation(first_orientation);
    last.set_orientation(if cursor.vertices.forward {
        Orientation::Reversed
    } else {
        Orientation::Forward
    });
}
pub(super) fn update_pcurves(
    support: &WireSupport,
    edge: &Edge,
    piece: &PieceGeometry,
    seam: bool,
) -> Result<(), Failure> {
    let builder = BRepBuilder::default();
    if seam {
        let pair = if edge.orientation() == Orientation::Forward {
            (&piece.planar, &piece.reversed)
        } else {
            (&piece.reversed, &piece.planar)
        };
        return builder.update_edge_pcurves_on_face(edge, pair, &support.face, 0.);
    }
    if !support.face.is_null() {
        builder.update_edge_pcurve_on_face(edge, &piece.planar, &support.face, 0.)?;
    }
    Ok(())
}
