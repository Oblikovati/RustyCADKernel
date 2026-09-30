// Rust translation of Sweep_NumShape.hxx/.lxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `Sweep_NumShape`: the indexed representation of a directing edge or vertex.
use rustycad_tkg3d::top_abs::{Orientation, ShapeEnum};

/// `Sweep_NumShape`: a directing edge (its index is its number of vertices, 0, 1 or 2)
/// or a vertex of it (its index is its rank in the edge, 1 or 2), with the closed and
/// infinite flags of the edge.
///
/// ```
/// use rustycad_tkg3d::top_abs::{Orientation, ShapeEnum};
/// use rustycad_tkprim::sweep::SweepNumShape;
/// let edge = SweepNumShape::with_flags(2, ShapeEnum::Edge, false, false, false);
/// assert_eq!(edge.index(), 2);
/// assert_eq!(edge.orientation(), Orientation::Forward);
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SweepNumShape {
    my_type: ShapeEnum,
    my_index: i32,
    my_closed: bool,
    my_beg_inf: bool,
    my_end_inf: bool,
}

impl Default for SweepNumShape {
    /// `Sweep_NumShape()`: a dummy shape of type `TopAbs_SHAPE` and index 0.
    fn default() -> Self {
        Self::new(0, ShapeEnum::Shape)
    }
}

impl SweepNumShape {
    /// `Sweep_NumShape(Index, Type)` with the default flags (not closed, finite);
    /// `SweepNumShape::new(1, ShapeEnum::Vertex)`.
    pub fn new(index: i32, kind: ShapeEnum) -> Self {
        Self::with_flags(index, kind, false, false, false)
    }

    /// `Sweep_NumShape(Index, Type, Closed, BegInf, EndInf)`;
    /// `SweepNumShape::with_flags(0, ShapeEnum::Edge, false, true, true)`.
    pub fn with_flags(
        index: i32,
        kind: ShapeEnum,
        closed: bool,
        beg_inf: bool,
        end_inf: bool,
    ) -> Self {
        Self {
            my_type: kind,
            my_index: index,
            my_closed: closed,
            my_beg_inf: beg_inf,
            my_end_inf: end_inf,
        }
    }

    /// `Init(Index, Type, Closed, BegInf, EndInf)`: reinitializes every field;
    /// `shape.init(1, ShapeEnum::Edge, false, false, true)`.
    pub fn init(
        &mut self,
        index: i32,
        kind: ShapeEnum,
        closed: bool,
        beg_inf: bool,
        end_inf: bool,
    ) {
        *self = Self::with_flags(index, kind, closed, beg_inf, end_inf);
    }

    /// `Index()`; `shape.index()`.
    pub fn index(&self) -> i32 {
        self.my_index
    }

    /// `Type()`; `shape.kind()`.
    pub fn kind(&self) -> ShapeEnum {
        self.my_type
    }

    /// `Closed()`; `shape.closed()`.
    pub fn closed(&self) -> bool {
        self.my_closed
    }

    /// `BegInfinite()`; `shape.beg_infinite()`.
    pub fn beg_infinite(&self) -> bool {
        self.my_beg_inf
    }

    /// `EndInfinite()`; `shape.end_infinite()`.
    pub fn end_infinite(&self) -> bool {
        self.my_end_inf
    }

    /// `Orientation()`: FORWARD for an edge and for the vertex of index 2, REVERSED for
    /// any other shape (the vertex of index 1 and a dummy shape); `shape.orientation()`.
    pub fn orientation(&self) -> Orientation {
        if self.my_type == ShapeEnum::Edge || self.my_index == 2 {
            return Orientation::Forward;
        }
        Orientation::Reversed
    }
}
