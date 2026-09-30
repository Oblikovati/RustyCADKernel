// Rust translation of FoundationClasses/TKMath/Poly/Poly_CoherentLink.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

use super::CoherentTriangle;
use rustycad_tkernel::standard::Failure;
/// `Poly_CoherentLink`: two nodes (ascending when built from a triangle) and the opposite
/// node of the triangle on each side. The native `void*` attribute is an opaque `usize`
/// tag here; `CoherentLink::from_nodes(1, 2)`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CoherentLink {
    pub(super) nodes: [i32; 2],
    pub(super) opposite_nodes: [i32; 2],
    attribute: Option<usize>,
}
impl Default for CoherentLink {
    fn default() -> Self {
        Self::from_nodes(-1, -1)
    }
}
impl CoherentLink {
    /// An empty (invalid) link; `CoherentLink::new()`.
    pub fn new() -> Self {
        Self::default()
    }
    /// A temporary link on two nodes without opposite nodes; `CoherentLink::from_nodes(1, 2)`.
    pub const fn from_nodes(node0: i32, node1: i32) -> Self {
        Self {
            nodes: [node0, node1],
            opposite_nodes: [-1, -1],
            attribute: None,
        }
    }
    /// The side of a triangle opposite to node `side` (0, 1 or 2), ordered so that node 0 is
    /// the smaller; `CoherentLink::from_triangle(&triangle, 0)?`.
    pub fn from_triangle(triangle: &CoherentTriangle, side: i32) -> Result<Self, Failure> {
        if !(0..=2).contains(&side) {
            return Err(Failure::with_type(
                "Standard_ProgramError",
                &format!(
                    "Poly_CoherentLink::Poly_CoherentLink: Wrong iSide parameter {side}; expected 0, 1 or 2"
                ),
                "",
            ));
        }
        const NEXT: [i32; 4] = [1, 2, 0, 1];
        let side_index = side as usize;
        let ends = [
            triangle.node(NEXT[side_index]),
            triangle.node(NEXT[side_index + 1]),
        ];
        let (own, connected) = (triangle.node(side), triangle.get_connected_node(side));
        let (nodes, opposite_nodes) = if ends[0] < ends[1] {
            (ends, [own, connected])
        } else {
            ([ends[1], ends[0]], [connected, own])
        };
        Ok(Self {
            nodes,
            opposite_nodes,
            attribute: None,
        })
    }
    /// Node 0 or 1 (only the lowest bit of `ind` counts); `link.node(0)`.
    pub const fn node(&self, ind: i32) -> i32 {
        self.nodes[(ind & 0x1) as usize]
    }
    /// Opposite node on the left (0) or right (1) side; `link.opposite_node(1)`.
    pub const fn opposite_node(&self, ind: i32) -> i32 {
        self.opposite_nodes[(ind & 0x1) as usize]
    }
    /// The attribute tag, `None` when unset; `link.get_attribute()`.
    pub const fn get_attribute(&self) -> Option<usize> {
        self.attribute
    }
    /// Sets the attribute tag; `link.set_attribute(Some(7))`.
    pub const fn set_attribute(&mut self, attribute: Option<usize>) {
        self.attribute = attribute;
    }
    /// True when either node is negative; `link.is_empty()`.
    pub const fn is_empty(&self) -> bool {
        self.nodes[0] < 0 || self.nodes[1] < 0
    }
    /// Invalidates nodes and opposite nodes, keeping the attribute; `link.nullify()`.
    pub const fn nullify(&mut self) {
        self.nodes = [-1, -1];
        self.opposite_nodes = [-1, -1];
    }
}
