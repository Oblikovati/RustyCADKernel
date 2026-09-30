//! The TKShHealing probe operations, one module per C++ probe file under tools/oracle.
pub mod analysis_basic;
pub mod complex_curve;
pub mod composite_surface;
pub mod explorer;
pub mod registrator;
pub mod reshape;
pub mod shape_build;
pub mod shape_extend;
pub mod wire_data;
// The reader and bit printers of the TKGeomAlgo probe, shared so that every toolkit
// prints the same token stream (tools/oracle/tkshhealing_probe.cxx includes
// tkgeomalgo_probe_common.cxx likewise). Each package uses a subset of them.
#[allow(dead_code, unused_imports)]
#[path = "../../../tkgeomalgo/examples/ga_probe/common.rs"]
pub mod common;
#[allow(dead_code)]
#[path = "../../../tkg2d/examples/g2_probe/print.rs"]
pub mod print;
// The shape programs, the face and surface readers and the shape digest of the TKTopAlgo
// probe (tktopalgo_probe_shapes.cxx, tktopalgo_probe_face.cxx and
// tktopalgo_probe_breplib_common.cxx natively), so both toolkits build and print shapes
// the same way.
#[allow(dead_code)]
#[path = "../../../tktopalgo/examples/tp_probe/brep_lib_common.rs"]
pub mod brep_lib_common;
#[allow(dead_code)]
#[path = "../../../tktopalgo/examples/tp_probe/face.rs"]
pub mod face;
#[allow(dead_code)]
#[path = "../../../tktopalgo/examples/tp_probe/shapes.rs"]
pub mod shapes;
#[allow(dead_code)]
#[path = "../../../tktopalgo/examples/tp_probe/shapes_geometry.rs"]
pub mod shapes_geometry;
