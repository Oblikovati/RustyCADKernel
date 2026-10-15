//! The TKOffset probe operations, one module per C++ probe file under tools/oracle.
pub mod analyse;
pub mod bitgte;
pub mod draft_angle;
pub mod inter3d;
pub mod make_offset;
pub mod make_pipe_shell;
pub mod maker;
pub mod middle_path;
pub mod offset;
pub mod thru_sections;
pub mod tool;
pub mod wrappers;

// The reader and bit printers of the TKGeomAlgo probe, the shape digest of the TKTopAlgo
// probe and the face frames of the TKPrim probe, shared so that every toolkit prints the
// same token stream (tools/oracle/tkoffset_probe.cxx includes tkgeomalgo_probe_common.cxx,
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
// The address-independent digest of BRepFill_Evolved results (TKBool probe,
// tools/oracle/tkbool_probe_evolved_digest.cxx natively).
#[allow(dead_code)]
#[path = "../../../tkbool/examples/bl_probe/evolved_digest.rs"]
pub mod evolved_digest;

// The face and shape program readers of the TKTopAlgo probe (tktopalgo_probe_face.cxx and
// tktopalgo_probe_shapes.cxx natively), which build the spines.
#[allow(dead_code)]
#[path = "../../../tktopalgo/examples/tp_probe/face.rs"]
pub mod face;
#[allow(dead_code)]
#[path = "../../../tktopalgo/examples/tp_probe/shapes.rs"]
pub mod shapes;
#[allow(dead_code)]
#[path = "../../../tktopalgo/examples/tp_probe/shapes_geometry.rs"]
pub mod shapes_geometry;
