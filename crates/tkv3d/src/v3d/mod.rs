//! The V3d package: the viewer and its views over the Graphic3d driver interfaces, the lights,
//! the grids, the clipping plane and the view trihedron.
//!
//! V3d talks to the graphic driver only through the TKService traits (v3d-design.md section 1);
//! the tests run it on the TKService null driver.
mod circular_grid;
mod computed_structure;
mod enums;
mod grid_common;
#[allow(unsafe_code)] // The public native const-char field has an anchored foreign-pointer seam.
mod light_name;
#[cfg(test)]
mod light_name_callback_fixture;
#[cfg(test)]
mod light_name_callback_tests;
mod lights;
mod package;
mod plane;
mod rectangular_grid;
mod trihedron;
mod trihedron_compute;
mod view;
#[cfg(test)]
mod view_camera_alias_driver;
#[cfg(test)]
mod view_camera_alias_fixture;
#[cfg(test)]
mod view_camera_alias_forward;
#[cfg(test)]
mod view_camera_alias_tests;
mod view_construction;
mod view_source;
#[cfg(test)]
mod view_source_failure_tests;
#[cfg(test)]
mod view_source_fixture;
#[cfg(test)]
mod view_source_list_fixture;
#[cfg(test)]
mod view_source_list_tests;
mod view_source_members;
#[cfg(test)]
mod view_source_tests;
pub use view_source_members::V3dViewMembers;
mod view_list_reference;
pub use view_list_reference::{V3dViewCursor, V3dViewListRef, V3dViewNodeRef};
mod view_source_clipping;
mod view_source_redraw;
pub use view_source::V3dViewSource;
pub use view_source_clipping::{v3d_view_add_clip_plane, v3d_view_remove_clip_plane};
pub use view_source_redraw::{v3d_view_redraw, v3d_view_redraw_immediate};
mod view_camera;
mod view_convert;
mod view_dump;
mod view_grid;
mod view_interact;
mod view_mapping;
pub(crate) use view_mapping::limited_zoom_coef;
mod view_redraw;
mod view_render;
mod view_subviews;
mod viewer;
mod viewer_activation;
mod viewer_dump;
mod viewer_grid;
mod viewer_iter;
mod viewer_lights;
mod viewer_plane;

pub use circular_grid::V3dCircularGrid;
pub use computed_structure::{V3dComputedStructure, V3dStructureComputer};
pub use enums::*;
pub use light_name::V3dLightName;
pub use lights::{
    V3dAmbientLight, V3dDirectionalLight, V3dPositionLight, V3dPositionalLight, V3dSpotLight,
};
pub use package::V3d;
pub use plane::V3dPlane;
pub use rectangular_grid::V3dRectangularGrid;
pub use trihedron::V3dTrihedron;
pub use view::V3dView;
pub use viewer::V3dViewer;

use rustycad_tkernel::standard::Failure;

/// `V3d_BadValue` (a `Standard_OutOfRange`); `v3d_bad_value("bad size 0")`.
pub fn v3d_bad_value(message: &str) -> Failure {
    Failure::with_type("V3d_BadValue", message, "")
}

/// `V3d_UnMapped` (a `Standard_DomainError`): the view has no window; `v3d_un_mapped("view has no window")`.
pub fn v3d_un_mapped(message: &str) -> Failure {
    Failure::with_type("V3d_UnMapped", message, "")
}

mod handle_chain_reference;
pub use handle_chain_reference::{V3dHandleChainCursor, V3dHandleChainNodeRef, V3dHandleChainRef};

mod light_list_reference;
pub use light_list_reference::{V3dLightCursor, V3dLightListRef, V3dLightNodeRef};

mod view_light_members;
mod viewer_light_mutation;

mod viewer_light_defaults;

#[cfg(test)]
mod viewer_light_probe_callback;
#[cfg(test)]
mod viewer_light_probe_fixture;
#[cfg(test)]
mod viewer_light_probe_replay;
#[cfg(test)]
mod viewer_light_probe_tests;

#[cfg(test)]
mod viewer_light_alias_tests;

#[cfg(test)]
mod viewer_light_edge_tests;

mod subview_sequence_reference;
#[cfg(test)]
mod viewer_light_member_tests;
pub use subview_sequence_reference::{V3dSubviewCursor, V3dSubviewNodeRef};
#[cfg(test)]
mod subview_sequence_reference_tests;

mod subview_sequence_field;
pub use subview_sequence_field::V3dSubviewSequenceRef;

#[cfg(test)]
mod subview_sequence_field_tests;

#[cfg(test)]
mod cview_bounds_camera_callback_tests;

#[cfg(test)]
mod view_clip_retarget_probe;

#[cfg(test)]
mod view_clip_callback_failure_replay;
#[cfg(test)]
mod view_clip_retarget_fixture;
#[cfg(test)]
mod view_clip_retarget_replay;
