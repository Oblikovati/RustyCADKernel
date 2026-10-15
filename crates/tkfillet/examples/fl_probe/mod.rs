//! The TKFillet probe operations, one module per C++ probe file under tools/oracle.
pub mod api;
pub mod app_func;
pub mod blend_func;
pub mod brep_blend;
pub mod builder0;
pub mod builder0_solve;
pub mod builder0_topo;
pub mod chamfer;
pub mod chfi2d;
pub mod chfi3d;
pub mod chfi_ds;
pub mod chfikpart;
pub mod fillet;
pub mod line_builders;
pub mod rst_rst;
pub mod surf_rst;
pub mod walking;
// The reader and bit printers of the TKGeomAlgo probe, shared so that both toolkits
// print the same token stream (tools/oracle/tkfillet_probe.cxx includes
// tkgeomalgo_probe_common.cxx likewise). Each package uses a subset of them.
#[allow(dead_code, unused_imports)]
#[path = "../../../tkgeomalgo/examples/ga_probe/common.rs"]
pub mod common;
#[allow(dead_code)]
#[path = "../../../tkg2d/examples/g2_probe/print.rs"]
pub mod print;
