// Rust translation of the erase and cleanup members of BRepMesh_MeshTool.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.
use super::{BRepMeshMeshTool, MapOfIntegerInteger, new_loop_edges};
use crate::brep_mesh::{BRepMeshEdge, DegreeOfFreedom};
use rustycad_tkernel::ncollection::PackedMap;
use rustycad_tkernel::standard::Failure;

impl BRepMeshMeshTool {
    /// `CleanFrontierLinks()`: erases the triangles lying to the right of each frontier
    /// link (with the free-link triangles around its nodes), then the freed links;
    /// `tool.clean_frontier_links()?`.
    pub fn clean_frontier_links(&self) -> Result<(), Failure> {
        let mut to_erase = PackedMap::<i32>::default();
        let mut loop_edges = new_loop_edges();
        for frontier_id in self.get_edges_by_type(DegreeOfFreedom::Frontier)?.iter() {
            self.collect_outer_triangle(frontier_id, &mut to_erase)?;
        }
        self.erase_triangles(&to_erase, &mut loop_edges)?;
        self.erase_free_links_of(&loop_edges)
    }

    /// The first triangle that uses the frontier link reversed, and the free-link
    /// triangles around the link nodes from its two other links.
    fn collect_outer_triangle(
        &self,
        frontier_id: i32,
        to_erase: &mut PackedMap<i32>,
    ) -> Result<(), Failure> {
        let link = self.structure.get_link(frontier_id)?;
        let pair = self.structure.elements_connected_to(frontier_id)?;
        for position in 1..=pair.extent() {
            let element_id = pair.index(position)?;
            let element = self.structure.get_element(element_id)?;
            let (edges, orientations) = (element.edges, element.orientations);
            let Some(n) = (0..3).find(|n| edges[*n] == frontier_id && !orientations[*n]) else {
                continue;
            };
            to_erase.add(element_id);
            self.collect_triangles_on_free_links_around_nodes_of(
                &link,
                edges[(n + 1) % 3],
                to_erase,
            )?;
            self.collect_triangles_on_free_links_around_nodes_of(
                &link,
                edges[(n + 2) % 3],
                to_erase,
            )?;
            return Ok(());
        }
        Ok(())
    }

    /// `EraseTriangles(theTriangles, theLoopEdges)`: each triangle in the packed-map
    /// order; `tool.erase_triangles(&triangles, &mut loop_edges)?`.
    pub fn erase_triangles(
        &self,
        triangles: &PackedMap<i32>,
        loop_edges: &mut MapOfIntegerInteger,
    ) -> Result<(), Failure> {
        for triangle in triangles.iter() {
            self.erase_triangle(triangle, loop_edges)?;
        }
        Ok(())
    }

    /// `EraseTriangle(theTriangleIndex, theLoopEdges)`: removes the triangle and binds
    /// its links with their orientations; a link bound a second time is unbound and
    /// removed; `tool.erase_triangle(1, &mut loop_edges)?`.
    pub fn erase_triangle(
        &self,
        triangle_index: i32,
        loop_edges: &mut MapOfIntegerInteger,
    ) -> Result<(), Failure> {
        let element = self.structure.get_element(triangle_index)?;
        self.structure.remove_element(triangle_index)?;
        for (edge, orientation) in element.edges.into_iter().zip(element.orientations) {
            if !loop_edges.bind(edge, orientation) {
                loop_edges.unbind(&edge);
                self.structure.remove_link(edge, false)?;
            }
        }
        Ok(())
    }

    /// `EraseFreeLinks()`: every link without element and not deleted is made free
    /// (in place, as the native const_cast does) and removed; `tool.erase_free_links()?`.
    pub fn erase_free_links(&self) -> Result<(), Failure> {
        let mut index = 1;
        while index <= self.structure.nb_links() {
            if self.structure.elements_connected_to(index)?.is_empty()
                && self.structure.get_link(index)?.movability() != DegreeOfFreedom::Deleted
            {
                self.structure
                    .set_link_movability(index, DegreeOfFreedom::Free)?;
                self.structure.remove_link(index, false)?;
            }
            index += 1;
        }
        Ok(())
    }

    /// `EraseFreeLinks(theLinks)`: the links of the map left without element, in the
    /// map's order; `tool.erase_free_links_of(&loop_edges)?`.
    pub fn erase_free_links_of(&self, links: &MapOfIntegerInteger) -> Result<(), Failure> {
        let keys: Vec<i32> = links.items().map(|(key, _)| *key).collect();
        for link in keys {
            if self.structure.elements_connected_to(link)?.is_empty() {
                self.structure.remove_link(link, false)?;
            }
        }
        Ok(())
    }

    /// `collectTrianglesOnFreeLinksAroundNodesOf`: depth first from `start_link`, the
    /// triangles of the free links that touch a node of the constraint.
    fn collect_triangles_on_free_links_around_nodes_of(
        &self,
        constraint: &BRepMeshEdge,
        start_link: i32,
        triangles: &mut PackedMap<i32>,
    ) -> Result<(), Failure> {
        let mut used_links = PackedMap::<i32>::default();
        let mut stack = vec![start_link];
        used_links.add(start_link);
        while let Some(link_index) = stack.pop() {
            let link = self.structure.get_link(link_index)?;
            if link.movability() != DegreeOfFreedom::Free || !touches_constraint(&link, constraint)
            {
                continue;
            }
            let pair = self.structure.elements_connected_to(link_index)?;
            for position in 1..=pair.extent() {
                let element_id = pair.index(position)?;
                triangles.add(element_id);
                for edge in self.structure.get_element(element_id)?.edges {
                    if edge != link_index && used_links.add(edge) {
                        stack.push(edge);
                    }
                }
            }
        }
        Ok(())
    }
}

/// Whether the link shares a node with the constraint.
fn touches_constraint(link: &BRepMeshEdge, constraint: &BRepMeshEdge) -> bool {
    [link.first_node(), link.last_node()]
        .iter()
        .any(|node| *node == constraint.first_node() || *node == constraint.last_node())
}
