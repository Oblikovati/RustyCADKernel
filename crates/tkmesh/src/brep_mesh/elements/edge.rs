use super::DegreeOfFreedom;
use rustycad_tkernel::ncollection::HashKey;
use std::ops::Deref;
/// Oriented pair of node indices; `BRepMeshOrientedEdge::new(1,2)`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BRepMeshOrientedEdge {
    first: i32,
    last: i32,
}
impl Default for BRepMeshOrientedEdge {
    fn default() -> Self {
        Self::new(-1, -1)
    }
}
impl BRepMeshOrientedEdge {
    /// Preserve node order without validation; `BRepMeshOrientedEdge::new(1,2)`.
    pub fn new(first: i32, last: i32) -> Self {
        Self { first, last }
    }
    /// Read the first node; `edge.first_node()`.
    pub fn first_node(&self) -> i32 {
        self.first
    }
    /// Read the last node; `edge.last_node()`.
    pub fn last_node(&self) -> i32 {
        self.last
    }
    /// Compare ordered indices; `edge.is_equal(&other)`.
    pub fn is_equal(&self, other: &Self) -> bool {
        self == other
    }
}
fn packed_node_hash(first: u16, last: u16) -> usize {
    let [a, b] = first.to_ne_bytes();
    let [c, d] = last.to_ne_bytes();
    u32::from_ne_bytes([a, b, c, d]) as usize
}
impl HashKey for BRepMeshOrientedEdge {
    fn occt_hash(&self) -> usize {
        packed_node_hash(self.first as u16, self.last as u16)
    }
}
/// Undirected mesh link with native oriented-edge base; `BRepMeshEdge::new(1,2,DegreeOfFreedom::Free)`.
#[derive(Clone, Copy, Debug)]
pub struct BRepMeshEdge {
    oriented: BRepMeshOrientedEdge,
    movability: DegreeOfFreedom,
}
impl Default for BRepMeshEdge {
    fn default() -> Self {
        Self::new(-1, -1, DegreeOfFreedom::Deleted)
    }
}
impl Deref for BRepMeshEdge {
    type Target = BRepMeshOrientedEdge;
    fn deref(&self) -> &Self::Target {
        &self.oriented
    }
}
impl BRepMeshEdge {
    /// Construct a link retaining endpoint order; `BRepMeshEdge::new(1,2,DegreeOfFreedom::Frontier)`.
    pub fn new(first: i32, last: i32, movability: DegreeOfFreedom) -> Self {
        Self {
            oriented: BRepMeshOrientedEdge::new(first, last),
            movability,
        }
    }
    /// Read the native constraint flag; `edge.movability()`.
    pub fn movability(&self) -> DegreeOfFreedom {
        self.movability
    }
    /// Replace the constraint flag; `edge.set_movability(DegreeOfFreedom::Deleted)`.
    pub fn set_movability(&mut self, movability: DegreeOfFreedom) {
        self.movability = movability;
    }
    /// Compare ordered indices regardless of deletion; `edge.is_same_orientation(&other)`.
    pub fn is_same_orientation(&self, other: &Self) -> bool {
        self.oriented.is_equal(&other.oriented)
    }
    /// Compare either endpoint order, excluding deleted links; `edge.is_equal(&other)`.
    pub fn is_equal(&self, other: &Self) -> bool {
        if self.movability == DegreeOfFreedom::Deleted
            || other.movability == DegreeOfFreedom::Deleted
        {
            return false;
        }
        self.is_same_orientation(other)
            || (self.first_node() == other.last_node() && self.last_node() == other.first_node())
    }
}
impl PartialEq for BRepMeshEdge {
    fn eq(&self, other: &Self) -> bool {
        self.is_equal(other)
    }
}
impl HashKey for BRepMeshEdge {
    fn occt_hash(&self) -> usize {
        let mut nodes = [self.first_node() as u16, self.last_node() as u16];
        nodes.sort_unstable();
        packed_node_hash(nodes[0], nodes[1])
    }
}
