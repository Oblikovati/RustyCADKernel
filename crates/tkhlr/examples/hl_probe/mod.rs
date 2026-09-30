//! The TKHLR probe operations, one module per C++ probe file under tools/oracle.
pub mod frame;
pub mod hlr_algo;
pub mod hlr_algo_internal;
pub mod hlr_algo_poly;
pub mod hlr_algo_projector;
pub mod intrv;
pub mod top_cnx;
// The reader and bit printers of the TKGeomAlgo probe, shared so that both toolkits
// print the same token stream (tools/oracle/tkhlr_probe.cxx includes
// tkgeomalgo_probe_common.cxx likewise). Each package uses a subset of them.
#[allow(dead_code, unused_imports)]
#[path = "../../../tkgeomalgo/examples/ga_probe/common.rs"]
pub mod common;
#[allow(dead_code)]
#[path = "../../../tkg2d/examples/g2_probe/print.rs"]
pub mod print;

pub mod contap;

pub mod contana;

pub mod contap_surface;

pub mod contap_hcurve;
#[allow(dead_code)]
#[path = "../../../tkgeomalgo/examples/ga_probe/curve2d.rs"]
pub mod curve2d;

pub mod contap_hcont;
mod contap_sampling_fixture;

pub mod contap_function;

pub mod contap_arc_function;

pub mod contap_iwline;

pub mod contap_search_records;

pub mod contap_search_inside;

pub mod contap_search_boundary;

pub mod contap_walking;
