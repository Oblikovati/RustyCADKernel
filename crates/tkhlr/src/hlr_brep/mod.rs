// HLRBRep at OCCT 3d097a0328e71b826377d4814ab05ec3c3d23871.
//! Exact hidden-line curve and topology tools.
mod area_limit;
pub use area_limit::AreaLimit;
mod b_curve_tool;
pub use b_curve_tool::BCurveTool;

mod b_surface_tool;
pub use b_surface_tool::BSurfaceTool;

// Consumed by the remaining HLRBRep_Data port; not a public OCCT declaration.
#[cfg_attr(not(test), expect(dead_code))]
mod rejection_cache;
