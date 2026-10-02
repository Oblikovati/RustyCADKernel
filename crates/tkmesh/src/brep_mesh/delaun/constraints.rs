//! Internal-edge constraint insertion and orchestration.
use super::BRepMeshDelaun;
use rustycad_tkernel::standard::Failure;

impl BRepMeshDelaun {
    /// Process internal fixed edges; `delaun.insert_internal_edges_probe()?`.
    #[doc(hidden)]
    pub fn insert_internal_edges_probe(&mut self) -> Result<(), Failure> {
        self.insert_internal_edges()
    }

    fn insert_internal_edges(&mut self) -> Result<(), Failure> {
        let edges = self.internal_edges()?;
        for edge_id in edges.iter().copied() {
            let mut sides = [true, true];
            let triangles = self.mesh_ref()?.elements_connected_to(edge_id)?;
            for index in 1..=triangles.extent() {
                let triangle_id = triangles.index(index)?;
                if triangle_id < 0 {
                    continue;
                }
                self.mark_internal_edge_side(edge_id, triangle_id, &mut sides)?;
            }
            self.mesh_missing_internal_sides(edge_id, sides)?;
        }
        Ok(())
    }

    fn mark_internal_edge_side(
        &self,
        edge_id: i32,
        triangle_id: i32,
        sides: &mut [bool; 2],
    ) -> Result<(), Failure> {
        let triangle = self.get_triangle(triangle_id)?;
        for slot in 0..3 {
            if triangle.edges[slot] == edge_id {
                sides[usize::from(!triangle.orientations[slot])] = false;
                break;
            }
        }
        Ok(())
    }

    fn mesh_missing_internal_sides(
        &mut self,
        edge_id: i32,
        sides: [bool; 2],
    ) -> Result<(), Failure> {
        if sides[0] {
            self.mesh_left_polygon(edge_id, true, None)?;
        }
        if sides[1] {
            self.mesh_left_polygon(edge_id, false, None)?;
        }
        Ok(())
    }
}
