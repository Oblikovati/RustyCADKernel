//! The TKV3d probe operations, one module per C++ probe file under tools/oracle.
pub mod prs3d;
pub mod v3d;

// The reader and bit printers of the TKGeomAlgo probe, shared so that every toolkit
// prints the same token stream (tools/oracle/tkv3d_probe.cxx includes
// tkgeomalgo_probe_common.cxx likewise). Each package uses a subset of them.
#[allow(dead_code, unused_imports)]
#[path = "../../../tkgeomalgo/examples/ga_probe/common.rs"]
pub mod common;
#[allow(dead_code)]
#[path = "../../../tkg2d/examples/g2_probe/print.rs"]
pub mod print;
// The primitive array printers of the TKService probe (`sv_array_state`), which the native
// probe includes from tools/oracle/tkservice_probe_array.cxx.
#[allow(dead_code)]
#[path = "../../../tkservice/examples/sv_probe/array.rs"]
pub mod array;

/// Prints the exception type of a failed call, as `GA_TRY(call)` does.
pub fn tried(result: Result<(), rustycad_tkernel::standard::Failure>) {
    print::show(result, |()| ());
}
