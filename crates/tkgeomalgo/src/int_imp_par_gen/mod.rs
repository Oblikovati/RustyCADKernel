// Rust translation of ModelingAlgorithms/TKGeomAlgo/IntImpParGen at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! OCCT package IntImpParGen: the generic intersection of an implicit 2D curve with
//! a bounded parametric one.
//!
//! The static members of the class `IntImpParGen` are the free functions
//! [`determine_transition`], [`determine_crossing_transition`],
//! [`determine_position`] and [`normalize_on_domain`]; the free functions of
//! IntImpParGen_Tool.hxx are in [`tool`]. The template IntImpParGen_Intersector.gxx
//! is [`Intersector`], generic over [`IntersectorTypes`] (the `ImpTool`, `ParTool`
//! and `ProjectOnPCurveTool` macros), and its function class is [`MyImpParTool`];
//! IntCurve and Geom2dInt declare the native instantiations as aliases
//! (docs/geomalgo-design.md).
mod intersector;
mod intersector_domain;
mod intersector_segments;
mod my_imp_par_tool;
mod package;
pub mod tool;
mod traits;

pub use intersector::Intersector;
pub use intersector_domain::{IntervalBounds, ResultBounds};
pub use my_imp_par_tool::MyImpParTool;
pub use package::{
    TransitionSide, determine_crossing_transition, determine_position, determine_transition,
    normalize_on_domain,
};
pub(crate) use package::{set_crossing_transitions, set_touch_transitions};
pub use traits::{ImpTool, IntersectorTypes, ParTool, ProjectOnPCurveTool};
