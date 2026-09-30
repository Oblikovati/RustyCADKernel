//! The TKPrim probe operations, one module per C++ probe file under tools/oracle.
pub mod brep_prim;
pub mod brep_prim_one_axis;
pub mod shape;
pub mod sweep;

// The reader and bit printers of the TKGeomAlgo probe and the shape digest of the TKTopAlgo
// probe, shared so that every toolkit prints the same token stream
// (tools/oracle/tkprim_probe.cxx includes tkgeomalgo_probe_common.cxx and
// tktopalgo_probe_breplib_common.cxx likewise). Each package uses a subset of them.
#[allow(dead_code)]
#[path = "../../../tktopalgo/examples/tp_probe/brep_lib_common.rs"]
pub mod brep_lib_common;
#[allow(dead_code, unused_imports)]
#[path = "../../../tkgeomalgo/examples/ga_probe/common.rs"]
pub mod common;
#[allow(dead_code)]
#[path = "../../../tkg2d/examples/g2_probe/print.rs"]
pub mod print;
