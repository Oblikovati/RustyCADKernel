// Rust translation of BRepGraph::TopoView::EdgeOps::WiresOf and FacesOf
// (ModelingData/TKBRep/BRepGraph/BRepGraph_TopoView.cxx)
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The edge members of the topology view that return reverse iterators.
use super::EdgeOps;
use crate::brep_graph::EdgeId;
use crate::brep_graph::reverse_iterator::{FacesOfEdge, WiresOfEdge};

impl<'g> EdgeOps<'g> {
    /// `WiresOf(theEdge)`: the active wires of the edge's active coedges.
    ///
    /// ```
    /// use rustycad_tkbrep::brep_graph::{BRepGraph, EdgeId};
    /// let graph = BRepGraph::new();
    /// assert!(!graph.topo().edges().wires_of(EdgeId::new(0)).more());
    /// ```
    pub fn wires_of(self, edge: EdgeId) -> WiresOfEdge<'g> {
        WiresOfEdge::new(self.graph, edge)
    }

    /// `WiresOf(theEdge, theStartIndex)`: resumed at a coedge relation index.
    pub fn wires_of_from(self, edge: EdgeId, start_index: u32) -> WiresOfEdge<'g> {
        WiresOfEdge::starting_at(self.graph, edge, start_index)
    }

    /// `FacesOf(theEdge)`: the active faces adjacent to the edge through active coedges.
    pub fn faces_of(self, edge: EdgeId) -> FacesOfEdge<'g> {
        FacesOfEdge::new(self.graph, edge)
    }

    /// `FacesOf(theEdge, theStartIndex)`: resumed at a coedge relation index.
    pub fn faces_of_from(self, edge: EdgeId, start_index: u32) -> FacesOfEdge<'g> {
        FacesOfEdge::starting_at(self.graph, edge, start_index)
    }
}

#[cfg(test)]
mod tests {
    use crate::brep_graph::{BRepGraph, FaceId};
    use crate::brep_graph_inc::ParityOrientation;

    #[test]
    fn faces_of_an_edge_are_reported_once() {
        let mut graph = BRepGraph::new();
        let storage = graph.inc_storage_mut();
        let (edge, wire, face) = (
            storage.append_edge(),
            storage.append_wire(),
            storage.append_face(),
        );
        for orientation in [ParityOrientation::FORWARD, ParityOrientation::REVERSED] {
            storage.create_co_edge_use(wire, edge, face, orientation);
        }
        let faces: Vec<FaceId> = graph.topo().edges().faces_of(edge).collect();
        assert_eq!(faces, vec![face]);
        assert_eq!(graph.topo().edges().wires_of(edge).count(), 1);
        assert!(!graph.topo().edges().faces_of_from(edge, 1).more());
        assert_eq!(graph.topo().edges().wires_of_from(edge, 2).count(), 0);
    }
}
