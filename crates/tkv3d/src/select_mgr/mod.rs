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
