// Rust translation of DataExchange/TKXSBase at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! OCCT TKXSBase: the foundation of the data exchange formats (STEP, IGES and the DE
//! wrappers): interface models of entities with their checks, generic libraries keyed by
//! protocol, transfer processes with their binders and actors, graphs of entities,
//! selections and work sessions.
//!
//! TKXSBase depends on TKBRep, TKernel, TKMath, TKG2d, TKG3d, TKTopAlgo, TKGeomBase and
//! TKShHealing natively (its EXTERNLIB); this crate depends on their ports. Each of the
//! 10 OCCT packages is one module named after it in snake case (`MoniTool` is
//! `moni_tool`, `IFSelect` is `if_select`, `XSControl` is `xs_control`), declared here up
//! front; a module whose package is not ported yet holds only its documentation.
//! docs/xsbase-design.md fixes how the C++ constructs of the packages map to Rust.
mod attr_core;
mod c_text;
pub mod if_graph;
pub mod interface;
pub mod lib_ctl;
pub mod moni_tool;
mod ostream_text;
pub mod transfer;
pub mod xs_algo;
pub mod xs_message;
