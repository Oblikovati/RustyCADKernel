//! The TKTopAlgo probe operations, one module per C++ probe file under tools/oracle.
pub mod bisector;
pub mod bisector_algo;
pub mod brep_builder_api;
pub mod mat;
pub mod mat_parts;
// The reader and bit printers of the TKGeomAlgo probe, shared so that both toolkits
// print the same token stream (tools/oracle/tktopalgo_probe.cxx includes
// tkgeomalgo_probe_common.cxx likewise). Each package uses a subset of them.
#[allow(dead_code, unused_imports)]
#[path = "../../../tkgeomalgo/examples/ga_probe/common.rs"]
pub mod common;
#[allow(dead_code)]
#[path = "../../../tkg2d/examples/g2_probe/print.rs"]
pub mod print;
// The 2D curve reader of the TKGeomAlgo probe (ga_curve2d), which the Bisector, MAT and
// MAT2d probes share with it.
#[path = "../../../tkgeomalgo/examples/ga_probe/curve2d.rs"]
pub mod curve2d;
