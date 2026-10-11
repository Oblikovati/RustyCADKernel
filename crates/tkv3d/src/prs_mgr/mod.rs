//! The PrsMgr package: presentable objects, their per-mode presentations and the presentation
//! manager that displays, highlights and updates them (v3d-design.md section 5).
mod enums;
mod presentable_defaults;
mod presentable_dump;
mod presentable_members;
pub(crate) use presentable_members::prs_mgr_initialize_hilight_mode;
mod presentable_object;
mod presentable_style;
mod presentable_updates;
mod presentation;
mod presentation_immediate;
mod presentation_manager;

pub use enums::{PRS_MGR_TOP_PROJECTOR_DEPENDANT, PrsMgrDisplayStatus, PrsMgrTypeOfPresentation3d};
pub use presentable_defaults::{
    prs_mgr_add_child, prs_mgr_add_clip_plane, prs_mgr_fill, prs_mgr_remove_child,
    prs_mgr_remove_clip_plane, prs_mgr_set_mutable, prs_mgr_set_transform_persistence,
    prs_mgr_set_z_layer, prs_mgr_update_clipping, prs_mgr_update_transformation,
};
pub use presentable_dump::prs_mgr_presentable_dump_json;
pub use presentable_members::{PrsMgrPresentableObjectMembers, prs_mgr_identity_trsf};
pub use presentable_object::{
    PrsMgrPresentableObject, PrsMgrPresentableObjectBase, PrsMgrPresentableObjectFields,
    prs_mgr_presentable_bind,
};
pub use presentable_style::{
    prs_mgr_bounding_box, prs_mgr_has_polygon_offsets, prs_mgr_material, prs_mgr_polygon_offsets,
    prs_mgr_set_material, prs_mgr_set_polygon_offsets, prs_mgr_set_transparency,
    prs_mgr_unset_attributes, prs_mgr_unset_material, prs_mgr_unset_transparency,
};
pub use presentable_updates::PrsMgrPresentableObjectUpdates;
pub use presentation::PrsMgrPresentation;
pub use presentation_manager::PrsMgrPresentationManager;
