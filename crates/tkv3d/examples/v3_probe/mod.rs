//! The TKV3d probe operations, one module per C++ probe file under tools/oracle.
pub mod brep_font;
pub mod dsg_prs;
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

pub mod selection_helpers;

pub mod intersector;

pub mod base_frustum;

pub mod frustum_core;

pub mod triangular;

pub mod intersector_queries;

pub mod axis;

pub mod rectangular;

pub mod frustum_set;

pub mod selecting_volume;

pub mod selection_ranking;

pub mod sensitive_entity_set;

pub mod selectable_object_set;

pub mod bvh_thread_pool;

pub mod viewer_selector_state;

#[allow(dead_code)]
#[path = "../../tests/support/boxed_selectable.rs"]
pub mod boxed_selectable;

pub mod selection_manager;

pub mod prebuild_threshold;

pub mod ais_selection;
pub mod ais_status;

pub mod ais_filters;

pub mod selector_geometry;

pub mod ais_context;

pub mod ais_object;

pub mod selector_overlap;

pub mod selector_traversal;

pub mod selector_pick;

pub mod selection_image;

pub mod sensitive_display;

pub mod ais_detection;

pub mod ais_selection_actions;

pub mod ais_load;

pub mod ais_display;

pub mod ais_lifecycle;

mod ais_lifecycle_detection;

pub mod stdprs_curve;

pub mod ais_line;

pub mod ais_point;

pub mod ais_axis;

pub mod ais_plane_trihedron;

pub mod deflection_curve;

pub mod ais_circle;

pub mod ais_animation;

pub mod animation_camera;
pub mod animation_object;
pub mod context_location;

pub mod plane_presentations;

pub mod ais_plane;

pub mod ais_point_cloud;

pub mod ais_trihedron;

pub mod ais_rubber_band;
