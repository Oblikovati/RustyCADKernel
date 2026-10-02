//! Reach constrained edges through triangles around one reference node.
use super::BRepMeshDelaun;
use crate::brep_mesh::PairOfIndex;
use rustycad_tkernel::standard::Failure;
use std::collections::HashSet;

impl BRepMeshDelaun {
    /// Follow incident free links until a constraint is found; `delaun.is_bound_to_frontier(1,1)?`.
    #[doc(hidden)]
    pub fn is_bound_to_frontier(&self, node: i32, link: i32) -> Result<bool, Failure> {
        let mut stack = vec![link];
        let mut visited = HashSet::new();
        while let Some(current) = stack.pop() {
            let adjacent = self.mesh_ref()?.elements_connected_to(current)?;
            if adjacent.is_empty() {
                return Ok(false);
            }
            if self.follow_incident_links(node, current, adjacent, &mut visited, &mut stack)? {
                return Ok(true);
            }
        }
        Ok(false)
    }

    fn follow_incident_links(
        &self,
        node: i32,
        current: i32,
        adjacent: PairOfIndex,
        visited: &mut HashSet<i32>,
        stack: &mut Vec<i32>,
    ) -> Result<bool, Failure> {
        for position in 1..=adjacent.extent() {
            let triangle_index = adjacent.index(position)?;
            if triangle_index < 0 {
                continue;
            }
            if self.follow_triangle_links(node, current, triangle_index, visited, stack)? {
                return Ok(true);
            }
        }
        Ok(false)
    }

    fn follow_triangle_links(
        &self,
        node: i32,
        current: i32,
        triangle_index: i32,
        visited: &mut HashSet<i32>,
        stack: &mut Vec<i32>,
    ) -> Result<bool, Failure> {
        for link in self.get_triangle(triangle_index)?.edges {
            if link == current || !self.link_touches_node(link, node)? {
                continue;
            }
            if self.get_edge(link)?.movability() != super::DegreeOfFreedom::Free {
                return Ok(true);
            }
            if visited.insert(link) {
                stack.push(link);
            }
        }
        Ok(false)
    }

    fn link_touches_node(&self, link: i32, node: i32) -> Result<bool, Failure> {
        let edge = self.get_edge(link)?;
        Ok(edge.first_node() == node || edge.last_node() == node)
    }
}
