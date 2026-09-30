// Rust translation of ModelingAlgorithms/TKGeomAlgo/GeomFill/GeomFill_ApproxStyle.hxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `GeomFill_ApproxStyle`: which part of a sweep drives its approximation.

/// The approximation style (`GeomFill_ApproxStyle`), with the native values;
/// `GeomFillApproxStyle::Location as i32 == 1`.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub enum GeomFillApproxStyle {
    /// `GeomFill_Section`.
    #[default]
    Section = 0,
    /// `GeomFill_Location`.
    Location = 1,
}
