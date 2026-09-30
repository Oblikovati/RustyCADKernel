// Rust translation of ModelingData/TKG3d/TopAbs/TopAbs_ShapeEnum.hxx, TopAbs_Orientation.hxx, TopAbs_State.hxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The three TopAbs enumerations. Discriminants keep the native declaration order,
//! so `as i32` is the integer OCCT stores, and `ALL` lists the enumerators in that
//! order for the loops of the package functions.

/// The type of a topological shape, from the most to the least complex
/// (`TopAbs_ShapeEnum`); `ShapeEnum::Face as i32 == 4`.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, PartialOrd, Ord)]
#[repr(i32)]
pub enum ShapeEnum {
    /// A group of any of the shapes below.
    Compound,
    /// A set of solids connected by their faces.
    CompSolid,
    /// A part of 3D space bounded by shells.
    Solid,
    /// A set of faces connected by some of the edges of their wire boundaries.
    Shell,
    /// A part of a surface bounded by a closed wire.
    Face,
    /// A sequence of edges connected by their vertices.
    Wire,
    /// A curve bounded by a vertex at each extremity.
    Edge,
    /// A zero-dimensional shape corresponding to a point.
    Vertex,
    /// Any shape (the generic value).
    Shape,
}
impl ShapeEnum {
    /// Every shape type in declaration order; `ShapeEnum::ALL[8] == ShapeEnum::Shape`.
    pub const ALL: [ShapeEnum; 9] = [
        ShapeEnum::Compound,
        ShapeEnum::CompSolid,
        ShapeEnum::Solid,
        ShapeEnum::Shell,
        ShapeEnum::Face,
        ShapeEnum::Wire,
        ShapeEnum::Edge,
        ShapeEnum::Vertex,
        ShapeEnum::Shape,
    ];
}

/// The orientation of a topological shape, or of a shape relative to another
/// (`TopAbs_Orientation`); `Orientation::Reversed as i32 == 1`.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, PartialOrd, Ord)]
#[repr(i32)]
pub enum Orientation {
    /// The default region is on the left of the direction of the shape.
    Forward,
    /// The default region is on the right.
    Reversed,
    /// The shape is inside the domain: both regions are interior.
    Internal,
    /// The shape is outside the domain: both regions are exterior.
    External,
}
impl Orientation {
    /// Every orientation in declaration order; `Orientation::ALL[2] == Orientation::Internal`.
    pub const ALL: [Orientation; 4] = [
        Orientation::Forward,
        Orientation::Reversed,
        Orientation::Internal,
        Orientation::External,
    ];
}

/// The position of a point or a set of points relative to a region of a shape
/// (`TopAbs_State`); `State::On as i32 == 2`.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, PartialOrd, Ord)]
#[repr(i32)]
pub enum State {
    /// Inside the region.
    In,
    /// Outside the region.
    Out,
    /// On the boundary of the region.
    On,
    /// Not classified.
    Unknown,
}
impl State {
    /// Every state in declaration order; `State::ALL[3] == State::Unknown`.
    pub const ALL: [State; 4] = [State::In, State::Out, State::On, State::Unknown];
}
