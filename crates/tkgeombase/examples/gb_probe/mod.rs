//! The TKGeomBase probe operations, one module per C++ probe file under tools/oracle.
// The token printers and readers are shared by every package probe, and each package
// uses a subset of them, so unused ones are expected in these two modules.
pub mod adv_class;
pub mod adv_eval;
pub mod adv_f2var;
pub mod adv_f2var2;
pub mod adv_math;
pub mod adv_math2;
pub mod app_cont;
pub mod app_par_curves;
pub mod bndlib;
#[allow(dead_code, unused_imports)]
pub mod common;
pub mod extrema_cc;
pub mod extrema_cs;
pub mod extrema_el;
pub mod extrema_el_surf;
pub mod extrema_pc;
pub mod extrema_ps;
pub mod extrema_ss;
pub mod extrema_swept;
pub mod fem_tool;
pub mod gc;
pub mod gce;
pub mod gcpnts;
pub mod geom2d_convert;
pub mod geom_convert;
pub mod geom_lib;
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
