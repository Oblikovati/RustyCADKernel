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

mod context_updates;
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
mod context_owner_walk;
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
