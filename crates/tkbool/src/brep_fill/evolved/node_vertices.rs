// Rust translation of BRepFill_Evolved::VertexFromNode at
// OCCT 3d097a0328e71b826377d4814ab05ec3c3d23871.
use super::ShapeOrigins;
use super::statics::{distance_to_oz, pos_on_face};
use rustycad_tkbrep::brep::BRepBuilder;
use rustycad_tkbrep::topods::{self, Edge, Shape, Vertex};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tktopalgo::mat::MatNode;

/// The vertices made at the nodes of the map, by node and spine shape (`MapNodeVertex`).
#[derive(Default)]
pub(super) struct NodeVertices {
    entries: Vec<(Handle<MatNode>, ShapeOrigins)>,
}

impl NodeVertices {
    /// The vertex of (`node`, `shape`), made when absent.
    fn vertex(&mut self, node: &Handle<MatNode>, shape: &Shape) -> Result<Vertex, Failure> {
        let index = match self
            .entries
            .iter()
            .position(|(n, _)| n.as_ptr() == node.as_ptr())
        {
            Some(i) => i,
            None => {
                self.entries.push((node.clone(), ShapeOrigins::default()));
                self.entries.len() - 1
            }
        };
        let map = &mut self.entries[index].1;
        if let Some(v) = map.seek(shape) {
            return topods::vertex(v);
        }
        let mut vn = Vertex::default();
        BRepBuilder::default().make_vertex(&mut vn);
        map.bind(shape.clone(), vn.as_ref().clone());
        Ok(vn)
    }
}

/// Standard_NullObject of a null map item.
pub(super) fn null_item(what: &str) -> Failure {
    Failure::with_type(
        "Standard_NullObject",
        &format!("BRepFill_Evolved: null {what}; expected a map item"),
        "BRepFill_Evolved",
    )
}

/// `VertexFromNode(aNode, E, VF, VL, MapNodeVertex, VN)` (static): the position of a
/// finite node on the profile edge (1 at `vf`, 2 inside, 3 at `vl`, 0 out) and its
/// vertex on that shape (made once per node and shape).
pub(super) fn vertex_from_node(
    a_node: &Handle<MatNode>,
    e: &Edge,
    vf: &Vertex,
    vl: &Vertex,
    map: &mut NodeVertices,
) -> Result<(i32, Option<Vertex>), Failure> {
    let node = a_node.get().ok_or_else(|| null_item("node"))?;
    let status = if node.infinite() {
        0
    } else {
        pos_on_face(node.distance(), distance_to_oz(vf)?, distance_to_oz(vl)?)
    };
    let shape_on_node = match status {
        2 => e.as_ref().clone(),
        1 => vf.as_ref().clone(),
        3 => vl.as_ref().clone(),
        _ => return Ok((status, None)),
    };
    Ok((status, Some(map.vertex(a_node, &shape_on_node)?)))
}
