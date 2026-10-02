//! Cleanup pass over free edges and triangles before auxiliary-node removal.
use super::triangle_build::loop_edges;
use super::{BRepMeshDelaun, DegreeOfFreedom};
use rustycad_tkernel::{ncollection::Map, standard::Failure};

impl BRepMeshDelaun {
    /// Remove free triangles unconnected to frontier constraints; `delaun.cleanup_mesh()?`.
    #[doc(hidden)]
    pub fn cleanup_mesh(&mut self) -> Result<(), Failure> {
        while self.cleanup_free_triangle_pass()? > 0 {}
        Ok(())
    }

    fn cleanup_free_triangle_pass(&mut self) -> Result<usize, Failure> {
        let mut dangling_links = loop_edges();
        let mut triangles = Map::default();
        self.collect_free_triangle_removals(&mut dangling_links, &mut triangles)?;
        let deleted = self.delete_free_triangles(&triangles, &mut dangling_links)?;
        self.remove_dangling_links(&dangling_links)?;
        Ok(deleted)
    }

    fn collect_free_triangle_removals(
        &self,
        dangling_links: &mut super::triangle_build::LoopEdges,
        triangles: &mut Map<i32>,
    ) -> Result<(), Failure> {
        for index in self.free_edges()?.iter() {
            self.collect_free_edge_removal(*index, dangling_links, triangles)?;
        }
        Ok(())
    }

    fn collect_free_edge_removal(
        &self,
        link: i32,
        dangling_links: &mut super::triangle_build::LoopEdges,
        triangles: &mut Map<i32>,
    ) -> Result<(), Failure> {
        let edge = self.get_edge(link)?;
        if edge.movability() == DegreeOfFreedom::Frontier {
            return Ok(());
        }
        let adjacent = self.mesh_ref()?.elements_connected_to(link)?;
        if adjacent.is_empty() {
            dangling_links.bind(link, true);
            return Ok(());
        }
        let triangle = adjacent.first_index();
        if self.triangle_is_protected(triangle, link)? {
            return Ok(());
        }
        self.mark_if_unbounded(triangle, link, &edge, triangles)
    }

    fn triangle_is_protected(&self, triangle_index: i32, link: i32) -> Result<bool, Failure> {
        let edges = self.get_triangle(triangle_index)?.edges;
        let Some(current) = edges.iter().position(|edge| *edge == link) else {
            return Ok(true);
        };
        for offset in 1..=2 {
            if self
                .adjacent_edge_protects_triangle(edges[(current + offset) % 3], triangle_index)?
            {
                return Ok(false);
            }
        }
        Ok(true)
    }

    fn adjacent_edge_protects_triangle(&self, link: i32, triangle: i32) -> Result<bool, Failure> {
        let adjacent = self.mesh_ref()?.elements_connected_to(link)?;
        if adjacent.extent() < 2 {
            return Ok(true);
        }
        for position in 1..=adjacent.extent() {
            let neighbor = adjacent.index(position)?;
            if neighbor == triangle {
                continue;
            }
            let nodes = self
                .mesh_ref()?
                .element_nodes(&self.get_triangle(neighbor)?)?;
            if nodes.iter().any(|node| self.is_auxiliary_vertex(*node)) {
                return Ok(true);
            }
        }
        Ok(false)
    }

    fn is_auxiliary_vertex(&self, node: i32) -> bool {
        self.auxiliary.iter().any(|candidate| *candidate == node)
    }

    fn mark_if_unbounded(
        &self,
        triangle: i32,
        link: i32,
        edge: &super::BRepMeshEdge,
        triangles: &mut Map<i32>,
    ) -> Result<(), Failure> {
        let first = self.is_bound_to_frontier(edge.first_node(), link)?;
        let last = self.is_bound_to_frontier(edge.last_node(), link)?;
        if !first || !last {
            triangles.add(triangle);
        }
        Ok(())
    }

    fn delete_free_triangles(
        &mut self,
        triangles: &Map<i32>,
        dangling_links: &mut super::triangle_build::LoopEdges,
    ) -> Result<usize, Failure> {
        let mut deleted = 0;
        for index in triangles.iter() {
            self.delete_triangle(*index, dangling_links)?;
            deleted += 1;
        }
        Ok(deleted)
    }

    fn remove_dangling_links(
        &self,
        dangling_links: &super::triangle_build::LoopEdges,
    ) -> Result<(), Failure> {
        for (link, _) in dangling_links.items() {
            if self.mesh_ref()?.elements_connected_to(*link)?.is_empty() {
                self.mesh_ref()?.remove_link(*link, false)?;
            }
        }
        Ok(())
    }
}
