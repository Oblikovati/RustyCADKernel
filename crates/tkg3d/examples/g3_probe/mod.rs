//! The TKG3d probe operations, one module per C++ probe file under tools/oracle.
pub mod bspl;
pub mod common;
pub mod curves;
#[path = "../../../tkg2d/examples/g2_probe/print.rs"]
pub mod print;
pub mod shapes;
pub mod simple;
