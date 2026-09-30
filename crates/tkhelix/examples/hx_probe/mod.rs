//! The TKHelix probe operations, one module per C++ probe file under tools/oracle.
pub mod placement;
// The reader and bit printers of the TKGeomAlgo probe, shared so that both toolkits
// print the same token stream (tools/oracle/tkhelix_probe.cxx includes
// tkgeomalgo_probe_common.cxx likewise). Each package uses a subset of them.
#[allow(dead_code, unused_imports)]
#[path = "../../../tkgeomalgo/examples/ga_probe/common.rs"]
pub mod common;
#[allow(dead_code)]
#[path = "../../../tkg2d/examples/g2_probe/print.rs"]
pub mod print;
