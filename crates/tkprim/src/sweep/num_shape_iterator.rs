// Rust translation of Sweep_NumShapeIterator.hxx/.lxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `Sweep_NumShapeIterator`: the vertices of a directing edge.
use super::SweepNumShape;
use rustycad_tkg3d::top_abs::{Orientation, ShapeEnum};

/// `Sweep_NumShapeIterator`: iterates on the vertices of a directing edge, with the
/// orientation each vertex has in the edge.
///
/// ```
/// use rustycad_tkg3d::top_abs::{Orientation, ShapeEnum};
/// use rustycad_tkprim::sweep::{SweepNumShape, SweepNumShapeIterator};
/// let mut it = SweepNumShapeIterator::new();
/// it.init(&SweepNumShape::new(2, ShapeEnum::Edge));
/// assert_eq!(it.value().index(), 1);
/// assert_eq!(it.orientation(), Orientation::Forward);
/// it.next();
/// assert_eq!(it.orientation(), Orientation::Reversed);
/// it.next();
/// assert!(!it.more());
/// ```
#[derive(Clone, Copy, Debug)]
pub struct SweepNumShapeIterator {
    my_num_shape: SweepNumShape,
    my_current_num_shape: SweepNumShape,
    my_current_range: i32,
    my_more: bool,
    my_current_orientation: Orientation,
}

impl Default for SweepNumShapeIterator {
    fn default() -> Self {
        Self::new()
    }
}

impl SweepNumShapeIterator {
    /// `Sweep_NumShapeIterator()`: no current vertex. The native constructor leaves the
    /// current orientation uninitialized; it starts FORWARD here
    /// (docs/tkprim-progress.md); `SweepNumShapeIterator::new()`.
    pub fn new() -> Self {
        Self {
            my_num_shape: SweepNumShape::new(0, ShapeEnum::Shape),
            my_current_num_shape: SweepNumShape::new(0, ShapeEnum::Shape),
            my_current_range: 0,
            my_more: false,
            my_current_orientation: Orientation::Forward,
        }
    }

    /// `Init(aShape)`: restarts on the vertices of `shape`. Only an edge resets the
    /// iteration; for any other shape the native body keeps the current state
    /// (docs/tkprim-progress.md); `it.init(&edge)`.
    pub fn init(&mut self, shape: &SweepNumShape) {
        self.my_num_shape = *shape;
        if shape.kind() != ShapeEnum::Edge {
            return;
        }
        let nbvert = shape.index();
        self.my_more = nbvert >= 1;
        if !self.my_more {
            return;
        }
        self.my_current_range = 1;
        self.my_current_num_shape =
            SweepNumShape::with_flags(1, ShapeEnum::Vertex, shape.closed(), false, false);
        // A lone vertex of an edge infinite at its beginning ends the edge.
        self.my_current_orientation = if nbvert == 1 && shape.beg_infinite() {
            Orientation::Reversed
        } else {
            Orientation::Forward
        };
    }

    /// `More()`; `it.more()`.
    pub fn more(&self) -> bool {
        self.my_more
    }

    /// `Next()`: moves to the next vertex, which ends the edge (REVERSED);
    /// `it.next()`.
    pub fn next(&mut self) {
        self.my_current_range += 1;
        self.my_more = self.my_current_range <= self.my_num_shape.index();
        if self.my_more && self.my_num_shape.kind() == ShapeEnum::Edge {
            self.my_current_num_shape = SweepNumShape::with_flags(
                self.my_current_range,
                ShapeEnum::Vertex,
                self.my_num_shape.closed(),
                false,
                false,
            );
            self.my_current_orientation = Orientation::Reversed;
        }
    }

    /// `Value()`: the current vertex; `it.value()`.
    pub fn value(&self) -> &SweepNumShape {
        &self.my_current_num_shape
    }

    /// `Orientation()`: the orientation of the current vertex in the edge;
    /// `it.orientation()`.
    pub fn orientation(&self) -> Orientation {
        self.my_current_orientation
    }
}
