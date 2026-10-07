//! The TKService probe operations, one module per C++ probe file under tools/oracle.
pub mod grid;
pub mod hid;
pub mod pbr;
pub mod pixel;
// The reader and bit printers of the TKGeomAlgo probe, shared so that every toolkit
// prints the same token stream (tools/oracle/tkservice_probe.cxx includes
// tkgeomalgo_probe_common.cxx likewise). Each package uses a subset of them.
#[allow(dead_code, unused_imports)]
#[path = "../../../tkgeomalgo/examples/ga_probe/common.rs"]
pub mod common;
#[allow(dead_code)]
#[path = "../../../tkg2d/examples/g2_probe/print.rs"]
pub mod print;
