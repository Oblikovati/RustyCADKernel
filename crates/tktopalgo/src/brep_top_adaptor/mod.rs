// Rust translation of ModelingAlgorithms/TKTopAlgo/BRepTopAdaptor at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! OCCT package BRepTopAdaptor: the topology tools of a face for the intersection
//! algorithms.
//!
//! [`BRepTopAdaptorFClass2d`] classifies UV points against a face through one polygon
//! per wire (exactly, with BRepClass, near the polygons); [`BRepTopAdaptorTopolTool`]
//! is the `Adaptor3d_TopolTool` of a face (its edge pcurves, their vertices as
//! [`BRepTopAdaptorHVertex`], the classification and the sample points);
//! [`BRepTopAdaptorTool`] pairs a surface with its tool. The derived classes of the
//! Adaptor3d bases are mapped as docs/topalgo-design.md section 4 fixes.
pub(crate) mod errors;
mod fclass2d;
mod fclass2d_build;
mod fclass2d_polygon;
mod fclass2d_refine;
mod h_vertex;
mod tool;
mod topol_tool;
mod topol_tool_domain;
mod topol_tool_samples;

pub use fclass2d::BRepTopAdaptorFClass2d;
pub use h_vertex::BRepTopAdaptorHVertex;
pub use tool::BRepTopAdaptorTool;
pub use topol_tool::BRepTopAdaptorTopolTool;
