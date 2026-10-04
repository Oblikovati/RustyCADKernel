// Rust translation of the editing members of FoundationClasses/TKMath/Poly/Poly_CoherentTriangulation.cxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

use super::{CoherentTriangulation, TwoIntegers};
use crate::poly::coherent::CoherentTriangle;
use rustycad_tkernel::{ncollection::List, standard::Failure};
impl CoherentTriangulation {
    /// Detaches a triangle from its nodes, neighbours and links and marks it removed;
    /// true when a node dropped it; `mesh.remove_triangle(0)?`.
    pub fn remove_triangle(&mut self, triangle: usize) -> Result<bool, Failure> {
        self.triangle(triangle)?;
        let allocator = self.allocator.clone();
        let mut removed = false;
        for side in 0..3 {
            let node = self.triangles[triangle].nodes[side];
            if node >= 0 {
                self.release_side_link(triangle, side)?;
                if self
                    .change_node(node)?
                    .remove_triangle_in(triangle, &allocator)?
                {
                    self.triangles[triangle].nodes[side] = -1;
                    removed = true;
                }
            }
            CoherentTriangle::remove_connection(&mut self.triangles, triangle, side as i32)?;
        }
        Ok(removed)
    }
    // A link shared with a neighbour loses this triangle's opposite node; otherwise it goes.
    fn release_side_link(&mut self, triangle: usize, side: usize) -> Result<(), Failure> {
        let Some(link) = self.triangles[triangle].links[side] else {
            return Ok(());
        };
        let own_node = self.triangles[triangle].nodes[side];
        let mut remove = true;
        if let Some(opposite) = self.triangles[triangle].connected[side]
            && self.triangles[opposite].links.contains(&Some(link))
        {
            let record = &mut self.links[link];
            if record.opposite_node(0) == own_node {
                record.opposite_nodes[0] = -1;
                remove = false;
            } else if record.opposite_node(1) == own_node {
                record.opposite_nodes[1] = -1;
                remove = false;
            }
        }
        if remove {
            self.remove_link(link)?;
        }
        Ok(())
    }
    /// Appends a triangle when all nodes are non-negative and returns its index;
    /// `mesh.add_triangle(0, 1, 2)?`.
    pub fn add_triangle(
        &mut self,
        node0: i32,
        node1: i32,
        node2: i32,
    ) -> Result<Option<usize>, Failure> {
        if node0 < 0 || node1 < 0 || node2 < 0 {
            return Ok(None);
        }
        self.triangles.push(CoherentTriangle::default());
        let index = self.triangles.len() - 1;
        self.replace_nodes(index, node0, node1, node2)?;
        Ok(Some(index))
    }
    /// Removes a triangle and re-inserts it on new nodes, connecting it to the triangles
    /// of those nodes and updating links when they exist; `mesh.replace_nodes(0, 0, 2, 3)?`.
    pub fn replace_nodes(
        &mut self,
        triangle: usize,
        node0: i32,
        node1: i32,
        node2: i32,
    ) -> Result<bool, Failure> {
        if !self.triangle(triangle)?.is_empty() {
            self.remove_triangle(triangle)?;
        }
        if node0 < 0 || node1 < 0 || node2 < 0 {
            return Ok(false);
        }
        self.triangles[triangle] = CoherentTriangle::new(node0, node1, node2);
        let allocator = self.allocator.clone();
        for node in [node0, node1, node2] {
            let incident: Vec<usize> = self.node(node)?.triangles().collect();
            for other in incident {
                CoherentTriangle::set_connection_any(&mut self.triangles, other, triangle)?;
            }
            self.change_node(node)?
                .add_triangle_in(triangle, allocator.clone())?;
        }
        if self.link_count > 0 {
            for side in 0..3 {
                self.update_side_link(triangle, side)?;
            }
        }
        Ok(true)
    }
    // Reuses the neighbour's link on a shared side, or creates a link for this side.
    fn update_side_link(&mut self, triangle: usize, side: usize) -> Result<(), Failure> {
        let own_node = self.triangles[triangle].nodes[side];
        let mut add = true;
        if let Some(opposite) = self.triangles[triangle].connected[side] {
            for j in 0..3 {
                if own_node != self.triangles[opposite].nodes_on_connected[j] {
                    continue;
                }
                let Some(link) = self.triangles[opposite].links[j] else {
                    continue;
                };
                let their_node = self.triangles[opposite].nodes[j];
                let record = &mut self.links[link];
                if record.opposite_node(0) == their_node {
                    record.opposite_nodes[1] = own_node;
                    add = false;
                } else if record.opposite_node(1) == their_node {
                    record.opposite_nodes[0] = own_node;
                    add = false;
                }
            }
        }
        if add {
            self.add_link(triangle, side as i32)?;
        }
        Ok(())
    }
    /// Removes triangles with a side shorter than `tolerance`, merging the side's end node
    /// into the other one; `removed` receives (removed, kept) node pairs;
    /// `mesh.remove_degenerated(1e-7, None)?`.
    pub fn remove_degenerated(
        &mut self,
        tolerance: f64,
        mut removed: Option<&mut List<TwoIntegers>>,
    ) -> Result<bool, Failure> {
        let tolerance2 = tolerance * tolerance;
        if let Some(list) = removed.as_deref_mut() {
            list.clear();
        }
        let mut result = false;
        // The native iterator stops at the size the array had when it started.
        for triangle in 0..self.triangles.len() {
            if self.triangles[triangle].is_empty() {
                continue;
            }
            if let Some(pair) = self.collapse_short_side(triangle, tolerance2)? {
                if let Some(list) = removed.as_deref_mut() {
                    list.append(pair);
                }
                result = true;
            }
        }
        Ok(result)
    }
    fn collapse_short_side(
        &mut self,
        triangle: usize,
        tolerance2: f64,
    ) -> Result<Option<TwoIntegers>, Failure> {
        const IND: [usize; 5] = [2, 0, 1, 2, 0];
        let nodes = self.triangles[triangle].nodes;
        let point = |this: &Self, k: usize| this.node(nodes[k]).map(|n| **n);
        let lengths = [
            (point(self, 2)? - point(self, 1)?).square_modulus(),
            (point(self, 0)? - point(self, 2)?).square_modulus(),
            (point(self, 1)? - point(self, 0)?).square_modulus(),
        ];
        let Some(side) = (0..3).find(|i| lengths[*i] < tolerance2) else {
            return Ok(None);
        };
        let (kept, merged) = (nodes[IND[side]], nodes[IND[side + 2]]);
        self.remove_triangle(triangle)?;
        self.reconnect_node(merged, kept, triangle)?;
        Ok(Some(TwoIntegers::new(merged, kept)))
    }
    // Moves every triangle of `merged` onto `kept`. A ring headed by the removed triangle
    // is skipped past as natively, where it would restart from the same head.
    fn reconnect_node(&mut self, merged: i32, kept: i32, removed: usize) -> Result<(), Failure> {
        loop {
            let ring: Vec<usize> = self.node(merged)?.triangles().collect();
            let Some(&head) = ring.first() else {
                return Ok(());
            };
            if head == removed {
                if ring.len() < 2 {
                    return Ok(());
                }
                return Err(Failure::with_type(
                    "Standard_ProgramError",
                    &format!(
                        "Poly_CoherentTriangulation::RemoveDegenerated: removed triangle {removed} still heads the ring of node {merged}; native code loops forever"
                    ),
                    "",
                ));
            }
            let mut nodes = self.triangle(head)?.nodes;
            if let Some(slot) = nodes.iter().position(|n| *n == merged) {
                nodes[slot] = kept;
            }
            self.remove_triangle(head)?;
            self.add_triangle(nodes[0], nodes[1], nodes[2])?;
        }
    }
}
