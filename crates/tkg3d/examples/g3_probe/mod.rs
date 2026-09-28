//! The TKG3d probe operations, one module per C++ probe file under tools/oracle.
pub mod adaptor3d;
pub mod adaptor3d_inputs;
pub mod advapprox;
pub mod bezier_surface;
pub mod bspl;
pub mod bspline_surface;
pub mod common;
pub mod conics;
pub mod curves;
pub mod offset_surface;
#[path = "../../../tkg2d/examples/g2_probe/print.rs"]
pub mod print;
pub mod shapes;
pub mod simple;
pub mod surfaces;
pub mod swept;
