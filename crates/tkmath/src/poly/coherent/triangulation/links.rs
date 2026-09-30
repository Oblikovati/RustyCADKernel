// Rust translation of the link members of FoundationClasses/TKMath/Poly/Poly_CoherentTriangulation.cxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

use super::CoherentTriangulation;
use crate::poly::coherent::CoherentLink;
use rustycad_tkernel::standard::Failure;
fn program_error(message: String) -> Failure {
    Failure::with_type("Standard_ProgramError", &message, "")
}
impl CoherentTriangulation {
    /// Detaches a link from the triangles on its sides and empties it; `mesh.remove_link(0)?`.
    pub fn remove_link(&mut self, link: usize) -> Result<(), Failure> {
        let record = *self.link_slot(link)?;
        let (found, sides) = self.find_triangle(&record)?;
        if found {
            for (side, triangle) in sides.into_iter().enumerate() {
                let node = record.opposite_node(side as i32);
                if let Some(triangle) = triangle.filter(|_| node >= 0) {
                    self.forget_link_at(triangle, node)?;
                }
            }
        }
        self.links[link] = CoherentLink::default();
        Ok(())
    }
    fn push_link(&mut self, link: CoherentLink) -> usize {
        let slot = self.link_count;
        if slot < self.links.len() {
            self.links[slot] = link;
        } else {
            self.links.push(link);
        }
        self.link_count += 1;
        slot
    }
    // A slot below the used count, or a stale slot that a triangle still refers to.
    fn link_slot(&self, link: usize) -> Result<&CoherentLink, Failure> {
        self.links
            .get(link)
            .ok_or_else(|| super::out_of_range("RemoveLink", link as i32, self.link_count))
    }
    fn forget_link_at(&mut self, triangle: usize, node: i32) -> Result<(), Failure> {
        let position = self.triangles[triangle]
            .nodes
            .iter()
            .position(|n| *n == node);
        let Some(position) = position else {
            return Err(program_error(format!(
                "Poly_CoherentTriangulation::RemoveLink:  wrong connectivity between triangles (node {node} not in triangle {triangle})"
            )));
        };
        self.triangles[triangle].links[position] = None;
        Ok(())
    }
    /// Appends the link of side `conn` of a triangle and attaches it to the triangle and to
    /// its neighbour on that side; `None` for a removed triangle; `mesh.add_link(0, 1)?`.
    pub fn add_link(&mut self, triangle: usize, conn: i32) -> Result<Option<usize>, Failure> {
        let record = *self.triangle(triangle)?;
        if record.is_empty() {
            return Ok(None);
        }
        let link = self.push_link(CoherentLink::from_triangle(&record, conn)?);
        self.triangles[triangle].links[conn as usize] = Some(link);
        let Some(opposite) = record.get_connected_tri(conn) else {
            return Ok(Some(link));
        };
        if self.triangles[opposite].is_empty() {
            return Ok(Some(link));
        }
        let connected_node = record.get_connected_node(conn);
        let Some(position) = self.triangles[opposite]
            .nodes
            .iter()
            .position(|n| *n == connected_node)
        else {
            return Err(program_error(format!(
                "Poly_CoherentTriangulation::AddLink: Bad connectivity of triangles (node {connected_node} not in triangle {opposite})"
            )));
        };
        self.triangles[opposite].links[position] = Some(link);
        Ok(Some(link))
    }
    /// Triangles on the left (0) and right (1) of a node pair, found through the triangles
    /// of its first node; `mesh.find_triangle(&CoherentLink::from_nodes(0, 1))?`.
    pub fn find_triangle(
        &self,
        link: &CoherentLink,
    ) -> Result<(bool, [Option<usize>; 2]), Failure> {
        let mut found = [None, None];
        let (first, second) = (link.node(0), link.node(1));
        let count = self.nodes.len() as i32;
        if link.is_empty() || first >= count || second >= count {
            return Ok((false, found));
        }
        for triangle in self.nodes[first as usize].triangles() {
            let nodes = self.triangles[triangle].nodes;
            let Some(at) = nodes.iter().position(|n| *n == first) else {
                return Err(program_error(format!(
                    "Poly_CoherentTriangulation::FindTriangle :  Data incoherence detected (triangle {triangle} lacks node {first})"
                )));
            };
            if nodes[(at + 1) % 3] == second {
                found[0] = Some(triangle);
            } else if nodes[(at + 2) % 3] == second {
                found[1] = Some(triangle);
            }
            if found[0].is_some() && found[1].is_some() {
                break;
            }
        }
        Ok((found.iter().any(Option::is_some), found))
    }
    /// Rebuilds every link: first one per shared side, then the missing boundary sides;
    /// returns the link count; `mesh.compute_links()?`.
    pub fn compute_links(&mut self) -> Result<i32, Failure> {
        self.link_count = 0;
        for triangle in 0..self.triangles.len() {
            let record = self.triangles[triangle];
            if record.is_empty() {
                continue;
            }
            for (side, (start, end)) in [(2, (0, 1)), (0, (1, 2)), (1, (2, 0))] {
                if record.node(start) < record.node(end) {
                    self.add_link(triangle, side)?;
                }
            }
        }
        for triangle in 0..self.triangles.len() {
            for side in 0..3 {
                let record = self.triangles[triangle];
                if !record.is_empty() && record.links[side as usize].is_none() {
                    self.add_link(triangle, side)?;
                }
            }
        }
        Ok(self.link_count as i32)
    }
    /// Drops every link and every triangle reference to one; `mesh.clear_links()`.
    pub fn clear_links(&mut self) {
        self.link_count = 0;
        for triangle in &mut self.triangles {
            triangle.links = [None; 3];
        }
    }
}
