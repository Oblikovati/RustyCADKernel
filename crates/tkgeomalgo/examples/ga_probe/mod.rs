//! The TKGeomAlgo probe operations, one module per C++ probe file under tools/oracle.
// The token printers and readers are shared by every package probe, and each package
// uses a subset of them, so unused ones are expected in these two modules.
#[allow(dead_code, unused_imports)]
pub mod common;
pub mod curve2d;
pub mod geom2d_api;
pub mod geom2d_int;
pub mod ginter;
pub mod int_curve;
pub mod int_imp_par_gen;
pub mod int_res2d;
pub mod intf;
pub mod intf_poly;
pub mod intf_tool;
#[allow(dead_code)]
#[path = "../../../tkg2d/examples/g2_probe/print.rs"]
pub mod print;
pub mod toptrans;
