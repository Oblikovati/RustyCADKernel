//! The SelectMgr package: selection modes and owners, the selecting volumes (frustums and
//! axis), the selection manager and the viewer selector (v3d-design.md section 6).
mod entity_owner;
mod enums;
mod filter;
mod selectable_members;
mod selectable_object;
mod selection;

pub use entity_owner::{
    SelectMgrEntityOwner, SelectMgrEntityOwnerBase, SelectMgrEntityOwnerFields,
    SelectMgrEntityOwnerMembers, SelectMgrEntityOwnerSource, select_mgr_entity_owner_bind,
    select_mgr_entity_owner_dump_json, select_mgr_owner_hilight_with_color,
};
pub use enums::*;
pub use filter::{
    SelectMgrAndFilter, SelectMgrAndOrFilter, SelectMgrCompositionFilter,
    SelectMgrCompositionFilterBase, SelectMgrDisabledObjects, SelectMgrFilter, SelectMgrOrFilter,
    select_mgr_composition_acts_on,
};
pub use selectable_members::SelectMgrSelectableObjectMembers;
pub use selectable_object::{
    SelectMgrSelectableObject, SelectMgrSelectableObjectBase, SelectMgrSelectableObjectFields,
    select_mgr_reset_transformation, select_mgr_selectable_dump_json, select_mgr_selection_of,
    select_mgr_set_z_layer, select_mgr_update_clipping, select_mgr_update_transformation,
};
pub use selection::{SelectMgrSelection, SelectMgrSensitiveEntity};

mod frustum_builder;
mod view_clip_range;
pub use frustum_builder::SelectMgrFrustumBuilder;
pub use view_clip_range::SelectMgrViewClipRange;

mod base_intersector;
mod ray_intersections;
pub use base_intersector::{
    SelectMgrBaseIntersector, SelectMgrBaseIntersectorBase, SelectMgrBaseIntersectorMembers,
    select_mgr_base_intersector_dump_json,
};
pub use ray_intersections::{
    SelectMgrRayTimes, select_mgr_ray_circle_intersection, select_mgr_ray_cylinder_intersection,
    select_mgr_ray_sphere_intersection,
};

mod base_frustum;
mod boundary_sphere;
pub use base_frustum::{SelectMgrBaseFrustum, SelectMgrBaseFrustumBase};
pub use boundary_sphere::select_mgr_boundary_intersect_sphere;

mod frustum;
mod frustum_arrays;
mod frustum_circle;
mod frustum_cylinder;
mod frustum_quadric_geometry;
mod frustum_sphere;
pub use frustum::SelectMgrFrustumCore;
mod frustum_box;
mod frustum_dump;
mod frustum_linear;
mod frustum_polygon;
mod frustum_ranges;

mod triangular_frustum;
mod triangular_geometry;
mod triangular_intersector;
pub use triangular_frustum::{SelectMgrSelectionTriangle, SelectMgrTriangularFrustum};

mod axis_intersector;
mod axis_linear;
mod axis_runtime_type;
pub use axis_intersector::SelectMgrAxisIntersector;

mod axis_members;
mod axis_polygon;
mod axis_quadric;
mod axis_triangle;

mod rectangular_construction;
mod rectangular_dump;
mod rectangular_frustum;
mod rectangular_geometry;
mod rectangular_intersector;
mod rectangular_linear;
mod rectangular_polygon;
mod rectangular_quadric;
mod rectangular_triangle;
mod selecting_triangle;
pub use rectangular_frustum::{SelectMgrRectangularFrustum, SelectMgrSelectionRectangle};

mod frustum_set_boundary;
mod frustum_set_build;
mod frustum_set_circle;
mod frustum_set_copy;
mod frustum_set_cylinder;
mod frustum_set_intersector;
mod frustum_set_sphere;
mod triangular_frustum_set;
pub use triangular_frustum_set::{SelectMgrSelectionPolyline, SelectMgrTriangularFrustumSet};

mod selecting_volume;
mod selecting_volume_clipping;
mod selecting_volume_queries;
pub use selecting_volume::SelectMgrSelectingVolumeManager;

mod sort_criterion;
mod tolerance_map;
pub use sort_criterion::SelectMgrSortCriterion;
pub use tolerance_map::SelectMgrToleranceMap;

mod sensitive_entity_bvh;
mod sensitive_entity_set;
pub use sensitive_entity_set::SelectMgrSensitiveEntitySet;

mod selectable_object_set;
mod selectable_subset;
pub use selectable_object_set::SelectMgrSelectableObjectSet;
pub use selectable_subset::SelectMgrBvhSubset;
mod selectable_set_bounds;
mod selectable_set_builder;
mod selectable_set_dump;
mod selectable_set_iterator;
mod selectable_set_update;
pub use selectable_set_iterator::SelectMgrSelectableObjectSetIterator;
mod bvh_thread;
mod bvh_thread_pool;
mod bvh_thread_sentry;
pub use bvh_thread::SelectMgrBvhThread;
pub use bvh_thread_pool::{SelectMgrBvhThreadPool, SelectMgrBvhThreadPoolFields};
pub use bvh_thread_sentry::SelectMgrBvhThreadSentry;
mod viewer_selector;
mod viewer_selector_results;
pub use viewer_selector::{
    SelectMgrViewerSelector, SelectMgrViewerSelectorBase, SelectMgrViewerSelectorFields,
    SelectMgrViewerSelectorResults, SelectMgrViewerSelectorSettings, SelectMgrViewerSelectorSource,
};
pub use viewer_selector_results::{
    SelectMgrPickedCriterion, SelectMgrPickedEntity, SelectMgrViewerSelectorResultMembers,
};
mod viewer_selector_members;
mod viewer_selector_objects;
pub use viewer_selector_members::SelectMgrViewerSelectorMembers;
pub use viewer_selector_objects::SelectMgrViewerSelectorObjectMembers;

mod selection_manager;
mod selection_manager_build;
mod selection_manager_lifecycle;
mod selection_manager_update;
mod viewer_selector_activation;
mod viewer_selector_dump;
pub use selection_manager::SelectMgrSelectionManager;
mod viewer_selector_geometry;
pub use viewer_selector_geometry::{
    SelectMgrViewerSelectorGeometryMembers, select_mgr_compute_frustum, select_mgr_update_point3d,
};

mod viewer_selector_debug;

mod viewer_selector_overlap;
pub use viewer_selector_overlap::select_mgr_check_overlap;

mod viewer_selector_clipping;
mod viewer_selector_object;
mod viewer_selector_object_transform;
mod viewer_selector_traversal;
mod viewer_selector_traversal_view;
mod viewer_selector_walk;
pub use viewer_selector_object::select_mgr_traverse_object;
pub use viewer_selector_traversal::SelectMgrViewerSelectorTraversalMembers;
pub use viewer_selector_traversal_view::SelectMgrTraversalView;

pub use viewer_selector_results::select_mgr_sort_selector_results;

mod viewer_selector_pick;
pub use viewer_selector_pick::SelectMgrViewerSelectorPickMembers;

mod selection_image;
mod selection_image_colors;
mod selection_image_depth;
mod selection_image_pixels;
mod viewer_selector_image;
pub use selection_image::{
    SelectMgrSelectionImageFiller, SelectMgrSelectionImageFillerBase,
    select_mgr_create_selection_image_filler,
};
pub use viewer_selector_image::SelectMgrViewerSelectorImageMembers;

mod sensitive_presentation;
mod sensitive_presentation_entities;
mod sensitive_presentation_solids;
mod sensitive_presentation_triangles;
pub use sensitive_presentation::SelectMgr;
mod viewer_selector_display;
pub use viewer_selector_display::SelectMgrViewerSelectorDisplayMembers;

#[cfg(test)]
mod camera_argument_fixture;
#[cfg(test)]
mod camera_argument_geometry_tests;
#[cfg(test)]
mod camera_argument_tests;

mod camera_field_reference;
pub use camera_field_reference::SelectMgrCameraFieldReference;

#[cfg(test)]
mod camera_getter_fixture;
#[cfg(test)]
mod camera_getter_rows;
#[cfg(test)]
mod camera_getter_tests;
