//! The TKTopAlgo probe operations, one module per C++ probe file under tools/oracle.
pub mod bisector;
pub mod bisector_algo;
pub mod brep_approx;
pub mod brep_approx_parts;
pub mod brep_approx_tool;
pub mod brep_bnd_lib;
pub mod brep_bnd_lib_obb;
pub mod brep_builder_api;
pub mod brep_builder_api_makers;
pub mod brep_builder_api_modify;
pub mod brep_check;
pub mod brep_class;
pub mod brep_class3d;
pub mod brep_gprop;
pub mod brep_gprop_frames;
pub mod brep_gprop_tools;
pub mod brep_int_curve_surface;
pub mod brep_lib_common;
pub mod brep_lib_edge;
pub mod brep_lib_face;
pub mod brep_lib_fuse;
pub mod brep_lib_package;
pub mod brep_lib_point_cloud;
pub mod brep_lib_shapes;
pub mod brep_lib_wire;
pub mod brep_top_adaptor;
pub mod brep_top_adaptor_domain;
pub mod face;
pub mod fast_sewing;
pub mod int_curves_face;
pub mod mat;
pub mod mat_parts;
pub mod sewing;
pub mod sewing_helpers;
pub mod shape;
pub mod shape_geometry;
pub mod shapes;
pub mod shapes_geometry;
// The reader and bit printers of the TKGeomAlgo probe, shared so that both toolkits
// print the same token stream (tools/oracle/tktopalgo_probe.cxx includes
// tkgeomalgo_probe_common.cxx likewise). Each package uses a subset of them.
#[allow(dead_code, unused_imports)]
#[path = "../../../tkgeomalgo/examples/ga_probe/common.rs"]
pub mod common;
#[allow(dead_code)]
#[path = "../../../tkg2d/examples/g2_probe/print.rs"]
pub mod print;
// The IntRes2d result printers of the TKGeomAlgo probe (tkgeomalgo_probe_intres2d.cxx
// natively); its own request operations are not in this probe's table.
#[allow(dead_code)]
#[path = "../../../tkgeomalgo/examples/ga_probe/int_res2d.rs"]
pub mod int_res2d;

// The 2D curve reader of the TKGeomAlgo probe (ga_curve2d), which the Bisector, MAT and
// MAT2d probes share with it.
#[path = "../../../tkgeomalgo/examples/ga_probe/curve2d.rs"]
pub mod curve2d;

// The IntPatch line printers of the TKGeomAlgo probe (tkgeomalgo_probe_intpatch_report.cxx
// natively) with the readers they need, for the BRepTopAdaptor domains of IntStart and
// IntPatch; their own request operations are not in this probe's table.
#[allow(dead_code)]
#[path = "../../../tkgeomalgo/examples/ga_probe/int_patch.rs"]
pub mod int_patch;
#[allow(dead_code)]
#[path = "../../../tkgeomalgo/examples/ga_probe/int_patch_report.rs"]
pub mod int_patch_report;
#[allow(dead_code)]
#[path = "../../../tkgeomalgo/examples/ga_probe/int_surf.rs"]
pub mod int_surf;
#[allow(dead_code)]
#[path = "../../../tkgeomalgo/examples/ga_probe/int_walk.rs"]
pub mod int_walk;
#[allow(dead_code)]
#[path = "../../../tkgeomalgo/examples/ga_probe/int_walk_iw.rs"]
pub mod int_walk_iw;
