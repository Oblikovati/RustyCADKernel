//! The TKGeomBase probe operations, one module per C++ probe file under tools/oracle.
// The token printers and readers are shared by every package probe, and each package
// uses a subset of them, so unused ones are expected in these two modules.
pub mod app_par_curves;
pub mod bndlib;
#[allow(dead_code, unused_imports)]
pub mod common;
pub mod extrema_el;
pub mod fem_tool;
pub mod gc;
pub mod gce;
pub mod gcpnts;
pub mod geom_tools;
pub mod gprop;
pub mod hermit;
pub mod int_ana;
pub mod int_ana2d;
pub mod lprop;
pub mod pc2d;
pub mod pc3d;
#[allow(dead_code)]
#[path = "../../../tkg2d/examples/g2_probe/print.rs"]
pub mod print;
