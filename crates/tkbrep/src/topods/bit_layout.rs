// Rust translation of ModelingData/TKBRep/TopoDS/TopoDS_TShape.hxx (enum BitLayout) at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `TopoDS_TShape::BitLayout`: the layout of the 16-bit state word. Bits 0-3 hold the
//! `TopAbs_ShapeEnum` value, bits 4-11 the flags, bits 12-15 are reserved.

/// `Bits_ShapeType_Mask`: bits 0-3, the shape type.
pub const BITS_SHAPE_TYPE_MASK: u16 = 0x000F;
/// `Bits_ShapeType_Shift`: the shape type is not shifted.
pub const BITS_SHAPE_TYPE_SHIFT: u16 = 0;
/// `Bit_Free`: bit 4.
pub const BIT_FREE: u16 = 0x0010;
/// `Bit_Modified`: bit 5.
pub const BIT_MODIFIED: u16 = 0x0020;
/// `Bit_Checked`: bit 6.
pub const BIT_CHECKED: u16 = 0x0040;
/// `Bit_Orientable`: bit 7.
pub const BIT_ORIENTABLE: u16 = 0x0080;
/// `Bit_Closed`: bit 8.
pub const BIT_CLOSED: u16 = 0x0100;
/// `Bit_Infinite`: bit 9.
pub const BIT_INFINITE: u16 = 0x0200;
/// `Bit_Convex`: bit 10.
pub const BIT_CONVEX: u16 = 0x0400;
/// `Bit_Locked`: bit 11.
pub const BIT_LOCKED: u16 = 0x0800;
/// `Bits_Reserved`: bits 12-15.
pub const BITS_RESERVED: u16 = 0xF000;
