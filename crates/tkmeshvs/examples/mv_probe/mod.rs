//! The TKMeshVS probe operations, one module per C++ probe file under tools/oracle.
pub mod build;
pub mod print;
pub mod select;
pub mod source;

// The recording display driver of the TKV3d probes (tools/oracle/tkv3d_probe_sensitive_display_driver.cxx).
#[allow(dead_code)]
#[path = "../../../tkv3d/tests/support/sensitive_display_driver.rs"]
pub mod display_driver;
