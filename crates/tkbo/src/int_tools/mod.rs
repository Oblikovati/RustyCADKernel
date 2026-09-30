//! Intersection data structures from OCCT IntTools at revision 3d097a0.
mod base_range_sample;
mod curve_range_sample;
mod range;
mod root;
mod surface_range_sample;
pub use base_range_sample::BaseRangeSample;
pub use curve_range_sample::CurveRangeSample;
pub use range::Range;
pub use root::Root;
pub use surface_range_sample::SurfaceRangeSample;
