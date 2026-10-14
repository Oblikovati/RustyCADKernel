//! The AIS package: interactive objects and the interactive context.
mod animation_object;
mod base_animation_object;
mod object_animation_transform;
pub use animation_object::AisAnimationObject;
pub use base_animation_object::{
    AisBaseAnimationObject, AisBaseAnimationObjectBase, AisBaseAnimationObjectFields,
    AisBaseAnimationObjectSource,
};
pub use object_animation_transform::ais_base_animation_object_update_transform;
mod animation_camera;
pub use animation_camera::{AisAnimationCamera, AisAnimationCameraFields};
mod animation;
mod animation_members;
mod animation_playback;
pub use animation::{
    AisAnimation, AisAnimationBase, AisAnimationFields, AisAnimationProgress, AisAnimationSource,
    AisAnimationState, AisAnimationTimer,
};
pub use animation_members::{AisAnimationChildren, AisAnimationMembers};
mod filters;
pub use filters::{
    AisAttributeFilter, AisBadEdgeFilter, AisC0RegularityFilter, AisDisplayMode, AisDisplayStatus,
    AisExclusionFilter, AisGraphicTool, AisSelectionModesConcurrency, AisSignatureFilter,
    AisTypeFilter, AisTypeOfAttribute,
};
mod selection_scheme;

pub use selection_scheme::AisSelectionScheme;
mod global_status;
mod select_status;
mod selection;
mod selection_edit;
mod selection_members;
pub use global_status::AisGlobalStatus;
pub use select_status::AisSelectStatus;
pub use selection::{
    AisSelectedOwner, AisSelection, AisSelectionBase, AisSelectionFields, AisSelectionSource,
};
pub use selection_edit::{ais_selection_add_select, ais_selection_clear, ais_selection_select};
pub use selection_members::{AisSelectedObjects, AisSelectionMembers, AisSelectionValue};
mod context_styles;
mod interactive_context;
mod interactive_enums;
mod interactive_object;
pub use interactive_context::{AisInteractiveContext, AisInteractiveContextFields};
pub use interactive_enums::{AisDragAction, AisKindOfInteractive};
pub use interactive_object::{
    AisInteractiveObject, AisInteractiveObjectBase, AisInteractiveObjectFields,
};
mod context_members;
pub use context_members::AisContextHandle;
mod context_selector_reference;
pub use context_selector_reference::AisMainSelectorRef;

mod context_attributes;
mod context_defaults;
mod context_updates;
pub use context_defaults::AisEntityOwners;
mod interactive_object_members;
pub use interactive_object_members::{
    AisApplicationOwner, AisInteractiveObjectMembers, ais_interactive_object_dump_json,
};

mod context_detection_cursor;
mod context_detection_cycle;
mod context_highlight;
mod context_move_to;
mod detection_status;
pub use context_highlight::AisHighlightStyle;
pub use detection_status::AisStatusOfDetection;

mod context_highlight_owners;
mod context_isoline_policies;
mod context_owner_walk;
mod context_selected_applicative;
pub use context_isoline_policies::AisTypeOfIso;
mod context_selected_members;
mod context_selection_actions;
mod context_unhighlight_owners;
mod pick_status;
pub use pick_status::AisStatusOfPick;

mod context_global_highlight;
mod context_object_highlight;
mod context_select_pick;

mod context_select_legacy;
mod context_set_selected;
mod context_toggle_selected;

mod context_load;
mod context_location;
mod context_object_status;
mod context_view_affinity;

mod context_display;
mod context_display_selection;
mod context_erase;

mod context_bulk_display;
mod context_clear_prs;
mod context_object_queries;
mod context_object_walk;
mod context_remove;

mod line;
mod line_appearance;
mod line_presentations;
pub use line::AisLine;

mod point;
mod point_appearance;
mod point_presentations;
pub use point::AisPoint;

mod axis;
mod axis_appearance;
mod axis_geometry;
mod axis_presentations;
pub use axis::{AisAxis, AisTypeOfAxis};

mod plane_trihedron;
mod plane_trihedron_presentations;
mod plane_trihedron_selection;
pub use plane_trihedron::AisPlaneTrihedron;

mod circle;
mod circle_presentations;
pub use circle::AisCircle;

pub use animation_playback::{
    ais_animation_pause, ais_animation_start, ais_animation_start_timer, ais_animation_stop,
    ais_animation_update, ais_animation_update_timer, ais_animation_update_with_children,
};

mod text_label;
mod text_label_layout;
mod text_label_presentations;
mod text_label_style;
pub use text_label::{AisTextLabel, AisTextLabelFields};

mod plane;
mod plane_appearance;
mod plane_geometry;
mod plane_persistence;
mod plane_presentations;
mod plane_selection;
pub use plane::{AisPlane, AisPlaneFields, AisTypeOfPlane};

mod point_cloud;
mod point_cloud_appearance;
mod point_cloud_geometry;
mod point_cloud_owner;
mod point_cloud_presentations;
pub use point_cloud::{
    AisPointCloud, AisPointCloudBase, AisPointCloudDisplayMode, AisPointCloudFields,
    AisPointCloudSelectionMode, AisPointCloudSource,
};
pub use point_cloud_appearance::{
    ais_point_cloud_set_color, ais_point_cloud_set_material, ais_point_cloud_unset_color,
    ais_point_cloud_unset_material,
};
pub use point_cloud_owner::{
    AisPointCloudOwner, AisPointCloudOwnerBase, AisPointCloudOwnerFields, AisPointCloudOwnerSource,
    ais_point_cloud_owner_hilight_with_color,
};
pub use point_cloud_presentations::{ais_point_cloud_compute, ais_point_cloud_compute_selection};

mod trihedron;
mod trihedron_appearance;
mod trihedron_highlighting;
mod trihedron_owner;
mod trihedron_presentations;
mod trihedron_primitives;
mod trihedron_selection;
pub use trihedron::{
    AisTrihedron, AisTrihedronBase, AisTrihedronFields, AisTrihedronSelectionMode,
    AisTrihedronSource,
};
pub use trihedron_appearance::*;
pub use trihedron_highlighting::{
    ais_trihedron_clear_selected, ais_trihedron_dump_json, ais_trihedron_hilight_owner,
    ais_trihedron_hilight_selected,
};
pub use trihedron_owner::{
    AisTrihedronOwner, AisTrihedronOwnerBase, AisTrihedronOwnerSource, ais_trihedron_owner_hilight,
};
pub use trihedron_presentations::{ais_trihedron_compute, ais_trihedron_compute_presentation};
pub use trihedron_primitives::ais_trihedron_update_primitives;
pub use trihedron_selection::{ais_trihedron_compute_selection, ais_trihedron_sensitive};

mod rubber_band;
mod rubber_band_presentations;
mod rubber_band_triangles;
pub use rubber_band::{
    AisRubberBand, AisRubberBandBase, AisRubberBandFields, AisRubberBandSource,
    ais_rubber_band_initialize_overlay,
};
pub use rubber_band_presentations::ais_rubber_band_compute;
pub use rubber_band_triangles::ais_rubber_band_fill_triangles;

mod view_input_buffer;
mod view_input_motion;
mod view_input_types;
pub use view_input_buffer::AisViewInputBuffer;
pub use view_input_motion::{
    AisViewInputDragging, AisViewInputOrbitRotation, AisViewInputPanning, AisViewInputRotation,
    AisViewInputZRotation,
};
pub use view_input_types::{
    AisViewInputBufferType, AisViewInputHighlighting, AisViewInputOrientation,
    AisViewInputSelection, AisViewSelectionTool,
};
mod navigation_modes;
mod walk_delta;
pub use navigation_modes::{
    AisMouseGesture, AisNavigationMode, AisNavigationModeBounds, AisRotationMode,
    AisRotationModeBounds,
};
pub use walk_delta::{AisWalkDelta, AisWalkPart, AisWalkRotation, AisWalkTranslation};

mod view_controller;
mod view_controller_parameters;
mod view_controller_selection;
mod view_controller_transfer;
mod view_controller_zoom;
pub use view_controller::AisViewController;

mod view_controller_clicks;
mod view_controller_gravity;
mod view_controller_input_state;
mod view_controller_keyboard;
mod view_controller_navigation;
mod view_controller_navigation_axes;
mod view_controller_navigation_motion;
mod view_controller_navigation_transform;
mod view_controller_picking;
mod view_controller_reset;
mod view_controller_touch;

mod view_controller_mouse_buttons;
mod view_controller_mouse_gestures;
mod view_controller_mouse_motion;
mod view_controller_mouse_position;
mod view_controller_mouse_state;

mod view_controller_space_mouse;
mod view_controller_window_listener;

mod light_source;
mod light_source_properties;
mod light_source_transforms;
pub use light_source::{AisLightSourceBase, AisLightSourceSource};
pub use light_source_properties::AisLightSourceMembers;
pub use light_source_transforms::{
    ais_light_source_set_local_transform, ais_light_source_update_local_transform,
    ais_light_source_update_persistence,
};
mod camera_frustum;
mod camera_frustum_geometry;
mod camera_frustum_presentations;
mod camera_frustum_selection;
pub use camera_frustum::{AisCameraFrustum, AisCameraFrustumFields, AisCameraFrustumSelectionMode};

mod view_controller_touch_pinch;
mod view_controller_touch_rotation;
mod view_controller_touch_synthesis;

mod color_scale;
mod color_scale_placement;
mod color_scale_policies;
mod color_scale_ranges;
pub use color_scale::AisColorScale;
mod color_scale_collections;
mod color_scale_dimensions;
pub use color_scale_collections::{
    AisColorScaleColors, AisColorScaleFormat, AisColorScaleLabels, AisColorScaleTitle,
};
mod color_scale_bar;
mod color_scale_layout;
mod color_scale_presentations;
mod color_scale_uniform;

mod context_selection_bounds;
mod view_controller_fit_all;

mod light_source_aspects;
mod light_source_geometry;

mod light_source_arrows;
mod light_source_presentation;
pub use light_source_presentation::ais_light_source_compute;
mod light_source_dragging;
mod light_source_highlight;
mod light_source_object;
mod light_source_selection;
pub use light_source_dragging::ais_light_source_process_dragging;
pub use light_source_object::AisLightSource;
pub use light_source_selection::AisLightSourceOwner;

pub use light_source_aspects::ais_light_source_initialize;

mod view_controller_camera_primitives;
mod view_controller_camera_zoom;

mod view_cube_appearance;
mod view_cube_axes;
mod view_cube_classification;
mod view_cube_compute;
mod view_cube_corners;
mod view_cube_geometry;
mod view_cube_labels;
mod view_cube_presentation;
mod view_cube_rectangle;
mod view_cube_submission;
pub use view_cube_classification::{
    ais_view_cube_is_box_corner, ais_view_cube_is_box_edge, ais_view_cube_is_box_side,
};
pub use view_cube_compute::ais_view_cube_compute;
pub use view_cube_presentation::{
    AisViewCubeGeometry, AisViewCubePresentationBase, AisViewCubePresentationSettings,
    AisViewCubePresentationSource, AisViewCubeTriangleCounts,
};
pub use view_cube_rectangle::ais_view_cube_round_rectangle_triangles;

pub use view_cube_corners::ais_view_cube_create_box_corner_triangles;
pub use view_cube_geometry::{
    ais_view_cube_create_box_edge_triangles, ais_view_cube_create_box_part_triangles,
    ais_view_cube_create_box_side_triangles,
};

mod view_cube_animation;
mod view_cube_camera_fit;
mod view_cube_highlight;
mod view_cube_interaction;
mod view_cube_owner;
mod view_cube_policies;
mod view_cube_selection;
mod view_cube_sensitive;
pub use view_cube_animation::{
    ais_view_cube_handle_click, ais_view_cube_start_animation, ais_view_cube_update_animation,
    ais_view_cube_update_animation_step,
};
pub use view_cube_camera_fit::ais_view_cube_view_fit_all;
pub use view_cube_highlight::ais_view_cube_hilight_owner_with_color;
pub use view_cube_interaction::{
    AisViewCube, AisViewCubeAnimationEvents, AisViewCubeInteractionBase,
    AisViewCubeInteractionFields, AisViewCubeSource,
};
pub use view_cube_owner::AisViewCubeOwner;
pub use view_cube_policies::AisViewCubeMembers;
pub use view_cube_selection::ais_view_cube_compute_selection;
pub use view_cube_sensitive::AisViewCubeSensitive;

mod view_controller_orbit_rotation;
mod view_controller_rotation_math;
mod view_controller_rotation_state;
mod view_controller_view_rotation;

mod view_cube_animation_reference;
pub use view_cube_animation_reference::AisViewCubeAnimationRef;
mod view_controller_camera_action_state;
mod view_controller_construction;
mod view_controller_lazy_move;

mod view_controller_camera_actions;
mod view_controller_camera_anchors;
mod view_controller_camera_feedback;
mod view_controller_camera_pan;
mod view_controller_camera_rotation_actions;
mod view_controller_camera_zoom_actions;

mod view_cube_label_maps;
pub use view_cube_label_maps::{
    AisViewCubeAxesLabels, AisViewCubeLabelHash, AisViewCubeSideLabels,
};
mod view_cube_geometry_members;
pub use view_cube_geometry_members::AisViewCubeGeometryMembers;
mod view_cube_label_members;
pub use view_cube_label_members::AisViewCubeLabelMembers;
mod view_cube_font_reference;
mod view_cube_shading_reference;
pub use view_cube_font_reference::AisViewCubeFontRef;
pub use view_cube_shading_reference::AisViewCubeShadingRef;
mod view_cube_color_reference;
mod view_cube_style_members;
pub use view_cube_color_reference::AisViewCubeColorRef;
pub use view_cube_style_members::AisViewCubeStyleMembers;
mod view_cube_text_members;
pub use view_cube_appearance::ais_view_cube_default_attributes;
pub use view_cube_text_members::AisViewCubeTextMembers;
mod view_cube_reset_members;
pub use view_cube_reset_members::{
    AisViewCubeResetMembers, ais_view_cube_default_highlight_attributes,
    ais_view_cube_unset_attributes, ais_view_cube_unset_hilight_attributes,
};

mod animation_camera_source;
mod animation_camera_update;
pub use animation_camera_source::{
    AisAnimationCameraBase, AisAnimationCameraMembers, AisAnimationCameraSource,
    ais_animation_camera_update,
};

mod xr_laser_color_reference;
mod xr_tracked_device;
mod xr_tracked_device_members;
mod xr_tracked_device_object;
mod xr_tracked_device_presentation;
mod xr_tracked_device_selection;
mod xr_tracked_device_texture;
pub use xr_laser_color_reference::AisXrLaserColorRef;
pub use xr_tracked_device::{
    AisXrTrackedDeviceBase, AisXrTrackedDeviceParent, AisXrTrackedDeviceSource,
};
pub use xr_tracked_device_members::AisXrTrackedDeviceMembers;
pub use xr_tracked_device_object::AisXrTrackedDevice;
pub use xr_tracked_device_presentation::{
    ais_xr_tracked_device_compute, ais_xr_tracked_device_compute_laser_ray,
};
pub use xr_tracked_device_selection::ais_xr_tracked_device_compute_selection;

#[cfg(test)]
pub(crate) mod xr_tests;

#[cfg(test)]
include!("test_modules.rs");

include!("shape_modules.rs");

include!("colored_shape_registration.rs");

mod textured_shape_retirement;
pub use textured_shape_retirement::{
    AisTexturedShapeRetirement, ais_textured_shape_with_retirement,
};

mod context_frame_handle_references;
