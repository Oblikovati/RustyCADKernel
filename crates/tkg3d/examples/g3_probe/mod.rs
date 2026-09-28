//! The TKG3d probe operations, one module per C++ probe file under tools/oracle.
pub mod advapprox;
pub mod bezier_surface;
pub mod bspl;
pub mod common;
pub mod conics;
pub mod curves;
#[path = "../../../tkg2d/examples/g2_probe/print.rs"]
pub mod print;
pub mod shapes;
pub mod simple;
pub mod surfaces;
pub mod swept;
