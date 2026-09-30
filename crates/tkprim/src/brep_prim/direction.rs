// Rust translation of BRepPrim_Direction.hxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `BRepPrim_Direction`: the six directions of the faces of a wedge.

/// `BRepPrim_Direction`: a face of a wedge by the axis it is normal to and its side;
/// `BRepPrimDirection::ZMax as i32 == 5`.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[repr(i32)]
pub enum BRepPrimDirection {
    /// `BRepPrim_XMin`: the face at XMin (the back face of a box).
    XMin,
    /// `BRepPrim_XMax`: the face at XMax (the front face of a box).
    XMax,
    /// `BRepPrim_YMin`: the face at YMin (the left face of a box).
    YMin,
    /// `BRepPrim_YMax`: the face at YMax (the right face of a box).
    YMax,
    /// `BRepPrim_ZMin`: the face at ZMin (the bottom face of a box).
    ZMin,
    /// `BRepPrim_ZMax`: the face at ZMax (the top face of a box).
    ZMax,
}

impl BRepPrimDirection {
    /// Every direction in declaration order; `BRepPrimDirection::ALL[2] == BRepPrimDirection::YMin`.
    pub const ALL: [BRepPrimDirection; 6] = [
        BRepPrimDirection::XMin,
        BRepPrimDirection::XMax,
        BRepPrimDirection::YMin,
        BRepPrimDirection::YMax,
        BRepPrimDirection::ZMin,
        BRepPrimDirection::ZMax,
    ];
}
