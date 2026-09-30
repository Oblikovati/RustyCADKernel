// Rust translation of ModelingData/TKGeomBase/GeomLProp at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! GeomLProp: local properties of Geom curves and surfaces.
//!
//! [`CLPropsBase`] (`GeomLProp_CLPropsBase`, with the aliases [`CLProps`] and
//! [`CLProps2d`]) and [`SLPropsBase`] ([`SLProps`]) are the native class templates,
//! generic over the curve or surface type and the access policy;
//! [`CurAndInf2d`] finds the curvature extrema and inflections of a 2D curve, and
//! [`continuity`] measures the continuity of two curves at a junction.
mod clprops;
mod continuity;
mod cur_and_inf2d;
mod numeric_cur_inf2d;
mod slprops;
pub mod surface_access;
mod surface_curvatures;
pub mod surface_utils;

pub use clprops::{CLProps, CLProps2d, CLPropsBase};
pub use continuity::{continuity, continuity_with_tolerances};
pub use cur_and_inf2d::CurAndInf2d;
pub use slprops::{SLProps, SLPropsBase};
