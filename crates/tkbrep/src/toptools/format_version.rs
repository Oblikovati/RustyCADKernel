// Rust translation of ModelingData/TKBRep/TopTools/TopTools_FormatVersion.hxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `TopTools_FormatVersion`: the versions of the TopTools ASCII format.

/// `TopTools_FormatVersion_VERSION_1`: no CurveOnSurface UV points; reading calls `Check`.
pub const VERSION_1: i32 = 1;
/// `TopTools_FormatVersion_VERSION_2`: stores the CurveOnSurface UV points.
pub const VERSION_2: i32 = 2;
/// `TopTools_FormatVersion_VERSION_3`: stores per-vertex normals of triangulation-only faces.
pub const VERSION_3: i32 = 3;
/// `TopTools_FormatVersion_CURRENT`.
pub const CURRENT: i32 = VERSION_3;
/// `TopTools_FormatVersion_LOWER`.
pub const LOWER: i32 = VERSION_1;
/// `TopTools_FormatVersion_UPPER`.
pub const UPPER: i32 = VERSION_3;
