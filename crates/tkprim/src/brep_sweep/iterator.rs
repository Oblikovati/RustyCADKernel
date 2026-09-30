// Rust translation of BRepSweep_Iterator.hxx/.lxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `BRepSweep_Iterator`: the direct sub-shapes of a generating shape.
use rustycad_tkbrep::topods::{Shape, ShapeIterator};
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::top_abs::Orientation;

/// `BRepSweep_Iterator`: a `TopoDS_Iterator` (composing orientations and locations)
/// over the direct sub-shapes of a generating shape.
///
/// ```
/// use rustycad_tkbrep::topods::Shape;
/// use rustycad_tkprim::brep_sweep::BRepSweepIterator;
/// let mut it = BRepSweepIterator::new();
/// it.init(&Shape::default());
/// assert!(!it.more());
/// ```
#[derive(Clone, Default)]
pub struct BRepSweepIterator {
    my_iterator: ShapeIterator,
}

impl BRepSweepIterator {
    /// `BRepSweep_Iterator()`: no current sub-shape; `BRepSweepIterator::new()`.
    pub fn new() -> Self {
        Self::default()
    }

    /// `Init(aShape)`: restarts on the sub-shapes of `shape`; `it.init(&shape)`.
    pub fn init(&mut self, shape: &Shape) {
        self.my_iterator.initialize(shape, true, true);
    }

    /// `More()`; `it.more()`.
    pub fn more(&self) -> bool {
        self.my_iterator.more()
    }

    /// `Next()`; `it.next()`.
    pub fn next(&mut self) {
        self.my_iterator.next();
    }

    /// `Value()`: the current sub-shape; Standard_NoSuchObject past the end;
    /// `it.value()?`.
    pub fn value(&self) -> Result<&Shape, Failure> {
        self.my_iterator.value()
    }

    /// `Orientation()`: the orientation of the current sub-shape; `it.orientation()?`.
    pub fn orientation(&self) -> Result<Orientation, Failure> {
        Ok(self.value()?.orientation())
    }
}
