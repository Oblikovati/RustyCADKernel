// Rust translation of ModelingAlgorithms/TKShHealing/ShapeCustom at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! OCCT package ShapeCustom: the modifications that convert the geometry of shapes (to
//! B-splines, to revolutions, restrictions), applied with `BRepTools_Modifier`.
//!
//! Ported so far (docs/tkshhealing-progress.md): [`ShapeCustomCurve2d`] (linear p-curves
//! as lines, knot removal), [`ShapeCustomRestrictionParameters`], the abstract
//! [`ShapeCustomModification`] and the modifications [`ShapeCustomTrsfModification`],
//! [`ShapeCustomDirectModification`], [`ShapeCustomConvertToRevolution`] and
//! [`ShapeCustomConvertToBSpline`].
//! docs/shhealing-design.md section 7 fixes how they map.
mod bspline_restriction;
mod convert_to_bspline;
mod convert_to_revolution;
mod curve2d;
mod direct_modification;
mod edge_copy;
mod modification;
mod restriction_parameters;
mod surface;
mod swept_to_elementary;
mod trsf_modification;

pub use bspline_restriction::{
    ShapeCustomBSplineRestriction, ShapeCustomBSplineRestrictionSettings, ShapeCustomRestrictionBox,
};
pub use convert_to_bspline::ShapeCustomConvertToBSpline;
pub use convert_to_revolution::ShapeCustomConvertToRevolution;
pub use curve2d::{Line2dConversion, ShapeCustomCurve2d};
pub use direct_modification::ShapeCustomDirectModification;
pub use modification::{ShapeCustomModification, ShapeCustomModificationFields};
pub use restriction_parameters::ShapeCustomRestrictionParameters;
pub use surface::ShapeCustomSurface;
pub use swept_to_elementary::ShapeCustomSweptToElementary;
pub use trsf_modification::ShapeCustomTrsfModification;
