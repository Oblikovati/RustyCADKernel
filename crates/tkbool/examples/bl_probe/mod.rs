//! The TKBool probe operations, one module per C++ probe file under tools/oracle.
pub mod brep_algo;
pub mod brep_fill_advanced_evolved;
pub mod brep_fill_compatible;
pub mod brep_fill_draft;
pub mod brep_fill_filling;
pub mod brep_fill_generator;
pub mod brep_fill_location_law;
pub mod brep_fill_multi_line;
pub mod brep_fill_offset_wire;
pub mod brep_fill_package;
pub mod brep_fill_pipe;
pub mod brep_fill_pipe_shell;
pub mod brep_fill_section;
pub mod brep_fill_section_law;
pub mod brep_fill_sweep;
pub mod brep_proj;
pub mod curve_explorer;
pub mod interference_tool;
pub mod point_explorer;
pub mod top_ope_brep_build;
pub mod top_ope_brep_build_tools;
pub mod top_ope_brep_ds;
pub mod top_ope_brep_ds_build;
pub mod top_ope_brep_ds_ops;
pub mod top_ope_brep_tool_classifier;
pub mod top_ope_brep_tool_curve_tool;
pub mod top_ope_brep_tool_regu;
pub mod top_ope_brep_tool_shapes;
pub mod top_ope_brep_tool_tool;
pub mod top_ope_brep_tool_topology;

// The reader and bit printers of the TKGeomAlgo probe, the shape digest of the TKTopAlgo
// probe and the face frames of the TKPrim probe, shared so that every toolkit prints the
// same token stream (tools/oracle/tkbool_probe.cxx includes tkgeomalgo_probe_common.cxx,
// tktopalgo_probe_breplib_common.cxx and tkprim_probe_shape.cxx likewise).
#[allow(dead_code)]
#[path = "../../../tktopalgo/examples/tp_probe/brep_lib_common.rs"]
pub mod brep_lib_common;
#[allow(dead_code, unused_imports)]
#[path = "../../../tkgeomalgo/examples/ga_probe/common.rs"]
pub mod common;
#[allow(dead_code)]
#[path = "../../../tkg2d/examples/g2_probe/print.rs"]
pub mod print;
#[allow(dead_code)]
#[path = "../../../tkprim/examples/pr_probe/shape.rs"]
pub mod shape;

// The face and shape program readers of the TKTopAlgo probe (tktopalgo_probe_face.cxx and
// tktopalgo_probe_shapes.cxx natively), which build the sections.
#[allow(dead_code)]
#[path = "../../../tktopalgo/examples/tp_probe/face.rs"]
pub mod face;
#[allow(dead_code)]
#[path = "../../../tktopalgo/examples/tp_probe/shapes.rs"]
pub mod shapes;
#[allow(dead_code)]
#[path = "../../../tktopalgo/examples/tp_probe/shapes_geometry.rs"]
pub mod shapes_geometry;

pub mod brep_fill_evolved;

pub mod evolved_digest;
