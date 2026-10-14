//! Standard selection construction helpers.
mod brep_selection_tool;
pub use brep_selection_tool::StdSelectBrepSelectionTool;
/// Native legacy alias; the underlying selector retains its current implementation scope.
/// Example: `let selector = StdSelectViewerSelector3d::new()`.
pub type StdSelectViewerSelector3d = crate::select_mgr::SelectMgrViewerSelector;

mod selection_image;
pub use selection_image::StdSelectTypeOfSelectionImage;
mod brep_edge_destination;
#[cfg(test)]
mod brep_edge_output_owner;
#[cfg(test)]
mod brep_edge_output_tests;
#[cfg(test)]
mod brep_edge_pick_tests;
mod brep_edge_polygons;
mod brep_edge_sampling;
#[cfg(test)]
mod brep_edge_selection_tests;
mod brep_edge_sensitive;
#[cfg(test)]
mod brep_selection_limit_tests;
mod brep_selection_priority;

mod filters;
mod shape;
pub use filters::{
    StdSelectEdgeFilter, StdSelectFaceFilter, StdSelectShapeTypeFilter, StdSelectTypeOfEdge,
    StdSelectTypeOfFace, std_select_set_drawer_for_brep_owner,
};
mod shape_reference;
pub use shape::{
    StdSelectShape, StdSelectShapeBase, StdSelectShapeMembers, StdSelectShapeSource,
    std_select_shape_compute, std_select_shape_dump_json,
};
pub use shape_reference::{StdSelectShapeFields, StdSelectShapeRef};
mod brep_owner;
mod brep_owner_dispatch;
pub use brep_owner_dispatch::std_select_brep_owner_dump_json;
mod brep_owner_highlight;
pub use brep_owner::{
    StdSelectBRepOwner, StdSelectBRepOwnerBase, StdSelectBRepOwnerMembers, StdSelectBRepOwnerSource,
};
mod brep_face_contour;
mod brep_face_destination;
mod brep_face_plane;
mod brep_face_quadric;
mod brep_face_sensitive;
#[cfg(test)]
mod brep_face_sequence_tests;
pub(crate) mod brep_selection_load;
mod brep_sensitive_compute;
mod brep_solid_cylinder;

#[cfg(test)]
mod brep_owner_state_tests;

#[cfg(test)]
mod brep_contour_mutation_curve;
#[cfg(test)]
mod brep_contour_orientation_tests;
#[cfg(test)]
mod brep_origin_callback_tests;

#[cfg(test)]
mod brep_colored_selection_slot_tests;
#[cfg(test)]
mod brep_edge_borrow_tests;
#[cfg(test)]
mod brep_origin_slot_owner;
#[cfg(test)]
mod brep_origin_slot_tests;
#[cfg(test)]
mod brep_owner_slot_curve_callback;
#[cfg(test)]
mod brep_owner_slot_tests;
#[cfg(test)]
mod brep_selection_slot_tests;
#[cfg(test)]
mod brep_subshape_owner_slot_tests;
