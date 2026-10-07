//! The TKService probe operations, one module per C++ probe file under tools/oracle.
pub mod array;
pub mod aspects;
pub mod camera;
pub mod clip_plane;
pub mod culling;
pub mod font;
pub mod grid;
pub mod hid;
pub mod light;
pub mod marker;
pub mod material;
pub mod params;
pub mod pbr;
pub mod pixel;
pub mod shader;
pub mod texture;
pub mod tpers;
// The reader and bit printers of the TKGeomAlgo probe, shared so that every toolkit
// prints the same token stream (tools/oracle/tkservice_probe.cxx includes
// tkgeomalgo_probe_common.cxx likewise). Each package uses a subset of them.
#[allow(dead_code, unused_imports)]
#[path = "../../../tkgeomalgo/examples/ga_probe/common.rs"]
pub mod common;
#[allow(dead_code)]
#[path = "../../../tkg2d/examples/g2_probe/print.rs"]
pub mod print;

/// Prints the exception type of a failed call, as `GA_TRY(call)` does.
pub fn tried(result: Result<(), rustycad_tkernel::standard::Failure>) {
    print::show(result, |()| ());
}
