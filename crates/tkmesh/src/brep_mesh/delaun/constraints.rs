//! Internal-edge constraint insertion and orchestration.
use super::triangle_build::{LoopEdges, loop_edges};
use super::{BRepMeshDelaun, BRepMeshTriangle};
use rustycad_tkernel::{ncollection::Map, standard::Failure};

impl BRepMeshDelaun {
    /// Process internal fixed edges; `delaun.insert_internal_edges_probe()?`.
    #[doc(hidden)]
    pub fn insert_internal_edges_probe(&mut self) -> Result<(), Failure> {
        self.insert_internal_edges()
    }

    /// Recover fixed links and adjust mesh boundaries; `delaun.process_constraints()?`.
    pub fn process_constraints(&mut self) -> Result<(), Failure> {
        self.insert_internal_edges()?;
        self.frontier_adjust()
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

    fn frontier_adjust(&mut self) -> Result<(), Failure> {
        let frontier = self.frontier()?;
        let mut failed = Vec::new();
        let mut deleted_edges = loop_edges();
        let mut skipped = Map::default();
        for pass in 1..=2 {
            self.remove_external_frontier_triangles(&frontier, &mut deleted_edges)?;
            self.remove_dangling_frontier_edges(&deleted_edges)?;
            self.recover_missing_frontier_loops(&frontier, pass, &mut failed, &mut skipped)?;
        }
        self.cleanup_mesh()?;
        self.retry_failed_frontier_loops(&failed, &mut skipped)
    }

    fn remove_external_frontier_triangles(
        &mut self,
        frontier: &Map<i32>,
        deleted_edges: &mut LoopEdges,
    ) -> Result<(), Failure> {
        for edge in frontier.iter().copied() {
            self.delete_external_frontier_triangle(edge, deleted_edges)?;
        }
        Ok(())
    }

    fn delete_external_frontier_triangle(
        &mut self,
        edge_id: i32,
        deleted_edges: &mut LoopEdges,
    ) -> Result<(), Failure> {
        let adjacent = self.mesh_ref()?.elements_connected_to(edge_id)?;
        for index in 1..=adjacent.extent() {
            let triangle_id = adjacent.index(index)?;
            if triangle_id < 0 {
                continue;
            }
            if self.triangle_has_reversed_edge(triangle_id, edge_id)? {
                self.delete_triangle(triangle_id, deleted_edges)?;
                break;
            }
        }
        Ok(())
    }

    fn triangle_has_reversed_edge(&self, triangle_id: i32, edge_id: i32) -> Result<bool, Failure> {
        let triangle = self.get_triangle(triangle_id)?;
        Ok(triangle_edge_slot(&triangle, edge_id).is_some_and(|slot| !triangle.orientations[slot]))
    }

    fn remove_dangling_frontier_edges(&self, edges: &LoopEdges) -> Result<(), Failure> {
        let dangling = edges.items().map(|(edge, _)| *edge).collect::<Vec<_>>();
        for edge in dangling {
            if self.mesh_ref()?.elements_connected_to(edge)?.is_empty() {
                self.mesh_ref()?.remove_link(edge, false)?;
            }
        }
        Ok(())
    }

    fn recover_missing_frontier_loops(
        &mut self,
        frontier: &Map<i32>,
        pass: i32,
        failed: &mut Vec<i32>,
        skipped: &mut Map<i32>,
    ) -> Result<(), Failure> {
        for edge in frontier.iter().copied() {
            if !self.mesh_ref()?.elements_connected_to(edge)?.is_empty() {
                continue;
            }
            let recovered = self.mesh_left_polygon(edge, true, Some(skipped))?;
            if pass == 2 && !recovered {
                failed.push(edge);
            }
        }
        Ok(())
    }

    fn retry_failed_frontier_loops(
        &mut self,
        failed: &[i32],
        skipped: &mut Map<i32>,
    ) -> Result<(), Failure> {
        for edge in failed.iter().copied() {
            if self.mesh_ref()?.elements_connected_to(edge)?.is_empty() {
                self.mesh_left_polygon(edge, true, Some(skipped))?;
            }
        }
        Ok(())
    }
}

fn triangle_edge_slot(triangle: &BRepMeshTriangle, edge_id: i32) -> Option<usize> {
    triangle.edges.iter().position(|edge| *edge == edge_id)
}
