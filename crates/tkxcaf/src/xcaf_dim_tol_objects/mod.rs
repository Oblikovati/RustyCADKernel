//! The XCAFDimTolObjects package: the description objects of the dimensions, geometric
//! tolerances and datums of a document (GD&T), their enumerations, and the tool that reads
//! them from a document.
mod datum_object;
mod dimension_enums;
mod dimension_object;
mod dimension_presentation;
mod geom_tolerance_object;
mod tolerance_enums;

pub use datum_object::XCAFDimTolObjectsDatumObject;
pub use dimension_enums::*;
pub use dimension_object::XCAFDimTolObjectsDimensionObject;
pub(crate) use dimension_object::{array_values, real_array};
pub use geom_tolerance_object::XCAFDimTolObjectsGeomToleranceObject;
pub use tolerance_enums::*;
