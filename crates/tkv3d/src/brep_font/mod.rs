//! Martensite port of OCCT's BRepFont package (3d097a0): glyph outline regularization
//! into planar topology and the TopoDS / BRepGraph glyph and text builders.
//!
//! - [`BRepFontPlanarRegion`](crate::brep_font::BRepFontPlanarRegion): the regularized vertex, curve, use, loop and region pools.
//! - [`BRepFontRegularizer`](crate::brep_font::BRepFontRegularizer): outline validation, healing, intersection checks and loop
//!   classification by the fill rule.
//! - [`BRepFontBuilder`](crate::brep_font::BRepFontBuilder): faces, wires and located text occurrences.
//!
//! The regularizer computes its quadratic roots in the x87 `long double` of the pinned
//! Linux/GCC oracle profile (`rustycad_tkmath::math_poly::LongDouble`).
mod builder;
mod builder_glyph;
mod builder_graph;
mod planar_region;
mod planar_region_curve;
mod regularizer;
mod regularizer_append;
mod regularizer_classify;
mod regularizer_intersect;
mod regularizer_line_intersect;
mod regularizer_scanline;

pub use builder::{
    BRepFontBuilder, BRepFontBuilderGlyphOptions, BRepFontBuilderTextOptions,
    BRepFontBuilderTextPlan, BRepFontBuilderTextPlanItem,
};
pub use planar_region::{
    BRepFontCurveEntity, BRepFontCurveId, BRepFontLoopEntity, BRepFontLoopId, BRepFontPlanarRegion,
    BRepFontPlanarRegionEntity, BRepFontPlanarRegionEntityKind, BRepFontPlanarRegionId,
    BRepFontPlanarRegionLoop, BRepFontPlanarRegionRegion, BRepFontPlanarRegionUse,
    BRepFontVertexEntity, BRepFontVertexId,
};
pub use planar_region_curve::BRepFontPlanarRegionCurve;
pub use regularizer::{
    BRepFontRegularizer, BRepFontRegularizerOptions, BRepFontRegularizerResult,
    BRepFontRegularizerStatus,
};
