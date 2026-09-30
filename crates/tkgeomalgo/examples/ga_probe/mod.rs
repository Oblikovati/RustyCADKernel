//! The TKGeomAlgo probe operations, one module per C++ probe file under tools/oracle.
// The token printers and readers are shared by every package probe, and each package
// uses a subset of them, so unused ones are expected in these two modules.
pub mod app_blend;
#[allow(dead_code, unused_imports)]
pub mod common;
pub mod curve2d;
pub mod fair_curve;
pub mod gcc_ana;
pub mod gcc_ana_lines;
pub mod gcc_ent;
pub mod gcc_int;
pub mod geom2d_api;
pub mod geom2d_gcc;
pub mod geom2d_gcc_circles;
pub mod geom2d_gcc_lines;
pub mod geom2d_hatch;
pub mod geom2d_hatcher;
pub mod geom2d_int;
pub mod gfill;
pub mod gfill_cf;
pub mod gfill_fill;
pub mod ginter;
pub mod hatch;
pub mod hatch_gen;
pub mod ics_geometry;
pub mod ics_interutils;
pub mod ics_polyhedron;
pub mod int_curve;
pub mod int_curve_surface;
pub mod int_curve_surface_inter;
pub mod int_imp;
pub mod int_imp_par_gen;
pub mod int_res2d;
pub mod int_surf;
pub mod int_walk;
pub mod int_walk_iw;
pub mod intf;
pub mod intf_poly;
pub mod intf_tool;
pub mod law;
pub mod local_analysis;
pub mod plate;
#[allow(dead_code)]
#[path = "../../../tkg2d/examples/g2_probe/print.rs"]
pub mod print;
pub mod toptrans;
