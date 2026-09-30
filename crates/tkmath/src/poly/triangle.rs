// Rust translation of FoundationClasses/TKMath/Poly/Poly_Triangle.hxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

use rustycad_tkernel::{ncollection::Array1, standard::Failure};
/// `Poly_Triangle`: three one-based node indices of a triangulation; `Triangle::new(1, 2, 3)`.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub struct Triangle {
    nodes: [i32; 3],
}
/// `Poly_Array1OfTriangle`; `Array1OfTriangle::with_bounds(1, 4)?`.
pub type Array1OfTriangle = Array1<'static, Triangle>;
impl Triangle {
    /// Stores the three node indices; `Triangle::new(1, 2, 3)`.
    pub const fn new(n1: i32, n2: i32, n3: i32) -> Self {
        Self {
            nodes: [n1, n2, n3],
        }
    }
    /// Replaces all three nodes; `triangle.set(4, 5, 6)`.
    pub const fn set(&mut self, n1: i32, n2: i32, n3: i32) {
        self.nodes = [n1, n2, n3];
    }
    /// Replaces the node at position 1, 2 or 3; `triangle.set_node(2, 7)?`.
    pub fn set_node(&mut self, index: i32, node: i32) -> Result<(), Failure> {
        *self.slot(index, "Set")? = node;
        Ok(())
    }
    /// Returns the three nodes in order; `let (a, b, c) = triangle.get();`.
    pub const fn get(&self) -> (i32, i32, i32) {
        (self.nodes[0], self.nodes[1], self.nodes[2])
    }
    /// Returns the node at position 1, 2 or 3; `triangle.value(1)?`.
    pub fn value(&self, index: i32) -> Result<i32, Failure> {
        Self::position(index, "Value").map(|position| self.nodes[position])
    }
    /// Borrows the node at position 1, 2 or 3 for writing; `*triangle.change_value(3)? = 9`.
    pub fn change_value(&mut self, index: i32) -> Result<&mut i32, Failure> {
        self.slot(index, "ChangeValue")
    }
    fn slot(&mut self, index: i32, member: &str) -> Result<&mut i32, Failure> {
        Self::position(index, member).map(|position| &mut self.nodes[position])
    }
    fn position(index: i32, member: &str) -> Result<usize, Failure> {
        if !(1..=3).contains(&index) {
            return Err(Failure::out_of_range(&format!(
                "Poly_Triangle::{member}(), invalid index {index}; expected 1, 2 or 3"
            )));
        }
        Ok((index - 1) as usize)
    }
}
