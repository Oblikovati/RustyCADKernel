use rustycad_tkbrep::topods::Shape;
/// Two shape occurrences retaining shared topology and independent orientation.
#[derive(Clone, Default)]
pub struct CoupleOfShape {
    first: Shape,
    second: Shape,
}
impl CoupleOfShape {
    /// Create two null occurrences; `let pair = CoupleOfShape::new();`.
    pub fn new() -> Self {
        Self::default()
    }
    /// Copy the first occurrence; `pair.set_shape1(&shape);`.
    pub fn set_shape1(&mut self, shape: &Shape) {
        self.first = shape.clone();
    }
    /// Borrow the first occurrence; `let first = pair.shape1();`.
    pub fn shape1(&self) -> &Shape {
        &self.first
    }
    /// Copy the second occurrence; `pair.set_shape2(&shape);`.
    pub fn set_shape2(&mut self, shape: &Shape) {
        self.second = shape.clone();
    }
    /// Borrow the second occurrence; `let second = pair.shape2();`.
    pub fn shape2(&self) -> &Shape {
        &self.second
    }
}
