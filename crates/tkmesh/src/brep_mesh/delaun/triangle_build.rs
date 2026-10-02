//! Native triangle/circumcircle registration is deliberately non-transactional.
use super::{BRepMeshDelaun, BRepMeshEdge, BRepMeshTriangle, DegreeOfFreedom};
use rustycad_tkernel::{
    ncollection::{BaseAllocator, DataMap, DefaultHashPolicy},
    standard::Failure,
};
pub(super) type LoopEdges = DataMap<i32, bool>;
pub(super) fn loop_edges() -> LoopEdges {
    let arena = BaseAllocator::incremental(crate::imesh_data::MEMORY_BLOCK_SIZE_HUGE);
    DataMap::with_allocator(DefaultHashPolicy, 10, arena)
}
impl BRepMeshDelaun {
    pub(super) fn add_triangle(
        &mut self,
        edges: [i32; 3],
        orientations: [bool; 3],
        nodes: [i32; 3],
    ) -> Result<(), Failure> {
        let index = self.mesh_ref()?.add_element(&BRepMeshTriangle::new(
            edges,
            orientations,
            DegreeOfFreedom::Free,
        ))?;
        if !self.initialized_circles {
            return Ok(());
        }
        let points = [
            *self.get_vertex(nodes[0])?.coord(),
            *self.get_vertex(nodes[1])?.coord(),
            *self.get_vertex(nodes[2])?.coord(),
        ];
        if !self.circles.bind_points(index, &points)? {
            self.mesh_ref()?.remove_element(index)?;
        }
        Ok(())
    }
    pub(super) fn add_signed_triangle(
        &mut self,
        edges: [i32; 3],
        nodes: [i32; 3],
    ) -> Result<(), Failure> {
        let indices = edges.map(i32::wrapping_abs);
        let orientations = edges.map(|index| index > 0);
        self.add_triangle(indices, orientations, nodes)
    }
    pub(super) fn oriented_nodes(&self, index: i32) -> Result<[i32; 2], Failure> {
        let edge = self.get_edge(index.wrapping_abs())?;
        let mut nodes = [edge.first_node(), edge.last_node()];
        if index <= 0 {
            nodes.swap(0, 1);
        }
        Ok(nodes)
    }
    pub(super) fn add_free_link(&self, first: i32, last: i32) -> Result<i32, Failure> {
        self.mesh_ref()?
            .add_link(&BRepMeshEdge::new(first, last, DegreeOfFreedom::Free))
    }
}
